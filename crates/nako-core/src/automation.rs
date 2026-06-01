use serde::{Deserialize, Serialize};

use crate::{
    AutomationArtifactId, AutomationProviderId, ExternalProvider,
    GeneratedArtifactMetadataApplyOutcomeId, GeneratedArtifactMetadataBulkApplyBatchId, JobId,
    LibraryId, MediaItemId, MediaSourceId, MetadataApplicationPersistenceCommit, MetadataField,
    NakoError, NewJob, ProviderMapping, ProviderMappingStatus, ProviderSubject,
    ProviderSubjectKind, Result,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationProviderStatus {
    Enabled,
    Disabled,
}

impl AutomationProviderStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(NakoError::Database {
                message: format!("unknown automation provider status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationCapability {
    Recommendation,
    MetadataCleanup,
    Summary,
    TitleMatch,
}

impl AutomationCapability {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recommendation => "recommendation",
            Self::MetadataCleanup => "metadata_cleanup",
            Self::Summary => "summary",
            Self::TitleMatch => "title_match",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "recommendation" => Ok(Self::Recommendation),
            "metadata_cleanup" => Ok(Self::MetadataCleanup),
            "summary" => Ok(Self::Summary),
            "title_match" => Ok(Self::TitleMatch),
            _ => Err(NakoError::Database {
                message: format!("unknown automation capability stored in database: {value}"),
            }),
        }
    }

    #[must_use]
    pub const fn default_artifact_kind(self) -> AutomationArtifactKind {
        match self {
            Self::Recommendation => AutomationArtifactKind::Recommendation,
            Self::MetadataCleanup => AutomationArtifactKind::MetadataSuggestion,
            Self::Summary => AutomationArtifactKind::Summary,
            Self::TitleMatch => AutomationArtifactKind::TitleMatch,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationArtifactKind {
    Recommendation,
    MetadataSuggestion,
    Summary,
    TitleMatch,
}

impl AutomationArtifactKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recommendation => "recommendation",
            Self::MetadataSuggestion => "metadata_suggestion",
            Self::Summary => "summary",
            Self::TitleMatch => "title_match",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "recommendation" => Ok(Self::Recommendation),
            "metadata_suggestion" => Ok(Self::MetadataSuggestion),
            "summary" => Ok(Self::Summary),
            "title_match" => Ok(Self::TitleMatch),
            _ => Err(NakoError::Database {
                message: format!("unknown automation artifact kind stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationArtifactStatus {
    Proposed,
    Accepted,
    Rejected,
}

impl AutomationArtifactStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            _ => Err(NakoError::Database {
                message: format!("unknown automation artifact status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactTargetKind {
    Library,
    MediaItem,
    MediaSource,
    Untargeted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactTarget {
    pub kind: GeneratedArtifactTargetKind,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
}

impl GeneratedArtifactTarget {
    #[must_use]
    pub fn from_scope(
        library_id: Option<LibraryId>,
        item_id: Option<MediaItemId>,
        source_id: Option<MediaSourceId>,
    ) -> Self {
        let kind = if source_id.is_some() {
            GeneratedArtifactTargetKind::MediaSource
        } else if item_id.is_some() {
            GeneratedArtifactTargetKind::MediaItem
        } else if library_id.is_some() {
            GeneratedArtifactTargetKind::Library
        } else {
            GeneratedArtifactTargetKind::Untargeted
        };

        Self {
            kind,
            library_id,
            item_id,
            source_id,
        }
    }

    #[must_use]
    pub fn from_artifact(artifact: &AutomationArtifactRecord) -> Self {
        Self::from_scope(artifact.library_id, artifact.item_id, artifact.source_id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactReadinessStatus {
    Ready,
    Blocked,
    Stale,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactReadinessReason {
    Ready,
    ArtifactAlreadyAccepted,
    ArtifactAlreadyRejected,
    InvalidPayloadJson,
    TargetRequired,
    UnsupportedTarget,
    MissingJob,
    MissingProvider,
    MissingLibrary,
    MissingMediaItem,
    MissingMediaSource,
    TargetMismatch,
    JobInputMismatch,
}

impl GeneratedArtifactReadinessReason {
    #[must_use]
    pub const fn is_stale(self) -> bool {
        matches!(
            self,
            Self::MissingJob
                | Self::MissingProvider
                | Self::MissingLibrary
                | Self::MissingMediaItem
                | Self::MissingMediaSource
                | Self::TargetMismatch
                | Self::JobInputMismatch
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactReadiness {
    pub status: GeneratedArtifactReadinessStatus,
    pub actionable: bool,
    pub reasons: Vec<GeneratedArtifactReadinessReason>,
}

impl GeneratedArtifactReadiness {
    #[must_use]
    pub fn ready() -> Self {
        Self {
            status: GeneratedArtifactReadinessStatus::Ready,
            actionable: true,
            reasons: vec![GeneratedArtifactReadinessReason::Ready],
        }
    }

    #[must_use]
    pub fn from_reasons(reasons: Vec<GeneratedArtifactReadinessReason>) -> Self {
        if reasons.is_empty() {
            return Self::ready();
        }

        let status = if reasons.iter().any(|reason| reason.is_stale()) {
            GeneratedArtifactReadinessStatus::Stale
        } else {
            GeneratedArtifactReadinessStatus::Blocked
        };

        Self {
            status,
            actionable: false,
            reasons,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactPayloadShape {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
    InvalidJson,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactPayloadSummary {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactProvenance {
    pub provider_id: AutomationProviderId,
    pub provider_name: Option<String>,
    pub job_id: JobId,
    pub capability: AutomationCapability,
    pub idempotency_key_fingerprint: Option<String>,
    pub prompt_fingerprint: Option<String>,
    pub attempt_count: Option<u32>,
    pub artifact_created_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactProposal {
    pub id: AutomationArtifactId,
    pub kind: AutomationArtifactKind,
    pub capability: AutomationCapability,
    pub status: AutomationArtifactStatus,
    pub target: GeneratedArtifactTarget,
    pub provenance: GeneratedArtifactProvenance,
    pub payload: GeneratedArtifactPayloadSummary,
    pub readiness: GeneratedArtifactReadiness,
    pub created_at: String,
    pub updated_at: String,
    pub accepted_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactReviewDecision {
    Accept,
    Reject,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactAcceptancePlanStatus {
    Ready,
    Blocked,
    Stale,
    AlreadyAccepted,
    AlreadyRejected,
}

impl GeneratedArtifactAcceptancePlanStatus {
    #[must_use]
    pub const fn executable(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactAcceptanceActionKind {
    StageMetadataAuthorityReview,
    RejectProposal,
    Noop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactAcceptancePlanReason {
    Ready,
    OperatorRejected,
    MetadataAuthorityApplyRequired,
    ProposalNotReady,
    UnsupportedArtifactKind,
    MissingMediaItemTarget,
    ArtifactAlreadyAccepted,
    ArtifactAlreadyRejected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactAcceptanceBoundary {
    pub accepted_into_canonical_metadata: bool,
    pub writes_sidecar: bool,
    pub writes_library_files: bool,
    pub applies_immediately: bool,
    pub requires_metadata_authority_apply: bool,
}

impl GeneratedArtifactAcceptanceBoundary {
    #[must_use]
    pub const fn deferred_metadata_authority() -> Self {
        Self {
            accepted_into_canonical_metadata: false,
            writes_sidecar: false,
            writes_library_files: false,
            applies_immediately: false,
            requires_metadata_authority_apply: true,
        }
    }

    #[must_use]
    pub const fn no_mutation() -> Self {
        Self {
            accepted_into_canonical_metadata: false,
            writes_sidecar: false,
            writes_library_files: false,
            applies_immediately: false,
            requires_metadata_authority_apply: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactAcceptancePlan {
    pub artifact_id: AutomationArtifactId,
    pub decision: GeneratedArtifactReviewDecision,
    pub status: GeneratedArtifactAcceptancePlanStatus,
    pub action: GeneratedArtifactAcceptanceActionKind,
    pub reasons: Vec<GeneratedArtifactAcceptancePlanReason>,
    pub capability: AutomationCapability,
    pub kind: AutomationArtifactKind,
    pub target: GeneratedArtifactTarget,
    pub payload: GeneratedArtifactPayloadSummary,
    pub readiness: GeneratedArtifactReadiness,
    pub boundary: GeneratedArtifactAcceptanceBoundary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactReviewResult {
    pub artifact_id: AutomationArtifactId,
    pub decision: GeneratedArtifactReviewDecision,
    pub artifact_status: AutomationArtifactStatus,
    pub accepted_at: Option<String>,
    pub idempotent_replay: bool,
    pub plan: GeneratedArtifactAcceptancePlan,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyPlanStatus {
    Ready,
    Blocked,
    Stale,
}

impl GeneratedArtifactMetadataApplyPlanStatus {
    #[must_use]
    pub const fn executable(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyPlanReason {
    Ready,
    ArtifactNotAccepted,
    UnsupportedArtifactKind,
    MissingLibraryTarget,
    MissingLibrary,
    MissingMediaItemTarget,
    MissingMediaItem,
    MissingMediaSource,
    TargetMismatch,
    InvalidPayloadJson,
    PayloadMustBeObject,
    NoSupportedMetadataFields,
    NoApplicableMetadataFields,
    ProviderMappingApplyDeferred,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataFieldAction {
    Apply,
    Skip,
    Noop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataFieldReason {
    Ready,
    FieldLocked,
    ExistingValuePresent,
    Unchanged,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataValueSummary {
    pub present: bool,
    pub empty: bool,
    pub value_fingerprint: Option<String>,
    pub value_bytes: Option<u64>,
    pub item_count: Option<u32>,
}

impl GeneratedArtifactMetadataValueSummary {
    #[must_use]
    pub const fn missing() -> Self {
        Self {
            present: false,
            empty: true,
            value_fingerprint: None,
            value_bytes: None,
            item_count: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyFieldPlan {
    pub field: MetadataField,
    pub action: GeneratedArtifactMetadataFieldAction,
    pub reasons: Vec<GeneratedArtifactMetadataFieldReason>,
    pub current: GeneratedArtifactMetadataValueSummary,
    pub incoming: GeneratedArtifactMetadataValueSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactProviderMappingAction {
    Apply,
    Skip,
    Noop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactProviderMappingReason {
    Ready,
    UnsupportedProvider,
    UnsupportedSubjectKind,
    MissingSubjectKey,
    InvalidSubjectKey,
    DuplicateProposal,
    ExistingAcceptedMapping,
    ExistingCandidateMapping,
    ExistingRejectedMapping,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactProviderSubjectPlan {
    pub provider: Option<ExternalProvider>,
    pub provider_name: Option<String>,
    pub subject_kind: Option<ProviderSubjectKind>,
    pub subject_kind_name: Option<String>,
    pub subject_key: Option<String>,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub locale: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactProviderMappingPlan {
    pub subject: GeneratedArtifactProviderSubjectPlan,
    pub action: GeneratedArtifactProviderMappingAction,
    pub reasons: Vec<GeneratedArtifactProviderMappingReason>,
    pub confidence_milli: Option<u16>,
    pub existing_mapping_status: Option<ProviderMappingStatus>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyPlan {
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataApplyPlanStatus,
    pub executable: bool,
    pub reasons: Vec<GeneratedArtifactMetadataApplyPlanReason>,
    pub target: GeneratedArtifactTarget,
    pub payload: GeneratedArtifactPayloadSummary,
    pub fields: Vec<GeneratedArtifactMetadataApplyFieldPlan>,
    pub provider_mappings: Vec<GeneratedArtifactProviderMappingPlan>,
    pub apply_field_count: u32,
    pub skipped_field_count: u32,
    pub noop_field_count: u32,
    pub apply_provider_mapping_count: u32,
    pub skipped_provider_mapping_count: u32,
    pub noop_provider_mapping_count: u32,
}

pub const GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS: usize = 100;
pub const GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS: &str =
    "metadata.generated_artifact_bulk_apply";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyPlanRequest {
    pub artifact_ids: Vec<AutomationArtifactId>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyPlanSelection {
    pub requested_artifact_count: u32,
    pub selected_artifact_count: u32,
    pub duplicate_artifact_count: u32,
    pub max_artifact_count: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct GeneratedArtifactMetadataBulkApplyPlanSummary {
    pub planned_artifact_count: u32,
    pub missing_artifact_count: u32,
    pub ready_artifact_count: u32,
    pub blocked_artifact_count: u32,
    pub stale_artifact_count: u32,
    pub executable_artifact_count: u32,
    pub apply_field_count: u32,
    pub skipped_field_count: u32,
    pub noop_field_count: u32,
    pub apply_provider_mapping_count: u32,
    pub skipped_provider_mapping_count: u32,
    pub noop_provider_mapping_count: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataBulkApplyPlanItemStatus {
    Planned,
    Missing,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataBulkApplyPlanItemReason {
    Planned,
    MissingArtifact,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyPlanItem {
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataBulkApplyPlanItemStatus,
    pub executable: bool,
    pub reasons: Vec<GeneratedArtifactMetadataBulkApplyPlanItemReason>,
    pub plan: Option<GeneratedArtifactMetadataApplyPlan>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyPlan {
    pub selection: GeneratedArtifactMetadataBulkApplyPlanSelection,
    pub summary: GeneratedArtifactMetadataBulkApplyPlanSummary,
    pub items: Vec<GeneratedArtifactMetadataBulkApplyPlanItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchRequest {
    pub artifact_ids: Vec<AutomationArtifactId>,
    pub idempotency_key: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataBulkApplyBatchStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl GeneratedArtifactMetadataBulkApplyBatchStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown generated artifact metadata bulk apply batch status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataBulkApplyBatchItemStatus {
    Pending,
    Skipped,
    Applied,
    Noop,
    Stale,
    Failed,
}

impl GeneratedArtifactMetadataBulkApplyBatchItemStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Skipped => "skipped",
            Self::Applied => "applied",
            Self::Noop => "noop",
            Self::Stale => "stale",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "skipped" => Ok(Self::Skipped),
            "applied" => Ok(Self::Applied),
            "noop" => Ok(Self::Noop),
            "stale" => Ok(Self::Stale),
            "failed" => Ok(Self::Failed),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown generated artifact metadata bulk apply batch item status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchItemCommit {
    pub artifact_id: AutomationArtifactId,
    pub position: u32,
    pub status: GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    pub idempotency_key: String,
    pub plan_item: GeneratedArtifactMetadataBulkApplyPlanItem,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchCommit {
    pub id: GeneratedArtifactMetadataBulkApplyBatchId,
    pub job: NewJob,
    pub idempotency_key: String,
    pub status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    pub selection: GeneratedArtifactMetadataBulkApplyPlanSelection,
    pub summary: GeneratedArtifactMetadataBulkApplyPlanSummary,
    pub items: Vec<GeneratedArtifactMetadataBulkApplyBatchItemCommit>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchExecutionSummary {
    pub total_item_count: u32,
    pub pending_item_count: u32,
    pub skipped_item_count: u32,
    pub applied_item_count: u32,
    pub noop_item_count: u32,
    pub stale_item_count: u32,
    pub failed_item_count: u32,
}

impl GeneratedArtifactMetadataBulkApplyBatchExecutionSummary {
    #[must_use]
    pub fn from_items(items: &[GeneratedArtifactMetadataBulkApplyBatchItemRecord]) -> Self {
        let mut summary = Self {
            total_item_count: items.len() as u32,
            ..Self::default()
        };
        for item in items {
            match item.status {
                GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending => {
                    summary.pending_item_count = summary.pending_item_count.saturating_add(1);
                }
                GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped => {
                    summary.skipped_item_count = summary.skipped_item_count.saturating_add(1);
                }
                GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied => {
                    summary.applied_item_count = summary.applied_item_count.saturating_add(1);
                }
                GeneratedArtifactMetadataBulkApplyBatchItemStatus::Noop => {
                    summary.noop_item_count = summary.noop_item_count.saturating_add(1);
                }
                GeneratedArtifactMetadataBulkApplyBatchItemStatus::Stale => {
                    summary.stale_item_count = summary.stale_item_count.saturating_add(1);
                }
                GeneratedArtifactMetadataBulkApplyBatchItemStatus::Failed => {
                    summary.failed_item_count = summary.failed_item_count.saturating_add(1);
                }
            }
        }
        summary
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit {
    pub batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    pub outcome_id: Option<GeneratedArtifactMetadataApplyOutcomeId>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchItemRecord {
    pub batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    pub artifact_id: AutomationArtifactId,
    pub position: u32,
    pub status: GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    pub idempotency_key: String,
    pub outcome_id: Option<GeneratedArtifactMetadataApplyOutcomeId>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub plan_item: GeneratedArtifactMetadataBulkApplyPlanItem,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataBulkApplyBatchRecord {
    pub id: GeneratedArtifactMetadataBulkApplyBatchId,
    pub job_id: JobId,
    pub idempotency_key: String,
    pub status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    pub selection: GeneratedArtifactMetadataBulkApplyPlanSelection,
    pub summary: GeneratedArtifactMetadataBulkApplyPlanSummary,
    pub execution_summary: GeneratedArtifactMetadataBulkApplyBatchExecutionSummary,
    pub items: Vec<GeneratedArtifactMetadataBulkApplyBatchItemRecord>,
    pub created_at: String,
    pub updated_at: String,
}

impl GeneratedArtifactMetadataBulkApplyBatchRecord {
    #[must_use]
    pub fn plan(&self) -> GeneratedArtifactMetadataBulkApplyPlan {
        GeneratedArtifactMetadataBulkApplyPlan {
            selection: self.selection.clone(),
            summary: self.summary.clone(),
            items: self
                .items
                .iter()
                .map(|item| item.plan_item.clone())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyRequest {
    pub artifact_id: AutomationArtifactId,
    pub idempotency_key: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyResultStatus {
    Applied,
    Noop,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyResult {
    pub outcome_id: Option<GeneratedArtifactMetadataApplyOutcomeId>,
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataApplyResultStatus,
    pub applied: bool,
    pub changed: bool,
    pub idempotent_replay: bool,
    pub applied_source: Option<String>,
    pub plan: GeneratedArtifactMetadataApplyPlan,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyOutcomeStatus {
    Applied,
    Noop,
    Failed,
}

impl GeneratedArtifactMetadataApplyOutcomeStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Noop => "noop",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "applied" => Ok(Self::Applied),
            "noop" => Ok(Self::Noop),
            "failed" => Ok(Self::Failed),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown generated artifact metadata apply outcome status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyOutcomeRecord {
    pub id: GeneratedArtifactMetadataApplyOutcomeId,
    pub artifact_id: AutomationArtifactId,
    pub idempotency_key: String,
    pub status: GeneratedArtifactMetadataApplyOutcomeStatus,
    pub applied: bool,
    pub changed: bool,
    pub applied_source: Option<String>,
    pub item_id: Option<MediaItemId>,
    pub plan: GeneratedArtifactMetadataApplyPlan,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyRecoveryAttention {
    NeedsRepair,
    NeedsReview,
    ReplayOnly,
    Resolved,
}

impl GeneratedArtifactMetadataApplyRecoveryAttention {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NeedsRepair => "needs_repair",
            Self::NeedsReview => "needs_review",
            Self::ReplayOnly => "replay_only",
            Self::Resolved => "resolved",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "needs_repair" => Ok(Self::NeedsRepair),
            "needs_review" => Ok(Self::NeedsReview),
            "replay_only" => Ok(Self::ReplayOnly),
            "resolved" => Ok(Self::Resolved),
            _ => Err(NakoError::InvalidInput {
                message: format!(
                    "unknown generated artifact metadata apply recovery attention: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyRecoverySource {
    ApplyOutcome,
    BulkBatchItem,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactMetadataApplyRecoveryReason {
    ApplyOutcomeApplied,
    ApplyOutcomeNoop,
    ApplyOutcomeFailed,
    BulkBatchItemApplied,
    BulkBatchItemNoop,
    BulkBatchItemStale,
    BulkBatchItemFailed,
    BulkBatchItemSkipped,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyRecoveryFilter {
    pub attention: Option<GeneratedArtifactMetadataApplyRecoveryAttention>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyRecoveryEntryRecord {
    pub source: GeneratedArtifactMetadataApplyRecoverySource,
    pub attention: GeneratedArtifactMetadataApplyRecoveryAttention,
    pub reason: GeneratedArtifactMetadataApplyRecoveryReason,
    pub artifact_id: AutomationArtifactId,
    pub outcome_id: Option<GeneratedArtifactMetadataApplyOutcomeId>,
    pub batch_id: Option<GeneratedArtifactMetadataBulkApplyBatchId>,
    pub batch_item_status: Option<GeneratedArtifactMetadataBulkApplyBatchItemStatus>,
    pub outcome_status: Option<GeneratedArtifactMetadataApplyOutcomeStatus>,
    pub item_id: Option<MediaItemId>,
    pub plan: Option<GeneratedArtifactMetadataApplyPlan>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl GeneratedArtifactMetadataApplyRecoveryEntryRecord {
    pub fn classify(
        source: GeneratedArtifactMetadataApplyRecoverySource,
        batch_item_status: Option<GeneratedArtifactMetadataBulkApplyBatchItemStatus>,
        outcome_status: Option<GeneratedArtifactMetadataApplyOutcomeStatus>,
    ) -> Result<(
        GeneratedArtifactMetadataApplyRecoveryAttention,
        GeneratedArtifactMetadataApplyRecoveryReason,
    )> {
        match source {
            GeneratedArtifactMetadataApplyRecoverySource::ApplyOutcome => match outcome_status {
                Some(GeneratedArtifactMetadataApplyOutcomeStatus::Applied) => Ok((
                    GeneratedArtifactMetadataApplyRecoveryAttention::Resolved,
                    GeneratedArtifactMetadataApplyRecoveryReason::ApplyOutcomeApplied,
                )),
                Some(GeneratedArtifactMetadataApplyOutcomeStatus::Noop) => Ok((
                    GeneratedArtifactMetadataApplyRecoveryAttention::ReplayOnly,
                    GeneratedArtifactMetadataApplyRecoveryReason::ApplyOutcomeNoop,
                )),
                Some(GeneratedArtifactMetadataApplyOutcomeStatus::Failed) => Ok((
                    GeneratedArtifactMetadataApplyRecoveryAttention::NeedsRepair,
                    GeneratedArtifactMetadataApplyRecoveryReason::ApplyOutcomeFailed,
                )),
                None => Err(NakoError::Database {
                    message:
                        "generated artifact metadata apply recovery outcome row is missing outcome status"
                            .to_owned(),
                }),
            },
            GeneratedArtifactMetadataApplyRecoverySource::BulkBatchItem => {
                match batch_item_status {
                    Some(GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied) => Ok((
                        GeneratedArtifactMetadataApplyRecoveryAttention::Resolved,
                        GeneratedArtifactMetadataApplyRecoveryReason::BulkBatchItemApplied,
                    )),
                    Some(GeneratedArtifactMetadataBulkApplyBatchItemStatus::Noop) => Ok((
                        GeneratedArtifactMetadataApplyRecoveryAttention::ReplayOnly,
                        GeneratedArtifactMetadataApplyRecoveryReason::BulkBatchItemNoop,
                    )),
                    Some(GeneratedArtifactMetadataBulkApplyBatchItemStatus::Stale) => Ok((
                        GeneratedArtifactMetadataApplyRecoveryAttention::NeedsRepair,
                        GeneratedArtifactMetadataApplyRecoveryReason::BulkBatchItemStale,
                    )),
                    Some(GeneratedArtifactMetadataBulkApplyBatchItemStatus::Failed) => Ok((
                        GeneratedArtifactMetadataApplyRecoveryAttention::NeedsRepair,
                        GeneratedArtifactMetadataApplyRecoveryReason::BulkBatchItemFailed,
                    )),
                    Some(GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped) => Ok((
                        GeneratedArtifactMetadataApplyRecoveryAttention::NeedsReview,
                        GeneratedArtifactMetadataApplyRecoveryReason::BulkBatchItemSkipped,
                    )),
                    Some(GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending) | None => {
                        Err(NakoError::Database {
                            message:
                                "generated artifact metadata apply recovery batch row is not terminal"
                                    .to_owned(),
                        })
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn returned_summary(
        entries: &[GeneratedArtifactMetadataApplyRecoveryEntryRecord],
    ) -> GeneratedArtifactMetadataApplyRecoverySummary {
        let mut summary = GeneratedArtifactMetadataApplyRecoverySummary {
            returned_entry_count: entries.len() as u32,
            needs_repair_count: 0,
            needs_review_count: 0,
            replay_only_count: 0,
            resolved_count: 0,
        };
        let mut index = 0;
        while index < entries.len() {
            match entries[index].attention {
                GeneratedArtifactMetadataApplyRecoveryAttention::NeedsRepair => {
                    summary.needs_repair_count += 1;
                }
                GeneratedArtifactMetadataApplyRecoveryAttention::NeedsReview => {
                    summary.needs_review_count += 1;
                }
                GeneratedArtifactMetadataApplyRecoveryAttention::ReplayOnly => {
                    summary.replay_only_count += 1;
                }
                GeneratedArtifactMetadataApplyRecoveryAttention::Resolved => {
                    summary.resolved_count += 1;
                }
            }
            index += 1;
        }
        summary
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyRecoverySummary {
    pub returned_entry_count: u32,
    pub needs_repair_count: u32,
    pub needs_review_count: u32,
    pub replay_only_count: u32,
    pub resolved_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactProviderMappingApplyCommit {
    pub subject: ProviderSubject,
    pub mapping: ProviderMapping,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedArtifactMetadataApplyOutcomeCommit {
    pub id: GeneratedArtifactMetadataApplyOutcomeId,
    pub artifact_id: AutomationArtifactId,
    pub idempotency_key: String,
    pub status: GeneratedArtifactMetadataApplyOutcomeStatus,
    pub applied: bool,
    pub changed: bool,
    pub applied_source: Option<String>,
    pub item_id: Option<MediaItemId>,
    pub plan: GeneratedArtifactMetadataApplyPlan,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub metadata_application: Option<MetadataApplicationPersistenceCommit>,
    pub provider_mappings: Vec<GeneratedArtifactProviderMappingApplyCommit>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAutomationProviderConfig {
    pub id: AutomationProviderId,
    pub name: String,
    pub base_url: String,
    pub secret_env: Option<String>,
    pub capabilities: Vec<AutomationCapability>,
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub status: AutomationProviderStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProviderConfigRecord {
    pub id: AutomationProviderId,
    pub name: String,
    pub base_url: String,
    pub secret_env: Option<String>,
    pub capabilities: Vec<AutomationCapability>,
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub status: AutomationProviderStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationJobInput {
    pub provider_id: AutomationProviderId,
    pub capability: AutomationCapability,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub prompt_json: String,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationJobSummary {
    pub provider_id: AutomationProviderId,
    pub capability: AutomationCapability,
    pub accepted_into_canonical_metadata: bool,
    pub artifact_ids: Vec<AutomationArtifactId>,
    pub output_json: String,
    pub attempt_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAutomationArtifact {
    pub id: AutomationArtifactId,
    pub job_id: JobId,
    pub provider_id: AutomationProviderId,
    pub capability: AutomationCapability,
    pub kind: AutomationArtifactKind,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub artifact_json: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationArtifactRecord {
    pub id: AutomationArtifactId,
    pub job_id: JobId,
    pub provider_id: AutomationProviderId,
    pub capability: AutomationCapability,
    pub kind: AutomationArtifactKind,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub artifact_json: String,
    pub status: AutomationArtifactStatus,
    pub created_at: String,
    pub updated_at: String,
    pub accepted_at: Option<String>,
}
