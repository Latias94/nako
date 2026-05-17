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
use taru_api::{TranscodeSessionResponse, transcode_session_response_from_record};
use taru_core::{MediaSourceId, TaruError, TranscodeSessionId};
use taru_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan,
    DirectPlayResponseStatus, content_type_for_file_name, parse_http_range_header,
    plan_direct_play_response,
};
use taru_transcode::RemuxContainer;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::instrument;

use crate::app::{
    DirectPlaySourceBody, HlsSourceRequest, RemuxSourceDisposition, RemuxSourceRequest, TaruApp,
};

use super::error::ApiResult;

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
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
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<PlaybackCapabilitiesQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.playback()
            .get_source_playback_decision(source_id, query.into())
            .await?,
    ))
}

#[instrument(skip(app, headers))]
pub(super) async fn stream_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    headers: HeaderMap,
) -> ApiResult<Response> {
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

#[instrument(skip(app, headers))]
pub(super) async fn head_stream_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let response = app
        .playback()
        .plan_direct_play_preflight(source_id, direct_play_range_request(&headers))
        .await?;

    Ok(empty_direct_play_response(&response))
}

#[instrument(skip(app, headers))]
pub(super) async fn remux_stream_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<RemuxPlaybackQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
    let remux = app
        .playback()
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
pub(super) async fn hls_playlist_source(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<PlaybackCapabilitiesQuery>,
) -> ApiResult<Response> {
    let playlist = app
        .playback()
        .hls_playlist(HlsSourceRequest {
            source_id,
            client: query.into(),
        })
        .await?;

    Ok(hls_playlist_response(playlist.body))
}

#[instrument(skip(app))]
pub(super) async fn hls_segment(
    State(app): State<TaruApp>,
    Path((session_id, segment_name)): Path<(TranscodeSessionId, String)>,
) -> ApiResult<Response> {
    let segment = app
        .playback()
        .plan_hls_segment(session_id, &segment_name)
        .await?;
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
pub(super) async fn get_playback_session(
    State(app): State<TaruApp>,
    Path(session_id): Path<TranscodeSessionId>,
) -> ApiResult<Json<TranscodeSessionResponse>> {
    Ok(Json(transcode_session_response_from_record(
        app.playback().get_transcode_session(session_id).await?,
    )))
}

#[instrument(skip(app))]
pub(super) async fn cancel_playback_session(
    State(app): State<TaruApp>,
    Path(session_id): Path<TranscodeSessionId>,
) -> ApiResult<Json<TranscodeSessionResponse>> {
    Ok(Json(transcode_session_response_from_record(
        app.playback().cancel_transcode_session(session_id).await?,
    )))
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

pub(crate) async fn stream_local_file_response(
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
