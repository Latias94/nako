use std::time::Duration;

use nako_core::{JobId, Library, LibraryId, LibraryRepository, NakoError, PageRequest, Result};
use nako_db::NakoDatabase;
use nako_library::{WatchFolderIntakePlan, WatchFolderIntakePlanInput, plan_watch_folder_intake};
use nako_vfs::StorageUri;
use tracing::{info, warn};

use super::{
    acquisition_intake::{
        AcquisitionIntakeAppService, DiscoverWatchFolderCandidatesRequest,
        WatchFolderDiscoveryFailureDiagnostic,
    },
    jobs::LibraryScanAppService,
    runtime::RuntimeSupervisor,
};

const WATCH_FOLDER_RUNTIME_INTERVAL_MS: u64 = 5_000;
const WATCH_FOLDER_RUNTIME_ERROR_BACKOFF_MS: u64 = 15_000;

#[derive(Clone, Debug)]
pub(crate) struct WatchFolderRuntimeAppService {
    store: NakoDatabase,
    acquisition_intake: AcquisitionIntakeAppService,
    library_scan: LibraryScanAppService,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WatchFolderRuntimeTickDiagnostic {
    pub(crate) library_id: LibraryId,
    pub(crate) monitored: bool,
    pub(crate) intake_plan: WatchFolderIntakePlan,
    pub(crate) discovery_failures: Vec<WatchFolderRuntimeFailureDiagnostic>,
    pub(crate) scan_job_id: Option<JobId>,
    pub(crate) reused_existing_scan: bool,
    pub(crate) backoff_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WatchFolderRuntimeFailureDiagnostic {
    pub(crate) uri_redacted: String,
    pub(crate) safe_message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WatchFolderRuntimeCoverageStatus {
    Started,
    Disabled,
    UnsupportedRoot,
    MissingRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WatchFolderRuntimeCoverageDiagnostic {
    pub(crate) library_id: LibraryId,
    pub(crate) library_name: String,
    pub(crate) root_scheme: Option<String>,
    pub(crate) root_ref_redacted: String,
    pub(crate) status: WatchFolderRuntimeCoverageStatus,
    pub(crate) safe_reason: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WatchFolderRuntimeCoverageReport {
    pub(crate) diagnostics: Vec<WatchFolderRuntimeCoverageDiagnostic>,
}

impl WatchFolderRuntimeCoverageReport {
    pub(crate) fn started_libraries(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.status == WatchFolderRuntimeCoverageStatus::Started)
            .count()
    }

    pub(crate) fn realtime_enabled_libraries(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.status != WatchFolderRuntimeCoverageStatus::Disabled)
            .count()
    }

    pub(crate) fn skipped_libraries(&self) -> usize {
        self.diagnostics
            .len()
            .saturating_sub(self.started_libraries())
    }
}

impl WatchFolderRuntimeAppService {
    pub(crate) fn new(
        store: NakoDatabase,
        acquisition_intake: AcquisitionIntakeAppService,
        library_scan: LibraryScanAppService,
    ) -> Self {
        Self {
            store,
            acquisition_intake,
            library_scan,
        }
    }

    pub(super) async fn start_enabled_watchers(
        &self,
        runtime: &RuntimeSupervisor,
    ) -> Result<WatchFolderRuntimeCoverageReport> {
        let coverage = self.runtime_coverage_report().await?;

        for diagnostic in coverage
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.status == WatchFolderRuntimeCoverageStatus::Started)
        {
            let service = self.clone();
            let library_id = diagnostic.library_id;
            let shutdown = runtime.shutdown_token();
            runtime.spawn(
                "watch_folder_runtime",
                "disk.scan.watch_folder",
                async move {
                    loop {
                        let delay = match service.tick_library(library_id).await {
                            Ok(diagnostic) => {
                                if let Some(job_id) = diagnostic.scan_job_id {
                                info!(
                                    library_id = %diagnostic.library_id,
                                    job_id = %job_id,
                                    newly_ready_candidates = diagnostic.intake_plan.summary.newly_ready_candidates,
                                    suppressed_candidates = diagnostic.intake_plan.summary.suppressed_candidates,
                                    reused_existing_scan = diagnostic.reused_existing_scan,
                                    "watch-folder runtime admitted library scan from stable candidates"
                                );
                                }
                                if diagnostic.backoff_required {
                                    warn!(
                                        library_id = %diagnostic.library_id,
                                        failure_count = diagnostic.discovery_failures.len(),
                                        "watch-folder runtime tick observed discovery failures"
                                    );
                                }
                                watch_folder_runtime_delay_after_tick(&diagnostic)
                            }
                            Err(err) => {
                                let safe_error = watch_folder_runtime_safe_error_message(&err);
                                warn!(
                                    library_id = %library_id,
                                    error = %safe_error,
                                    "watch-folder runtime tick failed"
                                );
                                Duration::from_millis(WATCH_FOLDER_RUNTIME_ERROR_BACKOFF_MS)
                            }
                        };

                        tokio::select! {
                            () = shutdown.cancelled() => break,
                            () = tokio::time::sleep(delay) => {}
                        }
                    }
                },
            );
        }

        Ok(coverage)
    }

    pub(crate) async fn runtime_coverage_report(&self) -> Result<WatchFolderRuntimeCoverageReport> {
        Ok(WatchFolderRuntimeCoverageReport {
            diagnostics: watch_folder_runtime_coverage_for_libraries(&self.list_libraries().await?),
        })
    }

    pub(crate) async fn tick_library(
        &self,
        library_id: LibraryId,
    ) -> Result<WatchFolderRuntimeTickDiagnostic> {
        let Some(library) = self.store.get_library(library_id).await? else {
            return Ok(WatchFolderRuntimeTickDiagnostic::unmonitored(library_id));
        };

        if !library.options.scan.realtime_monitor || !is_local_watch_folder_root(&library) {
            return Ok(WatchFolderRuntimeTickDiagnostic::unmonitored(library_id));
        }

        let discovery = self
            .acquisition_intake
            .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
                target_library_id: library_id,
                root_uri: None,
                max_depth: None,
            })
            .await?;
        let intake_plan = plan_watch_folder_intake(WatchFolderIntakePlanInput {
            ready_candidates: discovery.ready_candidates,
            inspecting_candidates: discovery.inspecting_candidates,
            blocked_candidates: discovery.blocked_candidates,
            recorded_candidates: discovery.recorded_candidates,
            newly_ready_candidates: discovery.newly_ready_candidates,
            suppressed_candidates: discovery.suppressed_candidates,
            active_suppressions: discovery.active_suppressions.len() as u64,
            failure_count: discovery.failures.len() as u64,
        });
        let discovery_failures = discovery
            .failures
            .into_iter()
            .map(WatchFolderRuntimeFailureDiagnostic::from)
            .collect::<Vec<_>>();
        let backoff_required = !discovery_failures.is_empty();
        let (scan_job_id, reused_existing_scan) = if intake_plan.summary.enqueue_scan {
            let outcome = self
                .library_scan
                .admit_watch_folder_library_scan(library_id)
                .await?;
            (Some(outcome.job_id()), outcome.reused_existing())
        } else {
            (None, false)
        };

        Ok(WatchFolderRuntimeTickDiagnostic {
            library_id,
            monitored: true,
            intake_plan,
            discovery_failures,
            scan_job_id,
            reused_existing_scan,
            backoff_required,
        })
    }

    async fn list_libraries(&self) -> Result<Vec<Library>> {
        let mut libraries = Vec::new();
        let mut offset = 0_u64;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let mut batch = self.store.list_libraries(page).await?;
            let returned = batch.len();
            libraries.append(&mut batch);

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(libraries);
            }

            offset = offset.saturating_add(returned as u64);
        }
    }
}

impl WatchFolderRuntimeTickDiagnostic {
    fn unmonitored(library_id: LibraryId) -> Self {
        Self {
            library_id,
            monitored: false,
            intake_plan: WatchFolderIntakePlan::idle(),
            discovery_failures: Vec::new(),
            scan_job_id: None,
            reused_existing_scan: false,
            backoff_required: false,
        }
    }
}

impl From<WatchFolderDiscoveryFailureDiagnostic> for WatchFolderRuntimeFailureDiagnostic {
    fn from(value: WatchFolderDiscoveryFailureDiagnostic) -> Self {
        Self {
            uri_redacted: value.uri_redacted,
            safe_message: value.safe_message,
        }
    }
}

pub(super) fn watch_folder_runtime_delay_after_tick(
    diagnostic: &WatchFolderRuntimeTickDiagnostic,
) -> Duration {
    if diagnostic.backoff_required {
        Duration::from_millis(WATCH_FOLDER_RUNTIME_ERROR_BACKOFF_MS)
    } else {
        Duration::from_millis(WATCH_FOLDER_RUNTIME_INTERVAL_MS)
    }
}

fn watch_folder_runtime_coverage_for_libraries(
    libraries: &[Library],
) -> Vec<WatchFolderRuntimeCoverageDiagnostic> {
    libraries
        .iter()
        .map(watch_folder_runtime_coverage_for_library)
        .collect()
}

fn watch_folder_runtime_coverage_for_library(
    library: &Library,
) -> WatchFolderRuntimeCoverageDiagnostic {
    let root = library
        .roots
        .first()
        .and_then(|root| StorageUri::parse(root).ok());
    let root_scheme = root.as_ref().map(|root| root.scheme().to_owned());
    let root_ref_redacted = root_scheme
        .as_deref()
        .map(redact_storage_scheme)
        .unwrap_or_else(|| "<redacted>".to_owned());

    let (status, safe_reason) = if !library.options.scan.realtime_monitor {
        (
            WatchFolderRuntimeCoverageStatus::Disabled,
            "realtime monitoring is disabled",
        )
    } else {
        match root.as_ref().map(StorageUri::scheme) {
            Some("local") => (
                WatchFolderRuntimeCoverageStatus::Started,
                "local watch-folder runtime started",
            ),
            Some(_) => (
                WatchFolderRuntimeCoverageStatus::UnsupportedRoot,
                "watch-folder runtime requires a local root",
            ),
            None => (
                WatchFolderRuntimeCoverageStatus::MissingRoot,
                "library has no parseable root",
            ),
        }
    };

    WatchFolderRuntimeCoverageDiagnostic {
        library_id: library.id,
        library_name: library.name.clone(),
        root_scheme,
        root_ref_redacted,
        status,
        safe_reason: safe_reason.to_owned(),
    }
}

fn is_local_watch_folder_root(library: &Library) -> bool {
    library
        .roots
        .first()
        .and_then(|root| StorageUri::parse(root).ok())
        .is_some_and(|root| root.scheme() == "local")
}

fn redact_storage_scheme(scheme: &str) -> String {
    format!("{scheme}://<redacted>")
}

fn watch_folder_runtime_safe_error_message(err: &NakoError) -> String {
    match err {
        NakoError::NotFound { entity, .. } => format!("{entity} was not found"),
        NakoError::InvalidInput { .. } => "invalid watch-folder runtime input".to_owned(),
        NakoError::Conflict { .. } => "watch-folder runtime conflict".to_owned(),
        NakoError::Unauthorized { .. } => "watch-folder runtime access is unauthorized".to_owned(),
        NakoError::Forbidden { .. } => "watch-folder runtime access is forbidden".to_owned(),
        NakoError::Unsupported(_) => "watch-folder runtime operation is unsupported".to_owned(),
        NakoError::Provider { .. } => "provider error".to_owned(),
        NakoError::Storage { kind, .. } => format!("storage error: {kind:?}"),
        NakoError::Database { .. } => "database error".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use nako_core::{LibraryOptions, LibraryPreset};

    use super::*;

    #[test]
    fn watch_folder_runtime_coverage_reports_started_and_skipped_reasons() {
        let diagnostics = watch_folder_runtime_coverage_for_libraries(&[
            library("Started", vec!["local:///Movies"], true),
            library("Disabled", vec!["local:///Disabled"], false),
            library("Remote", vec!["webdav:///Movies"], true),
            library("Missing", Vec::new(), true),
        ]);

        assert_eq!(diagnostics.len(), 4);
        assert_eq!(
            diagnostics[0].status,
            WatchFolderRuntimeCoverageStatus::Started
        );
        assert_eq!(diagnostics[0].root_scheme.as_deref(), Some("local"));
        assert_eq!(diagnostics[0].root_ref_redacted, "local://<redacted>");
        assert_eq!(
            diagnostics[1].status,
            WatchFolderRuntimeCoverageStatus::Disabled
        );
        assert_eq!(
            diagnostics[2].status,
            WatchFolderRuntimeCoverageStatus::UnsupportedRoot
        );
        assert_eq!(diagnostics[2].root_ref_redacted, "webdav://<redacted>");
        assert_eq!(
            diagnostics[3].status,
            WatchFolderRuntimeCoverageStatus::MissingRoot
        );
        assert_eq!(diagnostics[3].root_ref_redacted, "<redacted>");
        assert!(
            !format!("{diagnostics:?}").contains("local:///Movies"),
            "coverage diagnostics must not leak raw roots"
        );
    }

    #[test]
    fn watch_folder_runtime_safe_error_message_redacts_storage_details() {
        let error = NakoError::storage_io(
            "local:///Secret Folder/Leaked Movie.mkv?token=secret",
            "failed to inspect C:\\Secret Folder\\Leaked Movie.mkv?token=secret",
        );

        let safe = watch_folder_runtime_safe_error_message(&error);

        assert_eq!(safe, "storage error: Io");
        assert!(!safe.contains("Secret Folder"));
        assert!(!safe.contains("Leaked Movie"));
        assert!(!safe.contains("local:///"));
        assert!(!safe.contains("token=secret"));
        assert!(!safe.contains("C:\\"));
    }

    fn library(name: &str, roots: Vec<&str>, realtime_monitor: bool) -> Library {
        let mut options = LibraryOptions::from_preset(LibraryPreset::Movies);
        options.scan.realtime_monitor = realtime_monitor;
        Library {
            id: LibraryId::new(),
            name: name.to_owned(),
            roots: roots.into_iter().map(str::to_owned).collect(),
            options,
        }
    }
}
