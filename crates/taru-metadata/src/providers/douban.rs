use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use taru_core::{
    ExternalProvider, MediaKind, MetadataCandidateGraph, ProviderSubjectKind, Result, SecretString,
    TaruError,
};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataHttpRuntimeStatus, MetadataLookup, MetadataProvider,
};

use super::{
    DEFAULT_DOUBAN_API_BASE_URL, DOUBAN_PROVIDER_NAME, api_key_query, header_map_from_pairs,
    provider_parse_error, release_year,
};
#[derive(Clone, Debug)]
pub struct DoubanProviderConfig {
    pub api_key: Option<SecretString>,
    pub api_base_url: String,
    pub image_base_url: Option<String>,
    pub runtime: MetadataHttpRuntimeConfig,
    pub headers: Vec<(String, SecretString)>,
}

impl Default for DoubanProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base_url: DEFAULT_DOUBAN_API_BASE_URL.to_owned(),
            image_base_url: None,
            runtime: MetadataHttpRuntimeConfig::default(),
            headers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoubanMetadataProvider {
    runtime: MetadataHttpRuntime,
    config: DoubanProviderConfig,
}

impl DoubanMetadataProvider {
    pub fn new(config: DoubanProviderConfig) -> Result<Self> {
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

    fn query(&self) -> Vec<(String, String)> {
        api_key_query("apikey", &self.config.api_key)
    }

    fn headers(&self) -> Result<HeaderMap> {
        header_map_from_pairs(&self.config.headers)
    }
}

#[async_trait]
impl MetadataProvider for DoubanMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        ExternalProvider::Douban
    }

    fn provider_name(&self) -> &'static str {
        DOUBAN_PROVIDER_NAME
    }

    fn runtime_status(&self) -> Option<MetadataHttpRuntimeStatus> {
        Some(self.runtime.status())
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        if !lookup.kind.is_none_or(|kind| {
            matches!(
                kind,
                MediaKind::Movie
                    | MediaKind::Series
                    | MediaKind::Season
                    | MediaKind::Episode
                    | MediaKind::Unknown
            )
        }) {
            return Err(TaruError::Unsupported(
                "Douban provider supports video metadata lookups only",
            ));
        }

        let mut query = self.query();
        query.push(("q".to_owned(), lookup.title.clone()));
        query.push(("start".to_owned(), "0".to_owned()));
        query.push(("count".to_owned(), "10".to_owned()));
        let value = self
            .runtime
            .get_json(
                DOUBAN_PROVIDER_NAME,
                "search movies",
                self.endpoint("movie/search"),
                &query,
                self.headers()?,
            )
            .await?;
        let search: DoubanSearchResponse = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(DOUBAN_PROVIDER_NAME, "search movies", err))?;

        Ok(search
            .subjects
            .into_iter()
            .map(|subject| {
                let score = douban_search_score(&lookup, &subject);
                let provider_key = subject.id.clone();
                MetadataCandidate {
                    provider: ExternalProvider::Douban,
                    provider_key: provider_key.clone(),
                    score,
                    graph: MetadataCandidateGraph::for_provider(
                        ExternalProvider::Douban,
                        lookup.kind.unwrap_or(MediaKind::Unknown),
                        provider_subject_kind_for_media_kind(
                            lookup.kind.unwrap_or(MediaKind::Unknown),
                        ),
                        provider_key,
                        crate::mapping::douban_subject_to_metadata(
                            subject,
                            self.config.image_base_url.as_deref(),
                        ),
                    ),
                }
            })
            .collect())
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        if !matches!(
            request.kind,
            MediaKind::Movie
                | MediaKind::Series
                | MediaKind::Season
                | MediaKind::Episode
                | MediaKind::Unknown
        ) {
            return Err(TaruError::Unsupported(
                "Douban provider supports video metadata only",
            ));
        }

        let value = self
            .runtime
            .get_json(
                DOUBAN_PROVIDER_NAME,
                "movie details",
                self.endpoint(&format!("movie/subject/{}", request.provider_key)),
                &self.query(),
                self.headers()?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value).map_err(|err| {
            provider_parse_error(DOUBAN_PROVIDER_NAME, "serialize movie details", err)
        })?;
        let details: DoubanSubject = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(DOUBAN_PROVIDER_NAME, "movie details", err))?;

        let provider_key = details.id.clone();
        let graph = MetadataCandidateGraph::for_provider(
            ExternalProvider::Douban,
            request.kind,
            provider_subject_kind_for_media_kind(request.kind),
            provider_key.clone(),
            crate::mapping::douban_subject_to_metadata(
                details,
                self.config.image_base_url.as_deref(),
            ),
        );

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Douban,
            provider_key,
            graph,
            raw_json,
        })
    }
}

#[derive(Debug, Deserialize)]
struct DoubanSearchResponse {
    #[serde(default)]
    subjects: Vec<DoubanSubject>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DoubanSubject {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) original_title: Option<String>,
    #[serde(default)]
    pub(crate) alt_title: Option<String>,
    #[serde(default)]
    pub(crate) summary: Option<String>,
    #[serde(default)]
    pub(crate) year: Option<String>,
    #[serde(default)]
    pub(crate) images: Option<DoubanImages>,
    #[serde(default)]
    pub(crate) genres: Vec<String>,
    #[serde(default)]
    pub(crate) countries: Vec<String>,
    #[serde(default)]
    pub(crate) casts: Vec<DoubanPerson>,
    #[serde(default)]
    pub(crate) directors: Vec<DoubanPerson>,
    #[serde(default)]
    pub(crate) writers: Vec<DoubanPerson>,
    #[serde(default)]
    pub(crate) rating: Option<DoubanRating>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DoubanImages {
    #[serde(default)]
    pub(crate) small: Option<String>,
    #[serde(default)]
    pub(crate) medium: Option<String>,
    #[serde(default)]
    pub(crate) large: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DoubanPerson {
    #[serde(default)]
    pub(crate) id: Option<String>,
    #[serde(default)]
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DoubanRating {
    #[serde(default)]
    pub(crate) average: Option<f32>,
}

fn douban_search_score(lookup: &MetadataLookup, subject: &DoubanSubject) -> f32 {
    let mut score = 0.50;
    if subject.title.eq_ignore_ascii_case(&lookup.title)
        || subject
            .original_title
            .as_ref()
            .is_some_and(|title| title.eq_ignore_ascii_case(&lookup.title))
    {
        score += 0.30;
    }
    if lookup.year.is_some_and(|year| {
        subject
            .year
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
            .and_then(|rating| rating.average)
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
