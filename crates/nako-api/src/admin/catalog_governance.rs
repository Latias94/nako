use nako_client_protocol::PageInfo;
use nako_core::{
    CatalogGovernanceItemRecord, LibraryId, LocalInferenceEvidence, LocalInferenceEvidenceSource,
    MediaItemId, MediaKind, MediaSourceId,
};
use serde::{Deserialize, Serialize};

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
