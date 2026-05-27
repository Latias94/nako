use nako_api::public_client::{
    ClientPlaybackCapabilitiesDto, ClientPlaybackTargetKind, ClientPlaybackTargetNetworkScope,
    ClientPlaybackTargetTransportAuth, ClientRendererCommandState,
    ClientRendererControlCapabilitiesDto, ClientRendererControlCommand, ClientRendererSessionState,
    ErrorResponse, RendererCommandCompletionRequest, RendererCommandPollResponse,
    RendererRegistrationRequest, RendererSessionResponse, RendererSessionsResponse,
};
use nako_core::{RendererControlCommand, RendererSessionId, UserPrincipalId};

use crate::app::renderer::QueueRendererCommandRequest;

use super::*;

#[tokio::test]
async fn nako_renderer_registers_heartbeats_lists_and_polls_commands() {
    let (_temp, app, source, _store) =
        app_with_media_source_config("movie.mp4", b"movie bytes", |_| {}).await;
    let router = build_router(app.clone());
    let registration = renderer_registration_request("Living Room Desktop");

    let registered: RendererSessionResponse =
        request_body_json(&router, Method::POST, "/renderers", &registration).await;

    assert_eq!(registered.renderer.display_name, "Living Room Desktop");
    assert_eq!(
        registered.renderer.target_kind,
        ClientPlaybackTargetKind::NakoRemoteClient
    );
    assert_eq!(
        registered.renderer.state,
        ClientRendererSessionState::Online
    );
    assert!(
        registered
            .renderer
            .control_capabilities
            .commands
            .contains(&ClientRendererControlCommand::Play)
    );

    let serialized = serde_json::to_value(&registered).unwrap().to_string();
    assert!(!serialized.contains("principal"));
    assert!(!serialized.contains("payload_json"));

    let renderers: RendererSessionsResponse =
        request_json(&router, Method::GET, "/renderers").await;
    assert_eq!(renderers.renderers.len(), 1);
    assert_eq!(renderers.renderers[0].id, registered.renderer.id);
    assert_eq!(renderers.page.returned, 1);

    let renderer_session_id = registered.renderer.id.parse::<RendererSessionId>().unwrap();
    let heartbeat = nako_api::public_client::RendererHeartbeatRequest {
        state: ClientRendererSessionState::Online,
        media_capabilities: Some(ClientPlaybackCapabilitiesDto {
            direct_play: true,
            containers: vec!["mp4".to_owned(), "mkv".to_owned()],
            video_codecs: vec!["h264".to_owned(), "hevc".to_owned()],
            audio_codecs: vec!["aac".to_owned(), "opus".to_owned()],
        }),
        control_capabilities: Some(registration.control_capabilities.clone()),
        ttl_ms: Some(120_000),
    };
    let heartbeat_response: RendererSessionResponse = request_body_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/heartbeat"),
        &heartbeat,
    )
    .await;
    assert_eq!(
        heartbeat_response
            .renderer
            .media_capabilities
            .as_ref()
            .unwrap()
            .containers,
        vec!["mp4", "mkv"]
    );

    let queued = app
        .renderer()
        .queue_renderer_command(QueueRendererCommandRequest {
            renderer_session_id,
            controlling_principal_id: UserPrincipalId::local_admin(),
            command: RendererControlCommand::Play,
            item_id: Some(source.item_id),
            source_id: Some(source.id),
            playback_session_id: None,
            position_ms: Some(0),
            volume_percent: None,
            payload_json: None,
        })
        .await
        .unwrap();

    let polled: RendererCommandPollResponse = request_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/commands/next"),
    )
    .await;
    let polled_command = polled.command.expect("queued command is delivered");
    assert_eq!(polled_command.id, queued.id.to_string());
    assert_eq!(polled_command.command, ClientRendererControlCommand::Play);
    assert_eq!(polled_command.state, ClientRendererCommandState::Delivered);
    assert_eq!(polled_command.source_id, Some(source.id.to_string()));

    let completed: nako_api::public_client::RendererCommandResponse = request_body_json(
        &router,
        Method::POST,
        &format!(
            "/renderers/{renderer_session_id}/commands/{}/complete",
            polled_command.id
        ),
        &RendererCommandCompletionRequest {
            state: ClientRendererCommandState::Acknowledged,
            failure_message: None,
        },
    )
    .await;
    assert_eq!(
        completed.command.state,
        ClientRendererCommandState::Acknowledged
    );
}

#[tokio::test]
async fn public_renderer_registration_rejects_external_cast_protocol_targets() {
    let (_temp, app, _source, _store) =
        app_with_media_source_config("movie.mp4", b"movie bytes", |_| {}).await;
    let router = build_router(app);
    let mut registration = renderer_registration_request("Chromecast Adapter");
    registration.target_kind = ClientPlaybackTargetKind::Chromecast;
    registration.transport_auth = ClientPlaybackTargetTransportAuth::CastTicket;

    let response = response_body_json(&router, Method::POST, "/renderers", &registration).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: ErrorResponse = body_json(response).await;
    assert_eq!(body.code, "unsupported");
}

fn renderer_registration_request(display_name: &str) -> RendererRegistrationRequest {
    RendererRegistrationRequest {
        display_name: display_name.to_owned(),
        target_kind: ClientPlaybackTargetKind::NakoRemoteClient,
        network_scope: ClientPlaybackTargetNetworkScope::Local,
        transport_auth: ClientPlaybackTargetTransportAuth::Bearer,
        media_capabilities: Some(ClientPlaybackCapabilitiesDto {
            direct_play: true,
            containers: vec!["mp4".to_owned()],
            video_codecs: vec!["h264".to_owned()],
            audio_codecs: vec!["aac".to_owned()],
        }),
        control_capabilities: ClientRendererControlCapabilitiesDto {
            commands: vec![
                ClientRendererControlCommand::Play,
                ClientRendererControlCommand::Pause,
                ClientRendererControlCommand::Resume,
                ClientRendererControlCommand::Seek,
                ClientRendererControlCommand::Stop,
            ],
        },
        ttl_ms: Some(120_000),
    }
}
