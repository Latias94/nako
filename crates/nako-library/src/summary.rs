use nako_core::{
    IngestionFailureClass, JobId, Library, LibraryId, MediaSourceId, ScanSnapshotId,
    SourceFingerprintEscalationDecision,
};
use nako_vfs::StorageUri;
use serde::{Deserialize, Serialize};

use super::source_hash::SourceFingerprintHashMode;

use super::scan::{DiscoveredMediaSource, ScannedDirectory};

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
    pub used_stale_cache: bool,
    pub media_sources: Vec<DiscoveredMediaSource>,
    pub directories: Vec<ScannedDirectory>,
    pub failures: Vec<LibraryScanFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryIndexRequest {
    pub job_id: JobId,
    pub library: Library,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryIndexSummary {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub scan_id: ScanSnapshotId,
    pub scanned_roots: u64,
    pub discovered_files: u64,
    pub inserted_sources: u64,
    pub updated_sources: u64,
    pub tombstoned_sources: u64,
    pub failed_entries: u64,
    #[serde(skip)]
    pub source_fingerprint_hash_triggers: Vec<ScanSourceFingerprintHashTrigger>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScanSourceFingerprintHashTrigger {
    pub source_id: MediaSourceId,
    pub decision: SourceFingerprintEscalationDecision,
    pub mode: Option<SourceFingerprintHashMode>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryProbeRequest {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryProbeSummary {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub total_sources: u64,
    pub probed_sources: u64,
    pub skipped_sources: u64,
    pub failed_sources: u64,
    pub failures: Vec<LibraryProbeFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryProbeFailure {
    pub source_id: Option<MediaSourceId>,
    pub locator: String,
    pub failure_class: IngestionFailureClass,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanFailure {
    pub uri: StorageUri,
    pub target_kind: String,
    pub failure_class: IngestionFailureClass,
    pub message: String,
    pub retryable: bool,
}
