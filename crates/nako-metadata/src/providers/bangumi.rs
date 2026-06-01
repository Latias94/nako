use async_trait::async_trait;
use nako_core::{
    ExternalProvider, MediaKind, MetadataCandidateGraph, MetadataCandidateNode,
    MetadataCandidateRelationship, MetadataCandidateRelationshipKind, MetadataCandidateSource,
    MetadataCandidateSubject, NakoError, ProviderSubjectKind, Result, SecretString,
};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataHttpRuntimeStatus, MetadataLookup, MetadataProvider,
    MetadataProviderCapabilities, MetadataProviderCredentialRequirement,
};

use super::{
    BANGUMI_PROVIDER_NAME, DEFAULT_BANGUMI_API_BASE_URL, DEFAULT_BANGUMI_IMAGE_BASE_URL,
    bearer_headers, provider_parse_error, release_year,
};
#[derive(Clone, Debug)]
pub struct BangumiProviderConfig {
    pub access_token: Option<SecretString>,
    pub api_base_url: String,
    pub image_base_url: String,
    pub include_nsfw: bool,
    pub runtime: MetadataHttpRuntimeConfig,
}

impl Default for BangumiProviderConfig {
    fn default() -> Self {
        Self {
            access_token: None,
            api_base_url: DEFAULT_BANGUMI_API_BASE_URL.to_owned(),
            image_base_url: DEFAULT_BANGUMI_IMAGE_BASE_URL.to_owned(),
            include_nsfw: false,
            runtime: MetadataHttpRuntimeConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BangumiMetadataProvider {
    runtime: MetadataHttpRuntime,
    config: BangumiProviderConfig,
}

impl BangumiMetadataProvider {
    pub fn new(config: BangumiProviderConfig) -> Result<Self> {
        let runtime = MetadataHttpRuntime::new(config.runtime.clone())?;
        Ok(Self { runtime, config })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn headers(&self) -> Result<HeaderMap> {
        self.config
            .access_token
            .as_ref()
            .filter(|token| !token.is_blank())
            .map(bearer_headers)
            .unwrap_or_else(|| Ok(HeaderMap::new()))
    }
}

#[async_trait]
impl MetadataProvider for BangumiMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        ExternalProvider::Bangumi
    }

    fn provider_name(&self) -> &'static str {
        BANGUMI_PROVIDER_NAME
    }

    fn capabilities(&self) -> MetadataProviderCapabilities {
        MetadataProviderCapabilities {
            provider: ExternalProvider::Bangumi,
            provider_name: BANGUMI_PROVIDER_NAME.to_owned(),
            supported_media_kinds: vec![
                MediaKind::Movie,
                MediaKind::Series,
                MediaKind::Unknown,
            ],
            supported_subject_kinds: vec![
                ProviderSubjectKind::Movie,
                ProviderSubjectKind::Series,
                ProviderSubjectKind::Subject,
            ],
            supports_search: true,
            supports_fetch: true,
            supports_external_id_match: true,
            supports_hierarchy: false,
            credential_requirement: MetadataProviderCredentialRequirement::Optional,
            notes: vec![
                "Bangumi subject metadata is subject-level and anime-first; related episode graph preview does not imply direct Episode fetch support".to_owned(),
                "Hierarchy confirmation remains Nako-owned".to_owned(),
            ],
        }
    }

    fn runtime_status(&self) -> Option<MetadataHttpRuntimeStatus> {
        Some(self.runtime.status())
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        if !lookup.kind.is_none_or(|kind| {
            matches!(
                kind,
                MediaKind::Movie | MediaKind::Series | MediaKind::Unknown
            )
        }) {
            return Err(NakoError::Unsupported(
                "Bangumi provider supports subject-level anime lookups only; season and episode require dedicated endpoints",
            ));
        }

        let query = vec![
            ("limit".to_owned(), "10".to_owned()),
            ("offset".to_owned(), "0".to_owned()),
        ];
        let body = BangumiSearchRequest {
            keyword: lookup.title.clone(),
            sort: "match".to_owned(),
            filter: BangumiSearchFilter {
                subject_type: Some(vec![2]),
                nsfw: Some(self.config.include_nsfw),
            },
        };
        let value = self
            .runtime
            .post_json(
                BANGUMI_PROVIDER_NAME,
                "search subjects",
                self.endpoint("v0/search/subjects"),
                &query,
                self.headers()?,
                &body,
            )
            .await?;
        let search: BangumiSearchResponse = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(BANGUMI_PROVIDER_NAME, "search subjects", err))?;

        Ok(search
            .data
            .into_iter()
            .map(|subject| {
                let score = bangumi_search_score(&lookup, &subject);
                let provider_key = subject.id.to_string();
                MetadataCandidate {
                    provider: ExternalProvider::Bangumi,
                    provider_key: provider_key.clone(),
                    score,
                    graph: MetadataCandidateGraph::for_provider(
                        ExternalProvider::Bangumi,
                        lookup.kind.unwrap_or(MediaKind::Unknown),
                        provider_subject_kind_for_media_kind(
                            lookup.kind.unwrap_or(MediaKind::Unknown),
                        ),
                        provider_key,
                        crate::mapping::bangumi_subject_to_metadata(
                            subject,
                            &self.config.image_base_url,
                        ),
                    ),
                }
            })
            .collect())
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        if !matches!(
            request.kind,
            MediaKind::Movie | MediaKind::Series | MediaKind::Unknown
        ) {
            return Err(NakoError::Unsupported(
                "Bangumi provider supports subject-level anime fetches only; season and episode require dedicated endpoints",
            ));
        }

        let subject_value = self
            .runtime
            .get_json(
                BANGUMI_PROVIDER_NAME,
                "subject details",
                self.endpoint(&format!("v0/subjects/{}", request.provider_key)),
                &[],
                self.headers()?,
            )
            .await?;
        let details: BangumiSubject = serde_json::from_value(subject_value.clone())
            .map_err(|err| provider_parse_error(BANGUMI_PROVIDER_NAME, "subject details", err))?;

        let provider_key = details.id.to_string();
        let mut raw_value = subject_value.clone();
        let mut episodes = Vec::new();
        if request.kind == MediaKind::Series {
            let query = vec![
                ("subject_id".to_owned(), provider_key.clone()),
                ("type".to_owned(), "0".to_owned()),
                ("limit".to_owned(), "200".to_owned()),
                ("offset".to_owned(), "0".to_owned()),
            ];
            let episodes_value = self
                .runtime
                .get_json(
                    BANGUMI_PROVIDER_NAME,
                    "subject episodes",
                    self.endpoint("v0/episodes"),
                    &query,
                    self.headers()?,
                )
                .await?;
            let page: BangumiEpisodePage =
                serde_json::from_value(episodes_value.clone()).map_err(|err| {
                    provider_parse_error(BANGUMI_PROVIDER_NAME, "subject episodes", err)
                })?;
            episodes = page.data;
            raw_value = serde_json::json!({
                "subject": subject_value,
                "episodes": episodes_value,
            });
        }
        let raw_json = serde_json::to_string(&raw_value).map_err(|err| {
            provider_parse_error(BANGUMI_PROVIDER_NAME, "serialize subject details", err)
        })?;
        let mut graph = MetadataCandidateGraph::for_provider(
            ExternalProvider::Bangumi,
            request.kind,
            provider_subject_kind_for_media_kind(request.kind),
            provider_key.clone(),
            crate::mapping::bangumi_subject_to_metadata(details, &self.config.image_base_url),
        );
        append_bangumi_episode_graph(&mut graph, episodes);

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Bangumi,
            provider_key,
            graph,
            raw_json,
        })
    }
}

#[derive(Debug, Serialize)]
struct BangumiSearchRequest {
    keyword: String,
    sort: String,
    filter: BangumiSearchFilter,
}

#[derive(Debug, Serialize)]
struct BangumiSearchFilter {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    subject_type: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BangumiSearchResponse {
    #[serde(default)]
    data: Vec<BangumiSubject>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BangumiSubject {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) name_cn: String,
    #[serde(default)]
    pub(crate) summary: Option<String>,
    #[serde(default)]
    pub(crate) date: Option<String>,
    #[serde(default)]
    pub(crate) images: Option<BangumiImages>,
    #[serde(default)]
    pub(crate) infobox: Vec<BangumiInfoBoxItem>,
    #[serde(default)]
    pub(crate) tags: Vec<BangumiTag>,
    #[serde(default)]
    pub(crate) rating: Option<BangumiRating>,
}

#[derive(Debug, Deserialize)]
struct BangumiEpisodePage {
    #[serde(default)]
    data: Vec<BangumiEpisode>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BangumiEpisode {
    pub(crate) id: u64,
    #[serde(default, rename = "type")]
    pub(crate) episode_type: u8,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) name_cn: String,
    #[serde(default)]
    pub(crate) airdate: Option<String>,
    #[serde(default)]
    pub(crate) desc: Option<String>,
    #[serde(default)]
    pub(crate) duration_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BangumiImages {
    #[serde(default)]
    pub(crate) large: Option<String>,
    #[serde(default)]
    pub(crate) common: Option<String>,
    #[serde(default)]
    pub(crate) medium: Option<String>,
    #[serde(default)]
    pub(crate) small: Option<String>,
    #[serde(default)]
    pub(crate) grid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BangumiInfoBoxItem {
    pub(crate) key: String,
    pub(crate) value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BangumiTag {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BangumiRating {
    #[serde(default)]
    pub(crate) score: Option<f32>,
}

fn bangumi_search_score(lookup: &MetadataLookup, subject: &BangumiSubject) -> f32 {
    let mut score = 0.50;
    if subject.name.eq_ignore_ascii_case(&lookup.title)
        || subject.name_cn.eq_ignore_ascii_case(&lookup.title)
    {
        score += 0.30;
    }
    if lookup.year.is_some_and(|year| {
        subject
            .date
            .as_deref()
            .and_then(|value| release_year(Some(value)))
            .is_some_and(|release_year| release_year == year)
    }) {
        score += 0.15;
    }
    score
        + subject
            .rating
            .as_ref()
            .and_then(|rating| rating.score)
            .unwrap_or(0.0)
            / 200.0
}

fn provider_subject_kind_for_media_kind(kind: MediaKind) -> ProviderSubjectKind {
    match kind {
        MediaKind::Movie => ProviderSubjectKind::Movie,
        MediaKind::Series => ProviderSubjectKind::Series,
        MediaKind::Season => ProviderSubjectKind::Season,
        MediaKind::Episode => ProviderSubjectKind::Episode,
        MediaKind::Collection => ProviderSubjectKind::Collection,
        MediaKind::Extra | MediaKind::Unknown => ProviderSubjectKind::Subject,
    }
}

fn append_bangumi_episode_graph(graph: &mut MetadataCandidateGraph, episodes: Vec<BangumiEpisode>) {
    let Some(parent_subject) = graph.root_provider_subject().cloned() else {
        return;
    };

    for episode in episodes {
        if episode.episode_type != 0 {
            continue;
        }
        let metadata = crate::mapping::bangumi_episode_to_metadata(&episode);
        let subject = MetadataCandidateSubject {
            provider: ExternalProvider::Bangumi,
            subject_kind: ProviderSubjectKind::Episode,
            subject_key: episode.id.to_string(),
            title: metadata.title.clone(),
            release_year: release_year(metadata.release_date.as_deref()).map(i32::from),
            locale: None,
        };
        graph.relationships.push(MetadataCandidateRelationship {
            parent_subject: parent_subject.clone(),
            child_subject: subject.clone(),
            kind: MetadataCandidateRelationshipKind::Contains,
        });
        graph.related.push(MetadataCandidateNode {
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            kind: MediaKind::Episode,
            subject: Some(subject),
            metadata,
        });
    }
}
