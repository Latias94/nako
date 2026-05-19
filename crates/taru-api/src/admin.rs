use serde::{Deserialize, Serialize};
use taru_client_protocol::PageInfo;
use taru_client_protocol::{ClientImageOwner, PublicImageRefDto};
use taru_core::{
    ArtworkCandidateId, ArtworkCandidateStatus, CatalogGovernanceItemRecord, DomainEventKind,
    DomainEventSubject, EventId, ExternalProvider, ImageKind, IngestionFailureClass,
    IngestionFailurePhase, IngestionFailureRecord, IngestionFailureStatus, Job, JobId, JobKind,
    JobStatus, LibraryId, LibraryPreset, LocalInferenceEvidence, LocalInferenceEvidenceSource,
    ManagedArtworkAcceptanceRecord, ManagedArtworkArtifactId, ManagedArtworkArtifactRecord,
    ManagedArtworkIngestId, ManagedArtworkIngestProcessingRecord, ManagedArtworkIngestRecord,
    ManagedArtworkIngestStatus, MediaItemId, MediaKind, MediaSourceId, OutboxEventRecord,
    OutboxEventStatus, ScanSnapshotId, SelectedArtworkPublicationRecord, SelectedArtworkRecord,
    StagingManifestId, StagingManifestRecord, StagingPurpose, StagingState,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionState,
};
use taru_transcode::{
    HardwareAcceleration, HardwareAccelerationPolicy, HardwareAccelerationSelection,
};

use crate::metadata_diagnostics::MetadataProviderDiagnosticStatus;

pub const ADMIN_API_VERSION: &str = "v1";

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
            created_at: record.created_at,
            updated_at: record.updated_at,
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
    pub content_hash: Option<String>,
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
            content_hash: record.content_hash,
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
        let selected_artwork = SelectedArtworkSummary::from_record(publication.selected_artwork);
        let artifact = publication.artifact;
        let image = PublicImageRefDto {
            id: selected_artwork.id.to_string(),
            owner: ClientImageOwner::Item(selected_artwork.item_id.to_string()),
            kind: crate::image_kind_to_dto(selected_artwork.kind.clone()),
            url: format!("/images/{}", selected_artwork.id),
            width: artifact.width,
            height: artifact.height,
            language: None,
            media_type: artifact.media_type,
            etag: artifact.content_hash,
        };

        Self {
            selected_artwork,
            image,
            changed: publication.changed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SelectedArtworkSummary {
    pub id: taru_core::SelectedArtworkId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub kind: ImageKind,
    pub artifact_id: ManagedArtworkArtifactId,
    pub created_at: String,
    pub updated_at: String,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JobResponse {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub input: Option<serde_json::Value>,
    pub summary: Option<serde_json::Value>,
    pub error: Option<String>,
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
            input: job
                .input_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            summary: job
                .summary_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            error: job.error,
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
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackHardwareDiagnostics,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
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
    pub evidence: AdminPlaybackHardwareCapabilityEvidence,
    pub smoke_probe: AdminPlaybackHardwareSmokeProbe,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareCapabilityReason {
    Available,
    EncoderNotListed,
    ProbeError,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareCapabilityEvidence {
    CpuAlwaysAvailable,
    FfmpegEncoderListed,
    FfmpegEncoderMissing,
    FfmpegProbeError,
    StaticDetector,
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

#[cfg(test)]
mod tests {
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
    fn job_response_preserves_nfo_backup_retention_summary_for_admin_diagnostics() {
        let job = Job {
            id: JobId::new(),
            kind: JobKind::NfoExport,
            status: JobStatus::Succeeded,
            resource_class: "metadata.nfo.export".to_owned(),
            library_id: Some(LibraryId::new()),
            source_id: None,
            input_json: None,
            summary_json: Some(
                r#"{
                    "exported_items": 1,
                    "backed_up_items": 1,
                    "pruned_backup_items": 1,
                    "pruned_backups": 1,
                    "backups": [{
                        "backup_uri": "local:///demo.nfo.taru-backup-2",
                        "pruned_backups": ["local:///demo.nfo.taru-backup-1"]
                    }],
                    "prune_failures": []
                }"#
                .to_owned(),
            ),
            error: None,
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        };

        let response = JobResponse::from_job(job);
        let summary = response.summary.unwrap();

        assert_eq!(summary["backed_up_items"], 1);
        assert_eq!(summary["pruned_backup_items"], 1);
        assert_eq!(summary["pruned_backups"], 1);
        assert_eq!(
            summary["backups"][0]["pruned_backups"][0],
            "local:///demo.nfo.taru-backup-1"
        );
        assert_eq!(summary["prune_failures"].as_array().unwrap().len(), 0);
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
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("private-storage-handle"));
    }

    #[test]
    fn selected_artwork_publication_response_redacts_storage_uri() {
        let artifact_id = ManagedArtworkArtifactId::new();
        let selected = SelectedArtworkRecord {
            id: taru_core::SelectedArtworkId::new(),
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
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("managed-artwork://"));
        assert!(!body.contains("private-storage-handle"));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("cache_uri"));
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
            item: taru_core::MediaItem {
                id: MediaItemId::new(),
                kind: MediaKind::Unknown,
                parent_id: None,
                metadata: taru_core::CanonicalMetadata {
                    title: "Private Clip".to_owned(),
                    release_date: Some("2026-05-18".to_owned()),
                    ..taru_core::CanonicalMetadata::default()
                },
            },
            library_id: LibraryId::new(),
            source_count: 1,
            representative_source_id: Some(source_id),
            representative_file_name: Some("Private Clip.mkv".to_owned()),
            best_local_inference: Some(LocalInferenceEvidence {
                id: taru_core::LocalInferenceEvidenceId::new(),
                source_id,
                inferred_kind: MediaKind::Unknown,
                inferred_title: Some("Private Clip".to_owned()),
                inferred_year: None,
                inferred_season: None,
                inferred_episode: None,
                confidence_milli: Some(350),
                evidence_source: LocalInferenceEvidenceSource::Path,
                evidence_value: "local:///Users/admin/Private/Private Clip.mkv".to_owned(),
                inference_version: "taru-naming:1".to_owned(),
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
            output_path: "C:\\taru-cache\\hls\\secret\\playlist.m3u8".into(),
            state: TranscodeSessionState::Failed,
            failure_category: Some(TranscodeFailureCategory::Runner),
            failure_message: Some(
                "ffmpeg failed while writing C:\\taru-cache\\hls\\secret\\playlist.m3u8".to_owned(),
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
        assert!(!body.contains("taru-cache"));
        assert!(!body.contains("playlist.m3u8"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ffmpeg failed while writing"));
    }

    #[test]
    fn admin_playback_runtime_diagnostics_serializes_safe_summary_fields() {
        let response = AdminPlaybackRuntimeDiagnosticsResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: crate::API_VERSION.to_owned(),
            ffmpeg: AdminPlaybackFfmpegDiagnostics {
                probe_status: AdminPlaybackRuntimeStatus::Degraded,
                has_probe_error: true,
                hardware_capability_count: 4,
                available_gpu_capabilities: 1,
            },
            hardware: AdminPlaybackHardwareDiagnostics {
                policy: HardwareAccelerationPolicy {
                    requested: HardwareAcceleration::Nvenc,
                    fallback: taru_transcode::HardwareAccelerationFallback::Cpu,
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
                    evidence: AdminPlaybackHardwareCapabilityEvidence::FfmpegProbeError,
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
        assert_eq!(value["ffmpeg"]["probe_status"], "degraded");
        assert_eq!(value["hardware"]["policy"]["requested"], "nvenc");
        assert_eq!(value["hardware"]["selection"]["acceleration"], "none");
        assert_eq!(
            value["hardware"]["capabilities"][0]["reason_code"],
            "probe_error"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["evidence"],
            "ffmpeg_probe_error"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["smoke_probe"]["status"],
            "not_run"
        );
        assert_eq!(value["remote_playback"]["state_scope"], "process_local");
        assert!(!body.contains("ffmpeg_path"));
        assert!(!body.contains("remux_staging_root"));
        assert!(!body.contains("taru-cache"));
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
            local_path: "F:\\Taru\\secret-cache\\inputs\\Demo.mkv".to_owned(),
            size_bytes: Some(42),
            etag: Some("etag-secret".to_owned()),
            fingerprint: Some("fingerprint-secret".to_owned()),
            state: StagingState::Failed,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(1_300),
            active_leases: 0,
            validation_error: Some("failed at F:\\Taru\\secret-cache".to_owned()),
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
            public_api_version: crate::API_VERSION.to_owned(),
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
                    provider: taru_core::ExternalProvider::Tmdb,
                    status: crate::MetadataProviderDiagnosticStatus::Available,
                }],
            },
            runtime: AdminOverviewRuntimeSummary {
                active_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                succeeded_jobs: 0,
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
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["public_api_version"], crate::API_VERSION);
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
