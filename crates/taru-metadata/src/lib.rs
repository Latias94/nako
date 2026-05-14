use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{CanonicalMetadata, ExternalId, MediaKind, Result};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataLookup {
    pub kind: Option<MediaKind>,
    pub title: String,
    pub year: Option<u16>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidate {
    pub score: f32,
    pub metadata: CanonicalMetadata,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>>;
}
