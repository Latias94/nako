use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use taru_core::{Result, TaruError, TransactionManager};
use taru_db::SqliteStore;
use tokio::sync::Semaphore;

use crate::config::TaruServerConfig;

mod addons;
mod automation;
mod catalog;
mod job_runtime;
mod jobs;
mod library;
mod metadata;
mod metadata_runtime;
mod nfo;
pub(crate) mod playback;
mod runtime;
mod staging;
mod startup;
mod storage;
mod webhooks;

use addons::AddonAppService;
use automation::AutomationAppService;
use catalog::CatalogAppService;
use jobs::{JobAppService, LibraryScanAppService};
use library::LibraryAppService;
use metadata::MetadataAppService;
use nfo::NfoAppService;
#[cfg(test)]
pub(crate) use playback::DirectPlayStreamBody;
use playback::PlaybackAppService;
pub(crate) use playback::{
    DirectPlaySourceBody, HlsSourceRequest, RemuxSourceDisposition, RemuxSourceRequest,
};
use runtime::RuntimeSupervisor;
pub(crate) use runtime::RuntimeSupervisorDiagnostics;
#[cfg(test)]
use staging::cleanup_expired_staging_inputs;
use startup::{ServerStartupReport, ServerStartupWorkflow};
use storage::{StorageBackendRegistry, StorageDiagnosticsAppService};
use webhooks::WebhookAppService;

#[cfg(test)]
use playback::plan_direct_play_with_backend;

#[derive(Clone, Debug)]
pub struct TaruApp {
    inner: Arc<TaruAppInner>,
}

#[derive(Debug)]
struct TaruAppInner {
    config: TaruServerConfig,
    runtime: RuntimeSupervisor,
    jobs: JobAppService,
    library_scan: LibraryScanAppService,
    addons: AddonAppService,
    automation: AutomationAppService,
    webhooks: WebhookAppService,
    catalog: CatalogAppService,
    library: LibraryAppService,
    storage: StorageDiagnosticsAppService,
    metadata: MetadataAppService,
    nfo: NfoAppService,
    playback: PlaybackAppService,
    startup_report: ServerStartupReport,
}

impl Drop for TaruAppInner {
    fn drop(&mut self) {
        self.runtime.shutdown();
    }
}

impl TaruApp {
    pub async fn new(config: TaruServerConfig) -> Result<Self> {
        let store = SqliteStore::connect(&config.database_url).await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: TaruServerConfig, store: SqliteStore) -> Result<Self> {
        let webhook_permits = Arc::new(Semaphore::new(config.webhook_concurrency.max(1)));
        let storage_backends = StorageBackendRegistry::new(&config, store.clone());
        let runtime = RuntimeSupervisor::new();
        let scan_permits = Arc::new(Semaphore::new(config.scan_concurrency.max(1)));
        let metadata_permits = Arc::new(Semaphore::new(config.metadata_concurrency.max(1)));
        let metadata_providers = metadata_runtime::build_metadata_provider_registry(&config)?;
        let jobs = JobAppService::new(store.clone());
        let library_scan = LibraryScanAppService::new(
            config.clone(),
            store.clone(),
            scan_permits,
            storage_backends.clone(),
            runtime.clone(),
        );
        let addons = AddonAppService::new(store.clone());
        let automation = AutomationAppService::new(store.clone());
        let webhooks = WebhookAppService::new(store.clone(), webhook_permits);
        let catalog = CatalogAppService::new(store.clone());
        let library = LibraryAppService::new(store.clone());
        let storage = StorageDiagnosticsAppService::new(storage_backends.clone());
        let metadata = MetadataAppService::new(
            config.clone(),
            store.clone(),
            metadata_permits.clone(),
            metadata_providers,
            runtime.clone(),
        );
        let nfo = NfoAppService::new(
            config.clone(),
            store.clone(),
            metadata_permits,
            storage_backends.clone(),
            runtime.clone(),
        );
        let playback = PlaybackAppService::new(
            config.clone(),
            store.clone(),
            storage_backends,
            runtime.clone(),
        )?;

        let startup_report = ServerStartupWorkflow::new(&config, &store, metadata.clone())
            .run()
            .await?;

        Ok(Self {
            inner: Arc::new(TaruAppInner {
                runtime: runtime.clone(),
                jobs,
                library_scan,
                addons,
                automation,
                webhooks,
                catalog,
                library,
                storage,
                metadata,
                nfo,
                playback,
                startup_report,
                config,
            }),
        })
    }

    #[must_use]
    pub fn config(&self) -> &TaruServerConfig {
        &self.inner.config
    }

    #[must_use]
    pub(crate) fn addons(&self) -> AddonAppService {
        self.inner.addons.clone()
    }

    #[must_use]
    pub(crate) fn automation(&self) -> AutomationAppService {
        self.inner.automation.clone()
    }

    #[must_use]
    pub(crate) fn webhooks(&self) -> WebhookAppService {
        self.inner.webhooks.clone()
    }

    #[must_use]
    pub(crate) fn catalog(&self) -> CatalogAppService {
        self.inner.catalog.clone()
    }

    #[must_use]
    pub(crate) fn library(&self) -> LibraryAppService {
        self.inner.library.clone()
    }

    #[must_use]
    pub(crate) fn storage(&self) -> StorageDiagnosticsAppService {
        self.inner.storage.clone()
    }

    #[must_use]
    pub(crate) fn jobs(&self) -> JobAppService {
        self.inner.jobs.clone()
    }

    #[must_use]
    pub(crate) fn library_scan(&self) -> LibraryScanAppService {
        self.inner.library_scan.clone()
    }

    #[must_use]
    pub(crate) fn nfo(&self) -> NfoAppService {
        self.inner.nfo.clone()
    }

    #[must_use]
    pub(crate) fn metadata(&self) -> MetadataAppService {
        self.inner.metadata.clone()
    }

    #[must_use]
    pub(crate) fn playback(&self) -> PlaybackAppService {
        self.inner.playback.clone()
    }

    pub(crate) fn runtime_diagnostics(&self) -> RuntimeSupervisorDiagnostics {
        self.inner.runtime.diagnostics()
    }

    pub(crate) fn startup_report(&self) -> &ServerStartupReport {
        &self.inner.startup_report
    }

    pub(crate) fn shutdown_runtime(&self) {
        self.inner.runtime.shutdown();
    }
}

pub(crate) fn current_time_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| TaruError::InvalidInput {
            message: format!("system time is before UNIX epoch: {err}"),
        })?;

    i64::try_from(duration.as_millis()).map_err(|err| TaruError::InvalidInput {
        message: format!("current timestamp does not fit i64 milliseconds: {err}"),
    })
}

#[cfg(test)]
mod tests;
