use async_trait::async_trait;

use super::PageRequest;
use crate::{LibraryId, LocalInferenceEvidence, MediaItem, MediaSourceId, Result};

pub const DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI: u16 = 700;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogGovernanceItemListFilter {
    pub library_id: Option<LibraryId>,
    pub max_confidence_milli: u16,
}

impl Default for CatalogGovernanceItemListFilter {
    fn default() -> Self {
        Self {
            library_id: None,
            max_confidence_milli: DEFAULT_CATALOG_GOVERNANCE_CONFIDENCE_THRESHOLD_MILLI,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogGovernanceItemRecord {
    pub item: MediaItem,
    pub library_id: LibraryId,
    pub source_count: u32,
    pub representative_source_id: Option<MediaSourceId>,
    pub representative_file_name: Option<String>,
    pub best_local_inference: Option<LocalInferenceEvidence>,
    pub provider_mapping_count: u32,
    pub accepted_provider_mapping_count: u32,
    pub duplicate_relationship_count: u32,
}

#[async_trait]
pub trait CatalogGovernanceRepository: Send + Sync {
    async fn list_catalog_governance_items(
        &self,
        filter: CatalogGovernanceItemListFilter,
        page: PageRequest,
    ) -> Result<Vec<CatalogGovernanceItemRecord>>;
}
