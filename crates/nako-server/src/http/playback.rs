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
    BrowserPlaybackUrlKind, ClientHlsSegmentContainer, ClientHlsVariantPolicy,
    ClientPlaybackSessionState, PLAYBACK_SESSION_ID_HEADER,
    PlaybackSessionHeartbeatRequest as PublicPlaybackSessionHeartbeatRequest,
    PlaybackSessionResponse, playback_session_response_from_record, timestamp_ms_to_rfc3339,
};
use nako_core::AuthenticatedPrincipal;
use nako_core::{
    MediaSourceId, NakoError, PlaybackSessionId, PlaybackSessionMode, PlaybackSessionState,
    RendererSessionId,
};
use nako_playback::{
    ClientPlaybackCapabilities, PlaybackHlsSegmentContainer, PlaybackHlsVariantPolicy,
    PlaybackPreferenceContext, PlaybackTranscodeContainer,
};
use nako_streaming::{
    DirectPlayRangeRequest, DirectPlayResponsePlan, DirectPlayResponseStatus,
    content_type_for_file_name, parse_http_range_header,
};
use nako_transcode::{HlsPlaybackGeneration, RemuxContainer};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::instrument;

use crate::app::{
    BrowserPlaybackTicketMode, BrowserPlaybackTicketValidationRequest, DirectPlaySourceBody,
    DirectPlaybackPreflightRequest, DirectPlaybackSessionStreamRequest,
    DirectPlaybackStreamRequest, HlsPlaylistPlaybackRequest, HlsPlaylistSessionRequest,
    HlsSegmentPlaybackRequest, IssuedBrowserPlaybackTicket, NakoApp,
    PlaybackSessionHeartbeatRequest as AppPlaybackSessionHeartbeatRequest, PlaybackTraceContext,
    RemuxPlaybackPreflightRequest, RemuxPlaybackSessionStreamRequest, RemuxPlaybackStreamRequest,
    RendererTransportTicketScope, StartPlaybackSessionRequest, SubtitlePlaybackRequest,
    ValidateRendererTransportTicketRequest,
};

use super::{
    access::{RequiredLibraryAccess, has_library_access, require_source_access},
    error::ApiResult,
    trace_context::HttpTraceContext,
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
        .route(
            "/sources/{source_id}/subtitles/{stream_index}",
            get(subtitle_source),
        )
        .route("/playback/sessions/{session_id}", get(get_playback_session))
        .route(
            "/playback/sessions/{session_id}/cancel",
            post(cancel_playback_session),
        )
        .route(
            "/playback/sessions/{session_id}/heartbeat",
            post(heartbeat_playback_session),
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
    Ok(Json(
        app.playback()
            .get_source_playback_decision(&principal, source_id, query.into())
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

    let source = app
        .get_media_source_record(source_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "media_source",
            id: source_id.to_string(),
        })?;
    let remux_output_container = if matches!(mode, BrowserPlaybackTicketMode::Remux) {
        Some(requested_remux_container(request.capabilities.as_ref())?)
    } else {
        None
    };
    let mut client = browser_capabilities_to_client(request.capabilities.as_ref())?;
    if let Some(output_container) = remux_output_container {
        normalize_client_for_browser_remux_ticket(&mut client, output_container);
    }
    app.playback()
        .validate_browser_playback_ticket_request(BrowserPlaybackTicketValidationRequest {
            principal: principal.clone(),
            source_id,
            mode,
            subtitle_stream_index: request.subtitle_stream_index,
        })
        .await?;
    let now_ms = crate::app::current_time_ms()?;
    let playback_session = if let Some(session_mode) = playback_session_mode_from_browser(mode) {
        Some(
            app.playback()
                .start_playback_session(StartPlaybackSessionRequest {
                    principal_id: principal.principal_id.clone(),
                    source_id,
                    mode: session_mode,
                    client: Some(client),
                })
                .await?,
        )
    } else {
        None
    };
    let issued = if matches!(mode, BrowserPlaybackTicketMode::Subtitle) {
        app.playback_tickets().issue_subtitle_ticket(
            &principal,
            source_id,
            request
                .subtitle_stream_index
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "subtitle browser playback ticket requires subtitle_stream_index"
                        .to_owned(),
                })?,
            now_ms,
        )?
    } else {
        let playback_session_id = playback_session
            .as_ref()
            .expect("non-subtitle browser playback tickets allocate a playback session")
            .id;
        app.playback_tickets().issue_source_ticket(
            &principal,
            source_id,
            mode,
            playback_session_id,
            now_ms,
        )?
    };
    let url = browser_playback_url(
        &app,
        &principal,
        source_id,
        mode,
        request.subtitle_stream_index,
        request.capabilities.as_ref(),
        &issued,
    )
    .await?;

    Ok(Json(BrowserPlaybackTicketResponse {
        source_id: source_id.to_string(),
        item_id: Some(source.item_id.to_string()),
        playback_session_id: playback_session.map(|session| session.id.to_string()),
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
    if let Some(renderer_transport) = resolve_renderer_transport_principal(
        &app,
        source_id,
        PlaybackSessionMode::Direct,
        ticket_query.renderer_session_id.as_deref(),
        ticket_query.playback_session_id.as_deref(),
        ticket_query.renderer_ticket.as_deref(),
    )
    .await?
    {
        let direct_play = app
            .playback()
            .direct_playback_session_stream(DirectPlaybackSessionStreamRequest {
                principal: renderer_transport.principal,
                playback_session_id: renderer_transport.playback_session_id,
                source_id,
                range_request: direct_play_range_request(&headers),
            })
            .await?;

        if direct_play.response.is_range_not_satisfiable() {
            return Ok(empty_direct_play_response(&direct_play.response));
        }

        let mut response = stream_direct_play_response(
            direct_play.body,
            &direct_play.source_uri,
            &direct_play.response,
        )
        .await?;
        if let Some(session) = direct_play.session {
            insert_playback_session_header(&mut response, session.id);
        }

        return Ok(response);
    }

    let source_playback = resolve_source_playback_context(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Direct,
        ticket_query.ticket.as_deref(),
    )
    .await?;

    let direct_play = if let Some(playback_session_id) = source_playback.playback_session_id {
        app.playback()
            .direct_playback_session_stream(DirectPlaybackSessionStreamRequest {
                principal: source_playback.principal,
                playback_session_id,
                source_id,
                range_request: direct_play_range_request(&headers),
            })
            .await?
    } else {
        app.playback()
            .direct_playback_stream(DirectPlaybackStreamRequest {
                principal: source_playback.principal,
                source_id,
                range_request: direct_play_range_request(&headers),
                client: ClientPlaybackCapabilities::default(),
            })
            .await?
    };

    if direct_play.response.is_range_not_satisfiable() {
        return Ok(empty_direct_play_response(&direct_play.response));
    }

    let mut response = stream_direct_play_response(
        direct_play.body,
        &direct_play.source_uri,
        &direct_play.response,
    )
    .await?;
    if let Some(session) = direct_play.session {
        insert_playback_session_header(&mut response, session.id);
    }

    Ok(response)
}

#[instrument(skip(app, principal, ticket_query, headers))]
pub(super) async fn head_stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(ticket_query): Query<BrowserPlaybackTicketQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    if let Some(renderer_transport) = resolve_renderer_transport_principal(
        &app,
        source_id,
        PlaybackSessionMode::Direct,
        ticket_query.renderer_session_id.as_deref(),
        ticket_query.playback_session_id.as_deref(),
        ticket_query.renderer_ticket.as_deref(),
    )
    .await?
    {
        let direct_play = app
            .playback()
            .direct_playback_session_stream(DirectPlaybackSessionStreamRequest {
                principal: renderer_transport.principal,
                playback_session_id: renderer_transport.playback_session_id,
                source_id,
                range_request: direct_play_range_request(&headers),
            })
            .await?;
        let mut response = empty_direct_play_response(&direct_play.response);
        if let Some(session) = direct_play.session {
            insert_playback_session_header(&mut response, session.id);
        }

        return Ok(response);
    }

    let source_playback = resolve_source_playback_context(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Direct,
        ticket_query.ticket.as_deref(),
    )
    .await?;

    if let Some(playback_session_id) = source_playback.playback_session_id {
        let direct_play = app
            .playback()
            .direct_playback_session_preflight(DirectPlaybackSessionStreamRequest {
                principal: source_playback.principal,
                playback_session_id,
                source_id,
                range_request: direct_play_range_request(&headers),
            })
            .await?;
        let mut response = empty_direct_play_response(&direct_play.response);
        insert_playback_session_header(&mut response, direct_play.session.id);

        return Ok(response);
    } else {
        let direct_play = app
            .playback()
            .direct_playback_preflight(DirectPlaybackPreflightRequest {
                principal: source_playback.principal,
                source_id,
                range_request: direct_play_range_request(&headers),
                client: ClientPlaybackCapabilities::default(),
            })
            .await?;
        let mut response = empty_direct_play_response(&direct_play.response);
        insert_playback_session_header(&mut response, direct_play.session.id);

        Ok(response)
    }
}

#[instrument(skip(app, principal, query, headers))]
pub(super) async fn remux_stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<RemuxPlaybackQuery>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    if let Some(renderer_transport) = resolve_renderer_transport_principal(
        &app,
        source_id,
        PlaybackSessionMode::Remux,
        query.renderer_session_id.as_deref(),
        query.playback_session_id.as_deref(),
        query.renderer_ticket.as_deref(),
    )
    .await?
    {
        let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
        let remux = app
            .playback()
            .remux_playback_session_stream(RemuxPlaybackSessionStreamRequest {
                principal: renderer_transport.principal,
                playback_session_id: renderer_transport.playback_session_id,
                source_id,
                output_container,
                range_request: direct_play_range_request(&headers),
            })
            .await?;

        if remux.response.is_range_not_satisfiable() {
            return Ok(empty_direct_play_response(&remux.response));
        }

        let mut response = stream_local_file_response(
            &remux.output_path,
            &remux.output_path.display().to_string(),
            &remux.response,
        )
        .await?;
        insert_playback_session_header(&mut response, remux.session.id);
        return Ok(response);
    }

    let ticket = query.ticket.clone();
    let source_playback = resolve_source_playback_context(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Remux,
        ticket.as_deref(),
    )
    .await?;

    let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
    let remux = if let Some(playback_session_id) = source_playback.playback_session_id {
        app.playback()
            .remux_playback_session_stream(RemuxPlaybackSessionStreamRequest {
                principal: source_playback.principal,
                playback_session_id,
                source_id,
                output_container,
                range_request: direct_play_range_request(&headers),
            })
            .await?
    } else {
        let client: ClientPlaybackCapabilities = query.capabilities().into();
        app.playback()
            .remux_playback_stream(RemuxPlaybackStreamRequest {
                principal: source_playback.principal,
                source_id,
                client,
                output_container,
                range_request: direct_play_range_request(&headers),
            })
            .await?
    };

    if remux.response.is_range_not_satisfiable() {
        return Ok(empty_direct_play_response(&remux.response));
    }

    let mut response = stream_local_file_response(
        &remux.output_path,
        &remux.output_path.display().to_string(),
        &remux.response,
    )
    .await?;
    insert_playback_session_header(&mut response, remux.session.id);
    Ok(response)
}

#[instrument(skip(app, principal, query))]
pub(super) async fn head_remux_stream_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<RemuxPlaybackQuery>,
) -> ApiResult<Response> {
    if let Some(renderer_transport) = resolve_renderer_transport_principal(
        &app,
        source_id,
        PlaybackSessionMode::Remux,
        query.renderer_session_id.as_deref(),
        query.playback_session_id.as_deref(),
        query.renderer_ticket.as_deref(),
    )
    .await?
    {
        let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
        let remux = app
            .playback()
            .remux_playback_session_stream(RemuxPlaybackSessionStreamRequest {
                principal: renderer_transport.principal,
                playback_session_id: renderer_transport.playback_session_id,
                source_id,
                output_container,
                range_request: DirectPlayRangeRequest::None,
            })
            .await?;

        let mut response = empty_direct_play_response(&remux.response);
        insert_playback_session_header(&mut response, remux.session.id);
        return Ok(response);
    }

    let ticket = query.ticket.clone();
    let source_playback = resolve_source_playback_context(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Remux,
        ticket.as_deref(),
    )
    .await?;

    let output_container = query.output_container.unwrap_or(RemuxContainer::Mp4);
    if let Some(playback_session_id) = source_playback.playback_session_id {
        let remux = app
            .playback()
            .remux_playback_session_stream(RemuxPlaybackSessionStreamRequest {
                principal: source_playback.principal,
                playback_session_id,
                source_id,
                output_container,
                range_request: DirectPlayRangeRequest::None,
            })
            .await?;
        let mut response = empty_direct_play_response(&remux.response);
        insert_playback_session_header(&mut response, remux.session.id);
        return Ok(response);
    } else {
        let client: ClientPlaybackCapabilities = query.capabilities().into();
        let remux = app
            .playback()
            .remux_playback_preflight(RemuxPlaybackPreflightRequest {
                principal: source_playback.principal,
                source_id,
                client,
                output_container,
            })
            .await?;
        let mut response = empty_direct_play_response(&remux.response);
        insert_playback_session_header(&mut response, remux.session.id);

        Ok(response)
    }
}

#[instrument(skip(app, principal, query, http_trace_context))]
pub(super) async fn hls_playlist_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Extension(http_trace_context): Extension<HttpTraceContext>,
    Path(source_id): Path<MediaSourceId>,
    Query(query): Query<HlsPlaybackQuery>,
) -> ApiResult<Response> {
    let trace_context =
        PlaybackTraceContext::from_request_id(http_trace_context.request_id().to_owned());
    if let Some(renderer_transport) = resolve_renderer_transport_principal(
        &app,
        source_id,
        PlaybackSessionMode::Hls,
        query.renderer_session_id.as_deref(),
        query.playback_session_id.as_deref(),
        query.renderer_ticket.as_deref(),
    )
    .await?
    {
        let playlist = app
            .playback()
            .hls_playlist_for_playback_session(HlsPlaylistSessionRequest {
                principal: renderer_transport.principal,
                playback_session_id: renderer_transport.playback_session_id,
                source_id,
                playback_generation: query.playback_generation(),
                trace_context: Some(trace_context.clone()),
                transport_query: Some(format!(
                    "renderer_session_id={}&renderer_ticket={}",
                    renderer_transport.renderer_session_id,
                    query
                        .renderer_ticket
                        .as_deref()
                        .expect("renderer ticket was validated")
                )),
            })
            .await?;

        return Ok(hls_playlist_response(
            playlist.body,
            Some(playlist.session.id),
        ));
    }

    let ticket = query.ticket.clone();
    let source_playback = resolve_source_playback_context(
        &app,
        principal,
        source_id,
        BrowserPlaybackTicketMode::Hls,
        ticket.as_deref(),
    )
    .await?;

    let playlist = if let Some(playback_session_id) = source_playback.playback_session_id {
        app.playback()
            .hls_playlist_for_playback_session(HlsPlaylistSessionRequest {
                principal: source_playback.principal,
                playback_session_id,
                source_id,
                playback_generation: query.playback_generation(),
                trace_context: Some(trace_context.clone()),
                transport_query: ticket.as_deref().map(|ticket| format!("ticket={ticket}")),
            })
            .await?
    } else {
        let client: ClientPlaybackCapabilities = query.capabilities().into();
        app.playback()
            .hls_playlist_playback(HlsPlaylistPlaybackRequest {
                principal: source_playback.principal,
                source_id,
                client,
                preferences: query.preferences(),
                playback_generation: query.playback_generation(),
                trace_context: Some(trace_context.clone()),
                transport_query: ticket.as_deref().map(|ticket| format!("ticket={ticket}")),
            })
            .await?
    };

    Ok(hls_playlist_response(
        playlist.body,
        Some(playlist.session.id),
    ))
}

#[instrument(skip(app, principal, ticket_query))]
pub(super) async fn subtitle_source(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path((source_id, stream_index)): Path<(MediaSourceId, u32)>,
    Query(ticket_query): Query<BrowserPlaybackTicketQuery>,
) -> ApiResult<Response> {
    let principal = resolve_subtitle_playback_principal(
        &app,
        principal,
        source_id,
        stream_index,
        ticket_query.ticket.as_deref(),
    )
    .await?;
    let subtitle = app
        .playback()
        .subtitle_playback(SubtitlePlaybackRequest {
            principal,
            source_id,
            stream_index,
        })
        .await?;

    Ok(subtitle_response(
        subtitle.content,
        subtitle.content_type,
        subtitle.byte_len,
    ))
}

#[instrument(skip(app, principal, ticket_query))]
pub(super) async fn hls_segment(
    State(app): State<NakoApp>,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    Path((session_id, segment_name)): Path<(PlaybackSessionId, String)>,
    Query(ticket_query): Query<BrowserPlaybackTicketQuery>,
) -> ApiResult<Response> {
    let target = app
        .playback()
        .hls_segment_playback_target(session_id)
        .await?;

    let principal = if let Some(renderer_transport) =
        resolve_renderer_transport_principal_for_session(
            &app,
            target.source_id,
            PlaybackSessionMode::Hls,
            ticket_query.renderer_session_id.as_deref(),
            session_id,
            ticket_query.renderer_ticket.as_deref(),
        )
        .await?
    {
        renderer_transport.principal
    } else {
        let source_playback = resolve_source_playback_context(
            &app,
            principal,
            target.source_id,
            BrowserPlaybackTicketMode::Hls,
            ticket_query.ticket.as_deref(),
        )
        .await?;
        if source_playback
            .playback_session_id
            .is_some_and(|playback_session_id| playback_session_id != session_id)
        {
            return Err(invalid_browser_playback_ticket().into());
        }
        source_playback.principal
    };

    let segment = app
        .playback()
        .hls_segment_playback(HlsSegmentPlaybackRequest {
            principal,
            source_id: target.source_id,
            transcode_session_id: target.transcode_session_id,
            segment_name,
        })
        .await?;

    let mut response = stream_local_file_response(
        &segment.path,
        &segment.path.display().to_string(),
        &segment.response,
    )
    .await?;
    apply_hls_artifact_cache_headers(&mut response);
    Ok(response)
}

#[instrument(skip(app))]
pub(super) async fn get_playback_session(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(session_id): Path<PlaybackSessionId>,
) -> ApiResult<Json<PlaybackSessionResponse>> {
    let session = app.playback().get_playback_session(session_id).await?;
    require_playback_session_control_access(&app, &principal, &session).await?;

    Ok(Json(playback_session_response_from_record(session)))
}

#[instrument(skip(app))]
pub(super) async fn cancel_playback_session(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(session_id): Path<PlaybackSessionId>,
) -> ApiResult<Json<PlaybackSessionResponse>> {
    let session = app.playback().get_playback_session(session_id).await?;
    require_playback_session_control_access(&app, &principal, &session).await?;

    Ok(Json(playback_session_response_from_record(
        app.playback().cancel_playback_session(session_id).await?,
    )))
}

#[instrument(skip(app, request))]
pub(super) async fn heartbeat_playback_session(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(session_id): Path<PlaybackSessionId>,
    Json(request): Json<PublicPlaybackSessionHeartbeatRequest>,
) -> ApiResult<Json<PlaybackSessionResponse>> {
    let session = app.playback().get_playback_session(session_id).await?;
    require_playback_session_control_access(&app, &principal, &session).await?;

    Ok(Json(playback_session_response_from_record(
        app.playback()
            .record_playback_session_heartbeat(AppPlaybackSessionHeartbeatRequest {
                session_id,
                state: playback_session_state_from_public(&request.state)?,
                position_ms: request.position_ms,
                duration_ms: request.duration_ms,
            })
            .await?,
    )))
}

async fn require_playback_session_control_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    session: &nako_core::PlaybackSessionRecord,
) -> ApiResult<()> {
    if session.principal_id != principal.principal_id {
        return Err(playback_session_not_found(session.id).into());
    }
    let Some(source) = app.get_media_source_record(session.source_id).await? else {
        return Err(playback_session_not_found(session.id).into());
    };
    if !has_library_access(
        app,
        principal,
        source.library_id,
        RequiredLibraryAccess::Play,
    )
    .await?
    {
        return Err(playback_session_not_found(session.id).into());
    }

    Ok(())
}

fn browser_ticket_mode_from_public(
    mode: &BrowserPlaybackMode,
) -> ApiResult<BrowserPlaybackTicketMode> {
    match mode {
        BrowserPlaybackMode::Direct => Ok(BrowserPlaybackTicketMode::Direct),
        BrowserPlaybackMode::Remux => Ok(BrowserPlaybackTicketMode::Remux),
        BrowserPlaybackMode::Hls => Ok(BrowserPlaybackTicketMode::Hls),
        BrowserPlaybackMode::Subtitle => Ok(BrowserPlaybackTicketMode::Subtitle),
        BrowserPlaybackMode::Other(_) => Err(NakoError::InvalidInput {
            message: "unsupported browser playback mode".to_owned(),
        }
        .into()),
    }
}

fn playback_session_mode_from_browser(
    mode: BrowserPlaybackTicketMode,
) -> Option<PlaybackSessionMode> {
    match mode {
        BrowserPlaybackTicketMode::Direct => Some(PlaybackSessionMode::Direct),
        BrowserPlaybackTicketMode::Remux => Some(PlaybackSessionMode::Remux),
        BrowserPlaybackTicketMode::Hls => Some(PlaybackSessionMode::Hls),
        BrowserPlaybackTicketMode::Subtitle => None,
    }
}

fn playback_session_state_from_public(
    state: &ClientPlaybackSessionState,
) -> ApiResult<PlaybackSessionState> {
    match state {
        ClientPlaybackSessionState::Active => Ok(PlaybackSessionState::Active),
        ClientPlaybackSessionState::Paused => Ok(PlaybackSessionState::Paused),
        ClientPlaybackSessionState::CancelRequested => Ok(PlaybackSessionState::CancelRequested),
        ClientPlaybackSessionState::Cancelled => Ok(PlaybackSessionState::Cancelled),
        ClientPlaybackSessionState::Ended => Ok(PlaybackSessionState::Ended),
        ClientPlaybackSessionState::Failed => Ok(PlaybackSessionState::Failed),
        ClientPlaybackSessionState::Other(value) => Err(NakoError::InvalidInput {
            message: format!("unsupported playback session state: {value}"),
        }
        .into()),
    }
}

async fn browser_playback_url(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    source_id: MediaSourceId,
    mode: BrowserPlaybackTicketMode,
    subtitle_stream_index: Option<u32>,
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
        BrowserPlaybackTicketMode::Subtitle => {
            let stream_index = subtitle_stream_index.ok_or_else(|| NakoError::InvalidInput {
                message: "subtitle browser playback ticket requires subtitle_stream_index"
                    .to_owned(),
            })?;
            Ok(BrowserPlaybackUrlDto {
                kind: BrowserPlaybackUrlKind::Subtitle,
                url: format!(
                    "/sources/{source_id}/subtitles/{stream_index}?ticket={}",
                    issued.token
                ),
                content_type: subtitle_content_type_for_browser_url(
                    app,
                    principal,
                    source_id,
                    stream_index,
                )
                .await?,
                supports_range_requests: false,
            })
        }
    }
}

async fn subtitle_content_type_for_browser_url(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    source_id: MediaSourceId,
    stream_index: u32,
) -> ApiResult<String> {
    let probe = app.catalog().get_source_probe(principal, source_id).await?;
    let stream = probe
        .probe
        .streams
        .iter()
        .find(|stream| stream.index == stream_index)
        .ok_or_else(|| NakoError::NotFound {
            entity: "subtitle_stream",
            id: stream_index.to_string(),
        })?;
    let codec = stream
        .codec
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content_type = match codec.as_str() {
        "vtt" | "webvtt" => "text/vtt; charset=utf-8",
        "srt" => "application/x-subrip; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    };

    Ok(content_type.to_owned())
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

fn normalize_client_for_browser_remux_ticket(
    client: &mut ClientPlaybackCapabilities,
    output_container: RemuxContainer,
) {
    // Explicit remux tickets plan against the returned output container, not
    // source containers advertised for direct play.
    client.direct_play = true;
    client.containers = vec![output_container.file_extension().to_owned()];
}

fn browser_capabilities_to_client(
    capabilities: Option<&BrowserPlaybackCapabilitiesDto>,
) -> ApiResult<ClientPlaybackCapabilities> {
    let defaults = ClientPlaybackCapabilities::default();
    let Some(capabilities) = capabilities else {
        return Ok(defaults);
    };

    Ok(ClientPlaybackCapabilities {
        direct_play: capabilities.direct_play.unwrap_or(defaults.direct_play),
        containers: capabilities
            .container
            .clone()
            .unwrap_or(defaults.containers),
        video_codecs: capabilities
            .video_codec
            .clone()
            .unwrap_or(defaults.video_codecs),
        audio_codecs: capabilities
            .audio_codec
            .clone()
            .unwrap_or(defaults.audio_codecs),
        max_video_bitrate: capabilities
            .max_video_bitrate
            .or(defaults.max_video_bitrate),
        max_width: capabilities.max_width.or(defaults.max_width),
        max_height: capabilities.max_height.or(defaults.max_height),
        max_audio_channels: capabilities
            .max_audio_channels
            .or(defaults.max_audio_channels),
        supports_hdr: capabilities.supports_hdr.unwrap_or(defaults.supports_hdr),
        supports_subtitles: capabilities
            .supports_subtitles
            .unwrap_or(defaults.supports_subtitles),
        hls_variant_policy: match capabilities.hls_variant_policy.as_ref() {
            Some(ClientHlsVariantPolicy::SingleVariant) | None => {
                PlaybackHlsVariantPolicy::SingleVariant
            }
            Some(ClientHlsVariantPolicy::Adaptive) => PlaybackHlsVariantPolicy::Adaptive,
            Some(ClientHlsVariantPolicy::Other(value)) => {
                return Err(NakoError::InvalidInput {
                    message: format!("unsupported browser playback HLS variant policy: {value}"),
                }
                .into());
            }
        },
        hls_segment_container: match capabilities.hls_segment_container.as_ref() {
            Some(ClientHlsSegmentContainer::MpegTs) | None => PlaybackHlsSegmentContainer::MpegTs,
            Some(ClientHlsSegmentContainer::Fmp4) => PlaybackHlsSegmentContainer::Fmp4,
            Some(ClientHlsSegmentContainer::Other(value)) => {
                return Err(NakoError::InvalidInput {
                    message: format!("unsupported browser playback HLS segment container: {value}"),
                }
                .into());
            }
        },
    })
}

#[derive(Clone, Debug)]
struct ResolvedSourcePlayback {
    principal: AuthenticatedPrincipal,
    playback_session_id: Option<PlaybackSessionId>,
}

async fn resolve_source_playback_context(
    app: &NakoApp,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    source_id: MediaSourceId,
    mode: BrowserPlaybackTicketMode,
    ticket: Option<&str>,
) -> ApiResult<ResolvedSourcePlayback> {
    if let Some(ticket) = ticket {
        if ticket.trim().is_empty() {
            return Err(invalid_browser_playback_ticket().into());
        }
        let validated = app.playback_tickets().validate_source_ticket(
            ticket,
            source_id,
            mode,
            crate::app::current_time_ms()?,
        )?;
        return Ok(ResolvedSourcePlayback {
            principal: validated.principal,
            playback_session_id: Some(validated.playback_session_id),
        });
    }

    if let Some(Extension(principal)) = principal {
        return Ok(ResolvedSourcePlayback {
            principal,
            playback_session_id: None,
        });
    }

    Err(NakoError::Unauthorized {
        message: "authentication required".to_owned(),
    }
    .into())
}

async fn resolve_subtitle_playback_principal(
    app: &NakoApp,
    principal: Option<Extension<AuthenticatedPrincipal>>,
    source_id: MediaSourceId,
    stream_index: u32,
    ticket: Option<&str>,
) -> ApiResult<AuthenticatedPrincipal> {
    if let Some(ticket) = ticket {
        if ticket.trim().is_empty() {
            return Err(invalid_browser_playback_ticket().into());
        }
        let principal = app.playback_tickets().validate_subtitle_ticket(
            ticket,
            source_id,
            stream_index,
            crate::app::current_time_ms()?,
        )?;
        return Ok(principal);
    }

    if let Some(Extension(principal)) = principal {
        return Ok(principal);
    }

    Err(NakoError::Unauthorized {
        message: "authentication required".to_owned(),
    }
    .into())
}

#[derive(Clone, Debug)]
struct ResolvedRendererTransport {
    principal: AuthenticatedPrincipal,
    renderer_session_id: RendererSessionId,
    playback_session_id: PlaybackSessionId,
}

async fn resolve_renderer_transport_principal(
    app: &NakoApp,
    source_id: MediaSourceId,
    mode: PlaybackSessionMode,
    renderer_session_id: Option<&str>,
    playback_session_id: Option<&str>,
    renderer_ticket: Option<&str>,
) -> ApiResult<Option<ResolvedRendererTransport>> {
    let Some(playback_session_id) = playback_session_id else {
        if renderer_ticket.is_some() {
            return Err(invalid_renderer_transport_ticket().into());
        }

        return Ok(None);
    };
    let playback_session_id = parse_playback_session_id(playback_session_id)?;

    resolve_renderer_transport_principal_for_session(
        app,
        source_id,
        mode,
        renderer_session_id,
        playback_session_id,
        renderer_ticket,
    )
    .await
}

async fn resolve_renderer_transport_principal_for_session(
    app: &NakoApp,
    source_id: MediaSourceId,
    mode: PlaybackSessionMode,
    renderer_session_id: Option<&str>,
    playback_session_id: PlaybackSessionId,
    renderer_ticket: Option<&str>,
) -> ApiResult<Option<ResolvedRendererTransport>> {
    let Some(ticket) = renderer_ticket else {
        return Ok(None);
    };
    if ticket.trim().is_empty() {
        return Err(invalid_renderer_transport_ticket().into());
    }
    let Some(renderer_session_id) = renderer_session_id else {
        return Err(invalid_renderer_transport_ticket().into());
    };
    let renderer_session_id = parse_renderer_session_id(renderer_session_id)?;
    let renderer = app
        .renderer()
        .get_online_renderer(renderer_session_id)
        .await?;
    let validated =
        app.renderer_transport_tickets()
            .validate(ValidateRendererTransportTicketRequest {
                token: ticket.to_owned(),
                scope: RendererTransportTicketScope {
                    renderer_session_id,
                    playback_session_id,
                    source_id,
                    mode,
                    network_scope: renderer.network_scope,
                },
                now_ms: crate::app::current_time_ms()?,
            })?;
    if validated.principal.principal_id != renderer.owner_principal_id {
        return Err(invalid_renderer_transport_ticket().into());
    }

    require_source_access(
        app,
        &validated.principal,
        source_id,
        RequiredLibraryAccess::Play,
    )
    .await?;

    Ok(Some(ResolvedRendererTransport {
        principal: validated.principal,
        renderer_session_id,
        playback_session_id,
    }))
}

fn invalid_browser_playback_ticket() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid browser playback ticket".to_owned(),
    }
}

fn invalid_renderer_transport_ticket() -> NakoError {
    NakoError::Unauthorized {
        message: "invalid renderer transport ticket".to_owned(),
    }
}

fn playback_session_not_found(session_id: PlaybackSessionId) -> NakoError {
    NakoError::NotFound {
        entity: "playback_session",
        id: session_id.to_string(),
    }
}

fn parse_playback_session_id(value: &str) -> ApiResult<PlaybackSessionId> {
    value.parse::<PlaybackSessionId>().map_err(|err| {
        NakoError::InvalidInput {
            message: format!("invalid playback_session_id: {err}"),
        }
        .into()
    })
}

fn parse_renderer_session_id(value: &str) -> ApiResult<RendererSessionId> {
    value.parse::<RendererSessionId>().map_err(|err| {
        NakoError::InvalidInput {
            message: format!("invalid renderer_session_id: {err}"),
        }
        .into()
    })
}

fn format_ticket_timestamp(timestamp_ms: i64) -> String {
    timestamp_ms_to_rfc3339(Some(timestamp_ms)).unwrap_or_else(|| timestamp_ms.to_string())
}

fn insert_playback_session_header(response: &mut Response, session_id: PlaybackSessionId) {
    response.headers_mut().insert(
        PLAYBACK_SESSION_ID_HEADER,
        HeaderValue::from_str(&session_id.to_string()).expect("session id is a valid header value"),
    );
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

fn hls_playlist_response(body: String, session_id: Option<PlaybackSessionId>) -> Response {
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
            HeaderValue::from_str(&session_id.to_string())
                .expect("session id is a valid header value"),
        );
    }
    apply_hls_artifact_cache_headers(&mut response);
    response
}

fn subtitle_response(body: String, content_type: &'static str, byte_len: u64) -> Response {
    let mut response = Body::from(body).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&byte_len.to_string()).expect("content length is a valid header"),
    );
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
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));

    if let Some(content_range) = &plan.content_range {
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(content_range).expect("content range is a valid header"),
        );
    }
}

fn apply_hls_artifact_cache_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
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
    max_video_bitrate: Option<u64>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    max_audio_channels: Option<u32>,
    supports_hdr: Option<bool>,
    supports_subtitles: Option<bool>,
    hls_variant_policy: Option<PlaybackHlsVariantPolicy>,
    hls_segment_container: Option<PlaybackHlsSegmentContainer>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RemuxPlaybackQuery {
    direct_play: Option<bool>,
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    max_video_bitrate: Option<u64>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    max_audio_channels: Option<u32>,
    supports_hdr: Option<bool>,
    supports_subtitles: Option<bool>,
    hls_variant_policy: Option<PlaybackHlsVariantPolicy>,
    hls_segment_container: Option<PlaybackHlsSegmentContainer>,
    output_container: Option<RemuxContainer>,
    ticket: Option<String>,
    renderer_session_id: Option<String>,
    playback_session_id: Option<String>,
    renderer_ticket: Option<String>,
}

impl RemuxPlaybackQuery {
    fn capabilities(&self) -> PlaybackCapabilitiesQuery {
        PlaybackCapabilitiesQuery {
            direct_play: self.direct_play,
            container: self.container.clone(),
            video_codec: self.video_codec.clone(),
            audio_codec: self.audio_codec.clone(),
            max_video_bitrate: self.max_video_bitrate,
            max_width: self.max_width,
            max_height: self.max_height,
            max_audio_channels: self.max_audio_channels,
            supports_hdr: self.supports_hdr,
            supports_subtitles: self.supports_subtitles,
            hls_variant_policy: self.hls_variant_policy,
            hls_segment_container: self.hls_segment_container,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct HlsPlaybackQuery {
    direct_play: Option<bool>,
    container: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    max_video_bitrate: Option<u64>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    max_audio_channels: Option<u32>,
    supports_hdr: Option<bool>,
    supports_subtitles: Option<bool>,
    hls_variant_policy: Option<PlaybackHlsVariantPolicy>,
    hls_segment_container: Option<PlaybackHlsSegmentContainer>,
    start_position_ms: Option<u64>,
    audio_stream: Option<u32>,
    preferred_audio_language: Option<String>,
    subtitle_stream: Option<u32>,
    preferred_subtitle_language: Option<String>,
    ticket: Option<String>,
    renderer_session_id: Option<String>,
    playback_session_id: Option<String>,
    renderer_ticket: Option<String>,
}

impl HlsPlaybackQuery {
    fn capabilities(&self) -> PlaybackCapabilitiesQuery {
        PlaybackCapabilitiesQuery {
            direct_play: self.direct_play,
            container: self.container.clone(),
            video_codec: self.video_codec.clone(),
            audio_codec: self.audio_codec.clone(),
            max_video_bitrate: self.max_video_bitrate,
            max_width: self.max_width,
            max_height: self.max_height,
            max_audio_channels: self.max_audio_channels,
            supports_hdr: self.supports_hdr,
            supports_subtitles: self.supports_subtitles,
            hls_variant_policy: self.hls_variant_policy,
            hls_segment_container: self.hls_segment_container,
        }
    }

    fn preferences(&self) -> PlaybackPreferenceContext {
        PlaybackPreferenceContext {
            requested_audio_stream: self.audio_stream,
            preferred_audio_languages: csv_or_default(
                self.preferred_audio_language.clone(),
                Vec::new(),
            ),
            requested_subtitle_stream: self.subtitle_stream,
            preferred_subtitle_languages: csv_or_default(
                self.preferred_subtitle_language.clone(),
                Vec::new(),
            ),
            max_video_bitrate: None,
            prefer_hdr: None,
            remux_output_container: None,
            transcode_output_container: Some(PlaybackTranscodeContainer::Hls),
        }
    }

    fn playback_generation(&self) -> HlsPlaybackGeneration {
        HlsPlaybackGeneration::from_start_position_ms(self.start_position_ms.unwrap_or_default())
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct BrowserPlaybackTicketQuery {
    ticket: Option<String>,
    renderer_session_id: Option<String>,
    playback_session_id: Option<String>,
    renderer_ticket: Option<String>,
}

impl From<PlaybackCapabilitiesQuery> for ClientPlaybackCapabilities {
    fn from(value: PlaybackCapabilitiesQuery) -> Self {
        let defaults = ClientPlaybackCapabilities::default();

        Self {
            direct_play: value.direct_play.unwrap_or(defaults.direct_play),
            containers: csv_or_default(value.container, defaults.containers),
            video_codecs: csv_or_default(value.video_codec, defaults.video_codecs),
            audio_codecs: csv_or_default(value.audio_codec, defaults.audio_codecs),
            max_video_bitrate: value.max_video_bitrate.or(defaults.max_video_bitrate),
            max_width: value.max_width.or(defaults.max_width),
            max_height: value.max_height.or(defaults.max_height),
            max_audio_channels: value.max_audio_channels.or(defaults.max_audio_channels),
            supports_hdr: value.supports_hdr.unwrap_or(defaults.supports_hdr),
            supports_subtitles: value
                .supports_subtitles
                .unwrap_or(defaults.supports_subtitles),
            hls_variant_policy: value
                .hls_variant_policy
                .unwrap_or(defaults.hls_variant_policy),
            hls_segment_container: value
                .hls_segment_container
                .unwrap_or(defaults.hls_segment_container),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_flat_capabilities(capabilities: ClientPlaybackCapabilities) {
        assert!(!capabilities.direct_play);
        assert_eq!(
            capabilities.containers,
            vec!["mp4".to_owned(), "webm".to_owned()]
        );
        assert_eq!(
            capabilities.video_codecs,
            vec!["h264".to_owned(), "hevc".to_owned()]
        );
        assert_eq!(
            capabilities.audio_codecs,
            vec!["aac".to_owned(), "opus".to_owned()]
        );
        assert_eq!(capabilities.max_video_bitrate, Some(8_000_000));
        assert_eq!(capabilities.max_width, Some(1920));
        assert_eq!(capabilities.max_height, Some(1080));
        assert_eq!(capabilities.max_audio_channels, Some(2));
        assert!(!capabilities.supports_hdr);
        assert!(capabilities.supports_subtitles);
        assert_eq!(
            capabilities.hls_variant_policy,
            PlaybackHlsVariantPolicy::Adaptive
        );
        assert_eq!(
            capabilities.hls_segment_container,
            PlaybackHlsSegmentContainer::Fmp4
        );
    }

    #[test]
    fn playback_capability_queries_map_all_current_flat_fields() {
        let query = PlaybackCapabilitiesQuery {
            direct_play: Some(false),
            container: Some("mp4,webm".to_owned()),
            video_codec: Some("h264,hevc".to_owned()),
            audio_codec: Some("aac,opus".to_owned()),
            max_video_bitrate: Some(8_000_000),
            max_width: Some(1920),
            max_height: Some(1080),
            max_audio_channels: Some(2),
            supports_hdr: Some(false),
            supports_subtitles: Some(true),
            hls_variant_policy: Some(PlaybackHlsVariantPolicy::Adaptive),
            hls_segment_container: Some(PlaybackHlsSegmentContainer::Fmp4),
        };

        assert_flat_capabilities(ClientPlaybackCapabilities::from(query.clone()));

        let remux = RemuxPlaybackQuery {
            direct_play: query.direct_play,
            container: query.container.clone(),
            video_codec: query.video_codec.clone(),
            audio_codec: query.audio_codec.clone(),
            max_video_bitrate: query.max_video_bitrate,
            max_width: query.max_width,
            max_height: query.max_height,
            max_audio_channels: query.max_audio_channels,
            supports_hdr: query.supports_hdr,
            supports_subtitles: query.supports_subtitles,
            hls_variant_policy: query.hls_variant_policy,
            hls_segment_container: query.hls_segment_container,
            output_container: Some(RemuxContainer::Mkv),
            ticket: None,
            renderer_session_id: None,
            playback_session_id: None,
            renderer_ticket: None,
        };
        assert_flat_capabilities(ClientPlaybackCapabilities::from(remux.capabilities()));

        let hls = HlsPlaybackQuery {
            direct_play: query.direct_play,
            container: query.container,
            video_codec: query.video_codec,
            audio_codec: query.audio_codec,
            max_video_bitrate: query.max_video_bitrate,
            max_width: query.max_width,
            max_height: query.max_height,
            max_audio_channels: query.max_audio_channels,
            supports_hdr: query.supports_hdr,
            supports_subtitles: query.supports_subtitles,
            hls_variant_policy: query.hls_variant_policy,
            hls_segment_container: query.hls_segment_container,
            start_position_ms: Some(120_000),
            audio_stream: Some(1),
            preferred_audio_language: Some("eng,jpn".to_owned()),
            subtitle_stream: Some(2),
            preferred_subtitle_language: Some("eng".to_owned()),
            ticket: None,
            renderer_session_id: None,
            playback_session_id: None,
            renderer_ticket: None,
        };
        assert_flat_capabilities(ClientPlaybackCapabilities::from(hls.capabilities()));
    }

    #[test]
    fn browser_playback_ticket_capabilities_map_all_current_flat_fields() {
        let capabilities = BrowserPlaybackCapabilitiesDto {
            direct_play: Some(false),
            container: Some(vec!["mp4".to_owned(), "webm".to_owned()]),
            video_codec: Some(vec!["h264".to_owned(), "hevc".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned(), "opus".to_owned()]),
            max_video_bitrate: Some(8_000_000),
            max_width: Some(1920),
            max_height: Some(1080),
            max_audio_channels: Some(2),
            supports_hdr: Some(false),
            supports_subtitles: Some(true),
            hls_variant_policy: Some(ClientHlsVariantPolicy::Adaptive),
            hls_segment_container: Some(ClientHlsSegmentContainer::Fmp4),
            output_container: Some(BrowserPlaybackOutputContainer::Mkv),
        };

        assert_flat_capabilities(browser_capabilities_to_client(Some(&capabilities)).unwrap());
    }
}
