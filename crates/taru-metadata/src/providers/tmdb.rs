use std::fmt;

use async_trait::async_trait;
use serde::Deserialize;
use taru_core::{
    CanonicalMetadata, CollectionRef, ContentRating, Credit, CreditRole, ExternalId,
    ExternalProvider, ImageKind, ImageRef, MediaKind, Result, StudioRef, TaruError,
};

use crate::{
    MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataHttpRuntime,
    MetadataHttpRuntimeConfig, MetadataLookup, MetadataProvider,
};

use super::{
    DEFAULT_TMDB_API_BASE_URL, DEFAULT_TMDB_IMAGE_BASE_URL, DEFAULT_TMDB_LANGUAGE,
    TMDB_PROVIDER_NAME, bearer_headers, release_year, tmdb_parse_error,
};
#[derive(Clone, Eq, PartialEq)]
pub struct TmdbProviderConfig {
    pub read_access_token: String,
    pub api_base_url: String,
    pub image_base_url: String,
    pub language: String,
    pub include_adult: bool,
    pub runtime: MetadataHttpRuntimeConfig,
}

impl TmdbProviderConfig {
    #[must_use]
    pub fn new(read_access_token: impl Into<String>) -> Self {
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

impl fmt::Debug for TmdbProviderConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TmdbProviderConfig")
            .field("read_access_token", &"<redacted>")
            .field("api_base_url", &self.api_base_url)
            .field("image_base_url", &self.image_base_url)
            .field("language", &self.language)
            .field("include_adult", &self.include_adult)
            .field("runtime", &self.runtime)
            .finish()
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
                    metadata: tmdb_search_result_to_metadata(result, &self.config.image_base_url),
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
            metadata: tmdb_movie_details_to_metadata(details, &self.config.image_base_url),
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
struct TmdbMovieSearchResult {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    poster_path: Option<String>,
    #[serde(default)]
    backdrop_path: Option<String>,
    #[serde(default)]
    popularity: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TmdbMovieDetails {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    runtime: Option<u32>,
    #[serde(default)]
    tagline: Option<String>,
    #[serde(default)]
    genres: Vec<TmdbGenre>,
    #[serde(default)]
    belongs_to_collection: Option<TmdbCollection>,
    #[serde(default)]
    production_companies: Vec<TmdbProductionCompany>,
    #[serde(default)]
    poster_path: Option<String>,
    #[serde(default)]
    backdrop_path: Option<String>,
    #[serde(default)]
    imdb_id: Option<String>,
    #[serde(default)]
    credits: Option<TmdbCredits>,
    #[serde(default)]
    images: Option<TmdbImages>,
    #[serde(default)]
    release_dates: Option<TmdbReleaseDates>,
    #[serde(default)]
    external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TmdbCollection {
    id: u64,
    name: String,
    #[serde(default)]
    poster_path: Option<String>,
    #[serde(default)]
    backdrop_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbProductionCompany {
    id: u64,
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbCredits {
    #[serde(default)]
    cast: Vec<TmdbCastMember>,
    #[serde(default)]
    crew: Vec<TmdbCrewMember>,
}

#[derive(Debug, Deserialize)]
struct TmdbCastMember {
    #[serde(default)]
    id: Option<u64>,
    name: String,
    #[serde(default)]
    character: Option<String>,
    #[serde(default)]
    order: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TmdbCrewMember {
    #[serde(default)]
    id: Option<u64>,
    name: String,
    #[serde(default)]
    job: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbImages {
    #[serde(default)]
    posters: Vec<TmdbImage>,
    #[serde(default)]
    backdrops: Vec<TmdbImage>,
    #[serde(default)]
    logos: Vec<TmdbImage>,
}

#[derive(Debug, Deserialize)]
struct TmdbImage {
    file_path: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    iso_639_1: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbReleaseDates {
    #[serde(default)]
    results: Vec<TmdbCountryReleaseDates>,
}

#[derive(Debug, Deserialize)]
struct TmdbCountryReleaseDates {
    iso_3166_1: String,
    #[serde(default)]
    release_dates: Vec<TmdbReleaseDate>,
}

#[derive(Debug, Deserialize)]
struct TmdbReleaseDate {
    #[serde(default)]
    certification: String,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbExternalIds {
    #[serde(default)]
    imdb_id: Option<String>,
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

fn tmdb_search_result_to_metadata(
    result: TmdbMovieSearchResult,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    push_image_path(
        &mut images,
        ImageKind::Poster,
        result.poster_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );
    push_image_path(
        &mut images,
        ImageKind::Backdrop,
        result.backdrop_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );

    CanonicalMetadata {
        title: result.title,
        original_title: result.original_title,
        overview: result.overview,
        release_date: result.release_date,
        images,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: result.id.to_string(),
        }],
        ..CanonicalMetadata::default()
    }
}

pub(crate) fn tmdb_movie_details_to_metadata(
    details: TmdbMovieDetails,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut external_ids = vec![ExternalId {
        provider: ExternalProvider::Tmdb,
        value: details.id.to_string(),
    }];
    let imdb_id = details
        .external_ids
        .as_ref()
        .and_then(|ids| ids.imdb_id.as_ref())
        .or(details.imdb_id.as_ref())
        .filter(|value| !value.trim().is_empty());

    if let Some(imdb_id) = imdb_id {
        external_ids.push(ExternalId {
            provider: ExternalProvider::Imdb,
            value: imdb_id.clone(),
        });
    }

    let mut images = Vec::new();
    push_image_path(
        &mut images,
        ImageKind::Poster,
        details.poster_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );
    push_image_path(
        &mut images,
        ImageKind::Backdrop,
        details.backdrop_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );

    if let Some(collection) = details.belongs_to_collection.as_ref() {
        push_image_path(
            &mut images,
            ImageKind::Poster,
            collection.poster_path.as_deref(),
            image_base_url,
            None,
            None,
            None,
        );
        push_image_path(
            &mut images,
            ImageKind::Backdrop,
            collection.backdrop_path.as_deref(),
            image_base_url,
            None,
            None,
            None,
        );
    }

    if let Some(tmdb_images) = details.images.as_ref() {
        for image in &tmdb_images.posters {
            push_tmdb_image(&mut images, ImageKind::Poster, image, image_base_url);
        }
        for image in &tmdb_images.backdrops {
            push_tmdb_image(&mut images, ImageKind::Backdrop, image, image_base_url);
        }
        for image in &tmdb_images.logos {
            push_tmdb_image(&mut images, ImageKind::Logo, image, image_base_url);
        }
    }

    CanonicalMetadata {
        title: details.title,
        original_title: details.original_title,
        overview: details.overview,
        release_date: details.release_date,
        runtime_minutes: details.runtime,
        tagline: details.tagline,
        genres: details
            .genres
            .into_iter()
            .map(|genre| genre.name)
            .filter(|name| !name.trim().is_empty())
            .collect(),
        ratings: ratings_from_release_dates(details.release_dates.as_ref()),
        images,
        credits: credits_from_tmdb(details.credits.unwrap_or_default()),
        collections: details
            .belongs_to_collection
            .into_iter()
            .filter(|collection| !collection.name.trim().is_empty())
            .map(|collection| CollectionRef {
                name: collection.name,
                overview: None,
                sort_order: None,
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: collection.id.to_string(),
                }],
            })
            .collect(),
        studios: details
            .production_companies
            .into_iter()
            .filter(|company| !company.name.trim().is_empty())
            .map(|company| StudioRef {
                name: company.name,
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: company.id.to_string(),
                }],
            })
            .collect(),
        external_ids,
        ..CanonicalMetadata::default()
    }
}

fn ratings_from_release_dates(release_dates: Option<&TmdbReleaseDates>) -> Vec<ContentRating> {
    release_dates
        .into_iter()
        .flat_map(|dates| dates.results.iter())
        .filter_map(|country| {
            country
                .release_dates
                .iter()
                .find(|date| !date.certification.trim().is_empty())
                .map(|date| ContentRating {
                    source: format!("TMDB:{}", country.iso_3166_1),
                    value: date.certification.clone(),
                })
        })
        .collect()
}

fn credits_from_tmdb(credits: TmdbCredits) -> Vec<Credit> {
    let mut output = Vec::new();

    for member in credits.cast {
        output.push(Credit {
            name: member.name,
            role: CreditRole::Actor,
            character: member.character,
            order: member.order,
            external_ids: tmdb_person_external_ids(member.id),
        });
    }

    for member in credits.crew {
        output.push(Credit {
            name: member.name,
            role: credit_role_from_tmdb_job(member.job.as_deref()),
            character: None,
            order: None,
            external_ids: tmdb_person_external_ids(member.id),
        });
    }

    output
}

fn credit_role_from_tmdb_job(job: Option<&str>) -> CreditRole {
    match job.unwrap_or_default().to_ascii_lowercase().as_str() {
        "director" => CreditRole::Director,
        "writer" | "screenplay" | "story" => CreditRole::Writer,
        "producer" | "executive producer" => CreditRole::Producer,
        "creator" => CreditRole::Creator,
        value if value.is_empty() => CreditRole::Other("crew".to_owned()),
        value => CreditRole::Other(value.to_owned()),
    }
}

fn tmdb_person_external_ids(id: Option<u64>) -> Vec<ExternalId> {
    id.map(|id| ExternalId {
        provider: ExternalProvider::Tmdb,
        value: id.to_string(),
    })
    .into_iter()
    .collect()
}

fn push_tmdb_image(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    image: &TmdbImage,
    image_base_url: &str,
) {
    push_image_path(
        images,
        kind,
        Some(&image.file_path),
        image_base_url,
        image.width,
        image.height,
        image.iso_639_1.clone(),
    );
}

fn push_image_path(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    path: Option<&str>,
    image_base_url: &str,
    width: Option<u32>,
    height: Option<u32>,
    language: Option<String>,
) {
    let Some(path) = path.filter(|path| !path.trim().is_empty()) else {
        return;
    };

    let uri = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_owned()
    } else {
        format!("{}{}", image_base_url.trim_end_matches('/'), path)
    };

    if images
        .iter()
        .any(|image| image.kind == kind && image.uri == uri)
    {
        return;
    }

    images.push(ImageRef {
        kind,
        uri,
        provider: ExternalProvider::Tmdb,
        width,
        height,
        language,
    });
}
