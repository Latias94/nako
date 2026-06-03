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
    ) -> Result<usize> {
        let libraries = self.list_monitored_libraries().await?;

        for library in &libraries {
            let service = self.clone();
            let library_id = library.id;
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

        Ok(libraries.len())
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

    async fn list_monitored_libraries(&self) -> Result<Vec<Library>> {
        let mut libraries = Vec::new();
        let mut offset = 0_u64;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let mut batch = self.store.list_libraries(page).await?;
            let returned = batch.len();
            batch.retain(|library| {
                library.options.scan.realtime_monitor && is_local_watch_folder_root(library)
            });
            libraries.append(&mut batch);

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(libraries);
            }

            offset = offset.saturating_add(returned as u64);
        }
    }
}

fn is_local_watch_folder_root(library: &Library) -> bool {
    library
        .roots
        .first()
        .and_then(|root| StorageUri::parse(root).ok())
        .is_some_and(|root| root.scheme() == "local")
}
