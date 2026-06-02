use nako_client_protocol::{PageInfo, PublicImageRefDto};
use nako_core::{
    AddonId, AddonSideEffectId, ArtworkCandidateId, ArtworkCandidateSourceKind,
    ArtworkCandidateStatus, ImageKind, Job, JobId, JobKind, JobStatus, LibraryId,
    ManagedArtworkAcceptanceRecord, ManagedArtworkArtifactCleanupReport, ManagedArtworkArtifactId,
    ManagedArtworkArtifactLifecycleRecord, ManagedArtworkArtifactLifecycleSnapshot,
    ManagedArtworkArtifactLifecycleSummary, ManagedArtworkArtifactRecord,
    ManagedArtworkGalleryArtifactRecord, ManagedArtworkGalleryCandidateRecord,
    ManagedArtworkGallerySelectedRecord, ManagedArtworkGallerySnapshot, ManagedArtworkIngestId,
    ManagedArtworkIngestProcessingRecord, ManagedArtworkIngestRecord,
    ManagedArtworkIngestRequeueRecord, ManagedArtworkIngestStatus, MediaItemId, MediaSourceId,
    SelectedArtworkPublicationRecord, SelectedArtworkRecord, SelectedArtworkUnpublicationRecord,
};
use serde::{Deserialize, Serialize};

use crate::public_client::selected_artwork_to_public_image_ref;

use super::JobResponse;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AcceptManagedArtworkCandidateResponse {
    pub candidate_id: ArtworkCandidateId,
    pub candidate_status: ArtworkCandidateStatus,
    pub ingest: ManagedArtworkIngestSummary,
    pub job: JobResponse,
}

impl AcceptManagedArtworkCandidateResponse {
    #[must_use]
    pub fn from_acceptance(acceptance: ManagedArtworkAcceptanceRecord) -> Self {
        Self {
            candidate_id: acceptance.candidate.id,
            candidate_status: acceptance.candidate.status,
            ingest: ManagedArtworkIngestSummary::from_record(acceptance.ingest),
            job: JobResponse::from_job(acceptance.job),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProcessManagedArtworkIngestResponse {
    pub processed: bool,
    pub ingest: Option<ManagedArtworkIngestSummary>,
    pub artifact: Option<ManagedArtworkArtifactSummary>,
    pub job: Option<JobResponse>,
}

impl ProcessManagedArtworkIngestResponse {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            processed: false,
            ingest: None,
            artifact: None,
            job: None,
        }
    }

    #[must_use]
    pub fn from_processing(processing: ManagedArtworkIngestProcessingRecord) -> Self {
        Self {
            processed: true,
            ingest: Some(ManagedArtworkIngestSummary::from_record(processing.ingest)),
            artifact: processing
                .artifact
                .map(ManagedArtworkArtifactSummary::from_record),
            job: Some(JobResponse::from_job(processing.job)),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RequeueManagedArtworkIngestResponse {
    pub ingest: ManagedArtworkIngestSummary,
    pub job: ManagedArtworkIngestJobSummary,
    pub requeued: bool,
    pub had_failure: bool,
}

impl RequeueManagedArtworkIngestResponse {
    #[must_use]
    pub fn from_requeue(requeue: ManagedArtworkIngestRequeueRecord) -> Self {
        Self {
            ingest: ManagedArtworkIngestSummary::from_record(requeue.ingest),
            job: ManagedArtworkIngestJobSummary::from_job(requeue.job),
            requeued: requeue.requeued,
            had_failure: requeue.had_failure,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkIngestSummary {
    pub id: ManagedArtworkIngestId,
    pub candidate_id: ArtworkCandidateId,
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub status: ManagedArtworkIngestStatus,
    pub has_artifact: bool,
    pub has_failure: bool,
    pub failure_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ManagedArtworkIngestSummary {
    #[must_use]
    pub fn from_record(record: ManagedArtworkIngestRecord) -> Self {
        Self {
            id: record.id,
            candidate_id: record.candidate_id,
            job_id: record.job_id,
            library_id: record.library_id,
            item_id: record.item_id,
            kind: record.kind,
            status: record.status,
            has_artifact: record.artifact_id.is_some(),
            has_failure: record.failure_code.is_some(),
            failure_code: record.failure_code,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkIngestJobSummary {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub has_input: bool,
    pub has_summary: bool,
    pub has_error: bool,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl ManagedArtworkIngestJobSummary {
    #[must_use]
    pub fn from_job(job: Job) -> Self {
        Self {
            id: job.id,
            kind: job.kind,
            status: job.status,
            resource_class: job.resource_class,
            library_id: job.library_id,
            source_id: job.source_id,
            has_input: job.input_json.is_some(),
            has_summary: job.summary_json.is_some(),
            has_error: job.error.is_some(),
            queued_at: job.queued_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedArtworkArtifactSummary {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub has_content_hash: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ManagedArtworkArtifactSummary {
    #[must_use]
    pub fn from_record(record: ManagedArtworkArtifactRecord) -> Self {
        Self {
            id: record.id,
            ingest_id: record.ingest_id,
            library_id: record.library_id,
            item_id: record.item_id,
            kind: record.kind,
            has_content_hash: record.content_hash.is_some(),
            width: record.width,
            height: record.height,
            byte_len: record.byte_len,
            media_type: record.media_type,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublishSelectedArtworkResponse {
    pub selected_artwork: SelectedArtworkSummary,
    pub image: PublicImageRefDto,
    pub changed: bool,
}

impl PublishSelectedArtworkResponse {
    #[must_use]
    pub fn from_publication(publication: SelectedArtworkPublicationRecord) -> Self {
        let image = selected_artwork_to_public_image_ref(
            publication.selected_artwork.clone(),
            publication.artifact,
        );
        let selected_artwork = SelectedArtworkSummary::from_record(publication.selected_artwork);

        Self {
            selected_artwork,
            image,
            changed: publication.changed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnpublishSelectedArtworkResponse {
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub changed: bool,
    pub unpublished: Option<UnpublishedSelectedArtworkSummary>,
}

impl UnpublishSelectedArtworkResponse {
    #[must_use]
    pub fn from_unpublication(unpublication: SelectedArtworkUnpublicationRecord) -> Self {
        let unpublished = match (unpublication.unpublished, unpublication.artifact) {
            (Some(selected_artwork), Some(artifact)) => Some(
                UnpublishedSelectedArtworkSummary::from_records(selected_artwork, artifact),
            ),
            (None, None) => None,
            _ => None,
        };

        Self {
            item_id: unpublication.item_id,
            kind: unpublication.kind,
            changed: unpublication.changed,
            unpublished,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnpublishedSelectedArtworkSummary {
    pub selected_artwork: SelectedArtworkSummary,
    pub previous_image: PublicImageRefDto,
}

impl UnpublishedSelectedArtworkSummary {
    #[must_use]
    pub fn from_records(
        selected_artwork: SelectedArtworkRecord,
        artifact: ManagedArtworkArtifactRecord,
    ) -> Self {
        let previous_image =
            selected_artwork_to_public_image_ref(selected_artwork.clone(), artifact);
        Self {
            selected_artwork: SelectedArtworkSummary::from_record(selected_artwork),
            previous_image,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SelectedArtworkSummary {
    pub id: nako_core::SelectedArtworkId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub artifact_id: ManagedArtworkArtifactId,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactLifecycleResponse {
    pub summary: AdminManagedArtworkArtifactLifecycleSummary,
    pub artifacts: Vec<AdminManagedArtworkArtifactLifecycleItem>,
    pub page: PageInfo,
    pub dry_run: bool,
}

impl AdminManagedArtworkArtifactLifecycleResponse {
    #[must_use]
    pub fn from_snapshot(
        snapshot: ManagedArtworkArtifactLifecycleSnapshot,
        page: PageInfo,
    ) -> Self {
        Self {
            summary: AdminManagedArtworkArtifactLifecycleSummary::from_summary(snapshot.summary),
            artifacts: snapshot
                .artifacts
                .into_iter()
                .map(AdminManagedArtworkArtifactLifecycleItem::from_record)
                .collect(),
            page,
            dry_run: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactLifecycleSummary {
    pub total_artifacts: u32,
    pub protected_artifacts: u32,
    pub cleanup_candidate_artifacts: u32,
    pub known_total_bytes: u64,
    pub known_protected_bytes: u64,
    pub known_cleanup_candidate_bytes: u64,
    pub unknown_byte_len_artifacts: u32,
}

impl AdminManagedArtworkArtifactLifecycleSummary {
    #[must_use]
    pub const fn from_summary(summary: ManagedArtworkArtifactLifecycleSummary) -> Self {
        Self {
            total_artifacts: summary.total_artifacts,
            protected_artifacts: summary.protected_artifacts,
            cleanup_candidate_artifacts: summary.cleanup_candidate_artifacts,
            known_total_bytes: summary.known_total_bytes,
            known_protected_bytes: summary.known_protected_bytes,
            known_cleanup_candidate_bytes: summary.known_cleanup_candidate_bytes,
            unknown_byte_len_artifacts: summary.unknown_byte_len_artifacts,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactLifecycleItem {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub selected_artwork_count: u32,
    pub cleanup_candidate: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
    pub has_content_hash: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminManagedArtworkArtifactLifecycleItem {
    #[must_use]
    pub fn from_record(record: ManagedArtworkArtifactLifecycleRecord) -> Self {
        let cleanup_candidate = record.cleanup_candidate();
        let artifact = record.artifact;
        Self {
            id: artifact.id,
            ingest_id: artifact.ingest_id,
            library_id: artifact.library_id,
            item_id: artifact.item_id,
            kind: artifact.kind,
            selected_artwork_count: record.selected_artwork_count,
            cleanup_candidate,
            width: artifact.width,
            height: artifact.height,
            byte_len: artifact.byte_len,
            media_type: artifact.media_type,
            has_content_hash: artifact.content_hash.is_some(),
            created_at: artifact.created_at,
            updated_at: artifact.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactCleanupResponse {
    pub examined_artifacts: u32,
    pub cleanup_candidate_artifacts: u32,
    pub cleaned_artifacts: Vec<AdminManagedArtworkArtifactCleanupItem>,
    pub file_deleted_artifacts: u32,
    pub file_missing_artifacts: u32,
    pub file_delete_failed_artifacts: u32,
    pub dry_run: bool,
}

impl AdminManagedArtworkArtifactCleanupResponse {
    #[must_use]
    pub fn from_report(
        report: ManagedArtworkArtifactCleanupReport,
        file_cleanup: AdminManagedArtworkArtifactFileCleanupSummary,
    ) -> Self {
        Self {
            examined_artifacts: report.examined_artifacts,
            cleanup_candidate_artifacts: report.cleanup_candidate_artifacts,
            cleaned_artifacts: report
                .cleaned_artifacts
                .into_iter()
                .map(AdminManagedArtworkArtifactCleanupItem::from_record)
                .collect(),
            file_deleted_artifacts: file_cleanup.file_deleted_artifacts,
            file_missing_artifacts: file_cleanup.file_missing_artifacts,
            file_delete_failed_artifacts: file_cleanup.file_delete_failed_artifacts,
            dry_run: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactFileCleanupSummary {
    pub file_deleted_artifacts: u32,
    pub file_missing_artifacts: u32,
    pub file_delete_failed_artifacts: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactCleanupItem {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
}

impl AdminManagedArtworkArtifactCleanupItem {
    #[must_use]
    pub fn from_record(record: ManagedArtworkArtifactRecord) -> Self {
        Self {
            id: record.id,
            ingest_id: record.ingest_id,
            library_id: record.library_id,
            item_id: record.item_id,
            kind: record.kind,
            byte_len: record.byte_len,
            media_type: record.media_type,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStorageDriftResponse {
    pub summary: AdminManagedArtworkArtifactStorageDriftSummary,
    pub missing_artifacts: Vec<AdminManagedArtworkArtifactStorageDriftArtifact>,
    pub stray_files: Vec<AdminManagedArtworkArtifactStorageDriftFile>,
    pub page: PageInfo,
    pub dry_run: bool,
}

impl AdminManagedArtworkArtifactStorageDriftResponse {
    #[must_use]
    pub fn new(
        summary: AdminManagedArtworkArtifactStorageDriftSummary,
        missing_artifacts: Vec<AdminManagedArtworkArtifactStorageDriftArtifact>,
        stray_files: Vec<AdminManagedArtworkArtifactStorageDriftFile>,
        page: PageInfo,
    ) -> Self {
        Self {
            summary,
            missing_artifacts,
            stray_files,
            page,
            dry_run: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStorageDriftSummary {
    pub scanned_db_artifacts: u32,
    pub db_backed_present_artifacts: u32,
    pub db_backed_missing_artifacts: u32,
    pub db_backed_unresolvable_artifacts: u32,
    pub db_backed_metadata_read_failed_artifacts: u32,
    pub file_scan_limit: u32,
    pub scanned_files: u32,
    pub stray_files: u32,
    pub untracked_artifact_files: u32,
    pub unexpected_active_artifact_files: u32,
    pub unsupported_extension_files: u32,
    pub unrecognized_layout_files: u32,
    pub file_scan_truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStorageDriftArtifact {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub selected_artwork_count: u32,
    pub cleanup_candidate: bool,
    pub issue: AdminManagedArtworkArtifactStorageDriftArtifactIssue,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
}

impl AdminManagedArtworkArtifactStorageDriftArtifact {
    #[must_use]
    pub fn from_lifecycle_record(
        record: ManagedArtworkArtifactLifecycleRecord,
        issue: AdminManagedArtworkArtifactStorageDriftArtifactIssue,
    ) -> Self {
        let cleanup_candidate = record.cleanup_candidate();
        let artifact = record.artifact;
        Self {
            id: artifact.id,
            ingest_id: artifact.ingest_id,
            library_id: artifact.library_id,
            item_id: artifact.item_id,
            kind: artifact.kind,
            selected_artwork_count: record.selected_artwork_count,
            cleanup_candidate,
            issue,
            byte_len: artifact.byte_len,
            media_type: artifact.media_type,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminManagedArtworkArtifactStorageDriftArtifactIssue {
    MissingFile,
    UnresolvableExpectedPath,
    MetadataReadFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStorageDriftFile {
    pub reason: AdminManagedArtworkArtifactStorageDriftFileReason,
    pub recognized_artifact_id: Option<ManagedArtworkArtifactId>,
    pub extension: Option<String>,
    pub byte_len: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminManagedArtworkArtifactStorageDriftFileReason {
    UntrackedArtifactFile,
    UnexpectedActiveArtifactPath,
    UnsupportedExtension,
    UnrecognizedLayout,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactRemediationPlanResponse {
    pub summary: AdminManagedArtworkArtifactRemediationSummary,
    pub missing_artifacts: Vec<AdminManagedArtworkArtifactRemediationMissingArtifact>,
    pub stray_files: Vec<AdminManagedArtworkArtifactRemediationStrayFile>,
    pub page: PageInfo,
    pub dry_run: bool,
}

impl AdminManagedArtworkArtifactRemediationPlanResponse {
    #[must_use]
    pub fn new(
        summary: AdminManagedArtworkArtifactRemediationSummary,
        missing_artifacts: Vec<AdminManagedArtworkArtifactRemediationMissingArtifact>,
        stray_files: Vec<AdminManagedArtworkArtifactRemediationStrayFile>,
        page: PageInfo,
    ) -> Self {
        Self {
            summary,
            missing_artifacts,
            stray_files,
            page,
            dry_run: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactRemediationSummary {
    pub scanned_db_artifacts: u32,
    pub missing_db_backed_artifacts: u32,
    pub selected_missing_artifacts: u32,
    pub cleanup_candidate_missing_artifacts: u32,
    pub file_scan_limit: u32,
    pub scanned_files: u32,
    pub cleanable_stray_files: u32,
    pub blocked_stray_files: u32,
    pub file_scan_truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactRemediationMissingArtifact {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub selected_artwork_count: u32,
    pub cleanup_candidate: bool,
    pub issue: AdminManagedArtworkArtifactStorageDriftArtifactIssue,
    pub recommendation: AdminManagedArtworkArtifactMissingRemediationRecommendation,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
}

impl AdminManagedArtworkArtifactRemediationMissingArtifact {
    #[must_use]
    pub fn from_storage_drift(artifact: AdminManagedArtworkArtifactStorageDriftArtifact) -> Self {
        let recommendation = if artifact.selected_artwork_count > 0 {
            AdminManagedArtworkArtifactMissingRemediationRecommendation::RestoreOrRepublishSelectedArtwork
        } else {
            AdminManagedArtworkArtifactMissingRemediationRecommendation::RunArtifactCleanupOrReingest
        };
        Self {
            id: artifact.id,
            ingest_id: artifact.ingest_id,
            library_id: artifact.library_id,
            item_id: artifact.item_id,
            kind: artifact.kind,
            selected_artwork_count: artifact.selected_artwork_count,
            cleanup_candidate: artifact.cleanup_candidate,
            issue: artifact.issue,
            recommendation,
            byte_len: artifact.byte_len,
            media_type: artifact.media_type,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminManagedArtworkArtifactMissingRemediationRecommendation {
    RestoreOrRepublishSelectedArtwork,
    RunArtifactCleanupOrReingest,
    InspectStorageConfiguration,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactRemediationStrayFile {
    pub reason: AdminManagedArtworkArtifactStorageDriftFileReason,
    pub action: AdminManagedArtworkArtifactStrayFileRemediationAction,
    pub recognized_artifact_id: Option<ManagedArtworkArtifactId>,
    pub extension: Option<String>,
    pub byte_len: Option<u64>,
}

impl AdminManagedArtworkArtifactRemediationStrayFile {
    #[must_use]
    pub fn from_storage_drift(file: AdminManagedArtworkArtifactStorageDriftFile) -> Self {
        let action = if file.reason
            == AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile
            && file.recognized_artifact_id.is_some()
            && file.byte_len.is_some()
        {
            AdminManagedArtworkArtifactStrayFileRemediationAction::DeleteStrayFile
        } else {
            AdminManagedArtworkArtifactStrayFileRemediationAction::InspectManually
        };
        Self {
            reason: file.reason,
            action,
            recognized_artifact_id: file.recognized_artifact_id,
            extension: file.extension,
            byte_len: file.byte_len,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminManagedArtworkArtifactStrayFileRemediationAction {
    DeleteStrayFile,
    InspectManually,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStrayFileCleanupResponse {
    pub summary: AdminManagedArtworkArtifactStrayFileCleanupSummary,
    pub cleaned_files: Vec<AdminManagedArtworkArtifactStrayFileCleanupItem>,
    pub blocked_files: Vec<AdminManagedArtworkArtifactRemediationStrayFile>,
    pub dry_run: bool,
}

impl AdminManagedArtworkArtifactStrayFileCleanupResponse {
    #[must_use]
    pub fn new(
        summary: AdminManagedArtworkArtifactStrayFileCleanupSummary,
        cleaned_files: Vec<AdminManagedArtworkArtifactStrayFileCleanupItem>,
        blocked_files: Vec<AdminManagedArtworkArtifactRemediationStrayFile>,
    ) -> Self {
        Self {
            summary,
            cleaned_files,
            blocked_files,
            dry_run: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStrayFileCleanupSummary {
    pub file_scan_limit: u32,
    pub scanned_files: u32,
    pub cleanable_stray_files: u32,
    pub blocked_stray_files: u32,
    pub deleted_files: u32,
    pub missing_files: u32,
    pub failed_files: u32,
    pub file_scan_truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkArtifactStrayFileCleanupItem {
    pub recognized_artifact_id: ManagedArtworkArtifactId,
    pub extension: Option<String>,
    pub byte_len: Option<u64>,
    pub status: AdminManagedArtworkArtifactStrayFileCleanupStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminManagedArtworkArtifactStrayFileCleanupStatus {
    Deleted,
    Missing,
    Failed,
}

impl SelectedArtworkSummary {
    #[must_use]
    pub fn from_record(record: SelectedArtworkRecord) -> Self {
        Self {
            id: record.id,
            library_id: record.library_id,
            item_id: record.item_id,
            kind: record.kind,
            artifact_id: record.artifact_id,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkGalleryResponse {
    pub item_id: MediaItemId,
    pub summary: AdminManagedArtworkGallerySummary,
    pub candidates: Vec<AdminManagedArtworkGalleryCandidate>,
    pub artifacts: Vec<AdminManagedArtworkGalleryArtifact>,
    pub selected: Vec<AdminManagedArtworkGallerySelected>,
    pub page: PageInfo,
}

impl AdminManagedArtworkGalleryResponse {
    #[must_use]
    pub fn from_snapshot(snapshot: ManagedArtworkGallerySnapshot, page: PageInfo) -> Self {
        Self {
            item_id: snapshot.item_id,
            summary: AdminManagedArtworkGallerySummary {
                candidates: snapshot.summary.candidates,
                artifacts: snapshot.summary.artifacts,
                selected: snapshot.summary.selected,
            },
            candidates: snapshot
                .candidates
                .into_iter()
                .map(AdminManagedArtworkGalleryCandidate::from_record)
                .collect(),
            artifacts: snapshot
                .artifacts
                .into_iter()
                .map(AdminManagedArtworkGalleryArtifact::from_record)
                .collect(),
            selected: snapshot
                .selected
                .into_iter()
                .map(AdminManagedArtworkGallerySelected::from_record)
                .collect(),
            page,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkGallerySummary {
    pub candidates: u32,
    pub artifacts: u32,
    pub selected: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkGalleryCandidate {
    pub id: ArtworkCandidateId,
    pub addon_id: AddonId,
    pub side_effect_id: AddonSideEffectId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub source_kind: ArtworkCandidateSourceKind,
    pub status: ArtworkCandidateStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub ingest: Option<ManagedArtworkIngestSummary>,
    pub artifact_id: Option<ManagedArtworkArtifactId>,
    pub has_stored_artifact: bool,
    pub selected_artwork_count: u32,
    pub selected: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminManagedArtworkGalleryCandidate {
    #[must_use]
    pub fn from_record(record: ManagedArtworkGalleryCandidateRecord) -> Self {
        let has_stored_artifact = record.has_stored_artifact();
        let selected = record.selected();
        Self {
            id: record.id,
            addon_id: record.addon_id,
            side_effect_id: record.side_effect_id,
            library_id: record.library_id,
            item_id: record.item_id,
            kind: record.kind,
            source_kind: record.source_kind,
            status: record.status,
            width: record.width,
            height: record.height,
            language: record.language,
            ingest: record.ingest.map(ManagedArtworkIngestSummary::from_record),
            artifact_id: record.artifact_id,
            has_stored_artifact,
            selected_artwork_count: record.selected_artwork_count,
            selected,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkGalleryArtifact {
    pub id: ManagedArtworkArtifactId,
    pub ingest_id: ManagedArtworkIngestId,
    pub candidate_id: ArtworkCandidateId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub selected_artwork_count: u32,
    pub selected: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_len: Option<u64>,
    pub media_type: Option<String>,
    pub has_content_hash: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminManagedArtworkGalleryArtifact {
    #[must_use]
    pub fn from_record(record: ManagedArtworkGalleryArtifactRecord) -> Self {
        let selected = record.selected();
        Self {
            id: record.id,
            ingest_id: record.ingest_id,
            candidate_id: record.candidate_id,
            library_id: record.library_id,
            item_id: record.item_id,
            kind: record.kind,
            selected_artwork_count: record.selected_artwork_count,
            selected,
            width: record.width,
            height: record.height,
            byte_len: record.byte_len,
            media_type: record.media_type,
            has_content_hash: record.has_content_hash,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminManagedArtworkGallerySelected {
    pub selected_artwork: SelectedArtworkSummary,
    pub artifact: AdminManagedArtworkGalleryArtifact,
    pub image: PublicImageRefDto,
}

impl AdminManagedArtworkGallerySelected {
    #[must_use]
    pub fn from_record(record: ManagedArtworkGallerySelectedRecord) -> Self {
        let image = gallery_selected_artwork_to_public_image_ref(&record);
        Self {
            selected_artwork: SelectedArtworkSummary::from_record(record.selected_artwork),
            artifact: AdminManagedArtworkGalleryArtifact::from_record(record.artifact),
            image,
        }
    }
}

fn gallery_selected_artwork_to_public_image_ref(
    record: &ManagedArtworkGallerySelectedRecord,
) -> PublicImageRefDto {
    let selected = &record.selected_artwork;
    let artifact = &record.artifact;
    PublicImageRefDto {
        id: selected.id.to_string(),
        owner: nako_client_protocol::ClientImageOwner::Item(selected.item_id.to_string()),
        kind: crate::public_client::image_kind_to_dto(selected.kind.clone()),
        url: format!("/images/{}", selected.id),
        width: artifact.width,
        height: artifact.height,
        language: None,
        media_type: artifact.media_type.clone(),
        etag: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nako_core::JobPriority;

    #[test]
    fn managed_artwork_processing_response_redacts_storage_uri() {
        let ingest_id = ManagedArtworkIngestId::new();
        let artifact = ManagedArtworkArtifactRecord {
            id: ManagedArtworkArtifactId::new(),
            ingest_id,
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            kind: ImageKind::Poster,
            storage_uri: "managed-artwork://artifact/private-storage-handle".to_owned(),
            content_hash: Some("sha256-demo".to_owned()),
            width: Some(1),
            height: Some(1),
            byte_len: Some(68),
            media_type: Some("image/png".to_owned()),
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };

        let summary = ManagedArtworkArtifactSummary::from_record(artifact);
        let body = serde_json::to_string(&summary).unwrap();

        assert_eq!(summary.ingest_id, ingest_id);
        assert_eq!(summary.media_type.as_deref(), Some("image/png"));
        assert!(summary.has_content_hash);
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("private-storage-handle"));
        assert!(!body.contains("sha256-demo"));
        assert!(!body.contains("\"content_hash\""));
    }

    #[test]
    fn managed_artwork_ingest_requeue_response_redacts_job_payloads_errors_and_raw_locators() {
        let ingest_id = ManagedArtworkIngestId::new();
        let candidate_id = ArtworkCandidateId::new();
        let job_id = JobId::new();
        let response =
            RequeueManagedArtworkIngestResponse::from_requeue(ManagedArtworkIngestRequeueRecord {
                ingest: ManagedArtworkIngestRecord {
                    id: ingest_id,
                    candidate_id,
                    job_id,
                    library_id: LibraryId::new(),
                    item_id: MediaItemId::new(),
                    kind: ImageKind::Poster,
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                    created_at: "2026-05-19T00:00:00Z".to_owned(),
                    updated_at: "2026-05-19T00:00:01Z".to_owned(),
                },
                job: Job {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    status: JobStatus::Queued,
                    resource_class: "artwork.ingest".to_owned(),
                    priority: JobPriority::Normal,
                    library_id: Some(LibraryId::new()),
                    source_id: None,
                    input_json: Some(
                        r#"{"source_uri":"https://cdn.example.test/poster.png?token=secret"}"#
                            .to_owned(),
                    ),
                    summary_json: None,
                    error: None,
                    attempt: 1,
                    max_attempts: 1,
                    retry_of_job_id: None,
                    next_attempt_at: None,
                    queued_at: "2026-05-19T00:00:00Z".to_owned(),
                    started_at: None,
                    completed_at: None,
                },
                requeued: true,
                had_failure: true,
            });
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.requeued);
        assert!(response.had_failure);
        assert_eq!(response.ingest.id, ingest_id);
        assert_eq!(response.job.id, job_id);
        assert_eq!(response.job.status, JobStatus::Queued);
        assert!(response.job.has_input);
        assert!(!response.job.has_summary);
        assert!(!response.job.has_error);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("token=secret"));
        assert!(!body.contains("cdn.example.test"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
        assert!(!body.contains("error\":\""));
    }

    #[test]
    fn selected_artwork_publication_response_redacts_storage_uri() {
        managed_artwork_variant_publication_response_redacts_storage_uri_and_hash();
    }

    #[test]
    fn managed_artwork_variant_publication_response_redacts_storage_uri_and_hash() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let selected = SelectedArtworkRecord {
            id: nako_core::SelectedArtworkId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            kind: ImageKind::Poster,
            artifact_id,
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };
        let artifact = ManagedArtworkArtifactRecord {
            id: artifact_id,
            ingest_id: ManagedArtworkIngestId::new(),
            library_id: selected.library_id,
            item_id: selected.item_id,
            kind: selected.kind.clone(),
            storage_uri: "managed-artwork://artifact/private-storage-handle".to_owned(),
            content_hash: Some("sha256-public-etag".to_owned()),
            width: Some(1),
            height: Some(1),
            byte_len: Some(68),
            media_type: Some("image/png".to_owned()),
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };

        let response =
            PublishSelectedArtworkResponse::from_publication(SelectedArtworkPublicationRecord {
                selected_artwork: selected,
                artifact,
                changed: true,
            });
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.changed);
        assert!(response.image.url.starts_with("/images/"));
        assert_eq!(response.image.media_type.as_deref(), Some("image/png"));
        assert_eq!(response.image.etag, None);
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("private-storage-handle"));
        assert!(!body.contains("sha256-public-etag"));
        assert!(!body.contains("\"content_hash\""));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
    }

    #[test]
    fn selected_artwork_unpublish_response_redacts_storage_uri_and_hash() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let selected = SelectedArtworkRecord {
            id: nako_core::SelectedArtworkId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            kind: ImageKind::Poster,
            artifact_id,
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };
        let artifact = ManagedArtworkArtifactRecord {
            id: artifact_id,
            ingest_id: ManagedArtworkIngestId::new(),
            library_id: selected.library_id,
            item_id: selected.item_id,
            kind: selected.kind.clone(),
            storage_uri: "managed-artwork://artifact/private-storage-handle".to_owned(),
            content_hash: Some("sha256-unpublish-etag".to_owned()),
            width: Some(1),
            height: Some(1),
            byte_len: Some(68),
            media_type: Some("image/png".to_owned()),
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };

        let response = UnpublishSelectedArtworkResponse::from_unpublication(
            SelectedArtworkUnpublicationRecord {
                item_id: selected.item_id,
                kind: selected.kind.clone(),
                unpublished: Some(selected),
                artifact: Some(artifact),
                changed: true,
            },
        );
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.changed);
        assert!(response.unpublished.is_some());
        assert!(
            response
                .unpublished
                .as_ref()
                .unwrap()
                .previous_image
                .url
                .starts_with("/images/")
        );
        assert_eq!(
            response
                .unpublished
                .as_ref()
                .unwrap()
                .previous_image
                .media_type
                .as_deref(),
            Some("image/png")
        );
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("private-storage-handle"));
        assert!(!body.contains("sha256-unpublish-etag"));
        assert!(!body.contains("\"content_hash\""));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
    }

    #[test]
    fn managed_artwork_gallery_response_redacts_candidate_source_storage_and_hash_values() {
        let candidate_id = ArtworkCandidateId::new();
        let artifact_id = ManagedArtworkArtifactId::new();
        let ingest_id = ManagedArtworkIngestId::new();
        let selected = SelectedArtworkRecord {
            id: nako_core::SelectedArtworkId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            kind: ImageKind::Poster,
            artifact_id,
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };
        let artifact = ManagedArtworkGalleryArtifactRecord {
            id: artifact_id,
            ingest_id,
            candidate_id,
            library_id: selected.library_id,
            item_id: selected.item_id,
            kind: ImageKind::Poster,
            width: Some(1000),
            height: Some(1500),
            byte_len: Some(68),
            media_type: Some("image/png".to_owned()),
            has_content_hash: true,
            selected_artwork_count: 1,
            created_at: "2026-05-19T00:00:00Z".to_owned(),
            updated_at: "2026-05-19T00:00:00Z".to_owned(),
        };
        let response = AdminManagedArtworkGalleryResponse::from_snapshot(
            ManagedArtworkGallerySnapshot {
                item_id: selected.item_id,
                summary: nako_core::ManagedArtworkGallerySummary {
                    candidates: 1,
                    artifacts: 1,
                    selected: 1,
                },
                candidates: vec![ManagedArtworkGalleryCandidateRecord {
                    id: candidate_id,
                    addon_id: AddonId::new(),
                    side_effect_id: AddonSideEffectId::new(),
                    library_id: selected.library_id,
                    item_id: selected.item_id,
                    kind: ImageKind::Poster,
                    source_kind: ArtworkCandidateSourceKind::RemoteUrl,
                    width: Some(1000),
                    height: Some(1500),
                    language: Some("en".to_owned()),
                    status: ArtworkCandidateStatus::Accepted,
                    ingest: Some(ManagedArtworkIngestRecord {
                        id: ingest_id,
                        candidate_id,
                        job_id: JobId::new(),
                        library_id: selected.library_id,
                        item_id: selected.item_id,
                        kind: ImageKind::Poster,
                        status: ManagedArtworkIngestStatus::Stored,
                        artifact_id: Some(artifact_id),
                        failure_code: None,
                        created_at: "2026-05-19T00:00:00Z".to_owned(),
                        updated_at: "2026-05-19T00:00:00Z".to_owned(),
                    }),
                    artifact_id: Some(artifact_id),
                    selected_artwork_count: 1,
                    created_at: "2026-05-19T00:00:00Z".to_owned(),
                    updated_at: "2026-05-19T00:00:00Z".to_owned(),
                }],
                artifacts: vec![artifact.clone()],
                selected: vec![ManagedArtworkGallerySelectedRecord {
                    selected_artwork: selected,
                    artifact,
                }],
            },
            PageInfo::new(50, 0, 1),
        );
        let body = serde_json::to_string(&response).unwrap();

        assert_eq!(response.summary.candidates, 1);
        assert_eq!(response.candidates[0].artifact_id, Some(artifact_id));
        assert!(response.candidates[0].has_stored_artifact);
        assert!(response.candidates[0].selected);
        assert!(response.artifacts[0].selected);
        assert!(response.artifacts[0].has_content_hash);
        assert_eq!(
            response.selected[0].image.url,
            format!("/images/{}", response.selected[0].selected_artwork.id)
        );
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
        assert!(!body.contains("https://cdn.example.test"));
        assert!(!body.contains("token=secret"));
        assert!(!body.contains("sha256-private-content-hash"));
        assert!(!body.contains("\"content_hash\""));
    }

    #[test]
    fn managed_artwork_lifecycle_response_redacts_storage_authority_and_hash_values() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let ingest_id = ManagedArtworkIngestId::new();
        let lifecycle = AdminManagedArtworkArtifactLifecycleResponse::from_snapshot(
            ManagedArtworkArtifactLifecycleSnapshot {
                summary: ManagedArtworkArtifactLifecycleSummary {
                    total_artifacts: 1,
                    protected_artifacts: 0,
                    cleanup_candidate_artifacts: 1,
                    known_total_bytes: 68,
                    known_protected_bytes: 0,
                    known_cleanup_candidate_bytes: 68,
                    unknown_byte_len_artifacts: 0,
                },
                artifacts: vec![ManagedArtworkArtifactLifecycleRecord {
                    artifact: ManagedArtworkArtifactRecord {
                        id: artifact_id,
                        ingest_id,
                        library_id: LibraryId::new(),
                        item_id: MediaItemId::new(),
                        kind: ImageKind::Poster,
                        storage_uri: "managed-artwork://artifact/private-storage-handle".to_owned(),
                        content_hash: Some("sha256-private-content-hash".to_owned()),
                        width: Some(1),
                        height: Some(1),
                        byte_len: Some(68),
                        media_type: Some("image/png".to_owned()),
                        created_at: "2026-05-19T00:00:00Z".to_owned(),
                        updated_at: "2026-05-19T00:00:00Z".to_owned(),
                    },
                    selected_artwork_count: 0,
                }],
            },
            PageInfo::new(50, 0, 1),
        );
        let body = serde_json::to_string(&lifecycle).unwrap();

        assert!(lifecycle.dry_run);
        assert_eq!(lifecycle.summary.cleanup_candidate_artifacts, 1);
        assert!(lifecycle.artifacts[0].cleanup_candidate);
        assert!(lifecycle.artifacts[0].has_content_hash);
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("private-storage-handle"));
        assert!(!body.contains("sha256-private-content-hash"));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
    }

    #[test]
    fn managed_artwork_cleanup_response_redacts_storage_authority_and_hash_values() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let response = AdminManagedArtworkArtifactCleanupResponse::from_report(
            ManagedArtworkArtifactCleanupReport {
                examined_artifacts: 1,
                cleanup_candidate_artifacts: 1,
                cleaned_artifacts: vec![ManagedArtworkArtifactRecord {
                    id: artifact_id,
                    ingest_id: ManagedArtworkIngestId::new(),
                    library_id: LibraryId::new(),
                    item_id: MediaItemId::new(),
                    kind: ImageKind::Poster,
                    storage_uri: "managed-artwork://artifact/private-storage-handle".to_owned(),
                    content_hash: Some("sha256-private-cleanup-hash".to_owned()),
                    width: Some(1),
                    height: Some(1),
                    byte_len: Some(68),
                    media_type: Some("image/png".to_owned()),
                    created_at: "2026-05-19T00:00:00Z".to_owned(),
                    updated_at: "2026-05-19T00:00:00Z".to_owned(),
                }],
            },
            AdminManagedArtworkArtifactFileCleanupSummary {
                file_deleted_artifacts: 1,
                file_missing_artifacts: 0,
                file_delete_failed_artifacts: 0,
            },
        );
        let body = serde_json::to_string(&response).unwrap();

        assert!(!response.dry_run);
        assert_eq!(response.cleaned_artifacts[0].id, artifact_id);
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("private-storage-handle"));
        assert!(!body.contains("sha256-private-cleanup-hash"));
        assert!(!body.contains("\"content_hash\""));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
    }

    #[test]
    fn managed_artwork_storage_drift_response_redacts_storage_authority_and_paths() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let ingest_id = ManagedArtworkIngestId::new();
        let response = AdminManagedArtworkArtifactStorageDriftResponse::new(
            AdminManagedArtworkArtifactStorageDriftSummary {
                scanned_db_artifacts: 1,
                db_backed_present_artifacts: 0,
                db_backed_missing_artifacts: 1,
                db_backed_unresolvable_artifacts: 0,
                db_backed_metadata_read_failed_artifacts: 0,
                file_scan_limit: 50,
                scanned_files: 1,
                stray_files: 1,
                untracked_artifact_files: 1,
                unexpected_active_artifact_files: 0,
                unsupported_extension_files: 0,
                unrecognized_layout_files: 0,
                file_scan_truncated: false,
            },
            vec![AdminManagedArtworkArtifactStorageDriftArtifact {
                id: artifact_id,
                ingest_id,
                library_id: LibraryId::new(),
                item_id: MediaItemId::new(),
                kind: ImageKind::Poster,
                selected_artwork_count: 1,
                cleanup_candidate: false,
                issue: AdminManagedArtworkArtifactStorageDriftArtifactIssue::MissingFile,
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            }],
            vec![AdminManagedArtworkArtifactStorageDriftFile {
                reason: AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile,
                recognized_artifact_id: Some(ManagedArtworkArtifactId::new()),
                extension: Some("png".to_owned()),
                byte_len: Some(7),
            }],
            PageInfo::new(50, 0, 1),
        );
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.dry_run);
        assert_eq!(response.missing_artifacts[0].id, artifact_id);
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("\"content_hash\""));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
        assert!(!body.contains("private-path"));
        assert!(!body.contains("artifact_root"));
    }

    #[test]
    fn managed_artwork_remediation_responses_redact_storage_authority_and_paths() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let stray_artifact_id = ManagedArtworkArtifactId::new();
        let missing = AdminManagedArtworkArtifactRemediationMissingArtifact::from_storage_drift(
            AdminManagedArtworkArtifactStorageDriftArtifact {
                id: artifact_id,
                ingest_id: ManagedArtworkIngestId::new(),
                library_id: LibraryId::new(),
                item_id: MediaItemId::new(),
                kind: ImageKind::Poster,
                selected_artwork_count: 1,
                cleanup_candidate: false,
                issue: AdminManagedArtworkArtifactStorageDriftArtifactIssue::MissingFile,
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
        );
        let plan = AdminManagedArtworkArtifactRemediationPlanResponse::new(
            AdminManagedArtworkArtifactRemediationSummary {
                scanned_db_artifacts: 1,
                missing_db_backed_artifacts: 1,
                selected_missing_artifacts: 1,
                cleanup_candidate_missing_artifacts: 0,
                file_scan_limit: 50,
                scanned_files: 1,
                cleanable_stray_files: 1,
                blocked_stray_files: 0,
                file_scan_truncated: false,
            },
            vec![missing],
            vec![
                AdminManagedArtworkArtifactRemediationStrayFile::from_storage_drift(
                    AdminManagedArtworkArtifactStorageDriftFile {
                        reason:
                            AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile,
                        recognized_artifact_id: Some(stray_artifact_id),
                        extension: Some("png".to_owned()),
                        byte_len: Some(7),
                    },
                ),
            ],
            PageInfo::new(50, 0, 1),
        );
        let cleanup = AdminManagedArtworkArtifactStrayFileCleanupResponse::new(
            AdminManagedArtworkArtifactStrayFileCleanupSummary {
                file_scan_limit: 50,
                scanned_files: 1,
                cleanable_stray_files: 1,
                blocked_stray_files: 0,
                deleted_files: 1,
                missing_files: 0,
                failed_files: 0,
                file_scan_truncated: false,
            },
            vec![AdminManagedArtworkArtifactStrayFileCleanupItem {
                recognized_artifact_id: stray_artifact_id,
                extension: Some("png".to_owned()),
                byte_len: Some(7),
                status: AdminManagedArtworkArtifactStrayFileCleanupStatus::Deleted,
            }],
            Vec::new(),
        );
        let plan_body = serde_json::to_string(&plan).unwrap();
        let cleanup_body = serde_json::to_string(&cleanup).unwrap();

        assert!(plan.dry_run);
        assert!(!cleanup.dry_run);
        assert_eq!(
            plan.missing_artifacts[0].recommendation,
            AdminManagedArtworkArtifactMissingRemediationRecommendation::RestoreOrRepublishSelectedArtwork
        );
        assert_eq!(
            plan.stray_files[0].action,
            AdminManagedArtworkArtifactStrayFileRemediationAction::DeleteStrayFile
        );
        for body in [plan_body, cleanup_body] {
            assert!(!body.contains("storage_uri"));
            assert!(!body.contains("managed-artwork://"));
            assert!(!body.contains("\"content_hash\""));
            assert!(!body.contains("source_uri"));
            assert!(!body.contains("cache_uri"));
            assert!(!body.contains("artifact_root"));
            assert!(!body.contains("private-path"));
        }
    }
}
