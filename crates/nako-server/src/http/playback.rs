use std::{io::SeekFrom, path::Path as FsPath};

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use nako_api::public_client::{
    BrowserPlaybackCapabilitiesDto, BrowserPlaybackMode, BrowserPlaybackOutputContainer,
    BrowserPlaybackTicketRequest, BrowserPlaybackTicketResponse, BrowserPlaybackUrlDto,
    BrowserPlaybackUrlKind, PLAYBACK_SESSION_ID_HEADER, TranscodeSessionResponse,
    timestamp_ms_to_rfc3339, transcode_session_response_from_record,
};
use nako_core::AuthenticatedPrincipal;
use nako_core::{MediaSourceId, NakoError, TranscodeSessionId};
use nako_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan,
    DirectPlayResponseStatus, content_type_for_file_name, parse_http_range_header,
    plan_direct_play_response,
};
use nako_transcode::RemuxContainer;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::instrument;

use crate::app::{
    BrowserPlaybackTicketMode, DirectPlaySourceBody, HlsSourceRequest, IssuedBrowserPlaybackTicket,
    NakoApp, RemuxSourceDisposition, RemuxSourceRequest,
};

use super::{
    access::{RequiredLibraryAccess, require_source_access},
    error::ApiResult,
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route(
            "/sources/{source_id}/playback/decision",
            get(get_source_playback_decision),
        )
        .route(
            "/sources/{source_id}/playback/browser-ticket",
            post(create_browser_playback_ticket),
        )
        .route(
            "/sources/{source_id}/stream",
            get(stream_source).head(head_stream_source),
        )
        .route(
            "/sources/{source_id}/stream/remux",
            get(remux_stream_source).head(head_remux_stream_source),
        )
        .route(
            "/sources/{source_id}/stream/hls/playlist.m3u8",
            get(hls_playlist_source),
        )
        .route("/playback/sessions/{session_id}", get(get_playback_session))
        .route(
            "/playback/sessions/{session_id}/cancel",
            post(cancel_playback_session),
        )
        .route(
            "/playback/sessions/{session_id}/hls/segments/{segment_name}",
            get(hls_segment),
        )
}

#[instrument(skip(app))]
pub(super) async fn get_source_playback_decision(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<PlaybackCapabilitiesQuery>,
) -> ApiResult<impl IntoResponse> {
    require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;

    Ok(Json(
        app.playback()
            .get_source_playback_decision(source_id, query.into())
            .await?,
    ))
}

#[instrument(skip(app, principal, request))]
pub(super) async fn create_browser_playback_ticket(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(source_id): Path<MediaSourceId>,
    Json(request): Json<BrowserPlaybackTicketRequest>,
) -> ApiResult<Json<BrowserPlaybackTicketResponse>> {
    let mode = browser_ticket_mode_from_public(&request.mode)?;

    require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
    let source = app
        .get_media_source_record(source_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "media_source",
            id: source_id.to_string(),
        })?;
    let issued = app.playback_tickets().issue_source_ticket(
        &principal,
        source_id,
        mode,
        crate::app::current_time_ms()?,
    )?;
    let url = browser_playback_url(
        &app,
        source_id,
        mode,
        request.capabilities.as_ref(),
        &issued,
    )
    .await?;

    Ok(Json(BrowserPlaybackTicketResponse {
        source_id: source_id.to_string(),
        item_id: Some(source.item_id.to_string()),
        mode: request.mode,
        expires_at: format_ticket_timestamp(issued.expires_at_ms),
        urls: vec![url],
    }))
}

#[instrument(skip(app, principal, ticket_query, headers))]
pub(super) async fn stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(ticket_query): Query<BrowserPlaybackTicketQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let _principal = resolve_source_playback_principal(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Direct,
        ticket_query.ticket.as_deref(),
    )
    .await?;

    let direct_play = app
        .playback()
        .plan_direct_play(source_id, direct_play_range_request(&headers))
        .await?;

    if direct_play.response.is_range_not_satisfiable() {
        return Ok(empty_direct_play_response(&direct_play.response));
    }

    let uri = direct_play.source.locator.clone();
    stream_direct_play_response(direct_play.body, &uri, &direct_play.response).await
}

#[instrument(skip(app, principal, ticket_query, headers))]
pub(super) async fn head_stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(ticket_query): Query<BrowserPlaybackTicketQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let _principal = resolve_source_playback_principal(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Direct,
        ticket_query.ticket.as_deref(),
    )
    .await?;

    let response = app
        .playback()
        .plan_direct_play_preflight(source_id, direct_play_range_request(&headers))
        .await?;

    Ok(empty_direct_play_response(&response))
}

#[instrument(skip(app, principal, query, headers))]
pub(super) async fn remux_stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<RemuxPlaybackQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let ticket = query.ticket.clone();
    let _principal = resolve_source_playback_principal(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Remux,
        ticket.as_deref(),
    )
    .await?;

    let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
    let remux = app
        .playback()
        .remux_source_or_wait(RemuxSourceRequest {
            source_id,
            client: query.capabilities().into(),
            output_container,
        })
        .await?;

    if remux.disposition == RemuxSourceDisposition::Cancelled {
        return Err(NakoError::Provider {
            provider: "ffmpeg_remux".to_owned(),
            message: "remux session was cancelled".to_owned(),
        }
        .into());
    }

    let total_len = tokio::fs::metadata(&remux.output_path)
        .await
        .map_err(|err| {
            NakoError::storage_io(
                remux.output_path.display().to_string(),
                format!("failed to read remux output length: {err}"),
            )
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

    let session_id = remux.session.as_ref().map(|session| session.id.to_string());
    let mut response = stream_local_file_response(
        &remux.output_path,
        &remux.output_path.display().to_string(),
        &response_plan,
    )
    .await?;
    if let Some(session_id) = session_id {
        response.headers_mut().insert(
            PLAYBACK_SESSION_ID_HEADER,
            HeaderValue::from_str(&session_id).expect("session id is a valid header value"),
        );
    }
    Ok(response)
}

#[instrument(skip(app, principal, query))]
pub(super) async fn head_remux_stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<RemuxPlaybackQuery>,
) -> ApiResult<Response> {
    let ticket = query.ticket.clone();
    let _principal = resolve_source_playback_principal(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Remux,
        ticket.as_deref(),
    )
    .await?;

    let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
    let remux = app
        .playback()
        .start_remux_source(RemuxSourceRequest {
            source_id,
            client: query.capabilities().into(),
            output_container,
        })
        .await?;

    let response_plan = plan_direct_play_response(
        0,
        content_type_for_file_name(&format!("stream.{}", output_container.file_extension())),
        DirectPlayRangeRequest::None,
    );
    let mut response = empty_direct_play_response(&response_plan);
    response.headers_mut().insert(
        PLAYBACK_SESSION_ID_HEADER,
        HeaderValue::from_str(&remux.session.id.to_string())
            .expect("session id is a valid header value"),
    );
    Ok(response)
}

#[instrument(skip(app, principal, query))]
pub(super) async fn hls_playlist_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<HlsPlaybackQuery>,
) -> ApiResult<Response> {
    let ticket = query.ticket.clone();
    let _principal = resolve_source_playback_principal(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Hls,
        ticket.as_deref(),
    )
    .await?;

    let playlist = app
        .playback()
        .hls_playlist(HlsSourceRequest {
            source_id,
            client: query.capabilities().into(),
        })
        .await?;
    let body = if let Some(ticket) = ticket {
        append_ticket_to_hls_playlist_segments(&playlist.body, &ticket)
    } else {
        playlist.body
    };

    Ok(hls_playlist_response(
        body,
        Some(playlist.session.id.to_string()),
    ))
}

#[instrument(skip(app, principal, ticket_query))]
pub(super) async fn hls_segment(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path((session_id, segment_name)): Path<(TranscodeSessionId, String)>,
    Query(ticket_query): Query<BrowserPlaybackTicketQuery>,
) -> ApiResult<Response> {
    let session = app.playback().get_transcode_session(session_id).await?;
    let _principal = resolve_source_playback_principal(
        &app,
        principal,
        session.source_id,
        BrowserPlaybackTicketMode::Hls,
        ticket_query.ticket.as_deref(),
    )
    .await?;

    let segment = app
        .playback()
        .plan_hls_segment(session_id, &segment_name)
        .await?;
    let total_len = tokio::fs::metadata(&segment.path)
        .await
        .map_err(|err| {
            NakoError::storage_io(
                segment.path.display().to_string(),
                format!("failed to read hls segment length: {err}"),
            )
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
pub(super) async fn get_playback_session(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(session_id): Path<TranscodeSessionId>,
) -> ApiResult<Json<TranscodeSessionResponse>> {
    let session = app.playback().get_transcode_session(session_id).await?;
    require_source_access(
        &app,
        &principal,
        session.source_id,
        RequiredLibraryAccess::Play,
    )
    .await?;

    Ok(Json(transcode_session_response_from_record(session)))
}

#[instrument(skip(app))]
pub(super) async fn cancel_playback_session(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(session_id): Path<TranscodeSessionId>,
) -> ApiResult<Json<TranscodeSessionResponse>> {
    let session = app.playback().get_transcode_session(session_id).await?;
    require_source_access(
        &app,
        &principal,
        session.source_id,
        RequiredLibraryAccess::Play,
    )
    .await?;

    Ok(Json(transcode_session_response_from_record(
        app.playback().cancel_transcode_session(session_id).await?,
    )))
}

fn browser_ticket_mode_from_public(
    mode: &BrowserPlaybackMode,
) -> ApiResult<BrowserPlaybackTicketMode> {
    match mode {
        BrowserPlaybackMode::Direct => Ok(BrowserPlaybackTicketMode::Direct),
        BrowserPlaybackMode::Remux => Ok(BrowserPlaybackTicketMode::Remux),
        BrowserPlaybackMode::Hls => Ok(BrowserPlaybackTicketMode::Hls),
        BrowserPlaybackMode::Other(_) => Err(NakoError::InvalidInput {
            message: "unsupported browser playback mode".to_owned(),
        }
        .into()),
    }
}

async fn browser_playback_url(
    app: &NakoApp,
    source_id: MediaSourceId,
    mode: BrowserPlaybackTicketMode,
    capabilities: Option<&BrowserPlaybackCapabilitiesDto>,
    issued: &IssuedBrowserPlaybackTicket,
) -> ApiResult<BrowserPlaybackUrlDto> {
    match mode {
        BrowserPlaybackTicketMode::Direct => {
            let response = app
                .playback()
                .plan_direct_play_preflight(source_id, DirectPlayRangeRequest::None)
                .await?;
            Ok(BrowserPlaybackUrlDto {
                kind: BrowserPlaybackUrlKind::Stream,
                url: format!("/sources/{source_id}/stream?ticket={}", issued.token),
                content_type: response.content_type,
                supports_range_requests: true,
            })
        }
        BrowserPlaybackTicketMode::Remux => {
            let output_container = requested_remux_container(capabilities)?;
            Ok(BrowserPlaybackUrlDto {
                kind: BrowserPlaybackUrlKind::Stream,
                url: format!(
                    "/sources/{source_id}/stream/remux?output_container={}&ticket={}",
                    output_container.file_extension(),
                    issued.token
                ),
                content_type: content_type_for_file_name(&format!(
                    "stream.{}",
                    output_container.file_extension()
                ))
                .to_owned(),
                supports_range_requests: true,
            })
        }
        BrowserPlaybackTicketMode::Hls => Ok(BrowserPlaybackUrlDto {
            kind: BrowserPlaybackUrlKind::Playlist,
            url: format!(
                "/sources/{source_id}/stream/hls/playlist.m3u8?ticket={}",
                issued.token
            ),
            content_type: "application/vnd.apple.mpegurl".to_owned(),
            supports_range_requests: false,
        }),
    }
}

fn requested_remux_container(
    capabilities: Option<&BrowserPlaybackCapabilitiesDto>,
) -> ApiResult<RemuxContainer> {
    match capabilities.and_then(|capabilities| capabilities.output_container.as_ref()) {
        Some(BrowserPlaybackOutputContainer::Mp4) | None => Ok(RemuxContainer::Mp4),
        Some(BrowserPlaybackOutputContainer::Mkv) => Ok(RemuxContainer::Mkv),
        Some(BrowserPlaybackOutputContainer::Other(_)) => Err(NakoError::InvalidInput {
            message: "unsupported browser playback output container".to_owned(),
        }
        .into()),
    }
}

async fn resolve_source_playback_principal(
    app: &NakoApp,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    source_id: MediaSourceId,
    mode: BrowserPlaybackTicketMode,
    ticket: Option<&str>,
) -> ApiResult<AuthenticatedPrincipal> {
    if let Some(ticket) = ticket {
        if ticket.trim().is_empty() {
            return Err(invalid_browser_playback_ticket().into());
        }
        let principal = app.playback_tickets().validate_source_ticket(
            ticket,
            source_id,
            mode,
            crate::app::current_time_ms()?,
        )?;
        require_source_access(app, &principal, source_id, RequiredLibraryAccess::Play).await?;
        return Ok(principal);
    }

    if let Some(Extension(principal)) = principal {
        require_source_access(app, &principal, source_id, RequiredLibraryAccess::Play).await?;
        return Ok(principal);
    }

    Err(NakoError::Unauthorized {
        message: "authentication required".to_owned(),
    }
    .into())
}

fn invalid_browser_playback_ticket() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid browser playback ticket".to_owned(),
    }
}

fn format_ticket_timestamp(timestamp_ms: i64) -> String {
    timestamp_ms_to_rfc3339(Some(timestamp_ms)).unwrap_or_else(|| timestamp_ms.to_string())
}

fn append_ticket_to_hls_playlist_segments(body: &str, ticket: &str) -> String {
    body.lines()
        .map(|line| {
            if line.starts_with("/playback/sessions/") {
                let separator = if line.contains('?') { '&' } else { '?' };
                format!("{line}{separator}ticket={ticket}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

fn hls_playlist_response(body: String, session_id: Option<String>) -> Response {
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
    if let Some(session_id) = session_id {
        headers.insert(
            PLAYBACK_SESSION_ID_HEADER,
            HeaderValue::from_str(&session_id).expect("session id is a valid header value"),
        );
    }
    response
}

pub(crate) async fn stream_local_file_response(
    path: &FsPath,
    uri: &str,
    plan: &DirectPlayResponsePlan,
) -> ApiResult<Response> {
    let mut file = tokio::fs::File::open(path).await.map_err(|err| {
        NakoError::storage_io(
            uri.to_owned(),
            format!("failed to open stream source: {err}"),
        )
    })?;

    if plan.seek_offset > 0 {
        file.seek(SeekFrom::Start(plan.seek_offset))
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    uri.to_owned(),
                    format!("failed to seek stream source: {err}"),
                )
            })?;
    }

    let stream = ReaderStream::new(file.take(plan.body_len));
    let mut response = Body::from_stream(stream).into_response();
    apply_direct_play_headers(&mut response, plan);

    Ok(response)
}

pub(crate) async fn stream_direct_play_response(
    body: DirectPlaySourceBody,
    uri: &str,
    plan: &DirectPlayResponsePlan,
) -> ApiResult<Response> {
    match body {
        DirectPlaySourceBody::LocalPath(path) => stream_local_file_response(&path, uri, plan).await,
        DirectPlaySourceBody::Stream(stream) => {
            let stream = stream.into_read_stream();
            let mut response = Body::from_stream(stream.body).into_response();
            apply_direct_play_headers(&mut response, plan);
            Ok(response)
        }
        DirectPlaySourceBody::Empty => Ok(empty_direct_play_response(plan)),
    }
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

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct PlaybackCapabilitiesQuery {
    direct_play: Option<bool>,
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RemuxPlaybackQuery {
    direct_play: Option<bool>,
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    output_container: Option<RemuxContainer>,
    ticket: Option<String>,
}

impl RemuxPlaybackQuery {
    fn capabilities(self) -> PlaybackCapabilitiesQuery {
        PlaybackCapabilitiesQuery {
            direct_play: self.direct_play,
            container: self.container,
            video_codec: self.video_codec,
            audio_codec: self.audio_codec,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct HlsPlaybackQuery {
    direct_play: Option<bool>,
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    ticket: Option<String>,
}

impl HlsPlaybackQuery {
    fn capabilities(self) -> PlaybackCapabilitiesQuery {
        PlaybackCapabilitiesQuery {
            direct_play: self.direct_play,
            container: self.container,
            video_codec: self.video_codec,
            audio_codec: self.audio_codec,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct BrowserPlaybackTicketQuery {
    ticket: Option<String>,
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
