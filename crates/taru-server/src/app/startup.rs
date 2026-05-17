use std::collections::HashSet;

use taru_core::{
    LibraryRepository, Result, TaruError, TransactionManager, TranscodeFailureCategory,
    TranscodeSessionRepository,
};
use taru_db::SqliteStore;
use tracing::warn;

use super::{
    current_time_ms, metadata::MetadataAppService, staging::cleanup_expired_staging_inputs,
};
use crate::config::{TaruServerConfig, libraries_from_config};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerStartupReport {
    pub configured_libraries: usize,
    pub recovered_transcode_sessions: u64,
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
        let staging_cleanup = self.cleanup_staging_inputs().await?;
        let configured_libraries = self.ensure_configured_libraries().await?;
        let metadata_raw_cache_deleted = self
            .metadata
            .cleanup_metadata_raw_cache_on_startup()
            .await?;
        let metadata_lifecycle_tasks_started = self.metadata.start_metadata_lifecycle_tasks();

        Ok(ServerStartupReport {
            configured_libraries,
            recovered_transcode_sessions,
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

    async fn ensure_configured_libraries(&self) -> Result<usize> {
        let libraries = libraries_from_config(self.config);
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

        let count = libraries.len();
        for library in libraries {
            self.store.upsert_library(&library).await?;
        }

        Ok(count)
    }
}
