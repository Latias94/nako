use serde::{Deserialize, Serialize};

use crate::{
    ExternalProvider, LibraryId, LocalMetadataPolicy, ManagedImportArtifactId, MediaSourceId,
    Result, SourceDuplicateEvidenceKind, StagingManifestId, TaruError,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportSourceKind {
    OperatorUrl,
    WatchedCandidate,
    AddonProposed,
    LocalFile,
    VfsStaging,
    Other(String),
}

impl ManagedImportSourceKind {
    #[must_use]
    pub fn as_parts(&self) -> (&'static str, &str) {
        match self {
            Self::OperatorUrl => ("operator_url", ""),
            Self::WatchedCandidate => ("watched_candidate", ""),
            Self::AddonProposed => ("addon_proposed", ""),
            Self::LocalFile => ("local_file", ""),
            Self::VfsStaging => ("vfs_staging", ""),
            Self::Other(value) => ("other", value.as_str()),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, kind_key: String) -> Self {
        match kind {
            "operator_url" => Self::OperatorUrl,
            "watched_candidate" => Self::WatchedCandidate,
            "addon_proposed" => Self::AddonProposed,
            "local_file" => Self::LocalFile,
            "vfs_staging" => Self::VfsStaging,
            "other" => Self::Other(kind_key),
            _ => Self::Other(kind.to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportArtifactState {
    Proposed,
    Staging,
    Staged,
    Inspected,
    Planned,
    Accepted,
    Applying,
    Promoted,
    Rejected,
    Failed,
    CleanupPending,
    Cleaned,
}

impl ManagedImportArtifactState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Staging => "staging",
            Self::Staged => "staged",
            Self::Inspected => "inspected",
            Self::Planned => "planned",
            Self::Accepted => "accepted",
            Self::Applying => "applying",
            Self::Promoted => "promoted",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
            Self::CleanupPending => "cleanup_pending",
            Self::Cleaned => "cleaned",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "proposed" => Ok(Self::Proposed),
            "staging" => Ok(Self::Staging),
            "staged" => Ok(Self::Staged),
            "inspected" => Ok(Self::Inspected),
            "planned" => Ok(Self::Planned),
            "accepted" => Ok(Self::Accepted),
            "applying" => Ok(Self::Applying),
            "promoted" => Ok(Self::Promoted),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            "cleanup_pending" => Ok(Self::CleanupPending),
            "cleaned" => Ok(Self::Cleaned),
            _ => Err(TaruError::Database {
                message: format!(
                    "unknown managed import artifact state stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportArtifactListFilter {
    pub target_library_id: Option<LibraryId>,
    pub state: Option<ManagedImportArtifactState>,
    pub source_kind: Option<ManagedImportSourceKind>,
}

impl ManagedImportArtifactListFilter {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            target_library_id: None,
            state: None,
            source_kind: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewManagedImportArtifact {
    pub id: ManagedImportArtifactId,
    pub target_library_id: LibraryId,
    pub source_kind: ManagedImportSourceKind,
    pub source_uri: String,
    pub staging_manifest_id: Option<StagingManifestId>,
    pub artifact_uri: Option<String>,
    pub original_file_name: Option<String>,
    pub intended_locator: Option<String>,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub state: ManagedImportArtifactState,
    pub diagnostics_json: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportArtifactRecord {
    pub id: ManagedImportArtifactId,
    pub target_library_id: LibraryId,
    pub source_kind: ManagedImportSourceKind,
    pub source_uri: String,
    pub staging_manifest_id: Option<StagingManifestId>,
    pub artifact_uri: Option<String>,
    pub original_file_name: Option<String>,
    pub intended_locator: Option<String>,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub state: ManagedImportArtifactState,
    pub diagnostics_json: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportPromotionPlan {
    pub artifact_id: ManagedImportArtifactId,
    pub artifact_state: ManagedImportArtifactState,
    pub target_library_id: LibraryId,
    pub target_library_name: String,
    pub destination_locator: Option<String>,
    pub file_operations: Vec<ManagedImportPromotionFileOperation>,
    pub duplicate_hints: Vec<ManagedImportPromotionDuplicateHint>,
    pub nfo_authority: ManagedImportPromotionNfoAuthorityHint,
    pub provider_identity: ManagedImportPromotionProviderIdentityHint,
    pub blocked_reasons: Vec<ManagedImportPromotionBlockedReason>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportPromotionOperationKind {
    Copy,
    Move,
    Hardlink,
    Symlink,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportPromotionOperationStatus {
    Ready,
    Blocked,
    Unsupported,
    SourceMissing,
    SourceNotFile,
    TargetParentMissing,
    TargetParentNotDirectory,
    TargetExists,
    SecurityViolation,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportPromotionFileOperation {
    pub kind: ManagedImportPromotionOperationKind,
    pub status: ManagedImportPromotionOperationStatus,
    pub can_apply: bool,
    pub source_scheme: Option<String>,
    pub target_locator: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportPromotionDuplicateHint {
    pub existing_source_id: Option<MediaSourceId>,
    pub evidence_kind: SourceDuplicateEvidenceKind,
    pub confidence_milli: Option<u16>,
    pub size_matches: bool,
    pub fingerprint_matches: bool,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportPromotionNfoAuthorityHint {
    pub policy: LocalMetadataPolicy,
    pub sidecar_locator: Option<String>,
    pub has_sidecar: bool,
    pub import_supported: bool,
    pub export_supported: bool,
    pub would_read_sidecar: bool,
    pub would_create_sidecar: bool,
    pub backup_required: bool,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportPromotionProviderIdentityHint {
    pub configured_providers: Vec<ExternalProvider>,
    pub has_import_diagnostics: bool,
    pub needs_identity_review: bool,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportPromotionBlockedReason {
    ArtifactNotReady,
    MissingArtifactUri,
    MissingDestinationLocator,
    InvalidArtifactUri,
    InvalidDestinationLocator,
    DestinationEscapesLibrary,
    ProviderIdentityMissing,
    StoragePlanningUnavailable,
}
