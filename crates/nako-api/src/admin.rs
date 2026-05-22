use nako_client_protocol::PageInfo;
use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateState, AutomationArtifactId,
    AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability, AutomationProviderId,
    CatalogGovernanceItemRecord, DomainEventKind, DomainEventSubject, EventId, ExternalProvider,
    GeneratedArtifactAcceptanceActionKind, GeneratedArtifactAcceptanceBoundary,
    GeneratedArtifactAcceptancePlan, GeneratedArtifactAcceptancePlanReason,
    GeneratedArtifactAcceptancePlanStatus, GeneratedArtifactPayloadShape,
    GeneratedArtifactPayloadSummary, GeneratedArtifactProposal, GeneratedArtifactProvenance,
    GeneratedArtifactReadiness, GeneratedArtifactReadinessReason, GeneratedArtifactReadinessStatus,
    GeneratedArtifactReviewDecision, GeneratedArtifactReviewResult, GeneratedArtifactTarget,
    GeneratedArtifactTargetKind, IngestionFailureClass, IngestionFailurePhase,
    IngestionFailureRecord, IngestionFailureStatus, Job, JobCancellationRequestRecord, JobId,
    JobKind, JobStatus, LibraryId, LibraryPreset, LocalInferenceEvidence,
    LocalInferenceEvidenceSource, ManagedImportArtifactId, MediaItemId, MediaKind, MediaSourceId,
    OutboxEventRecord, OutboxEventStatus, ScanSnapshotId, StagingManifestId, StagingManifestRecord,
    StagingPurpose, StagingState, TranscodeFailureCategory, TranscodeSessionId,
    TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionState,
};
use nako_transcode::{
    HardwareAcceleration, HardwareAccelerationPolicy, HardwareAccelerationReadiness,
    HardwareAccelerationReadinessReason, HardwareAccelerationReadinessStatus,
    HardwareAccelerationSelection,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::metadata_diagnostics::MetadataProviderDiagnosticStatus;

pub const ADMIN_API_VERSION: &str = "v1";

mod managed_artwork;
pub use managed_artwork::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceItemListResponse {
    pub items: Vec<AdminCatalogGovernanceItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceItem {
    pub item_id: MediaItemId,
    pub library_id: LibraryId,
    pub kind: MediaKind,
    pub parent_id: Option<MediaItemId>,
    pub title: String,
    pub release_date: Option<String>,
    pub source_count: u32,
    pub representative_source_id: Option<MediaSourceId>,
    pub representative_file_name: Option<String>,
    pub local_inference: Option<AdminLocalInferenceSummary>,
    pub provider_mapping_count: u32,
    pub accepted_provider_mapping_count: u32,
    pub duplicate_relationship_count: u32,
    pub issues: Vec<AdminCatalogGovernanceIssue>,
}

impl AdminCatalogGovernanceItem {
    #[must_use]
    pub fn from_record(
        record: CatalogGovernanceItemRecord,
        low_confidence_threshold_milli: u16,
    ) -> Self {
        let local_inference = record
            .best_local_inference
            .map(AdminLocalInferenceSummary::from_evidence);
        let mut issues = Vec::new();

        if record.item.kind == MediaKind::Unknown {
            issues.push(AdminCatalogGovernanceIssue::UnknownKind);
        }
        if local_inference
            .as_ref()
            .and_then(|inference| inference.confidence_milli)
            .is_some_and(|confidence| confidence <= low_confidence_threshold_milli)
        {
            issues.push(AdminCatalogGovernanceIssue::LowLocalInferenceConfidence);
        }
        if record.accepted_provider_mapping_count == 0 {
            issues.push(AdminCatalogGovernanceIssue::MissingAcceptedProviderMapping);
        }

        Self {
            item_id: record.item.id,
            library_id: record.library_id,
            kind: record.item.kind,
            parent_id: record.item.parent_id,
            title: record.item.metadata.title,
            release_date: record.item.metadata.release_date,
            source_count: record.source_count,
            representative_source_id: record.representative_source_id,
            representative_file_name: record.representative_file_name,
            local_inference,
            provider_mapping_count: record.provider_mapping_count,
            accepted_provider_mapping_count: record.accepted_provider_mapping_count,
            duplicate_relationship_count: record.duplicate_relationship_count,
            issues,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLocalInferenceSummary {
    pub source_id: MediaSourceId,
    pub inferred_kind: MediaKind,
    pub inferred_title: Option<String>,
    pub inferred_year: Option<i32>,
    pub inferred_season: Option<u32>,
    pub inferred_episode: Option<u32>,
    pub confidence_milli: Option<u16>,
    pub evidence_source: LocalInferenceEvidenceSource,
    pub has_evidence: bool,
    pub inference_version: String,
}

impl AdminLocalInferenceSummary {
    #[must_use]
    pub fn from_evidence(evidence: LocalInferenceEvidence) -> Self {
        Self {
            source_id: evidence.source_id,
            inferred_kind: evidence.inferred_kind,
            inferred_title: evidence.inferred_title,
            inferred_year: evidence.inferred_year,
            inferred_season: evidence.inferred_season,
            inferred_episode: evidence.inferred_episode,
            confidence_milli: evidence.confidence_milli,
            evidence_source: evidence.evidence_source,
            has_evidence: !evidence.evidence_value.trim().is_empty(),
            inference_version: evidence.inference_version,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCatalogGovernanceIssue {
    UnknownKind,
    LowLocalInferenceConfidence,
    MissingAcceptedProviderMapping,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobResponse {
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

impl JobResponse {
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
pub struct AdminJobListResponse {
    pub jobs: Vec<AdminJobListItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminJobCancelRequestResponse {
    pub job: AdminJobListItem,
    pub requested: bool,
    pub terminal: bool,
    pub cancel_requested_at: Option<String>,
}

impl AdminJobCancelRequestResponse {
    #[must_use]
    pub fn from_record(record: JobCancellationRequestRecord) -> Self {
        Self {
            job: AdminJobListItem::from_job(record.job),
            requested: record.requested,
            terminal: record.terminal,
            cancel_requested_at: record.cancel_requested_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminJobListItem {
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

impl AdminJobListItem {
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
pub struct AdminOutboxEventListResponse {
    pub events: Vec<AdminOutboxEventListItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOutboxEventListItem {
    pub id: EventId,
    pub kind: DomainEventKind,
    pub subject: DomainEventSubject,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub status: OutboxEventStatus,
    pub attempts: u32,
    pub has_payload: bool,
    pub has_error: bool,
    pub occurred_at: String,
    pub updated_at: String,
    pub next_attempt_at: Option<String>,
}

impl AdminOutboxEventListItem {
    #[must_use]
    pub fn from_record(event: OutboxEventRecord) -> Self {
        Self {
            id: event.id,
            kind: event.kind,
            subject: event.subject,
            library_id: event.library_id,
            source_id: event.source_id,
            status: event.status,
            attempts: event.attempts,
            has_payload: !event.payload_json.trim().is_empty(),
            has_error: event.last_error.is_some(),
            occurred_at: event.occurred_at,
            updated_at: event.updated_at,
            next_attempt_at: event.next_attempt_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSessionListResponse {
    pub sessions: Vec<AdminPlaybackSessionListItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAcquisitionIntakeCandidateListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub candidates: Vec<AdminAcquisitionIntakeCandidateDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAcquisitionIntakeCandidateDiagnostic {
    pub id: AcquisitionIntakeCandidateId,
    pub target_library_id: LibraryId,
    pub source_kind: String,
    pub custom_source_kind: bool,
    pub source_scheme: Option<String>,
    pub source_ref_redacted: String,
    pub source_key_fingerprint: String,
    pub has_display_name: bool,
    pub has_intended_locator: bool,
    pub size_bytes: Option<u64>,
    pub has_fingerprint: bool,
    pub managed_import_artifact_id: Option<ManagedImportArtifactId>,
    pub state: AcquisitionIntakeCandidateState,
    pub has_diagnostics: bool,
    pub first_seen_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactProposalListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub proposals: Vec<AdminGeneratedArtifactProposal>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactProposal {
    pub id: AutomationArtifactId,
    pub kind: AutomationArtifactKind,
    pub capability: AutomationCapability,
    pub status: AutomationArtifactStatus,
    pub target: AdminGeneratedArtifactTarget,
    pub provenance: AdminGeneratedArtifactProvenance,
    pub payload: AdminGeneratedArtifactPayloadSummary,
    pub readiness: AdminGeneratedArtifactReadiness,
    pub created_at: String,
    pub updated_at: String,
    pub accepted_at: Option<String>,
}

impl AdminGeneratedArtifactProposal {
    #[must_use]
    pub fn from_proposal(proposal: GeneratedArtifactProposal) -> Self {
        Self {
            id: proposal.id,
            kind: proposal.kind,
            capability: proposal.capability,
            status: proposal.status,
            target: AdminGeneratedArtifactTarget::from_target(proposal.target),
            provenance: AdminGeneratedArtifactProvenance::from_provenance(proposal.provenance),
            payload: AdminGeneratedArtifactPayloadSummary::from_summary(proposal.payload),
            readiness: AdminGeneratedArtifactReadiness::from_readiness(proposal.readiness),
            created_at: proposal.created_at,
            updated_at: proposal.updated_at,
            accepted_at: proposal.accepted_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactTarget {
    pub kind: GeneratedArtifactTargetKind,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
}

impl AdminGeneratedArtifactTarget {
    #[must_use]
    pub const fn from_target(target: GeneratedArtifactTarget) -> Self {
        Self {
            kind: target.kind,
            library_id: target.library_id,
            item_id: target.item_id,
            source_id: target.source_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactProvenance {
    pub provider_id: AutomationProviderId,
    pub provider_name: Option<String>,
    pub job_id: JobId,
    pub capability: AutomationCapability,
    pub idempotency_key_fingerprint: Option<String>,
    pub prompt_fingerprint: Option<String>,
    pub attempt_count: Option<u32>,
    pub artifact_created_at: String,
}

impl AdminGeneratedArtifactProvenance {
    #[must_use]
    pub fn from_provenance(provenance: GeneratedArtifactProvenance) -> Self {
        Self {
            provider_id: provenance.provider_id,
            provider_name: provenance.provider_name,
            job_id: provenance.job_id,
            capability: provenance.capability,
            idempotency_key_fingerprint: provenance.idempotency_key_fingerprint,
            prompt_fingerprint: provenance.prompt_fingerprint,
            attempt_count: provenance.attempt_count,
            artifact_created_at: provenance.artifact_created_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactPayloadSummary {
    pub valid_json: bool,
    pub shape: GeneratedArtifactPayloadShape,
    pub payload_fingerprint: String,
    pub payload_bytes: u64,
    pub object_field_count: Option<u32>,
    pub array_item_count: Option<u32>,
    pub has_textual_values: bool,
    pub has_explanation: bool,
    pub confidence_milli: Option<u16>,
}

impl AdminGeneratedArtifactPayloadSummary {
    #[must_use]
    pub fn from_summary(summary: GeneratedArtifactPayloadSummary) -> Self {
        Self {
            valid_json: summary.valid_json,
            shape: summary.shape,
            payload_fingerprint: summary.payload_fingerprint,
            payload_bytes: summary.payload_bytes,
            object_field_count: summary.object_field_count,
            array_item_count: summary.array_item_count,
            has_textual_values: summary.has_textual_values,
            has_explanation: summary.has_explanation,
            confidence_milli: summary.confidence_milli,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactReadiness {
    pub status: GeneratedArtifactReadinessStatus,
    pub actionable: bool,
    pub reasons: Vec<GeneratedArtifactReadinessReason>,
}

impl AdminGeneratedArtifactReadiness {
    #[must_use]
    pub fn from_readiness(readiness: GeneratedArtifactReadiness) -> Self {
        Self {
            status: readiness.status,
            actionable: readiness.actionable,
            reasons: readiness.reasons,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactReviewRequest {
    pub decision: GeneratedArtifactReviewDecision,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactReviewPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub plan: AdminGeneratedArtifactAcceptancePlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactReviewResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub artifact_id: AutomationArtifactId,
    pub decision: GeneratedArtifactReviewDecision,
    pub artifact_status: AutomationArtifactStatus,
    pub accepted_at: Option<String>,
    pub idempotent_replay: bool,
    pub plan: AdminGeneratedArtifactAcceptancePlan,
}

impl AdminGeneratedArtifactReviewResponse {
    #[must_use]
    pub fn from_result(result: GeneratedArtifactReviewResult) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: crate::public_client::API_VERSION.to_owned(),
            artifact_id: result.artifact_id,
            decision: result.decision,
            artifact_status: result.artifact_status,
            accepted_at: result.accepted_at,
            idempotent_replay: result.idempotent_replay,
            plan: AdminGeneratedArtifactAcceptancePlan::from_plan(result.plan),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactAcceptancePlan {
    pub artifact_id: AutomationArtifactId,
    pub decision: GeneratedArtifactReviewDecision,
    pub status: GeneratedArtifactAcceptancePlanStatus,
    pub action: GeneratedArtifactAcceptanceActionKind,
    pub reasons: Vec<GeneratedArtifactAcceptancePlanReason>,
    pub capability: AutomationCapability,
    pub kind: AutomationArtifactKind,
    pub target: AdminGeneratedArtifactTarget,
    pub payload: AdminGeneratedArtifactPayloadSummary,
    pub readiness: AdminGeneratedArtifactReadiness,
    pub boundary: GeneratedArtifactAcceptanceBoundary,
}

impl AdminGeneratedArtifactAcceptancePlan {
    #[must_use]
    pub fn from_plan(plan: GeneratedArtifactAcceptancePlan) -> Self {
        Self {
            artifact_id: plan.artifact_id,
            decision: plan.decision,
            status: plan.status,
            action: plan.action,
            reasons: plan.reasons,
            capability: plan.capability,
            kind: plan.kind,
            target: AdminGeneratedArtifactTarget::from_target(plan.target),
            payload: AdminGeneratedArtifactPayloadSummary::from_summary(plan.payload),
            readiness: AdminGeneratedArtifactReadiness::from_readiness(plan.readiness),
            boundary: plan.boundary,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderDiscoveryRequest {
    pub target_library_id: LibraryId,
    pub root_uri: Option<String>,
    pub max_depth: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderDiscoveryResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub target_library_id: LibraryId,
    pub root_scheme: Option<String>,
    pub root_ref_redacted: String,
    pub ready_candidates: u64,
    pub blocked_candidates: u64,
    pub incomplete_candidates: u64,
    pub unsupported_candidates: u64,
    pub recorded_candidates: u64,
    pub failures: Vec<AdminWatchFolderDiscoveryFailure>,
    pub writes_library: bool,
    pub managed_import_artifacts_created: bool,
    pub promotion_apply: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderDiscoveryFailure {
    pub ref_redacted: String,
    pub safe_message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSessionListItem {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub request_key: String,
    pub state: TranscodeSessionState,
    pub failure_category: Option<TranscodeFailureCategory>,
    pub has_failure_message: bool,
    pub active: bool,
    pub terminal: bool,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl AdminPlaybackSessionListItem {
    #[must_use]
    pub fn from_record(session: TranscodeSessionRecord) -> Self {
        Self {
            id: session.id,
            source_id: session.source_id,
            kind: session.kind,
            request_key: session.request_key,
            state: session.state,
            failure_category: session.failure_category,
            has_failure_message: session.failure_message.is_some(),
            active: session.state.is_active(),
            terminal: session.state.is_terminal(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            started_at: session.started_at,
            completed_at: session.completed_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRuntimeDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub readiness: AdminPlaybackReadinessDiagnostics,
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackHardwareDiagnostics,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportEvidenceResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub subject: AdminPlaybackSupportSubject,
    pub session: Option<AdminPlaybackSupportSessionEvidence>,
    pub source: Option<AdminPlaybackSupportSourceEvidence>,
    pub runtime: AdminPlaybackSupportRuntimeEvidence,
    pub redaction: AdminPlaybackSupportRedactionEvidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportSubject {
    pub session_id: Option<TranscodeSessionId>,
    pub source_id: Option<MediaSourceId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportSessionEvidence {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub state: TranscodeSessionState,
    pub failure_category: Option<TranscodeFailureCategory>,
    pub has_failure_message: bool,
    pub active: bool,
    pub terminal: bool,
    pub request_key_fingerprint: String,
    pub output_artifact_kind: AdminPlaybackOutputArtifactKind,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl AdminPlaybackSupportSessionEvidence {
    #[must_use]
    pub fn from_record(session: TranscodeSessionRecord) -> Self {
        Self {
            id: session.id,
            source_id: session.source_id,
            kind: session.kind,
            state: session.state,
            failure_category: session.failure_category,
            has_failure_message: session.failure_message.is_some(),
            active: session.state.is_active(),
            terminal: session.state.is_terminal(),
            request_key_fingerprint: stable_fingerprint(&session.request_key),
            output_artifact_kind: AdminPlaybackOutputArtifactKind::from_session_kind(session.kind),
            created_at: session.created_at,
            updated_at: session.updated_at,
            started_at: session.started_at,
            completed_at: session.completed_at,
        }
    }
}

impl AdminPlaybackSupportSourceEvidence {
    #[must_use]
    pub fn from_record(source: nako_core::MediaSource) -> Self {
        Self {
            source_id: source.id,
            library_id: source.library_id,
            item_id: source.item_id,
            source_scheme: storage_scheme(&source.locator),
            file_name: source.file_name,
            size_bytes: source.size_bytes,
            has_fingerprint: source.fingerprint.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackOutputArtifactKind {
    RemuxFile,
    HlsPlaylist,
}

impl AdminPlaybackOutputArtifactKind {
    const fn from_session_kind(kind: TranscodeSessionKind) -> Self {
        match kind {
            TranscodeSessionKind::Remux => Self::RemuxFile,
            TranscodeSessionKind::HlsTranscode => Self::HlsPlaylist,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportSourceEvidence {
    pub source_id: MediaSourceId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub source_scheme: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub has_fingerprint: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportRuntimeEvidence {
    pub readiness: AdminPlaybackReadinessDiagnostics,
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackSupportHardwareEvidence,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportHardwareEvidence {
    pub policy: HardwareAccelerationPolicy,
    pub selected_acceleration: HardwareAcceleration,
    pub fallback_used: bool,
    pub capability_count: u32,
    pub unavailable_capabilities: Vec<AdminPlaybackSupportHardwareCapabilityEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportHardwareCapabilityEvidence {
    pub accelerator: HardwareAcceleration,
    pub reason_code: AdminPlaybackHardwareCapabilityReason,
    pub encoder_discovery_status: AdminPlaybackHardwareEncoderDiscoveryStatus,
    pub device_initialization_status: AdminPlaybackHardwareDeviceInitializationStatus,
    pub smoke_probe_status: AdminPlaybackHardwareSmokeProbeStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportRedactionEvidence {
    pub paths_redacted: bool,
    pub source_references_redacted: bool,
    pub ffmpeg_commands_redacted: bool,
    pub stderr_redacted: bool,
    pub credentials_redacted: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackReadinessDiagnostics {
    pub status: AdminPlaybackReadinessStatus,
    pub reason: AdminPlaybackReadinessReason,
    pub checks: Vec<AdminPlaybackReadinessCheck>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

impl From<HardwareAccelerationReadinessStatus> for AdminPlaybackReadinessStatus {
    fn from(status: HardwareAccelerationReadinessStatus) -> Self {
        match status {
            HardwareAccelerationReadinessStatus::Ready => Self::Ready,
            HardwareAccelerationReadinessStatus::Degraded => Self::Degraded,
            HardwareAccelerationReadinessStatus::Unavailable => Self::Unavailable,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessReason {
    Ready,
    FfmpegProbeReady,
    CpuRequested,
    RequestedAcceleratorReady,
    RequestedAcceleratorUnavailableFallbackToCpu,
    RequestedAcceleratorUnavailableFailPolicy,
    ProbeError,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    SelectedAccelerationReady,
    CpuFallbackActive,
    TranscodeBudgetReady,
    TranscodeBudgetClamped,
    RemotePlaybackBudgetReady,
    RemotePlaybackBudgetClamped,
    StagingReady,
    StagingBudgetDisabled,
}

impl From<HardwareAccelerationReadinessReason> for AdminPlaybackReadinessReason {
    fn from(reason: HardwareAccelerationReadinessReason) -> Self {
        match reason {
            HardwareAccelerationReadinessReason::CpuRequested => Self::CpuRequested,
            HardwareAccelerationReadinessReason::RequestedAcceleratorReady => {
                Self::RequestedAcceleratorReady
            }
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu => {
                Self::RequestedAcceleratorUnavailableFallbackToCpu
            }
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFailPolicy => {
                Self::RequestedAcceleratorUnavailableFailPolicy
            }
            HardwareAccelerationReadinessReason::ProbeError => Self::ProbeError,
            HardwareAccelerationReadinessReason::DeviceInitializationFailed => {
                Self::DeviceInitializationFailed
            }
            HardwareAccelerationReadinessReason::SmokeProbeFailed => Self::SmokeProbeFailed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackReadinessCheck {
    pub name: AdminPlaybackReadinessCheckName,
    pub status: AdminPlaybackReadinessStatus,
    pub reason: AdminPlaybackReadinessReason,
}

impl AdminPlaybackReadinessCheck {
    #[must_use]
    pub const fn ready(
        name: AdminPlaybackReadinessCheckName,
        reason: AdminPlaybackReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminPlaybackReadinessStatus::Ready,
            reason,
        }
    }

    #[must_use]
    pub const fn degraded(
        name: AdminPlaybackReadinessCheckName,
        reason: AdminPlaybackReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminPlaybackReadinessStatus::Degraded,
            reason,
        }
    }

    #[must_use]
    pub const fn unavailable(
        name: AdminPlaybackReadinessCheckName,
        reason: AdminPlaybackReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminPlaybackReadinessStatus::Unavailable,
            reason,
        }
    }

    #[must_use]
    pub fn from_hardware(readiness: HardwareAccelerationReadiness) -> Self {
        Self {
            name: AdminPlaybackReadinessCheckName::HardwareAcceleration,
            status: readiness.status.into(),
            reason: readiness.reason.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessCheckName {
    FfmpegProbe,
    HardwareAcceleration,
    SelectedFallback,
    TranscodeBudget,
    RemotePlaybackBudget,
    Staging,
}

impl AdminPlaybackReadinessDiagnostics {
    #[must_use]
    pub fn from_checks(checks: Vec<AdminPlaybackReadinessCheck>) -> Self {
        let status = if checks
            .iter()
            .any(|check| check.status == AdminPlaybackReadinessStatus::Unavailable)
        {
            AdminPlaybackReadinessStatus::Unavailable
        } else if checks
            .iter()
            .any(|check| check.status == AdminPlaybackReadinessStatus::Degraded)
        {
            AdminPlaybackReadinessStatus::Degraded
        } else {
            AdminPlaybackReadinessStatus::Ready
        };
        let reason = checks
            .iter()
            .find(|check| check.status == status)
            .map_or(AdminPlaybackReadinessReason::Ready, |check| check.reason);

        Self {
            status,
            reason,
            checks,
        }
    }

    #[must_use]
    pub fn from_hardware(readiness: HardwareAccelerationReadiness) -> Self {
        Self::from_checks(vec![AdminPlaybackReadinessCheck::from_hardware(readiness)])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub summary: AdminStorageStagingSummary,
    pub records: Vec<AdminStorageStagingRecord>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingSummary {
    pub configured_max_bytes: u64,
    pub used_manifest_bytes: u64,
    pub cleanup_on_startup: bool,
    pub retention_ms: u64,
    pub startup_deleted_records: u32,
    pub startup_deleted_files: u32,
    pub process_cached_backends: u32,
    pub vfs_cache: AdminVfsCacheSummary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheSummary {
    pub object_count: u64,
    pub listing_count: u64,
    pub failure_count: u64,
    pub stale_object_count: u64,
    pub stale_listing_count: u64,
    pub last_failure_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingRecord {
    pub id: StagingManifestId,
    pub source_scheme: String,
    pub purpose: StagingPurpose,
    pub state: StagingState,
    pub size_bytes: Option<u64>,
    pub has_etag: bool,
    pub has_fingerprint: bool,
    pub active_leases: u32,
    pub has_validation_error: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_accessed_at_ms: i64,
    pub expires_at_ms: Option<i64>,
}

impl AdminStorageStagingRecord {
    #[must_use]
    pub fn from_record(record: StagingManifestRecord) -> Self {
        Self {
            id: record.id,
            source_scheme: record.source_scheme,
            purpose: record.purpose,
            state: record.state,
            size_bytes: record.size_bytes,
            has_etag: record.etag.is_some(),
            has_fingerprint: record.fingerprint.is_some(),
            active_leases: record.active_leases,
            has_validation_error: record.validation_error.is_some(),
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
            last_accessed_at_ms: record.last_accessed_at_ms,
            expires_at_ms: record.expires_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminServerConfigDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub auth: AdminAuthConfigDiagnostics,
    pub network: AdminNetworkAccessDiagnostics,
    pub database: AdminDatabaseConfigDiagnostics,
    pub runtime: AdminRuntimeConfigDiagnostics,
    pub libraries: Vec<AdminLibraryConfigDiagnostics>,
    pub metadata: AdminMetadataConfigDiagnostics,
    pub transcode: AdminTranscodeConfigDiagnostics,
    pub staging: AdminConfigStagingDiagnostics,
    pub playback: AdminConfigPlaybackDiagnostics,
    pub artwork: AdminArtworkConfigDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAuthConfigDiagnostics {
    pub enabled: bool,
    pub token_env: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkAccessDiagnostics {
    pub exposure_mode: AdminNetworkExposureMode,
    pub readiness: AdminNetworkReadinessDiagnostics,
    pub external_endpoint: AdminNetworkExternalEndpointDiagnostics,
    pub trusted_proxy: AdminTrustedProxyDiagnostics,
    pub origins: AdminOriginPolicyDiagnostics,
    pub tunnel_providers: Vec<AdminTunnelProviderDiagnostics>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkExposureMode {
    LocalOnly,
    PrivateNetwork,
    ReverseProxy,
    TunnelProvider,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkReadinessDiagnostics {
    pub status: AdminNetworkReadinessStatus,
    pub reason: AdminNetworkReadinessReason,
    pub checks: Vec<AdminNetworkReadinessCheck>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkReadinessReason {
    Ready,
    LocalOnly,
    AuthDisabled,
    MissingExternalBaseUrl,
    MissingTrustedProxySources,
    MissingTunnelProvider,
    MissingTunnelToken,
    BrowserOriginsNotConfigured,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkReadinessCheck {
    pub name: AdminNetworkReadinessCheckName,
    pub status: AdminNetworkReadinessStatus,
    pub reason: AdminNetworkReadinessReason,
}

impl AdminNetworkReadinessCheck {
    #[must_use]
    pub const fn ready(
        name: AdminNetworkReadinessCheckName,
        reason: AdminNetworkReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminNetworkReadinessStatus::Ready,
            reason,
        }
    }

    #[must_use]
    pub const fn degraded(
        name: AdminNetworkReadinessCheckName,
        reason: AdminNetworkReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminNetworkReadinessStatus::Degraded,
            reason,
        }
    }

    #[must_use]
    pub const fn unavailable(
        name: AdminNetworkReadinessCheckName,
        reason: AdminNetworkReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminNetworkReadinessStatus::Unavailable,
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminNetworkReadinessCheckName {
    ExposureMode,
    Auth,
    ExternalEndpoint,
    TrustedProxy,
    OriginPolicy,
    TunnelProvider,
}

impl AdminNetworkReadinessDiagnostics {
    #[must_use]
    pub fn from_checks(checks: Vec<AdminNetworkReadinessCheck>) -> Self {
        let status = if checks
            .iter()
            .any(|check| check.status == AdminNetworkReadinessStatus::Unavailable)
        {
            AdminNetworkReadinessStatus::Unavailable
        } else if checks
            .iter()
            .any(|check| check.status == AdminNetworkReadinessStatus::Degraded)
        {
            AdminNetworkReadinessStatus::Degraded
        } else {
            AdminNetworkReadinessStatus::Ready
        };
        let reason = checks
            .iter()
            .find(|check| check.status == status)
            .map_or(AdminNetworkReadinessReason::Ready, |check| check.reason);

        Self {
            status,
            reason,
            checks,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminNetworkExternalEndpointDiagnostics {
    pub configured: bool,
    pub scheme: Option<String>,
    pub host_fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTrustedProxyDiagnostics {
    pub headers_enabled: bool,
    pub source_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOriginPolicyDiagnostics {
    pub allowed_origin_count: u32,
    pub configured: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTunnelProviderDiagnostics {
    pub id: String,
    pub kind: AdminTunnelProviderKind,
    pub endpoint_configured: bool,
    pub endpoint_scheme: Option<String>,
    pub endpoint_host_fingerprint: Option<String>,
    pub token_env: Option<String>,
    pub token_present: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminTunnelProviderKind {
    External,
    CloudflareTunnel,
    TailscaleFunnel,
    Ngrok,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminDatabaseConfigDiagnostics {
    pub configured_backend_kind: String,
    pub active_backend_kind: String,
    pub url_scheme: String,
    pub runtime_supported: bool,
    pub migrated_on_startup: bool,
    pub capabilities: AdminDatabaseBackendCapabilitiesDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminDatabaseBackendCapabilitiesDiagnostics {
    pub lifecycle: bool,
    pub libraries: bool,
    pub jobs: bool,
    pub job_leases: bool,
    pub media: bool,
    pub scan_commits: bool,
    pub metadata: bool,
    pub catalog: bool,
    pub playback_state: bool,
    pub transcode_sessions: bool,
    pub event_outbox: bool,
    pub addons: bool,
    pub automation: bool,
    pub managed_artwork: bool,
    pub vfs_cache: bool,
    pub webhooks: bool,
    pub search_index: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRuntimeConfigDiagnostics {
    pub listen_addr: String,
    pub scan_concurrency: usize,
    pub probe_concurrency: usize,
    pub metadata_concurrency: usize,
    pub remux_concurrency: usize,
    pub webhook_concurrency: usize,
    pub remux_timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryConfigDiagnostics {
    pub id: LibraryId,
    pub name: String,
    pub preset: LibraryPreset,
    pub backend_kind: StorageBackendKind,
    pub root_scheme: String,
    pub has_webdav_password_env: bool,
    pub webdav_timeout_ms: Option<u64>,
    pub webdav_max_attempts: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataConfigDiagnostics {
    pub raw_cache_retention_ms: u64,
    pub raw_cache_cleanup_on_startup: bool,
    pub raw_cache_cleanup_interval_ms: u64,
    pub runtime: AdminMetadataRuntimeConfigDiagnostics,
    pub maintenance_policies: u32,
    pub providers: Vec<AdminMetadataProviderConfigDiagnostics>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataRuntimeConfigDiagnostics {
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub min_interval_ms: u64,
    pub concurrency: usize,
    pub user_agent: String,
    pub has_proxy: bool,
    pub circuit_breaker_failures: u32,
    pub circuit_breaker_backoff_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataProviderConfigDiagnostics {
    pub provider: ExternalProvider,
    pub enabled: bool,
    pub token_env: Option<String>,
    pub api_key_env: Option<String>,
    pub has_api_base_url: bool,
    pub has_image_base_url: bool,
    pub language: Option<String>,
    pub include_adult: bool,
    pub header_count: u32,
    pub secret_header_count: u32,
    pub has_provider_runtime_override: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTranscodeConfigDiagnostics {
    pub hardware_policy: HardwareAccelerationPolicy,
    pub cpu_concurrency: usize,
    pub gpu_concurrency: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminConfigStagingDiagnostics {
    pub max_bytes: u64,
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminConfigPlaybackDiagnostics {
    pub remote_stream_concurrency: usize,
    pub remote_stage_concurrency: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminArtworkConfigDiagnostics {
    pub artifact_root_configured: bool,
    pub fetch_timeout_ms: u64,
    pub fetch_max_attempts: u32,
    pub fetch_max_bytes: u64,
    pub fetch_concurrency: usize,
    pub ingest_worker_enabled: bool,
    pub ingest_worker_idle_ms: u64,
    pub fetch_user_agent: String,
    pub has_fetch_proxy: bool,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackFfmpegDiagnostics {
    pub probe_status: AdminPlaybackRuntimeStatus,
    pub has_probe_error: bool,
    pub hardware_capability_count: u32,
    pub available_gpu_capabilities: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackRuntimeStatus {
    Ready,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareDiagnostics {
    pub policy: HardwareAccelerationPolicy,
    pub selection: HardwareAccelerationSelection,
    pub capabilities: Vec<AdminPlaybackHardwareCapability>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareCapability {
    pub accelerator: HardwareAcceleration,
    pub available: bool,
    pub reason_code: AdminPlaybackHardwareCapabilityReason,
    pub encoder_discovery: AdminPlaybackHardwareEncoderDiscovery,
    pub device_initialization: AdminPlaybackHardwareDeviceInitialization,
    pub smoke_probe: AdminPlaybackHardwareSmokeProbe,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareCapabilityReason {
    Available,
    EncoderNotListed,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    ProbeError,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareSmokeProbe {
    pub status: AdminPlaybackHardwareSmokeProbeStatus,
    pub operator_check: String,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareSmokeProbeStatus {
    NotRequired,
    NotRun,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareEncoderDiscovery {
    pub status: AdminPlaybackHardwareEncoderDiscoveryStatus,
    pub encoder: Option<String>,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareEncoderDiscoveryStatus {
    NotRequired,
    Listed,
    Missing,
    ProbeError,
    Static,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareDeviceInitialization {
    pub status: AdminPlaybackHardwareDeviceInitializationStatus,
    pub operator_check: String,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareDeviceInitializationStatus {
    NotRequired,
    NotRun,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackTranscodeBudgetDiagnostics {
    pub configured_cpu_slots: usize,
    pub configured_gpu_slots: usize,
    pub effective_cpu_slots: usize,
    pub effective_gpu_slots: usize,
    pub selected_hls_slots: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRemuxRuntimeDiagnostics {
    pub max_concurrent_sessions: usize,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRemoteBudgetDiagnostics {
    pub backend_count: u32,
    pub stream_permits_available: usize,
    pub stream_permits_max: usize,
    pub stage_permits_available: usize,
    pub stage_permits_max: usize,
    pub state_scope: StorageBackendRuntimeStateScope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackStagingDiagnostics {
    pub max_bytes: u64,
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
    pub startup_deleted_records: u32,
    pub startup_deleted_files: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub status: AdminOverviewStatus,
    pub storage: AdminOverviewStorageSummary,
    pub metadata: AdminOverviewMetadataSummary,
    pub runtime: AdminOverviewRuntimeSummary,
    pub startup: AdminOverviewStartupSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminOverviewStatus {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewStorageSummary {
    pub total_backends: u32,
    pub ready_backends: u32,
    pub degraded_backends: u32,
    pub unavailable_backends: u32,
    pub backends: Vec<AdminOverviewStorageBackendSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewStorageBackendSummary {
    pub library_id: LibraryId,
    pub library_name: String,
    pub backend_kind: StorageBackendKind,
    pub status: StorageBackendStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewMetadataSummary {
    pub total_providers: u32,
    pub available_providers: u32,
    pub disabled_providers: u32,
    pub unavailable_providers: u32,
    pub providers: Vec<AdminOverviewMetadataProviderSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewMetadataProviderSummary {
    pub provider: ExternalProvider,
    pub status: MetadataProviderDiagnosticStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewRuntimeSummary {
    pub active_tasks: u32,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub succeeded_jobs: u64,
    pub cancelled_jobs: u64,
    pub failed_jobs: u64,
    pub shutdown_requested: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewStartupSummary {
    pub configured_libraries: u32,
    pub recovered_transcode_sessions: u64,
    pub recovered_jobs: u64,
    pub staging_deleted_records: u32,
    pub staging_deleted_files: u32,
    pub metadata_raw_cache_deleted: u64,
    pub metadata_lifecycle_tasks_started: u32,
    pub artwork_ingest_worker_started: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailuresResponse {
    pub library_id: LibraryId,
    pub failures: Vec<IngestionFailureDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureDiagnostic {
    #[serde(flatten)]
    pub failure: IngestionFailureDto,
    pub retryable_now: bool,
}

impl IngestionFailureDiagnostic {
    #[must_use]
    pub fn from_record(failure: IngestionFailureRecord) -> Self {
        let retryable_now = failure.status == IngestionFailureStatus::Open && failure.retryable;
        Self {
            failure: failure.into(),
            retryable_now,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureDto {
    pub library_id: LibraryId,
    pub job_id: Option<JobId>,
    pub scan_id: Option<ScanSnapshotId>,
    pub source_id: Option<MediaSourceId>,
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
    pub target_kind: String,
    pub failure_class: IngestionFailureClass,
    pub status: IngestionFailureStatus,
    pub message: String,
    pub retryable: bool,
    pub attempts: u32,
    pub first_failed_at_ms: i64,
    pub last_failed_at_ms: i64,
    pub resolved_at_ms: Option<i64>,
    pub ignored_at_ms: Option<i64>,
}

impl From<IngestionFailureRecord> for IngestionFailureDto {
    fn from(failure: IngestionFailureRecord) -> Self {
        Self {
            library_id: failure.library_id,
            job_id: failure.job_id,
            scan_id: failure.scan_id,
            source_id: failure.source_id,
            phase: failure.phase,
            target_uri: failure.target_uri,
            target_kind: failure.target_kind,
            failure_class: failure.failure_class,
            status: failure.status,
            message: failure.message,
            retryable: failure.retryable,
            attempts: failure.attempts,
            first_failed_at_ms: failure.first_failed_at_ms,
            last_failed_at_ms: failure.last_failed_at_ms,
            resolved_at_ms: failure.resolved_at_ms,
            ignored_at_ms: failure.ignored_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IgnoreIngestionFailureRequest {
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendDiagnosticsResponse {
    pub backends: Vec<StorageBackendDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendDiagnostic {
    pub library_id: LibraryId,
    pub library_name: String,
    pub root_uri: String,
    pub backend_kind: StorageBackendKind,
    pub scheme: String,
    pub status: StorageBackendStatus,
    pub reason: Option<String>,
    pub registry: StorageBackendRegistryDiagnostic,
    pub health: StorageBackendHealthDiagnostic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendKind {
    Local,
    WebDav,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendRegistryDiagnostic {
    pub cached: bool,
    pub stream_permits_available: usize,
    pub stream_permits_max: usize,
    pub stage_permits_available: usize,
    pub stage_permits_max: usize,
    pub state_scope: StorageBackendRuntimeStateScope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendHealthDiagnostic {
    pub consecutive_errors: u64,
    pub last_success_at_ms: Option<i64>,
    pub last_error_at_ms: Option<i64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendRuntimeStateScope {
    ProcessLocal,
}

fn stable_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());

    format!("sha256:{}", lowercase_hex(&digest[..16]))
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn storage_scheme(reference: &str) -> String {
    reference
        .split_once("://")
        .map_or("unknown", |(scheme, _path)| {
            if scheme.trim().is_empty() {
                "unknown"
            } else {
                scheme
            }
        })
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use crate::{
        metadata_diagnostics::MetadataProviderDiagnosticStatus, public_client::API_VERSION,
    };

    use super::*;

    #[test]
    fn ingestion_failure_diagnostic_serializes_explicit_dto_fields() {
        let record = IngestionFailureRecord {
            library_id: LibraryId::new(),
            job_id: Some(JobId::new()),
            scan_id: Some(ScanSnapshotId::new()),
            source_id: None,
            phase: IngestionFailurePhase::Scan,
            target_uri: "webdav:///Movies/Broken/".to_owned(),
            target_kind: "directory".to_owned(),
            failure_class: IngestionFailureClass::Storage,
            status: IngestionFailureStatus::Open,
            message: "failed to list directory".to_owned(),
            retryable: true,
            attempts: 2,
            first_failed_at_ms: 10,
            last_failed_at_ms: 20,
            resolved_at_ms: None,
            ignored_at_ms: None,
        };

        let diagnostic = IngestionFailureDiagnostic::from_record(record);
        let value = serde_json::to_value(&diagnostic).unwrap();

        assert_eq!(diagnostic.failure.attempts, 2);
        assert!(diagnostic.retryable_now);
        assert_eq!(value["phase"], "scan");
        assert_eq!(value["failure_class"], "storage");
        assert_eq!(value["status"], "open");
        assert!(value.get("failure").is_none());
    }

    #[test]
    fn job_response_redacts_raw_payloads_summaries_and_errors() {
        let job = Job {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            status: JobStatus::Failed,
            resource_class: "disk.scan".to_owned(),
            library_id: Some(LibraryId::new()),
            source_id: Some(MediaSourceId::new()),
            input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
            summary_json: Some(r#"{"output_path":"C:\\media\\private.nfo"}"#.to_owned()),
            error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        };

        let response = JobResponse::from_job(job);
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.has_input);
        assert!(response.has_summary);
        assert!(response.has_error);
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("\"input\":"));
        assert!(!body.contains("\"summary\":"));
        assert!(!body.contains("\"error\":"));
    }

    #[test]
    fn admin_job_list_item_redacts_raw_payloads_and_errors() {
        let job = Job {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            status: JobStatus::Failed,
            resource_class: "disk.scan".to_owned(),
            library_id: Some(LibraryId::new()),
            source_id: Some(MediaSourceId::new()),
            input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
            summary_json: Some(r#"{"output_path":"C:\\media\\private.nfo"}"#.to_owned()),
            error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        };

        let item = AdminJobListItem::from_job(job);
        let body = serde_json::to_string(&item).unwrap();

        assert!(item.has_input);
        assert!(item.has_summary);
        assert!(item.has_error);
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
    }

    #[test]
    fn admin_job_cancel_request_response_redacts_raw_payloads_and_errors() {
        let record = JobCancellationRequestRecord {
            job: Job {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                status: JobStatus::Running,
                resource_class: "disk.scan".to_owned(),
                library_id: Some(LibraryId::new()),
                source_id: Some(MediaSourceId::new()),
                input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
                summary_json: Some(r#"{"output_path":"C:\\media\\private.nfo"}"#.to_owned()),
                error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
                queued_at: "2026-05-17T00:00:00Z".to_owned(),
                started_at: Some("2026-05-17T00:00:01Z".to_owned()),
                completed_at: None,
            },
            requested: true,
            terminal: false,
            cancel_requested_at: Some("2026-05-17T00:00:03Z".to_owned()),
        };

        let response = AdminJobCancelRequestResponse::from_record(record);
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.requested);
        assert!(!response.terminal);
        assert!(response.job.has_input);
        assert!(response.job.has_summary);
        assert!(response.job.has_error);
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
        assert!(!body.contains("error\":\"token"));
    }

    #[test]
    fn admin_outbox_event_list_item_redacts_payload_idempotency_key_and_error() {
        let library_id = LibraryId::new();
        let event = OutboxEventRecord {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: "library_scan:secret-key".to_owned(),
            payload_json: r#"{"token":"admin-token","path":"F:\\Media\\Movies"}"#.to_owned(),
            status: OutboxEventStatus::Failed,
            attempts: 2,
            last_error: Some("failed with admin-token at F:\\Media\\Movies".to_owned()),
            occurred_at: "2026-05-18T00:00:00Z".to_owned(),
            updated_at: "2026-05-18T00:00:01Z".to_owned(),
            next_attempt_at: None,
        };

        let item = AdminOutboxEventListItem::from_record(event);
        let body = serde_json::to_string(&item).unwrap();

        assert_eq!(item.kind, DomainEventKind::LibraryScanned);
        assert_eq!(item.status, OutboxEventStatus::Failed);
        assert_eq!(item.attempts, 2);
        assert!(item.has_payload);
        assert!(item.has_error);
        assert!(!body.contains("payload_json"));
        assert!(!body.contains("idempotency_key"));
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("Media"));
        assert!(!body.contains("failed with"));
    }

    #[test]
    fn admin_catalog_governance_item_redacts_local_inference_evidence_value() {
        let source_id = MediaSourceId::new();
        let record = CatalogGovernanceItemRecord {
            item: nako_core::MediaItem {
                id: MediaItemId::new(),
                kind: MediaKind::Unknown,
                parent_id: None,
                metadata: nako_core::CanonicalMetadata {
                    title: "Private Clip".to_owned(),
                    release_date: Some("2026-05-18".to_owned()),
                    ..nako_core::CanonicalMetadata::default()
                },
            },
            library_id: LibraryId::new(),
            source_count: 1,
            representative_source_id: Some(source_id),
            representative_file_name: Some("Private Clip.mkv".to_owned()),
            best_local_inference: Some(LocalInferenceEvidence {
                id: nako_core::LocalInferenceEvidenceId::new(),
                source_id,
                inferred_kind: MediaKind::Unknown,
                inferred_title: Some("Private Clip".to_owned()),
                inferred_year: None,
                inferred_season: None,
                inferred_episode: None,
                confidence_milli: Some(350),
                evidence_source: LocalInferenceEvidenceSource::Path,
                evidence_value: "local:///Users/admin/Private/Private Clip.mkv".to_owned(),
                inference_version: "nako-naming:1".to_owned(),
            }),
            provider_mapping_count: 0,
            accepted_provider_mapping_count: 0,
            duplicate_relationship_count: 0,
        };

        let item = AdminCatalogGovernanceItem::from_record(record, 700);
        let body = serde_json::to_string(&item).unwrap();

        assert_eq!(item.kind, MediaKind::Unknown);
        assert_eq!(
            item.local_inference.as_ref().unwrap().confidence_milli,
            Some(350)
        );
        assert!(
            item.issues
                .contains(&AdminCatalogGovernanceIssue::UnknownKind)
        );
        assert!(
            item.issues
                .contains(&AdminCatalogGovernanceIssue::LowLocalInferenceConfidence)
        );
        assert!(
            item.issues
                .contains(&AdminCatalogGovernanceIssue::MissingAcceptedProviderMapping)
        );
        assert!(!body.contains("evidence_value"));
        assert!(!body.contains("local:///Users"));
        assert!(!body.contains("/Private/"));
    }

    #[test]
    fn admin_playback_session_list_item_redacts_output_path_and_failure_message() {
        let session = TranscodeSessionRecord {
            id: TranscodeSessionId::new(),
            source_id: MediaSourceId::new(),
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: "transcode-profile:v1;kind=hls_single_variant".to_owned(),
            output_path: "C:\\nako-cache\\hls\\secret\\playlist.m3u8".into(),
            state: TranscodeSessionState::Failed,
            failure_category: Some(TranscodeFailureCategory::Runner),
            failure_message: Some(
                "ffmpeg failed while writing C:\\nako-cache\\hls\\secret\\playlist.m3u8".to_owned(),
            ),
            created_at: "2026-05-18T00:00:00Z".to_owned(),
            updated_at: "2026-05-18T00:00:01Z".to_owned(),
            started_at: Some("2026-05-18T00:00:00Z".to_owned()),
            completed_at: Some("2026-05-18T00:00:01Z".to_owned()),
        };

        let item = AdminPlaybackSessionListItem::from_record(session);
        let body = serde_json::to_string(&item).unwrap();

        assert_eq!(item.kind, TranscodeSessionKind::HlsTranscode);
        assert_eq!(item.state, TranscodeSessionState::Failed);
        assert_eq!(
            item.failure_category,
            Some(TranscodeFailureCategory::Runner)
        );
        assert!(item.has_failure_message);
        assert!(!item.active);
        assert!(item.terminal);
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("playlist.m3u8"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ffmpeg failed while writing"));
    }

    #[test]
    fn admin_playback_support_evidence_redacts_session_secrets_but_keeps_support_facts() {
        let source_id = MediaSourceId::new();
        let request_key =
            "transcode-request:v1;source=source-revision:v1;digest=demo;profile=secret-profile"
                .to_owned();
        let session = TranscodeSessionRecord {
            id: TranscodeSessionId::new(),
            source_id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: request_key.clone(),
            output_path: "C:\\nako-cache\\hls\\secret\\playlist.m3u8".into(),
            state: TranscodeSessionState::Failed,
            failure_category: Some(TranscodeFailureCategory::Runner),
            failure_message: Some(
                "ffmpeg failed while writing C:\\nako-cache\\hls\\secret\\playlist.m3u8".to_owned(),
            ),
            created_at: "2026-05-18T00:00:00Z".to_owned(),
            updated_at: "2026-05-18T00:00:01Z".to_owned(),
            started_at: Some("2026-05-18T00:00:00Z".to_owned()),
            completed_at: Some("2026-05-18T00:00:01Z".to_owned()),
        };

        let evidence = AdminPlaybackSupportSessionEvidence::from_record(session);
        let body = serde_json::to_string(&evidence).unwrap();

        assert_eq!(evidence.source_id, source_id);
        assert_eq!(evidence.kind, TranscodeSessionKind::HlsTranscode);
        assert_eq!(
            evidence.failure_category,
            Some(TranscodeFailureCategory::Runner)
        );
        assert!(evidence.has_failure_message);
        assert_eq!(
            evidence.output_artifact_kind,
            AdminPlaybackOutputArtifactKind::HlsPlaylist
        );
        assert!(evidence.request_key_fingerprint.starts_with("sha256:"));
        assert_ne!(evidence.request_key_fingerprint, request_key);
        assert!(!body.contains("secret-profile"));
        assert!(!body.contains("transcode-request"));
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("playlist.m3u8"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ffmpeg failed while writing"));
    }

    #[test]
    fn admin_playback_support_source_evidence_keeps_scheme_not_locator() {
        let source = nako_core::MediaSource {
            id: MediaSourceId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            locator: "webdav:///Movies/Private/Secret Demo.mkv?token=admin-token".to_owned(),
            file_name: "Secret Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("sha256:private-fingerprint".to_owned()),
        };

        let evidence = AdminPlaybackSupportSourceEvidence::from_record(source);
        let body = serde_json::to_string(&evidence).unwrap();

        assert_eq!(evidence.source_scheme, "webdav");
        assert_eq!(evidence.file_name, "Secret Demo.mkv");
        assert_eq!(evidence.size_bytes, Some(42));
        assert!(evidence.has_fingerprint);
        assert!(!body.contains("locator"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private-fingerprint"));
    }

    #[test]
    fn admin_acquisition_intake_diagnostics_use_redacted_refs_not_raw_sources() {
        let diagnostic = AdminAcquisitionIntakeCandidateDiagnostic {
            id: AcquisitionIntakeCandidateId::new(),
            target_library_id: LibraryId::new(),
            source_kind: "watch_folder".to_owned(),
            custom_source_kind: false,
            source_scheme: Some("local".to_owned()),
            source_ref_redacted: "local://<redacted>".to_owned(),
            source_key_fingerprint: "sha256:0123456789abcdef0123456789abcdef".to_owned(),
            has_display_name: true,
            has_intended_locator: true,
            size_bytes: Some(42),
            has_fingerprint: true,
            managed_import_artifact_id: Some(ManagedImportArtifactId::new()),
            state: AcquisitionIntakeCandidateState::Ready,
            has_diagnostics: true,
            first_seen_at_ms: 1_000,
            last_seen_at_ms: 1_100,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
        };
        let response = AdminAcquisitionIntakeCandidateListResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            candidates: vec![diagnostic],
            page: PageInfo {
                limit: 10,
                offset: 0,
                returned: 1,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["candidates"][0]["source_kind"], "watch_folder");
        assert_eq!(value["candidates"][0]["source_scheme"], "local");
        assert_eq!(
            value["candidates"][0]["source_ref_redacted"],
            "local://<redacted>"
        );
        assert_eq!(value["candidates"][0]["state"], "ready");
        assert_eq!(value["page"]["returned"], 1);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("\"intended_locator\""));
        assert!(!body.contains("\"display_name\""));
        assert!(!body.contains("diagnostics_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("token"));
        assert!(!body.contains("private"));
    }

    #[test]
    fn admin_watch_folder_discovery_response_redacts_root_and_failures() {
        let response = AdminWatchFolderDiscoveryResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            target_library_id: LibraryId::new(),
            root_scheme: Some("local".to_owned()),
            root_ref_redacted: "local://<redacted>".to_owned(),
            ready_candidates: 2,
            blocked_candidates: 1,
            incomplete_candidates: 1,
            unsupported_candidates: 0,
            recorded_candidates: 3,
            failures: vec![AdminWatchFolderDiscoveryFailure {
                ref_redacted: "local://<redacted>".to_owned(),
                safe_message: "storage error: NotFound".to_owned(),
            }],
            writes_library: false,
            managed_import_artifacts_created: false,
            promotion_apply: false,
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["root_ref_redacted"], "local://<redacted>");
        assert_eq!(value["ready_candidates"], 2);
        assert_eq!(value["failures"][0]["ref_redacted"], "local://<redacted>");
        assert_eq!(value["writes_library"], false);
        assert_eq!(value["managed_import_artifacts_created"], false);
        assert_eq!(value["promotion_apply"], false);
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("uri_redacted"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("token"));
        assert!(!body.contains("C:\\"));
    }

    #[test]
    fn admin_generated_artifact_proposals_expose_summaries_not_raw_prompt_or_payload() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let provider_id = AutomationProviderId::new();
        let job_id = JobId::new();
        let proposal = GeneratedArtifactProposal {
            id: AutomationArtifactId::new(),
            kind: AutomationArtifactKind::MetadataSuggestion,
            capability: AutomationCapability::MetadataCleanup,
            status: AutomationArtifactStatus::Proposed,
            target: GeneratedArtifactTarget {
                kind: GeneratedArtifactTargetKind::MediaSource,
                library_id: Some(library_id),
                item_id: Some(item_id),
                source_id: Some(source_id),
            },
            provenance: GeneratedArtifactProvenance {
                provider_id,
                provider_name: Some("External automation gateway".to_owned()),
                job_id,
                capability: AutomationCapability::MetadataCleanup,
                idempotency_key_fingerprint: Some(
                    "sha256:11111111111111111111111111111111".to_owned(),
                ),
                prompt_fingerprint: Some("sha256:22222222222222222222222222222222".to_owned()),
                attempt_count: Some(2),
                artifact_created_at: "2026-05-22T00:00:00Z".to_owned(),
            },
            payload: GeneratedArtifactPayloadSummary {
                valid_json: true,
                shape: GeneratedArtifactPayloadShape::Object,
                payload_fingerprint: "sha256:33333333333333333333333333333333".to_owned(),
                payload_bytes: 512,
                object_field_count: Some(3),
                array_item_count: None,
                has_textual_values: true,
                has_explanation: true,
                confidence_milli: Some(810),
            },
            readiness: GeneratedArtifactReadiness {
                status: GeneratedArtifactReadinessStatus::Ready,
                actionable: true,
                reasons: vec![GeneratedArtifactReadinessReason::Ready],
            },
            created_at: "2026-05-22T00:00:00Z".to_owned(),
            updated_at: "2026-05-22T00:00:01Z".to_owned(),
            accepted_at: None,
        };

        let response = AdminGeneratedArtifactProposalListResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            proposals: vec![AdminGeneratedArtifactProposal::from_proposal(proposal)],
            page: PageInfo {
                limit: 20,
                offset: 0,
                returned: 1,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["proposals"][0]["capability"], "metadata_cleanup");
        assert_eq!(value["proposals"][0]["target"]["kind"], "media_source");
        assert_eq!(value["proposals"][0]["readiness"]["status"], "ready");
        assert_eq!(value["proposals"][0]["payload"]["confidence_milli"], 810);
        assert_eq!(
            value["proposals"][0]["payload"]["payload_fingerprint"],
            "sha256:33333333333333333333333333333333"
        );
        assert!(!body.contains("prompt_json"));
        assert!(!body.contains("artifact_json"));
        assert!(!body.contains("source_locator"));
        assert!(!body.contains("source_fingerprint"));
        assert!(!body.contains("secret_env"));
        assert!(!body.contains("raw"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("C:\\"));
        assert!(!body.contains("admin-token"));
    }

    #[test]
    fn admin_generated_artifact_review_response_exposes_boundary_not_raw_payload() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let artifact_id = AutomationArtifactId::new();
        let plan = GeneratedArtifactAcceptancePlan {
            artifact_id,
            decision: GeneratedArtifactReviewDecision::Accept,
            status: GeneratedArtifactAcceptancePlanStatus::Ready,
            action: GeneratedArtifactAcceptanceActionKind::StageMetadataAuthorityReview,
            reasons: vec![
                GeneratedArtifactAcceptancePlanReason::Ready,
                GeneratedArtifactAcceptancePlanReason::MetadataAuthorityApplyRequired,
            ],
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            target: GeneratedArtifactTarget {
                kind: GeneratedArtifactTargetKind::MediaSource,
                library_id: Some(library_id),
                item_id: Some(item_id),
                source_id: Some(source_id),
            },
            payload: GeneratedArtifactPayloadSummary {
                valid_json: true,
                shape: GeneratedArtifactPayloadShape::Object,
                payload_fingerprint: "sha256:33333333333333333333333333333333".to_owned(),
                payload_bytes: 512,
                object_field_count: Some(3),
                array_item_count: None,
                has_textual_values: true,
                has_explanation: true,
                confidence_milli: Some(810),
            },
            readiness: GeneratedArtifactReadiness {
                status: GeneratedArtifactReadinessStatus::Ready,
                actionable: true,
                reasons: vec![GeneratedArtifactReadinessReason::Ready],
            },
            boundary: GeneratedArtifactAcceptanceBoundary::deferred_metadata_authority(),
        };
        let response =
            AdminGeneratedArtifactReviewResponse::from_result(GeneratedArtifactReviewResult {
                artifact_id,
                decision: GeneratedArtifactReviewDecision::Accept,
                artifact_status: AutomationArtifactStatus::Accepted,
                accepted_at: Some("2026-05-22T00:00:02Z".to_owned()),
                idempotent_replay: false,
                plan,
            });

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["decision"], "accept");
        assert_eq!(value["artifact_status"], "accepted");
        assert_eq!(value["plan"]["status"], "ready");
        assert_eq!(value["plan"]["action"], "stage_metadata_authority_review");
        assert_eq!(
            value["plan"]["boundary"]["accepted_into_canonical_metadata"],
            false
        );
        assert_eq!(value["plan"]["boundary"]["writes_sidecar"], false);
        assert_eq!(value["plan"]["boundary"]["writes_library_files"], false);
        assert_eq!(value["plan"]["boundary"]["applies_immediately"], false);
        assert_eq!(
            value["plan"]["boundary"]["requires_metadata_authority_apply"],
            true
        );
        assert!(!body.contains("prompt_json"));
        assert!(!body.contains("artifact_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("raw"));
    }

    #[test]
    fn admin_network_access_diagnostics_serializes_readiness_without_secret_urls() {
        let response = AdminServerConfigDiagnosticsResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            auth: AdminAuthConfigDiagnostics {
                enabled: true,
                token_env: Some("NAKO_ADMIN_TOKEN".to_owned()),
            },
            network: AdminNetworkAccessDiagnostics {
                exposure_mode: AdminNetworkExposureMode::ReverseProxy,
                readiness: AdminNetworkReadinessDiagnostics::from_checks(vec![
                    AdminNetworkReadinessCheck::ready(
                        AdminNetworkReadinessCheckName::Auth,
                        AdminNetworkReadinessReason::Ready,
                    ),
                    AdminNetworkReadinessCheck::degraded(
                        AdminNetworkReadinessCheckName::OriginPolicy,
                        AdminNetworkReadinessReason::BrowserOriginsNotConfigured,
                    ),
                ]),
                external_endpoint: AdminNetworkExternalEndpointDiagnostics {
                    configured: true,
                    scheme: Some("https".to_owned()),
                    host_fingerprint: Some("sha256:0123456789abcdef".to_owned()),
                },
                trusted_proxy: AdminTrustedProxyDiagnostics {
                    headers_enabled: true,
                    source_count: 2,
                },
                origins: AdminOriginPolicyDiagnostics {
                    allowed_origin_count: 0,
                    configured: false,
                },
                tunnel_providers: vec![AdminTunnelProviderDiagnostics {
                    id: "cloudflared".to_owned(),
                    kind: AdminTunnelProviderKind::CloudflareTunnel,
                    endpoint_configured: true,
                    endpoint_scheme: Some("https".to_owned()),
                    endpoint_host_fingerprint: Some("sha256:fedcba9876543210".to_owned()),
                    token_env: Some("NAKO_TUNNEL_TOKEN".to_owned()),
                    token_present: true,
                }],
            },
            database: AdminDatabaseConfigDiagnostics {
                configured_backend_kind: "sqlite".to_owned(),
                active_backend_kind: "sqlite".to_owned(),
                url_scheme: "sqlite".to_owned(),
                runtime_supported: true,
                migrated_on_startup: true,
                capabilities: AdminDatabaseBackendCapabilitiesDiagnostics {
                    lifecycle: true,
                    libraries: true,
                    jobs: true,
                    job_leases: true,
                    media: true,
                    scan_commits: true,
                    metadata: true,
                    catalog: true,
                    playback_state: true,
                    transcode_sessions: true,
                    event_outbox: true,
                    addons: true,
                    automation: true,
                    managed_artwork: true,
                    vfs_cache: true,
                    webhooks: true,
                    search_index: true,
                },
            },
            runtime: AdminRuntimeConfigDiagnostics {
                listen_addr: "127.0.0.1:3000".to_owned(),
                scan_concurrency: 1,
                probe_concurrency: 1,
                metadata_concurrency: 1,
                remux_concurrency: 1,
                webhook_concurrency: 1,
                remux_timeout_ms: 30_000,
            },
            libraries: Vec::new(),
            metadata: AdminMetadataConfigDiagnostics {
                raw_cache_retention_ms: 0,
                raw_cache_cleanup_on_startup: false,
                raw_cache_cleanup_interval_ms: 0,
                runtime: AdminMetadataRuntimeConfigDiagnostics {
                    timeout_ms: 1_000,
                    max_attempts: 1,
                    min_interval_ms: 0,
                    concurrency: 1,
                    user_agent: "nako-test".to_owned(),
                    has_proxy: false,
                    circuit_breaker_failures: 1,
                    circuit_breaker_backoff_ms: 1,
                },
                maintenance_policies: 0,
                providers: Vec::new(),
            },
            transcode: AdminTranscodeConfigDiagnostics {
                hardware_policy: HardwareAccelerationPolicy {
                    requested: HardwareAcceleration::None,
                    fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
                },
                cpu_concurrency: 1,
                gpu_concurrency: 1,
            },
            staging: AdminConfigStagingDiagnostics {
                max_bytes: 1,
                retention_ms: 1,
                cleanup_on_startup: false,
            },
            playback: AdminConfigPlaybackDiagnostics {
                remote_stream_concurrency: 1,
                remote_stage_concurrency: 1,
            },
            artwork: AdminArtworkConfigDiagnostics {
                artifact_root_configured: false,
                fetch_timeout_ms: 1,
                fetch_max_attempts: 1,
                fetch_max_bytes: 1,
                fetch_concurrency: 1,
                ingest_worker_enabled: false,
                ingest_worker_idle_ms: 1,
                fetch_user_agent: "nako-test".to_owned(),
                has_fetch_proxy: false,
                max_width: 1,
                max_height: 1,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["network"]["exposure_mode"], "reverse_proxy");
        assert_eq!(value["network"]["readiness"]["status"], "degraded");
        assert_eq!(
            value["network"]["readiness"]["reason"],
            "browser_origins_not_configured"
        );
        assert_eq!(value["network"]["external_endpoint"]["scheme"], "https");
        assert_eq!(value["network"]["trusted_proxy"]["source_count"], 2);
        assert_eq!(value["network"]["origins"]["allowed_origin_count"], 0);
        assert_eq!(
            value["network"]["tunnel_providers"][0]["kind"],
            "cloudflare_tunnel"
        );
        assert_eq!(
            value["network"]["tunnel_providers"][0]["token_env"],
            "NAKO_TUNNEL_TOKEN"
        );
        assert_eq!(
            value["network"]["tunnel_providers"][0]["token_present"],
            true
        );
        assert!(!body.contains("external_base_url"));
        assert!(!body.contains("trusted_proxy_sources"));
        assert!(!body.contains("allowed_origins"));
        assert!(!body.contains("public_url"));
        assert!(!body.contains("nako.example"));
        assert!(!body.contains("cloudflare-token-secret"));
        assert!(!body.contains("Authorization"));
        assert!(!body.contains("x-forwarded"));
    }

    #[test]
    fn admin_playback_runtime_diagnostics_serializes_safe_summary_fields() {
        let response = AdminPlaybackRuntimeDiagnosticsResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            readiness: AdminPlaybackReadinessDiagnostics::from_checks(vec![
                AdminPlaybackReadinessCheck::degraded(
                    AdminPlaybackReadinessCheckName::FfmpegProbe,
                    AdminPlaybackReadinessReason::ProbeError,
                ),
                AdminPlaybackReadinessCheck::ready(
                    AdminPlaybackReadinessCheckName::TranscodeBudget,
                    AdminPlaybackReadinessReason::TranscodeBudgetReady,
                ),
            ]),
            ffmpeg: AdminPlaybackFfmpegDiagnostics {
                probe_status: AdminPlaybackRuntimeStatus::Degraded,
                has_probe_error: true,
                hardware_capability_count: 4,
                available_gpu_capabilities: 1,
            },
            hardware: AdminPlaybackHardwareDiagnostics {
                policy: HardwareAccelerationPolicy {
                    requested: HardwareAcceleration::Nvenc,
                    fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
                },
                selection: HardwareAccelerationSelection {
                    acceleration: HardwareAcceleration::None,
                    fallback_used: true,
                    reason: "nvenc is unavailable; falling back to cpu".to_owned(),
                },
                capabilities: vec![AdminPlaybackHardwareCapability {
                    accelerator: HardwareAcceleration::Nvenc,
                    available: false,
                    reason_code: AdminPlaybackHardwareCapabilityReason::ProbeError,
                    encoder_discovery: AdminPlaybackHardwareEncoderDiscovery {
                        status: AdminPlaybackHardwareEncoderDiscoveryStatus::ProbeError,
                        encoder: None,
                        has_detail: true,
                    },
                    device_initialization: AdminPlaybackHardwareDeviceInitialization {
                        status: AdminPlaybackHardwareDeviceInitializationStatus::NotRun,
                        operator_check: "Verify the NVIDIA driver and FFmpeg can initialize NVENC"
                            .to_owned(),
                        has_detail: false,
                    },
                    smoke_probe: AdminPlaybackHardwareSmokeProbe {
                        status: AdminPlaybackHardwareSmokeProbeStatus::NotRun,
                        operator_check: "Run an NVENC H.264 encode smoke test on the host"
                            .to_owned(),
                        has_detail: false,
                    },
                }],
            },
            transcode: AdminPlaybackTranscodeBudgetDiagnostics {
                configured_cpu_slots: 0,
                configured_gpu_slots: 2,
                effective_cpu_slots: 1,
                effective_gpu_slots: 2,
                selected_hls_slots: 1,
            },
            remux: AdminPlaybackRemuxRuntimeDiagnostics {
                max_concurrent_sessions: 1,
                timeout_ms: 30_000,
            },
            remote_playback: AdminPlaybackRemoteBudgetDiagnostics {
                backend_count: 1,
                stream_permits_available: 8,
                stream_permits_max: 8,
                stage_permits_available: 2,
                stage_permits_max: 2,
                state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
            },
            staging: AdminPlaybackStagingDiagnostics {
                max_bytes: 100,
                retention_ms: 200,
                cleanup_on_startup: true,
                startup_deleted_records: 1,
                startup_deleted_files: 1,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["readiness"]["status"], "degraded");
        assert_eq!(value["readiness"]["reason"], "probe_error");
        assert_eq!(value["readiness"]["checks"][0]["name"], "ffmpeg_probe");
        assert_eq!(value["readiness"]["checks"][0]["status"], "degraded");
        assert_eq!(value["readiness"]["checks"][1]["name"], "transcode_budget");
        assert_eq!(value["ffmpeg"]["probe_status"], "degraded");
        assert_eq!(value["hardware"]["policy"]["requested"], "nvenc");
        assert_eq!(value["hardware"]["selection"]["acceleration"], "none");
        assert_eq!(
            value["hardware"]["capabilities"][0]["reason_code"],
            "probe_error"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["encoder_discovery"]["status"],
            "probe_error"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["device_initialization"]["status"],
            "not_run"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["smoke_probe"]["status"],
            "not_run"
        );
        assert_eq!(value["remote_playback"]["state_scope"], "process_local");
        assert!(!body.contains("ffmpeg_path"));
        assert!(!body.contains("remux_staging_root"));
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("token"));
    }

    #[test]
    fn admin_storage_staging_record_redacts_paths_source_uri_and_errors() {
        let record = StagingManifestRecord {
            id: StagingManifestId::new(),
            source_uri: "webdav:///Movies/Private/Demo.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::FfmpegInput,
            local_path: "F:\\Nako\\secret-cache\\inputs\\Demo.mkv".to_owned(),
            size_bytes: Some(42),
            etag: Some("etag-secret".to_owned()),
            fingerprint: Some("fingerprint-secret".to_owned()),
            state: StagingState::Failed,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(1_300),
            active_leases: 0,
            validation_error: Some("failed at F:\\Nako\\secret-cache".to_owned()),
        };

        let item = AdminStorageStagingRecord::from_record(record);
        let body = serde_json::to_string(&item).unwrap();

        assert_eq!(item.source_scheme, "webdav");
        assert_eq!(item.purpose, StagingPurpose::FfmpegInput);
        assert_eq!(item.state, StagingState::Failed);
        assert_eq!(item.size_bytes, Some(42));
        assert!(item.has_etag);
        assert!(item.has_fingerprint);
        assert!(item.has_validation_error);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("secret-cache"));
        assert!(!body.contains("etag-secret"));
        assert!(!body.contains("fingerprint-secret"));
        assert!(!body.contains("failed at"));
    }

    #[test]
    fn admin_overview_response_serializes_safe_summary_fields() {
        let library_id = LibraryId::new();
        let response = AdminOverviewResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            status: AdminOverviewStatus::Healthy,
            storage: AdminOverviewStorageSummary {
                total_backends: 1,
                ready_backends: 1,
                degraded_backends: 0,
                unavailable_backends: 0,
                backends: vec![AdminOverviewStorageBackendSummary {
                    library_id,
                    library_name: "Movies".to_owned(),
                    backend_kind: StorageBackendKind::Local,
                    status: StorageBackendStatus::Ready,
                }],
            },
            metadata: AdminOverviewMetadataSummary {
                total_providers: 1,
                available_providers: 1,
                disabled_providers: 0,
                unavailable_providers: 0,
                providers: vec![AdminOverviewMetadataProviderSummary {
                    provider: nako_core::ExternalProvider::Tmdb,
                    status: MetadataProviderDiagnosticStatus::Available,
                }],
            },
            runtime: AdminOverviewRuntimeSummary {
                active_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                succeeded_jobs: 0,
                cancelled_jobs: 0,
                failed_jobs: 0,
                shutdown_requested: false,
            },
            startup: AdminOverviewStartupSummary {
                configured_libraries: 1,
                recovered_transcode_sessions: 0,
                recovered_jobs: 0,
                staging_deleted_records: 0,
                staging_deleted_files: 0,
                metadata_raw_cache_deleted: 0,
                metadata_lifecycle_tasks_started: 0,
                artwork_ingest_worker_started: false,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["public_api_version"], API_VERSION);
        assert_eq!(value["status"], "healthy");
        assert_eq!(value["storage"]["ready_backends"], 1);
        assert_eq!(value["storage"]["backends"][0]["status"], "ready");
        assert_eq!(value["metadata"]["providers"][0]["provider"], "tmdb");
        assert!(!body.contains("secret"));
        assert!(!body.contains("token"));
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ProviderRawResponse"));
    }
}
