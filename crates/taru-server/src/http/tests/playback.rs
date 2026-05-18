use super::*;

async fn wait_for_marker(marker: &std::path::Path) {
    for _ in 0..250 {
        if marker.exists() {
            return;
        }
        sleep(Duration::from_millis(20)).await;
    }
    panic!("remux fixture did not start before timeout: {marker:?}");
}

#[tokio::test]
async fn playback_decision_and_direct_stream_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mp4"), b"0123456789").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        auth: crate::config::AuthConfig::disabled(),
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
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
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
    let decision_json = request_json::<serde_json::Value>(
        &router,
        Method::GET,
        &format!("/sources/{}/playback/decision?direct_play=false", source.id),
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

    assert_eq!(
        decision.decision.mode,
        taru_api::ClientPlaybackMode::DirectPlay
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
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        auth: crate::config::AuthConfig::disabled(),
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
        staging: StagingConfig::default(),
        playback: PlaybackConfig {
            remote_stream_concurrency: 1,
            remote_stage_concurrency: 1,
        },
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: taru_core::LibraryPreset::Movies,
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
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: taru_core::MediaItemId::new(),
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

    let second = tokio::time::timeout(
        Duration::from_millis(50),
        router.clone().oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/sources/{}/stream", source.id))
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;
    assert!(second.is_err());

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

    assert_eq!(session_response.session.id, session.id.to_string());
    assert_eq!(
        session_response.session.state,
        ClientTranscodeSessionState::Finished
    );
    let session_json = serde_json::to_value(&session_response).unwrap();
    assert!(session_json["session"].get("output_path").is_none());
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

    let session = store
        .find_active_transcode_session(source.id, TranscodeSessionKind::Remux, "remux:mp4")
        .await
        .unwrap()
        .unwrap();
    let cancel_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{}/cancel", session.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_body = body_json::<TranscodeSessionResponse>(cancel_response).await;
    assert_eq!(cancel_body.session.id, session.id.to_string());
    assert!(matches!(
        cancel_body.session.state,
        ClientTranscodeSessionState::CancelRequested | ClientTranscodeSessionState::Cancelled
    ));
    assert_eq!(
        cancel_body.session.failure_category,
        Some(ClientTranscodeFailureCategory::Cancelled)
    );

    let first_response = first.await.unwrap();
    assert_eq!(first_response.status(), StatusCode::BAD_GATEWAY);
    let first_error = body_json::<ErrorResponse>(first_response).await;
    assert_eq!(first_error.code, "ffmpeg_error");

    let mut final_session = None;
    for _ in 0..50 {
        let session_response = request_json::<TranscodeSessionResponse>(
            &router,
            Method::GET,
            &format!("/playback/sessions/{}", session.id),
        )
        .await;
        if session_response.session.state == ClientTranscodeSessionState::Cancelled {
            final_session = Some(session_response.session);
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    let final_session = final_session.expect("cancelled remux session should become terminal");

    assert_eq!(
        final_session.failure_category,
        Some(ClientTranscodeFailureCategory::Cancelled)
    );
    assert!(final_session.completed_at.is_some());
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

    let session = store
        .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, "remux:mp4")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.state, TranscodeSessionState::Finished);

    let cancel_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{}/cancel", session.id))
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
            request_key: "remux:mp4".to_owned(),
            output_path: temp.path().join("stale.mp4"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let cancel_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{}/cancel", stale.id))
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
        .find_latest_transcode_session(source.id, TranscodeSessionKind::HlsTranscode, "hls:single")
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
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        taru_api::ClientErrorCode::from_code(&error.code),
        Some(taru_api::ClientErrorCode::NotFound)
    );
    assert!(error.message.contains("not found"));
}

#[tokio::test]
async fn paginated_routes_echo_page_info_and_reject_large_limits() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let sources_path = format!("/libraries/{library_id}/sources?limit=10&offset=20");

    let sources =
        request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path).await;
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
    assert_eq!(error.code, taru_api::ClientErrorCode::InvalidInput.as_str());
    assert!(
        error
            .message
            .contains("limit must be less than or equal to")
    );
}
