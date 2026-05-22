use serde::{Deserialize, Serialize};

use crate::{
    AutomationArtifactId, AutomationProviderId, JobId, LibraryId, MediaItemId, MediaSourceId,
    NakoError, Result,
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
