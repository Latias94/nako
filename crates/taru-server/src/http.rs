use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use taru_api::{
    API_VERSION, EnqueueAutomationJobRequest, ErrorResponse, HealthResponse, JobResponse,
    RegisterAddonRequest, SourceProbeResponse, UpsertAutomationProviderRequest,
    UpsertWebhookEndpointRequest,
};
use taru_core::{
    AddonId, AddonStatus, AutomationProviderId, EventId, GenreId, JobId, LibraryId, MediaItemId,
    MediaSourceId, PageRequest, PersonId, TagId, TaruError, WebhookEndpointId,
};
use tracing::{error, instrument, warn};

use crate::app::TaruApp;

mod playback;

use playback::{
    get_playback_session, get_source_playback_decision, head_stream_source, hls_playlist_source,
    hls_segment, remux_stream_source, stream_source,
};

pub fn build_router(app: TaruApp) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/libraries", get(list_libraries))
        .route("/libraries/{library_id}/scan", post(scan_library))
        .route("/libraries/{library_id}/nfo/import", post(import_nfo))
        .route("/libraries/{library_id}/nfo/export", post(export_nfo))
        .route("/libraries/{library_id}/sources", get(list_library_sources))
        .route("/items", get(list_items))
        .route("/items/{item_id}", get(get_item))
        .route("/items/{item_id}/credits", get(list_item_credits))
        .route("/items/{item_id}/images", get(list_item_images))
        .route("/people", get(list_people))
        .route("/people/{person_id}", get(get_person))
        .route("/people/{person_id}/items", get(list_person_items))
        .route("/tags", get(list_tags))
        .route("/tags/{tag_id}/items", get(list_tag_items))
        .route("/genres", get(list_genres))
        .route("/genres/{genre_id}/items", get(list_genre_items))
        .route("/search", get(search_items))
        .route(
            "/items/{item_id}/metadata/refresh",
            post(refresh_item_metadata),
        )
        .route("/sources/{source_id}/probe", get(get_source_probe))
        .route(
            "/sources/{source_id}/playback/decision",
            get(get_source_playback_decision),
        )
        .route(
            "/sources/{source_id}/stream",
            get(stream_source).head(head_stream_source),
        )
        .route(
            "/sources/{source_id}/stream/remux",
            get(remux_stream_source),
        )
        .route(
            "/sources/{source_id}/stream/hls/playlist.m3u8",
            get(hls_playlist_source),
        )
        .route("/playback/sessions/{session_id}", get(get_playback_session))
        .route(
            "/playback/sessions/{session_id}/hls/segments/{segment_name}",
            get(hls_segment),
        )
        .route(
            "/webhooks/endpoints",
            get(list_webhook_endpoints).post(upsert_webhook_endpoint),
        )
        .route(
            "/webhooks/endpoints/{endpoint_id}",
            get(get_webhook_endpoint),
        )
        .route(
            "/events/{event_id}/webhook-attempts",
            get(list_webhook_delivery_attempts),
        )
        .route(
            "/events/{event_id}/webhooks/deliver",
            post(deliver_webhooks_for_event),
        )
        .route(
            "/automation/providers",
            get(list_automation_providers).post(upsert_automation_provider),
        )
        .route(
            "/automation/providers/{provider_id}",
            get(get_automation_provider),
        )
        .route("/addons", get(list_addons).post(register_addon))
        .route("/addons/{addon_id}", get(get_addon))
        .route("/automation/jobs", post(enqueue_automation_job))
        .route(
            "/automation/jobs/{job_id}/artifacts",
            get(list_automation_job_artifacts),
        )
        .route(
            "/items/{item_id}/automation/artifacts",
            get(list_item_automation_artifacts),
        )
        .route("/jobs/{job_id}", get(get_job))
        .with_state(app)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        version: API_VERSION.to_owned(),
    })
}

#[instrument(skip(app))]
async fn list_libraries(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_libraries(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn scan_library(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_library_scan(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn import_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_nfo_import(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn export_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_nfo_export(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn list_library_sources(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_library_sources(library_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
async fn list_items(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_items(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn get_item(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_item(item_id).await?))
}

#[instrument(skip(app))]
async fn list_item_credits(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_item_credits(item_id).await?))
}

#[instrument(skip(app))]
async fn list_item_images(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_item_images(item_id).await?))
}

#[instrument(skip(app))]
async fn list_people(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_people(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn get_person(
    State(app): State<TaruApp>,
    Path(person_id): Path<PersonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_person(person_id).await?))
}

#[instrument(skip(app))]
async fn list_person_items(
    State(app): State<TaruApp>,
    Path(person_id): Path<PersonId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_person_items(person_id, page.try_into()?).await?,
    ))
}

#[instrument(skip(app))]
async fn list_tags(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_tags(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn list_tag_items(
    State(app): State<TaruApp>,
    Path(tag_id): Path<TagId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_tag_items(tag_id, page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn list_genres(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_genres(page.try_into()?).await?))
}

#[instrument(skip(app))]
async fn list_genre_items(
    State(app): State<TaruApp>,
    Path(genre_id): Path<GenreId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_genre_items(genre_id, page.try_into()?).await?,
    ))
}

#[instrument(skip(app))]
async fn search_items(
    State(app): State<TaruApp>,
    Query(query): Query<SearchPageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = query.page.try_into()?;
    let facets = query
        .facet
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|facet| !facet.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Ok(Json(app.search_items(query.q, facets, page).await?))
}

#[instrument(skip(app))]
async fn refresh_item_metadata(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_metadata_refresh(item_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn get_source_probe(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
) -> ApiResult<Json<SourceProbeResponse>> {
    Ok(Json(app.get_source_probe(source_id).await?))
}

#[instrument(skip(app))]
async fn get_job(
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<Json<JobResponse>> {
    Ok(Json(JobResponse::from_job(app.get_job(job_id).await?)))
}

#[instrument(skip(app))]
async fn upsert_webhook_endpoint(
    State(app): State<TaruApp>,
    Json(request): Json<UpsertWebhookEndpointRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.upsert_webhook_endpoint(request).await?))
}

#[instrument(skip(app))]
async fn list_webhook_endpoints(State(app): State<TaruApp>) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_enabled_webhook_endpoints().await?))
}

#[instrument(skip(app))]
async fn get_webhook_endpoint(
    State(app): State<TaruApp>,
    Path(endpoint_id): Path<WebhookEndpointId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_webhook_endpoint(endpoint_id).await?))
}

#[instrument(skip(app))]
async fn list_webhook_delivery_attempts(
    State(app): State<TaruApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_webhook_delivery_attempts(event_id).await?))
}

#[instrument(skip(app))]
async fn deliver_webhooks_for_event(
    State(app): State<TaruApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.deliver_webhooks_for_event(event_id).await?))
}

#[instrument(skip(app))]
async fn upsert_automation_provider(
    State(app): State<TaruApp>,
    Json(request): Json<UpsertAutomationProviderRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.upsert_automation_provider(request).await?))
}

#[instrument(skip(app))]
async fn list_automation_providers(State(app): State<TaruApp>) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_enabled_automation_providers().await?))
}

#[instrument(skip(app))]
async fn get_automation_provider(
    State(app): State<TaruApp>,
    Path(provider_id): Path<AutomationProviderId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_automation_provider(provider_id).await?))
}

#[instrument(skip(app))]
async fn register_addon(
    State(app): State<TaruApp>,
    Json(request): Json<RegisterAddonRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.register_addon(request).await?))
}

#[instrument(skip(app))]
async fn list_addons(
    State(app): State<TaruApp>,
    Query(query): Query<AddonListQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_addon_registrations(query.status).await?))
}

#[instrument(skip(app))]
async fn get_addon(
    State(app): State<TaruApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_addon_registration(addon_id).await?))
}

#[instrument(skip(app))]
async fn enqueue_automation_job(
    State(app): State<TaruApp>,
    Json(request): Json<EnqueueAutomationJobRequest>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_automation_job(request).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
async fn list_automation_job_artifacts(
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_automation_artifacts_for_job(job_id).await?))
}

#[instrument(skip(app))]
async fn list_item_automation_artifacts(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_automation_artifacts_for_item(item_id, page.try_into()?)
            .await?,
    ))
}

type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
struct PageQuery {
    limit: Option<u32>,
    offset: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct SearchPageQuery {
    #[serde(default)]
    q: String,
    facet: Option<String>,
    #[serde(flatten)]
    page: PageQuery,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
struct AddonListQuery {
    status: Option<AddonStatus>,
}

impl TryFrom<PageQuery> for PageRequest {
    type Error = TaruError;

    fn try_from(value: PageQuery) -> Result<Self, Self::Error> {
        let limit = value.limit.unwrap_or(PageRequest::DEFAULT_LIMIT);

        if limit > PageRequest::MAX_LIMIT {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "limit must be less than or equal to {}",
                    PageRequest::MAX_LIMIT
                ),
            });
        }

        Ok(PageRequest {
            limit,
            offset: value.offset.unwrap_or_default(),
        }
        .clamped())
    }
}

#[derive(Debug)]
struct ApiError(TaruError);

impl From<TaruError> for ApiError {
    fn from(value: TaruError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = status_for_error(&self.0);
        let body = ErrorResponse {
            code: code_for_error(&self.0).to_owned(),
            message: public_message(&self.0),
        };

        if status.is_server_error() {
            error!(error = %self.0, status = %status, "request failed");
        } else {
            warn!(error = %self.0, status = %status, "request rejected");
        }

        (status, Json(body)).into_response()
    }
}

fn status_for_error(error: &TaruError) -> StatusCode {
    match error {
        TaruError::Storage { message, .. } if is_staging_budget_exhausted(message) => {
            StatusCode::INSUFFICIENT_STORAGE
        }
        TaruError::Storage { message, .. } if is_storage_timeout(message) => {
            StatusCode::GATEWAY_TIMEOUT
        }
        TaruError::Storage { message, .. } if is_storage_rate_limited(message) => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        TaruError::InvalidInput { .. } | TaruError::Unsupported(_) => StatusCode::BAD_REQUEST,
        TaruError::NotFound { .. } => StatusCode::NOT_FOUND,
        TaruError::Conflict { .. } => StatusCode::CONFLICT,
        TaruError::Provider { .. } | TaruError::Storage { .. } => StatusCode::BAD_GATEWAY,
        TaruError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn code_for_error(error: &TaruError) -> &'static str {
    match error {
        TaruError::InvalidInput { .. } => "invalid_input",
        TaruError::NotFound { .. } => "not_found",
        TaruError::Conflict { .. } => "conflict",
        TaruError::Unsupported(_) => "unsupported",
        TaruError::Provider { provider, .. } if is_ffmpeg_provider(provider) => "ffmpeg_error",
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { message, .. } if is_staging_budget_exhausted(message) => {
            "staging_budget_exhausted"
        }
        TaruError::Storage { message, .. } if is_staging_validation_mismatch(message) => {
            "staging_validation_mismatch"
        }
        TaruError::Storage { message, .. } if is_storage_timeout(message) => "storage_timeout",
        TaruError::Storage { message, .. } if is_storage_unauthorized(message) => {
            "storage_unauthorized"
        }
        TaruError::Storage { message, .. } if is_storage_rate_limited(message) => {
            "storage_rate_limited"
        }
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
    }
}

fn public_message(error: &TaruError) -> String {
    match error {
        TaruError::Database { .. } => "database operation failed".to_owned(),
        TaruError::Provider { provider, .. } if is_ffmpeg_provider(provider) => {
            "ffmpeg operation failed".to_owned()
        }
        TaruError::Provider { provider, .. } => {
            format!("external provider operation failed: {provider}")
        }
        TaruError::Storage { message, .. } if is_staging_budget_exhausted(message) => {
            "staging disk budget exhausted".to_owned()
        }
        TaruError::Storage { message, .. } if is_staging_validation_mismatch(message) => {
            "staged input validation failed".to_owned()
        }
        TaruError::Storage { message, .. } if is_storage_timeout(message) => {
            "storage backend timed out".to_owned()
        }
        TaruError::Storage { message, .. } if is_storage_unauthorized(message) => {
            "storage backend rejected credentials".to_owned()
        }
        TaruError::Storage { message, .. } if is_storage_rate_limited(message) => {
            "storage backend rate limited the request".to_owned()
        }
        TaruError::Storage { .. } => "storage operation failed".to_owned(),
        TaruError::InvalidInput { .. }
        | TaruError::NotFound { .. }
        | TaruError::Conflict { .. }
        | TaruError::Unsupported(_) => error.to_string(),
    }
}

fn is_ffmpeg_provider(provider: &str) -> bool {
    provider == "ffmpeg" || provider == "ffmpeg_remux" || provider == "ffmpeg_hls"
}

fn is_staging_budget_exhausted(message: &str) -> bool {
    message.contains("staging disk budget exhausted")
}

fn is_staging_validation_mismatch(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("staged") && message.contains("did not match")
        || message.contains("staging validation")
}

fn is_storage_timeout(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("timed out")
        || message.contains("timeout")
        || message.contains("request timeout")
        || message.contains("408")
}

fn is_storage_unauthorized(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("unauthorized")
        || message.contains("forbidden")
        || message.contains("401")
        || message.contains("403")
}

fn is_storage_rate_limited(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("too many requests")
        || message.contains("rate limit")
        || message.contains("429")
}

#[cfg(test)]
mod tests;
