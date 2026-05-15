use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{CanonicalMetadata, ExternalId, ExternalProvider, MediaKind, Result};
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataLookup {
    pub kind: Option<MediaKind>,
    pub title: String,
    pub year: Option<u16>,
    pub language: Option<String>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidate {
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub score: f32,
    pub metadata: CanonicalMetadata,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataFetchRequest {
    pub kind: MediaKind,
    pub provider_key: String,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataFetchResult {
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub metadata: CanonicalMetadata,
    pub raw_json: String,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn provider(&self) -> ExternalProvider;

    fn provider_name(&self) -> &'static str;

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>>;

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult>;
}
