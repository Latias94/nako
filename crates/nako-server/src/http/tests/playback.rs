use super::*;

fn process_backed_hls_playlist_readiness_timeout() -> std::time::Duration {
    // Full-suite HLS gates on Windows start many fake FFmpeg processes at once.
    // Keep this guard above that startup tail while still bounding a hang.
    let seconds = if cfg!(windows) { 180 } else { 60 };
    std::time::Duration::from_secs(seconds)
}

async fn wait_for_marker(marker: &std::path::Path) {
    for _ in 0..250 {
        if marker.exists() {
            return;
        }
        sleep(Duration::from_millis(20)).await;
    }
    panic!("remux fixture did not start before timeout: {marker:?}");
}

fn ticket_param(url: &str) -> &str {
    url.split_once('?')
        .and_then(|(_, query)| {
            query
                .split('&')
                .find_map(|part| part.strip_prefix("ticket="))
        })
        .expect("browser playback URL contains ticket query parameter")
}

fn sidecar_subtitle_stream(index: u32, language: &str, codec: &str) -> MediaStreamInfo {
    MediaStreamInfo {
        index,
        kind: MediaStreamKind::Subtitle,
        codec: Some(codec.to_owned()),
        language: Some(language.to_owned()),
        duration_ms: None,
        bit_rate: None,
        width: None,
        height: None,
        channels: None,
        sample_rate: None,
        technical: MediaStreamTechnicalFacts {
            origin: Some(MediaStreamOrigin::Sidecar),
            disposition: MediaStreamDisposition {
                default: true,
                ..MediaStreamDisposition::default()
            },
            ..MediaStreamTechnicalFacts::default()
        },
    }
}

fn multi_audio_probe() -> MediaProbeResult {
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
                bit_rate: Some(2_000_000),
                width: Some(1280),
                height: Some(720),
                channels: None,
                sample_rate: None,
                technical: Default::default(),
            },
            MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: Some("eng".to_owned()),
                duration_ms: None,
                bit_rate: Some(128_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
                technical: Default::default(),
            },
            MediaStreamInfo {
                index: 2,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: Some("jpn".to_owned()),
                duration_ms: None,
                bit_rate: Some(128_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
                technical: Default::default(),
            },
        ],
    }
}

fn multi_subtitle_probe() -> MediaProbeResult {
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
                bit_rate: Some(2_000_000),
                width: Some(1280),
                height: Some(720),
                channels: None,
                sample_rate: None,
                technical: Default::default(),
            },
            MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: Some("eng".to_owned()),
                duration_ms: None,
                bit_rate: Some(128_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
                technical: Default::default(),
            },
            sidecar_subtitle_stream(2, "eng", "vtt"),
            sidecar_subtitle_stream(3, "jpn", "vtt"),
        ],
    }
}

#[tokio::test]
async fn public_playback_profile_presets_route_returns_safe_authenticated_catalog() {
    let (_temp, app, _source, _store) =
        app_with_media_source_config("demo.mp4", b"media", |_| {}).await;
    let router =
        public_client_router_with_principal(app, AuthenticatedPrincipal::bootstrap_admin());

    let response = request_json::<nako_api::public_client::PlaybackProfilePresetsResponse>(
        &router,
        Method::GET,
        "/playback/profile-presets",
    )
    .await;
    let value = serde_json::to_value(&response).unwrap();
    let serialized = serde_json::to_string(&value).unwrap().to_ascii_lowercase();

    assert!(response.presets.len() >= 9);
    assert!(response.presets.iter().all(|preset| preset.family
        != nako_api::public_client::ClientPlaybackProfileFamily::Other("unknown".to_owned())));
    let chromium = response
        .presets
        .iter()
        .find(|preset| preset.device_family == "browser_chromium")
        .expect("browser_chromium preset should be returned");
    assert_eq!(chromium.profile_version, 1);
    assert!(chromium.direct_play);
    assert!(chromium.containers.iter().any(|value| value == "mp4"));
    assert!(chromium.video_codecs.iter().any(|value| value == "h264"));
    assert!(chromium.audio_codecs.iter().any(|value| value == "aac"));
    assert!(!chromium.supports_hdr);
    assert!(chromium.supports_subtitles);
    assert_eq!(
        chromium.hls_variant_policy,
        nako_api::public_client::ClientHlsVariantPolicy::Adaptive
    );
    assert_eq!(
        chromium.hls_segment_container,
        nako_api::public_client::ClientHlsSegmentContainer::Fmp4
    );

    for forbidden in [
        "unknown",
        "ffmpeg",
        "gpu",
        "hardware",
        "operator",
        "resource_pressure",
        "bearer",
        "token",
        "principal",
        "source_locator",
        "source_uri",
        "local_path",
        "output_path",
        "raw_locator",
        "operator_policy",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "public playback profile preset route leaked forbidden term: {forbidden}"
        );
    }
}

#[tokio::test]
async fn public_playback_profile_presets_route_requires_bearer_auth() {
    let (_temp, app, _source, _store) =
        app_with_media_source_config("demo.mp4", b"media", |_| {}).await;
    let router = build_router_with_auth(app, auth::InboundAuthState::bearer_token("secret"));

    let unauthenticated = response_for(&router, Method::GET, "/playback/profile-presets").await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauthenticated
            .headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap(),
        "Bearer"
    );

    let authenticated = request_json_with_bearer::<
        nako_api::public_client::PlaybackProfilePresetsResponse,
    >(&router, Method::GET, "/playback/profile-presets", "secret")
    .await;
    assert!(!authenticated.presets.is_empty());
}

async fn hls_playlist_body_and_transcode_session(
    router: &Router,
    store: &NakoDatabase,
    uri: &str,
) -> (String, TranscodeSessionId) {
    let response = response_for(router, Method::GET, uri).await;
    let status = response.status();
    if status != StatusCode::OK {
        let body = String::from_utf8_lossy(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .into_owned();
        panic!("expected hls playlist {uri} to return OK, got {status}: {body}");
    }
    let playback_session_id: PlaybackSessionId = response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("hls playlist should expose playback session id")
        .parse()
        .unwrap();
    let playlist = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let playback_session = store
        .get_playback_session(playback_session_id)
        .await
        .unwrap()
        .expect("hls playlist header should point at durable playback session");
    (
        playlist,
        playback_session
            .transcode_session_id
            .expect("hls playback session should link transcode session"),
    )
}

fn assert_audio_default(playlist: &str, default_language: &str) {
    let audio_lines = playlist
        .lines()
        .filter(|line| line.starts_with("#EXT-X-MEDIA:TYPE=AUDIO"))
        .collect::<Vec<_>>();

    assert_eq!(audio_lines.len(), 2);
    assert_eq!(
        audio_lines
            .iter()
            .filter(|line| line.contains("DEFAULT=YES"))
            .count(),
        1
    );
    assert!(
        audio_lines.iter().any(|line| {
            line.contains(&format!("NAME=\"{default_language}\"")) && line.contains("DEFAULT=YES")
        }),
        "playlist did not default {default_language}: {playlist}"
    );
}

fn assert_subtitle_default(playlist: &str, default_language: &str) {
    let subtitle_lines = playlist
        .lines()
        .filter(|line| line.starts_with("#EXT-X-MEDIA:TYPE=SUBTITLES"))
        .collect::<Vec<_>>();

    assert!(
        !subtitle_lines.is_empty(),
        "playlist did not contain subtitle renditions: {playlist}"
    );
    assert_eq!(
        subtitle_lines
            .iter()
            .filter(|line| line.contains("DEFAULT=YES"))
            .count(),
        1
    );
    assert!(
        subtitle_lines.iter().any(|line| {
            line.contains(&format!("NAME=\"{default_language}\"")) && line.contains("DEFAULT=YES")
        }),
        "playlist did not default {default_language}: {playlist}"
    );
}

async fn latest_playback_session_for_source(
    store: &NakoDatabase,
    source_id: MediaSourceId,
    mode: PlaybackSessionMode,
) -> PlaybackSessionRecord {
    for _ in 0..250 {
        if let Some(session) = store
            .list_playback_sessions(
                PlaybackSessionListFilter {
                    principal_id: None,
                    source_id: Some(source_id),
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .into_iter()
            .find(|session| session.mode == mode)
        {
            return session;
        }

        sleep(Duration::from_millis(20)).await;
    }

    panic!("playback route should persist a matching playback session")
}

async fn wait_for_transcode_state(
    store: &NakoDatabase,
    session_id: TranscodeSessionId,
    expected: TranscodeSessionState,
) {
    let mut last_state = None;
    for _ in 0..250 {
        let state = store
            .get_transcode_session(session_id)
            .await
            .unwrap()
            .unwrap()
            .state;
        if state == expected {
            return;
        }
        last_state = Some(state);
        sleep(Duration::from_millis(20)).await;
    }

    panic!("transcode session {session_id} did not reach {expected:?}; last state: {last_state:?}");
}

async fn wait_for_hls_segment_ok(router: &Router, uri: &str) -> axum::response::Response {
    let mut last_status = None;
    for _ in 0..250 {
        let response = response_for(router, Method::GET, uri).await;
        let status = response.status();
        if status == StatusCode::OK {
            return response;
        }
        if status != StatusCode::CONFLICT {
            panic!("expected hls segment {uri} to become OK, got {status}");
        }
        last_status = Some(status);
        sleep(Duration::from_millis(20)).await;
    }

    panic!("hls segment {uri} did not become OK; last status: {last_status:?}");
}

#[tokio::test]
async fn playback_decision_and_direct_stream_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mp4"), b"0123456789").unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///demo.mp4".to_owned(),
        file_name: "demo.mp4".to_owned(),
        size_bytes: Some(10),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
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
                        technical: Default::default(),
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
                        technical: Default::default(),
                    },
                ],
            },
        )
        .await
        .unwrap();
    let router = build_router(app);

    let decision = request_json::<nako_api::public_client::PlaybackDecisionResponse>(
        &router,
        Method::GET,
        &format!("/sources/{}/playback/decision", source.id),
    )
    .await;
    let decision_json = request_json::<serde_json::Value>(
        &router,
        Method::GET,
        &format!("/sources/{}/playback/decision?direct_play=false", source.id),
    )
    .await;
    let response = router
        .clone()
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

    assert_eq!(
        decision.decision.mode,
        nako_api::public_client::ClientPlaybackMode::DirectPlay
    );
    assert_eq!(
        decision.decision.report.selected_mode,
        nako_api::public_client::ClientPlaybackMode::DirectPlay
    );
    assert!(decision.decision.report.direct_play.supported);
    assert_eq!(
        decision_json["decision"]["report"]["direct_play"]["reasons"][0],
        "direct_play_disabled"
    );
    assert_eq!(
        decision_json["decision"]["report"]["selection_reasons"][0],
        "direct_play_disabled"
    );
    assert!(decision_json["source"].get("locator").is_none());
    assert!(
        decision_json["decision"]["transcode_plan"]
            .get("input_locator")
            .is_none()
    );
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
async fn playback_decision_route_exposes_safe_selection_reasons_from_flat_capability_query() {
    let (_temp, app, source, store) =
        app_with_media_source_config("codec-gap.mp4", b"media", |_| {}).await;
    let mut probe = compatible_probe();
    probe.container = Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned());
    store.upsert_media_probe(source.id, &probe).await.unwrap();
    let router = build_router(app);

    let decision = request_json::<nako_api::public_client::PlaybackDecisionResponse>(
        &router,
        Method::GET,
        &format!(
            "/sources/{}/playback/decision?container=mp4&video_codec=vp9&audio_codec=aac",
            source.id
        ),
    )
    .await;
    let body = serde_json::to_string(&decision).unwrap();

    assert_eq!(
        decision.decision.mode,
        nako_api::public_client::ClientPlaybackMode::Transcode
    );
    assert_eq!(
        decision.decision.reason,
        nako_api::public_client::ClientPlaybackDecisionReason::SourceCodecsUnsupported
    );
    let video_codec_unsupported =
        nako_api::public_client::ClientPlaybackCompatibilityCondition::VideoCodecUnsupported;
    assert_eq!(
        decision.decision.report.selection_reasons,
        vec![video_codec_unsupported.clone()]
    );
    assert_eq!(
        decision.decision.report.selection_reason_details[0].condition,
        video_codec_unsupported
    );
    assert_eq!(
        decision.decision.report.selection_reason_details[0].summary,
        "Video codec unsupported"
    );
    assert!(
        decision.decision.report.selection_reason_details[0]
            .detail
            .contains("video codec")
    );
    assert!(
        decision
            .decision
            .report
            .direct_play
            .reasons
            .contains(&video_codec_unsupported)
    );
    assert!(
        decision
            .decision
            .report
            .direct_play
            .reason_details
            .iter()
            .any(|detail| {
                detail.condition == video_codec_unsupported
                    && detail.summary == "Video codec unsupported"
            })
    );
    assert!(decision.decision.report.transcode.supported);
    assert!(body.contains("Video codec unsupported"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("Bearer"));
    assert!(!body.contains("ffmpeg"));
}

#[tokio::test]
async fn playback_routes_require_play_library_access() {
    let (_temp, app, source, store) =
        app_with_media_source_config("access-denied.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Browse)
            .await;
    let router = public_client_router_with_principal(app, principal);

    let decision = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/playback/decision", source.id),
    )
    .await;
    let stream = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/stream", source.id),
    )
    .await;
    let remux = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/stream/remux?output_container=mp4", source.id),
    )
    .await;
    let hls = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/stream/hls/playlist.m3u8", source.id),
    )
    .await;

    assert_eq!(decision.status(), StatusCode::FORBIDDEN);
    let decision: ErrorResponse = body_json(decision).await;
    assert_eq!(decision.code, "forbidden");
    assert!(
        decision
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial, got {}",
        decision.message
    );
    assert_eq!(stream.status(), StatusCode::FORBIDDEN);
    let stream: ErrorResponse = body_json(stream).await;
    assert_eq!(stream.code, "forbidden");
    assert!(
        stream
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial, got {}",
        stream.message
    );
    assert_eq!(remux.status(), StatusCode::FORBIDDEN);
    let remux: ErrorResponse = body_json(remux).await;
    assert_eq!(remux.code, "forbidden");
    assert!(
        remux
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial, got {}",
        remux.message
    );
    assert_eq!(hls.status(), StatusCode::FORBIDDEN);
    let hls: ErrorResponse = body_json(hls).await;
    assert_eq!(hls.code, "forbidden");
    assert!(
        hls.message.contains("required Library Access level 'play'"),
        "expected library play access denial, got {}",
        hls.message
    );
}

#[tokio::test]
async fn subtitle_route_serves_sidecar_text_without_exposing_locator() {
    let (temp, router, source, store) = router_with_media_source("demo.mkv", b"media").await;
    fs::write(
        temp.path().join("demo.zh-hant.srt"),
        "1\n00:00:01,000 --> 00:00:02,000\nHello\n",
    )
    .unwrap();
    let mut probe = compatible_probe();
    probe
        .streams
        .push(sidecar_subtitle_stream(2, "zh-hant", "srt"));
    store.upsert_media_probe(source.id, &probe).await.unwrap();

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/sources/{}/subtitles/2", source.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/x-subrip; charset=utf-8")
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        std::str::from_utf8(&body).unwrap(),
        "1\n00:00:01,000 --> 00:00:02,000\nHello\n"
    );

    fs::remove_file(temp.path().join("demo.zh-hant.srt")).unwrap();
    let missing = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/subtitles/2", source.id),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    let error = body_json::<ErrorResponse>(missing).await;
    assert_eq!(error.code, "not_found");
    assert!(!error.message.contains("local:///"));
    assert!(!error.message.contains("demo.zh-hant.srt"));
}

#[tokio::test]
async fn subtitle_route_requires_play_library_access() {
    let (_temp, app, source, store) =
        app_with_media_source_config("subtitle-access.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Browse)
            .await;
    let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
    permissions.allow_media_playback = false;
    store
        .upsert_playback_policy(&PlaybackPolicy::user(
            principal.user_id,
            source.library_id,
            permissions,
            2,
        ))
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let response = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/subtitles/2", source.id),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(error.code, "forbidden");
    assert!(
        error
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial, got {}",
        error.message
    );
    assert!(!error.message.contains("media_playback"));
    assert!(!error.message.contains("subtitle_sidecar"));
}

#[tokio::test]
async fn playback_decision_returns_safe_target_and_policy_denial() {
    let (_temp, app, source, store) =
        app_with_media_source_config("policy-decision.mkv", b"media", |_| {}).await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
    permissions.allow_remux = false;
    store
        .upsert_playback_policy(&PlaybackPolicy::user(
            principal.user_id,
            source.library_id,
            permissions,
            2,
        ))
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let decision = request_json::<nako_api::public_client::PlaybackDecisionResponse>(
        &router,
        Method::GET,
        &format!(
            "/sources/{}/playback/decision?container=mp4&video_codec=h264&audio_codec=aac",
            source.id
        ),
    )
    .await;
    let body = serde_json::to_string(&decision).unwrap();

    assert_eq!(
        decision.target.kind,
        nako_api::public_client::ClientPlaybackTargetKind::Browser
    );
    assert_eq!(
        decision.target.transport_auth,
        nako_api::public_client::ClientPlaybackTargetTransportAuth::BrowserTicket
    );
    assert_eq!(
        decision.decision.mode,
        nako_api::public_client::ClientPlaybackMode::Denied
    );
    assert_eq!(
        decision.decision.reason,
        nako_api::public_client::ClientPlaybackDecisionReason::PolicyDenied
    );
    assert_eq!(
        decision.decision.report.selected_mode,
        nako_api::public_client::ClientPlaybackMode::Denied
    );
    assert_eq!(
        decision.decision.report.denial.as_ref().unwrap().permission,
        nako_api::public_client::ClientPlaybackPermission::Remux
    );
    let policy_denied = nako_api::public_client::ClientPlaybackCompatibilityCondition::PolicyDenied;
    assert_eq!(
        decision.decision.report.direct_play.reasons[0],
        policy_denied
    );
    assert_eq!(decision.decision.report.selection_reasons[0], policy_denied);
    assert_eq!(
        decision.decision.report.selection_reason_details[0].condition,
        policy_denied
    );
    assert_eq!(
        decision.decision.report.selection_reason_details[0].summary,
        "Playback policy denied"
    );
    assert!(
        decision.decision.report.selection_reason_details[0]
            .detail
            .contains("policy blocked")
    );
    assert_eq!(
        decision.decision.report.direct_play.reason_details[0].condition,
        policy_denied
    );
    let denial = decision.decision.denial.unwrap();
    assert_eq!(
        denial.permission,
        nako_api::public_client::ClientPlaybackPermission::Remux
    );
    assert_eq!(
        denial.reason,
        nako_api::public_client::ClientPlaybackPermissionDecisionReason::RemuxDisabled
    );
    assert!(body.contains("Playback policy denied"));
    assert!(!body.contains("user_id"));
    assert!(!body.contains("role"));
    assert!(!body.contains("policy_rows"));
    assert!(!body.contains("local:///"));
}

#[tokio::test]
async fn browser_playback_session_currently_has_no_renderer_session_surface() {
    let (_temp, app, source, store) =
        app_with_media_source_config("renderer-gap.mp4", b"0123456789", |_| {}).await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let router = build_router(app);

    let decision = request_json::<nako_api::public_client::PlaybackDecisionResponse>(
        &router,
        Method::GET,
        &format!(
            "/sources/{}/playback/decision?container=mp4&video_codec=h264&audio_codec=aac",
            source.id
        ),
    )
    .await;
    assert_eq!(
        decision.target.kind,
        nako_api::public_client::ClientPlaybackTargetKind::Browser
    );
    assert!(decision.target.control_capabilities.commands.is_empty());

    let response = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/stream", source.id),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let session_id = response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("direct stream should expose playback session id")
        .to_owned();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"0123456789");

    let session_response = request_json::<PlaybackSessionResponse>(
        &router,
        Method::GET,
        &format!("/playback/sessions/{session_id}"),
    )
    .await;
    let session_json = serde_json::to_value(&session_response).unwrap();
    assert_eq!(session_response.session.id, session_id);
    assert_eq!(
        session_response.session.mode,
        nako_api::public_client::ClientPlaybackSessionMode::Direct
    );
    assert_eq!(
        session_response.session.state,
        nako_api::public_client::ClientPlaybackSessionState::Active
    );
    assert!(session_json["session"].get("renderer_session_id").is_none());
    assert!(session_json["session"].get("target").is_none());
    assert!(session_json["session"].get("target_kind").is_none());
    assert!(
        session_json["session"]
            .get("control_capabilities")
            .is_none()
    );
    assert!(session_json["session"].get("supported_commands").is_none());
    assert!(session_json["session"].get("command_endpoint").is_none());

    let heartbeat = request_body_json::<PlaybackSessionResponse, _>(
        &router,
        Method::POST,
        &format!("/playback/sessions/{session_id}/heartbeat"),
        &nako_api::public_client::PlaybackSessionHeartbeatRequest {
            state: nako_api::public_client::ClientPlaybackSessionState::Paused,
            position_ms: Some(1_000),
            duration_ms: Some(10_000),
        },
    )
    .await;
    let heartbeat_json = serde_json::to_value(&heartbeat).unwrap();
    assert_eq!(
        heartbeat.session.state,
        nako_api::public_client::ClientPlaybackSessionState::Paused
    );
    assert!(
        heartbeat_json["session"]
            .get("renderer_session_id")
            .is_none()
    );
    assert!(heartbeat_json["session"].get("target").is_none());
    assert!(
        heartbeat_json["session"]
            .get("supported_commands")
            .is_none()
    );
}

#[tokio::test]
async fn browser_ticket_play_access_currently_allows_all_playback_modes() {
    let (_temp, app, source, store) =
        app_with_media_source_config("policy-gap.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let router = public_client_router_with_principal(app, principal);
    let modes = [
        (
            nako_api::public_client::BrowserPlaybackMode::Direct,
            nako_api::public_client::BrowserPlaybackUrlKind::Stream,
        ),
        (
            nako_api::public_client::BrowserPlaybackMode::Remux,
            nako_api::public_client::BrowserPlaybackUrlKind::Stream,
        ),
        (
            nako_api::public_client::BrowserPlaybackMode::Hls,
            nako_api::public_client::BrowserPlaybackUrlKind::Playlist,
        ),
    ];

    for (mode, expected_url_kind) in modes {
        let response =
            request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
                &router,
                Method::POST,
                &format!("/sources/{}/playback/browser-ticket", source.id),
                &nako_api::public_client::BrowserPlaybackTicketRequest {
                    mode: mode.clone(),
                    capabilities: None,
                    subtitle_stream_index: None,
                },
            )
            .await;

        assert_eq!(response.mode, mode);
        assert_eq!(response.urls.len(), 1);
        assert_eq!(response.urls[0].kind, expected_url_kind);
        assert!(response.urls[0].url.contains("ticket="));
    }
}

#[tokio::test]
async fn browser_ticket_respects_effective_playback_policy_before_issue() {
    let (_temp, app, source, store) =
        app_with_media_source_config("policy-denied.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
    permissions.allow_remux = false;
    store
        .upsert_playback_policy(&PlaybackPolicy::user(
            principal.user_id,
            source.library_id,
            permissions,
            2,
        ))
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let response = response_for_body_json(
        &router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &nako_api::public_client::BrowserPlaybackTicketRequest {
            mode: nako_api::public_client::BrowserPlaybackMode::Remux,
            capabilities: None,
            subtitle_stream_index: None,
        },
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn direct_stream_head_returns_headers_without_body() {
    let (_temp, router, source, _store) = router_with_media_source("demo.mp4", b"0123456789").await;

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
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
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
async fn direct_stream_route_records_playback_session_without_transcode_artifact() {
    let (_temp, router, source, store) =
        router_with_media_source("direct-session.mp4", b"0123456789").await;

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

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let session_id = response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("direct stream should expose a playback session id")
        .to_owned();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"2345");

    let session = store
        .get_playback_session(session_id.parse().unwrap())
        .await
        .unwrap()
        .expect("direct stream session header should point at durable playback session");
    assert_eq!(session.source_id, source.id);
    assert_eq!(session.item_id, source.item_id);
    assert_eq!(session.mode, PlaybackSessionMode::Direct);
    assert_eq!(session.state, PlaybackSessionState::Active);
    assert!(session.transcode_session_id.is_none());
    assert!(session.client_capabilities_json.is_some());

    let transcode_artifacts = store
        .list_transcode_sessions(
            TranscodeSessionListFilter {
                source_id: Some(source.id),
                kind: None,
                state: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert!(
        transcode_artifacts.is_empty(),
        "direct play must not create fake transcode artifacts"
    );
}

#[tokio::test]
async fn browser_playback_ticket_streams_direct_bytes_without_bearer() {
    let (_temp, app, source, store) =
        app_with_media_source_config("ticket.mp4", b"0123456789", |_| {}).await;
    let router = build_router_with_auth(app, auth::InboundAuthState::bearer_token("secret"));
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Direct,
        capabilities: Some(nako_api::public_client::BrowserPlaybackCapabilitiesDto {
            direct_play: Some(true),
            container: Some(vec!["mp4".to_owned()]),
            video_codec: Some(vec!["h264".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned()]),
            output_container: None,
            ..nako_api::public_client::BrowserPlaybackCapabilitiesDto::default()
        }),
        subtitle_stream_index: None,
    };

    let unauthenticated = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/stream", source.id),
    )
    .await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let ticket_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/sources/{}/playback/browser-ticket", source.id))
                .header(header::AUTHORIZATION, "Bearer secret")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&issue_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ticket_response.status(), StatusCode::OK);
    let ticket =
        body_json::<nako_api::public_client::BrowserPlaybackTicketResponse>(ticket_response).await;
    assert_eq!(
        ticket.mode,
        nako_api::public_client::BrowserPlaybackMode::Direct
    );
    assert_eq!(
        ticket.urls[0].kind,
        nako_api::public_client::BrowserPlaybackUrlKind::Stream
    );
    assert!(ticket.urls[0].url.contains("ticket="));
    assert!(!ticket.urls[0].url.contains("Bearer"));
    let playback_session_id = ticket
        .playback_session_id
        .as_deref()
        .expect("direct browser ticket exposes heartbeat session identity");
    assert!(!ticket.urls[0].url.contains("playback_session_id"));
    let ticket_token = ticket_param(&ticket.urls[0].url);
    assert!(ticket_token.starts_with("nako_bpt_"));
    assert!(!ticket_token.contains(&source.id.to_string()));

    let stream_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&ticket.urls[0].url)
                .header(header::RANGE, "bytes=2-5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(stream_response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        stream_response
            .headers()
            .get(PLAYBACK_SESSION_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(playback_session_id)
    );
    let bytes = to_bytes(stream_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&bytes[..], b"2345");
    let session = store
        .get_playback_session(playback_session_id.parse().unwrap())
        .await
        .unwrap()
        .expect("ticket response should point at a durable playback session");
    assert_eq!(session.source_id, source.id);
    assert_eq!(session.mode, PlaybackSessionMode::Direct);
    assert!(session.transcode_session_id.is_none());

    let heartbeat_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/playback/sessions/{playback_session_id}/heartbeat"
                ))
                .header(header::AUTHORIZATION, "Bearer secret")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&nako_api::public_client::PlaybackSessionHeartbeatRequest {
                        state: nako_api::public_client::ClientPlaybackSessionState::Active,
                        position_ms: Some(2),
                        duration_ms: Some(10),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(heartbeat_response.status(), StatusCode::OK);
    let heartbeat =
        body_json::<nako_api::public_client::PlaybackSessionResponse>(heartbeat_response).await;
    assert_eq!(heartbeat.session.id, playback_session_id);

    let invalid = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/stream?ticket=not-a-ticket", source.id),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    let error = body_json::<ErrorResponse>(invalid).await;
    assert_eq!(error.code, "unauthorized");
    assert!(!error.message.contains("not-a-ticket"));
}

#[tokio::test]
async fn browser_playback_ticket_streams_sidecar_subtitle_without_bearer() {
    let (temp, app, source, store) =
        app_with_media_source_config("ticket-subtitle.mkv", b"media", |_| {}).await;
    fs::write(
        temp.path().join("ticket-subtitle.en.vtt"),
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello\n",
    )
    .unwrap();
    let mut probe = compatible_probe();
    probe.streams.push(sidecar_subtitle_stream(2, "en", "vtt"));
    store.upsert_media_probe(source.id, &probe).await.unwrap();
    let router = build_router_with_auth(app, auth::InboundAuthState::bearer_token("secret"));
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Subtitle,
        capabilities: None,
        subtitle_stream_index: Some(2),
    };

    let unauthenticated = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/subtitles/2", source.id),
    )
    .await;
    assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

    let ticket_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/sources/{}/playback/browser-ticket", source.id))
                .header(header::AUTHORIZATION, "Bearer secret")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&issue_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ticket_response.status(), StatusCode::OK);
    let ticket =
        body_json::<nako_api::public_client::BrowserPlaybackTicketResponse>(ticket_response).await;

    assert_eq!(
        ticket.mode,
        nako_api::public_client::BrowserPlaybackMode::Subtitle
    );
    assert!(ticket.playback_session_id.is_none());
    assert_eq!(
        ticket.urls[0].kind,
        nako_api::public_client::BrowserPlaybackUrlKind::Subtitle
    );
    assert_eq!(ticket.urls[0].content_type, "text/vtt; charset=utf-8");
    assert!(ticket.urls[0].url.contains("/subtitles/2?ticket="));
    assert!(!ticket.urls[0].url.contains("Bearer"));
    let token = ticket_param(&ticket.urls[0].url);
    assert!(token.starts_with("nako_bpt_"));
    assert!(!token.contains(&source.id.to_string()));

    let subtitle = response_for(&router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(subtitle.status(), StatusCode::OK);
    let body = to_bytes(subtitle.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        std::str::from_utf8(&body).unwrap(),
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello\n"
    );

    let wrong_stream = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/subtitles/3?ticket={token}", source.id),
    )
    .await;
    assert_eq!(wrong_stream.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn subtitle_browser_playback_ticket_rejects_revocation_at_use() {
    let (temp, app, source, store) =
        app_with_media_source_config("ticket-subtitle-revoked.mkv", b"media", |_| {}).await;
    fs::write(
        temp.path().join("ticket-subtitle-revoked.en.vtt"),
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello\n",
    )
    .unwrap();
    let mut probe = compatible_probe();
    probe.streams.push(sidecar_subtitle_stream(2, "en", "vtt"));
    store.upsert_media_probe(source.id, &probe).await.unwrap();
    let play_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let play_user_id = play_principal.user_id;
    let play_router = public_client_router_with_principal(app, play_principal);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Subtitle,
        capabilities: None,
        subtitle_stream_index: Some(2),
    };
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &play_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(play_user_id),
            source.library_id,
        )
        .await
        .unwrap();

    let revoked = response_for(&play_router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(revoked.status(), StatusCode::FORBIDDEN);
    let revoked: ErrorResponse = body_json(revoked).await;
    assert_eq!(revoked.code, "forbidden");
    assert!(
        revoked
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial after revocation, got {}",
        revoked.message
    );
    assert!(!revoked.message.contains("subtitle_sidecar"));
}

#[tokio::test]
async fn browser_playback_ticket_response_exposes_only_control_session_identity() {
    let (_temp, app, source, _store) =
        app_with_media_source_config("ticket-scope.mp4", b"0123456789", |_| {}).await;
    let router = build_router(app);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Direct,
        capabilities: None,
        subtitle_stream_index: None,
    };

    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    let value = serde_json::to_value(&ticket).unwrap();
    assert!(value.get("renderer_session_id").is_none());
    assert_eq!(
        value["playback_session_id"].as_str(),
        ticket.playback_session_id.as_deref()
    );
    assert!(ticket.playback_session_id.is_some());
    assert!(value.get("renderer_command_id").is_none());
    assert!(value.get("network_scope").is_none());
    assert!(value.get("transport_auth").is_none());
    assert!(!ticket.urls[0].url.contains("playback_session_id"));

    let body = serde_json::to_string(&ticket).unwrap();
    assert!(!body.contains("renderer_session"));
    assert!(!body.contains("network_scope"));
    assert!(!body.contains("cast_ticket"));
    assert!(body.contains("nako_bpt_"));
}

#[tokio::test]
async fn browser_playback_ticket_heartbeat_requires_owner_and_play_access() {
    let (_temp, app, source, store) =
        app_with_media_source_config("ticket-heartbeat.mp4", b"0123456789", |_| {}).await;
    let owner_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let owner_user_id = owner_principal.user_id;
    let owner_router = public_client_router_with_principal(app.clone(), owner_principal);
    let other_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let other_router = public_client_router_with_principal(app.clone(), other_principal);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Direct,
        capabilities: None,
        subtitle_stream_index: None,
    };
    let heartbeat = nako_api::public_client::PlaybackSessionHeartbeatRequest {
        state: nako_api::public_client::ClientPlaybackSessionState::Active,
        position_ms: Some(1),
        duration_ms: Some(10),
    };

    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &owner_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;
    let playback_session_id = ticket
        .playback_session_id
        .as_deref()
        .expect("direct ticket exposes heartbeat session identity");

    let owner = response_body_json(
        &owner_router,
        Method::POST,
        &format!("/playback/sessions/{playback_session_id}/heartbeat"),
        &heartbeat,
    )
    .await;
    assert_eq!(owner.status(), StatusCode::OK);

    let non_owner = response_body_json(
        &other_router,
        Method::POST,
        &format!("/playback/sessions/{playback_session_id}/heartbeat"),
        &heartbeat,
    )
    .await;
    assert_eq!(non_owner.status(), StatusCode::NOT_FOUND);

    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(owner_user_id),
            source.library_id,
        )
        .await
        .unwrap();
    let revoked_owner = response_body_json(
        &owner_router,
        Method::POST,
        &format!("/playback/sessions/{playback_session_id}/heartbeat"),
        &heartbeat,
    )
    .await;
    assert_eq!(revoked_owner.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn browser_playback_ticket_rejects_browse_only_access_and_revocation_at_use() {
    let (_temp, app, source, store) =
        app_with_media_source_config("ticket-access.mp4", b"0123456789", |_| {}).await;
    let browse_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Browse)
            .await;
    let browse_router = public_client_router_with_principal(app.clone(), browse_principal);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Direct,
        capabilities: None,
        subtitle_stream_index: None,
    };

    let browse_only = response_body_json(
        &browse_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;
    assert_eq!(browse_only.status(), StatusCode::FORBIDDEN);
    let browse_only: ErrorResponse = body_json(browse_only).await;
    assert_eq!(browse_only.code, "forbidden");
    assert!(
        browse_only
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial, got {}",
        browse_only.message
    );

    let play_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let play_user_id = play_principal.user_id;
    let play_router = public_client_router_with_principal(app, play_principal);
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &play_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(play_user_id),
            source.library_id,
        )
        .await
        .unwrap();

    let revoked = response_for(&play_router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(revoked.status(), StatusCode::FORBIDDEN);
    let revoked: ErrorResponse = body_json(revoked).await;
    assert_eq!(revoked.code, "forbidden");
    assert!(
        revoked
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial after revocation, got {}",
        revoked.message
    );
}

#[tokio::test]
async fn remux_browser_playback_ticket_rejects_revocation_at_use() {
    let script_root = tempfile::tempdir().unwrap();
    let marker = script_root.path().join("remux.started");
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "ticket_remux_revoked", true, &marker);
    let (_temp, app, source, store) =
        app_with_media_source_config("ticket-remux.mkv", b"media", |config| {
            config.ffmpeg_path = ffmpeg_path.clone();
        })
        .await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let play_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let play_user_id = play_principal.user_id;
    let play_router = public_client_router_with_principal(app, play_principal);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Remux,
        capabilities: Some(nako_api::public_client::BrowserPlaybackCapabilitiesDto {
            direct_play: Some(false),
            container: Some(vec!["mkv".to_owned()]),
            video_codec: Some(vec!["h264".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned()]),
            output_container: Some(nako_api::public_client::BrowserPlaybackOutputContainer::Mp4),
            ..nako_api::public_client::BrowserPlaybackCapabilitiesDto::default()
        }),
        subtitle_stream_index: None,
    };
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &play_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(play_user_id),
            source.library_id,
        )
        .await
        .unwrap();

    let revoked = response_for(&play_router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(revoked.status(), StatusCode::FORBIDDEN);
    let revoked: ErrorResponse = body_json(revoked).await;
    assert_eq!(revoked.code, "forbidden");
    assert!(
        revoked
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial after revocation, got {}",
        revoked.message
    );
    assert!(
        !marker.exists(),
        "revoked Remux ticket use must reject before starting ffmpeg"
    );
}

#[tokio::test]
async fn browser_playback_ticket_is_scoped_to_playback_mode() {
    let (_temp, router, source, _store) =
        router_with_media_source("ticket-scope.mp4", b"0123456789").await;
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Direct,
        capabilities: None,
        subtitle_stream_index: None,
    };
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;
    let token = ticket_param(&ticket.urls[0].url);

    let remux = response_for(
        &router,
        Method::GET,
        &format!(
            "/sources/{}/stream/remux?output_container=mp4&ticket={token}",
            source.id
        ),
    )
    .await;
    assert_eq!(remux.status(), StatusCode::UNAUTHORIZED);

    let hls = response_for(
        &router,
        Method::GET,
        &format!(
            "/sources/{}/stream/hls/playlist.m3u8?ticket={token}",
            source.id
        ),
    )
    .await;
    assert_eq!(hls.status(), StatusCode::UNAUTHORIZED);

    let subtitle = response_for(
        &router,
        Method::GET,
        &format!("/sources/{}/subtitles/2?ticket={token}", source.id),
    )
    .await;
    assert_eq!(subtitle.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn direct_stream_zero_byte_file_returns_empty_ok() {
    let (_temp, router, source, _store) = router_with_media_source("empty.mp4", b"").await;

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
async fn direct_stream_response_proxies_vfs_body_stream() {
    let uri = StorageUri::parse("webdav:///Movies/Demo.mkv").unwrap();
    let range = Some(ByteRange {
        offset: 2,
        length: Some(4),
    });
    let body =
        crate::app::DirectPlaySourceBody::Stream(crate::app::DirectPlayStreamBody::unbudgeted(
            ReadStream::from_bytes(uri, range, b"2345".to_vec()),
        ));
    let response_plan = plan_direct_play_response(
        10,
        "video/mp4",
        DirectPlayRangeRequest::Range(RequestedByteRange {
            start: Some(2),
            end: Some(5),
        }),
    );

    let response = stream_direct_play_response(body, "webdav:///Movies/Demo.mkv", &response_plan)
        .await
        .unwrap();

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
async fn remote_direct_stream_permit_lives_until_response_body_is_dropped() {
    let server = MockWebDavServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig {
            remote_stream_concurrency: 1,
            remote_stage_concurrency: 1,
            ..PlaybackConfig::default()
        },
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: Some(crate::config::WebDavLibraryConfig {
                root: "webdav:///Movies".to_owned(),
                base_url: server.base_url(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 1,
            }),
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Remote Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "webdav:///Movies/Demo.mkv".to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let router = build_router(app);

    let first_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/sources/{}/stream", source.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_body = first_response.into_body();

    let second_busy_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/sources/{}/stream", source.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_busy_response.status(), StatusCode::CONFLICT);
    let busy_error = body_json::<ErrorResponse>(second_busy_response).await;
    assert_eq!(busy_error.code, "conflict");
    assert!(busy_error.message.contains("remote_stream"));
    assert!(!busy_error.message.contains("webdav:///"));

    drop(first_body);
    let second_response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/sources/{}/stream", source.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);
    let bytes = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&bytes[..], b"demo");
}

#[tokio::test]
async fn direct_stream_rejects_unsatisfiable_and_multi_ranges() {
    let (_temp, router, source, _store) = router_with_media_source("demo.mp4", b"0123456789").await;

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
    assert_eq!(
        response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let session_header = response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"emux");
    assert!(session_header.is_some());

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
async fn browser_playback_ticket_streams_remux_bytes() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, _marker, _store) =
        router_with_remux_source(false).await;
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Remux,
        capabilities: Some(nako_api::public_client::BrowserPlaybackCapabilitiesDto {
            direct_play: Some(false),
            container: Some(vec!["mkv".to_owned()]),
            video_codec: Some(vec!["h264".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned()]),
            output_container: Some(nako_api::public_client::BrowserPlaybackOutputContainer::Mp4),
            ..nako_api::public_client::BrowserPlaybackCapabilitiesDto::default()
        }),
        subtitle_stream_index: None,
    };
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    assert_eq!(
        ticket.mode,
        nako_api::public_client::BrowserPlaybackMode::Remux
    );
    let playback_session_id = ticket
        .playback_session_id
        .as_deref()
        .expect("remux browser ticket exposes heartbeat session identity");
    assert!(ticket.urls[0].url.contains("output_container=mp4"));
    assert!(ticket.urls[0].supports_range_requests);
    assert!(!ticket.urls[0].url.contains("playback_session_id"));

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&ticket.urls[0].url)
                .header(header::RANGE, "bytes=1-4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() != StatusCode::PARTIAL_CONTENT {
        let status = response.status();
        let error = body_json::<ErrorResponse>(response).await;
        panic!(
            "expected remux browser ticket response 206, got {status}: {} {}",
            error.code, error.message
        );
    }
    assert_eq!(
        response
            .headers()
            .get(PLAYBACK_SESSION_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(playback_session_id)
    );
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("video/mp4")
    );
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"emux");
}

#[tokio::test]
async fn head_remux_stream_route_exposes_session_without_body() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, _marker, store) =
        router_with_remux_source(false).await;
    let path = format!("/sources/{}/stream/remux?output_container=mp4", source.id);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri(&path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
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
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let session =
        latest_playback_session_for_source(&store, source.id, PlaybackSessionMode::Remux).await;
    assert!(session.transcode_session_id.is_some());
    let session_id = session.id.to_string();
    assert_eq!(
        response
            .headers()
            .get(PLAYBACK_SESSION_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(session_id.as_str())
    );
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(bytes.is_empty());
}

#[tokio::test]
async fn head_remux_stream_route_exposes_active_session_before_ffmpeg_finishes() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, marker, store) =
        router_with_remux_source(true).await;
    let path = format!("/sources/{}/stream/remux?output_container=mp4", source.id);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::HEAD)
                .uri(&path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let session_id = response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("active remux preflight should expose a public session id")
        .to_owned();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(bytes.is_empty());

    wait_for_marker(&marker).await;

    let playback_session = store
        .get_playback_session(session_id.parse().unwrap())
        .await
        .unwrap()
        .expect("playback session header should point at durable playback session");
    let active = store
        .find_active_transcode_session(
            source.id,
            TranscodeSessionKind::Remux,
            &local_remux_request_key(&source, nako_transcode::RemuxContainer::Mp4),
        )
        .await
        .unwrap()
        .expect("remux preflight session should still be active");
    assert_eq!(playback_session.transcode_session_id, Some(active.id));
    assert!(active.state.is_active());

    let cancel_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{session_id}/cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);

    let mut final_session = None;
    let mut last_state = None;
    for _ in 0..150 {
        let session_response = request_json::<PlaybackSessionResponse>(
            &router,
            Method::GET,
            &format!("/playback/sessions/{session_id}"),
        )
        .await;
        last_state = Some(session_response.session.state.clone());
        if session_response.session.state
            == nako_api::public_client::ClientPlaybackSessionState::Cancelled
        {
            final_session = Some(session_response.session);
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    let final_session = final_session.unwrap_or_else(|| {
        panic!("active remux preflight should cancel; last state: {last_state:?}")
    });

    assert_eq!(
        final_session.transcode_session_id.as_deref(),
        Some(active.id.to_string().as_str())
    );
}

#[tokio::test]
async fn playback_session_route_returns_remux_session_state() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, _marker, _store) =
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

    let session_id = response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("remux stream should expose playback session id")
        .to_owned();
    let session_response = request_json::<PlaybackSessionResponse>(
        &router,
        Method::GET,
        &format!("/playback/sessions/{session_id}"),
    )
    .await;

    assert_eq!(session_response.session.id, session_id);
    assert_eq!(
        session_response.session.mode,
        nako_api::public_client::ClientPlaybackSessionMode::Remux
    );
    assert_eq!(
        session_response.session.state,
        nako_api::public_client::ClientPlaybackSessionState::Active
    );
    assert!(session_response.session.transcode_session_id.is_some());
    let session_json = serde_json::to_value(&session_response).unwrap();
    assert!(session_json["session"].get("output_path").is_none());
}

#[tokio::test]
async fn playback_session_route_maps_internal_failure_taxonomy_to_public_contract() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, _marker, store) =
        router_with_remux_source(false).await;
    let session = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "taxonomy:test".to_owned(),
            output_path: "cache/remux/private/stream.mp4".into(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            session.id,
            TranscodeSessionState::Failed,
            Some(TranscodeFailureCategory::Plan),
            Some("playback transcode planning failed".to_owned()),
        )
        .await
        .unwrap();

    let playback_session = store
        .create_playback_session(NewPlaybackSession {
            id: PlaybackSessionId::new(),
            principal_id: UserPrincipalId::local_admin(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Remux,
            state: PlaybackSessionState::Failed,
            client_capabilities_json: None,
            started_at_ms: 1_779_814_400_000,
            updated_at_ms: 1_779_814_401_000,
        })
        .await
        .unwrap();
    store
        .link_playback_session_transcode(playback_session.id, session.id)
        .await
        .unwrap();

    let response = request_json::<PlaybackSessionResponse>(
        &router,
        Method::GET,
        &format!("/playback/sessions/{}", playback_session.id),
    )
    .await;

    assert_eq!(response.session.id, playback_session.id.to_string());
    assert_eq!(
        response.session.state,
        nako_api::public_client::ClientPlaybackSessionState::Failed
    );
    assert_eq!(
        response.session.transcode_session_id.as_deref(),
        Some(session.id.to_string().as_str())
    );
    let json = serde_json::to_string(&response).unwrap();
    assert!(!json.contains("playback transcode planning failed"));
    assert!(!json.contains("output_path"));
    assert!(!json.contains("cache/remux/private"));
}

#[tokio::test]
async fn playback_session_route_redacts_raw_persisted_failure_message() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, _marker, store) =
        router_with_remux_source(false).await;
    let session = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "taxonomy:raw-message".to_owned(),
            output_path: "cache/remux/private/stream.mp4".into(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            session.id,
            TranscodeSessionState::Failed,
            Some(TranscodeFailureCategory::Runner),
            Some(
                "ffmpeg failed at C:\\secret\\movie.mkv with webdav:///Movies/secret.mkv"
                    .to_owned(),
            ),
        )
        .await
        .unwrap();

    let playback_session = store
        .create_playback_session(NewPlaybackSession {
            id: PlaybackSessionId::new(),
            principal_id: UserPrincipalId::local_admin(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Remux,
            state: PlaybackSessionState::Failed,
            client_capabilities_json: None,
            started_at_ms: 1_779_814_400_000,
            updated_at_ms: 1_779_814_401_000,
        })
        .await
        .unwrap();
    store
        .link_playback_session_transcode(playback_session.id, session.id)
        .await
        .unwrap();

    let response = request_json::<PlaybackSessionResponse>(
        &router,
        Method::GET,
        &format!("/playback/sessions/{}", playback_session.id),
    )
    .await;

    assert_eq!(response.session.id, playback_session.id.to_string());
    assert_eq!(
        response.session.state,
        nako_api::public_client::ClientPlaybackSessionState::Failed
    );
    let json = serde_json::to_string(&response).unwrap();
    assert!(!json.contains("C:\\secret"));
    assert!(!json.contains("webdav:///"));
    assert!(!json.contains("cache/remux/private"));
}

#[tokio::test]
async fn playback_session_cancel_route_cancels_active_remux_session() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, marker, store) =
        router_with_remux_source(true).await;
    let remux_path = format!("/sources/{}/stream/remux?output_container=mp4", source.id);
    let first_router = router.clone();
    let first_path = remux_path.clone();
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

    wait_for_marker(&marker).await;

    let active_transcode = store
        .find_active_transcode_session(
            source.id,
            TranscodeSessionKind::Remux,
            &local_remux_request_key(&source, nako_transcode::RemuxContainer::Mp4),
        )
        .await
        .unwrap()
        .unwrap();
    let playback_session =
        latest_playback_session_for_source(&store, source.id, PlaybackSessionMode::Remux).await;
    assert_eq!(
        playback_session.transcode_session_id,
        Some(active_transcode.id)
    );

    let cancel_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{}/cancel", playback_session.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_body = body_json::<PlaybackSessionResponse>(cancel_response).await;
    assert_eq!(cancel_body.session.id, playback_session.id.to_string());
    assert_eq!(
        cancel_body.session.state,
        nako_api::public_client::ClientPlaybackSessionState::Cancelled
    );
    assert_eq!(
        cancel_body.session.transcode_session_id.as_deref(),
        Some(active_transcode.id.to_string().as_str())
    );

    let first_response = first.await.unwrap();
    assert_eq!(first_response.status(), StatusCode::BAD_GATEWAY);
    let first_error = body_json::<ErrorResponse>(first_response).await;
    assert_eq!(first_error.code, "ffmpeg_error");

    let mut final_session = None;
    for _ in 0..50 {
        let session_response = request_json::<PlaybackSessionResponse>(
            &router,
            Method::GET,
            &format!("/playback/sessions/{}", playback_session.id),
        )
        .await;
        if session_response.session.state
            == nako_api::public_client::ClientPlaybackSessionState::Cancelled
        {
            final_session = Some(session_response.session);
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    let final_session = final_session.expect("cancelled remux session should become terminal");

    assert!(final_session.ended_at.is_some());
}

#[tokio::test]
async fn playback_session_cancel_route_rejects_terminal_session() {
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

    let transcode_session = store
        .find_latest_transcode_session(
            source.id,
            TranscodeSessionKind::Remux,
            &local_remux_request_key(&source, nako_transcode::RemuxContainer::Mp4),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(transcode_session.state, TranscodeSessionState::Finished);
    let playback_session =
        latest_playback_session_for_source(&store, source.id, PlaybackSessionMode::Remux).await;
    assert_eq!(
        playback_session.transcode_session_id,
        Some(transcode_session.id)
    );
    store
        .set_playback_session_state(
            playback_session.id,
            PlaybackSessionState::Ended,
            Some(1_779_814_402_000),
        )
        .await
        .unwrap();

    let cancel_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{}/cancel", playback_session.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::CONFLICT);
    let error = body_json::<ErrorResponse>(cancel_response).await;
    assert_eq!(error.code, "conflict");
    assert!(error.message.contains("already terminal"));
}

#[tokio::test]
async fn playback_session_cancel_route_rejects_process_local_stale_active_session() {
    let (temp, router, source, store) = router_with_hls_source().await;
    let stale = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: local_remux_request_key(&source, nako_transcode::RemuxContainer::Mp4),
            output_path: temp.path().join("stale.mp4"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    let playback_session = store
        .create_playback_session(NewPlaybackSession {
            id: PlaybackSessionId::new(),
            principal_id: UserPrincipalId::local_admin(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Remux,
            state: PlaybackSessionState::Active,
            client_capabilities_json: None,
            started_at_ms: 1_779_814_400_000,
            updated_at_ms: 1_779_814_401_000,
        })
        .await
        .unwrap();
    store
        .link_playback_session_transcode(playback_session.id, stale.id)
        .await
        .unwrap();

    let cancel_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{}/cancel", playback_session.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::CONFLICT);
    let error = body_json::<ErrorResponse>(cancel_response).await;
    assert_eq!(error.code, "conflict");
    assert!(error.message.contains("not running in this process"));
}

#[tokio::test]
async fn hls_playlist_and_segment_routes_work() {
    let (_temp, router, source, store) = router_with_hls_source().await;
    let playlist_path = format!("/sources/{}/stream/hls/playlist.m3u8", source.id);
    let request_id = "REQ-HLS-TRACE-ROUTE";
    let normalized_request_id = "req-hls-trace-route";

    let playlist_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&playlist_path)
                .header(crate::http::trace_context::X_REQUEST_ID_HEADER, request_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(playlist_response.status(), StatusCode::OK);
    assert_eq!(
        playlist_response
            .headers()
            .get(crate::http::trace_context::X_REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(normalized_request_id)
    );
    assert_eq!(
        playlist_response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/vnd.apple.mpegurl")
    );
    assert_eq!(
        playlist_response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let session_header = playlist_response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    let session = store
        .find_latest_transcode_session(
            source.id,
            TranscodeSessionKind::HlsTranscode,
            &local_hls_request_key(&source, nako_transcode::HardwareAcceleration::None),
        )
        .await
        .unwrap()
        .unwrap();
    let playback_session = store
        .get_playback_session(
            session_header
                .as_deref()
                .expect("hls playlist should expose playback session id")
                .parse()
                .unwrap(),
        )
        .await
        .unwrap()
        .expect("hls playlist header should point at durable playback session");
    assert_eq!(playback_session.transcode_session_id, Some(session.id));
    let playlist = String::from_utf8(
        to_bytes(playlist_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let segment_path = format!(
        "/playback/sessions/{}/hls/segments/segment_00000.ts",
        playback_session.id
    );

    assert!(playlist.contains(&segment_path));

    let segment_response = wait_for_hls_segment_ok(&router, &segment_path).await;

    assert_eq!(segment_response.status(), StatusCode::OK);
    assert_eq!(
        segment_response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("video/mp2t")
    );
    assert_eq!(
        segment_response
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let segment = to_bytes(segment_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&segment[..], b"segment");

    wait_for_transcode_state(&store, session.id, TranscodeSessionState::Finished).await;
    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    let finished_event = events
        .iter()
        .find(|event| {
            event.kind == DomainEventKind::PlaybackSessionFinished
                && event.subject == DomainEventSubject::PlaybackSession(session.id)
                && event.source_id == Some(source.id)
        })
        .expect("finished HLS route should emit an outbox event");
    let payload: serde_json::Value = serde_json::from_str(&finished_event.payload_json).unwrap();
    assert_eq!(payload["request_id"].as_str(), Some(normalized_request_id));
    assert!(!finished_event.payload_json.contains(request_id));

    let legacy_segment_path = format!(
        "/playback/sessions/{}/hls/segments/segment_00000.ts",
        session.id
    );
    assert!(!playlist.contains(&legacy_segment_path));
    let legacy_segment_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&legacy_segment_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(legacy_segment_response.status(), StatusCode::NOT_FOUND);

    let missing = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/playback/sessions/{}/hls/segments/missing.ts",
                    playback_session.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn hls_playlist_route_accepts_preferred_audio_language_defaults() {
    let (_temp, router, source, store) = router_with_hls_source().await;
    store
        .upsert_media_probe(source.id, &multi_audio_probe())
        .await
        .unwrap();
    let first_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?preferred_audio_language=JPN,eng,jpn",
        source.id
    );
    let second_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?preferred_audio_language=jpn,eng",
        source.id
    );

    let (first_playlist, first_transcode_session_id) =
        hls_playlist_body_and_transcode_session(&router, &store, &first_path).await;
    let (second_playlist, second_transcode_session_id) =
        hls_playlist_body_and_transcode_session(&router, &store, &second_path).await;

    assert_audio_default(&first_playlist, "jpn");
    assert_audio_default(&second_playlist, "jpn");
    assert_eq!(second_transcode_session_id, first_transcode_session_id);
}

#[tokio::test]
async fn hls_playlist_route_audio_stream_overrides_preferred_audio_language() {
    let (_temp, router, source, store) = router_with_hls_source().await;
    store
        .upsert_media_probe(source.id, &multi_audio_probe())
        .await
        .unwrap();
    let playlist_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?audio_stream=1&preferred_audio_language=jpn",
        source.id
    );

    let (playlist, _transcode_session_id) =
        hls_playlist_body_and_transcode_session(&router, &store, &playlist_path).await;

    assert_audio_default(&playlist, "eng");
}

#[tokio::test]
async fn hls_playlist_route_accepts_preferred_subtitle_language_defaults() {
    let (_temp, router, source, store) = router_with_hls_source().await;
    store
        .upsert_media_probe(source.id, &multi_subtitle_probe())
        .await
        .unwrap();
    let first_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?preferred_subtitle_language=JPN,eng,jpn",
        source.id
    );
    let second_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?preferred_subtitle_language=jpn,eng",
        source.id
    );

    let (first_playlist, first_transcode_session_id) =
        hls_playlist_body_and_transcode_session(&router, &store, &first_path).await;
    let (second_playlist, second_transcode_session_id) =
        hls_playlist_body_and_transcode_session(&router, &store, &second_path).await;

    assert_subtitle_default(&first_playlist, "jpn");
    assert_subtitle_default(&second_playlist, "jpn");
    assert_eq!(second_transcode_session_id, first_transcode_session_id);
}

#[tokio::test]
async fn hls_playlist_route_subtitle_stream_overrides_preferred_subtitle_language() {
    let (_temp, router, source, store) = router_with_hls_source().await;
    store
        .upsert_media_probe(source.id, &multi_subtitle_probe())
        .await
        .unwrap();
    let playlist_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?subtitle_stream=2&preferred_subtitle_language=jpn",
        source.id
    );

    let (playlist, _transcode_session_id) =
        hls_playlist_body_and_transcode_session(&router, &store, &playlist_path).await;

    assert_subtitle_default(&playlist, "eng");
}

#[tokio::test]
async fn hls_playlist_route_returns_while_transcode_session_is_running() {
    let (_temp, router, source, store) = router_with_running_hls_source().await;
    let playlist_path = format!("/sources/{}/stream/hls/playlist.m3u8", source.id);

    let playlist_response = tokio::time::timeout(
        process_backed_hls_playlist_readiness_timeout(),
        router.clone().oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&playlist_path)
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await
    .expect("hls playlist route should return before ffmpeg exits")
    .unwrap();

    assert_eq!(playlist_response.status(), StatusCode::OK);
    let playback_session_id: PlaybackSessionId = playlist_response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("hls playlist should expose playback session id")
        .parse()
        .unwrap();
    let playback_session = store
        .get_playback_session(playback_session_id)
        .await
        .unwrap()
        .unwrap();
    let transcode_session_id = playback_session
        .transcode_session_id
        .expect("hls playback session should link transcode session");
    let transcode = store
        .get_transcode_session(transcode_session_id)
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

    assert_eq!(transcode.state, TranscodeSessionState::Running);
    assert!(playlist.contains(&format!(
        "/playback/sessions/{playback_session_id}/hls/segments/segment_00000.ts"
    )));

    let segment_uri =
        format!("/playback/sessions/{playback_session_id}/hls/segments/segment_00000.ts");
    let segment_response = wait_for_hls_segment_ok(&router, &segment_uri).await;
    let _segment_body = to_bytes(segment_response.into_body(), usize::MAX)
        .await
        .unwrap();

    let missing_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/playback/sessions/{playback_session_id}/hls/segments/segment_00001.ts"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_response.status(), StatusCode::CONFLICT);
    let _missing_body = to_bytes(missing_response.into_body(), usize::MAX)
        .await
        .unwrap();

    let cancel_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{playback_session_id}/cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    wait_for_transcode_state(
        &store,
        transcode_session_id,
        TranscodeSessionState::Cancelled,
    )
    .await;
}

#[tokio::test]
async fn hls_playlist_route_accepts_seek_start_position() {
    let (_temp, router, source, store) = router_with_hls_source().await;
    let playlist_path = format!(
        "/sources/{}/stream/hls/playlist.m3u8?start_position_ms=45250",
        source.id
    );

    let playlist_response = router
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

    let sessions = store
        .list_transcode_sessions(
            TranscodeSessionListFilter {
                source_id: Some(source.id),
                kind: Some(TranscodeSessionKind::HlsTranscode),
                state: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();

    assert!(sessions.iter().any(|session| {
        session
            .request_key
            .contains("hls-playback-generation:v1%3Bstart_ms%3D45250")
    }));
}

#[tokio::test]
async fn hls_segment_route_serves_existing_running_segment() {
    let (temp, router, source, store) = router_with_hls_source().await;
    let active_dir = temp.path().join("active-hls");
    fs::create_dir_all(&active_dir).unwrap();
    fs::write(active_dir.join("segment_00000.ts"), b"partial-segment").unwrap();
    let active = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: local_hls_request_key(&source, nako_transcode::HardwareAcceleration::None),
            output_path: active_dir.join("playlist.m3u8"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    let playback_session = store
        .create_playback_session(NewPlaybackSession {
            id: PlaybackSessionId::new(),
            principal_id: UserPrincipalId::local_admin(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Hls,
            state: PlaybackSessionState::Active,
            client_capabilities_json: None,
            started_at_ms: 1,
            updated_at_ms: 1,
        })
        .await
        .unwrap();
    store
        .link_playback_session_transcode(playback_session.id, active.id)
        .await
        .unwrap();
    let path = format!(
        "/playback/sessions/{}/hls/segments/segment_00000.ts",
        playback_session.id
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

    assert_eq!(response.status(), StatusCode::OK);
    let segment = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&segment[..], b"partial-segment");
}

#[tokio::test]
async fn browser_playback_ticket_protects_hls_playlist_and_segments() {
    let (_temp, router, source, _store) = router_with_hls_source().await;
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Hls,
        capabilities: Some(nako_api::public_client::BrowserPlaybackCapabilitiesDto {
            direct_play: Some(true),
            container: Some(vec!["mp4".to_owned()]),
            video_codec: Some(vec!["h264".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned()]),
            output_container: None,
            ..nako_api::public_client::BrowserPlaybackCapabilitiesDto::default()
        }),
        subtitle_stream_index: None,
    };

    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    assert_eq!(
        ticket.mode,
        nako_api::public_client::BrowserPlaybackMode::Hls
    );
    let playback_session_id = ticket
        .playback_session_id
        .as_deref()
        .expect("hls browser ticket exposes heartbeat session identity");
    assert_eq!(
        ticket.urls[0].kind,
        nako_api::public_client::BrowserPlaybackUrlKind::Playlist
    );
    assert!(ticket.urls[0].url.contains("ticket="));
    assert!(!ticket.urls[0].url.contains("playback_session_id"));

    let playlist_response = response_for(&router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(playlist_response.status(), StatusCode::OK);
    assert_eq!(
        playlist_response
            .headers()
            .get(PLAYBACK_SESSION_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(playback_session_id)
    );
    let playlist = response_text(playlist_response).await;
    assert!(playlist.contains("?ticket="));
    assert!(!playlist.contains("Bearer"));
    let segment_uri = playlist
        .lines()
        .find(|line| line.starts_with("/playback/sessions/"))
        .expect("playlist contains ticketed segment URL")
        .to_owned();
    assert!(segment_uri.starts_with(&format!("/playback/sessions/{playback_session_id}/")));

    let segment_response = wait_for_hls_segment_ok(&router, &segment_uri).await;
    let segment = to_bytes(segment_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&segment[..], b"segment");

    let segment_path = segment_uri
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(segment_uri.as_str());
    let invalid = response_for(
        &router,
        Method::GET,
        &format!("{segment_path}?ticket=not-a-ticket"),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn hls_browser_playback_ticket_rejects_revocation_at_playlist_use() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_ticket_playlist_revoked");
    let (_temp, app, source, store) =
        app_with_media_source_config("ticket-hls-playlist.mkv", b"media", |config| {
            config.ffmpeg_path = ffmpeg_path.clone();
        })
        .await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let play_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let play_user_id = play_principal.user_id;
    let play_router = public_client_router_with_principal(app, play_principal);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Hls,
        capabilities: Some(nako_api::public_client::BrowserPlaybackCapabilitiesDto {
            direct_play: Some(true),
            container: Some(vec!["mp4".to_owned()]),
            video_codec: Some(vec!["h264".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned()]),
            output_container: None,
            ..nako_api::public_client::BrowserPlaybackCapabilitiesDto::default()
        }),
        subtitle_stream_index: None,
    };
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &play_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(play_user_id),
            source.library_id,
        )
        .await
        .unwrap();

    let revoked = response_for(&play_router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(revoked.status(), StatusCode::FORBIDDEN);
    let revoked: ErrorResponse = body_json(revoked).await;
    assert_eq!(revoked.code, "forbidden");
    assert!(
        revoked
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial after revocation, got {}",
        revoked.message
    );
}

#[tokio::test]
async fn hls_browser_playback_ticket_rejects_revocation_at_segment_use() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_ticket_segment_revoked");
    let (_temp, app, source, store) =
        app_with_media_source_config("ticket-hls-segment.mkv", b"media", |config| {
            config.ffmpeg_path = ffmpeg_path.clone();
        })
        .await;
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let play_principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    let play_user_id = play_principal.user_id;
    let play_router = public_client_router_with_principal(app, play_principal);
    let issue_request = nako_api::public_client::BrowserPlaybackTicketRequest {
        mode: nako_api::public_client::BrowserPlaybackMode::Hls,
        capabilities: Some(nako_api::public_client::BrowserPlaybackCapabilitiesDto {
            direct_play: Some(true),
            container: Some(vec!["mp4".to_owned()]),
            video_codec: Some(vec!["h264".to_owned()]),
            audio_codec: Some(vec!["aac".to_owned()]),
            output_container: None,
            ..nako_api::public_client::BrowserPlaybackCapabilitiesDto::default()
        }),
        subtitle_stream_index: None,
    };
    let ticket = request_body_json::<nako_api::public_client::BrowserPlaybackTicketResponse, _>(
        &play_router,
        Method::POST,
        &format!("/sources/{}/playback/browser-ticket", source.id),
        &issue_request,
    )
    .await;

    let playlist_response = response_for(&play_router, Method::GET, &ticket.urls[0].url).await;
    assert_eq!(playlist_response.status(), StatusCode::OK);
    let playlist = response_text(playlist_response).await;
    let segment_uri = playlist
        .lines()
        .find(|line| line.starts_with("/playback/sessions/"))
        .expect("playlist contains ticketed segment URL")
        .to_owned();

    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(play_user_id),
            source.library_id,
        )
        .await
        .unwrap();

    let revoked = response_for(&play_router, Method::GET, &segment_uri).await;
    assert_eq!(revoked.status(), StatusCode::FORBIDDEN);
    let revoked: ErrorResponse = body_json(revoked).await;
    assert_eq!(revoked.code, "forbidden");
    assert!(
        revoked
            .message
            .contains("required Library Access level 'play'"),
        "expected library play access denial after revocation, got {}",
        revoked.message
    );
}

#[tokio::test]
async fn hls_segment_route_rejects_unfinished_session() {
    let (temp, router, source, store) = router_with_hls_source().await;
    let active = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: local_hls_request_key(&source, nako_transcode::HardwareAcceleration::None),
            output_path: temp.path().join("active.m3u8"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    let playback_session = store
        .create_playback_session(NewPlaybackSession {
            id: PlaybackSessionId::new(),
            principal_id: UserPrincipalId::local_admin(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Hls,
            state: PlaybackSessionState::Active,
            client_capabilities_json: None,
            started_at_ms: 1,
            updated_at_ms: 1,
        })
        .await
        .unwrap();
    store
        .link_playback_session_transcode(playback_session.id, active.id)
        .await
        .unwrap();
    let path = format!(
        "/playback/sessions/{}/hls/segments/segment_00000.ts",
        playback_session.id
    );

    let response = router
        .clone()
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

    let legacy_path = format!(
        "/playback/sessions/{}/hls/segments/segment_00000.ts",
        active.id
    );
    let legacy_response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(legacy_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(legacy_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn remux_stream_route_waits_for_in_flight_duplicate_and_reuses_session() {
    let (_temp, router, source, _staging_root, _ffmpeg_path, marker, store) =
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

    wait_for_marker(&marker).await;

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

    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate_session_id = duplicate
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("duplicate remux stream should expose reused session id")
        .to_owned();
    let duplicate_bytes = to_bytes(duplicate.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&duplicate_bytes[..], b"remuxed");

    let first_response = first.await.unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_session_id = first_response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("first remux stream should expose session id")
        .to_owned();
    assert_ne!(first_session_id, duplicate_session_id);
    let first_playback_session = store
        .get_playback_session(first_session_id.parse().unwrap())
        .await
        .unwrap()
        .expect("first remux response should expose durable playback session");
    let duplicate_playback_session = store
        .get_playback_session(duplicate_session_id.parse().unwrap())
        .await
        .unwrap()
        .expect("duplicate remux response should expose durable playback session");
    assert_eq!(
        first_playback_session.transcode_session_id,
        duplicate_playback_session.transcode_session_id
    );
    assert!(first_playback_session.transcode_session_id.is_some());
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
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        nako_api::public_client::ClientErrorCode::from_code(&error.code),
        Some(nako_api::public_client::ClientErrorCode::NotFound)
    );
    assert!(error.message.contains("not found"));
}

#[tokio::test]
async fn paginated_routes_echo_page_info_and_reject_large_limits() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let sources_path = format!("/libraries/{library_id}/sources?limit=10&offset=20");

    let sources = request_json::<nako_api::public_client::LibrarySourcesResponse>(
        &router,
        Method::GET,
        &sources_path,
    )
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
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::InvalidInput.as_str()
    );
    assert!(
        error
            .message
            .contains("limit must be less than or equal to")
    );
}
