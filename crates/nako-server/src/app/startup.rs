use std::collections::HashSet;

use nako_core::{
    IdentityAccessRepository, JobRepository, ManagedArtworkRepository, NakoError, Result,
    RoleAssignment, TranscodeFailureCategory, TranscodeSessionRepository, UserPrincipalId,
    UserRole, bootstrap_admin_user,
};
use nako_db::NakoDatabase;
use nako_vfs::StorageUri;
use tracing::warn;

use super::{
    addons::AddonAppService,
    artwork::ManagedArtworkAppService,
    current_time_ms,
    library_reconciliation::{
        ConfiguredLibraryReconciliationReport, ConfiguredLibraryReconciliationService,
    },
    metadata::MetadataAppService,
    playback_artifact_cleanup::cleanup_expired_playback_artifacts,
    runtime::RuntimeSupervisor,
    staging::cleanup_expired_staging_inputs,
    watch_folder_runtime::WatchFolderRuntimeAppService,
    watch_folder_runtime::WatchFolderRuntimeCoverageReport,
};
use crate::config::{LocalLibraryConfig, NakoServerConfig, libraries_from_config};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerStartupReport {
    pub database_migrated: bool,
    pub configured_libraries: usize,
    pub library_reconciliation: ConfiguredLibraryReconciliationReport,
    pub recovered_transcode_sessions: u64,
    pub recovered_jobs: u64,
    pub staging_cleanup: Option<ServerStartupStagingCleanupReport>,
    pub playback_artifact_cleanup: Option<ServerStartupPlaybackArtifactCleanupReport>,
    pub metadata_raw_cache_deleted: u64,
    pub metadata_lifecycle_tasks_started: usize,
    pub artwork_ingest_worker_started: bool,
    pub addon_event_scheduler_started: bool,
    pub watch_folder_runtimes_started: usize,
    pub watch_folder_runtime_coverage: WatchFolderRuntimeCoverageReport,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerStartupStagingCleanupReport {
    pub deleted_records: usize,
    pub deleted_files: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ServerStartupPlaybackArtifactCleanupReport {
    pub examined_artifacts: u32,
    pub deleted_artifacts: u32,
    pub deleted_files: u32,
    pub deleted_directories: u32,
    pub deleted_bytes: u64,
    pub skipped_security: u32,
}

#[derive(Debug)]
pub(crate) struct ServerStartupWorkflow<'a> {
    config: &'a NakoServerConfig,
    store: &'a NakoDatabase,
    metadata: MetadataAppService,
    startup_runtime: ServerStartupRuntime<'a>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ServerStartupRuntime<'a> {
    artwork: &'a ManagedArtworkAppService,
    addons: &'a AddonAppService,
    watch_folder_runtime: &'a WatchFolderRuntimeAppService,
    runtime: &'a RuntimeSupervisor,
}

impl<'a> ServerStartupRuntime<'a> {
    pub(crate) fn new(
        artwork: &'a ManagedArtworkAppService,
        addons: &'a AddonAppService,
        watch_folder_runtime: &'a WatchFolderRuntimeAppService,
        runtime: &'a RuntimeSupervisor,
    ) -> Self {
        Self {
            artwork,
            addons,
            watch_folder_runtime,
            runtime,
        }
    }
}

impl<'a> ServerStartupWorkflow<'a> {
    pub(crate) fn new(
        config: &'a NakoServerConfig,
        store: &'a NakoDatabase,
        metadata: MetadataAppService,
        startup_runtime: ServerStartupRuntime<'a>,
    ) -> Self {
        Self {
            config,
            store,
            metadata,
            startup_runtime,
        }
    }

    pub(crate) async fn run(&self) -> Result<ServerStartupReport> {
        let mut report = self.build_initial_report().await?;
        self.start_control_plane_runtimes(&mut report).await?;
        Ok(report)
    }

    async fn build_initial_report(&self) -> Result<ServerStartupReport> {
        let recovered_transcode_sessions = self.recover_stale_transcode_sessions().await?;
        let recovered_jobs = self.recover_unfinished_jobs().await?;
        let staging_cleanup = self.cleanup_staging_inputs().await?;
        let playback_artifact_cleanup = self.cleanup_playback_artifacts().await?;
        let library_reconciliation = self.reconcile_configured_libraries().await?;
        let configured_libraries = library_reconciliation.configured_libraries;
        self.ensure_bootstrap_admin_user().await?;
        let metadata_raw_cache_deleted = self
            .metadata
            .cleanup_metadata_raw_cache_on_startup()
            .await?;
        let metadata_lifecycle_tasks_started = self.metadata.start_metadata_lifecycle_tasks();

        Ok(ServerStartupReport {
            database_migrated: true,
            configured_libraries,
            library_reconciliation,
            recovered_transcode_sessions,
            recovered_jobs,
            staging_cleanup,
            playback_artifact_cleanup,
            metadata_raw_cache_deleted,
            metadata_lifecycle_tasks_started,
            artwork_ingest_worker_started: false,
            addon_event_scheduler_started: false,
            watch_folder_runtimes_started: 0,
            watch_folder_runtime_coverage: WatchFolderRuntimeCoverageReport::default(),
        })
    }

    async fn start_control_plane_runtimes(&self, report: &mut ServerStartupReport) -> Result<()> {
        report.artwork_ingest_worker_started = if self.config.artwork.ingest_worker_enabled {
            self.startup_runtime
                .artwork
                .start_ingest_worker(self.startup_runtime.runtime)
        } else {
            false
        };
        report.addon_event_scheduler_started = self
            .startup_runtime
            .addons
            .start_addon_event_scheduler(self.config.addon_event_scheduler);
        report.watch_folder_runtime_coverage = self
            .startup_runtime
            .watch_folder_runtime
            .start_enabled_watchers(self.startup_runtime.runtime)
            .await?;
        report.watch_folder_runtimes_started =
            report.watch_folder_runtime_coverage.started_libraries();

        Ok(())
    }

    async fn ensure_bootstrap_admin_user(&self) -> Result<()> {
        let now_ms = current_time_ms()?;
        let user = match self
            .store
            .get_user_by_principal(&UserPrincipalId::local_admin())
            .await?
        {
            Some(user) => user,
            None => {
                let user = bootstrap_admin_user(now_ms);
                self.store.upsert_user(&user).await?;
                user
            }
        };

        let mut roles = self.store.list_role_assignments(user.id).await?;
        if !roles
            .iter()
            .any(|assignment| assignment.role == UserRole::Administrator)
        {
            roles.push(RoleAssignment {
                user_id: user.id,
                role: UserRole::Administrator,
                granted_at_ms: now_ms,
            });
            self.store.replace_role_assignments(user.id, &roles).await?;
        }

        Ok(())
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
        let recovered_managed_artwork = self
            .store
            .fail_unfinished_managed_artwork_ingests(
                "startup_recovery".to_owned(),
                "managed artwork ingest was unfinished during server startup".to_owned(),
                Some(
                    serde_json::json!({
                        "status": "failed",
                        "failure_code": "startup_recovery"
                    })
                    .to_string(),
                ),
            )
            .await?;
        if recovered_managed_artwork > 0 {
            warn!(
                recovered_managed_artwork,
                "marked unfinished managed artwork ingests failed during startup"
            );
        }

        let recovered_other_jobs = self
            .store
            .fail_unfinished_jobs("job was unfinished during server startup".to_owned())
            .await?;
        let recovered_jobs = recovered_managed_artwork + recovered_other_jobs;
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

    async fn cleanup_playback_artifacts(
        &self,
    ) -> Result<Option<ServerStartupPlaybackArtifactCleanupReport>> {
        if !self.config.playback.transcode_artifact_cleanup_on_startup {
            return Ok(None);
        }

        let cleanup = cleanup_expired_playback_artifacts(
            self.store,
            &self.config.remux_staging_root,
            self.config.playback.transcode_artifact_retention_ms,
            current_time_ms()?,
        )
        .await?;
        if cleanup.deleted_artifacts > 0 {
            warn!(
                deleted_artifacts = cleanup.deleted_artifacts,
                deleted_files = cleanup.deleted_files,
                deleted_directories = cleanup.deleted_directories,
                "cleaned expired playback transcode artifacts during startup"
            );
        }

        Ok(Some(cleanup))
    }

    async fn reconcile_configured_libraries(
        &self,
    ) -> Result<ConfiguredLibraryReconciliationReport> {
        validate_configured_library_roots(&self.config.libraries)?;
        let libraries = libraries_from_config(self.config);
        ConfiguredLibraryReconciliationService::new(self.store)
            .reconcile(libraries)
            .await
    }
}

fn validate_configured_library_roots(libraries: &[LocalLibraryConfig]) -> Result<()> {
    let mut seen_roots = HashSet::new();

    for library in libraries {
        let (root_key, root_display) = configured_library_backend_root(library)?;
        if !seen_roots.insert(root_key) {
            return Err(NakoError::InvalidInput {
                message: format!("duplicate configured library root: {root_display}"),
            });
        }
    }

    Ok(())
}

fn configured_library_backend_root(library: &LocalLibraryConfig) -> Result<(String, String)> {
    let Some(webdav) = library.webdav.as_ref() else {
        let root = library.root.display().to_string();
        return Ok((format!("local:{root}"), root));
    };
    let root = StorageUri::parse(&webdav.root)?;
    if root.scheme() != "webdav" {
        return Err(NakoError::InvalidInput {
            message: format!(
                "configured WebDAV library root must use webdav scheme: {}",
                webdav.root
            ),
        });
    }

    let endpoint = webdav.base_url.trim_end_matches('/');
    Ok((
        format!("webdav:{endpoint}:{}", root.as_str()),
        root.as_str().to_owned(),
    ))
}
