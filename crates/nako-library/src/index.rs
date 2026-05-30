use nako_core::{Result, ScanSnapshotId, ScanStatus};
use nako_vfs::StorageUri;

use super::{
    ingestion::{
        LibraryDirectoryObservationCommit, LibraryIngestionWorkflow, LibraryScanFailureCommit,
        LibrarySourceIngestionDisposition, LibrarySourceObservationCommit,
    },
    scan::LibraryScanner,
    summary::{LibraryIndexRequest, LibraryIndexSummary, LibraryScanRequest},
};

#[derive(Debug)]
pub struct LibraryIndexService<S, R> {
    scanner: S,
    repository: R,
}

impl<S, R> LibraryIndexService<S, R> {
    pub fn new(scanner: S, repository: R) -> Self {
        Self {
            scanner,
            repository,
        }
    }

    #[must_use]
    pub fn scanner(&self) -> &S {
        &self.scanner
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<S, R> LibraryIndexService<S, R>
where
    S: LibraryScanner,
    R: LibraryIngestionWorkflow,
{
    pub async fn index_library(&self, request: LibraryIndexRequest) -> Result<LibraryIndexSummary> {
        self.repository
            .ensure_library_for_ingestion(&request.library)
            .await?;
        let scan_id = ScanSnapshotId::new();

        let mut summary = LibraryIndexSummary {
            job_id: request.job_id,
            library_id: request.library.id,
            scan_id,
            scanned_roots: 0,
            discovered_files: 0,
            inserted_sources: 0,
            updated_sources: 0,
            tombstoned_sources: 0,
            failed_entries: 0,
        };

        let first_root = request
            .library
            .roots
            .first()
            .map(String::as_str)
            .unwrap_or("local:///");
        self.repository
            .begin_ingestion_scan(scan_id, request.library.id, first_root)
            .await?;

        let result = self.index_roots(&request, scan_id, &mut summary).await;

        match result {
            Ok(scan) => {
                if scan.complete {
                    summary.tombstoned_sources += self
                        .repository
                        .tombstone_sources_missing_from_scan(request.library.id, scan_id)
                        .await?;
                }
                self.repository
                    .complete_ingestion_scan(scan_id, ScanStatus::Succeeded, None)
                    .await?;
                Ok(summary)
            }
            Err(err) => {
                self.repository
                    .complete_ingestion_scan(scan_id, ScanStatus::Failed, Some(err.to_string()))
                    .await?;
                Err(err)
            }
        }
    }

    async fn index_roots(
        &self,
        request: &LibraryIndexRequest,
        scan_id: ScanSnapshotId,
        summary: &mut LibraryIndexSummary,
    ) -> Result<IndexRootsOutcome> {
        let mut complete = true;

        for root in &request.library.roots {
            let root = StorageUri::parse(root)?;
            let scan = self
                .scanner
                .scan(LibraryScanRequest {
                    job_id: request.job_id,
                    library_id: request.library.id,
                    root,
                    force: request.force,
                })
                .await?;

            summary.scanned_roots += 1;
            summary.discovered_files += scan.discovered_files;
            summary.failed_entries += scan.failures.len() as u64;
            if scan.used_stale_cache || !scan.failures.is_empty() {
                complete = false;
            }

            for failure in scan.failures {
                self.repository
                    .record_scan_failure(LibraryScanFailureCommit {
                        library_id: request.library.id,
                        job_id: request.job_id,
                        scan_id,
                        failure,
                    })
                    .await?;
            }

            for directory in scan.directories {
                self.repository
                    .commit_directory_observation(LibraryDirectoryObservationCommit {
                        library_id: request.library.id,
                        scan_id,
                        directory,
                    })
                    .await?;
            }

            let scan_source_locators = scan
                .media_sources
                .iter()
                .map(|source| source.uri.as_str().to_owned())
                .collect::<Vec<_>>();

            for discovered in scan.media_sources {
                let source_summary = self
                    .repository
                    .commit_source_observation(LibrarySourceObservationCommit {
                        library_id: request.library.id,
                        scan_id,
                        discovered,
                        scan_source_locators: scan_source_locators.clone(),
                    })
                    .await?;

                match source_summary.disposition {
                    LibrarySourceIngestionDisposition::Inserted => {
                        summary.inserted_sources += 1;
                    }
                    LibrarySourceIngestionDisposition::Updated => {
                        summary.updated_sources += 1;
                    }
                }
            }
        }

        Ok(IndexRootsOutcome { complete })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IndexRootsOutcome {
    complete: bool,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use nako_core::{
        JobId, Library, LibraryId, LibraryOptions, LibraryPreset,
        LibraryScanSourcePersistenceSummary, MediaSourceId, Result, ScanSnapshot, ScanSnapshotId,
        ScanStatus,
    };
    use nako_vfs::StorageUri;

    use super::*;
    use crate::{
        DiscoveredMediaSource, LibraryScanSummary, LibrarySourceIngestionSummary, ScannedDirectory,
    };

    #[tokio::test]
    async fn index_service_uses_workflow_port_without_repository_traits() {
        let scanner = SingleSourceScanner;
        let workflow = RecordingIngestionWorkflow::default();
        let service = LibraryIndexService::new(scanner, workflow.clone());
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };

        let summary = service
            .index_library(LibraryIndexRequest {
                job_id: JobId::new(),
                library: library.clone(),
                force: false,
            })
            .await
            .unwrap();
        let calls = workflow.calls.lock().unwrap().clone();

        assert_eq!(summary.library_id, library.id);
        assert_eq!(summary.scanned_roots, 1);
        assert_eq!(summary.discovered_files, 1);
        assert_eq!(summary.inserted_sources, 1);
        assert_eq!(summary.updated_sources, 0);
        assert_eq!(summary.tombstoned_sources, 2);
        assert_eq!(calls.ensure_libraries, vec![library.id]);
        assert_eq!(calls.begun_roots, vec!["local:///Movies".to_owned()]);
        assert_eq!(calls.directories, vec!["local:///Movies/".to_owned()]);
        assert_eq!(
            calls.sources,
            vec!["local:///Movies/The Matrix (1999).mkv".to_owned()]
        );
        assert_eq!(
            calls.completed,
            vec![(summary.scan_id, ScanStatus::Succeeded, None)]
        );
    }

    struct SingleSourceScanner;

    #[async_trait]
    impl LibraryScanner for SingleSourceScanner {
        async fn scan(&self, request: LibraryScanRequest) -> Result<LibraryScanSummary> {
            Ok(LibraryScanSummary {
                job_id: request.job_id,
                discovered_files: 1,
                changed_files: 0,
                removed_files: 0,
                used_stale_cache: false,
                media_sources: vec![DiscoveredMediaSource {
                    uri: StorageUri::parse("local:///Movies/The Matrix (1999).mkv").unwrap(),
                    file_name: "The Matrix (1999).mkv".to_owned(),
                    size_bytes: Some(6),
                    modified_at: None,
                    etag: None,
                    fingerprint: None,
                    fingerprint_evidence_kind:
                        nako_core::SourceFingerprintEvidenceKind::LocatorOnly,
                    fingerprint_confidence_milli: 250,
                    stale: false,
                }],
                directories: vec![ScannedDirectory {
                    uri: StorageUri::parse("local:///Movies/").unwrap(),
                    etag: None,
                    modified_at: None,
                    child_count: 1,
                    stale: false,
                }],
                failures: Vec::new(),
            })
        }
    }

    #[derive(Clone, Default)]
    struct RecordingIngestionWorkflow {
        calls: Arc<Mutex<RecordedCalls>>,
    }

    #[derive(Clone, Debug, Default)]
    struct RecordedCalls {
        ensure_libraries: Vec<LibraryId>,
        begun_roots: Vec<String>,
        directories: Vec<String>,
        sources: Vec<String>,
        completed: Vec<(ScanSnapshotId, ScanStatus, Option<String>)>,
    }

    #[async_trait]
    impl LibraryIngestionWorkflow for RecordingIngestionWorkflow {
        async fn ensure_library_for_ingestion(&self, library: &Library) -> Result<()> {
            self.calls.lock().unwrap().ensure_libraries.push(library.id);
            Ok(())
        }

        async fn begin_ingestion_scan(
            &self,
            id: ScanSnapshotId,
            library_id: LibraryId,
            root: &str,
        ) -> Result<ScanSnapshot> {
            self.calls.lock().unwrap().begun_roots.push(root.to_owned());
            Ok(scan_snapshot(
                id,
                library_id,
                root,
                ScanStatus::Running,
                None,
            ))
        }

        async fn complete_ingestion_scan(
            &self,
            id: ScanSnapshotId,
            status: ScanStatus,
            error: Option<String>,
        ) -> Result<ScanSnapshot> {
            self.calls
                .lock()
                .unwrap()
                .completed
                .push((id, status, error.clone()));
            Ok(scan_snapshot(id, LibraryId::new(), "", status, error))
        }

        async fn record_scan_failure(&self, commit: LibraryScanFailureCommit) -> Result<()> {
            self.calls
                .lock()
                .unwrap()
                .directories
                .push(commit.failure.uri.as_str().to_owned());
            Ok(())
        }

        async fn commit_directory_observation(
            &self,
            commit: LibraryDirectoryObservationCommit,
        ) -> Result<()> {
            self.calls
                .lock()
                .unwrap()
                .directories
                .push(commit.directory.uri.as_str().to_owned());
            Ok(())
        }

        async fn commit_source_observation(
            &self,
            commit: LibrarySourceObservationCommit,
        ) -> Result<LibrarySourceIngestionSummary> {
            self.calls
                .lock()
                .unwrap()
                .sources
                .push(commit.discovered.uri.as_str().to_owned());
            Ok(LibrarySourceIngestionSummary {
                disposition: LibrarySourceIngestionDisposition::Inserted,
                persistence: LibraryScanSourcePersistenceSummary {
                    item_ids: Vec::new(),
                    source_id: MediaSourceId::new(),
                    library_item_states: 0,
                    local_inference_evidence: 0,
                    search_projections: 0,
                    source_duplicate_relationships: 0,
                    resolved_ingestion_failures: 0,
                },
            })
        }

        async fn tombstone_sources_missing_from_scan(
            &self,
            _library_id: LibraryId,
            _scan_id: ScanSnapshotId,
        ) -> Result<u64> {
            Ok(2)
        }
    }

    fn scan_snapshot(
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
        status: ScanStatus,
        error: Option<String>,
    ) -> ScanSnapshot {
        ScanSnapshot {
            id,
            library_id,
            root: root.to_owned(),
            started_at: "0".to_owned(),
            completed_at: None,
            status,
            error,
        }
    }
}
