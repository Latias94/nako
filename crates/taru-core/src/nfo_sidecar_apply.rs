use serde::{Deserialize, Serialize};

use crate::{
    LibraryId, MediaItemId, MediaSourceId, NfoSidecarApplyId, Result, TaruError, UserPrincipalId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NfoSidecarApplyOperationKind {
    ExportSidecar,
    ImportSidecar,
    RoundTripUpdate,
}

impl NfoSidecarApplyOperationKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExportSidecar => "export_sidecar",
            Self::ImportSidecar => "import_sidecar",
            Self::RoundTripUpdate => "round_trip_update",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "export_sidecar" => Ok(Self::ExportSidecar),
            "import_sidecar" => Ok(Self::ImportSidecar),
            "round_trip_update" => Ok(Self::RoundTripUpdate),
            _ => Err(TaruError::Database {
                message: format!("unknown NFO sidecar apply operation stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NfoSidecarApplyState {
    Requested,
    ValidatingPreview,
    Accepted,
    WritingSidecar,
    ImportingMetadata,
    Committed,
    Rejected,
    FailedBeforeMutation,
    RepairPending,
    RollbackComplete,
}

impl NfoSidecarApplyState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::ValidatingPreview => "validating_preview",
            Self::Accepted => "accepted",
            Self::WritingSidecar => "writing_sidecar",
            Self::ImportingMetadata => "importing_metadata",
            Self::Committed => "committed",
            Self::Rejected => "rejected",
            Self::FailedBeforeMutation => "failed_before_mutation",
            Self::RepairPending => "repair_pending",
            Self::RollbackComplete => "rollback_complete",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "requested" => Ok(Self::Requested),
            "validating_preview" => Ok(Self::ValidatingPreview),
            "accepted" => Ok(Self::Accepted),
            "writing_sidecar" => Ok(Self::WritingSidecar),
            "importing_metadata" => Ok(Self::ImportingMetadata),
            "committed" => Ok(Self::Committed),
            "rejected" => Ok(Self::Rejected),
            "failed_before_mutation" => Ok(Self::FailedBeforeMutation),
            "repair_pending" => Ok(Self::RepairPending),
            "rollback_complete" => Ok(Self::RollbackComplete),
            _ => Err(TaruError::Database {
                message: format!("unknown NFO sidecar apply state stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewNfoSidecarApply {
    pub id: NfoSidecarApplyId,
    pub target_library_id: LibraryId,
    pub media_item_id: MediaItemId,
    pub media_source_id: Option<MediaSourceId>,
    pub requested_by: UserPrincipalId,
    pub idempotency_key: String,
    pub operation_kind: NfoSidecarApplyOperationKind,
    pub sidecar_locator: String,
    pub accepted_preview_json: String,
    pub accepted_warnings_json: Option<String>,
    pub policy_version: String,
    pub state: NfoSidecarApplyState,
    pub outcome_json: Option<String>,
    pub safe_error_code: Option<String>,
    pub safe_message: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NfoSidecarApplyRecord {
    pub id: NfoSidecarApplyId,
    pub target_library_id: LibraryId,
    pub media_item_id: MediaItemId,
    pub media_source_id: Option<MediaSourceId>,
    pub requested_by: UserPrincipalId,
    pub idempotency_key: String,
    pub operation_kind: NfoSidecarApplyOperationKind,
    pub sidecar_locator: String,
    pub accepted_preview_json: String,
    pub accepted_warnings_json: Option<String>,
    pub policy_version: String,
    pub state: NfoSidecarApplyState,
    pub outcome_json: Option<String>,
    pub safe_error_code: Option<String>,
    pub safe_message: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}
