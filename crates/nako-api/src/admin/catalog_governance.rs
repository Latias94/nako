use nako_client_protocol::PageInfo;
use nako_core::{
    CatalogGovernanceItemRecord, LibraryId, LocalInferenceEvidence, LocalInferenceEvidenceSource,
    MediaItem, MediaItemId, MediaKind, MediaSourceId, MetadataSource, ProviderMapping,
    ProviderMappingId, ProviderMappingStatus, ProviderSubject, ProviderSubjectId,
};
use serde::{Deserialize, Serialize};

use super::ADMIN_API_VERSION;
use crate::public_client::API_VERSION;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceItemListResponse {
    pub items: Vec<AdminCatalogGovernanceItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceItem {
    pub item_id: MediaItemId,
    pub library_id: LibraryId,
    pub kind: MediaKind,
    pub parent_id: Option<MediaItemId>,
    pub title: String,
    pub release_date: Option<String>,
    pub source_count: u32,
    pub representative_source_id: Option<MediaSourceId>,
    pub representative_file_name: Option<String>,
    pub local_inference: Option<AdminLocalInferenceSummary>,
    pub provider_mapping_count: u32,
    pub accepted_provider_mapping_count: u32,
    pub duplicate_relationship_count: u32,
    pub issues: Vec<AdminCatalogGovernanceIssue>,
}

impl AdminCatalogGovernanceItem {
    #[must_use]
    pub fn from_record(
        record: CatalogGovernanceItemRecord,
        low_confidence_threshold_milli: u16,
    ) -> Self {
        let local_inference = record
            .best_local_inference
            .map(AdminLocalInferenceSummary::from_evidence);
        let mut issues = Vec::new();

        if record.item.kind == MediaKind::Unknown {
            issues.push(AdminCatalogGovernanceIssue::UnknownKind);
        }
        if local_inference
            .as_ref()
            .and_then(|inference| inference.confidence_milli)
            .is_some_and(|confidence| confidence <= low_confidence_threshold_milli)
        {
            issues.push(AdminCatalogGovernanceIssue::LowLocalInferenceConfidence);
        }
        if record.accepted_provider_mapping_count == 0 {
            issues.push(AdminCatalogGovernanceIssue::MissingAcceptedProviderMapping);
        }

        Self {
            item_id: record.item.id,
            library_id: record.library_id,
            kind: record.item.kind,
            parent_id: record.item.parent_id,
            title: record.item.metadata.title,
            release_date: record.item.metadata.release_date,
            source_count: record.source_count,
            representative_source_id: record.representative_source_id,
            representative_file_name: record.representative_file_name,
            local_inference,
            provider_mapping_count: record.provider_mapping_count,
            accepted_provider_mapping_count: record.accepted_provider_mapping_count,
            duplicate_relationship_count: record.duplicate_relationship_count,
            issues,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLocalInferenceSummary {
    pub source_id: MediaSourceId,
    pub inferred_kind: MediaKind,
    pub inferred_title: Option<String>,
    pub inferred_year: Option<i32>,
    pub inferred_season: Option<u32>,
    pub inferred_episode: Option<u32>,
    pub confidence_milli: Option<u16>,
    pub evidence_source: LocalInferenceEvidenceSource,
    pub has_evidence: bool,
    pub inference_version: String,
}

impl AdminLocalInferenceSummary {
    #[must_use]
    pub fn from_evidence(evidence: LocalInferenceEvidence) -> Self {
        Self {
            source_id: evidence.source_id,
            inferred_kind: evidence.inferred_kind,
            inferred_title: evidence.inferred_title,
            inferred_year: evidence.inferred_year,
            inferred_season: evidence.inferred_season,
            inferred_episode: evidence.inferred_episode,
            confidence_milli: evidence.confidence_milli,
            evidence_source: evidence.evidence_source,
            has_evidence: !evidence.evidence_value.trim().is_empty(),
            inference_version: evidence.inference_version,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCatalogGovernanceIssue {
    UnknownKind,
    LowLocalInferenceConfidence,
    MissingAcceptedProviderMapping,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceItemDetailResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub item: AdminCatalogGovernanceItem,
    pub provider_mappings: Vec<AdminCatalogGovernanceProviderMappingSummary>,
    pub repair_actions: Vec<AdminCatalogGovernanceRepairAction>,
}

impl AdminCatalogGovernanceItemDetailResponse {
    #[must_use]
    pub fn new(
        item: AdminCatalogGovernanceItem,
        provider_mappings: Vec<AdminCatalogGovernanceProviderMappingSummary>,
    ) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            item,
            provider_mappings,
            repair_actions: vec![AdminCatalogGovernanceRepairAction::ProviderMappingReview],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceProviderMappingSummary {
    pub mapping_id: ProviderMappingId,
    pub item_id: MediaItemId,
    pub status: ProviderMappingStatus,
    pub confidence_milli: Option<u16>,
    pub source: MetadataSource,
    pub subject: AdminCatalogGovernanceProviderSubjectSummary,
}

impl AdminCatalogGovernanceProviderMappingSummary {
    #[must_use]
    pub fn from_mapping_and_subject(mapping: ProviderMapping, subject: ProviderSubject) -> Self {
        Self {
            mapping_id: mapping.id,
            item_id: mapping.item_id,
            status: mapping.status,
            confidence_milli: mapping.confidence_milli,
            source: mapping.source,
            subject: AdminCatalogGovernanceProviderSubjectSummary::from_subject(subject),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceProviderSubjectSummary {
    pub subject_id: ProviderSubjectId,
    pub provider: nako_core::ExternalProvider,
    pub subject_kind: nako_core::ProviderSubjectKind,
    pub subject_key: String,
    pub title: Option<String>,
    pub release_year: Option<i32>,
    pub locale: Option<String>,
}

impl AdminCatalogGovernanceProviderSubjectSummary {
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCatalogGovernanceRepairAction {
    ProviderMappingReview,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCatalogGovernanceProviderMappingReviewDecision {
    Accept,
    Reject,
}

impl AdminCatalogGovernanceProviderMappingReviewDecision {
    #[must_use]
    pub const fn target_status(self) -> ProviderMappingStatus {
        match self {
            Self::Accept => ProviderMappingStatus::Accepted,
            Self::Reject => ProviderMappingStatus::Rejected,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceProviderMappingReviewRequest {
    pub decision: AdminCatalogGovernanceProviderMappingReviewDecision,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceProviderMappingReviewPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub plan: AdminCatalogGovernanceProviderMappingReviewPlan,
}

impl AdminCatalogGovernanceProviderMappingReviewPlanResponse {
    #[must_use]
    pub fn new(plan: AdminCatalogGovernanceProviderMappingReviewPlan) -> Self {
        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            plan,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceProviderMappingReviewResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub item_id: MediaItemId,
    pub mapping_id: ProviderMappingId,
    pub decision: AdminCatalogGovernanceProviderMappingReviewDecision,
    pub previous_status: ProviderMappingStatus,
    pub current_status: ProviderMappingStatus,
    pub changed: bool,
    pub idempotent_replay: bool,
    pub plan: AdminCatalogGovernanceProviderMappingReviewPlan,
}

impl AdminCatalogGovernanceProviderMappingReviewResponse {
    #[must_use]
    pub fn new(
        previous_status: ProviderMappingStatus,
        plan: AdminCatalogGovernanceProviderMappingReviewPlan,
    ) -> Self {
        let changed = previous_status != plan.target_status;

        Self {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            item_id: plan.item.item_id,
            mapping_id: plan.mapping.mapping_id,
            decision: plan.decision,
            previous_status,
            current_status: plan.target_status,
            changed,
            idempotent_replay: !changed,
            plan,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceProviderMappingReviewPlan {
    pub item: AdminCatalogGovernanceItem,
    pub mapping: AdminCatalogGovernanceProviderMappingSummary,
    pub decision: AdminCatalogGovernanceProviderMappingReviewDecision,
    pub current_status: ProviderMappingStatus,
    pub target_status: ProviderMappingStatus,
    pub status: AdminCatalogGovernanceRepairPlanStatus,
    pub readiness: AdminCatalogGovernanceRepairReadiness,
    pub boundary: AdminCatalogGovernanceRepairBoundary,
}

impl AdminCatalogGovernanceProviderMappingReviewPlan {
    #[must_use]
    pub fn new(
        item: AdminCatalogGovernanceItem,
        mapping: AdminCatalogGovernanceProviderMappingSummary,
        decision: AdminCatalogGovernanceProviderMappingReviewDecision,
    ) -> Self {
        let current_status = mapping.status;
        let target_status = decision.target_status();
        let idempotent = current_status == target_status;

        Self {
            item,
            mapping,
            decision,
            current_status,
            target_status,
            status: AdminCatalogGovernanceRepairPlanStatus::Ready,
            readiness: AdminCatalogGovernanceRepairReadiness {
                status: AdminCatalogGovernanceRepairPlanStatus::Ready,
                actionable: true,
                reasons: if idempotent {
                    vec![AdminCatalogGovernanceRepairReason::AlreadyInTargetStatus]
                } else {
                    vec![AdminCatalogGovernanceRepairReason::ProviderMappingStatusChange]
                },
            },
            boundary: AdminCatalogGovernanceRepairBoundary::provider_mapping_status_only(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCatalogGovernanceRepairPlanStatus {
    Ready,
    Blocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCatalogGovernanceRepairReason {
    ProviderMappingStatusChange,
    AlreadyInTargetStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceRepairReadiness {
    pub status: AdminCatalogGovernanceRepairPlanStatus,
    pub actionable: bool,
    pub reasons: Vec<AdminCatalogGovernanceRepairReason>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCatalogGovernanceRepairBoundary {
    pub updates_provider_mapping_status: bool,
    pub updates_canonical_metadata: bool,
    pub updates_provider_subject: bool,
    pub updates_local_inference: bool,
    pub updates_source_duplicates: bool,
    pub updates_hierarchy: bool,
    pub writes_nfo: bool,
    pub writes_library_files: bool,
    pub updates_artwork: bool,
    pub updates_playback_state: bool,
}

impl AdminCatalogGovernanceRepairBoundary {
    #[must_use]
    pub const fn provider_mapping_status_only() -> Self {
        Self {
            updates_provider_mapping_status: true,
            updates_canonical_metadata: false,
            updates_provider_subject: false,
            updates_local_inference: false,
            updates_source_duplicates: false,
            updates_hierarchy: false,
            writes_nfo: false,
            writes_library_files: false,
            updates_artwork: false,
            updates_playback_state: false,
        }
    }
}

#[must_use]
pub fn catalog_governance_record_from_item_sources_and_counts(
    item: MediaItem,
    library_id: LibraryId,
    sources: Vec<nako_core::MediaSource>,
    best_local_inference: Option<LocalInferenceEvidence>,
    provider_mapping_count: u32,
    accepted_provider_mapping_count: u32,
    duplicate_relationship_count: u32,
) -> CatalogGovernanceItemRecord {
    let representative = sources.first();

    CatalogGovernanceItemRecord {
        item,
        library_id,
        source_count: sources.len() as u32,
        representative_source_id: representative.map(|source| source.id),
        representative_file_name: representative.map(|source| source.file_name.clone()),
        best_local_inference,
        provider_mapping_count,
        accepted_provider_mapping_count,
        duplicate_relationship_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_catalog_governance_item_redacts_local_inference_evidence_value() {
        let source_id = MediaSourceId::new();
        let record = CatalogGovernanceItemRecord {
            item: nako_core::MediaItem {
                id: MediaItemId::new(),
                kind: MediaKind::Unknown,
                parent_id: None,
                metadata: nako_core::CanonicalMetadata {
                    title: "Private Clip".to_owned(),
                    release_date: Some("2026-05-18".to_owned()),
                    ..nako_core::CanonicalMetadata::default()
                },
            },
            library_id: LibraryId::new(),
            source_count: 1,
            representative_source_id: Some(source_id),
            representative_file_name: Some("Private Clip.mkv".to_owned()),
            best_local_inference: Some(LocalInferenceEvidence {
                id: nako_core::LocalInferenceEvidenceId::new(),
                source_id,
                inferred_kind: MediaKind::Unknown,
                inferred_title: Some("Private Clip".to_owned()),
                inferred_year: None,
                inferred_season: None,
                inferred_episode: None,
                confidence_milli: Some(350),
                evidence_source: LocalInferenceEvidenceSource::Path,
                evidence_value: "local:///Users/admin/Private/Private Clip.mkv".to_owned(),
                inference_version: "nako-naming:1".to_owned(),
            }),
            provider_mapping_count: 0,
            accepted_provider_mapping_count: 0,
            duplicate_relationship_count: 0,
        };

        let item = AdminCatalogGovernanceItem::from_record(record, 700);
        let body = serde_json::to_string(&item).unwrap();

        let local_inference = item.local_inference.expect("local inference");
        assert!(local_inference.has_evidence);
        assert_eq!(local_inference.confidence_milli, Some(350));
        assert!(
            item.issues
                .contains(&AdminCatalogGovernanceIssue::UnknownKind)
        );
        assert!(
            item.issues
                .contains(&AdminCatalogGovernanceIssue::LowLocalInferenceConfidence)
        );
        assert!(
            item.issues
                .contains(&AdminCatalogGovernanceIssue::MissingAcceptedProviderMapping)
        );
        assert!(!body.contains("evidence_value"));
        assert!(!body.contains("local:///Users"));
        assert!(!body.contains("/Private/"));
    }
}
