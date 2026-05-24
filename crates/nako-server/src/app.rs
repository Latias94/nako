use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use nako_core::{NakoError, Result};
use nako_db::{
    DatabaseBackendCapabilities, DatabaseBackendKind, DatabaseConnectOptions, NakoDatabase,
};

use crate::config::{NakoServerConfig, resolve_database_url};

pub(crate) mod acquisition_intake;
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

use acquisition_intake::AcquisitionIntakeAppService;
use addons::AddonAppService;
#[cfg(test)]
pub(crate) use addons::set_test_outbound_task_dispatch_secret;
use artwork::ManagedArtworkAppService;
pub(crate) use artwork::{ImageVariantRequest, ManagedArtworkImageBytes};
use automation::AutomationAppService;
use catalog::CatalogAppService;
use composition::{NakoAppComposition, NakoAppServices};
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
pub struct NakoApp {
    inner: Arc<NakoAppComposition>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DatabaseDiagnostics {
    pub(crate) backend_kind: DatabaseBackendKind,
    pub(crate) capabilities: DatabaseBackendCapabilities,
}

impl NakoApp {
    pub async fn new(config: NakoServerConfig) -> Result<Self> {
        let store = NakoDatabase::connect_with_options(DatabaseConnectOptions {
            backend: config.database_backend,
            url: resolve_database_url(&config)?,
            sqlite_runtime: None,
        })
        .await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: NakoServerConfig, store: NakoDatabase) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(NakoAppComposition::build(config, store).await?),
        })
    }

    #[must_use]
    pub fn config(&self) -> &NakoServerConfig {
        &self.inner.config
    }

    #[must_use]
    pub(crate) fn database_diagnostics(&self) -> DatabaseDiagnostics {
        DatabaseDiagnostics {
            backend_kind: self.inner.store.backend_kind(),
            capabilities: self.inner.store.capabilities(),
        }
    }

    fn services(&self) -> &NakoAppServices {
        &self.inner.services
    }

    #[must_use]
    pub(crate) fn addons(&self) -> AddonAppService {
        self.services().addons.clone()
    }

    #[must_use]
    pub(crate) fn acquisition_intake(&self) -> AcquisitionIntakeAppService {
        self.services().acquisition_intake.clone()
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
    pub(crate) fn managed_import(&self) -> managed_import::ManagedImportAppService {
        self.services().managed_import.clone()
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
        .map_err(|err| NakoError::InvalidInput {
            message: format!("system time is before UNIX epoch: {err}"),
        })?;

    i64::try_from(duration.as_millis()).map_err(|err| NakoError::InvalidInput {
        message: format!("current timestamp does not fit i64 milliseconds: {err}"),
    })
}

#[cfg(test)]
mod tests;
