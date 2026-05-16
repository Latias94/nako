use async_trait::async_trait;

use super::PageRequest;
use crate::{
    DirectorySnapshot, LibraryId, MediaItem, MediaSource, Result, ScanSnapshot, ScanSnapshotId,
    ScanStatus, SourceState,
};

#[async_trait]
pub trait ScanRepository: Send + Sync {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot>;

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot>;

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>>;

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()>;

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>>;

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()>;

    async fn record_scanned_media_source(
        &self,
        item: &MediaItem,
        source: &MediaSource,
        state: &SourceState,
    ) -> Result<()>;

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>>;

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>>;
}
