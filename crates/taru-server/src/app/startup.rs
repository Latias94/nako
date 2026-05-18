use taru_core::{
    JobRepository, Result, TransactionManager, TranscodeFailureCategory, TranscodeSessionRepository,
};
use taru_db::SqliteStore;
use tracing::warn;

use super::{
    current_time_ms,
    library_reconciliation::{
        ConfiguredLibraryReconciliationReport, ConfiguredLibraryReconciliationService,
    },
    metadata::MetadataAppService,
    staging::cleanup_expired_staging_inputs,
};
use crate::config::{TaruServerConfig, libraries_from_config};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerStartupReport {
    pub configured_libraries: usize,
    pub library_reconciliation: ConfiguredLibraryReconciliationReport,
    pub recovered_transcode_sessions: u64,
    pub recovered_jobs: u64,
    pub staging_cleanup: Option<ServerStartupStagingCleanupReport>,
    pub metadata_raw_cache_deleted: u64,
    pub metadata_lifecycle_tasks_started: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerStartupStagingCleanupReport {
    pub deleted_records: usize,
    pub deleted_files: usize,
}

#[derive(Debug)]
pub(crate) struct ServerStartupWorkflow<'a> {
    config: &'a TaruServerConfig,
    store: &'a SqliteStore,
    metadata: MetadataAppService,
}

impl<'a> ServerStartupWorkflow<'a> {
    pub(crate) fn new(
        config: &'a TaruServerConfig,
        store: &'a SqliteStore,
        metadata: MetadataAppService,
    ) -> Self {
        Self {
            config,
            store,
            metadata,
        }
    }

    pub(crate) async fn run(&self) -> Result<ServerStartupReport> {
        self.store.migrate().await?;

        let recovered_transcode_sessions = self.recover_stale_transcode_sessions().await?;
        let recovered_jobs = self.recover_unfinished_jobs().await?;
        let staging_cleanup = self.cleanup_staging_inputs().await?;
        let library_reconciliation = self.reconcile_configured_libraries().await?;
        let configured_libraries = library_reconciliation.configured_libraries;
        let metadata_raw_cache_deleted = self
            .metadata
            .cleanup_metadata_raw_cache_on_startup()
            .await?;
        let metadata_lifecycle_tasks_started = self.metadata.start_metadata_lifecycle_tasks();

        Ok(ServerStartupReport {
            configured_libraries,
            library_reconciliation,
            recovered_transcode_sessions,
            recovered_jobs,
            staging_cleanup,
            metadata_raw_cache_deleted,
            metadata_lifecycle_tasks_started,
        })
    }

    async fn recover_stale_transcode_sessions(&self) -> Result<u64> {
        let recovered_sessions = self
            .store
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

        Ok(recovered_sessions)
    }

    async fn recover_unfinished_jobs(&self) -> Result<u64> {
        let recovered_jobs = self
            .store
            .fail_unfinished_jobs("job was unfinished during server startup".to_owned())
            .await?;
        if recovered_jobs > 0 {
            warn!(
                recovered_jobs,
                "marked unfinished durable jobs failed during startup"
            );
        }

        Ok(recovered_jobs)
    }

    async fn cleanup_staging_inputs(&self) -> Result<Option<ServerStartupStagingCleanupReport>> {
        if !self.config.staging.cleanup_on_startup {
            return Ok(None);
        }

        let cleanup = cleanup_expired_staging_inputs(self.store, current_time_ms()?).await?;
        if cleanup.deleted_records > 0 || cleanup.deleted_files > 0 {
            warn!(
                deleted_records = cleanup.deleted_records,
                deleted_files = cleanup.deleted_files,
                "cleaned expired staged inputs during startup"
            );
        }

        Ok(Some(ServerStartupStagingCleanupReport {
            deleted_records: cleanup.deleted_records,
            deleted_files: cleanup.deleted_files,
        }))
    }

    async fn reconcile_configured_libraries(
        &self,
    ) -> Result<ConfiguredLibraryReconciliationReport> {
        let libraries = libraries_from_config(self.config);
        ConfiguredLibraryReconciliationService::new(self.store)
            .reconcile(libraries)
            .await
    }
}
