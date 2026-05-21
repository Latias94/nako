use serde::{Deserialize, Serialize};
use taru_core::Result;
use taru_core::{JobId, LibraryId, LocalMetadataPolicy, MediaItemId, MediaSourceId};
use taru_vfs::StorageUri;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NfoSidecarOperation {
    Import,
    Export,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoSidecarCheckpoint {
    pub operation: NfoSidecarOperation,
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NfoCancellationDecision {
    #[default]
    Continue,
    Cancel,
}

#[async_trait::async_trait]
pub trait NfoCancellationCheck: Send + Sync {
    async fn check(&self, checkpoint: NfoSidecarCheckpoint) -> Result<NfoCancellationDecision>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopNfoCancellationCheck;

#[async_trait::async_trait]
impl NfoCancellationCheck for NoopNfoCancellationCheck {
    async fn check(&self, _checkpoint: NfoSidecarCheckpoint) -> Result<NfoCancellationDecision> {
        Ok(NfoCancellationDecision::Continue)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "summary")]
pub enum NfoLibraryRunOutcome<T> {
    Completed(T),
    Cancelled(T),
}

impl<T> NfoLibraryRunOutcome<T> {
    #[must_use]
    pub fn summary(&self) -> &T {
        match self {
            Self::Completed(summary) | Self::Cancelled(summary) => summary,
        }
    }

    #[must_use]
    pub fn into_summary(self) -> T {
        match self {
            Self::Completed(summary) | Self::Cancelled(summary) => summary,
        }
    }

    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(_))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoSidecar {
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
    pub source_locator: String,
    pub nfo_uri: StorageUri,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoJobInput {
    pub library_id: taru_core::LibraryId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoImportRequest {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoImportSourceRequest {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoExportRequest {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoExportSourceRequest {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub policy: LocalMetadataPolicy,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoImportSummary {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub scanned_sources: u64,
    pub discovered_nfo: u64,
    pub imported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoImportSourceSummary {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub scanned_sources: u64,
    pub discovered_nfo: u64,
    pub imported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoExportSummary {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub scanned_sources: u64,
    pub exported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    #[serde(default)]
    pub backed_up_items: u64,
    #[serde(default)]
    pub backups: Vec<NfoBackupReport>,
    #[serde(default)]
    pub pruned_backup_items: u64,
    #[serde(default)]
    pub pruned_backups: u64,
    #[serde(default)]
    pub prune_failures: Vec<NfoBackupPruneFailure>,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoExportSourceSummary {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub scanned_sources: u64,
    pub exported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    #[serde(default)]
    pub backed_up_items: u64,
    #[serde(default)]
    pub backups: Vec<NfoBackupReport>,
    #[serde(default)]
    pub pruned_backup_items: u64,
    #[serde(default)]
    pub pruned_backups: u64,
    #[serde(default)]
    pub prune_failures: Vec<NfoBackupPruneFailure>,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoBackupReport {
    pub source_id: MediaSourceId,
    pub locator: String,
    pub original_uri: StorageUri,
    pub backup_uri: StorageUri,
    #[serde(default)]
    pub pruned_backups: Vec<StorageUri>,
    #[serde(default)]
    pub prune_failures: Vec<NfoBackupPruneFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoBackupPruneFailure {
    pub source_id: MediaSourceId,
    pub locator: String,
    pub backup_uri: StorageUri,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NfoFailureKind {
    NfoParse,
    NfoPreservation,
    NfoRender,
    NfoConflict,
    StorageRead,
    StorageWrite,
    StorageUnsupported,
    StorageBackup,
    MissingMediaItem,
    InvalidSidecarPath,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoFailure {
    pub source_id: MediaSourceId,
    pub locator: String,
    #[serde(default)]
    pub kind: NfoFailureKind,
    pub message: String,
}
