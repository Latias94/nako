use nako_core::{
    ManagedImportPromotionApplyRecord, ManagedImportPromotionApplyState,
    ManagedImportPromotionBlockedReason, ManagedImportPromotionOperationKind,
    ManagedImportPromotionPlan, NakoError, Result,
};
use nako_vfs::{
    StorageApplyReport, StorageApplyStatus, StorageCleanupReport, StorageCleanupStatus,
};

use super::{PromotionCatalogCommit, PromotionCatalogCommitFailure};

pub(super) fn accepted_promotion_plan_json(
    plan: &ManagedImportPromotionPlan,
    operation_kind: ManagedImportPromotionOperationKind,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "artifact_id": plan.artifact_id,
        "artifact_state": plan.artifact_state,
        "target_library_id": plan.target_library_id,
        "destination_locator": plan.destination_locator,
        "operation_kind": operation_kind,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "nfo_policy": plan.nfo_authority.policy,
        "provider_identity_review": plan.provider_identity.needs_identity_review,
        "blocked_reasons": plan.blocked_reasons,
        "writes_library": false
    }))
    .map_err(database_error)
}

pub(super) fn accepted_blocked_reasons_json(
    accepted_blocked_reasons: &[ManagedImportPromotionBlockedReason],
    has_duplicate_hints: bool,
    nfo_backup_required: bool,
    provider_identity_review: bool,
) -> Result<Option<String>> {
    if accepted_blocked_reasons.is_empty()
        && !has_duplicate_hints
        && !nfo_backup_required
        && !provider_identity_review
    {
        return Ok(None);
    }

    serde_json::to_string(&serde_json::json!({
        "accepted_blocked_reasons": accepted_blocked_reasons,
        "has_duplicate_hints": has_duplicate_hints,
        "nfo_backup_required": nfo_backup_required,
        "provider_identity_review": provider_identity_review
    }))
    .map(Some)
    .map_err(database_error)
}

pub(super) fn storage_applying_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": false,
        "media_source_mutation": false,
        "operation_kind": record.operation_kind,
        "state": ManagedImportPromotionApplyState::ApplyingStorage
    }))
    .map_err(database_error)
}

pub(super) fn storage_applied_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    plan: &ManagedImportPromotionPlan,
    apply_report: &StorageApplyReport,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": apply_report.applied,
        "media_source_mutation": false,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme()
    }))
    .map_err(database_error)
}

pub(super) fn promoted_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    plan: &ManagedImportPromotionPlan,
    apply_report: &StorageApplyReport,
    catalog_commit: &PromotionCatalogCommit,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": true,
        "storage_mutation": apply_report.applied,
        "media_source_mutation": true,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme(),
        "destination_locator": record.destination_locator,
        "media_item_id": catalog_commit.item_id,
        "media_source_id": catalog_commit.source_id,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "duplicate_relationship_count": catalog_commit.duplicate_relationship_count
    }))
    .map_err(database_error)
}

pub(super) fn post_storage_catalog_failure_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    plan: &ManagedImportPromotionPlan,
    apply_report: &StorageApplyReport,
    cleanup_report: &StorageCleanupReport,
    failure: &PromotionCatalogCommitFailure,
    cleanup_complete: bool,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": failure.catalog_may_have_partial_writes,
        "storage_mutation": apply_report.applied,
        "media_source_mutation": failure.media_source_may_have_been_written,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme(),
        "destination_locator": record.destination_locator,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "catalog_commit_started": true,
        "catalog_commit_completed": false,
        "catalog_may_have_partial_writes": failure.catalog_may_have_partial_writes,
        "storage_cleanup_attempted": true,
        "storage_cleanup_complete": cleanup_complete,
        "cleanup_status": cleanup_report.status
    }))
    .map_err(database_error)
}

pub(super) fn storage_apply_failure_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    apply_report: &StorageApplyReport,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": false,
        "media_source_mutation": false,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme()
    }))
    .map_err(database_error)
}

pub(super) fn pre_mutation_failure_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    safe_error_code: &str,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": false,
        "media_source_mutation": false,
        "target_created": false,
        "operation_kind": record.operation_kind,
        "safe_error_code": safe_error_code
    }))
    .map_err(database_error)
}

pub(super) fn storage_cleanup_complete(report: &StorageCleanupReport) -> bool {
    report.cleaned
        || matches!(
            report.status,
            StorageCleanupStatus::Cleaned | StorageCleanupStatus::TargetMissing
        )
}

pub(super) fn storage_apply_error_code(status: StorageApplyStatus) -> &'static str {
    match status {
        StorageApplyStatus::Applied => "storage_apply_applied",
        StorageApplyStatus::Unsupported => "storage_apply_unsupported",
        StorageApplyStatus::SourceMissing => "storage_apply_source_missing",
        StorageApplyStatus::SourceNotFile => "storage_apply_source_not_file",
        StorageApplyStatus::TargetParentMissing => "storage_apply_target_parent_missing",
        StorageApplyStatus::TargetParentNotDirectory => "storage_apply_target_parent_not_directory",
        StorageApplyStatus::TargetExists => "storage_apply_target_exists",
        StorageApplyStatus::SecurityViolation => "storage_apply_security_violation",
        StorageApplyStatus::ApplyFailed => "storage_apply_failed",
    }
}

pub(super) fn storage_apply_safe_message(status: StorageApplyStatus) -> String {
    match status {
        StorageApplyStatus::Unsupported => {
            "storage backend does not support the accepted apply kind"
        }
        StorageApplyStatus::SourceMissing => "promotion source artifact is missing",
        StorageApplyStatus::SourceNotFile => "promotion source artifact is not a file",
        StorageApplyStatus::TargetParentMissing => "promotion target parent is missing",
        StorageApplyStatus::TargetParentNotDirectory => {
            "promotion target parent is not a directory"
        }
        StorageApplyStatus::TargetExists => "promotion target already exists",
        StorageApplyStatus::SecurityViolation => {
            "promotion storage apply violated storage safety rules"
        }
        StorageApplyStatus::ApplyFailed => "promotion storage apply failed before catalog mutation",
        StorageApplyStatus::Applied => "promotion storage apply succeeded",
    }
    .to_owned()
}

fn database_error<E: std::fmt::Display>(err: E) -> NakoError {
    NakoError::Database {
        message: err.to_string(),
    }
}
