use std::time::Duration;

use nako_core::{JobId, Library, LibraryId, LibraryRepository, PageRequest, Result};
use nako_db::NakoDatabase;
use nako_vfs::StorageUri;
use tracing::{info, warn};

use super::{
    acquisition_intake::{AcquisitionIntakeAppService, DiscoverWatchFolderCandidatesRequest},
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
    pub(crate) newly_ready_candidates: u64,
    pub(crate) suppressed_candidates: u64,
    pub(crate) enqueued_job_id: Option<JobId>,
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
                                if let Some(job_id) = diagnostic.enqueued_job_id {
                                    info!(
                                        library_id = %diagnostic.library_id,
                                        job_id = %job_id,
                                        newly_ready_candidates = diagnostic.newly_ready_candidates,
                                        "watch-folder runtime queued library scan from stable candidates"
                                    );
                                }
                                Duration::from_millis(WATCH_FOLDER_RUNTIME_INTERVAL_MS)
                            }
                            Err(err) => {
                                warn!(
                                    library_id = %library_id,
                                    error = %err,
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
            return Ok(WatchFolderRuntimeTickDiagnostic {
                library_id,
                monitored: false,
                newly_ready_candidates: 0,
                suppressed_candidates: 0,
                enqueued_job_id: None,
            });
        };

        if !library.options.scan.realtime_monitor || !is_local_watch_folder_root(&library) {
            return Ok(WatchFolderRuntimeTickDiagnostic {
                library_id,
                monitored: false,
                newly_ready_candidates: 0,
                suppressed_candidates: 0,
                enqueued_job_id: None,
            });
        }

        let discovery = self
            .acquisition_intake
            .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
                target_library_id: library_id,
                root_uri: None,
                max_depth: None,
            })
            .await?;
        let enqueued_job_id = if discovery.newly_ready_candidates > 0 {
            Some(self.library_scan.enqueue_library_scan(library_id).await?.id)
        } else {
            None
        };

        Ok(WatchFolderRuntimeTickDiagnostic {
            library_id,
            monitored: true,
            newly_ready_candidates: discovery.newly_ready_candidates,
            suppressed_candidates: discovery.suppressed_candidates,
            enqueued_job_id,
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
