use std::convert::TryFrom;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use taru_core::{
    CanonicalMetadata, ContentRating, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind,
    MediaKind, Result, TaruError,
};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataLookup, MetadataProvider,
};

use super::{
    DEFAULT_DOUBAN_API_BASE_URL, DOUBAN_PROVIDER_NAME, api_key_query, header_map_from_pairs,
    provider_parse_error, push_provider_image_uri, release_year,
};
#[derive(Clone, Debug)]
pub struct DoubanProviderConfig {
    pub api_key: Option<String>,
    pub api_base_url: String,
    pub image_base_url: Option<String>,
    pub runtime: MetadataHttpRuntimeConfig,
    pub headers: Vec<(String, String)>,
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
                MetadataCandidate {
                    provider: ExternalProvider::Douban,
                    provider_key: subject.id.clone(),
                    score,
                    metadata: douban_subject_to_metadata(
                        subject,
                        self.config.image_base_url.as_deref(),
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

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Douban,
            provider_key: details.id.clone(),
            metadata: douban_subject_to_metadata(details, self.config.image_base_url.as_deref()),
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
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    alt_title: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    year: Option<String>,
    #[serde(default)]
    images: Option<DoubanImages>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    countries: Vec<String>,
    #[serde(default)]
    casts: Vec<DoubanPerson>,
    #[serde(default)]
    directors: Vec<DoubanPerson>,
    #[serde(default)]
    writers: Vec<DoubanPerson>,
    #[serde(default)]
    rating: Option<DoubanRating>,
}

#[derive(Debug, Deserialize)]
struct DoubanImages {
    #[serde(default)]
    small: Option<String>,
    #[serde(default)]
    medium: Option<String>,
    #[serde(default)]
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DoubanPerson {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct DoubanRating {
    #[serde(default)]
    average: Option<f32>,
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

pub(crate) fn douban_subject_to_metadata(
    subject: DoubanSubject,
    image_base_url: Option<&str>,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    if let Some(subject_images) = subject.images.as_ref() {
        for uri in [
            subject_images.large.as_deref(),
            subject_images.medium.as_deref(),
            subject_images.small.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_provider_image_uri(
                &mut images,
                ImageKind::Poster,
                Some(uri),
                image_base_url.unwrap_or_default(),
                ExternalProvider::Douban,
                None,
                None,
                None,
            );
        }
    }

    let mut credits = Vec::new();
    for person in subject.directors {
        push_douban_credit(&mut credits, person, CreditRole::Director);
    }
    for person in subject.writers {
        push_douban_credit(&mut credits, person, CreditRole::Writer);
    }
    for (order, person) in subject.casts.into_iter().enumerate() {
        let mut credit = douban_person_credit(person, CreditRole::Actor);
        credit.order = u32::try_from(order).ok();
        credits.push(credit);
    }

    let release_date = subject
        .year
        .as_ref()
        .filter(|year| year.len() == 4 && year.chars().all(|character| character.is_ascii_digit()))
        .map(|year| format!("{year}-01-01"));

    CanonicalMetadata {
        title: subject.title,
        original_title: subject.original_title.or(subject.alt_title),
        overview: subject.summary.filter(|value| !value.trim().is_empty()),
        release_date,
        genres: subject
            .genres
            .into_iter()
            .filter(|genre| !genre.trim().is_empty())
            .collect(),
        tags: subject
            .countries
            .into_iter()
            .filter(|country| !country.trim().is_empty())
            .collect(),
        ratings: subject
            .rating
            .and_then(|rating| rating.average)
            .map(|score| ContentRating {
                source: "Douban:score".to_owned(),
                value: score.to_string(),
            })
            .into_iter()
            .collect(),
        images,
        credits,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Douban,
            value: subject.id,
        }],
        ..CanonicalMetadata::default()
    }
}

fn push_douban_credit(credits: &mut Vec<Credit>, person: DoubanPerson, role: CreditRole) {
    if person.name.trim().is_empty() {
        return;
    }
    credits.push(douban_person_credit(person, role));
}

fn douban_person_credit(person: DoubanPerson, role: CreditRole) -> Credit {
    Credit {
        name: person.name,
        role,
        character: None,
        order: None,
        external_ids: person
            .id
            .filter(|id| !id.trim().is_empty())
            .map(|id| ExternalId {
                provider: ExternalProvider::Douban,
                value: id,
            })
            .into_iter()
            .collect(),
    }
}
