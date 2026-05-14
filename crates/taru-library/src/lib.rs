use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{JobId, LibraryId, Result};
use taru_vfs::StorageUri;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanRequest {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub root: StorageUri,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanSummary {
    pub job_id: JobId,
    pub discovered_files: u64,
    pub changed_files: u64,
    pub removed_files: u64,
}

#[async_trait]
pub trait LibraryScanner: Send + Sync {
    async fn scan(&self, request: LibraryScanRequest) -> Result<LibraryScanSummary>;
}
