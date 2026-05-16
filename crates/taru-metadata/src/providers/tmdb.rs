use async_trait::async_trait;
use serde::Deserialize;
use taru_core::{ExternalProvider, MediaKind, Result, SecretString, TaruError};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataHttpRuntimeStatus, MetadataLookup, MetadataProvider,
};

use super::{
    DEFAULT_TMDB_API_BASE_URL, DEFAULT_TMDB_IMAGE_BASE_URL, DEFAULT_TMDB_LANGUAGE,
    TMDB_PROVIDER_NAME, bearer_headers, release_year, tmdb_parse_error,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TmdbProviderConfig {
    pub read_access_token: SecretString,
    pub api_base_url: String,
    pub image_base_url: String,
    pub language: String,
    pub include_adult: bool,
    pub runtime: MetadataHttpRuntimeConfig,
}

impl TmdbProviderConfig {
    #[must_use]
    pub fn new(read_access_token: impl Into<SecretString>) -> Self {
        Self {
            read_access_token: read_access_token.into(),
            api_base_url: DEFAULT_TMDB_API_BASE_URL.to_owned(),
            image_base_url: DEFAULT_TMDB_IMAGE_BASE_URL.to_owned(),
            language: DEFAULT_TMDB_LANGUAGE.to_owned(),
            include_adult: false,
            runtime: MetadataHttpRuntimeConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TmdbMetadataProvider {
    runtime: MetadataHttpRuntime,
    config: TmdbProviderConfig,
}

impl TmdbMetadataProvider {
    pub fn new(config: TmdbProviderConfig) -> Result<Self> {
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

    fn language(&self, override_language: Option<&str>) -> String {
        override_language
            .filter(|language| !language.trim().is_empty())
            .unwrap_or(&self.config.language)
            .to_owned()
    }
}

#[async_trait]
impl MetadataProvider for TmdbMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        ExternalProvider::Tmdb
    }

    fn provider_name(&self) -> &'static str {
        TMDB_PROVIDER_NAME
    }

    fn runtime_status(&self) -> Option<MetadataHttpRuntimeStatus> {
        Some(self.runtime.status())
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        if !lookup
            .kind
            .is_none_or(|kind| kind == MediaKind::Movie || kind == MediaKind::Unknown)
        {
            return Err(TaruError::Unsupported(
                "TMDB provider search currently supports movie lookups only",
            ));
        }

        let mut query = vec![
            ("query".to_owned(), lookup.title.clone()),
            (
                "include_adult".to_owned(),
                self.config.include_adult.to_string(),
            ),
            (
                "language".to_owned(),
                self.language(lookup.language.as_deref()),
            ),
            ("page".to_owned(), "1".to_owned()),
        ];

        if let Some(year) = lookup.year {
            query.push(("primary_release_year".to_owned(), year.to_string()));
        }

        let value = self
            .runtime
            .get_json(
                TMDB_PROVIDER_NAME,
                "search movie",
                self.endpoint("search/movie"),
                &query,
                bearer_headers(&self.config.read_access_token)?,
            )
            .await?;
        let search: TmdbSearchResponse =
            serde_json::from_value(value).map_err(|err| tmdb_parse_error("search movie", err))?;

        let candidates = search
            .results
            .into_iter()
            .map(|result| {
                let score = tmdb_search_score(&lookup, &result);
                MetadataCandidate {
                    provider: ExternalProvider::Tmdb,
                    provider_key: result.id.to_string(),
                    score,
                    metadata: crate::mapping::tmdb_search_result_to_metadata(
                        result,
                        &self.config.image_base_url,
                    ),
                }
            })
            .collect();

        Ok(candidates)
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        if request.kind != MediaKind::Movie {
            return Err(TaruError::Unsupported(
                "TMDB provider fetch currently supports movie metadata only",
            ));
        }

        let query = [
            (
                "language".to_owned(),
                self.language(request.language.as_deref()),
            ),
            (
                "append_to_response".to_owned(),
                "credits,images,release_dates,external_ids".to_owned(),
            ),
        ];
        let value = self
            .runtime
            .get_json(
                TMDB_PROVIDER_NAME,
                "movie details",
                self.endpoint(&format!("movie/{}", request.provider_key)),
                &query,
                bearer_headers(&self.config.read_access_token)?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value)
            .map_err(|err| tmdb_parse_error("serialize movie details", err))?;
        let details: TmdbMovieDetails =
            serde_json::from_value(value).map_err(|err| tmdb_parse_error("movie details", err))?;

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: details.id.to_string(),
            metadata: crate::mapping::tmdb_movie_details_to_metadata(
                details,
                &self.config.image_base_url,
            ),
            raw_json,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    #[serde(default)]
    results: Vec<TmdbMovieSearchResult>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbMovieSearchResult {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) original_title: Option<String>,
    #[serde(default)]
    pub(crate) overview: Option<String>,
    #[serde(default)]
    pub(crate) release_date: Option<String>,
    #[serde(default)]
    pub(crate) poster_path: Option<String>,
    #[serde(default)]
    pub(crate) backdrop_path: Option<String>,
    #[serde(default)]
    pub(crate) popularity: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbMovieDetails {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) original_title: Option<String>,
    #[serde(default)]
    pub(crate) overview: Option<String>,
    #[serde(default)]
    pub(crate) release_date: Option<String>,
    #[serde(default)]
    pub(crate) runtime: Option<u32>,
    #[serde(default)]
    pub(crate) tagline: Option<String>,
    #[serde(default)]
    pub(crate) genres: Vec<TmdbGenre>,
    #[serde(default)]
    pub(crate) belongs_to_collection: Option<TmdbCollection>,
    #[serde(default)]
    pub(crate) production_companies: Vec<TmdbProductionCompany>,
    #[serde(default)]
    pub(crate) poster_path: Option<String>,
    #[serde(default)]
    pub(crate) backdrop_path: Option<String>,
    #[serde(default)]
    pub(crate) imdb_id: Option<String>,
    #[serde(default)]
    pub(crate) credits: Option<TmdbCredits>,
    #[serde(default)]
    pub(crate) images: Option<TmdbImages>,
    #[serde(default)]
    pub(crate) release_dates: Option<TmdbReleaseDates>,
    #[serde(default)]
    pub(crate) external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbGenre {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbCollection {
    pub(crate) id: u64,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) poster_path: Option<String>,
    #[serde(default)]
    pub(crate) backdrop_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbProductionCompany {
    pub(crate) id: u64,
    pub(crate) name: String,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct TmdbCredits {
    #[serde(default)]
    pub(crate) cast: Vec<TmdbCastMember>,
    #[serde(default)]
    pub(crate) crew: Vec<TmdbCrewMember>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbCastMember {
    #[serde(default)]
    pub(crate) id: Option<u64>,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) character: Option<String>,
    #[serde(default)]
    pub(crate) order: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbCrewMember {
    #[serde(default)]
    pub(crate) id: Option<u64>,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) job: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct TmdbImages {
    #[serde(default)]
    pub(crate) posters: Vec<TmdbImage>,
    #[serde(default)]
    pub(crate) backdrops: Vec<TmdbImage>,
    #[serde(default)]
    pub(crate) logos: Vec<TmdbImage>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbImage {
    pub(crate) file_path: String,
    #[serde(default)]
    pub(crate) width: Option<u32>,
    #[serde(default)]
    pub(crate) height: Option<u32>,
    #[serde(default)]
    pub(crate) iso_639_1: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct TmdbReleaseDates {
    #[serde(default)]
    pub(crate) results: Vec<TmdbCountryReleaseDates>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbCountryReleaseDates {
    pub(crate) iso_3166_1: String,
    #[serde(default)]
    pub(crate) release_dates: Vec<TmdbReleaseDate>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbReleaseDate {
    #[serde(default)]
    pub(crate) certification: String,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct TmdbExternalIds {
    #[serde(default)]
    pub(crate) imdb_id: Option<String>,
}

fn tmdb_search_score(lookup: &MetadataLookup, result: &TmdbMovieSearchResult) -> f32 {
    let mut score = 0.50;

    if result.title.eq_ignore_ascii_case(&lookup.title)
        || result
            .original_title
            .as_ref()
            .is_some_and(|title| title.eq_ignore_ascii_case(&lookup.title))
    {
        score += 0.25;
    }

    if lookup.year.is_some_and(|year| {
        result
            .release_date
            .as_deref()
            .and_then(|value| release_year(Some(value)))
            .is_some_and(|release_year| release_year == year)
    }) {
        score += 0.20;
    }

    score + (result.popularity.clamp(0.0, 100.0) / 2_000.0)
}
