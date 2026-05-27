use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use nako_api::public_client::{
    ClientPlaybackCapabilitiesDto, ClientPlaybackTargetKind, ClientPlaybackTargetNetworkScope,
    ClientPlaybackTargetTransportAuth, ClientRendererCommandState,
    ClientRendererControlCapabilitiesDto, ClientRendererControlCommand, ClientRendererSessionState,
    RendererCommandCompletionRequest, RendererCommandPollResponse, RendererHeartbeatRequest,
    RendererPlayCommandRequest, RendererPlayCommandResponse, RendererRegistrationRequest,
    RendererSessionResponse, RendererSessionsResponse, page_info_from_request,
    renderer_command_poll_response_from_record, renderer_command_response_from_record,
    renderer_play_command_response_from_records, renderer_session_response_from_record,
    renderer_session_to_dto,
};
use nako_core::{
    AuthenticatedPrincipal, MediaSourceId, NakoError, PlaybackTargetKind,
    PlaybackTargetNetworkScope, PlaybackTargetTransportAuth, RendererCommandId,
    RendererCommandState, RendererControlCapabilities, RendererControlCommand, RendererSessionId,
    RendererSessionState,
};
use nako_playback::ClientPlaybackCapabilities;
use tracing::instrument;

use crate::app::{
    NakoApp,
    casting::PlayOnRendererRequest as AppPlayOnRendererRequest,
    renderer::{
        CompleteRendererCommandRequest as AppCompleteRendererCommandRequest,
        RegisterRendererRequest as AppRegisterRendererRequest,
        RendererHeartbeatUpdate as AppRendererHeartbeatUpdate,
    },
};

use super::{
    access::{RequiredLibraryAccess, require_source_access},
    error::ApiResult,
    query::PageQuery,
};

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

    Ok(Json(renderer_command_poll_response_from_record(command)))
}

#[instrument(skip(app, principal, request))]
async fn play_on_renderer(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(renderer_session_id): Path<RendererSessionId>,
    Json(request): Json<RendererPlayCommandRequest>,
) -> ApiResult<Json<RendererPlayCommandResponse>> {
    let source_id = parse_media_source_id(&request.source_id)?;
    require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;

    let output = app
        .casting()
        .play_on_renderer(AppPlayOnRendererRequest {
            principal,
            renderer_session_id,
            source_id,
            position_ms: request.position_ms,
        })
        .await?;

    Ok(Json(renderer_play_command_response_from_records(
        output.command,
        output.session,
    )))
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
    ClientPlaybackCapabilities {
        direct_play: capabilities.direct_play,
        containers: capabilities.containers,
        video_codecs: capabilities.video_codecs,
        audio_codecs: capabilities.audio_codecs,
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
