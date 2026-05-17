use serde::{Deserialize, Serialize};
use taru_core::{JobId, LocalMetadataPolicy, MediaItemId, MediaSourceId};
use taru_vfs::StorageUri;

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
pub struct NfoExportRequest {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
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
pub struct NfoExportSummary {
    pub job_id: JobId,
    pub library_id: taru_core::LibraryId,
    pub scanned_sources: u64,
    pub exported_items: u64,
    pub skipped_items: u64,
    pub failed_items: u64,
    #[serde(default)]
    pub backed_up_items: u64,
    #[serde(default)]
    pub backups: Vec<NfoBackupReport>,
    pub failures: Vec<NfoFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoBackupReport {
    pub source_id: MediaSourceId,
    pub locator: String,
    pub original_uri: StorageUri,
    pub backup_uri: StorageUri,
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
