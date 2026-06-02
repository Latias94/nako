use nako_client_protocol::PageInfo;
use nako_core::{
    MediaItemId, MediaKind, MetadataCandidateRecord, MetadataCandidateReviewApplicationAction,
    MetadataCandidateReviewApplicationPlan, MetadataCandidateReviewApplicationReason,
    MetadataCandidateReviewId, MetadataCandidateReviewNode as CoreMetadataCandidateReviewNode,
    MetadataCandidateReviewRecord,
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

impl AdminMetadataCandidateReviewResponse {
    #[must_use]
    pub fn new(
        review: MetadataCandidateReviewRecord,
        application_plan: MetadataCandidateReviewApplicationPlan,
    ) -> Self {
        let boundary =
            AdminMetadataCandidateReviewApplicationBoundary::from_plan(&application_plan);

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            review: AdminMetadataCandidateReviewDetail::from_record(review),
            application_plan: AdminMetadataCandidateReviewApplicationPlan::from_plan(
                application_plan,
            ),
            boundary,
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

fn fingerprint_text(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(value.as_bytes());
    format!("{:x}", digest)[..16].to_owned()
}
