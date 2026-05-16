use std::{
    collections::HashSet,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use taru_core::{
    LibraryRepository, Result, TaruError, TransactionManager, TranscodeFailureCategory,
    TranscodeSessionRepository,
};
use taru_db::SqliteStore;
use tokio::sync::Semaphore;
use tracing::warn;

use crate::config::{TaruServerConfig, libraries_from_config};

mod addons;
mod automation;
mod catalog;
mod jobs;
mod library;
mod metadata;
mod metadata_runtime;
mod nfo;
pub(crate) mod playback;
mod runtime;
mod staging;
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
use runtime::{RuntimeSupervisor, RuntimeSupervisorDiagnostics};
use staging::cleanup_expired_staging_inputs;
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
    store: SqliteStore,
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
        store.migrate().await?;
        let recovered_sessions = store
            .fail_stale_transcode_sessions(
                TranscodeFailureCategory::Stale,
                "session was active during server startup".to_owned(),
            )
            .await?;
        if recovered_sessions > 0 {
            warn!(
                recovered_sessions,
                "marked stale transcode sessions failed during startup"
            );
        }
        if config.staging.cleanup_on_startup {
            let cleanup = cleanup_expired_staging_inputs(&store, current_time_ms()?).await?;
            if cleanup.deleted_records > 0 || cleanup.deleted_files > 0 {
                warn!(
                    deleted_records = cleanup.deleted_records,
                    deleted_files = cleanup.deleted_files,
                    "cleaned expired staged inputs during startup"
                );
            }
        }

        let webhook_permits = Arc::new(Semaphore::new(config.webhook_concurrency.max(1)));
        let storage_backends = StorageBackendRegistry::new(&config, store.clone());
        let runtime = RuntimeSupervisor::new();
        let scan_permits = Arc::new(Semaphore::new(config.scan_concurrency.max(1)));
        let metadata_permits = Arc::new(Semaphore::new(config.metadata_concurrency.max(1)));
        let metadata_providers = metadata_runtime::build_metadata_provider_registry(&config)?;
        let app = Self {
            inner: Arc::new(TaruAppInner {
                runtime: runtime.clone(),
                jobs: JobAppService::new(store.clone()),
                library_scan: LibraryScanAppService::new(
                    config.clone(),
                    store.clone(),
                    scan_permits,
                    storage_backends.clone(),
                    runtime.clone(),
                ),
                addons: AddonAppService::new(store.clone()),
                automation: AutomationAppService::new(store.clone()),
                webhooks: WebhookAppService::new(store.clone(), webhook_permits),
                catalog: CatalogAppService::new(store.clone()),
                library: LibraryAppService::new(store.clone()),
                storage: StorageDiagnosticsAppService::new(storage_backends.clone()),
                metadata: MetadataAppService::new(
                    config.clone(),
                    store.clone(),
                    metadata_permits.clone(),
                    metadata_providers,
                    runtime.clone(),
                ),
                nfo: NfoAppService::new(
                    config.clone(),
                    store.clone(),
                    metadata_permits,
                    storage_backends.clone(),
                    runtime.clone(),
                ),
                playback: PlaybackAppService::new(
                    config.clone(),
                    store.clone(),
                    storage_backends,
                    runtime,
                )?,
                config,
                store,
            }),
        };

        app.ensure_configured_libraries().await?;
        app.metadata()
            .cleanup_metadata_raw_cache_on_startup()
            .await?;
        app.metadata().start_metadata_lifecycle_tasks();
        Ok(app)
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

    pub(crate) fn shutdown_runtime(&self) {
        self.inner.runtime.shutdown();
    }

    async fn ensure_configured_libraries(&self) -> Result<()> {
        let libraries = libraries_from_config(self.config());
        if libraries.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "server config must include at least one library".to_owned(),
            });
        }

        let mut seen = HashSet::new();
        for library in &libraries {
            if !seen.insert(library.id) {
                return Err(TaruError::InvalidInput {
                    message: format!("duplicate configured library id: {}", library.id),
                });
            }
        }

        for library in libraries {
            self.inner.store.upsert_library(&library).await?;
        }
        Ok(())
    }
}

fn current_time_ms() -> Result<i64> {
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
