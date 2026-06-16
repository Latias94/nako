use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use nako_api::public_client::{
    ClientHlsSegmentContainer, ClientHlsVariantPolicy, ClientPlaybackCapabilitiesDto,
    ClientPlaybackTargetKind, ClientPlaybackTargetNetworkScope, ClientPlaybackTargetTransportAuth,
    ClientRendererCommandState, ClientRendererControlCapabilitiesDto, ClientRendererControlCommand,
    ClientRendererSessionState, RendererCommandCompletionRequest, RendererCommandPollResponse,
    RendererCommandTransportDto, RendererCommandTransportUrlDto, RendererHeartbeatRequest,
    RendererPlayCommandRequest, RendererPlayCommandResponse, RendererRegistrationRequest,
    RendererSessionResponse, RendererSessionsResponse, RendererTransportMode,
    RendererTransportUrlKind, page_info_from_request,
    renderer_command_poll_response_from_record_with_transport,
    renderer_command_response_from_record,
    renderer_play_command_response_from_records_with_transport,
    renderer_session_response_from_record, renderer_session_to_dto, timestamp_ms_to_rfc3339,
};
use nako_core::{
    AuthenticatedPrincipal, MediaSourceId, NakoError, PlaybackSessionMode, PlaybackSessionRecord,
    PlaybackTargetKind, PlaybackTargetNetworkScope, PlaybackTargetTransportAuth, RendererCommandId,
    RendererCommandRecord, RendererCommandState, RendererControlCapabilities,
    RendererControlCommand, RendererSessionId, RendererSessionRecord, RendererSessionState,
};
use nako_playback::{
    ClientPlaybackCapabilities, PlaybackHlsSegmentContainer, PlaybackHlsVariantPolicy,
};
use nako_transcode::RemuxContainer;
use tracing::instrument;

use crate::app::{
    IssueRendererTransportTicketRequest, NakoApp, RendererPlaybackTransportPlan,
    RendererTransportTicketScope,
    casting::{
        PlayOnRendererRequest as AppPlayOnRendererRequest, renderer_command_transport_payload,
    },
    renderer::{
        CompleteRendererCommandRequest as AppCompleteRendererCommandRequest,
        RegisterRendererRequest as AppRegisterRendererRequest,
        RendererHeartbeatUpdate as AppRendererHeartbeatUpdate,
    },
};

use super::{error::ApiResult, query::PageQuery};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/renderers", get(list_renderers).post(register_renderer))
        .route(
            "/renderers/{renderer_session_id}/heartbeat",
            post(heartbeat_renderer),
        )
        .route(
            "/renderers/{renderer_session_id}/commands/next",
            post(poll_next_renderer_command),
        )
        .route(
            "/renderers/{renderer_session_id}/commands/play",
            post(play_on_renderer),
        )
        .route(
            "/renderers/{renderer_session_id}/commands/{command_id}/complete",
            post(complete_renderer_command),
        )
}

#[instrument(skip(app, principal, request))]
async fn register_renderer(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<RendererRegistrationRequest>,
) -> ApiResult<Json<RendererSessionResponse>> {
    let renderer = app
        .renderer()
        .register_renderer(AppRegisterRendererRequest {
            principal_id: principal.principal_id,
            display_name: request.display_name,
            target_kind: playback_target_kind_from_client(&request.target_kind)?,
            network_scope: playback_target_network_scope_from_client(&request.network_scope)?,
            transport_auth: playback_target_transport_auth_from_client(&request.transport_auth)?,
            media_capabilities: request.media_capabilities.map(client_playback_capabilities),
            control_capabilities: renderer_control_capabilities_from_client(
                request.control_capabilities,
            )?,
            ttl_ms: request.ttl_ms,
        })
        .await?;

    Ok(Json(renderer_session_response_from_record(renderer)))
}

#[instrument(skip(app, principal))]
async fn list_renderers(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = page.try_into()?;
    let renderers = app
        .renderer()
        .list_controllable_renderers(&principal.principal_id, page)
        .await?;
    let returned = renderers.len();

    Ok(Json(RendererSessionsResponse {
        renderers: renderers.into_iter().map(renderer_session_to_dto).collect(),
        page: page_info_from_request(page, returned),
    }))
}

#[instrument(skip(app, principal, request))]
async fn heartbeat_renderer(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(renderer_session_id): Path<RendererSessionId>,
    Json(request): Json<RendererHeartbeatRequest>,
) -> ApiResult<Json<RendererSessionResponse>> {
    let renderer = app
        .renderer()
        .heartbeat_renderer(AppRendererHeartbeatUpdate {
            principal_id: principal.principal_id,
            renderer_session_id,
            state: renderer_session_state_from_client(&request.state)?,
            media_capabilities: request.media_capabilities.map(client_playback_capabilities),
            control_capabilities: request
                .control_capabilities
                .map(renderer_control_capabilities_from_client)
                .transpose()?,
            ttl_ms: request.ttl_ms,
        })
        .await?;

    Ok(Json(renderer_session_response_from_record(renderer)))
}

#[instrument(skip(app, principal))]
async fn poll_next_renderer_command(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(renderer_session_id): Path<RendererSessionId>,
) -> ApiResult<Json<RendererCommandPollResponse>> {
    let command = app
        .renderer()
        .poll_next_command(&principal.principal_id, renderer_session_id)
        .await?;
    let transport = if let Some(command) = command.as_ref() {
        renderer_transport_for_command(&app, &principal, command).await?
    } else {
        None
    };

    Ok(Json(
        renderer_command_poll_response_from_record_with_transport(command, transport),
    ))
}

#[instrument(skip(app, principal, request))]
async fn play_on_renderer(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(renderer_session_id): Path<RendererSessionId>,
    Json(request): Json<RendererPlayCommandRequest>,
) -> ApiResult<Json<RendererPlayCommandResponse>> {
    let source_id = parse_media_source_id(&request.source_id)?;

    let output = app
        .casting()
        .play_on_renderer(AppPlayOnRendererRequest {
            principal: principal.clone(),
            renderer_session_id,
            source_id,
            position_ms: request.position_ms,
        })
        .await?;
    let transport = renderer_transport_for_command(&app, &principal, &output.command).await?;

    Ok(Json(
        renderer_play_command_response_from_records_with_transport(
            output.command,
            output.session,
            transport,
        ),
    ))
}

async fn renderer_transport_for_command(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    command: &RendererCommandRecord,
) -> ApiResult<Option<RendererCommandTransportDto>> {
    if !matches!(command.command, RendererControlCommand::Play) {
        return Ok(None);
    }
    let (Some(source_id), Some(playback_session_id)) =
        (command.source_id, command.playback_session_id)
    else {
        return Ok(None);
    };
    let renderer = app
        .renderer()
        .get_controllable_renderer(&principal.principal_id, command.renderer_session_id)
        .await?;
    if !matches!(
        renderer.transport_auth,
        PlaybackTargetTransportAuth::CastTicket
    ) {
        return Ok(None);
    }

    let session = app
        .playback()
        .get_playback_session(playback_session_id)
        .await?;
    if session.source_id != source_id {
        return Err(NakoError::InvalidInput {
            message: format!(
                "renderer command {} source_id does not match playback session {}",
                command.id, session.id
            ),
        }
        .into());
    }
    if session.principal_id != principal.principal_id {
        return Err(NakoError::Forbidden {
            message: "renderer command playback session belongs to a different principal"
                .to_owned(),
        }
        .into());
    }
    let transport_plan = if let Some(payload) = renderer_command_transport_payload(command) {
        payload.transport
    } else {
        renderer_transport_plan_for_session(app, &session).await?
    };
    if transport_plan.mode != session.mode {
        return Err(NakoError::InvalidInput {
            message: format!(
                "renderer command transport mode {} does not match playback session {} mode {}",
                transport_plan.mode.as_str(),
                session.id,
                session.mode.as_str()
            ),
        }
        .into());
    }

    Ok(Some(issue_renderer_command_transport(
        app,
        principal,
        &renderer,
        &session,
        source_id,
        &transport_plan,
    )?))
}

async fn renderer_transport_plan_for_session(
    app: &NakoApp,
    session: &PlaybackSessionRecord,
) -> ApiResult<RendererPlaybackTransportPlan> {
    match session.mode {
        PlaybackSessionMode::Direct => {
            let response = app
                .playback()
                .plan_direct_play_preflight(
                    session.source_id,
                    nako_streaming::DirectPlayRangeRequest::None,
                )
                .await?;
            Ok(RendererPlaybackTransportPlan {
                mode: PlaybackSessionMode::Direct,
                remux_container: None,
                content_type: response.content_type,
                supports_range_requests: true,
            })
        }
        PlaybackSessionMode::Remux => {
            let container = RemuxContainer::Mp4;
            Ok(RendererPlaybackTransportPlan {
                mode: PlaybackSessionMode::Remux,
                remux_container: Some(container),
                content_type: nako_streaming::content_type_for_file_name(&format!(
                    "stream.{}",
                    container.file_extension()
                ))
                .to_owned(),
                supports_range_requests: true,
            })
        }
        PlaybackSessionMode::Hls => Ok(RendererPlaybackTransportPlan {
            mode: PlaybackSessionMode::Hls,
            remux_container: None,
            content_type: "application/vnd.apple.mpegurl".to_owned(),
            supports_range_requests: false,
        }),
    }
}

fn issue_renderer_command_transport(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    renderer: &RendererSessionRecord,
    session: &PlaybackSessionRecord,
    source_id: MediaSourceId,
    plan: &RendererPlaybackTransportPlan,
) -> ApiResult<RendererCommandTransportDto> {
    let issued = app
        .renderer_transport_tickets()
        .issue(IssueRendererTransportTicketRequest {
            principal: principal.clone(),
            scope: RendererTransportTicketScope {
                renderer_session_id: renderer.id,
                playback_session_id: session.id,
                source_id,
                mode: plan.mode,
                network_scope: renderer.network_scope,
            },
            now_ms: crate::app::current_time_ms()?,
        })?;
    let query = format!(
        "renderer_session_id={}&playback_session_id={}&renderer_ticket={}",
        renderer.id, session.id, issued.token
    );
    let url = match plan.mode {
        PlaybackSessionMode::Direct => format!("/sources/{source_id}/stream?{query}"),
        PlaybackSessionMode::Remux => {
            let container = plan.remux_container.unwrap_or(RemuxContainer::Mp4);
            format!(
                "/sources/{source_id}/stream/remux?output_container={}&{query}",
                container.file_extension()
            )
        }
        PlaybackSessionMode::Hls => {
            format!("/sources/{source_id}/stream/hls/playlist.m3u8?{query}")
        }
    };

    Ok(RendererCommandTransportDto {
        mode: renderer_transport_mode(plan.mode),
        expires_at: timestamp_ms_to_rfc3339(Some(issued.expires_at_ms))
            .unwrap_or_else(|| issued.expires_at_ms.to_string()),
        urls: vec![RendererCommandTransportUrlDto {
            kind: renderer_transport_url_kind(plan.mode),
            url,
            content_type: plan.content_type.clone(),
            supports_range_requests: plan.supports_range_requests,
        }],
    })
}

fn renderer_transport_mode(mode: PlaybackSessionMode) -> RendererTransportMode {
    match mode {
        PlaybackSessionMode::Direct => RendererTransportMode::Direct,
        PlaybackSessionMode::Remux => RendererTransportMode::Remux,
        PlaybackSessionMode::Hls => RendererTransportMode::Hls,
    }
}

fn renderer_transport_url_kind(mode: PlaybackSessionMode) -> RendererTransportUrlKind {
    match mode {
        PlaybackSessionMode::Direct | PlaybackSessionMode::Remux => {
            RendererTransportUrlKind::Stream
        }
        PlaybackSessionMode::Hls => RendererTransportUrlKind::Playlist,
    }
}

#[instrument(skip(app, principal, request))]
async fn complete_renderer_command(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path((renderer_session_id, command_id)): Path<(RendererSessionId, RendererCommandId)>,
    Json(request): Json<RendererCommandCompletionRequest>,
) -> ApiResult<impl IntoResponse> {
    let command = app
        .renderer()
        .complete_command(AppCompleteRendererCommandRequest {
            principal_id: principal.principal_id,
            renderer_session_id,
            command_id,
            state: renderer_command_state_from_client(&request.state)?,
            failure_message: request.failure_message,
        })
        .await?;

    Ok(Json(renderer_command_response_from_record(command)))
}

fn client_playback_capabilities(
    capabilities: ClientPlaybackCapabilitiesDto,
) -> ClientPlaybackCapabilities {
    let defaults = ClientPlaybackCapabilities::default();

    ClientPlaybackCapabilities {
        direct_play: capabilities.direct_play,
        device_family: capabilities.device_family,
        profile_version: capabilities.profile_version,
        containers: capabilities.containers,
        video_codecs: capabilities.video_codecs,
        audio_codecs: capabilities.audio_codecs,
        max_video_bitrate: capabilities.max_video_bitrate,
        max_width: capabilities.max_width,
        max_height: capabilities.max_height,
        max_audio_channels: capabilities.max_audio_channels,
        supports_hdr: capabilities.supports_hdr.unwrap_or(defaults.supports_hdr),
        supports_subtitles: capabilities
            .supports_subtitles
            .unwrap_or(defaults.supports_subtitles),
        hls_variant_policy: match capabilities.hls_variant_policy {
            Some(ClientHlsVariantPolicy::Adaptive) => PlaybackHlsVariantPolicy::Adaptive,
            Some(ClientHlsVariantPolicy::SingleVariant) | None => {
                PlaybackHlsVariantPolicy::SingleVariant
            }
            Some(ClientHlsVariantPolicy::Other(_)) => defaults.hls_variant_policy,
        },
        hls_segment_container: match capabilities.hls_segment_container {
            Some(ClientHlsSegmentContainer::Fmp4) => PlaybackHlsSegmentContainer::Fmp4,
            Some(ClientHlsSegmentContainer::MpegTs) | None => PlaybackHlsSegmentContainer::MpegTs,
            Some(ClientHlsSegmentContainer::Other(_)) => defaults.hls_segment_container,
        },
    }
}

fn parse_media_source_id(value: &str) -> Result<MediaSourceId, NakoError> {
    value
        .parse::<MediaSourceId>()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid source_id: {err}"),
        })
}

fn playback_target_kind_from_client(
    kind: &ClientPlaybackTargetKind,
) -> Result<PlaybackTargetKind, NakoError> {
    PlaybackTargetKind::parse(kind.wire_value()).ok_or_else(|| NakoError::InvalidInput {
        message: format!("unsupported renderer target kind: {}", kind.wire_value()),
    })
}

fn playback_target_network_scope_from_client(
    scope: &ClientPlaybackTargetNetworkScope,
) -> Result<PlaybackTargetNetworkScope, NakoError> {
    PlaybackTargetNetworkScope::parse(scope.wire_value()).ok_or_else(|| NakoError::InvalidInput {
        message: format!(
            "unsupported renderer target network scope: {}",
            scope.wire_value()
        ),
    })
}

fn playback_target_transport_auth_from_client(
    auth: &ClientPlaybackTargetTransportAuth,
) -> Result<PlaybackTargetTransportAuth, NakoError> {
    PlaybackTargetTransportAuth::parse(auth.wire_value()).ok_or_else(|| NakoError::InvalidInput {
        message: format!(
            "unsupported renderer target transport auth: {}",
            auth.wire_value()
        ),
    })
}

fn renderer_control_capabilities_from_client(
    capabilities: ClientRendererControlCapabilitiesDto,
) -> Result<RendererControlCapabilities, NakoError> {
    let commands = capabilities
        .commands
        .iter()
        .map(renderer_control_command_from_client)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RendererControlCapabilities { commands })
}

fn renderer_control_command_from_client(
    command: &ClientRendererControlCommand,
) -> Result<RendererControlCommand, NakoError> {
    RendererControlCommand::parse(command.wire_value()).ok_or_else(|| NakoError::InvalidInput {
        message: format!("unsupported renderer command: {}", command.wire_value()),
    })
}

fn renderer_session_state_from_client(
    state: &ClientRendererSessionState,
) -> Result<RendererSessionState, NakoError> {
    RendererSessionState::parse(state.wire_value()).ok_or_else(|| NakoError::InvalidInput {
        message: format!("unsupported renderer session state: {}", state.wire_value()),
    })
}

fn renderer_command_state_from_client(
    state: &ClientRendererCommandState,
) -> Result<RendererCommandState, NakoError> {
    RendererCommandState::parse(state.wire_value()).ok_or_else(|| NakoError::InvalidInput {
        message: format!("unsupported renderer command state: {}", state.wire_value()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_media_capabilities_map_all_current_flat_fields() {
        let capabilities = client_playback_capabilities(ClientPlaybackCapabilitiesDto {
            direct_play: false,
            device_family: Some("browser_chromium".to_owned()),
            profile_version: Some(1),
            containers: vec!["mp4".to_owned(), "webm".to_owned()],
            video_codecs: vec!["h264".to_owned(), "hevc".to_owned()],
            audio_codecs: vec!["aac".to_owned(), "opus".to_owned()],
            max_video_bitrate: Some(8_000_000),
            max_width: Some(1920),
            max_height: Some(1080),
            max_audio_channels: Some(2),
            supports_hdr: Some(false),
            supports_subtitles: Some(true),
            hls_variant_policy: Some(ClientHlsVariantPolicy::Adaptive),
            hls_segment_container: Some(ClientHlsSegmentContainer::Fmp4),
        });

        assert!(!capabilities.direct_play);
        assert_eq!(
            capabilities.device_family.as_deref(),
            Some("browser_chromium")
        );
        assert_eq!(capabilities.profile_version, Some(1));
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
}
