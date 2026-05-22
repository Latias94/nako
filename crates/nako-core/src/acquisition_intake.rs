use serde::{Deserialize, Serialize};

use crate::{AcquisitionIntakeCandidateId, LibraryId, ManagedImportArtifactId, NakoError, Result};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionIntakeSourceKind {
    WatchFolder,
    OperatorSubmitted,
    ExternalDownloadOutput,
    AddonProposed,
    Other(String),
}

impl AcquisitionIntakeSourceKind {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::WatchFolder => ("watch_folder", ""),
            Self::OperatorSubmitted => ("operator_submitted", ""),
            Self::ExternalDownloadOutput => ("external_download_output", ""),
            Self::AddonProposed => ("addon_proposed", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, kind_key: String) -> Self {
        match kind {
            "watch_folder" => Self::WatchFolder,
            "operator_submitted" => Self::OperatorSubmitted,
            "external_download_output" => Self::ExternalDownloadOutput,
            "addon_proposed" => Self::AddonProposed,
            "other" => Self::Other(kind_key),
            _ => Self::Other(kind.to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionIntakeCandidateState {
    Discovered,
    Inspecting,
    Ready,
    Blocked,
    Accepted,
    Rejected,
    Failed,
    Superseded,
}

impl AcquisitionIntakeCandidateState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Inspecting => "inspecting",
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "discovered" => Ok(Self::Discovered),
            "inspecting" => Ok(Self::Inspecting),
            "ready" => Ok(Self::Ready),
            "blocked" => Ok(Self::Blocked),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            "superseded" => Ok(Self::Superseded),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown acquisition intake candidate state stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcquisitionIntakeCandidateListFilter {
    pub target_library_id: Option<LibraryId>,
    pub state: Option<AcquisitionIntakeCandidateState>,
    pub source_kind: Option<AcquisitionIntakeSourceKind>,
    pub managed_import_artifact_id: Option<ManagedImportArtifactId>,
}

impl AcquisitionIntakeCandidateListFilter {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            target_library_id: None,
            state: None,
            source_kind: None,
            managed_import_artifact_id: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAcquisitionIntakeCandidate {
    pub id: AcquisitionIntakeCandidateId,
    pub target_library_id: LibraryId,
    pub source_kind: AcquisitionIntakeSourceKind,
    pub source_key: String,
    pub source_uri: String,
    pub display_name: Option<String>,
    pub intended_locator: Option<String>,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub managed_import_artifact_id: Option<ManagedImportArtifactId>,
    pub state: AcquisitionIntakeCandidateState,
    pub diagnostics_json: Option<String>,
    pub first_seen_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcquisitionIntakeCandidateRecord {
    pub id: AcquisitionIntakeCandidateId,
    pub target_library_id: LibraryId,
    pub source_kind: AcquisitionIntakeSourceKind,
    pub source_key: String,
    pub source_uri: String,
    pub display_name: Option<String>,
    pub intended_locator: Option<String>,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub managed_import_artifact_id: Option<ManagedImportArtifactId>,
    pub state: AcquisitionIntakeCandidateState,
    pub diagnostics_json: Option<String>,
    pub first_seen_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}
