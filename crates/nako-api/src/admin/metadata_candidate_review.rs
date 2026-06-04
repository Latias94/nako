use nako_client_protocol::PageInfo;
use nako_core::{
    JobId, LibraryId, MediaItemId, MediaKind, MetadataCandidateRecord,
    MetadataCandidateReviewApplicationAction, MetadataCandidateReviewApplicationPlan,
    MetadataCandidateReviewApplicationReason, MetadataCandidateReviewBatchExecutionSummary,
    MetadataCandidateReviewBatchId, MetadataCandidateReviewBatchItemRecord,
    MetadataCandidateReviewBatchItemStatus, MetadataCandidateReviewBatchPlanSelection,
    MetadataCandidateReviewBatchPlanSummary, MetadataCandidateReviewBatchRecord,
    MetadataCandidateReviewBatchStatus, MetadataCandidateReviewId,
    MetadataCandidateReviewNode as CoreMetadataCandidateReviewNode, MetadataCandidateReviewRecord,
    MetadataCandidateReviewRelatedHierarchyApplicationAction,
    MetadataCandidateReviewRelatedHierarchyApplicationPlan,
    MetadataCandidateReviewRelatedHierarchyApplicationReason,
    MetadataCandidateReviewRelatedHierarchyApplicationTargetPlan,
    MetadataCandidateReviewRelationship as CoreMetadataCandidateReviewRelationship,
    MetadataCandidateReviewStatus, MetadataCandidateSource, MetadataCandidateSubject,
    MetadataSource, PageRequest, ProviderMapping, ProviderMappingId, ProviderMappingStatus,
    ProviderSubject, ProviderSubjectId,
};
use serde::{Deserialize, Serialize};

use super::ADMIN_API_VERSION;
use crate::public_client::API_VERSION;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub review: AdminMetadataCandidateReviewDetail,
    pub application_plan: AdminMetadataCandidateReviewApplicationPlan,
    pub boundary: AdminMetadataCandidateReviewApplicationBoundary,
    pub governance: AdminMetadataCandidateReviewGovernance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub item_id: MediaItemId,
    pub reviews: Vec<AdminMetadataCandidateReviewListEntry>,
    pub page: PageInfo,
}

impl AdminMetadataCandidateReviewListResponse {
    #[must_use]
    pub fn new(
        item_id: MediaItemId,
        reviews: Vec<(
            MetadataCandidateReviewRecord,
            MetadataCandidateReviewApplicationPlan,
        )>,
        page: PageRequest,
    ) -> Self {
        let reviews: Vec<_> = reviews
            .into_iter()
            .map(|(review, application_plan)| {
                AdminMetadataCandidateReviewListEntry::from_record(review, application_plan)
            })
            .collect();

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            item_id,
            page: PageInfo::new(page.limit, page.offset, saturating_u32_len(reviews.len())),
            reviews,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewQueueResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub reviews: Vec<AdminMetadataCandidateReviewListEntry>,
    pub page: PageInfo,
}

impl AdminMetadataCandidateReviewQueueResponse {
    #[must_use]
    pub fn new(
        reviews: Vec<(
            MetadataCandidateReviewRecord,
            MetadataCandidateReviewApplicationPlan,
        )>,
        page: PageRequest,
    ) -> Self {
        let reviews: Vec<_> = reviews
            .into_iter()
            .map(|(review, application_plan)| {
                AdminMetadataCandidateReviewListEntry::from_record(review, application_plan)
            })
            .collect();

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            page: PageInfo::new(page.limit, page.offset, saturating_u32_len(reviews.len())),
            reviews,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchPlanRequest {
    pub review_ids: Vec<MetadataCandidateReviewId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub summary: AdminMetadataCandidateReviewBatchPlanSummary,
    pub reviews: Vec<AdminMetadataCandidateReviewListEntry>,
}

impl AdminMetadataCandidateReviewBatchPlanResponse {
    #[must_use]
    pub fn new(
        reviews: Vec<(
            MetadataCandidateReviewRecord,
            MetadataCandidateReviewApplicationPlan,
        )>,
        requested_count: usize,
        max_review_count: usize,
    ) -> Self {
        let mut summary = AdminMetadataCandidateReviewBatchPlanSummary {
            requested_count: saturating_u32_len(requested_count),
            returned_count: saturating_u32_len(reviews.len()),
            max_review_count: saturating_u32_len(max_review_count),
            apply_count: 0,
            noop_count: 0,
            skip_count: 0,
        };
        let reviews: Vec<_> = reviews
            .into_iter()
            .map(|(review, application_plan)| {
                match application_plan.action {
                    MetadataCandidateReviewApplicationAction::Apply => summary.apply_count += 1,
                    MetadataCandidateReviewApplicationAction::Noop => summary.noop_count += 1,
                    MetadataCandidateReviewApplicationAction::Skip => summary.skip_count += 1,
                }
                AdminMetadataCandidateReviewListEntry::from_record(review, application_plan)
            })
            .collect();

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            summary,
            reviews,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchPlanSummary {
    pub requested_count: u32,
    pub returned_count: u32,
    pub max_review_count: u32,
    pub apply_count: u32,
    pub noop_count: u32,
    pub skip_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchApplyItemRequest {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchApplyRequest {
    pub idempotency_key: String,
    pub reviews: Vec<AdminMetadataCandidateReviewBatchApplyItemRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchCreateRequest {
    pub idempotency_key: String,
    pub reviews: Vec<AdminMetadataCandidateReviewBatchApplyItemRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub batch: AdminMetadataCandidateReviewBatch,
}

impl AdminMetadataCandidateReviewBatchResponse {
    #[must_use]
    pub fn from_batch(batch: MetadataCandidateReviewBatchRecord) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            batch: AdminMetadataCandidateReviewBatch::from_batch(batch),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatch {
    pub id: MetadataCandidateReviewBatchId,
    pub job_id: JobId,
    pub status: MetadataCandidateReviewBatchStatus,
    pub idempotency_key_fingerprint: String,
    pub selection: MetadataCandidateReviewBatchPlanSelection,
    pub summary: MetadataCandidateReviewBatchPlanSummary,
    pub execution_summary: MetadataCandidateReviewBatchExecutionSummary,
    pub items: Vec<AdminMetadataCandidateReviewBatchItem>,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminMetadataCandidateReviewBatch {
    #[must_use]
    pub fn from_batch(batch: MetadataCandidateReviewBatchRecord) -> Self {
        Self {
            id: batch.id,
            job_id: batch.job_id,
            status: batch.status,
            idempotency_key_fingerprint: fingerprint_text(&batch.idempotency_key),
            selection: batch.selection,
            summary: batch.summary,
            execution_summary: batch.execution_summary,
            items: batch
                .items
                .into_iter()
                .map(AdminMetadataCandidateReviewBatchItem::from_item)
                .collect(),
            created_at: batch.created_at,
            updated_at: batch.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchItem {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub position: u32,
    pub status: MetadataCandidateReviewBatchItemStatus,
    pub idempotency_key_fingerprint: String,
    pub expected_updated_at_ms: Option<i64>,
    pub provider_subject_id: Option<ProviderSubjectId>,
    pub provider_mapping_id: Option<ProviderMappingId>,
    pub error: Option<AdminMetadataCandidateReviewBatchApplyError>,
    pub plan: AdminMetadataCandidateReviewApplicationPlan,
    pub boundary: AdminMetadataCandidateReviewApplicationBoundary,
    pub governance: AdminMetadataCandidateReviewGovernance,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminMetadataCandidateReviewBatchItem {
    #[must_use]
    pub fn from_item(item: MetadataCandidateReviewBatchItemRecord) -> Self {
        let boundary = AdminMetadataCandidateReviewApplicationBoundary::from_plan(&item.plan);
        let governance = AdminMetadataCandidateReviewGovernance::from_batch_item(&item);

        Self {
            review_id: item.review_id,
            item_id: item.item_id,
            position: item.position,
            status: item.status,
            idempotency_key_fingerprint: fingerprint_text(&item.idempotency_key),
            expected_updated_at_ms: item.expected_updated_at_ms,
            provider_subject_id: item.provider_subject_id,
            provider_mapping_id: item.provider_mapping_id,
            error: batch_item_error(item.error_code, item.error_message),
            plan: AdminMetadataCandidateReviewApplicationPlan::from_plan(item.plan),
            boundary,
            governance,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchApplyResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub idempotency_key_fingerprint: String,
    pub summary: AdminMetadataCandidateReviewBatchApplySummary,
    pub results: Vec<AdminMetadataCandidateReviewBatchApplyResult>,
}

impl AdminMetadataCandidateReviewBatchApplyResponse {
    #[must_use]
    pub fn new(
        idempotency_key: &str,
        results: Vec<AdminMetadataCandidateReviewBatchApplyResult>,
        requested_count: usize,
        max_review_count: usize,
    ) -> Self {
        let mut summary = AdminMetadataCandidateReviewBatchApplySummary {
            requested_count: saturating_u32_len(requested_count),
            returned_count: saturating_u32_len(results.len()),
            max_review_count: saturating_u32_len(max_review_count),
            applied_count: 0,
            changed_count: 0,
            noop_count: 0,
            replay_count: 0,
            skipped_count: 0,
            blocked_count: 0,
            stale_count: 0,
            conflict_count: 0,
            failed_count: 0,
        };

        for result in &results {
            if result.changed {
                summary.changed_count += 1;
            }
            if result.idempotent_replay {
                summary.replay_count += 1;
            }
            match result.status {
                AdminMetadataCandidateReviewBatchApplyResultStatus::Applied => {
                    summary.applied_count += 1;
                }
                AdminMetadataCandidateReviewBatchApplyResultStatus::Noop => {
                    summary.noop_count += 1;
                }
                AdminMetadataCandidateReviewBatchApplyResultStatus::Replayed => {}
                AdminMetadataCandidateReviewBatchApplyResultStatus::Skipped => {
                    summary.skipped_count += 1;
                }
                AdminMetadataCandidateReviewBatchApplyResultStatus::Blocked => {
                    summary.blocked_count += 1;
                }
                AdminMetadataCandidateReviewBatchApplyResultStatus::Stale => {
                    summary.stale_count += 1;
                }
                AdminMetadataCandidateReviewBatchApplyResultStatus::Conflict => {
                    summary.conflict_count += 1;
                }
                AdminMetadataCandidateReviewBatchApplyResultStatus::Failed => {
                    summary.failed_count += 1;
                }
            }
        }

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            idempotency_key_fingerprint: fingerprint_text(idempotency_key),
            summary,
            results,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchApplySummary {
    pub requested_count: u32,
    pub returned_count: u32,
    pub max_review_count: u32,
    pub applied_count: u32,
    pub changed_count: u32,
    pub noop_count: u32,
    pub replay_count: u32,
    pub skipped_count: u32,
    pub blocked_count: u32,
    pub stale_count: u32,
    pub conflict_count: u32,
    pub failed_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchApplyResult {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub status: AdminMetadataCandidateReviewBatchApplyResultStatus,
    pub applied: bool,
    pub changed: bool,
    pub idempotent_replay: bool,
    pub idempotency_key_fingerprint: String,
    pub plan: Option<AdminMetadataCandidateReviewApplicationPlan>,
    pub provider_subject: Option<AdminMetadataCandidateReviewProviderSubject>,
    pub provider_mapping: Option<AdminMetadataCandidateReviewProviderMapping>,
    pub boundary: Option<AdminMetadataCandidateReviewApplicationBoundary>,
    pub governance: Option<AdminMetadataCandidateReviewGovernance>,
    pub error: Option<AdminMetadataCandidateReviewBatchApplyError>,
}

impl AdminMetadataCandidateReviewBatchApplyResult {
    #[must_use]
    pub fn from_application(
        review: MetadataCandidateReviewRecord,
        plan: MetadataCandidateReviewApplicationPlan,
        provider_subject: Option<ProviderSubject>,
        provider_mapping: Option<ProviderMapping>,
        changed: bool,
        idempotency_key: &str,
    ) -> Self {
        let response = AdminMetadataCandidateReviewApplyResponse::from_application(
            review,
            plan,
            provider_subject,
            provider_mapping,
            changed,
            idempotency_key,
        );
        let status = if response.changed {
            AdminMetadataCandidateReviewBatchApplyResultStatus::Applied
        } else if matches!(
            response.plan.action,
            AdminMetadataCandidateReviewApplicationAction::Noop
        ) {
            AdminMetadataCandidateReviewBatchApplyResultStatus::Noop
        } else if response.idempotent_replay {
            AdminMetadataCandidateReviewBatchApplyResultStatus::Replayed
        } else {
            AdminMetadataCandidateReviewBatchApplyResultStatus::Applied
        };

        Self {
            review_id: response.review_id,
            item_id: response.item_id,
            status,
            applied: response.applied,
            changed: response.changed,
            idempotent_replay: response.idempotent_replay,
            idempotency_key_fingerprint: response.idempotency_key_fingerprint,
            plan: Some(response.plan),
            provider_subject: response.provider_subject,
            provider_mapping: response.provider_mapping,
            boundary: Some(response.boundary),
            governance: Some(response.governance),
            error: None,
        }
    }

    #[must_use]
    pub fn from_skipped_plan(
        review: MetadataCandidateReviewRecord,
        plan: MetadataCandidateReviewApplicationPlan,
        idempotency_key: &str,
    ) -> Self {
        let status = if plan
            .reasons
            .contains(&MetadataCandidateReviewApplicationReason::ReviewNotAccepted)
        {
            AdminMetadataCandidateReviewBatchApplyResultStatus::Skipped
        } else {
            AdminMetadataCandidateReviewBatchApplyResultStatus::Blocked
        };
        let boundary = AdminMetadataCandidateReviewApplicationBoundary::from_plan(&plan);
        let governance = AdminMetadataCandidateReviewGovernance::from_review_plan(&review, &plan);

        Self {
            review_id: review.id,
            item_id: review.item_id,
            status,
            applied: false,
            changed: false,
            idempotent_replay: false,
            idempotency_key_fingerprint: fingerprint_text(idempotency_key),
            plan: Some(AdminMetadataCandidateReviewApplicationPlan::from_plan(plan)),
            provider_subject: None,
            provider_mapping: None,
            boundary: Some(boundary),
            governance: Some(governance),
            error: None,
        }
    }

    #[must_use]
    pub fn from_error(
        review_id: MetadataCandidateReviewId,
        item_id: MediaItemId,
        status: AdminMetadataCandidateReviewBatchApplyResultStatus,
        error: AdminMetadataCandidateReviewBatchApplyError,
        idempotency_key: &str,
    ) -> Self {
        Self {
            review_id,
            item_id,
            status,
            applied: false,
            changed: false,
            idempotent_replay: false,
            idempotency_key_fingerprint: fingerprint_text(idempotency_key),
            plan: None,
            provider_subject: None,
            provider_mapping: None,
            boundary: None,
            governance: None,
            error: Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewBatchApplyResultStatus {
    Applied,
    Noop,
    Replayed,
    Skipped,
    Blocked,
    Stale,
    Conflict,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewBatchApplyError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewListEntry {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub source: MetadataCandidateSource,
    pub source_key: String,
    pub status: MetadataCandidateReviewStatus,
    pub root: AdminMetadataCandidateReviewNode,
    pub related_count: u32,
    pub relationship_count: u32,
    pub application_plan: AdminMetadataCandidateReviewApplicationPlan,
    pub boundary: AdminMetadataCandidateReviewApplicationBoundary,
    pub governance: AdminMetadataCandidateReviewGovernance,
    pub expires_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl AdminMetadataCandidateReviewListEntry {
    #[must_use]
    pub fn from_record(
        review: MetadataCandidateReviewRecord,
        application_plan: MetadataCandidateReviewApplicationPlan,
    ) -> Self {
        let related_count = saturating_u32_len(review.plan.related.len());
        let relationship_count = saturating_u32_len(review.plan.relationships.len());
        let boundary =
            AdminMetadataCandidateReviewApplicationBoundary::from_plan(&application_plan);
        let governance =
            AdminMetadataCandidateReviewGovernance::from_review_plan(&review, &application_plan);

        Self {
            review_id: review.id,
            item_id: review.item_id,
            source: review.source,
            source_key: review.source_key,
            status: review.status,
            root: AdminMetadataCandidateReviewNode::from_node(review.plan.root),
            related_count,
            relationship_count,
            application_plan: AdminMetadataCandidateReviewApplicationPlan::from_plan(
                application_plan,
            ),
            boundary,
            governance,
            expires_at_ms: review.expires_at_ms,
            created_at_ms: review.created_at_ms,
            updated_at_ms: review.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewApplyRequest {
    pub item_id: MediaItemId,
    pub expected_updated_at_ms: Option<i64>,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewApplyResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub applied: bool,
    pub changed: bool,
    pub idempotent_replay: bool,
    pub idempotency_key_fingerprint: String,
    pub plan: AdminMetadataCandidateReviewApplicationPlan,
    pub provider_subject: Option<AdminMetadataCandidateReviewProviderSubject>,
    pub provider_mapping: Option<AdminMetadataCandidateReviewProviderMapping>,
    pub boundary: AdminMetadataCandidateReviewApplicationBoundary,
    pub governance: AdminMetadataCandidateReviewGovernance,
}

impl AdminMetadataCandidateReviewApplyResponse {
    #[must_use]
    pub fn from_application(
        review: MetadataCandidateReviewRecord,
        plan: MetadataCandidateReviewApplicationPlan,
        provider_subject: Option<ProviderSubject>,
        provider_mapping: Option<ProviderMapping>,
        changed: bool,
        idempotency_key: &str,
    ) -> Self {
        let applied = provider_mapping.is_some();
        let idempotent_replay = applied && !changed;
        let boundary = AdminMetadataCandidateReviewApplicationBoundary::from_plan(&plan);
        let governance = AdminMetadataCandidateReviewGovernance::from_application_result(
            &review,
            &plan,
            provider_mapping.as_ref(),
            changed,
            idempotent_replay,
        );

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            review_id: review.id,
            item_id: review.item_id,
            applied,
            changed,
            idempotent_replay,
            idempotency_key_fingerprint: fingerprint_text(idempotency_key),
            plan: AdminMetadataCandidateReviewApplicationPlan::from_plan(plan),
            provider_subject: provider_subject
                .map(AdminMetadataCandidateReviewProviderSubject::from_subject),
            provider_mapping: provider_mapping
                .map(AdminMetadataCandidateReviewProviderMapping::from_mapping),
            boundary,
            governance,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
    pub item_id: MediaItemId,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub plan: AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan,
    pub boundary: AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary,
}

impl AdminMetadataCandidateReviewRelatedHierarchyPlanResponse {
    #[must_use]
    pub fn from_plan(
        review: MetadataCandidateReviewRecord,
        plan: MetadataCandidateReviewRelatedHierarchyApplicationPlan,
    ) -> Self {
        let boundary =
            AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary::from_plan(&plan);

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            review_id: review.id,
            item_id: review.item_id,
            plan: AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan::from_plan(plan),
            boundary,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
    pub item_id: MediaItemId,
    pub expected_updated_at_ms: Option<i64>,
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyApplyResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub applied: bool,
    pub changed: bool,
    pub idempotent_replay: bool,
    pub idempotency_key_fingerprint: String,
    pub plan: AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan,
    pub provider_subjects: Vec<AdminMetadataCandidateReviewProviderSubject>,
    pub provider_mappings: Vec<AdminMetadataCandidateReviewProviderMapping>,
    pub confirmed_item_ids: Vec<MediaItemId>,
    pub boundary: AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary,
}

impl AdminMetadataCandidateReviewRelatedHierarchyApplyResponse {
    #[must_use]
    pub fn from_application(
        review: MetadataCandidateReviewRecord,
        plan: MetadataCandidateReviewRelatedHierarchyApplicationPlan,
        provider_subjects: Vec<ProviderSubject>,
        provider_mappings: Vec<ProviderMapping>,
        confirmed_item_ids: Vec<MediaItemId>,
        changed: bool,
        idempotency_key: &str,
    ) -> Self {
        let applied = !provider_mappings.is_empty() || !confirmed_item_ids.is_empty();
        let idempotent_replay = applied && !changed;
        let boundary =
            AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary::from_plan(&plan);

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            review_id: review.id,
            item_id: review.item_id,
            applied,
            changed,
            idempotent_replay,
            idempotency_key_fingerprint: fingerprint_text(idempotency_key),
            plan: AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan::from_plan(plan),
            provider_subjects: provider_subjects
                .into_iter()
                .map(AdminMetadataCandidateReviewProviderSubject::from_subject)
                .collect(),
            provider_mappings: provider_mappings
                .into_iter()
                .map(AdminMetadataCandidateReviewProviderMapping::from_mapping)
                .collect(),
            confirmed_item_ids,
            boundary,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub action: AdminMetadataCandidateReviewRelatedHierarchyApplicationAction,
    pub reasons: Vec<AdminMetadataCandidateReviewRelatedHierarchyApplicationReason>,
    pub source: Option<MetadataSource>,
    pub root_subject: Option<MetadataCandidateSubject>,
    pub root_mapping_id: Option<ProviderMappingId>,
    pub root_mapping_status: Option<ProviderMappingStatus>,
    pub target_count: u32,
    pub mapping_change_count: u32,
    pub provisional_state_change_count: u32,
    pub targets: Vec<AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget>,
}

impl AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan {
    #[must_use]
    pub fn from_plan(plan: MetadataCandidateReviewRelatedHierarchyApplicationPlan) -> Self {
        Self {
            review_id: plan.review_id,
            item_id: plan.item_id,
            action: AdminMetadataCandidateReviewRelatedHierarchyApplicationAction::from(
                plan.action,
            ),
            reasons: plan
                .reasons
                .into_iter()
                .map(AdminMetadataCandidateReviewRelatedHierarchyApplicationReason::from)
                .collect(),
            source: plan.source,
            root_subject: plan.root_subject,
            root_mapping_id: plan.root_mapping_id,
            root_mapping_status: plan.root_mapping_status,
            target_count: plan.target_count,
            mapping_change_count: plan.mapping_change_count,
            provisional_state_change_count: plan.provisional_state_change_count,
            targets: plan
                .targets
                .into_iter()
                .map(AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget::from_target)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget {
    pub item_id: MediaItemId,
    pub library_ids: Vec<LibraryId>,
    pub subject: MetadataCandidateSubject,
    pub source: MetadataSource,
    pub existing_subject_id: Option<ProviderSubjectId>,
    pub existing_mapping_id: Option<ProviderMappingId>,
    pub existing_mapping_status: Option<ProviderMappingStatus>,
    pub mapping_change_required: bool,
    pub provisional_library_state_count: u32,
}

impl AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget {
    #[must_use]
    pub fn from_target(
        target: MetadataCandidateReviewRelatedHierarchyApplicationTargetPlan,
    ) -> Self {
        Self {
            item_id: target.item_id,
            library_ids: target.library_ids,
            subject: target.subject,
            source: target.source,
            existing_subject_id: target.existing_subject_id,
            existing_mapping_id: target.existing_mapping_id,
            existing_mapping_status: target.existing_mapping_status,
            mapping_change_required: target.mapping_change_required,
            provisional_library_state_count: target.provisional_library_state_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewRelatedHierarchyApplicationAction {
    Apply,
    Skip,
    Noop,
}

impl From<MetadataCandidateReviewRelatedHierarchyApplicationAction>
    for AdminMetadataCandidateReviewRelatedHierarchyApplicationAction
{
    fn from(value: MetadataCandidateReviewRelatedHierarchyApplicationAction) -> Self {
        match value {
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Apply => Self::Apply,
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Skip => Self::Skip,
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Noop => Self::Noop,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewRelatedHierarchyApplicationReason {
    ReviewNotAccepted,
    MissingRootSubject,
    UnsupportedSource,
    MissingAcceptedRootMapping,
    NoSafeRelatedHierarchyRelationships,
    Ready,
    AlreadyApplied,
}

impl From<MetadataCandidateReviewRelatedHierarchyApplicationReason>
    for AdminMetadataCandidateReviewRelatedHierarchyApplicationReason
{
    fn from(value: MetadataCandidateReviewRelatedHierarchyApplicationReason) -> Self {
        match value {
            MetadataCandidateReviewRelatedHierarchyApplicationReason::ReviewNotAccepted => {
                Self::ReviewNotAccepted
            }
            MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingRootSubject => {
                Self::MissingRootSubject
            }
            MetadataCandidateReviewRelatedHierarchyApplicationReason::UnsupportedSource => {
                Self::UnsupportedSource
            }
            MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingAcceptedRootMapping => {
                Self::MissingAcceptedRootMapping
            }
            MetadataCandidateReviewRelatedHierarchyApplicationReason::NoSafeRelatedHierarchyRelationships => {
                Self::NoSafeRelatedHierarchyRelationships
            }
            MetadataCandidateReviewRelatedHierarchyApplicationReason::Ready => Self::Ready,
            MetadataCandidateReviewRelatedHierarchyApplicationReason::AlreadyApplied => {
                Self::AlreadyApplied
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary {
    pub read_only: bool,
    pub applies_on_read: bool,
    pub apply_mutation_required: bool,
    pub apply_updates_root_provider_subject: bool,
    pub apply_updates_root_provider_mapping: bool,
    pub apply_updates_related_provider_subjects: bool,
    pub apply_updates_related_provider_mappings: bool,
    pub apply_confirms_related_library_item_state: bool,
    pub updates_parent_hierarchy: bool,
    pub updates_canonical_metadata: bool,
    pub writes_nfo: bool,
    pub writes_library_files: bool,
}

impl AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary {
    #[must_use]
    pub const fn from_plan(plan: &MetadataCandidateReviewRelatedHierarchyApplicationPlan) -> Self {
        let would_apply = matches!(
            plan.action,
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Apply
        );
        let mapping_change_required = would_apply && plan.mapping_change_count > 0;

        Self {
            read_only: true,
            applies_on_read: false,
            apply_mutation_required: would_apply,
            apply_updates_root_provider_subject: false,
            apply_updates_root_provider_mapping: false,
            apply_updates_related_provider_subjects: mapping_change_required,
            apply_updates_related_provider_mappings: mapping_change_required,
            apply_confirms_related_library_item_state: would_apply
                && plan.provisional_state_change_count > 0,
            updates_parent_hierarchy: false,
            updates_canonical_metadata: false,
            writes_nfo: false,
            writes_library_files: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewProviderSubject {
    pub subject_id: ProviderSubjectId,
    pub provider: nako_core::ExternalProvider,
    pub subject_kind: nako_core::ProviderSubjectKind,
    pub subject_key: String,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub locale: Option<String>,
}

impl AdminMetadataCandidateReviewProviderSubject {
    #[must_use]
    pub fn from_subject(subject: ProviderSubject) -> Self {
        Self {
            subject_id: subject.id,
            provider: subject.provider,
            subject_kind: subject.subject_kind,
            subject_key: subject.subject_key,
            title: subject.title,
            release_year: subject.release_year,
            locale: subject.locale,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewProviderMapping {
    pub mapping_id: ProviderMappingId,
    pub item_id: MediaItemId,
    pub subject_id: ProviderSubjectId,
    pub status: ProviderMappingStatus,
    pub confidence_milli: Option<u16>,
    pub source: MetadataSource,
}

impl AdminMetadataCandidateReviewProviderMapping {
    #[must_use]
    pub fn from_mapping(mapping: ProviderMapping) -> Self {
        Self {
            mapping_id: mapping.id,
            item_id: mapping.item_id,
            subject_id: mapping.subject_id,
            status: mapping.status,
            confidence_milli: mapping.confidence_milli,
            source: mapping.source,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewGovernance {
    pub audit_timeline: AdminMetadataCandidateReviewAuditTimeline,
    pub undo_plan: AdminMetadataCandidateReviewUndoPlan,
}

impl AdminMetadataCandidateReviewGovernance {
    #[must_use]
    pub fn from_review_plan(
        review: &MetadataCandidateReviewRecord,
        plan: &MetadataCandidateReviewApplicationPlan,
    ) -> Self {
        Self {
            audit_timeline: AdminMetadataCandidateReviewAuditTimeline {
                read_only: true,
                replay_safe: true,
                events: review_plan_audit_events(review, plan),
            },
            undo_plan: AdminMetadataCandidateReviewUndoPlan::from_plan(
                plan,
                Some(review.updated_at_ms),
                plan.existing_mapping_id,
                plan.existing_mapping_status,
                false,
            ),
        }
    }

    #[must_use]
    pub fn from_application_result(
        review: &MetadataCandidateReviewRecord,
        plan: &MetadataCandidateReviewApplicationPlan,
        provider_mapping: Option<&ProviderMapping>,
        changed: bool,
        idempotent_replay: bool,
    ) -> Self {
        let target_mapping_id = provider_mapping
            .map(|mapping| mapping.id)
            .or(plan.existing_mapping_id);
        let target_mapping_status = provider_mapping
            .map(|mapping| mapping.status)
            .or(plan.existing_mapping_status);
        let mut events = review_plan_audit_events(review, plan);
        events.push(AdminMetadataCandidateReviewAuditEvent {
            kind: AdminMetadataCandidateReviewAuditEventKind::ApplicationResult,
            at_ms: None,
            status: Some(review.status),
            batch_item_status: None,
            action: Some(AdminMetadataCandidateReviewApplicationAction::from(
                plan.action,
            )),
            changed: Some(changed),
            idempotent_replay: Some(idempotent_replay),
            provider_mapping_id: target_mapping_id,
        });

        Self {
            audit_timeline: AdminMetadataCandidateReviewAuditTimeline {
                read_only: true,
                replay_safe: true,
                events,
            },
            undo_plan: AdminMetadataCandidateReviewUndoPlan::from_plan(
                plan,
                Some(review.updated_at_ms),
                target_mapping_id,
                target_mapping_status,
                true,
            ),
        }
    }

    #[must_use]
    pub fn from_batch_item(item: &MetadataCandidateReviewBatchItemRecord) -> Self {
        let target_mapping_id = item.provider_mapping_id.or(item.plan.existing_mapping_id);
        let target_mapping_status = if item.provider_mapping_id.is_some() {
            Some(ProviderMappingStatus::Accepted)
        } else {
            item.plan.existing_mapping_status
        };
        let application_result_observed = matches!(
            item.status,
            MetadataCandidateReviewBatchItemStatus::Applied
                | MetadataCandidateReviewBatchItemStatus::Noop
        ) && item.provider_mapping_id.is_some();

        Self {
            audit_timeline: AdminMetadataCandidateReviewAuditTimeline {
                read_only: true,
                replay_safe: true,
                events: vec![
                    AdminMetadataCandidateReviewAuditEvent {
                        kind: AdminMetadataCandidateReviewAuditEventKind::ApplicationPlanRead,
                        at_ms: None,
                        status: None,
                        batch_item_status: Some(item.status),
                        action: Some(AdminMetadataCandidateReviewApplicationAction::from(
                            item.plan.action,
                        )),
                        changed: None,
                        idempotent_replay: None,
                        provider_mapping_id: item.plan.existing_mapping_id,
                    },
                    AdminMetadataCandidateReviewAuditEvent {
                        kind: AdminMetadataCandidateReviewAuditEventKind::BatchItemStatus,
                        at_ms: None,
                        status: None,
                        batch_item_status: Some(item.status),
                        action: Some(AdminMetadataCandidateReviewApplicationAction::from(
                            item.plan.action,
                        )),
                        changed: None,
                        idempotent_replay: None,
                        provider_mapping_id: item.provider_mapping_id,
                    },
                ],
            },
            undo_plan: AdminMetadataCandidateReviewUndoPlan::from_plan(
                &item.plan,
                item.expected_updated_at_ms,
                target_mapping_id,
                target_mapping_status,
                application_result_observed,
            ),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewAuditTimeline {
    pub read_only: bool,
    pub replay_safe: bool,
    pub events: Vec<AdminMetadataCandidateReviewAuditEvent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewAuditEvent {
    pub kind: AdminMetadataCandidateReviewAuditEventKind,
    pub at_ms: Option<i64>,
    pub status: Option<MetadataCandidateReviewStatus>,
    pub batch_item_status: Option<MetadataCandidateReviewBatchItemStatus>,
    pub action: Option<AdminMetadataCandidateReviewApplicationAction>,
    pub changed: Option<bool>,
    pub idempotent_replay: Option<bool>,
    pub provider_mapping_id: Option<ProviderMappingId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewAuditEventKind {
    ReviewCreated,
    ReviewStatusCurrent,
    ApplicationPlanRead,
    ApplicationResult,
    BatchItemStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewUndoPlan {
    pub read_only: bool,
    pub undo_mutation_available: bool,
    pub replay_safe: bool,
    pub stale_state_guard_updated_at_ms: Option<i64>,
    pub target_mapping_id: Option<ProviderMappingId>,
    pub target_mapping_status: Option<ProviderMappingStatus>,
    pub mode: AdminMetadataCandidateReviewUndoMode,
    pub reasons: Vec<AdminMetadataCandidateReviewUndoReason>,
}

impl AdminMetadataCandidateReviewUndoPlan {
    fn from_plan(
        plan: &MetadataCandidateReviewApplicationPlan,
        stale_state_guard_updated_at_ms: Option<i64>,
        target_mapping_id: Option<ProviderMappingId>,
        target_mapping_status: Option<ProviderMappingStatus>,
        application_result_observed: bool,
    ) -> Self {
        let mode = if target_mapping_id.is_some()
            && (application_result_observed
                || matches!(plan.action, MetadataCandidateReviewApplicationAction::Noop))
        {
            AdminMetadataCandidateReviewUndoMode::ManualRootProviderMappingReview
        } else if matches!(plan.action, MetadataCandidateReviewApplicationAction::Apply) {
            AdminMetadataCandidateReviewUndoMode::DeferredUntilApplyOutcomeAudit
        } else {
            AdminMetadataCandidateReviewUndoMode::NoMutationObserved
        };
        let mut reasons = vec![
            AdminMetadataCandidateReviewUndoReason::ReadOnlyTrustSlice,
            AdminMetadataCandidateReviewUndoReason::RootOnlyProviderMappingBoundary,
            AdminMetadataCandidateReviewUndoReason::RelatedHierarchyUndoDeferred,
            AdminMetadataCandidateReviewUndoReason::PublicClientContractUnchanged,
        ];

        match mode {
            AdminMetadataCandidateReviewUndoMode::ManualRootProviderMappingReview => {
                reasons
                    .push(AdminMetadataCandidateReviewUndoReason::MissingPersistedPreApplySnapshot);
                reasons
                    .push(AdminMetadataCandidateReviewUndoReason::ProviderMappingMayPreexistReview);
                reasons.push(AdminMetadataCandidateReviewUndoReason::StaleStateGuardRequired);
            }
            AdminMetadataCandidateReviewUndoMode::DeferredUntilApplyOutcomeAudit => {
                reasons.push(AdminMetadataCandidateReviewUndoReason::ApplyOutcomeAuditRequired);
                reasons
                    .push(AdminMetadataCandidateReviewUndoReason::MissingPersistedPreApplySnapshot);
                reasons.push(AdminMetadataCandidateReviewUndoReason::StaleStateGuardRequired);
            }
            AdminMetadataCandidateReviewUndoMode::NoMutationObserved => {
                reasons.push(
                    AdminMetadataCandidateReviewUndoReason::NoProviderMappingMutationObserved,
                );
            }
        }

        Self {
            read_only: true,
            undo_mutation_available: false,
            replay_safe: true,
            stale_state_guard_updated_at_ms,
            target_mapping_id,
            target_mapping_status,
            mode,
            reasons,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewUndoMode {
    NoMutationObserved,
    DeferredUntilApplyOutcomeAudit,
    ManualRootProviderMappingReview,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewUndoReason {
    ReadOnlyTrustSlice,
    NoProviderMappingMutationObserved,
    ApplyOutcomeAuditRequired,
    MissingPersistedPreApplySnapshot,
    ProviderMappingMayPreexistReview,
    RootOnlyProviderMappingBoundary,
    RelatedHierarchyUndoDeferred,
    PublicClientContractUnchanged,
    StaleStateGuardRequired,
}

fn review_plan_audit_events(
    review: &MetadataCandidateReviewRecord,
    plan: &MetadataCandidateReviewApplicationPlan,
) -> Vec<AdminMetadataCandidateReviewAuditEvent> {
    vec![
        AdminMetadataCandidateReviewAuditEvent {
            kind: AdminMetadataCandidateReviewAuditEventKind::ReviewCreated,
            at_ms: Some(review.created_at_ms),
            status: None,
            batch_item_status: None,
            action: None,
            changed: None,
            idempotent_replay: None,
            provider_mapping_id: None,
        },
        AdminMetadataCandidateReviewAuditEvent {
            kind: AdminMetadataCandidateReviewAuditEventKind::ReviewStatusCurrent,
            at_ms: Some(review.updated_at_ms),
            status: Some(review.status),
            batch_item_status: None,
            action: None,
            changed: None,
            idempotent_replay: None,
            provider_mapping_id: None,
        },
        AdminMetadataCandidateReviewAuditEvent {
            kind: AdminMetadataCandidateReviewAuditEventKind::ApplicationPlanRead,
            at_ms: None,
            status: Some(review.status),
            batch_item_status: None,
            action: Some(AdminMetadataCandidateReviewApplicationAction::from(
                plan.action,
            )),
            changed: None,
            idempotent_replay: None,
            provider_mapping_id: plan.existing_mapping_id,
        },
    ]
}

impl AdminMetadataCandidateReviewResponse {
    #[must_use]
    pub fn new(
        review: MetadataCandidateReviewRecord,
        application_plan: MetadataCandidateReviewApplicationPlan,
    ) -> Self {
        let boundary =
            AdminMetadataCandidateReviewApplicationBoundary::from_plan(&application_plan);
        let governance =
            AdminMetadataCandidateReviewGovernance::from_review_plan(&review, &application_plan);

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            review: AdminMetadataCandidateReviewDetail::from_record(review),
            application_plan: AdminMetadataCandidateReviewApplicationPlan::from_plan(
                application_plan,
            ),
            boundary,
            governance,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewDetail {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub source: MetadataCandidateSource,
    pub source_key: String,
    pub status: MetadataCandidateReviewStatus,
    pub root: AdminMetadataCandidateReviewNode,
    pub related: Vec<AdminMetadataCandidateReviewNode>,
    pub relationships: Vec<AdminMetadataCandidateReviewRelationship>,
    pub related_count: u32,
    pub relationship_count: u32,
    pub expires_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl AdminMetadataCandidateReviewDetail {
    #[must_use]
    pub fn from_record(review: MetadataCandidateReviewRecord) -> Self {
        let related_count = saturating_u32_len(review.plan.related.len());
        let relationship_count = saturating_u32_len(review.plan.relationships.len());

        Self {
            review_id: review.id,
            item_id: review.item_id,
            source: review.source,
            source_key: review.source_key,
            status: review.status,
            root: AdminMetadataCandidateReviewNode::from_node(review.plan.root),
            related: review
                .plan
                .related
                .into_iter()
                .map(AdminMetadataCandidateReviewNode::from_node)
                .collect(),
            relationships: review
                .plan
                .relationships
                .into_iter()
                .map(AdminMetadataCandidateReviewRelationship::from_relationship)
                .collect(),
            related_count,
            relationship_count,
            expires_at_ms: review.expires_at_ms,
            created_at_ms: review.created_at_ms,
            updated_at_ms: review.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewNode {
    pub source: MetadataCandidateSource,
    pub kind: MediaKind,
    pub subject: Option<MetadataCandidateSubject>,
    pub metadata: AdminMetadataCandidateReviewMetadataSummary,
}

impl AdminMetadataCandidateReviewNode {
    #[must_use]
    pub fn from_node(node: CoreMetadataCandidateReviewNode) -> Self {
        Self {
            source: node.source,
            kind: node.kind,
            subject: node.subject,
            metadata: AdminMetadataCandidateReviewMetadataSummary::from_record(node.metadata),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewMetadataSummary {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub sort_title: Option<String>,
    pub release_date: Option<String>,
    pub runtime_minutes: Option<u32>,
    pub description_present: bool,
    pub tagline_present: bool,
    pub genre_count: u32,
    pub tag_count: u32,
    pub rating_count: u32,
    pub image_count: u32,
    pub credit_count: u32,
    pub collection_count: u32,
    pub studio_count: u32,
    pub external_id_count: u32,
}

impl AdminMetadataCandidateReviewMetadataSummary {
    #[must_use]
    pub fn from_record(record: MetadataCandidateRecord) -> Self {
        Self {
            title: record.title,
            original_title: record.original_title,
            sort_title: record.sort_title,
            release_date: record.release_date,
            runtime_minutes: record.runtime_minutes,
            description_present: record
                .overview
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            tagline_present: record
                .tagline
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            genre_count: saturating_u32_len(record.genres.len()),
            tag_count: saturating_u32_len(record.tags.len()),
            rating_count: saturating_u32_len(record.ratings.len()),
            image_count: saturating_u32_len(record.images.len()),
            credit_count: saturating_u32_len(record.credits.len()),
            collection_count: saturating_u32_len(record.collections.len()),
            studio_count: saturating_u32_len(record.studios.len()),
            external_id_count: saturating_u32_len(record.external_ids.len()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewRelationship {
    pub parent_subject: MetadataCandidateSubject,
    pub child_subject: MetadataCandidateSubject,
    pub kind: nako_core::MetadataCandidateRelationshipKind,
}

impl AdminMetadataCandidateReviewRelationship {
    #[must_use]
    pub fn from_relationship(relationship: CoreMetadataCandidateReviewRelationship) -> Self {
        Self {
            parent_subject: relationship.parent_subject,
            child_subject: relationship.child_subject,
            kind: relationship.kind,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewApplicationPlan {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub action: AdminMetadataCandidateReviewApplicationAction,
    pub reasons: Vec<AdminMetadataCandidateReviewApplicationReason>,
    pub source: Option<MetadataSource>,
    pub root_subject: Option<MetadataCandidateSubject>,
    pub existing_mapping_id: Option<ProviderMappingId>,
    pub existing_mapping_status: Option<ProviderMappingStatus>,
}

impl AdminMetadataCandidateReviewApplicationPlan {
    #[must_use]
    pub fn from_plan(plan: MetadataCandidateReviewApplicationPlan) -> Self {
        Self {
            review_id: plan.review_id,
            item_id: plan.item_id,
            action: AdminMetadataCandidateReviewApplicationAction::from(plan.action),
            reasons: plan
                .reasons
                .into_iter()
                .map(AdminMetadataCandidateReviewApplicationReason::from)
                .collect(),
            source: plan.source,
            root_subject: plan.root_subject,
            existing_mapping_id: plan.existing_mapping_id,
            existing_mapping_status: plan.existing_mapping_status,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewApplicationAction {
    Apply,
    Skip,
    Noop,
}

impl From<MetadataCandidateReviewApplicationAction>
    for AdminMetadataCandidateReviewApplicationAction
{
    fn from(value: MetadataCandidateReviewApplicationAction) -> Self {
        match value {
            MetadataCandidateReviewApplicationAction::Apply => Self::Apply,
            MetadataCandidateReviewApplicationAction::Skip => Self::Skip,
            MetadataCandidateReviewApplicationAction::Noop => Self::Noop,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminMetadataCandidateReviewApplicationReason {
    ReviewNotAccepted,
    MissingRootSubject,
    UnsupportedSource,
    ExistingAcceptedMapping,
    ExistingCandidateMapping,
    ExistingRejectedMapping,
    Ready,
}

impl From<MetadataCandidateReviewApplicationReason>
    for AdminMetadataCandidateReviewApplicationReason
{
    fn from(value: MetadataCandidateReviewApplicationReason) -> Self {
        match value {
            MetadataCandidateReviewApplicationReason::ReviewNotAccepted => Self::ReviewNotAccepted,
            MetadataCandidateReviewApplicationReason::MissingRootSubject => {
                Self::MissingRootSubject
            }
            MetadataCandidateReviewApplicationReason::UnsupportedSource => Self::UnsupportedSource,
            MetadataCandidateReviewApplicationReason::ExistingAcceptedMapping => {
                Self::ExistingAcceptedMapping
            }
            MetadataCandidateReviewApplicationReason::ExistingCandidateMapping => {
                Self::ExistingCandidateMapping
            }
            MetadataCandidateReviewApplicationReason::ExistingRejectedMapping => {
                Self::ExistingRejectedMapping
            }
            MetadataCandidateReviewApplicationReason::Ready => Self::Ready,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataCandidateReviewApplicationBoundary {
    pub read_only: bool,
    pub applies_on_read: bool,
    pub apply_mutation_required: bool,
    pub apply_updates_root_provider_subject: bool,
    pub apply_updates_root_provider_mapping: bool,
    pub apply_updates_related_provider_subjects: bool,
    pub apply_updates_related_provider_mappings: bool,
    pub updates_canonical_metadata: bool,
    pub updates_hierarchy: bool,
    pub writes_nfo: bool,
    pub writes_library_files: bool,
}

impl AdminMetadataCandidateReviewApplicationBoundary {
    #[must_use]
    pub const fn from_plan(plan: &MetadataCandidateReviewApplicationPlan) -> Self {
        let would_apply = matches!(plan.action, MetadataCandidateReviewApplicationAction::Apply);

        Self {
            read_only: true,
            applies_on_read: false,
            apply_mutation_required: would_apply,
            apply_updates_root_provider_subject: would_apply,
            apply_updates_root_provider_mapping: would_apply,
            apply_updates_related_provider_subjects: false,
            apply_updates_related_provider_mappings: false,
            updates_canonical_metadata: false,
            updates_hierarchy: false,
            writes_nfo: false,
            writes_library_files: false,
        }
    }
}

fn saturating_u32_len(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn batch_item_error(
    code: Option<String>,
    message: Option<String>,
) -> Option<AdminMetadataCandidateReviewBatchApplyError> {
    match (code, message) {
        (Some(code), Some(message)) => {
            Some(AdminMetadataCandidateReviewBatchApplyError { code, message })
        }
        (Some(code), None) => Some(AdminMetadataCandidateReviewBatchApplyError {
            code,
            message: String::new(),
        }),
        (None, Some(message)) => Some(AdminMetadataCandidateReviewBatchApplyError {
            code: "batch_item_error".to_owned(),
            message,
        }),
        (None, None) => None,
    }
}

fn fingerprint_text(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(value.as_bytes());
    format!("{:x}", digest)[..16].to_owned()
}
