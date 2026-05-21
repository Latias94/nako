use async_trait::async_trait;
use serde::Deserialize;
use taru_core::{
    ExternalProvider, MediaKind, MetadataCandidateGraph, ProviderSubjectKind, Result, SecretString,
    TaruError,
};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataHttpRuntimeStatus, MetadataLookup, MetadataProvider,
    MetadataProviderCapabilities, MetadataProviderCredentialRequirement,
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

    fn capabilities(&self) -> MetadataProviderCapabilities {
        MetadataProviderCapabilities {
            provider: ExternalProvider::Tmdb,
            provider_name: TMDB_PROVIDER_NAME.to_owned(),
            supported_media_kinds: vec![
                MediaKind::Movie,
                MediaKind::Series,
                MediaKind::Season,
                MediaKind::Episode,
            ],
            supported_subject_kinds: vec![
                ProviderSubjectKind::Movie,
                ProviderSubjectKind::Series,
                ProviderSubjectKind::Season,
                ProviderSubjectKind::Episode,
            ],
            supports_search: true,
            supports_fetch: true,
            supports_external_id_match: true,
            supports_hierarchy: true,
            credential_requirement: MetadataProviderCredentialRequirement::Required,
            notes: vec![
                "search supports movie and series lookups; season and episode fetch require TMDB compound provider keys".to_owned(),
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
            return Err(TaruError::Unsupported(
                "TMDB provider search currently supports movie and series lookups only",
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
            let year_key = if lookup.kind == Some(MediaKind::Series) {
                "first_air_date_year"
            } else {
                "primary_release_year"
            };
            query.push((year_key.to_owned(), year.to_string()));
        }

        let (operation, endpoint) = if lookup.kind == Some(MediaKind::Series) {
            ("search tv", self.endpoint("search/tv"))
        } else {
            ("search movie", self.endpoint("search/movie"))
        };

        let value = self
            .runtime
            .get_json(
                TMDB_PROVIDER_NAME,
                operation,
                endpoint,
                &query,
                bearer_headers(&self.config.read_access_token)?,
            )
            .await?;
        let search: TmdbSearchResponse =
            serde_json::from_value(value).map_err(|err| tmdb_parse_error(operation, err))?;

        let candidates = search
            .results
            .into_iter()
            .map(|result| {
                let score = tmdb_search_score(&lookup, &result);
                let provider_key = result.id.to_string();
                MetadataCandidate {
                    provider: ExternalProvider::Tmdb,
                    provider_key: provider_key.clone(),
                    score,
                    graph: MetadataCandidateGraph::for_provider(
                        ExternalProvider::Tmdb,
                        lookup.kind.unwrap_or(MediaKind::Movie),
                        provider_subject_kind_for_media_kind(
                            lookup.kind.unwrap_or(MediaKind::Movie),
                        ),
                        provider_key,
                        crate::mapping::tmdb_search_result_to_metadata(
                            result,
                            &self.config.image_base_url,
                        ),
                    ),
                }
            })
            .collect();

        Ok(candidates)
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
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
        let (operation, endpoint) = match request.kind {
            MediaKind::Movie => (
                "movie details",
                self.endpoint(&format!("movie/{}", request.provider_key)),
            ),
            MediaKind::Series => (
                "tv details",
                self.endpoint(&format!("tv/{}", request.provider_key)),
            ),
            MediaKind::Season => {
                let (series_id, season_number) = split_tmdb_season_key(&request.provider_key)?;
                (
                    "tv season details",
                    self.endpoint(&format!("tv/{series_id}/season/{season_number}")),
                )
            }
            MediaKind::Episode => {
                let (series_id, season_number, episode_number) =
                    split_tmdb_episode_key(&request.provider_key)?;
                (
                    "tv episode details",
                    self.endpoint(&format!(
                        "tv/{series_id}/season/{season_number}/episode/{episode_number}"
                    )),
                )
            }
            _ => {
                return Err(TaruError::Unsupported(
                    "TMDB provider fetch supports movie, series, season, and episode metadata only",
                ));
            }
        };
        let value = self
            .runtime
            .get_json(
                TMDB_PROVIDER_NAME,
                operation,
                endpoint,
                &query,
                bearer_headers(&self.config.read_access_token)?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value)
            .map_err(|err| tmdb_parse_error(&format!("serialize {operation}"), err))?;

        let (provider_key, graph) = match request.kind {
            MediaKind::Movie => {
                let details: TmdbMovieDetails = serde_json::from_value(value)
                    .map_err(|err| tmdb_parse_error(operation, err))?;
                let provider_key = details.id.to_string();
                (
                    provider_key.clone(),
                    MetadataCandidateGraph::for_provider(
                        ExternalProvider::Tmdb,
                        MediaKind::Movie,
                        ProviderSubjectKind::Movie,
                        provider_key,
                        crate::mapping::tmdb_movie_details_to_metadata(
                            details,
                            &self.config.image_base_url,
                        ),
                    ),
                )
            }
            MediaKind::Series => {
                let details: TmdbSeriesDetails = serde_json::from_value(value)
                    .map_err(|err| tmdb_parse_error(operation, err))?;
                let provider_key = details.id.to_string();
                (
                    provider_key.clone(),
                    MetadataCandidateGraph::for_provider(
                        ExternalProvider::Tmdb,
                        MediaKind::Series,
                        ProviderSubjectKind::Series,
                        provider_key,
                        crate::mapping::tmdb_series_details_to_metadata(
                            details,
                            &self.config.image_base_url,
                        ),
                    ),
                )
            }
            MediaKind::Season => {
                let details: TmdbSeasonDetails = serde_json::from_value(value)
                    .map_err(|err| tmdb_parse_error(operation, err))?;
                let provider_key = request.provider_key;
                (
                    provider_key.clone(),
                    MetadataCandidateGraph::for_provider(
                        ExternalProvider::Tmdb,
                        MediaKind::Season,
                        ProviderSubjectKind::Season,
                        provider_key,
                        crate::mapping::tmdb_season_details_to_metadata(
                            details,
                            &self.config.image_base_url,
                        ),
                    ),
                )
            }
            MediaKind::Episode => {
                let details: TmdbEpisodeDetails = serde_json::from_value(value)
                    .map_err(|err| tmdb_parse_error(operation, err))?;
                let provider_key = request.provider_key;
                (
                    provider_key.clone(),
                    MetadataCandidateGraph::for_provider(
                        ExternalProvider::Tmdb,
                        MediaKind::Episode,
                        ProviderSubjectKind::Episode,
                        provider_key,
                        crate::mapping::tmdb_episode_details_to_metadata(
                            details,
                            &self.config.image_base_url,
                        ),
                    ),
                )
            }
            _ => unreachable!("request kind was validated above"),
        };

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key,
            graph,
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
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) original_title: Option<String>,
    #[serde(default)]
    pub(crate) original_name: Option<String>,
    #[serde(default)]
    pub(crate) overview: Option<String>,
    #[serde(default)]
    pub(crate) release_date: Option<String>,
    #[serde(default)]
    pub(crate) first_air_date: Option<String>,
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

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbSeriesDetails {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) original_name: Option<String>,
    #[serde(default)]
    pub(crate) overview: Option<String>,
    #[serde(default)]
    pub(crate) first_air_date: Option<String>,
    #[serde(default)]
    pub(crate) episode_run_time: Vec<u32>,
    #[serde(default)]
    pub(crate) tagline: Option<String>,
    #[serde(default)]
    pub(crate) genres: Vec<TmdbGenre>,
    #[serde(default)]
    pub(crate) production_companies: Vec<TmdbProductionCompany>,
    #[serde(default)]
    pub(crate) poster_path: Option<String>,
    #[serde(default)]
    pub(crate) backdrop_path: Option<String>,
    #[serde(default)]
    pub(crate) credits: Option<TmdbCredits>,
    #[serde(default)]
    pub(crate) images: Option<TmdbImages>,
    #[serde(default)]
    pub(crate) external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbSeasonDetails {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) overview: Option<String>,
    #[serde(default)]
    pub(crate) air_date: Option<String>,
    #[serde(default)]
    pub(crate) season_number: Option<u32>,
    #[serde(default)]
    pub(crate) poster_path: Option<String>,
    #[serde(default)]
    pub(crate) credits: Option<TmdbCredits>,
    #[serde(default)]
    pub(crate) images: Option<TmdbImages>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbEpisodeDetails {
    pub(crate) id: u64,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) overview: Option<String>,
    #[serde(default)]
    pub(crate) air_date: Option<String>,
    #[serde(default)]
    pub(crate) episode_number: Option<u32>,
    #[serde(default)]
    pub(crate) season_number: Option<u32>,
    #[serde(default)]
    pub(crate) runtime: Option<u32>,
    #[serde(default)]
    pub(crate) still_path: Option<String>,
    #[serde(default)]
    pub(crate) credits: Option<TmdbCredits>,
    #[serde(default)]
    pub(crate) images: Option<TmdbImages>,
}

fn tmdb_search_score(lookup: &MetadataLookup, result: &TmdbMovieSearchResult) -> f32 {
    let mut score = 0.50;
    let title = result_title(result);

    if title.eq_ignore_ascii_case(&lookup.title)
        || result
            .original_title
            .as_ref()
            .is_some_and(|title| title.eq_ignore_ascii_case(&lookup.title))
        || result
            .original_name
            .as_ref()
            .is_some_and(|title| title.eq_ignore_ascii_case(&lookup.title))
    {
        score += 0.25;
    }

    if lookup.year.is_some_and(|year| {
        result
            .release_date
            .as_deref()
            .or(result.first_air_date.as_deref())
            .and_then(|value| release_year(Some(value)))
            .is_some_and(|release_year| release_year == year)
    }) {
        score += 0.20;
    }

    score + (result.popularity.clamp(0.0, 100.0) / 2_000.0)
}

pub(crate) fn result_title(result: &TmdbMovieSearchResult) -> String {
    if result.title.trim().is_empty() {
        result.name.clone()
    } else {
        result.title.clone()
    }
}

pub(crate) fn result_original_title(result: &TmdbMovieSearchResult) -> Option<String> {
    result
        .original_title
        .clone()
        .or_else(|| result.original_name.clone())
}

pub(crate) fn result_release_date(result: &TmdbMovieSearchResult) -> Option<String> {
    result
        .release_date
        .clone()
        .or_else(|| result.first_air_date.clone())
}

fn split_tmdb_season_key(value: &str) -> Result<(&str, &str)> {
    let parts = split_slash_key(value, 2)?;
    Ok((parts[0], parts[1]))
}

fn split_tmdb_episode_key(value: &str) -> Result<(&str, &str, &str)> {
    let parts = split_slash_key(value, 3)?;
    Ok((parts[0], parts[1], parts[2]))
}

fn split_slash_key(value: &str, expected_parts: usize) -> Result<Vec<&str>> {
    let parts = value.split('/').collect::<Vec<_>>();
    if parts.len() != expected_parts || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(TaruError::InvalidInput {
            message: format!(
                "TMDB provider key must contain {expected_parts} slash-separated parts"
            ),
        });
    }

    Ok(parts)
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
