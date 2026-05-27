use nako_core::{
    AuthenticatedPrincipal, MediaSourceId, NakoError, PlaybackTargetId, RendererCommandRecord,
    RendererControlCommand, RendererSessionId, RendererSessionRecord, Result,
};
use nako_playback::{ClientPlaybackCapabilities, PlaybackTarget};
use serde::{Deserialize, Serialize};

use super::{
    PlaybackAppService, RendererAppService,
    playback::{RendererPlaybackTransportPlan, StartRendererPlaybackSessionRequest},
    renderer::QueueRendererCommandRequest,
};

#[derive(Clone, Debug)]
pub(crate) struct PlayOnRendererRequest {
    pub principal: AuthenticatedPrincipal,
    pub renderer_session_id: RendererSessionId,
    pub source_id: MediaSourceId,
    pub position_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlayOnRendererOutput {
    pub command: RendererCommandRecord,
    pub session: nako_core::PlaybackSessionRecord,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RendererCommandTransportPayload {
    pub transport: RendererPlaybackTransportPlan,
}

#[derive(Clone, Debug)]
pub(crate) struct CastingAppService {
    renderer: RendererAppService,
    playback: PlaybackAppService,
}

impl CastingAppService {
    pub(crate) fn new(renderer: RendererAppService, playback: PlaybackAppService) -> Self {
        Self { renderer, playback }
    }

    pub(crate) async fn play_on_renderer(
        &self,
        request: PlayOnRendererRequest,
    ) -> Result<PlayOnRendererOutput> {
        let renderer = self
            .renderer
            .get_controllable_renderer(&request.principal.principal_id, request.renderer_session_id)
            .await?;
        if !renderer
            .control_capabilities
            .supports(RendererControlCommand::Play)
        {
            return Err(NakoError::Forbidden {
                message: format!(
                    "renderer session {} does not support play command",
                    renderer.id
                ),
            });
        }

        let target = renderer_playback_target(&renderer)?;
        let playback = self
            .playback
            .start_renderer_playback_session(StartRendererPlaybackSessionRequest {
                principal: request.principal.clone(),
                source_id: request.source_id,
                target,
            })
            .await?;
        let command = self
            .renderer
            .queue_renderer_command(QueueRendererCommandRequest {
                renderer_session_id: request.renderer_session_id,
                controlling_principal_id: request.principal.principal_id.clone(),
                command: RendererControlCommand::Play,
                item_id: Some(playback.session.item_id),
                source_id: Some(request.source_id),
                playback_session_id: Some(playback.session.id),
                position_ms: request.position_ms,
                volume_percent: None,
                payload_json: Some(renderer_command_transport_payload_json(
                    &playback.transport,
                )?),
            })
            .await?;
        self.renderer
            .attach_playback_session(
                &request.principal.principal_id,
                request.renderer_session_id,
                playback.session.id,
            )
            .await?;

        Ok(PlayOnRendererOutput {
            command,
            session: playback.session,
        })
    }
}

pub(crate) fn renderer_command_transport_payload(
    command: &RendererCommandRecord,
) -> Option<RendererCommandTransportPayload> {
    command
        .payload_json
        .as_deref()
        .and_then(|payload| serde_json::from_str(payload).ok())
}

fn renderer_command_transport_payload_json(
    transport: &RendererPlaybackTransportPlan,
) -> Result<String> {
    serde_json::to_string(&RendererCommandTransportPayload {
        transport: transport.clone(),
    })
    .map_err(|err| NakoError::InvalidInput {
        message: format!("renderer command transport payload could not be serialized: {err}"),
    })
}

fn renderer_playback_target(renderer: &RendererSessionRecord) -> Result<PlaybackTarget> {
    Ok(PlaybackTarget {
        id: PlaybackTargetId::new(),
        kind: renderer.target_kind,
        display_name: renderer.display_name.clone(),
        network_scope: renderer.network_scope,
        transport_auth: renderer.transport_auth,
        media_capabilities: renderer_media_capabilities(renderer)?,
        control_capabilities: renderer.control_capabilities.clone(),
    })
}

fn renderer_media_capabilities(
    renderer: &RendererSessionRecord,
) -> Result<ClientPlaybackCapabilities> {
    let Some(value) = renderer.media_capabilities_json.as_deref() else {
        return Ok(ClientPlaybackCapabilities::default());
    };

    serde_json::from_str(value).map_err(|err| NakoError::InvalidInput {
        message: format!("renderer media capabilities are invalid: {err}"),
    })
}
