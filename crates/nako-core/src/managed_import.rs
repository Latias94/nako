use serde::{Deserialize, Serialize};

use crate::{
    ExternalProvider, LibraryId, LocalMetadataPolicy, ManagedImportArtifactId,
    ManagedImportPromotionApplyId, MediaSourceId, NakoError, Result, SourceDuplicateEvidenceKind,
    StagingManifestId, UserPrincipalId,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportSourceKind {
    OperatorUrl,
    WatchedCandidate,
    AddonProposed,
    ResourceSearchSelection,
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
            Self::ResourceSearchSelection => ("resource_search_selection", ""),
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
            "resource_search_selection" => Self::ResourceSearchSelection,
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
            _ => Err(NakoError::Database {
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

impl ManagedImportPromotionOperationKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Move => "move",
            Self::Hardlink => "hardlink",
            Self::Symlink => "symlink",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "copy" => Ok(Self::Copy),
            "move" => Ok(Self::Move),
            "hardlink" => Ok(Self::Hardlink),
            "symlink" => Ok(Self::Symlink),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown managed import promotion operation stored in database: {value}"
                ),
            }),
        }
    }
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedImportPromotionApplyState {
    Requested,
    Validating,
    Accepted,
    ApplyingStorage,
    CommittingCatalog,
    Promoted,
    Rejected,
    FailedBeforeMutation,
    CleanupPending,
    CleanupComplete,
    RollbackComplete,
}

impl ManagedImportPromotionApplyState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Validating => "validating",
            Self::Accepted => "accepted",
            Self::ApplyingStorage => "applying_storage",
            Self::CommittingCatalog => "committing_catalog",
            Self::Promoted => "promoted",
            Self::Rejected => "rejected",
            Self::FailedBeforeMutation => "failed_before_mutation",
            Self::CleanupPending => "cleanup_pending",
            Self::CleanupComplete => "cleanup_complete",
            Self::RollbackComplete => "rollback_complete",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "requested" => Ok(Self::Requested),
            "validating" => Ok(Self::Validating),
            "accepted" => Ok(Self::Accepted),
            "applying_storage" => Ok(Self::ApplyingStorage),
            "committing_catalog" => Ok(Self::CommittingCatalog),
            "promoted" => Ok(Self::Promoted),
            "rejected" => Ok(Self::Rejected),
            "failed_before_mutation" => Ok(Self::FailedBeforeMutation),
            "cleanup_pending" => Ok(Self::CleanupPending),
            "cleanup_complete" => Ok(Self::CleanupComplete),
            "rollback_complete" => Ok(Self::RollbackComplete),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown managed import promotion apply state stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewManagedImportPromotionApply {
    pub id: ManagedImportPromotionApplyId,
    pub artifact_id: ManagedImportArtifactId,
    pub target_library_id: LibraryId,
    pub requested_by: UserPrincipalId,
    pub idempotency_key: String,
    pub operation_kind: ManagedImportPromotionOperationKind,
    pub source_artifact_uri: Option<String>,
    pub destination_locator: String,
    pub accepted_plan_json: String,
    pub accepted_warnings_json: Option<String>,
    pub state: ManagedImportPromotionApplyState,
    pub outcome_json: Option<String>,
    pub safe_error_code: Option<String>,
    pub safe_message: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManagedImportPromotionApplyRecord {
    pub id: ManagedImportPromotionApplyId,
    pub artifact_id: ManagedImportArtifactId,
    pub target_library_id: LibraryId,
    pub requested_by: UserPrincipalId,
    pub idempotency_key: String,
    pub operation_kind: ManagedImportPromotionOperationKind,
    pub source_artifact_uri: Option<String>,
    pub destination_locator: String,
    pub accepted_plan_json: String,
    pub accepted_warnings_json: Option<String>,
    pub state: ManagedImportPromotionApplyState,
    pub outcome_json: Option<String>,
    pub safe_error_code: Option<String>,
    pub safe_message: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}
