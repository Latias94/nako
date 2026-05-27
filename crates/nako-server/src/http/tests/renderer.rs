use nako_api::{
    admin::{
        AdminRendererAdapterKind, AdminRendererAdapterStatus, AdminRendererControlPlane,
        AdminRendererDiscoveryMode, AdminRendererMediaTransport, AdminRendererReadinessReason,
        AdminRendererReadinessStatus, AdminRendererRuntimeDiagnosticsResponse,
    },
    public_client::{
        ClientPlaybackCapabilitiesDto, ClientPlaybackTargetKind, ClientPlaybackTargetNetworkScope,
        ClientPlaybackTargetTransportAuth, ClientRendererCommandState,
        ClientRendererControlCapabilitiesDto, ClientRendererControlCommand,
        ClientRendererSessionState, ErrorResponse, RendererCommandCompletionRequest,
        RendererCommandPollResponse, RendererPlayCommandRequest, RendererPlayCommandResponse,
        RendererRegistrationRequest, RendererSessionResponse, RendererSessionsResponse,
    },
};
use nako_core::{
    RendererCommandListFilter, RendererControlCommand, RendererSessionId,
    RendererSessionRepository, UserPrincipalId,
};

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
async fn renderer_play_command_creates_playback_session_and_queues_command() {
    let (_temp, app, source, _store) =
        app_with_media_source_config("movie.mp4", b"movie bytes", |_| {}).await;
    let router = build_router(app.clone());
    let registered: RendererSessionResponse = request_body_json(
        &router,
        Method::POST,
        "/renderers",
        &renderer_registration_request("Living Room Desktop"),
    )
    .await;
    let renderer_session_id = registered.renderer.id.parse::<RendererSessionId>().unwrap();

    let play: RendererPlayCommandResponse = request_body_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/commands/play"),
        &RendererPlayCommandRequest {
            source_id: source.id.to_string(),
            position_ms: Some(37_000),
        },
    )
    .await;

    assert_eq!(play.session.source_id, source.id.to_string());
    assert_eq!(play.session.item_id, source.item_id.to_string());
    assert_eq!(play.command.command, ClientRendererControlCommand::Play);
    assert_eq!(play.command.state, ClientRendererCommandState::Queued);
    assert_eq!(
        play.command.playback_session_id.as_deref(),
        Some(play.session.id.as_str())
    );
    assert_eq!(play.command.position_ms, Some(37_000));

    let attached = app
        .renderer()
        .get_controllable_renderer(&UserPrincipalId::local_admin(), renderer_session_id)
        .await
        .unwrap();
    assert_eq!(
        attached.active_playback_session_id.map(|id| id.to_string()),
        Some(play.session.id.clone())
    );

    let polled: RendererCommandPollResponse = request_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/commands/next"),
    )
    .await;
    let command = polled.command.expect("play command is queued for renderer");
    assert_eq!(command.id, play.command.id);
    assert_eq!(command.state, ClientRendererCommandState::Delivered);
    assert_eq!(command.playback_session_id, Some(play.session.id));
}

#[tokio::test]
async fn renderer_play_command_denied_by_policy_creates_no_runtime_records() {
    let (_temp, app, source, store) =
        app_with_media_source_config("movie.mp4", b"movie bytes", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let router = public_client_router_with_principal(app, principal);
    let registered: RendererSessionResponse = request_body_json(
        &router,
        Method::POST,
        "/renderers",
        &renderer_registration_request("Viewer Desktop"),
    )
    .await;
    let renderer_session_id = registered.renderer.id.parse::<RendererSessionId>().unwrap();

    let response = response_body_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/commands/play"),
        &RendererPlayCommandRequest {
            source_id: source.id.to_string(),
            position_ms: None,
        },
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body: ErrorResponse = body_json(response).await;
    assert_eq!(body.code, "forbidden");
    assert!(
        body.message.contains("remote_control"),
        "expected remote_control denial, got {}",
        body.message
    );
    assert!(
        store
            .list_playback_sessions(
                PlaybackSessionListFilter::default(),
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_renderer_commands(
                RendererCommandListFilter {
                    renderer_session_id: Some(renderer_session_id),
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_transcode_sessions(
                TranscodeSessionListFilter::default(),
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn renderer_play_command_currently_rejects_remux_decision_without_runtime_records() {
    let (_temp, app, source, store) =
        app_with_media_source_config("renderer-remux-gap.mkv", b"movie bytes", |_| {}).await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let router = build_router(app);
    let registered: RendererSessionResponse = request_body_json(
        &router,
        Method::POST,
        "/renderers",
        &renderer_registration_request("Remux Gap Desktop"),
    )
    .await;
    let renderer_session_id = registered.renderer.id.parse::<RendererSessionId>().unwrap();

    let response = response_body_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/commands/play"),
        &RendererPlayCommandRequest {
            source_id: source.id.to_string(),
            position_ms: None,
        },
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: ErrorResponse = body_json(response).await;
    assert_eq!(body.code, "unsupported");
    assert!(
        body.message.contains("direct-play decision"),
        "expected current direct-only renderer gap, got {}",
        body.message
    );
    assert_renderer_play_gap_created_no_runtime_records(&store, source.id, renderer_session_id)
        .await;
}

#[tokio::test]
async fn renderer_play_command_currently_rejects_hls_decision_without_runtime_records() {
    let (_temp, app, source, store) =
        app_with_media_source_config("renderer-hls-gap.mp4", b"movie bytes", |_| {}).await;
    let router = build_router(app);
    let mut registration = renderer_registration_request("HLS Gap Desktop");
    registration.media_capabilities = Some(ClientPlaybackCapabilitiesDto {
        direct_play: false,
        containers: vec!["mp4".to_owned()],
        video_codecs: vec!["h264".to_owned()],
        audio_codecs: vec!["aac".to_owned()],
    });
    let registered: RendererSessionResponse =
        request_body_json(&router, Method::POST, "/renderers", &registration).await;
    let renderer_session_id = registered.renderer.id.parse::<RendererSessionId>().unwrap();

    let response = response_body_json(
        &router,
        Method::POST,
        &format!("/renderers/{renderer_session_id}/commands/play"),
        &RendererPlayCommandRequest {
            source_id: source.id.to_string(),
            position_ms: None,
        },
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: ErrorResponse = body_json(response).await;
    assert_eq!(body.code, "unsupported");
    assert!(
        body.message.contains("direct-play decision"),
        "expected current direct-only renderer gap, got {}",
        body.message
    );
    assert_renderer_play_gap_created_no_runtime_records(&store, source.id, renderer_session_id)
        .await;
}

#[tokio::test]
async fn admin_v1_playback_renderers_reports_safe_diagnostics_and_adapter_readiness() {
    let (_temp, app, _source, _store) =
        app_with_media_source_config("movie.mp4", b"movie bytes", |_| {}).await;
    let router = build_router(app);
    let registered: RendererSessionResponse = request_body_json(
        &router,
        Method::POST,
        "/renderers",
        &renderer_registration_request("Living Room Desktop"),
    )
    .await;

    let diagnostics: AdminRendererRuntimeDiagnosticsResponse =
        request_json(&router, Method::GET, "/admin/v1/playback/renderers").await;

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        diagnostics.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(
        diagnostics.readiness.status,
        AdminRendererReadinessStatus::Ready
    );
    assert_eq!(
        diagnostics.readiness.reason,
        AdminRendererReadinessReason::RendererRepositoryReady
    );
    assert_eq!(diagnostics.summary.returned_sessions, 1);
    assert_eq!(diagnostics.summary.online_sessions, 1);
    assert_eq!(diagnostics.summary.offline_sessions, 0);
    assert_eq!(diagnostics.summary.revoked_sessions, 0);
    assert_eq!(diagnostics.summary.expired_sessions, 0);
    assert_eq!(diagnostics.summary.active_playback_sessions, 0);
    assert_eq!(diagnostics.page.returned, 1);

    let nako_adapter = diagnostics
        .adapters
        .iter()
        .find(|adapter| adapter.adapter == AdminRendererAdapterKind::NakoRemoteClient)
        .expect("nako remote client adapter readiness is reported");
    assert_eq!(nako_adapter.status, AdminRendererAdapterStatus::Ready);
    assert_eq!(
        nako_adapter.control_plane,
        AdminRendererControlPlane::PublicClientPolling
    );
    assert_eq!(
        nako_adapter.discovery,
        AdminRendererDiscoveryMode::ClientRegistration
    );
    assert_eq!(
        nako_adapter.media_transport,
        AdminRendererMediaTransport::AuthenticatedNakoClientStream
    );

    for planned in [
        AdminRendererAdapterKind::NakoRemoteClientCastSafeTransport,
        AdminRendererAdapterKind::Chromecast,
        AdminRendererAdapterKind::DlnaRenderer,
        AdminRendererAdapterKind::Airplay,
    ] {
        assert!(
            diagnostics
                .adapters
                .iter()
                .any(|adapter| adapter.adapter == planned
                    && adapter.status == AdminRendererAdapterStatus::Planned),
            "expected planned adapter diagnostics for {planned:?}"
        );
    }

    let session = diagnostics
        .sessions
        .iter()
        .find(|session| session.id.to_string() == registered.renderer.id)
        .expect("registered renderer is listed for admin diagnostics");
    assert_eq!(session.display_name, "Living Room Desktop");
    assert_eq!(
        session.target_kind,
        nako_core::PlaybackTargetKind::NakoRemoteClient
    );
    assert_eq!(
        session.transport_auth,
        nako_core::PlaybackTargetTransportAuth::Bearer
    );
    assert!(session.direct_play_supported);
    assert!(session.has_media_capabilities);
    assert!(
        session
            .supported_commands
            .contains(&RendererControlCommand::Play)
    );

    let body = serde_json::to_string(&diagnostics).unwrap();
    for forbidden in [
        "principal",
        "payload_json",
        "media_capabilities_json",
        "source_locator",
        "local_path",
        "bearer_token",
        "access_token",
        "token_value",
    ] {
        assert!(
            !body.contains(forbidden),
            "renderer diagnostics leaked forbidden term: {forbidden}"
        );
    }
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

#[tokio::test]
async fn public_renderer_registration_currently_rejects_nako_remote_cast_ticket_transport() {
    let (_temp, app, _source, _store) =
        app_with_media_source_config("movie.mp4", b"movie bytes", |_| {}).await;
    let router = build_router(app);
    let mut registration = renderer_registration_request("Cast Ticket Desktop");
    registration.transport_auth = ClientPlaybackTargetTransportAuth::CastTicket;

    let response = response_body_json(&router, Method::POST, "/renderers", &registration).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: ErrorResponse = body_json(response).await;
    assert_eq!(body.code, "unsupported");
    assert!(
        body.message
            .contains("requires bearer-authenticated Nako clients"),
        "expected current bearer-only renderer registration boundary, got {}",
        body.message
    );
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

async fn assert_renderer_play_gap_created_no_runtime_records(
    store: &NakoDatabase,
    source_id: MediaSourceId,
    renderer_session_id: RendererSessionId,
) {
    assert!(
        store
            .list_playback_sessions(
                PlaybackSessionListFilter::default(),
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_renderer_commands(
                RendererCommandListFilter {
                    renderer_session_id: Some(renderer_session_id),
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_transcode_sessions(
                TranscodeSessionListFilter {
                    source_id: Some(source_id),
                    kind: None,
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}
