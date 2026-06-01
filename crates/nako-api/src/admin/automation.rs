use nako_client_protocol::PageInfo;
use nako_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability,
    AutomationProviderId, GeneratedArtifactAcceptanceActionKind,
    GeneratedArtifactAcceptanceBoundary, GeneratedArtifactAcceptancePlan,
    GeneratedArtifactAcceptancePlanReason, GeneratedArtifactAcceptancePlanStatus,
    GeneratedArtifactMetadataApplyFieldPlan, GeneratedArtifactMetadataApplyOutcomeId,
    GeneratedArtifactMetadataApplyPlan, GeneratedArtifactMetadataApplyPlanReason,
    GeneratedArtifactMetadataApplyPlanStatus, GeneratedArtifactMetadataApplyResult,
    GeneratedArtifactMetadataApplyResultStatus,
    GeneratedArtifactMetadataBulkApplyBatchExecutionSummary,
    GeneratedArtifactMetadataBulkApplyBatchId, GeneratedArtifactMetadataBulkApplyBatchItemRecord,
    GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    GeneratedArtifactMetadataBulkApplyBatchRecord, GeneratedArtifactMetadataBulkApplyBatchStatus,
    GeneratedArtifactMetadataBulkApplyPlan, GeneratedArtifactMetadataBulkApplyPlanItem,
    GeneratedArtifactMetadataBulkApplyPlanItemReason,
    GeneratedArtifactMetadataBulkApplyPlanItemStatus,
    GeneratedArtifactMetadataBulkApplyPlanSelection, GeneratedArtifactMetadataBulkApplyPlanSummary,
    GeneratedArtifactMetadataFieldAction, GeneratedArtifactMetadataFieldReason,
    GeneratedArtifactMetadataValueSummary, GeneratedArtifactPayloadShape,
    GeneratedArtifactPayloadSummary, GeneratedArtifactProposal, GeneratedArtifactProvenance,
    GeneratedArtifactReadiness, GeneratedArtifactReadinessReason, GeneratedArtifactReadinessStatus,
    GeneratedArtifactReviewDecision, GeneratedArtifactReviewResult, GeneratedArtifactTarget,
    GeneratedArtifactTargetKind, JobId, LibraryId, MediaItemId, MediaSourceId, MetadataField,
};
use serde::{Deserialize, Serialize};

use super::ADMIN_API_VERSION;
use crate::public_client::API_VERSION;

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
            public_api_version: API_VERSION.to_owned(),
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
pub struct AdminGeneratedArtifactMetadataApplyPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub plan: AdminGeneratedArtifactMetadataApplyPlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyPlanRequest {
    pub artifact_ids: Vec<AutomationArtifactId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyRequest {
    pub artifact_ids: Vec<AutomationArtifactId>,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub plan: AdminGeneratedArtifactMetadataBulkApplyPlan,
}

impl AdminGeneratedArtifactMetadataBulkApplyPlanResponse {
    #[must_use]
    pub fn from_plan(plan: GeneratedArtifactMetadataBulkApplyPlan) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            plan: AdminGeneratedArtifactMetadataBulkApplyPlan::from_plan(plan),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyBatchResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub batch: AdminGeneratedArtifactMetadataBulkApplyBatch,
}

impl AdminGeneratedArtifactMetadataBulkApplyBatchResponse {
    #[must_use]
    pub fn from_batch(batch: GeneratedArtifactMetadataBulkApplyBatchRecord) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            batch: AdminGeneratedArtifactMetadataBulkApplyBatch::from_batch(batch),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataApplyRequest {
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataApplyResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub outcome_id: Option<GeneratedArtifactMetadataApplyOutcomeId>,
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataApplyResultStatus,
    pub applied: bool,
    pub changed: bool,
    pub idempotent_replay: bool,
    pub applied_source: Option<String>,
    pub plan: AdminGeneratedArtifactMetadataApplyPlan,
}

impl AdminGeneratedArtifactMetadataApplyResponse {
    #[must_use]
    pub fn from_result(result: GeneratedArtifactMetadataApplyResult) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            outcome_id: result.outcome_id,
            artifact_id: result.artifact_id,
            status: result.status,
            applied: result.applied,
            changed: result.changed,
            idempotent_replay: result.idempotent_replay,
            applied_source: result.applied_source,
            plan: AdminGeneratedArtifactMetadataApplyPlan::from_plan(result.plan),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataApplyPlan {
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataApplyPlanStatus,
    pub executable: bool,
    pub reasons: Vec<GeneratedArtifactMetadataApplyPlanReason>,
    pub target: AdminGeneratedArtifactTarget,
    pub payload: AdminGeneratedArtifactPayloadSummary,
    pub fields: Vec<AdminGeneratedArtifactMetadataApplyFieldPlan>,
    pub apply_field_count: u32,
    pub skipped_field_count: u32,
    pub noop_field_count: u32,
}

impl AdminGeneratedArtifactMetadataApplyPlan {
    #[must_use]
    pub fn from_plan(plan: GeneratedArtifactMetadataApplyPlan) -> Self {
        Self {
            artifact_id: plan.artifact_id,
            status: plan.status,
            executable: plan.executable,
            reasons: plan.reasons,
            target: AdminGeneratedArtifactTarget::from_target(plan.target),
            payload: AdminGeneratedArtifactPayloadSummary::from_summary(plan.payload),
            fields: plan
                .fields
                .into_iter()
                .map(AdminGeneratedArtifactMetadataApplyFieldPlan::from_plan)
                .collect(),
            apply_field_count: plan.apply_field_count,
            skipped_field_count: plan.skipped_field_count,
            noop_field_count: plan.noop_field_count,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataApplyFieldPlan {
    pub field: MetadataField,
    pub action: GeneratedArtifactMetadataFieldAction,
    pub reasons: Vec<GeneratedArtifactMetadataFieldReason>,
    pub current: GeneratedArtifactMetadataValueSummary,
    pub incoming: GeneratedArtifactMetadataValueSummary,
}

impl AdminGeneratedArtifactMetadataApplyFieldPlan {
    #[must_use]
    pub fn from_plan(plan: GeneratedArtifactMetadataApplyFieldPlan) -> Self {
        Self {
            field: plan.field,
            action: plan.action,
            reasons: plan.reasons,
            current: plan.current,
            incoming: plan.incoming,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyPlan {
    pub selection: GeneratedArtifactMetadataBulkApplyPlanSelection,
    pub summary: GeneratedArtifactMetadataBulkApplyPlanSummary,
    pub items: Vec<AdminGeneratedArtifactMetadataBulkApplyPlanItem>,
}

impl AdminGeneratedArtifactMetadataBulkApplyPlan {
    #[must_use]
    pub fn from_plan(plan: GeneratedArtifactMetadataBulkApplyPlan) -> Self {
        Self {
            selection: plan.selection,
            summary: plan.summary,
            items: plan
                .items
                .into_iter()
                .map(AdminGeneratedArtifactMetadataBulkApplyPlanItem::from_item)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyPlanItem {
    pub artifact_id: AutomationArtifactId,
    pub status: GeneratedArtifactMetadataBulkApplyPlanItemStatus,
    pub executable: bool,
    pub reasons: Vec<GeneratedArtifactMetadataBulkApplyPlanItemReason>,
    pub plan: Option<AdminGeneratedArtifactMetadataApplyPlan>,
}

impl AdminGeneratedArtifactMetadataBulkApplyPlanItem {
    #[must_use]
    pub fn from_item(item: GeneratedArtifactMetadataBulkApplyPlanItem) -> Self {
        Self {
            artifact_id: item.artifact_id,
            status: item.status,
            executable: item.executable,
            reasons: item.reasons,
            plan: item
                .plan
                .map(AdminGeneratedArtifactMetadataApplyPlan::from_plan),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyBatch {
    pub id: GeneratedArtifactMetadataBulkApplyBatchId,
    pub job_id: JobId,
    pub status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    pub selection: GeneratedArtifactMetadataBulkApplyPlanSelection,
    pub summary: GeneratedArtifactMetadataBulkApplyPlanSummary,
    pub execution_summary: GeneratedArtifactMetadataBulkApplyBatchExecutionSummary,
    pub items: Vec<AdminGeneratedArtifactMetadataBulkApplyBatchItem>,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminGeneratedArtifactMetadataBulkApplyBatch {
    #[must_use]
    pub fn from_batch(batch: GeneratedArtifactMetadataBulkApplyBatchRecord) -> Self {
        Self {
            id: batch.id,
            job_id: batch.job_id,
            status: batch.status,
            selection: batch.selection,
            summary: batch.summary,
            execution_summary: batch.execution_summary,
            items: batch
                .items
                .into_iter()
                .map(AdminGeneratedArtifactMetadataBulkApplyBatchItem::from_item)
                .collect(),
            created_at: batch.created_at,
            updated_at: batch.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminGeneratedArtifactMetadataBulkApplyBatchItem {
    pub artifact_id: AutomationArtifactId,
    pub position: u32,
    pub status: GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    pub outcome_id: Option<GeneratedArtifactMetadataApplyOutcomeId>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub plan_item: AdminGeneratedArtifactMetadataBulkApplyPlanItem,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminGeneratedArtifactMetadataBulkApplyBatchItem {
    #[must_use]
    pub fn from_item(item: GeneratedArtifactMetadataBulkApplyBatchItemRecord) -> Self {
        Self {
            artifact_id: item.artifact_id,
            position: item.position,
            status: item.status,
            outcome_id: item.outcome_id,
            error_code: item.error_code,
            error_message: item.error_message,
            plan_item: AdminGeneratedArtifactMetadataBulkApplyPlanItem::from_item(item.plan_item),
            created_at: item.created_at,
            updated_at: item.updated_at,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_artifact_proposals_expose_summaries_not_raw_prompt_or_payload() {
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
    fn generated_artifact_review_response_exposes_boundary_not_raw_payload() {
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
    fn generated_artifact_metadata_apply_plan_exposes_field_summaries_not_raw_payload() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let artifact_id = AutomationArtifactId::new();
        let plan = GeneratedArtifactMetadataApplyPlan {
            artifact_id,
            status: GeneratedArtifactMetadataApplyPlanStatus::Ready,
            executable: true,
            reasons: vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
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
            fields: vec![GeneratedArtifactMetadataApplyFieldPlan {
                field: MetadataField::Overview,
                action: GeneratedArtifactMetadataFieldAction::Apply,
                reasons: vec![GeneratedArtifactMetadataFieldReason::Ready],
                current: GeneratedArtifactMetadataValueSummary::missing(),
                incoming: GeneratedArtifactMetadataValueSummary {
                    present: true,
                    empty: false,
                    value_fingerprint: Some("sha256:44444444444444444444444444444444".to_owned()),
                    value_bytes: Some(27),
                    item_count: None,
                },
            }],
            apply_field_count: 1,
            skipped_field_count: 0,
            noop_field_count: 0,
        };
        let response = AdminGeneratedArtifactMetadataApplyPlanResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            plan: AdminGeneratedArtifactMetadataApplyPlan::from_plan(plan),
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["plan"]["status"], "ready");
        assert_eq!(value["plan"]["executable"], true);
        assert_eq!(value["plan"]["fields"][0]["field"], "overview");
        assert_eq!(value["plan"]["fields"][0]["action"], "apply");
        assert_eq!(
            value["plan"]["fields"][0]["incoming"]["value_fingerprint"],
            "sha256:44444444444444444444444444444444"
        );
        assert!(!body.contains("private generated overview"));
        assert!(!body.contains("artifact_json"));
        assert!(!body.contains("prompt_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("secret"));
    }

    #[test]
    fn generated_artifact_metadata_apply_bulk_plan_exposes_aggregate_summaries_not_raw_payload() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let artifact_id = AutomationArtifactId::new();
        let missing_artifact_id = AutomationArtifactId::new();
        let request = AdminGeneratedArtifactMetadataBulkApplyPlanRequest {
            artifact_ids: vec![artifact_id, artifact_id, missing_artifact_id],
        };
        let plan = GeneratedArtifactMetadataApplyPlan {
            artifact_id,
            status: GeneratedArtifactMetadataApplyPlanStatus::Ready,
            executable: true,
            reasons: vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
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
            fields: vec![GeneratedArtifactMetadataApplyFieldPlan {
                field: MetadataField::Overview,
                action: GeneratedArtifactMetadataFieldAction::Apply,
                reasons: vec![GeneratedArtifactMetadataFieldReason::Ready],
                current: GeneratedArtifactMetadataValueSummary::missing(),
                incoming: GeneratedArtifactMetadataValueSummary {
                    present: true,
                    empty: false,
                    value_fingerprint: Some("sha256:44444444444444444444444444444444".to_owned()),
                    value_bytes: Some(27),
                    item_count: None,
                },
            }],
            apply_field_count: 1,
            skipped_field_count: 0,
            noop_field_count: 0,
        };
        let response = AdminGeneratedArtifactMetadataBulkApplyPlanResponse::from_plan(
            GeneratedArtifactMetadataBulkApplyPlan {
                selection: GeneratedArtifactMetadataBulkApplyPlanSelection {
                    requested_artifact_count: 3,
                    selected_artifact_count: 2,
                    duplicate_artifact_count: 1,
                    max_artifact_count: 100,
                },
                summary: GeneratedArtifactMetadataBulkApplyPlanSummary {
                    planned_artifact_count: 1,
                    missing_artifact_count: 1,
                    ready_artifact_count: 1,
                    blocked_artifact_count: 0,
                    stale_artifact_count: 0,
                    executable_artifact_count: 1,
                    apply_field_count: 1,
                    skipped_field_count: 0,
                    noop_field_count: 0,
                },
                items: vec![
                    GeneratedArtifactMetadataBulkApplyPlanItem {
                        artifact_id,
                        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned,
                        executable: true,
                        reasons: vec![GeneratedArtifactMetadataBulkApplyPlanItemReason::Planned],
                        plan: Some(plan),
                    },
                    GeneratedArtifactMetadataBulkApplyPlanItem {
                        artifact_id: missing_artifact_id,
                        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Missing,
                        executable: false,
                        reasons: vec![
                            GeneratedArtifactMetadataBulkApplyPlanItemReason::MissingArtifact,
                        ],
                        plan: None,
                    },
                ],
            },
        );

        let request_value = serde_json::to_value(&request).unwrap();
        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(request_value["artifact_ids"].as_array().unwrap().len(), 3);
        assert_eq!(value["plan"]["selection"]["requested_artifact_count"], 3);
        assert_eq!(value["plan"]["selection"]["selected_artifact_count"], 2);
        assert_eq!(value["plan"]["selection"]["duplicate_artifact_count"], 1);
        assert_eq!(value["plan"]["summary"]["planned_artifact_count"], 1);
        assert_eq!(value["plan"]["summary"]["missing_artifact_count"], 1);
        assert_eq!(value["plan"]["summary"]["apply_field_count"], 1);
        assert_eq!(value["plan"]["items"][0]["status"], "planned");
        assert_eq!(
            value["plan"]["items"][0]["plan"]["fields"][0]["field"],
            "overview"
        );
        assert_eq!(value["plan"]["items"][1]["status"], "missing");
        assert!(value["plan"]["items"][1]["plan"].is_null());
        assert!(!body.contains("private generated overview"));
        assert!(!body.contains("artifact_json"));
        assert!(!body.contains("prompt_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("secret"));
    }

    #[test]
    fn generated_artifact_metadata_apply_bulk_batch_response_is_redacted_and_result_safe() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let artifact_id = AutomationArtifactId::new();
        let missing_artifact_id = AutomationArtifactId::new();
        let outcome_id = GeneratedArtifactMetadataApplyOutcomeId::new();
        let request = AdminGeneratedArtifactMetadataBulkApplyRequest {
            artifact_ids: vec![artifact_id, missing_artifact_id],
            idempotency_key: "bulk:operator-confirmation".to_owned(),
        };
        let plan = GeneratedArtifactMetadataApplyPlan {
            artifact_id,
            status: GeneratedArtifactMetadataApplyPlanStatus::Ready,
            executable: true,
            reasons: vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
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
            fields: vec![GeneratedArtifactMetadataApplyFieldPlan {
                field: MetadataField::Overview,
                action: GeneratedArtifactMetadataFieldAction::Apply,
                reasons: vec![GeneratedArtifactMetadataFieldReason::Ready],
                current: GeneratedArtifactMetadataValueSummary::missing(),
                incoming: GeneratedArtifactMetadataValueSummary {
                    present: true,
                    empty: false,
                    value_fingerprint: Some("sha256:44444444444444444444444444444444".to_owned()),
                    value_bytes: Some(27),
                    item_count: None,
                },
            }],
            apply_field_count: 1,
            skipped_field_count: 0,
            noop_field_count: 0,
        };
        let batch = GeneratedArtifactMetadataBulkApplyBatchRecord {
            id: GeneratedArtifactMetadataBulkApplyBatchId::new(),
            job_id: JobId::new(),
            idempotency_key: "bulk:operator-confirmation".to_owned(),
            status: GeneratedArtifactMetadataBulkApplyBatchStatus::Completed,
            selection: GeneratedArtifactMetadataBulkApplyPlanSelection {
                requested_artifact_count: 2,
                selected_artifact_count: 2,
                duplicate_artifact_count: 0,
                max_artifact_count: 100,
            },
            summary: GeneratedArtifactMetadataBulkApplyPlanSummary {
                planned_artifact_count: 1,
                missing_artifact_count: 1,
                ready_artifact_count: 1,
                blocked_artifact_count: 0,
                stale_artifact_count: 0,
                executable_artifact_count: 1,
                apply_field_count: 1,
                skipped_field_count: 0,
                noop_field_count: 0,
            },
            execution_summary: GeneratedArtifactMetadataBulkApplyBatchExecutionSummary {
                total_item_count: 2,
                pending_item_count: 0,
                skipped_item_count: 1,
                applied_item_count: 1,
                noop_item_count: 0,
                stale_item_count: 0,
                failed_item_count: 0,
            },
            items: vec![
                GeneratedArtifactMetadataBulkApplyBatchItemRecord {
                    batch_id: GeneratedArtifactMetadataBulkApplyBatchId::new(),
                    artifact_id,
                    position: 0,
                    status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied,
                    idempotency_key: "bulk:item:secret-key".to_owned(),
                    outcome_id: Some(outcome_id),
                    error_code: None,
                    error_message: None,
                    plan_item: GeneratedArtifactMetadataBulkApplyPlanItem {
                        artifact_id,
                        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned,
                        executable: true,
                        reasons: vec![GeneratedArtifactMetadataBulkApplyPlanItemReason::Planned],
                        plan: Some(plan),
                    },
                    created_at: "2026-06-01T00:00:00Z".to_owned(),
                    updated_at: "2026-06-01T00:00:01Z".to_owned(),
                },
                GeneratedArtifactMetadataBulkApplyBatchItemRecord {
                    batch_id: GeneratedArtifactMetadataBulkApplyBatchId::new(),
                    artifact_id: missing_artifact_id,
                    position: 1,
                    status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped,
                    idempotency_key: "bulk:item:missing-secret-key".to_owned(),
                    outcome_id: None,
                    error_code: Some("missing_artifact".to_owned()),
                    error_message: Some("artifact was missing during planning".to_owned()),
                    plan_item: GeneratedArtifactMetadataBulkApplyPlanItem {
                        artifact_id: missing_artifact_id,
                        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Missing,
                        executable: false,
                        reasons: vec![
                            GeneratedArtifactMetadataBulkApplyPlanItemReason::MissingArtifact,
                        ],
                        plan: None,
                    },
                    created_at: "2026-06-01T00:00:00Z".to_owned(),
                    updated_at: "2026-06-01T00:00:01Z".to_owned(),
                },
            ],
            created_at: "2026-06-01T00:00:00Z".to_owned(),
            updated_at: "2026-06-01T00:00:01Z".to_owned(),
        };
        let response = AdminGeneratedArtifactMetadataBulkApplyBatchResponse::from_batch(batch);

        let request_value = serde_json::to_value(&request).unwrap();
        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(request_value["artifact_ids"].as_array().unwrap().len(), 2);
        assert_eq!(
            request_value["idempotency_key"],
            "bulk:operator-confirmation"
        );
        assert_eq!(value["batch"]["status"], "completed");
        assert_eq!(value["batch"]["execution_summary"]["applied_item_count"], 1);
        assert_eq!(value["batch"]["execution_summary"]["skipped_item_count"], 1);
        assert_eq!(value["batch"]["items"][0]["status"], "applied");
        assert_eq!(
            value["batch"]["items"][0]["outcome_id"],
            outcome_id.to_string()
        );
        assert_eq!(
            value["batch"]["items"][0]["plan_item"]["plan"]["fields"][0]["field"],
            "overview"
        );
        assert_eq!(value["batch"]["items"][1]["status"], "skipped");
        assert!(value["batch"]["items"][1]["plan_item"]["plan"].is_null());
        assert!(
            !value["batch"]
                .as_object()
                .unwrap()
                .contains_key("idempotency_key")
        );
        assert!(
            !value["batch"]["items"][0]
                .as_object()
                .unwrap()
                .contains_key("idempotency_key")
        );
        assert!(!body.contains("bulk:operator-confirmation"));
        assert!(!body.contains("bulk:item:secret-key"));
        assert!(!body.contains("private generated overview"));
        assert!(!body.contains("artifact_json"));
        assert!(!body.contains("prompt_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("secret"));
    }

    #[test]
    fn generated_artifact_metadata_apply_response_is_redacted_and_replay_safe() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let artifact_id = AutomationArtifactId::new();
        let outcome_id = GeneratedArtifactMetadataApplyOutcomeId::new();
        let request = AdminGeneratedArtifactMetadataApplyRequest {
            idempotency_key: "metadata-apply:operator-confirmation".to_owned(),
        };
        let plan = GeneratedArtifactMetadataApplyPlan {
            artifact_id,
            status: GeneratedArtifactMetadataApplyPlanStatus::Ready,
            executable: true,
            reasons: vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
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
            fields: vec![GeneratedArtifactMetadataApplyFieldPlan {
                field: MetadataField::Overview,
                action: GeneratedArtifactMetadataFieldAction::Apply,
                reasons: vec![GeneratedArtifactMetadataFieldReason::Ready],
                current: GeneratedArtifactMetadataValueSummary::missing(),
                incoming: GeneratedArtifactMetadataValueSummary {
                    present: true,
                    empty: false,
                    value_fingerprint: Some("sha256:44444444444444444444444444444444".to_owned()),
                    value_bytes: Some(27),
                    item_count: None,
                },
            }],
            apply_field_count: 1,
            skipped_field_count: 0,
            noop_field_count: 0,
        };
        let response = AdminGeneratedArtifactMetadataApplyResponse::from_result(
            GeneratedArtifactMetadataApplyResult {
                outcome_id: Some(outcome_id),
                artifact_id,
                status: GeneratedArtifactMetadataApplyResultStatus::Applied,
                applied: true,
                changed: true,
                idempotent_replay: true,
                applied_source: Some("user".to_owned()),
                plan,
            },
        );

        let request_value = serde_json::to_value(&request).unwrap();
        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(
            request_value["idempotency_key"],
            "metadata-apply:operator-confirmation"
        );
        assert_eq!(value["outcome_id"], outcome_id.to_string());
        assert_eq!(value["artifact_id"], artifact_id.to_string());
        assert_eq!(value["status"], "applied");
        assert_eq!(value["applied"], true);
        assert_eq!(value["changed"], true);
        assert_eq!(value["idempotent_replay"], true);
        assert_eq!(value["applied_source"], "user");
        assert_eq!(value["plan"]["fields"][0]["field"], "overview");
        assert_eq!(
            value["plan"]["fields"][0]["incoming"]["value_fingerprint"],
            "sha256:44444444444444444444444444444444"
        );
        assert!(!body.contains("private generated overview"));
        assert!(!body.contains("artifact_json"));
        assert!(!body.contains("prompt_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("secret"));
    }
}
