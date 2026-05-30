use async_trait::async_trait;
use nako_core::{
    CatalogRepository, DirectorySnapshot, IngestionFailurePhase, IngestionFailureRepository, JobId,
    Library, LibraryId, LibraryItemRepository, LibraryRepository,
    LibraryScanSourcePersistenceSummary, MediaRepository, NewIngestionFailure, PageRequest, Result,
    ScanRepository, ScanSnapshot, ScanSnapshotId, ScanStatus,
};

use super::{
    failure::ingestion_failure_time_ms,
    scan::{DiscoveredMediaSource, ScannedDirectory},
    summary::LibraryScanFailure,
};

mod source_commit;

use source_commit::{SourceObservationDisposition, plan_source_observation_commit};

#[async_trait]
pub trait LibraryIngestionWorkflow: Send + Sync {
    async fn ensure_library_for_ingestion(&self, library: &Library) -> Result<()>;

    async fn begin_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot>;

    async fn complete_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot>;

    async fn record_scan_failure(&self, commit: LibraryScanFailureCommit) -> Result<()>;

    async fn commit_directory_observation(
        &self,
        commit: LibraryDirectoryObservationCommit,
    ) -> Result<()>;

    async fn commit_source_observation(
        &self,
        commit: LibrarySourceObservationCommit,
    ) -> Result<LibrarySourceIngestionSummary>;

    async fn tombstone_sources_missing_from_scan(
        &self,
        library_id: LibraryId,
        scan_id: ScanSnapshotId,
    ) -> Result<u64>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryScanFailureCommit {
    pub library_id: LibraryId,
    pub job_id: JobId,
    pub scan_id: ScanSnapshotId,
    pub failure: LibraryScanFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryDirectoryObservationCommit {
    pub library_id: LibraryId,
    pub scan_id: ScanSnapshotId,
    pub directory: ScannedDirectory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrarySourceObservationCommit {
    pub library_id: LibraryId,
    pub scan_id: ScanSnapshotId,
    pub discovered: DiscoveredMediaSource,
    pub scan_source_locators: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrarySourceIngestionSummary {
    pub disposition: LibrarySourceIngestionDisposition,
    pub persistence: LibraryScanSourcePersistenceSummary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LibrarySourceIngestionDisposition {
    Inserted,
    Updated,
}

impl LibrarySourceIngestionDisposition {
    #[must_use]
    pub const fn is_update(self) -> bool {
        matches!(self, Self::Updated)
    }
}

impl From<SourceObservationDisposition> for LibrarySourceIngestionDisposition {
    fn from(value: SourceObservationDisposition) -> Self {
        if value.is_update() {
            Self::Updated
        } else {
            Self::Inserted
        }
    }
}

#[async_trait]
impl<T> LibraryIngestionWorkflow for T
where
    T: CatalogRepository
        + IngestionFailureRepository
        + LibraryItemRepository
        + LibraryRepository
        + MediaRepository
        + ScanRepository,
{
    async fn ensure_library_for_ingestion(&self, library: &Library) -> Result<()> {
        LibraryRepository::upsert_library(self, library).await
    }

    async fn begin_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        ScanRepository::begin_scan_snapshot(self, id, library_id, root).await
    }

    async fn complete_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        ScanRepository::complete_scan_snapshot(self, id, status, error).await
    }

    async fn record_scan_failure(&self, commit: LibraryScanFailureCommit) -> Result<()> {
        IngestionFailureRepository::record_ingestion_failure(
            self,
            NewIngestionFailure {
                library_id: commit.library_id,
                job_id: Some(commit.job_id),
                scan_id: Some(commit.scan_id),
                source_id: None,
                phase: IngestionFailurePhase::Scan,
                target_uri: commit.failure.uri.as_str().to_owned(),
                target_kind: commit.failure.target_kind,
                failure_class: commit.failure.failure_class,
                message: commit.failure.message,
                retryable: commit.failure.retryable,
                failed_at_ms: ingestion_failure_time_ms(),
            },
        )
        .await?;

        Ok(())
    }

    async fn commit_directory_observation(
        &self,
        commit: LibraryDirectoryObservationCommit,
    ) -> Result<()> {
        ScanRepository::upsert_directory_snapshot(
            self,
            &DirectorySnapshot {
                scan_id: commit.scan_id,
                uri: commit.directory.uri.as_str().to_owned(),
                etag: commit.directory.etag,
                modified_at: commit.directory.modified_at,
                child_count: commit.directory.child_count,
            },
        )
        .await?;
        IngestionFailureRepository::resolve_ingestion_failure(
            self,
            commit.library_id,
            IngestionFailurePhase::Scan,
            commit.directory.uri.as_str(),
            ingestion_failure_time_ms(),
        )
        .await?;

        Ok(())
    }

    async fn commit_source_observation(
        &self,
        commit: LibrarySourceObservationCommit,
    ) -> Result<LibrarySourceIngestionSummary> {
        let plan = plan_source_observation_commit(self, commit).await?;
        let persistence = ScanRepository::commit_library_scan_source(self, &plan.commit).await?;

        Ok(LibrarySourceIngestionSummary {
            disposition: plan.disposition.into(),
            persistence,
        })
    }

    async fn tombstone_sources_missing_from_scan(
        &self,
        library_id: LibraryId,
        scan_id: ScanSnapshotId,
    ) -> Result<u64> {
        let mut offset = 0;
        let mut tombstoned = 0;

        loop {
            let states = ScanRepository::list_source_states(
                self,
                library_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
            let returned = states.len();

            for mut state in states {
                if state.last_seen_scan_id != scan_id && !state.tombstoned {
                    state.tombstoned = true;
                    ScanRepository::upsert_source_state(self, &state).await?;
                    tombstoned += 1;
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(tombstoned)
    }
}
