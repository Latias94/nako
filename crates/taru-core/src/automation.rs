use serde::{Deserialize, Serialize};

use crate::{
    AutomationArtifactId, AutomationProviderId, JobId, LibraryId, MediaItemId, MediaSourceId,
    Result, TaruError,
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
            _ => Err(TaruError::Database {
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
            _ => Err(TaruError::Database {
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
            _ => Err(TaruError::Database {
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
            _ => Err(TaruError::Database {
                message: format!("unknown automation artifact status stored in database: {value}"),
            }),
        }
    }
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
