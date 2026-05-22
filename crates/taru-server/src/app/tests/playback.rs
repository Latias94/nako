use super::*;

#[tokio::test]
async fn remux_source_runs_runner_and_reuses_completed_output() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
    let request = RemuxSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
        output_container: RemuxContainer::Mp4,
    };

    let output = app.playback().remux_source(request.clone()).await.unwrap();
    let session = output.session.as_ref().unwrap();

    assert_eq!(output.disposition, RemuxSourceDisposition::Finished);
    assert_eq!(session.state, TranscodeSessionState::Finished);
    assert!(
        output
            .output_path
            .starts_with(&app.config().remux_staging_root)
    );
    assert_eq!(fs::read_to_string(&output.output_path).unwrap(), "remuxed");
    assert_eq!(
        app.playback()
            .get_transcode_session(session.id)
            .await
            .unwrap()
            .state,
        TranscodeSessionState::Finished
    );
    assert_eq!(
        store
            .find_latest_transcode_session(
                source.id,
                TranscodeSessionKind::Remux,
                &session.request_key,
            )
            .await
            .unwrap()
            .unwrap()
            .id,
        session.id
    );
    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::PlaybackSessionFinished
            && event.subject == DomainEventSubject::PlaybackSession(session.id)
            && event.source_id == Some(source.id)
            && !event
                .payload_json
                .contains(&app.config().remux_staging_root.display().to_string())
    }));

    let reused = app.playback().remux_source(request.clone()).await.unwrap();

    assert_eq!(reused.disposition, RemuxSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.as_ref().unwrap().id, session.id);
    assert_eq!(reused.output_path, output.output_path);
    assert_eq!(fs::read_to_string(reused.output_path).unwrap(), "remuxed");

    let config = app.config().clone();
    drop(app);
    fs::remove_file(ffmpeg_path).unwrap();
    let restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let restarted_reused = restarted.playback().remux_source(request).await.unwrap();

    assert_eq!(
        restarted_reused.disposition,
        RemuxSourceDisposition::ReusedExisting
    );
    assert_eq!(restarted_reused.session.as_ref().unwrap().id, session.id);
}

#[tokio::test]
async fn remux_source_rejects_persisted_active_duplicate() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let request_identity = local_remux_request_identity(&source, RemuxContainer::Mp4);
    let key = RemuxRequestKey {
        source_id: source.id,
        request_identity: request_identity.clone(),
    };
    let staging = RemuxStagingPolicy::new(&app.config().remux_staging_root).unwrap();
    let active = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: key.persisted_request_key(),
            output_path: staging
                .output_path(source.id, &request_identity, RemuxContainer::Mp4)
                .unwrap(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let err = app
        .playback()
        .remux_source(RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        })
        .await
        .unwrap_err();

    let TaruError::Conflict { message } = err else {
        panic!("expected remux duplicate conflict");
    };
    assert!(message.contains("already in progress"));
    assert!(message.contains(&active.id.to_string()));
}

#[tokio::test]
async fn remux_source_persists_runner_failure() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_failing_ffmpeg_script_with_stderr(
        script_root.path(),
        "failure",
        "raw ffmpeg failed while reading C:\\secret\\movie.mkv and webdav:///Movies/secret.mkv",
    );
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let request_key = RemuxRequestKey {
        source_id: source.id,
        request_identity: local_remux_request_identity(&source, RemuxContainer::Mp4),
    }
    .persisted_request_key();

    let err = app
        .playback()
        .remux_source(RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected remux provider failure");
    };
    assert_eq!(provider, "ffmpeg_remux");
    assert_eq!(message, "remux runner failed");

    let session = store
        .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.state, TranscodeSessionState::Failed);
    assert_eq!(
        session.failure_category,
        Some(TranscodeFailureCategory::Runner)
    );
    assert_eq!(
        session.failure_message.as_deref(),
        Some("remux runner failed")
    );
    let failure = session.failure_message.as_deref().unwrap();
    assert!(!failure.contains("C:\\secret"));
    assert!(!failure.contains("webdav:///"));
}

#[tokio::test]
async fn remux_source_persists_timeout_failure_category() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_slow_ffmpeg_script(script_root.path(), "timeout");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let mut config = app.config().clone();
    config.remux_timeout_ms = 100;
    drop(app);
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let request_key = RemuxRequestKey {
        source_id: source.id,
        request_identity: local_remux_request_identity(&source, RemuxContainer::Mp4),
    }
    .persisted_request_key();

    let err = app
        .playback()
        .remux_source(RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected remux provider timeout");
    };
    assert_eq!(provider, "ffmpeg_remux");
    assert_eq!(message, "remux runner timed out");

    let session = store
        .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.state, TranscodeSessionState::Failed);
    assert_eq!(
        session.failure_category,
        Some(TranscodeFailureCategory::Timeout)
    );
    assert_eq!(
        session.failure_message.as_deref(),
        Some("remux runner timed out")
    );
}

#[tokio::test]
async fn app_startup_marks_stale_transcode_sessions_failed() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let config = app.config().clone();
    let staging = RemuxStagingPolicy::new(&config.remux_staging_root).unwrap();
    let stale_id = TranscodeSessionId::new();
    let request_identity = local_remux_request_identity(&source, RemuxContainer::Mp4);

    store
        .create_transcode_session(NewTranscodeSession {
            id: stale_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: RemuxRequestKey {
                source_id: source.id,
                request_identity: request_identity.clone(),
            }
            .persisted_request_key(),
            output_path: staging
                .output_path(source.id, &request_identity, RemuxContainer::Mp4)
                .unwrap(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    drop(app);
    let _restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let stale = store
        .get_transcode_session(stale_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(stale.state, TranscodeSessionState::Failed);
    assert_eq!(
        stale.failure_category,
        Some(TranscodeFailureCategory::Stale)
    );
}

#[tokio::test]
async fn hls_source_runs_runner_and_reuses_completed_session() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
    };

    let output = app.playback().hls_source(request.clone()).await.unwrap();
    let session_id = output.session.id;

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert_eq!(output.session.kind, TranscodeSessionKind::HlsTranscode);
    assert_eq!(output.session.state, TranscodeSessionState::Finished);
    assert!(
        fs::read_to_string(&output.playlist_path)
            .unwrap()
            .contains("#EXTM3U")
    );
    assert_eq!(
        fs::read_to_string(output.segment_dir.join("segment_00000.ts")).unwrap(),
        "segment"
    );

    let playlist = app.playback().hls_playlist(request.clone()).await.unwrap();
    assert!(playlist.body.contains(&format!(
        "/playback/sessions/{session_id}/hls/segments/segment_00000.ts"
    )));

    let segment = app
        .playback()
        .plan_hls_segment(session_id, "segment_00000.ts")
        .await
        .unwrap();
    assert_eq!(segment.content_type, "video/mp2t");
    assert!(segment.path.ends_with("segment_00000.ts"));
    assert!(
        app.playback()
            .plan_hls_segment(session_id, "../segment_00000.ts")
            .await
            .is_err()
    );
    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::PlaybackSessionFinished
            && event.subject == DomainEventSubject::PlaybackSession(session_id)
            && event.source_id == Some(source.id)
            && !event
                .payload_json
                .contains(&app.config().remux_staging_root.display().to_string())
    }));

    fs::remove_file(ffmpeg_path).unwrap();
    let reused = app.playback().hls_source(request.clone()).await.unwrap();
    assert_eq!(reused.disposition, HlsSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.id, session_id);

    let config = app.config().clone();
    drop(app);
    let restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let restarted_reused = restarted.playback().hls_source(request).await.unwrap();

    assert_eq!(
        restarted_reused.disposition,
        HlsSourceDisposition::ReusedExisting
    );
    assert_eq!(restarted_reused.session.id, session_id);
}

#[tokio::test]
async fn hls_source_uses_selected_cpu_acceleration_when_gpu_falls_back() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_cpu_only_hls_ffmpeg_script(script_root.path(), "hls_cpu_fallback");
    let (_temp, app, _store, source) = remux_app_with_source_and_transcode(
        ffmpeg_path,
        TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Nvenc,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 1,
            gpu_concurrency: 1,
        },
    )
    .await;

    let output = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
        })
        .await
        .unwrap();

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(
        fs::read_to_string(output.playlist_path)
            .unwrap()
            .contains("#EXTM3U")
    );
}

#[tokio::test]
async fn hls_source_request_identity_separates_selected_hardware_profiles() {
    let script_root = tempfile::tempdir().unwrap();
    let cpu_ffmpeg = fake_cpu_only_hls_ffmpeg_script(script_root.path(), "hls_cpu_profile");
    let (_temp, app, store, source) = remux_app_with_source_and_transcode(
        cpu_ffmpeg,
        TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Nvenc,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 1,
            gpu_concurrency: 1,
        },
    )
    .await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
    };

    let cpu_output = app.playback().hls_source(request.clone()).await.unwrap();
    assert_eq!(cpu_output.disposition, HlsSourceDisposition::Finished);
    assert!(
        cpu_output
            .session
            .request_key
            .contains("kind%3Dhls_single_variant")
    );
    assert!(cpu_output.session.request_key.contains("hw%3Dnone"));

    let mut config = app.config().clone();
    config.ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_nvenc_profile");
    drop(app);
    let gpu_app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let gpu_output = gpu_app.playback().hls_source(request).await.unwrap();

    assert_eq!(gpu_output.disposition, HlsSourceDisposition::Finished);
    assert_ne!(gpu_output.session.id, cpu_output.session.id);
    assert_ne!(gpu_output.playlist_path, cpu_output.playlist_path);
    assert!(gpu_output.session.request_key.contains("hw%3Dnvenc"));
}

#[tokio::test]
async fn hls_source_request_identity_changes_when_source_revision_changes() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_source_revision");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
    };

    let first = app.playback().hls_source(request.clone()).await.unwrap();
    let mut changed_source = source.clone();
    changed_source.size_bytes = Some(6);
    changed_source.fingerprint = Some("local:changed".to_owned());
    store.upsert_media_source(&changed_source).await.unwrap();
    let second = app.playback().hls_source(request).await.unwrap();

    assert_eq!(first.disposition, HlsSourceDisposition::Finished);
    assert_eq!(second.disposition, HlsSourceDisposition::Finished);
    assert_ne!(first.session.id, second.session.id);
    assert_ne!(first.session.request_key, second.session.request_key);
    assert_ne!(first.playlist_path, second.playlist_path);
    assert!(
        first
            .session
            .request_key
            .starts_with("transcode-request:v1;source=source-revision:v1;")
    );
    assert!(
        first
            .session
            .request_key
            .contains(";profile=transcode-profile:v1")
    );
}

#[tokio::test]
async fn hls_service_rejects_unavailable_gpu_when_fallback_is_fail() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_cpu_only_hls_ffmpeg_script(script_root.path(), "hls_gpu_required");
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path,
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Nvenc,
            hardware_fallback: HardwareAccelerationFallback::Fail,
            cpu_concurrency: 1,
            gpu_concurrency: 1,
        },
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();

    let err = TaruApp::new_with_store(config, store).await.unwrap_err();

    assert!(matches!(err, TaruError::Unsupported(_)));
    assert!(err.to_string().contains("hardware accelerator"));
}

#[tokio::test]
async fn hls_source_rejects_persisted_active_duplicate() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let staging = HlsStagingPolicy::new(app.config().remux_staging_root.join("hls")).unwrap();
    let request_identity = local_hls_request_identity(&source, HardwareAcceleration::None);
    let layout = staging
        .single_variant_layout(source.id, &request_identity)
        .unwrap();
    let active = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: request_identity.persisted_request_key().to_owned(),
            output_path: layout.playlist_path,
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let err = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
        })
        .await
        .unwrap_err();

    let TaruError::Conflict { message } = err else {
        panic!("expected hls duplicate conflict");
    };
    assert!(message.contains("already in progress"));
    assert!(message.contains(&active.id.to_string()));

    let segment_err = app
        .playback()
        .plan_hls_segment(active.id, "segment_00000.ts")
        .await
        .unwrap_err();
    let TaruError::Conflict { message } = segment_err else {
        panic!("expected hls segment readiness conflict");
    };
    assert!(message.contains("is not ready"));
}

#[tokio::test]
async fn hls_source_persists_runner_failure() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_failing_hls_ffmpeg_script(script_root.path(), "hls_failure");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;

    let err = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected hls provider failure");
    };
    assert_eq!(provider, "ffmpeg_hls");
    assert_eq!(message, "hls runner failed");

    let session = store
        .find_latest_transcode_session(
            source.id,
            TranscodeSessionKind::HlsTranscode,
            local_hls_request_identity(&source, HardwareAcceleration::None).persisted_request_key(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.state, TranscodeSessionState::Failed);
    assert_eq!(
        session.failure_category,
        Some(TranscodeFailureCategory::Runner)
    );
    assert_eq!(
        session.failure_message.as_deref(),
        Some("hls runner failed")
    );
}

#[tokio::test]
async fn direct_play_uses_vfs_stream_when_backend_has_no_local_path() {
    let backend = RemotePlaybackBackend {
        bytes: b"remote-media".to_vec(),
        local_path_hint: None,
    };
    let source = remote_media_source("webdav:///Movies/Demo.mkv");
    let uri = StorageUri::parse(&source.locator).unwrap();
    let range = taru_streaming::RequestedByteRange {
        start: Some(2),
        end: Some(5),
    };

    let (response, body) = plan_direct_play_with_backend(
        &source,
        &uri,
        &backend,
        DirectPlayRangeRequest::Range(range),
    )
    .await
    .unwrap();

    assert_eq!(response.body_len, 4);
    assert_eq!(response.content_range.as_deref(), Some("bytes 2-5/12"));
    let DirectPlaySourceBody::Stream(stream) = body else {
        panic!("expected direct play to return a VFS stream");
    };
    assert_eq!(
        stream.stream.range,
        Some(ByteRange {
            offset: 2,
            length: Some(4)
        })
    );
}

#[tokio::test]
async fn ffmpeg_source_path_stages_remote_backend_without_local_path_hint() {
    let temp = tempfile::tempdir().unwrap();
    let backend = RemotePlaybackBackend {
        bytes: b"remote-media".to_vec(),
        local_path_hint: None,
    };
    let source = remote_media_source("webdav:///Movies/Demo.mkv");
    let uri = StorageUri::parse(&source.locator).unwrap();
    let staging_root = temp.path().join("remux").join("inputs");

    let input_path =
        source_path_for_ffmpeg_with_backend(&source, &uri, &backend, staging_root.clone())
            .await
            .unwrap();

    assert!(input_path.starts_with(&staging_root));
    assert_eq!(fs::read(&input_path).unwrap(), b"remote-media");
    assert!(!input_path.display().to_string().contains("webdav://"));
}

#[tokio::test]
async fn source_path_for_ffmpeg_records_manifest_for_remote_staging() {
    let server = MockWebDavServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let staging_root = temp.path().join("cache").join("remux");
    let library_id = LibraryId::new();
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(
        TaruServerConfig {
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
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: staging_root.clone(),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![LocalLibraryConfig {
                id: library_id,
                name: "Remote Movies".to_owned(),
                root: temp.path().join("unused-local-root"),
                preset: taru_core::LibraryPreset::Movies,
                webdav: Some(WebDavLibraryConfig {
                    root: "webdav:///Movies".to_owned(),
                    base_url: server.base_url(),
                    username: None,
                    password_env: None,
                    timeout_ms: 5_000,
                    max_attempts: 1,
                }),
            }],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let source = MediaSource {
        library_id,
        ..remote_media_source("webdav:///Movies/Demo.mkv")
    };

    let input_path = app
        .playback()
        .source_path_for_ffmpeg(&source)
        .await
        .unwrap();

    assert!(input_path.starts_with(staging_root.join("inputs")));
    assert_eq!(fs::read(&input_path).unwrap(), b"demo");
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::FfmpegInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.source_uri, "webdav:///Movies/Demo.mkv");
    assert_eq!(record.source_scheme, "webdav");
    assert_eq!(record.local_path, input_path.display().to_string());
    assert_eq!(record.size_bytes, Some(4));
    assert_eq!(record.etag.as_deref(), Some("etag-demo"));
    assert_eq!(record.fingerprint.as_deref(), Some("webdav:etag=etag-demo"));
    assert!(record.expires_at_ms.unwrap() > record.created_at_ms);
}

#[tokio::test]
async fn ffmpeg_source_path_reuses_local_path_hint_without_staging() {
    let temp = tempfile::tempdir().unwrap();
    let local_path = temp.path().join("library").join("demo.mkv");
    fs::create_dir_all(local_path.parent().unwrap()).unwrap();
    fs::write(&local_path, b"local-media").unwrap();
    let backend = RemotePlaybackBackend {
        bytes: b"remote-media".to_vec(),
        local_path_hint: Some(local_path.clone()),
    };
    let source = remote_media_source("local:///demo.mkv");
    let uri = StorageUri::parse(&source.locator).unwrap();

    let input_path = source_path_for_ffmpeg_with_backend(
        &source,
        &uri,
        &backend,
        temp.path().join("remux").join("inputs"),
    )
    .await
    .unwrap();

    assert_eq!(input_path, local_path);
}

#[test]
fn remux_staging_policy_rejects_escaping_roots() {
    assert!(RemuxStagingPolicy::new(PathBuf::new()).is_err());
    assert!(RemuxStagingPolicy::new(PathBuf::from("cache/../outside")).is_err());

    let policy = RemuxStagingPolicy::new(PathBuf::from("cache/remux")).unwrap();
    let source = remote_media_source("local:///demo.mkv");
    let request_identity = local_remux_request_identity(&source, RemuxContainer::Mkv);
    let output = policy
        .output_path(MediaSourceId::new(), &request_identity, RemuxContainer::Mkv)
        .unwrap();

    assert!(output.starts_with(PathBuf::from("cache/remux")));
    assert_eq!(
        output.extension().and_then(|value| value.to_str()),
        Some("mkv")
    );
}

#[test]
fn hls_staging_policy_rejects_escaping_roots() {
    assert!(HlsStagingPolicy::new(PathBuf::new()).is_err());
    assert!(HlsStagingPolicy::new(PathBuf::from("cache/../outside")).is_err());

    let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
    let source = remote_media_source("local:///demo.mkv");
    let request_identity = local_hls_request_identity(&source, HardwareAcceleration::None);
    let layout = policy
        .single_variant_layout(MediaSourceId::new(), &request_identity)
        .unwrap();

    assert!(layout.output_dir.starts_with(PathBuf::from("cache/hls")));
    assert!(layout.playlist_path.starts_with(PathBuf::from("cache/hls")));
    assert!(
        layout
            .segment_pattern
            .starts_with(PathBuf::from("cache/hls"))
    );
    assert_eq!(
        layout
            .playlist_path
            .file_name()
            .and_then(|value| value.to_str()),
        Some("playlist.m3u8")
    );
}
