use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{
    CanonicalMetadata, ExternalId, ExternalProvider, MediaKind, MetadataCandidateGraph,
    ProviderSubjectKind, Result,
};

use crate::runtime::MetadataHttpRuntimeStatus;
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
    pub graph: MetadataCandidateGraph,
}

impl MetadataCandidate {
    #[must_use]
    pub fn metadata(&self) -> CanonicalMetadata {
        self.graph.canonical_metadata()
    }
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
    pub graph: MetadataCandidateGraph,
    pub raw_json: String,
}

impl MetadataFetchResult {
    #[must_use]
    pub fn metadata(&self) -> CanonicalMetadata {
        self.graph.canonical_metadata()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderCapabilities {
    pub provider: ExternalProvider,
    pub provider_name: String,
    pub supported_media_kinds: Vec<MediaKind>,
    pub supported_subject_kinds: Vec<ProviderSubjectKind>,
    pub supports_search: bool,
    pub supports_fetch: bool,
    pub supports_external_id_match: bool,
    pub supports_hierarchy: bool,
    pub credential_requirement: MetadataProviderCredentialRequirement,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl MetadataProviderCapabilities {
    #[must_use]
    pub fn generic(provider: ExternalProvider, provider_name: impl Into<String>) -> Self {
        Self {
            provider,
            provider_name: provider_name.into(),
            supported_media_kinds: vec![
                MediaKind::Movie,
                MediaKind::Series,
                MediaKind::Season,
                MediaKind::Episode,
                MediaKind::Unknown,
            ],
            supported_subject_kinds: vec![
                ProviderSubjectKind::Movie,
                ProviderSubjectKind::Series,
                ProviderSubjectKind::Season,
                ProviderSubjectKind::Episode,
                ProviderSubjectKind::Subject,
            ],
            supports_search: true,
            supports_fetch: true,
            supports_external_id_match: true,
            supports_hierarchy: false,
            credential_requirement: MetadataProviderCredentialRequirement::Optional,
            notes: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderCredentialRequirement {
    None,
    Optional,
    Required,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn provider(&self) -> ExternalProvider;

    fn provider_name(&self) -> &'static str;

    fn capabilities(&self) -> MetadataProviderCapabilities {
        MetadataProviderCapabilities::generic(self.provider(), self.provider_name())
    }

    fn runtime_status(&self) -> Option<MetadataHttpRuntimeStatus> {
        None
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>>;

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult>;
}
