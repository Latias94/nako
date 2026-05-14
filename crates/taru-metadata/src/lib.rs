use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};
use taru_core::{
    CanonicalMetadata, ContentRating, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind,
    ImageRef, JobId, MediaItem, MediaItemId, MediaKind, MediaRepository, MetadataField,
    MetadataFieldLock, MetadataRepository, ProviderRawResponse, Result, TaruError,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const DEFAULT_TMDB_API_BASE_URL: &str = "https://api.themoviedb.org/3";
const DEFAULT_TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original";
const DEFAULT_TMDB_LANGUAGE: &str = "en-US";
const TMDB_PROVIDER_NAME: &str = "tmdb";

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshJobInput {
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub force: bool,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshRequest {
    pub job_id: JobId,
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub force: bool,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshSummary {
    pub job_id: JobId,
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub matched_by: MetadataMatchKind,
    pub updated: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataMatchKind {
    ExternalId,
    Search,
}

#[derive(Debug)]
pub struct MetadataRefreshService<P, R> {
    provider: P,
    repository: R,
}

impl<P, R> MetadataRefreshService<P, R> {
    pub fn new(provider: P, repository: R) -> Self {
        Self {
            provider,
            repository,
        }
    }

    #[must_use]
    pub fn provider(&self) -> &P {
        &self.provider
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<P, R> MetadataRefreshService<P, R>
where
    P: MetadataProvider,
    R: MediaRepository + MetadataRepository,
{
    pub async fn refresh_item(
        &self,
        request: MetadataRefreshRequest,
    ) -> Result<MetadataRefreshSummary> {
        if request.provider != self.provider.provider() {
            return Err(TaruError::Unsupported(
                "metadata refresh service was constructed for a different provider",
            ));
        }

        let existing = self
            .repository
            .get_media_item(request.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: request.item_id.to_string(),
            })?;

        if existing.kind != MediaKind::Movie {
            return Err(TaruError::Unsupported(
                "TMDB metadata refresh currently supports movie items only",
            ));
        }

        let (provider_key, matched_by) = self.resolve_provider_key(&request, &existing).await?;
        let fetched = self
            .provider
            .fetch(MetadataFetchRequest {
                kind: existing.kind,
                provider_key: provider_key.clone(),
                language: request.language.clone(),
            })
            .await?;

        let locks = self.repository.list_field_locks(existing.id).await?;
        let policy = MetadataMergePolicy::from_locks(&locks);
        let merged_metadata = policy.merge(&existing.metadata, &fetched.metadata);
        let updated = merged_metadata != existing.metadata;
        let updated_item = MediaItem {
            metadata: merged_metadata,
            ..existing
        };

        self.repository.upsert_media_item(&updated_item).await?;
        self.repository
            .upsert_provider_raw_response(&ProviderRawResponse {
                item_id: updated_item.id,
                provider: fetched.provider.clone(),
                provider_key: fetched.provider_key.clone(),
                fetched_at: now_utc_string()?,
                body_json: fetched.raw_json,
            })
            .await?;

        Ok(MetadataRefreshSummary {
            job_id: request.job_id,
            item_id: updated_item.id,
            provider: fetched.provider,
            provider_key: fetched.provider_key,
            matched_by,
            updated,
        })
    }

    async fn resolve_provider_key(
        &self,
        request: &MetadataRefreshRequest,
        item: &MediaItem,
    ) -> Result<(String, MetadataMatchKind)> {
        if let Some(external_id) = item
            .metadata
            .external_ids
            .iter()
            .find(|external_id| external_id.provider == request.provider)
        {
            return Ok((external_id.value.clone(), MetadataMatchKind::ExternalId));
        }

        let lookup = MetadataLookup {
            kind: Some(item.kind),
            title: item.metadata.title.clone(),
            year: release_year(item.metadata.release_date.as_deref()),
            language: request.language.clone(),
            external_ids: item.metadata.external_ids.clone(),
        };
        let candidates = self.provider.search(lookup).await?;
        let candidate = candidates
            .into_iter()
            .filter(|candidate| candidate.provider == request.provider)
            .max_by(|left, right| left.score.total_cmp(&right.score))
            .ok_or_else(|| TaruError::NotFound {
                entity: "metadata_candidate",
                id: item.id.to_string(),
            })?;

        Ok((candidate.provider_key, MetadataMatchKind::Search))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetadataMergePolicy {
    locked_fields: HashSet<MetadataField>,
}

impl MetadataMergePolicy {
    #[must_use]
    pub fn from_locks(locks: &[MetadataFieldLock]) -> Self {
        Self {
            locked_fields: locks
                .iter()
                .filter(|lock| lock.locked)
                .map(|lock| lock.field)
                .collect(),
        }
    }

    #[must_use]
    pub fn merge(
        &self,
        existing: &CanonicalMetadata,
        incoming: &CanonicalMetadata,
    ) -> CanonicalMetadata {
        let mut merged = existing.clone();

        if !self.is_locked(MetadataField::Title) {
            merged.title = incoming.title.clone();
        }
        if !self.is_locked(MetadataField::OriginalTitle) {
            merged.original_title = incoming.original_title.clone();
        }
        if !self.is_locked(MetadataField::SortTitle) {
            merged.sort_title = incoming.sort_title.clone();
        }
        if !self.is_locked(MetadataField::Overview) {
            merged.overview = incoming.overview.clone();
        }
        if !self.is_locked(MetadataField::ReleaseDate) {
            merged.release_date = incoming.release_date.clone();
        }
        if !self.is_locked(MetadataField::RuntimeMinutes) {
            merged.runtime_minutes = incoming.runtime_minutes;
        }
        if !self.is_locked(MetadataField::Tagline) {
            merged.tagline = incoming.tagline.clone();
        }
        if !self.is_locked(MetadataField::Genres) {
            merged.genres = incoming.genres.clone();
        }
        if !self.is_locked(MetadataField::Ratings) {
            merged.ratings = incoming.ratings.clone();
        }
        if !self.is_locked(MetadataField::Images) {
            merged.images = incoming.images.clone();
        }
        if !self.is_locked(MetadataField::Credits) {
            merged.credits = incoming.credits.clone();
        }
        if !self.is_locked(MetadataField::ExternalIds) {
            merged.external_ids = incoming.external_ids.clone();
        }

        merged
    }

    fn is_locked(&self, field: MetadataField) -> bool {
        self.locked_fields.contains(&field)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TmdbProviderConfig {
    pub read_access_token: String,
    pub api_base_url: String,
    pub image_base_url: String,
    pub language: String,
    pub include_adult: bool,
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
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct TmdbMetadataProvider {
    client: reqwest::Client,
    config: TmdbProviderConfig,
}

impl TmdbMetadataProvider {
    #[must_use]
    pub fn new(config: TmdbProviderConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
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

        let response = self
            .client
            .get(self.endpoint("search/movie"))
            .bearer_auth(&self.config.read_access_token)
            .query(&query)
            .send()
            .await
            .map_err(tmdb_request_error)?;

        let value = response_json(response, "search movie").await?;
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
        let response = self
            .client
            .get(self.endpoint(&format!("movie/{}", request.provider_key)))
            .bearer_auth(&self.config.read_access_token)
            .query(&query)
            .send()
            .await
            .map_err(tmdb_request_error)?;
        let value = response_json(response, "movie details").await?;
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
struct TmdbMovieDetails {
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

async fn response_json(response: reqwest::Response, operation: &str) -> Result<serde_json::Value> {
    let status = response.status();
    let text = response.text().await.map_err(tmdb_request_error)?;

    if !status.is_success() {
        return Err(TaruError::Provider {
            provider: TMDB_PROVIDER_NAME.to_owned(),
            message: format!(
                "{operation} returned HTTP {status}: {}",
                truncate_message(&text, 240)
            ),
        });
    }

    serde_json::from_str(&text).map_err(|err| tmdb_parse_error(operation, err))
}

fn tmdb_request_error(error: reqwest::Error) -> TaruError {
    TaruError::Provider {
        provider: TMDB_PROVIDER_NAME.to_owned(),
        message: error.to_string(),
    }
}

fn tmdb_parse_error(operation: &str, error: impl ToString) -> TaruError {
    TaruError::Provider {
        provider: TMDB_PROVIDER_NAME.to_owned(),
        message: format!(
            "failed to parse TMDB {operation} response: {}",
            error.to_string()
        ),
    }
}

fn truncate_message(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    value.chars().take(max_chars).collect::<String>()
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

fn tmdb_movie_details_to_metadata(
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

fn release_year(value: Option<&str>) -> Option<u16> {
    let year = value?.get(0..4)?;

    if year.chars().all(|character| character.is_ascii_digit()) {
        year.parse().ok()
    } else {
        None
    }
}

fn now_utc_string() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| TaruError::InvalidInput {
            message: format!("failed to format metadata refresh timestamp: {err}"),
        })
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use taru_core::{
        Library, LibraryId, LibraryRepository, MediaRepository, MetadataRepository, MetadataSource,
        TransactionManager,
    };
    use taru_db::SqliteStore;

    use super::*;

    #[test]
    fn merge_preserves_locked_fields() {
        let item_id = MediaItemId::new();
        let policy = MetadataMergePolicy::from_locks(&[
            MetadataFieldLock {
                item_id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            },
            MetadataFieldLock {
                item_id,
                field: MetadataField::Genres,
                locked: true,
                source: MetadataSource::Nfo,
            },
        ]);
        let existing = CanonicalMetadata {
            title: "Local Title".to_owned(),
            overview: Some("old".to_owned()),
            genres: vec!["Local".to_owned()],
            ..CanonicalMetadata::default()
        };
        let incoming = CanonicalMetadata {
            title: "Provider Title".to_owned(),
            overview: Some("new".to_owned()),
            genres: vec!["Action".to_owned()],
            tagline: Some("Wake up.".to_owned()),
            ..CanonicalMetadata::default()
        };

        let merged = policy.merge(&existing, &incoming);

        assert_eq!(merged.title, "Local Title");
        assert_eq!(merged.overview, Some("new".to_owned()));
        assert_eq!(merged.genres, vec!["Local"]);
        assert_eq!(merged.tagline, Some("Wake up.".to_owned()));
    }

    #[tokio::test]
    async fn refresh_searches_fetches_caches_raw_and_preserves_locks() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let item = seed_movie(&store, "Local Matrix", Some("1999".to_owned()), vec![]).await;
        store
            .upsert_field_lock(&MetadataFieldLock {
                item_id: item.id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            })
            .await
            .unwrap();

        let provider = MockMetadataProvider {
            search_count: Arc::new(AtomicUsize::new(0)),
            fetch_count: Arc::new(AtomicUsize::new(0)),
            search_candidates: vec![MetadataCandidate {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                score: 0.95,
                metadata: CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    ..CanonicalMetadata::default()
                },
            }],
            fetch_result: MetadataFetchResult {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                metadata: CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    overview: Some("A hacker discovers the nature of reality.".to_owned()),
                    genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
                raw_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
            },
        };
        let search_count = provider.search_count.clone();
        let fetch_count = provider.fetch_count.clone();
        let service = MetadataRefreshService::new(provider, store.clone());

        let summary = service
            .refresh_item(MetadataRefreshRequest {
                job_id: JobId::new(),
                item_id: item.id,
                provider: ExternalProvider::Tmdb,
                force: false,
                language: None,
            })
            .await
            .unwrap();
        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let raw = store
            .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "603")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(summary.provider_key, "603");
        assert_eq!(summary.matched_by, MetadataMatchKind::Search);
        assert!(summary.updated);
        assert_eq!(loaded.metadata.title, "Local Matrix");
        assert_eq!(
            loaded.metadata.overview,
            Some("A hacker discovers the nature of reality.".to_owned())
        );
        assert_eq!(
            loaded.metadata.genres,
            vec!["Action".to_owned(), "Science Fiction".to_owned()]
        );
        assert_eq!(raw.body_json, r#"{"id":603,"title":"The Matrix"}"#);
        assert_eq!(search_count.load(Ordering::SeqCst), 1);
        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn refresh_uses_existing_external_id_without_search() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(
            &store,
            "The Matrix",
            Some("1999".to_owned()),
            vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "603".to_owned(),
            }],
        )
        .await;
        let provider = MockMetadataProvider {
            search_count: Arc::new(AtomicUsize::new(0)),
            fetch_count: Arc::new(AtomicUsize::new(0)),
            search_candidates: Vec::new(),
            fetch_result: MetadataFetchResult {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                metadata: CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    runtime_minutes: Some(136),
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
                raw_json: r#"{"id":603,"runtime":136}"#.to_owned(),
            },
        };
        let search_count = provider.search_count.clone();
        let service = MetadataRefreshService::new(provider, store.clone());

        let summary = service
            .refresh_item(MetadataRefreshRequest {
                job_id: JobId::new(),
                item_id: item.id,
                provider: ExternalProvider::Tmdb,
                force: false,
                language: None,
            })
            .await
            .unwrap();

        assert_eq!(summary.matched_by, MetadataMatchKind::ExternalId);
        assert_eq!(search_count.load(Ordering::SeqCst), 0);
        assert_eq!(
            store
                .get_media_item(item.id)
                .await
                .unwrap()
                .unwrap()
                .metadata
                .runtime_minutes,
            Some(136)
        );
    }

    #[test]
    fn tmdb_movie_details_maps_core_metadata() {
        let details: TmdbMovieDetails = serde_json::from_str(
            r#"
            {
              "id": 603,
              "title": "The Matrix",
              "original_title": "The Matrix",
              "overview": "A hacker discovers the nature of reality.",
              "release_date": "1999-03-31",
              "runtime": 136,
              "tagline": "Welcome to the Real World",
              "genres": [{"id": 28, "name": "Action"}],
              "poster_path": "/poster.jpg",
              "backdrop_path": "/backdrop.jpg",
              "external_ids": {"imdb_id": "tt0133093"},
              "credits": {
                "cast": [
                  {"id": 6384, "name": "Keanu Reeves", "character": "Neo", "order": 0}
                ],
                "crew": [
                  {"id": 9339, "name": "Lana Wachowski", "job": "Director"}
                ]
              },
              "images": {
                "posters": [
                  {"file_path": "/poster.jpg", "width": 1000, "height": 1500, "iso_639_1": "en"}
                ],
                "backdrops": [],
                "logos": []
              },
              "release_dates": {
                "results": [
                  {"iso_3166_1": "US", "release_dates": [{"certification": "R"}]}
                ]
              }
            }
            "#,
        )
        .unwrap();

        let metadata = tmdb_movie_details_to_metadata(details, DEFAULT_TMDB_IMAGE_BASE_URL);

        assert_eq!(metadata.title, "The Matrix");
        assert_eq!(metadata.runtime_minutes, Some(136));
        assert_eq!(metadata.genres, vec!["Action"]);
        assert_eq!(
            metadata.ratings,
            vec![ContentRating {
                source: "TMDB:US".to_owned(),
                value: "R".to_owned()
            }]
        );
        assert!(metadata.images.iter().any(|image| {
            image.kind == ImageKind::Poster
                && image.uri == "https://image.tmdb.org/t/p/original/poster.jpg"
        }));
        assert!(metadata.credits.iter().any(|credit| {
            credit.name == "Lana Wachowski" && credit.role == CreditRole::Director
        }));
        assert!(metadata.external_ids.iter().any(|external_id| {
            external_id.provider == ExternalProvider::Imdb && external_id.value == "tt0133093"
        }));
    }

    async fn seed_movie(
        store: &SqliteStore,
        title: &str,
        release_date: Option<String>,
        external_ids: Vec<ExternalId>,
    ) -> MediaItem {
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: title.to_owned(),
                release_date,
                external_ids,
                ..CanonicalMetadata::default()
            },
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        item
    }

    struct MockMetadataProvider {
        search_count: Arc<AtomicUsize>,
        fetch_count: Arc<AtomicUsize>,
        search_candidates: Vec<MetadataCandidate>,
        fetch_result: MetadataFetchResult,
    }

    #[async_trait]
    impl MetadataProvider for MockMetadataProvider {
        fn provider(&self) -> ExternalProvider {
            ExternalProvider::Tmdb
        }

        fn provider_name(&self) -> &'static str {
            "mock-tmdb"
        }

        async fn search(&self, _lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
            self.search_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.search_candidates.clone())
        }

        async fn fetch(&self, _request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.fetch_result.clone())
        }
    }
}
