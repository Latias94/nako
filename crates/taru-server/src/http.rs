use std::{io::SeekFrom, path::Path as FsPath};

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use taru_api::{
    API_VERSION, ErrorResponse, HealthResponse, JobResponse, SourceProbeResponse,
    TranscodeSessionResponse, UpsertWebhookEndpointRequest,
};
use taru_core::{
    EventId, GenreId, JobId, LibraryId, MediaItemId, MediaSourceId, PageRequest, PersonId, TagId,
    TaruError, TranscodeSessionId, WebhookEndpointId,
};
use taru_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan,
    DirectPlayResponseStatus, content_type_for_file_name, parse_http_range_header,
    plan_direct_play_response,
};
use taru_transcode::RemuxContainer;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::{error, instrument, warn};

use crate::app::{HlsSourceRequest, RemuxSourceDisposition, RemuxSourceRequest, TaruApp};

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
async fn get_source_playback_decision(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<PlaybackCapabilitiesQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.get_source_playback_decision(source_id, query.into())
            .await?,
    ))
}

#[instrument(skip(app, headers))]
async fn stream_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let direct_play = app
        .plan_direct_play(source_id, direct_play_range_request(&headers))
        .await?;

    if direct_play.response.is_range_not_satisfiable() {
        return Ok(empty_direct_play_response(&direct_play.response));
    }

    stream_local_file_response(
        &direct_play.local_path,
        &direct_play.source.locator,
        &direct_play.response,
    )
    .await
}

#[instrument(skip(app, headers))]
async fn head_stream_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let direct_play = app
        .plan_direct_play(source_id, direct_play_range_request(&headers))
        .await?;

    Ok(empty_direct_play_response(&direct_play.response))
}

#[instrument(skip(app, headers))]
async fn remux_stream_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<RemuxPlaybackQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
    let remux = app
        .remux_source(RemuxSourceRequest {
            source_id,
            client: query.capabilities.into(),
            output_container,
        })
        .await?;

    if remux.disposition == RemuxSourceDisposition::Cancelled {
        return Err(TaruError::Provider {
            provider: "ffmpeg_remux".to_owned(),
            message: "remux session was cancelled".to_owned(),
        }
        .into());
    }

    let total_len = tokio::fs::metadata(&remux.output_path)
        .await
        .map_err(|err| TaruError::Storage {
            uri: remux.output_path.display().to_string(),
            message: format!("failed to read remux output length: {err}"),
        })?
        .len();
    let response_plan = plan_direct_play_response(
        total_len,
        content_type_for_file_name(&format!("stream.{}", output_container.file_extension())),
        direct_play_range_request(&headers),
    );

    if response_plan.is_range_not_satisfiable() {
        return Ok(empty_direct_play_response(&response_plan));
    }

    stream_local_file_response(
        &remux.output_path,
        &remux.output_path.display().to_string(),
        &response_plan,
    )
    .await
}

#[instrument(skip(app))]
async fn hls_playlist_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<PlaybackCapabilitiesQuery>,
) -> ApiResult<Response> {
    let playlist = app
        .hls_playlist(HlsSourceRequest {
            source_id,
            client: query.into(),
        })
        .await?;

    Ok(hls_playlist_response(playlist.body))
}

#[instrument(skip(app))]
async fn hls_segment(
    State(app): State<TaruApp>,
    Path((session_id, segment_name)): Path<(TranscodeSessionId, String)>,
) -> ApiResult<Response> {
    let segment = app.plan_hls_segment(session_id, &segment_name).await?;
    let total_len = tokio::fs::metadata(&segment.path)
        .await
        .map_err(|err| TaruError::Storage {
            uri: segment.path.display().to_string(),
            message: format!("failed to read hls segment length: {err}"),
        })?
        .len();
    let response_plan = plan_direct_play_response(
        total_len,
        segment.content_type,
        DirectPlayRangeRequest::None,
    );

    stream_local_file_response(
        &segment.path,
        &segment.path.display().to_string(),
        &response_plan,
    )
    .await
}

#[instrument(skip(app))]
async fn get_job(
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<Json<JobResponse>> {
    Ok(Json(JobResponse::from_job(app.get_job(job_id).await?)))
}

#[instrument(skip(app))]
async fn get_playback_session(
    State(app): State<TaruApp>,
    Path(session_id): Path<TranscodeSessionId>,
) -> ApiResult<Json<TranscodeSessionResponse>> {
    Ok(Json(TranscodeSessionResponse::from_session(
        app.get_transcode_session(session_id).await?,
    )))
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

#[derive(Clone, Debug, Default, Deserialize)]
struct PlaybackCapabilitiesQuery {
    direct_play: Option<bool>,
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct RemuxPlaybackQuery {
    #[serde(flatten)]
    capabilities: PlaybackCapabilitiesQuery,
    output_container: Option<RemuxContainer>,
}

impl From<PlaybackCapabilitiesQuery> for ClientPlaybackCapabilities {
    fn from(value: PlaybackCapabilitiesQuery) -> Self {
        let defaults = ClientPlaybackCapabilities::default();

        Self {
            direct_play: value.direct_play.unwrap_or(defaults.direct_play),
            containers: csv_or_default(value.container, defaults.containers),
            video_codecs: csv_or_default(value.video_codec, defaults.video_codecs),
            audio_codecs: csv_or_default(value.audio_codec, defaults.audio_codecs),
        }
    }
}

fn csv_or_default(value: Option<String>, default: Vec<String>) -> Vec<String> {
    let Some(value) = value else {
        return default;
    };
    let values = value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if values.is_empty() { default } else { values }
}

fn direct_play_range_request(headers: &HeaderMap) -> DirectPlayRangeRequest {
    let Some(value) = headers.get(header::RANGE) else {
        return DirectPlayRangeRequest::None;
    };

    let Ok(value) = value.to_str() else {
        return DirectPlayRangeRequest::Invalid;
    };

    match parse_http_range_header(value) {
        Ok(range) => DirectPlayRangeRequest::Range(range),
        Err(_) => DirectPlayRangeRequest::Invalid,
    }
}

fn empty_direct_play_response(plan: &DirectPlayResponsePlan) -> Response {
    let mut response = Body::empty().into_response();
    apply_direct_play_headers(&mut response, plan);
    response
}

fn hls_playlist_response(body: String) -> Response {
    let body_len = body.len();
    let mut response = Body::from(body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.apple.mpegurl"),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&body_len.to_string()).expect("content length is a valid header"),
    );
    response
}

async fn stream_local_file_response(
    path: &FsPath,
    uri: &str,
    plan: &DirectPlayResponsePlan,
) -> ApiResult<Response> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|err| TaruError::Storage {
            uri: uri.to_owned(),
            message: format!("failed to open stream source: {err}"),
        })?;

    if plan.seek_offset > 0 {
        file.seek(SeekFrom::Start(plan.seek_offset))
            .await
            .map_err(|err| TaruError::Storage {
                uri: uri.to_owned(),
                message: format!("failed to seek stream source: {err}"),
            })?;
    }

    let stream = ReaderStream::new(file.take(plan.body_len));
    let mut response = Body::from_stream(stream).into_response();
    apply_direct_play_headers(&mut response, plan);

    Ok(response)
}

fn apply_direct_play_headers(response: &mut Response, plan: &DirectPlayResponsePlan) {
    *response.status_mut() = direct_play_status_code(plan.status);
    let headers = response.headers_mut();
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&plan.content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&plan.body_len.to_string())
            .expect("content length is a valid header"),
    );

    if let Some(content_range) = &plan.content_range {
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(content_range).expect("content range is a valid header"),
        );
    }
}

fn direct_play_status_code(status: DirectPlayResponseStatus) -> StatusCode {
    match status {
        DirectPlayResponseStatus::Ok => StatusCode::OK,
        DirectPlayResponseStatus::PartialContent => StatusCode::PARTIAL_CONTENT,
        DirectPlayResponseStatus::RangeNotSatisfiable => StatusCode::RANGE_NOT_SATISFIABLE,
    }
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
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
    }
}

fn public_message(error: &TaruError) -> String {
    match error {
        TaruError::Database { .. } => "database operation failed".to_owned(),
        TaruError::Provider { provider, .. } => {
            format!("external provider operation failed: {provider}")
        }
        TaruError::Storage { .. } => "storage operation failed".to_owned(),
        TaruError::InvalidInput { .. }
        | TaruError::NotFound { .. }
        | TaruError::Conflict { .. }
        | TaruError::Unsupported(_) => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path as FsPath, PathBuf},
        time::Duration,
    };

    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, header},
    };
    use serde::{Serialize, de::DeserializeOwned};
    use taru_api::{
        HealthResponse, JobResponse, LibraryListResponse, TranscodeSessionResponse,
        WebhookDeliveryAttemptsResponse, WebhookEndpointResponse, WebhookEndpointsResponse,
    };
    use taru_core::{
        CanonicalMetadata, CatalogRepository, CreditRole, DomainEventKind, DomainEventSubject,
        EventId, EventOutboxRepository, Genre, GenreId, ImageAsset, ImageAssetId, ImageKind,
        ImageOwner, ItemCredit, ItemGenre, ItemTag, JobId, JobKind, JobStatus, LibraryId,
        MediaItem, MediaKind, MediaProbeRepository, MediaProbeResult, MediaRepository, MediaSource,
        MediaSourceId, MediaStreamInfo, MediaStreamKind, MetadataSource, NewOutboxEvent,
        NewTranscodeSession, Person, PersonId, Tag, TagId, TranscodeSessionId,
        TranscodeSessionKind, TranscodeSessionRepository, TranscodeSessionState,
        WebhookEndpointStatus,
    };
    use taru_db::SqliteStore;
    use taru_search::{SearchDocument, SearchIndex};
    use taru_streaming::PlaybackMode;
    use tokio::time::sleep;
    use tower::ServiceExt;

    use super::*;
    use crate::config::{LocalLibraryConfig, MetadataConfig, TaruServerConfig, TranscodeConfig};

    #[tokio::test]
    async fn health_and_libraries_routes_work() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;

        let health = request_json::<HealthResponse>(&router, Method::GET, "/health").await;
        let libraries =
            request_json::<LibraryListResponse>(&router, Method::GET, "/libraries").await;

        assert_eq!(health.status, "ok");
        assert_eq!(libraries.libraries.len(), 1);
        assert_eq!(libraries.libraries[0].id, library_id);
    }

    #[tokio::test]
    async fn webhook_endpoint_routes_validate_and_list_enabled_endpoints() {
        let temp = tempfile::tempdir().unwrap();
        let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
        let response = request_body_json::<WebhookEndpointResponse, _>(
            &router,
            Method::POST,
            "/webhooks/endpoints",
            &UpsertWebhookEndpointRequest {
                id: None,
                name: "receiver".to_owned(),
                url: "https://example.test/taru-webhook".to_owned(),
                secret_env: Some("TARU_WEBHOOK_SECRET".to_owned()),
                subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
                timeout_ms: Some(5_000),
                max_attempts: Some(3),
                status: WebhookEndpointStatus::Enabled,
            },
        )
        .await;

        assert_eq!(response.endpoint.name, "receiver");
        assert_eq!(
            response.endpoint.secret_env,
            Some("TARU_WEBHOOK_SECRET".to_owned())
        );

        let list =
            request_json::<WebhookEndpointsResponse>(&router, Method::GET, "/webhooks/endpoints")
                .await;
        assert_eq!(list.endpoints, vec![response.endpoint.clone()]);

        let detail_path = format!("/webhooks/endpoints/{}", response.endpoint.id);
        let detail =
            request_json::<WebhookEndpointResponse>(&router, Method::GET, &detail_path).await;
        assert_eq!(detail, response);

        let invalid = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/webhooks/endpoints")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&UpsertWebhookEndpointRequest {
                            id: None,
                            name: "bad".to_owned(),
                            url: "file:///tmp/webhook".to_owned(),
                            secret_env: None,
                            subscribed_event_kinds: vec![
                                DomainEventKind::LibraryScanned.as_str().to_owned(),
                            ],
                            timeout_ms: Some(5_000),
                            max_attempts: Some(3),
                            status: WebhookEndpointStatus::Enabled,
                        })
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn webhook_attempt_route_lists_attempts_for_existing_event() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(
            TaruServerConfig {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                database_url: "sqlite::memory:".to_owned(),
                ffprobe_path: PathBuf::from("ffprobe"),
                ffmpeg_path: PathBuf::from("ffmpeg"),
                scan_concurrency: 1,
                probe_concurrency: 1,
                metadata_concurrency: 1,
                remux_concurrency: 1,
                webhook_concurrency: 2,
                remux_timeout_ms: 30 * 60 * 1_000,
                remux_staging_root: temp.path().join("taru-cache").join("remux"),
                metadata: MetadataConfig::default(),
                transcode: TranscodeConfig::default(),
                library: LocalLibraryConfig {
                    id: library_id,
                    name: "Movies".to_owned(),
                    root: temp.path().to_path_buf(),
                    preset: taru_core::LibraryPreset::Movies,
                },
            },
            store.clone(),
        )
        .await
        .unwrap();
        let event = store
            .enqueue_outbox_event(NewOutboxEvent {
                id: EventId::new(),
                kind: DomainEventKind::LibraryScanned,
                subject: DomainEventSubject::Library(library_id),
                library_id: Some(library_id),
                source_id: None,
                idempotency_key: format!("library.scanned:{library_id}"),
                payload_json: format!(r#"{{"library_id":"{library_id}"}}"#),
            })
            .await
            .unwrap();
        let router = build_router(app);
        let path = format!("/events/{}/webhook-attempts", event.id);

        let attempts =
            request_json::<WebhookDeliveryAttemptsResponse>(&router, Method::GET, &path).await;

        assert_eq!(attempts.event_id, event.id);
        assert!(attempts.attempts.is_empty());
    }

    #[tokio::test]
    async fn scan_route_queues_background_job() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;
        let path = format!("/libraries/{library_id}/scan");

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let job = body_json::<JobResponse>(response).await;
        assert_eq!(job.kind, taru_core::JobKind::LibraryScan);
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.library_id, Some(library_id));
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("library_id"))
                .and_then(serde_json::Value::as_str),
            Some(library_id.to_string().as_str())
        );

        let loaded_path = format!("/jobs/{}", job.id);
        let loaded_job = request_json::<JobResponse>(&router, Method::GET, &loaded_path).await;
        assert_eq!(loaded_job.id, job.id);
    }

    #[tokio::test]
    async fn nfo_routes_queue_background_jobs() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;

        let import_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/libraries/{library_id}/nfo/import"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let export_response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/libraries/{library_id}/nfo/export"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(import_response.status(), StatusCode::ACCEPTED);
        assert_eq!(export_response.status(), StatusCode::ACCEPTED);
        let import_job = body_json::<JobResponse>(import_response).await;
        let export_job = body_json::<JobResponse>(export_response).await;

        assert_eq!(import_job.kind, JobKind::NfoImport);
        assert_eq!(import_job.resource_class, "metadata.nfo.import");
        assert_eq!(import_job.library_id, Some(library_id));
        assert_eq!(
            import_job
                .input
                .as_ref()
                .and_then(|input| input.get("policy"))
                .and_then(serde_json::Value::as_str),
            Some("local_first")
        );
        assert_eq!(export_job.kind, JobKind::NfoExport);
        assert_eq!(export_job.resource_class, "metadata.nfo.export");
        assert_eq!(export_job.library_id, Some(library_id));
    }

    #[tokio::test]
    async fn missing_job_returns_404() {
        let temp = tempfile::tempdir().unwrap();
        let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
        let missing = JobId::new();
        let path = format!("/jobs/{missing}");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn metadata_refresh_route_queues_background_job() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata,
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        let router = build_router(app);
        let path = format!("/items/{}/metadata/refresh", item.id);

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let job = body_json::<JobResponse>(response).await;
        assert_eq!(job.kind, JobKind::MetadataRefresh);
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.library_id, Some(library_id));
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("item_id"))
                .and_then(serde_json::Value::as_str),
            Some(item.id.to_string().as_str())
        );
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("provider"))
                .and_then(serde_json::Value::as_str),
            Some("tmdb")
        );
        assert_eq!(
            job.input
                .as_ref()
                .and_then(|input| input.get("refresh_mode"))
                .and_then(serde_json::Value::as_str),
            Some("default")
        );
    }

    #[tokio::test]
    async fn empty_sources_and_items_routes_work() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;
        let sources_path = format!("/libraries/{library_id}/sources");

        let sources =
            request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path)
                .await;
        let items = request_json::<taru_api::ItemsResponse>(&router, Method::GET, "/items").await;

        assert_eq!(sources.library.id, library_id);
        assert_eq!(sources.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
        assert_eq!(sources.page.offset, 0);
        assert!(sources.sources.is_empty());
        assert_eq!(items.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
        assert!(items.items.is_empty());
    }

    #[tokio::test]
    async fn search_route_returns_indexed_items() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Search Route Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert(SearchDocument {
                item_id: item.id,
                title: item.metadata.title.clone(),
                body: "A route test fixture".to_owned(),
                facets: vec!["genre:test".to_owned()],
            })
            .await
            .unwrap();
        let router = build_router(app);

        let result = request_json::<taru_api::SearchResponse>(
            &router,
            Method::GET,
            "/search?q=route&facet=genre:test",
        )
        .await;

        assert_eq!(result.page.returned, 1);
        assert_eq!(result.hits[0].item.id, item.id);
    }

    #[tokio::test]
    async fn browse_routes_return_catalog_graph() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Browse Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///Browse Demo.mkv".to_owned(),
            file_name: "Browse Demo.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };
        let person = Person {
            id: PersonId::new(),
            name: "Demo Actor".to_owned(),
            sort_name: None,
            overview: None,
            external_ids: Vec::new(),
        };
        let genre = Genre {
            id: GenreId::new(),
            name: "Science Fiction".to_owned(),
            source: MetadataSource::Nfo,
        };
        let tag = Tag {
            id: TagId::new(),
            name: "favorite".to_owned(),
            source: MetadataSource::User,
        };
        let image = ImageAsset {
            id: ImageAssetId::new(),
            owner: ImageOwner::Item(item.id),
            kind: ImageKind::Poster,
            source_uri: "local:///poster.jpg".to_owned(),
            provider: taru_core::ExternalProvider::Local,
            cache_uri: None,
            width: None,
            height: None,
            language: None,
            selected: true,
            content_hash: None,
            etag: None,
        };

        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        store.upsert_person(&person).await.unwrap();
        store
            .upsert_item_credit(&ItemCredit {
                item_id: item.id,
                person_id: person.id,
                role: CreditRole::Actor,
                character: Some("Lead".to_owned()),
                sort_order: Some(0),
            })
            .await
            .unwrap();
        store.upsert_genre(&genre).await.unwrap();
        store
            .upsert_item_genre(&ItemGenre {
                item_id: item.id,
                genre_id: genre.id,
            })
            .await
            .unwrap();
        store.upsert_tag(&tag).await.unwrap();
        store
            .upsert_item_tag(&ItemTag {
                item_id: item.id,
                tag_id: tag.id,
            })
            .await
            .unwrap();
        store.upsert_image_asset(&image).await.unwrap();
        let router = build_router(app);

        let detail = request_json::<taru_api::ItemDetailResponse>(
            &router,
            Method::GET,
            &format!("/items/{}", item.id),
        )
        .await;
        let credits = request_json::<taru_api::ItemCreditsResponse>(
            &router,
            Method::GET,
            &format!("/items/{}/credits", item.id),
        )
        .await;
        let images = request_json::<taru_api::ImagesResponse>(
            &router,
            Method::GET,
            &format!("/items/{}/images", item.id),
        )
        .await;
        let people =
            request_json::<taru_api::PeopleResponse>(&router, Method::GET, "/people").await;
        let person_items = request_json::<taru_api::PersonItemsResponse>(
            &router,
            Method::GET,
            &format!("/people/{}/items", person.id),
        )
        .await;
        let tags = request_json::<taru_api::TagsResponse>(&router, Method::GET, "/tags").await;
        let tag_items = request_json::<taru_api::TagItemsResponse>(
            &router,
            Method::GET,
            &format!("/tags/{}/items", tag.id),
        )
        .await;
        let genres =
            request_json::<taru_api::GenreListResponse>(&router, Method::GET, "/genres").await;
        let genre_items = request_json::<taru_api::GenreItemsResponse>(
            &router,
            Method::GET,
            &format!("/genres/{}/items", genre.id),
        )
        .await;

        assert_eq!(detail.item.id, item.id);
        assert_eq!(detail.sources[0].id, source.id);
        assert_eq!(detail.credits.len(), 1);
        assert_eq!(credits.people[0].name, "Demo Actor");
        assert_eq!(images.images[0].id, image.id);
        assert_eq!(people.people[0].id, person.id);
        assert_eq!(person_items.items[0].id, item.id);
        assert_eq!(tags.tags[0].name, "favorite");
        assert_eq!(tag_items.items[0].id, item.id);
        assert_eq!(genres.genres[0].name, "Science Fiction");
        assert_eq!(genre_items.items[0].id, item.id);
    }

    #[tokio::test]
    async fn playback_decision_and_direct_stream_routes_work() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("demo.mp4"), b"0123456789").unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///demo.mp4".to_owned(),
            file_name: "demo.mp4".to_owned(),
            size_bytes: Some(10),
            fingerprint: None,
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        store
            .upsert_media_probe(
                source.id,
                &MediaProbeResult {
                    duration_ms: Some(1_000),
                    container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
                    bit_rate: None,
                    streams: vec![
                        MediaStreamInfo {
                            index: 0,
                            kind: MediaStreamKind::Video,
                            codec: Some("h264".to_owned()),
                            language: None,
                            duration_ms: None,
                            bit_rate: None,
                            width: Some(1920),
                            height: Some(1080),
                            channels: None,
                            sample_rate: None,
                        },
                        MediaStreamInfo {
                            index: 1,
                            kind: MediaStreamKind::Audio,
                            codec: Some("aac".to_owned()),
                            language: None,
                            duration_ms: None,
                            bit_rate: None,
                            width: None,
                            height: None,
                            channels: Some(2),
                            sample_rate: Some(48_000),
                        },
                    ],
                },
            )
            .await
            .unwrap();
        let router = build_router(app);

        let decision = request_json::<taru_api::PlaybackDecisionResponse>(
            &router,
            Method::GET,
            &format!("/sources/{}/playback/decision", source.id),
        )
        .await;
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/sources/{}/stream", source.id))
                    .header(header::RANGE, "bytes=2-5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(decision.decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_RANGE)
                .and_then(|value| value.to_str().ok()),
            Some("bytes 2-5/10")
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&bytes[..], b"2345");
    }

    #[tokio::test]
    async fn direct_stream_head_returns_headers_without_body() {
        let (_temp, router, source) = router_with_media_source("demo.mp4", b"0123456789").await;

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::HEAD)
                    .uri(format!("/sources/{}/stream", source.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok()),
            Some("10")
        );
        assert_eq!(
            response
                .headers()
                .get(header::ACCEPT_RANGES)
                .and_then(|value| value.to_str().ok()),
            Some("bytes")
        );
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("video/mp4")
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(bytes.is_empty());
    }

    #[tokio::test]
    async fn direct_stream_zero_byte_file_returns_empty_ok() {
        let (_temp, router, source) = router_with_media_source("empty.mp4", b"").await;

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/sources/{}/stream", source.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|value| value.to_str().ok()),
            Some("0")
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(bytes.is_empty());
    }

    #[tokio::test]
    async fn direct_stream_rejects_unsatisfiable_and_multi_ranges() {
        let (_temp, router, source) = router_with_media_source("demo.mp4", b"0123456789").await;

        for range in ["bytes=20-30", "bytes=0-1,2-3"] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri(format!("/sources/{}/stream", source.id))
                        .header(header::RANGE, range)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
            assert_eq!(
                response
                    .headers()
                    .get(header::CONTENT_RANGE)
                    .and_then(|value| value.to_str().ok()),
                Some("bytes */10")
            );
            assert_eq!(
                response
                    .headers()
                    .get(header::CONTENT_LENGTH)
                    .and_then(|value| value.to_str().ok()),
                Some("0")
            );
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert!(bytes.is_empty());
        }
    }

    #[tokio::test]
    async fn remux_stream_route_runs_and_reuses_completed_output() {
        let (_temp, router, source, _staging_root, ffmpeg_path, _marker, _store) =
            router_with_remux_source(false).await;
        let path = format!("/sources/{}/stream/remux?output_container=mp4", source.id);

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&path)
                    .header(header::RANGE, "bytes=1-4")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("video/mp4")
        );
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_RANGE)
                .and_then(|value| value.to_str().ok()),
            Some("bytes 1-4/7")
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&bytes[..], b"emux");

        fs::remove_file(ffmpeg_path).unwrap();

        let reused = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(reused.status(), StatusCode::OK);
        let bytes = to_bytes(reused.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&bytes[..], b"remuxed");
    }

    #[tokio::test]
    async fn playback_session_route_returns_remux_session_state() {
        let (_temp, router, source, _staging_root, _ffmpeg_path, _marker, store) =
            router_with_remux_source(false).await;
        let remux_path = format!("/sources/{}/stream/remux?output_container=mp4", source.id);

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&remux_path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let session = store
            .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, "remux:mp4")
            .await
            .unwrap()
            .unwrap();
        let session_response = request_json::<TranscodeSessionResponse>(
            &router,
            Method::GET,
            &format!("/playback/sessions/{}", session.id),
        )
        .await;

        assert_eq!(session_response.session.id, session.id);
        assert_eq!(
            session_response.session.state,
            TranscodeSessionState::Finished
        );
    }

    #[tokio::test]
    async fn hls_playlist_and_segment_routes_work() {
        let (_temp, router, source, store) = router_with_hls_source().await;
        let playlist_path = format!("/sources/{}/stream/hls/playlist.m3u8", source.id);

        let playlist_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&playlist_path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(playlist_response.status(), StatusCode::OK);
        assert_eq!(
            playlist_response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/vnd.apple.mpegurl")
        );

        let session = store
            .find_latest_transcode_session(
                source.id,
                TranscodeSessionKind::HlsTranscode,
                "hls:single",
            )
            .await
            .unwrap()
            .unwrap();
        let playlist = String::from_utf8(
            to_bytes(playlist_response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let segment_path = format!(
            "/playback/sessions/{}/hls/segments/segment_00000.ts",
            session.id
        );

        assert!(playlist.contains(&segment_path));

        let segment_response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&segment_path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(segment_response.status(), StatusCode::OK);
        assert_eq!(
            segment_response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("video/mp2t")
        );
        let segment = to_bytes(segment_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&segment[..], b"segment");

        let missing = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/playback/sessions/{}/hls/segments/missing.ts",
                        session.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn hls_segment_route_rejects_unfinished_session() {
        let (temp, router, source, store) = router_with_hls_source().await;
        let active = store
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: source.id,
                kind: TranscodeSessionKind::HlsTranscode,
                request_key: "hls:single".to_owned(),
                output_path: temp.path().join("active.m3u8"),
                state: TranscodeSessionState::Running,
            })
            .await
            .unwrap();
        let path = format!(
            "/playback/sessions/{}/hls/segments/segment_00000.ts",
            active.id
        );

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let error = body_json::<ErrorResponse>(response).await;
        assert_eq!(error.code, "conflict");
        assert!(error.message.contains("is not ready"));
    }

    #[tokio::test]
    async fn remux_stream_route_maps_in_flight_duplicate_to_conflict() {
        let (_temp, router, source, _staging_root, _ffmpeg_path, marker, _store) =
            router_with_remux_source(true).await;
        let path = format!("/sources/{}/stream/remux?output_container=mp4", source.id);
        let first_router = router.clone();
        let first_path = path.clone();
        let first = tokio::spawn(async move {
            first_router
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri(first_path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
        });

        for _ in 0..50 {
            if marker.exists() {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
        assert!(marker.exists());

        let duplicate = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(duplicate.status(), StatusCode::CONFLICT);
        let error = body_json::<ErrorResponse>(duplicate).await;
        assert_eq!(error.code, "conflict");
        assert!(error.message.contains("already in progress"));

        let first_response = first.await.unwrap();
        assert_eq!(first_response.status(), StatusCode::OK);
        let bytes = to_bytes(first_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&bytes[..], b"remuxed");
    }

    #[tokio::test]
    async fn missing_source_probe_returns_404() {
        let temp = tempfile::tempdir().unwrap();
        let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
        let missing = MediaSourceId::new();
        let path = format!("/sources/{missing}/probe");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn paginated_routes_echo_page_info_and_reject_large_limits() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let router = test_router(temp.path().to_path_buf(), library_id).await;
        let sources_path = format!("/libraries/{library_id}/sources?limit=10&offset=20");

        let sources =
            request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path)
                .await;
        assert_eq!(sources.page.limit, 10);
        assert_eq!(sources.page.offset, 20);

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/items?limit=501")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn router_with_media_source(
        file_name: &str,
        content: &[u8],
    ) -> (tempfile::TempDir, Router, MediaSource) {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(file_name), content).unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: file_name.to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: format!("local:///{file_name}"),
            file_name: file_name.to_owned(),
            size_bytes: Some(content.len() as u64),
            fingerprint: None,
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        let router = build_router(app);

        (temp, router, source)
    }

    async fn router_with_remux_source(
        slow: bool,
    ) -> (
        tempfile::TempDir,
        Router,
        MediaSource,
        PathBuf,
        PathBuf,
        PathBuf,
        SqliteStore,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("remux.started");
        let ffmpeg_path = fake_ffmpeg_script(temp.path(), "remux", slow, &marker);
        let library_root = temp.path().join("library");
        let staging_root = temp.path().join("cache").join("remux");
        fs::create_dir_all(&library_root).unwrap();
        fs::write(library_root.join("demo.mkv"), b"media").unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: ffmpeg_path.clone(),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: staging_root.clone(),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: library_root,
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        store
            .upsert_media_probe(source.id, &compatible_probe())
            .await
            .unwrap();
        let router = build_router(app);

        (
            temp,
            router,
            source,
            staging_root,
            ffmpeg_path,
            marker,
            store,
        )
    }

    async fn router_with_hls_source() -> (tempfile::TempDir, Router, MediaSource, SqliteStore) {
        let temp = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_hls_ffmpeg_script(temp.path(), "hls");
        let library_root = temp.path().join("library");
        let staging_root = temp.path().join("cache").join("remux");
        fs::create_dir_all(&library_root).unwrap();
        fs::write(library_root.join("demo.mkv"), b"media").unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path,
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: staging_root,
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: library_root,
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: taru_core::MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        store
            .upsert_media_probe(source.id, &compatible_probe())
            .await
            .unwrap();
        let router = build_router(app);

        (temp, router, source, store)
    }

    fn compatible_probe() -> MediaProbeResult {
        MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                MediaStreamInfo {
                    index: 0,
                    kind: MediaStreamKind::Video,
                    codec: Some("h264".to_owned()),
                    language: None,
                    duration_ms: None,
                    bit_rate: None,
                    width: Some(1920),
                    height: Some(1080),
                    channels: None,
                    sample_rate: None,
                },
                MediaStreamInfo {
                    index: 1,
                    kind: MediaStreamKind::Audio,
                    codec: Some("aac".to_owned()),
                    language: None,
                    duration_ms: None,
                    bit_rate: None,
                    width: None,
                    height: None,
                    channels: Some(2),
                    sample_rate: Some(48_000),
                },
            ],
        }
    }

    fn fake_ffmpeg_script(root: &FsPath, name: &str, slow: bool, marker: &FsPath) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let mut content = String::from("#!/bin/sh\n");
            content.push_str("for arg do out=\"$arg\"; done\n");
            if slow {
                content.push_str(&format!("printf started > \"{}\"\n", marker.display()));
            }
            content.push_str("printf remuxed > \"$out\"\n");
            if slow {
                content.push_str("sleep 1\n");
            }
            content.push_str("exit 0\n");
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal enabledelayedexpansion\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto run\r\n");
            content.push_str("set out=%~1\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":run\r\n");
            if slow {
                content.push_str(&format!(
                    "<nul set /p dummy=started>\"{}\"\r\n",
                    marker.display()
                ));
            }
            content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
            if slow {
                content.push_str("ping -n 3 127.0.0.1 > nul\r\n");
            }
            content.push_str("exit /b 0\r\n");
            fs::write(&path, content).unwrap();
            path
        }
    }

    fn fake_hls_ffmpeg_script(root: &FsPath, name: &str) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let mut content = String::from("#!/bin/sh\n");
            content.push_str("for arg do out=\"$arg\"; done\n");
            content.push_str("dir=$(dirname \"$out\")\n");
            content.push_str("mkdir -p \"$dir\"\n");
            content.push_str(
                "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
            );
            content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
            content.push_str("exit 0\n");
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal enabledelayedexpansion\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto run\r\n");
            content.push_str("set out=%~1\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":run\r\n");
            content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
            content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
            content.push_str(">\"%out%\" echo #EXTM3U\r\n");
            content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
            content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
            content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
            content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
            content.push_str("exit /b 0\r\n");
            fs::write(&path, content).unwrap();
            path
        }
    }

    async fn test_router(root: PathBuf, library_id: LibraryId) -> Router {
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 2,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: root.join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root,
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store).await.unwrap();
        build_router(app)
    }

    async fn request_json<T>(router: &Router, method: Method, uri: &str) -> T
    where
        T: DeserializeOwned,
    {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        body_json(response).await
    }

    async fn request_body_json<T, B>(router: &Router, method: Method, uri: &str, body: &B) -> T
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        body_json(response).await
    }

    async fn body_json<T>(response: axum::response::Response) -> T
    where
        T: DeserializeOwned,
    {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }
}
