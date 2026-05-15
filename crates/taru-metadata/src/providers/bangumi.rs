use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use taru_core::{
    CanonicalMetadata, ContentRating, ExternalId, ExternalProvider, ImageKind, MediaKind, Result,
    StudioRef, TaruError,
};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataLookup, MetadataProvider,
};

use super::{
    BANGUMI_PROVIDER_NAME, DEFAULT_BANGUMI_API_BASE_URL, DEFAULT_BANGUMI_IMAGE_BASE_URL,
    bearer_headers, first_non_empty, non_empty_string, provider_parse_error,
    push_provider_image_uri, release_year,
};
#[derive(Clone, Debug)]
pub struct BangumiProviderConfig {
    pub access_token: Option<String>,
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
            .as_deref()
            .filter(|token| !token.trim().is_empty())
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
                "Bangumi provider supports video metadata lookups only",
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
                MetadataCandidate {
                    provider: ExternalProvider::Bangumi,
                    provider_key: subject.id.to_string(),
                    score,
                    metadata: crate::mapping::bangumi_subject_to_metadata(
                        subject,
                        &self.config.image_base_url,
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
                "Bangumi provider supports video metadata only",
            ));
        }

        let value = self
            .runtime
            .get_json(
                BANGUMI_PROVIDER_NAME,
                "subject details",
                self.endpoint(&format!("v0/subjects/{}", request.provider_key)),
                &[],
                self.headers()?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value).map_err(|err| {
            provider_parse_error(BANGUMI_PROVIDER_NAME, "serialize subject details", err)
        })?;
        let details: BangumiSubject = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(BANGUMI_PROVIDER_NAME, "subject details", err))?;

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Bangumi,
            provider_key: details.id.to_string(),
            metadata: crate::mapping::bangumi_subject_to_metadata(
                details,
                &self.config.image_base_url,
            ),
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
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    name_cn: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    images: Option<BangumiImages>,
    #[serde(default)]
    infobox: Vec<BangumiInfoBoxItem>,
    #[serde(default)]
    tags: Vec<BangumiTag>,
    #[serde(default)]
    rating: Option<BangumiRating>,
}

#[derive(Debug, Deserialize)]
struct BangumiImages {
    #[serde(default)]
    large: Option<String>,
    #[serde(default)]
    common: Option<String>,
    #[serde(default)]
    medium: Option<String>,
    #[serde(default)]
    small: Option<String>,
    #[serde(default)]
    grid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BangumiInfoBoxItem {
    key: String,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct BangumiTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BangumiRating {
    #[serde(default)]
    score: Option<f32>,
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

pub(crate) fn bangumi_subject_to_metadata(
    subject: BangumiSubject,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    if let Some(subject_images) = subject.images.as_ref() {
        for uri in [
            subject_images.large.as_deref(),
            subject_images.common.as_deref(),
            subject_images.medium.as_deref(),
            subject_images.small.as_deref(),
            subject_images.grid.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_provider_image_uri(
                &mut images,
                ImageKind::Poster,
                Some(uri),
                image_base_url,
                ExternalProvider::Bangumi,
                None,
                None,
                None,
            );
        }
    }

    let studios = bangumi_infobox_strings(&subject.infobox, &["动画制作", "制作", "製作"])
        .into_iter()
        .map(|name| StudioRef {
            name,
            external_ids: Vec::new(),
        })
        .collect();
    let tags = subject
        .tags
        .into_iter()
        .map(|tag| tag.name)
        .filter(|name| !name.trim().is_empty())
        .collect();

    CanonicalMetadata {
        title: first_non_empty(&[Some(subject.name_cn.as_str()), Some(subject.name.as_str())])
            .unwrap_or_default(),
        original_title: non_empty_string(subject.name),
        overview: subject.summary.filter(|value| !value.trim().is_empty()),
        release_date: subject.date.filter(|value| !value.trim().is_empty()),
        runtime_minutes: None,
        tags,
        ratings: subject
            .rating
            .and_then(|rating| rating.score)
            .map(|score| ContentRating {
                source: "Bangumi:score".to_owned(),
                value: score.to_string(),
            })
            .into_iter()
            .collect(),
        images,
        studios,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Bangumi,
            value: subject.id.to_string(),
        }],
        ..CanonicalMetadata::default()
    }
}

fn bangumi_infobox_strings(items: &[BangumiInfoBoxItem], keys: &[&str]) -> Vec<String> {
    items
        .iter()
        .filter(|item| keys.iter().any(|key| item.key == *key))
        .flat_map(|item| metadata_strings_from_json(&item.value))
        .collect()
}

fn metadata_strings_from_json(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(value) => non_empty_string(value.clone()).into_iter().collect(),
        serde_json::Value::Array(values) => values
            .iter()
            .flat_map(metadata_strings_from_json)
            .collect::<Vec<_>>(),
        serde_json::Value::Object(map) => map
            .get("v")
            .or_else(|| map.get("value"))
            .into_iter()
            .flat_map(metadata_strings_from_json)
            .collect(),
        _ => Vec::new(),
    }
}
