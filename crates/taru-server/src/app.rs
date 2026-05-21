use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use taru_core::{Result, TaruError};
use taru_db::{DatabaseConnectOptions, TaruDatabase};

use crate::config::{TaruServerConfig, resolve_database_url};

mod addons;
mod artwork;
mod automation;
mod catalog;
mod composition;
mod job_runtime;
mod jobs;
mod library;
mod library_reconciliation;
mod managed_import;
mod metadata;
mod metadata_runtime;
mod nfo;
pub(crate) mod playback;
mod runtime;
mod staging;
mod startup;
mod storage;
pub(crate) mod user_playback;
mod webhooks;

use addons::AddonAppService;
use artwork::ManagedArtworkAppService;
pub(crate) use artwork::{ImageVariantRequest, ManagedArtworkImageBytes};
use automation::AutomationAppService;
use catalog::CatalogAppService;
use composition::{TaruAppComposition, TaruAppServices};
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
pub(crate) use runtime::RuntimeSupervisorDiagnostics;
#[cfg(test)]
use staging::cleanup_expired_staging_inputs;
use startup::ServerStartupReport;
use storage::StorageDiagnosticsAppService;
use user_playback::UserPlaybackAppService;
use webhooks::WebhookAppService;

#[cfg(test)]
use playback::plan_direct_play_with_backend;

#[derive(Clone, Debug)]
pub struct TaruApp {
    inner: Arc<TaruAppComposition>,
}

impl TaruApp {
    pub async fn new(config: TaruServerConfig) -> Result<Self> {
        let store = TaruDatabase::connect_with_options(DatabaseConnectOptions {
            backend: config.database_backend,
            url: resolve_database_url(&config)?,
            sqlite_runtime: None,
        })
        .await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: TaruServerConfig, store: TaruDatabase) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(TaruAppComposition::build(config, store).await?),
        })
    }

    #[must_use]
    pub fn config(&self) -> &TaruServerConfig {
        &self.inner.config
    }

    #[must_use]
    pub(crate) fn store(&self) -> &TaruDatabase {
        &self.inner.store
    }

    fn services(&self) -> &TaruAppServices {
        &self.inner.services
    }

    #[must_use]
    pub(crate) fn addons(&self) -> AddonAppService {
        self.services().addons.clone()
    }

    #[must_use]
    pub(crate) fn artwork(&self) -> ManagedArtworkAppService {
        self.services().artwork.clone()
    }

    #[must_use]
    pub(crate) fn automation(&self) -> AutomationAppService {
        self.services().automation.clone()
    }

    #[must_use]
    pub(crate) fn webhooks(&self) -> WebhookAppService {
        self.services().webhooks.clone()
    }

    #[must_use]
    pub(crate) fn catalog(&self) -> CatalogAppService {
        self.services().catalog.clone()
    }

    #[must_use]
    pub(crate) fn library(&self) -> LibraryAppService {
        self.services().library.clone()
    }

    #[must_use]
    pub(crate) fn storage(&self) -> StorageDiagnosticsAppService {
        self.services().storage.clone()
    }

    #[must_use]
    pub(crate) fn jobs(&self) -> JobAppService {
        self.services().jobs.clone()
    }

    #[must_use]
    pub(crate) fn library_scan(&self) -> LibraryScanAppService {
        self.services().library_scan.clone()
    }

    #[must_use]
    pub(crate) fn nfo(&self) -> NfoAppService {
        self.services().nfo.clone()
    }

    #[must_use]
    pub(crate) fn metadata(&self) -> MetadataAppService {
        self.services().metadata.clone()
    }

    #[must_use]
    pub(crate) fn playback(&self) -> PlaybackAppService {
        self.services().playback.clone()
    }

    #[must_use]
    pub(crate) fn user_playback(&self) -> UserPlaybackAppService {
        self.services().user_playback.clone()
    }

    pub(crate) fn runtime_diagnostics(&self) -> RuntimeSupervisorDiagnostics {
        self.inner.runtime.diagnostics()
    }

    pub(crate) fn startup_report(&self) -> &ServerStartupReport {
        &self.inner.startup_report
    }

    pub(crate) fn shutdown_runtime(&self) {
        self.inner.shutdown_runtime();
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
