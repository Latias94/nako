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
    let restarted = NakoApp::new_with_store(config, store.clone())
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
async fn remux_source_currently_starts_without_principal_or_playback_policy() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "policy_gap");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;

    let output = app
        .playback()
        .remux_source(RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        })
        .await
        .unwrap();

    assert_eq!(output.disposition, RemuxSourceDisposition::Finished);
    assert_eq!(
        output.session.as_ref().unwrap().kind,
        TranscodeSessionKind::Remux
    );
    assert_eq!(
        output.session.as_ref().unwrap().state,
        TranscodeSessionState::Finished
    );
}

#[tokio::test]
async fn direct_playback_policy_denial_does_not_create_session() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "policy_denied_direct");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = local_playback_viewer(&store, source.library_id).await;
    let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
    permissions.allow_direct_play = false;
    store
        .upsert_playback_policy(&PlaybackPolicy::user(
            principal.user_id,
            source.library_id,
            permissions,
            2,
        ))
        .await
        .unwrap();

    let err = app
        .playback()
        .direct_playback_stream(DirectPlaybackStreamRequest {
            principal: principal.clone(),
            source_id: source.id,
            range_request: DirectPlayRangeRequest::None,
            client: ClientPlaybackCapabilities::default(),
        })
        .await
        .unwrap_err();

    let NakoError::Forbidden { message } = err else {
        panic!("expected playback policy forbidden error");
    };
    assert!(message.contains("direct_play"));
    assert!(
        store
            .list_playback_sessions(
                PlaybackSessionListFilter {
                    principal_id: Some(principal.principal_id),
                    source_id: Some(source.id),
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn remux_playback_policy_denial_does_not_create_sessions_or_artifacts() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "policy_denied_remux");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = local_playback_viewer(&store, source.library_id).await;
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

    let err = app
        .playback()
        .remux_playback_stream(RemuxPlaybackStreamRequest {
            principal: principal.clone(),
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
            range_request: DirectPlayRangeRequest::None,
        })
        .await
        .unwrap_err();

    let NakoError::Forbidden { message } = err else {
        panic!("expected playback policy forbidden error");
    };
    assert!(message.contains("remux"));
    assert!(
        store
            .list_playback_sessions(
                PlaybackSessionListFilter {
                    principal_id: Some(principal.principal_id),
                    source_id: Some(source.id),
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
                    source_id: Some(source.id),
                    kind: Some(TranscodeSessionKind::Remux),
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn hls_playback_policy_denial_does_not_create_sessions_or_artifacts() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "policy_denied_hls");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = local_playback_viewer(&store, source.library_id).await;
    let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
    permissions.allow_video_transcode = false;
    store
        .upsert_playback_policy(&PlaybackPolicy::user(
            principal.user_id,
            source.library_id,
            permissions,
            2,
        ))
        .await
        .unwrap();

    let err = app
        .playback()
        .hls_playlist_playback(HlsPlaylistPlaybackRequest {
            principal: principal.clone(),
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::default(),
            transport_query: None,
        })
        .await
        .unwrap_err();

    let NakoError::Forbidden { message } = err else {
        panic!("expected playback policy forbidden error");
    };
    assert!(message.contains("video_transcode"));
    assert!(
        store
            .list_playback_sessions(
                PlaybackSessionListFilter {
                    principal_id: Some(principal.principal_id),
                    source_id: Some(source.id),
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
                    source_id: Some(source.id),
                    kind: Some(TranscodeSessionKind::HlsTranscode),
                    state: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
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

    let NakoError::Conflict { message } = err else {
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

    let NakoError::Provider { provider, message } = err else {
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
    let app = NakoApp::new_with_store(config, store.clone())
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

    let NakoError::Provider { provider, message } = err else {
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
    let _restarted = NakoApp::new_with_store(config, store.clone())
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
        preferences: PlaybackPreferenceContext::default(),
        playback_generation: HlsPlaybackGeneration::default(),
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
    assert_eq!(segment.response.content_type, "video/mp2t");
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
    let persisted_session = store
        .get_transcode_session(session_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted_session.runtime_metrics.frame_count, Some(12));
    assert_eq!(
        persisted_session.runtime_metrics.output_time_ms,
        Some(1_500)
    );

    fs::remove_file(ffmpeg_path).unwrap();
    let reused = app.playback().hls_source(request.clone()).await.unwrap();
    assert_eq!(reused.disposition, HlsSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.id, session_id);

    let config = app.config().clone();
    drop(app);
    let restarted = NakoApp::new_with_store(config, store.clone())
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
async fn hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_running_hls_ffmpeg_script(script_root.path(), "hls_running_playlist");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = local_playback_viewer(&store, source.library_id).await;

    let playlist = tokio::time::timeout(
        std::time::Duration::from_millis(800),
        app.playback()
            .hls_playlist_playback(HlsPlaylistPlaybackRequest {
                principal,
                source_id: source.id,
                client: ClientPlaybackCapabilities::default(),
                preferences: PlaybackPreferenceContext::default(),
                playback_generation: HlsPlaybackGeneration::default(),
                transport_query: None,
            }),
    )
    .await
    .expect("hls playlist should be returned before the runner exits")
    .unwrap();

    let transcode_session_id = playlist
        .session
        .transcode_session_id
        .expect("hls playback session should link a transcode session");
    let running = store
        .get_transcode_session(transcode_session_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(running.state, TranscodeSessionState::Running);
    assert!(playlist.body.contains("#EXTM3U"));
    assert!(playlist.body.contains(&format!(
        "/playback/sessions/{}/hls/segments/segment_00000.ts",
        playlist.session.id
    )));

    let segment = app
        .playback()
        .plan_hls_segment(transcode_session_id, "segment_00000.ts")
        .await
        .unwrap();
    assert_eq!(segment.response.content_type, "video/mp2t");
    assert!(segment.path.ends_with("segment_00000.ts"));

    let missing = app
        .playback()
        .plan_hls_segment(transcode_session_id, "segment_00001.ts")
        .await
        .unwrap_err();
    let NakoError::Conflict { message } = missing else {
        panic!("expected missing running hls segment readiness conflict");
    };
    assert!(message.contains("is not ready"));

    app.playback()
        .cancel_playback_session(playlist.session.id)
        .await
        .unwrap();
    wait_for_transcode_state(
        &store,
        transcode_session_id,
        TranscodeSessionState::Cancelled,
    )
    .await;
}

#[tokio::test]
async fn hls_source_selected_audio_stream_reaches_ffmpeg_map() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path =
        fake_hls_ffmpeg_script_requiring_audio_map(script_root.path(), "hls_audio_map", "0:2");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
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
            },
        )
        .await
        .unwrap();

    let output = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            preferences: PlaybackPreferenceContext {
                requested_audio_stream: Some(2),
                ..PlaybackPreferenceContext::default()
            },
            playback_generation: HlsPlaybackGeneration::default(),
        })
        .await
        .unwrap();

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(output.session.request_key.contains("audio%3D2"));
}

#[tokio::test]
async fn hls_source_multi_audio_generates_audio_sidecar_renditions_and_artifacts() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_audio_sidecars");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
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
            },
        )
        .await
        .unwrap();
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
        preferences: PlaybackPreferenceContext {
            requested_audio_stream: Some(2),
            ..PlaybackPreferenceContext::default()
        },
        playback_generation: HlsPlaybackGeneration::default(),
    };

    let output = app.playback().hls_source(request.clone()).await.unwrap();
    let session_id = output.session.id;

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(output.session.request_key.contains("audio%3D2"));
    assert!(
        output
            .session
            .request_key
            .contains(";request_variant=hls-media-renditions:v1%3Baudios%3D0:1:0:eng|1:2:1:jpn")
    );
    assert!(output.segment_dir.join("audio_0.m3u8").exists());
    assert!(output.segment_dir.join("audio_0_00000.aac").exists());
    assert!(output.segment_dir.join("audio_1.m3u8").exists());
    assert!(output.segment_dir.join("audio_1_00000.aac").exists());
    assert!(
        fs::read_to_string(output.segment_dir.join("audio_1.m3u8"))
            .unwrap()
            .contains("audio_1_00000.aac")
    );

    let playlist = app.playback().hls_playlist(request.clone()).await.unwrap();
    assert!(playlist.body.contains("#EXT-X-MEDIA:TYPE=AUDIO"));
    assert!(playlist.body.contains("GROUP-ID=\"nako-audio\""));
    assert!(playlist.body.contains("NAME=\"eng\",DEFAULT=NO"));
    assert!(playlist.body.contains("NAME=\"jpn\",DEFAULT=YES"));
    assert!(playlist.body.contains(&format!(
        "URI=\"/playback/sessions/{session_id}/hls/segments/audio_0.m3u8\""
    )));
    assert!(playlist.body.contains(&format!(
        "AUDIO=\"nako-audio\"\n/playback/sessions/{session_id}/hls/segments/playlist.m3u8"
    )));

    let audio_playlist = app
        .playback()
        .plan_hls_segment(session_id, "audio_0.m3u8")
        .await
        .unwrap();
    assert_eq!(
        audio_playlist.response.content_type,
        "application/vnd.apple.mpegurl"
    );
    let audio_segment = app
        .playback()
        .plan_hls_segment(session_id, "audio_1_00000.aac")
        .await
        .unwrap();
    assert_eq!(audio_segment.response.content_type, "audio/aac");

    fs::remove_file(ffmpeg_path).unwrap();
    let reused = app.playback().hls_source(request).await.unwrap();

    assert_eq!(reused.disposition, HlsSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.id, session_id);
}

#[tokio::test]
async fn hls_source_runs_fmp4_runtime_layout_and_rewrites_init_map() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_fmp4_success");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities {
            hls_segment_container: nako_playback::PlaybackHlsSegmentContainer::Fmp4,
            ..ClientPlaybackCapabilities::default()
        },
        preferences: PlaybackPreferenceContext::default(),
        playback_generation: HlsPlaybackGeneration::default(),
    };

    let output = app.playback().hls_source(request.clone()).await.unwrap();
    let session_id = output.session.id;

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(output.session.request_key.contains("hls_segment%3Dfmp4"));
    assert!(output.segment_dir.join("init.mp4").exists());
    assert_eq!(
        fs::read_to_string(output.segment_dir.join("segment_00000.m4s")).unwrap(),
        "segment"
    );
    assert!(!output.segment_dir.join("segment_00000.ts").exists());

    let playlist = app.playback().hls_playlist(request).await.unwrap();
    assert!(playlist.body.contains(&format!(
        "#EXT-X-MAP:URI=\"/playback/sessions/{session_id}/hls/segments/init.mp4\""
    )));
    assert!(playlist.body.contains(&format!(
        "/playback/sessions/{session_id}/hls/segments/segment_00000.m4s"
    )));

    let init = app
        .playback()
        .plan_hls_segment(session_id, "init.mp4")
        .await
        .unwrap();
    assert_eq!(init.response.content_type, "video/mp4");

    let segment = app
        .playback()
        .plan_hls_segment(session_id, "segment_00000.m4s")
        .await
        .unwrap();
    assert_eq!(segment.response.content_type, "video/mp4");
}

#[tokio::test]
async fn hls_source_runs_adaptive_fmp4_ladder_and_rewrites_master_playlist() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_adaptive_success");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities {
            hls_variant_policy: nako_playback::PlaybackHlsVariantPolicy::Adaptive,
            hls_segment_container: nako_playback::PlaybackHlsSegmentContainer::Fmp4,
            ..ClientPlaybackCapabilities::default()
        },
        preferences: PlaybackPreferenceContext::default(),
        playback_generation: HlsPlaybackGeneration::default(),
    };

    let output = app.playback().hls_source(request.clone()).await.unwrap();
    let session_id = output.session.id;

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(output.playlist_path.ends_with("master.m3u8"));
    assert!(output.session.request_key.contains("kind%3Dhls_adaptive"));
    assert!(
        output
            .session
            .request_key
            .contains("hls_variant%3Dadaptive")
    );
    assert!(output.session.request_key.contains("hls_segment%3Dfmp4"));
    assert!(output.segment_dir.join("variant_0.m3u8").exists());
    assert!(output.segment_dir.join("variant_1.m3u8").exists());
    assert!(output.segment_dir.join("variant_0_init.mp4").exists());
    assert_eq!(
        fs::read_to_string(output.segment_dir.join("variant_0_segment_00000.m4s")).unwrap(),
        "segment"
    );

    let playlist = app.playback().hls_playlist(request).await.unwrap();
    assert!(playlist.body.contains(&format!(
        "/playback/sessions/{session_id}/hls/segments/variant_0.m3u8"
    )));
    assert!(playlist.body.contains(&format!(
        "/playback/sessions/{session_id}/hls/segments/variant_1.m3u8"
    )));

    let variant_playlist = app
        .playback()
        .plan_hls_segment(session_id, "variant_0.m3u8")
        .await
        .unwrap();
    assert_eq!(
        variant_playlist.response.content_type,
        "application/vnd.apple.mpegurl"
    );

    let init = app
        .playback()
        .plan_hls_segment(session_id, "variant_0_init.mp4")
        .await
        .unwrap();
    assert_eq!(init.response.content_type, "video/mp4");

    let segment = app
        .playback()
        .plan_hls_segment(session_id, "variant_0_segment_00000.m4s")
        .await
        .unwrap();
    assert_eq!(segment.response.content_type, "video/mp4");
    assert!(
        app.playback()
            .plan_hls_segment(session_id, "init.mp4")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn hls_source_selected_subtitle_uses_sidecar_rendition_identity_and_artifacts() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_subtitle_sidecar");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
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
                        kind: MediaStreamKind::Subtitle,
                        codec: Some("subrip".to_owned()),
                        language: Some("jpn".to_owned()),
                        duration_ms: None,
                        bit_rate: None,
                        width: None,
                        height: None,
                        channels: None,
                        sample_rate: None,
                        technical: Default::default(),
                    },
                ],
            },
        )
        .await
        .unwrap();
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
        preferences: PlaybackPreferenceContext {
            requested_subtitle_stream: Some(2),
            ..PlaybackPreferenceContext::default()
        },
        playback_generation: HlsPlaybackGeneration::default(),
    };

    let output = app.playback().hls_source(request.clone()).await.unwrap();
    let session_id = output.session.id;

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(
        output
            .session
            .request_key
            .contains("subtitle_strategy%3Dsidecar_selected")
    );
    assert!(
        output
            .session
            .request_key
            .contains(";request_variant=hls-media-renditions:v1%3Bsubtitles%3D0:2:jpn")
    );
    assert!(output.segment_dir.join("subtitle_0.m3u8").exists());
    assert!(
        fs::read_to_string(output.segment_dir.join("subtitle_0.m3u8"))
            .unwrap()
            .contains("subtitle_0_00000.vtt")
    );
    assert!(output.segment_dir.join("subtitle_0_00000.vtt").exists());

    let playlist = app.playback().hls_playlist(request.clone()).await.unwrap();
    assert!(playlist.body.contains("#EXT-X-MEDIA:TYPE=SUBTITLES"));
    assert!(playlist.body.contains("GROUP-ID=\"nako-subtitles\""));
    assert!(playlist.body.contains("LANGUAGE=\"jpn\""));
    assert!(playlist.body.contains(&format!(
        "URI=\"/playback/sessions/{session_id}/hls/segments/subtitle_0.m3u8\""
    )));
    assert!(playlist.body.contains(&format!(
        "SUBTITLES=\"nako-subtitles\"\n/playback/sessions/{session_id}/hls/segments/playlist.m3u8"
    )));

    let subtitle_playlist = app
        .playback()
        .plan_hls_segment(session_id, "subtitle_0.m3u8")
        .await
        .unwrap();
    assert_eq!(
        subtitle_playlist.response.content_type,
        "application/vnd.apple.mpegurl"
    );
    let subtitle_segment = app
        .playback()
        .plan_hls_segment(session_id, "subtitle_0_00000.vtt")
        .await
        .unwrap();
    assert_eq!(subtitle_segment.response.content_type, "text/vtt");

    fs::remove_file(ffmpeg_path).unwrap();
    let reused = app.playback().hls_source(request).await.unwrap();

    assert_eq!(reused.disposition, HlsSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.id, session_id);
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
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::default(),
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
async fn hls_source_falls_back_to_cpu_when_source_facts_do_not_match_hardware_decode() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_source_cpu_fallback");
    let (_temp, app, store, source) = remux_app_with_source_and_transcode(
        ffmpeg_path,
        TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Vaapi,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 1,
            gpu_concurrency: 1,
        },
    )
    .await;
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![
                    MediaStreamInfo {
                        index: 0,
                        kind: MediaStreamKind::Video,
                        codec: Some("hevc".to_owned()),
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

    let output = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::default(),
        })
        .await
        .unwrap();

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(output.session.request_key.contains("requested%3Dvaapi"));
    assert!(output.session.request_key.contains("encode%3Dnone"));
    assert!(output.session.request_key.contains("fallback_used%3Dtrue"));
}

#[tokio::test]
async fn hls_source_request_identity_separates_selected_acceleration_profiles() {
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
        preferences: PlaybackPreferenceContext::default(),
        playback_generation: HlsPlaybackGeneration::default(),
    };

    let cpu_output = app.playback().hls_source(request.clone()).await.unwrap();
    assert_eq!(cpu_output.disposition, HlsSourceDisposition::Finished);
    assert!(
        cpu_output
            .session
            .request_key
            .contains("kind%3Dhls_single_variant")
    );
    assert!(cpu_output.session.request_key.contains("encode%3Dnone"));

    let mut config = app.config().clone();
    config.ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_nvenc_profile");
    drop(app);
    let gpu_app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let gpu_output = gpu_app.playback().hls_source(request).await.unwrap();

    assert_eq!(gpu_output.disposition, HlsSourceDisposition::Finished);
    assert_ne!(gpu_output.session.id, cpu_output.session.id);
    assert_ne!(gpu_output.playlist_path, cpu_output.playlist_path);
    assert!(gpu_output.session.request_key.contains("encode%3Dnvenc"));
}

#[tokio::test]
async fn hls_source_request_identity_changes_when_source_revision_changes() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_source_revision");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
        preferences: PlaybackPreferenceContext::default(),
        playback_generation: HlsPlaybackGeneration::default(),
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
async fn hls_source_request_identity_separates_seek_generation() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_seek_generation_identity");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;
    let initial = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
        preferences: PlaybackPreferenceContext::default(),
        playback_generation: HlsPlaybackGeneration::default(),
    };
    let seeked = HlsSourceRequest {
        playback_generation: HlsPlaybackGeneration::from_start_position_ms(45_000),
        ..initial.clone()
    };

    let initial_output = app.playback().hls_source(initial.clone()).await.unwrap();
    let seeked_output = app.playback().hls_source(seeked.clone()).await.unwrap();

    assert_eq!(initial_output.disposition, HlsSourceDisposition::Finished);
    assert_eq!(seeked_output.disposition, HlsSourceDisposition::Finished);
    assert_ne!(initial_output.session.id, seeked_output.session.id);
    assert_ne!(
        initial_output.session.request_key,
        seeked_output.session.request_key
    );
    assert_ne!(initial_output.playlist_path, seeked_output.playlist_path);
    assert!(
        !initial_output
            .session
            .request_key
            .contains("hls-playback-generation")
    );
    assert!(
        seeked_output
            .session
            .request_key
            .contains(";request_variant=hls-playback-generation:v1%3Bstart_ms%3D45000")
    );

    let reused_seek = app.playback().hls_source(seeked).await.unwrap();
    assert_eq!(
        reused_seek.disposition,
        HlsSourceDisposition::ReusedExisting
    );
    assert_eq!(reused_seek.session.id, seeked_output.session.id);
}

#[tokio::test]
async fn hls_source_seek_generation_reaches_ffmpeg_command() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path =
        fake_hls_ffmpeg_script_requiring_seek(script_root.path(), "hls_seek_command", "45.250");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;

    let output = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::from_start_position_ms(45_250),
        })
        .await
        .unwrap();

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(
        output
            .session
            .request_key
            .contains("hls-playback-generation:v1%3Bstart_ms%3D45250")
    );
}

#[tokio::test]
async fn hls_source_adaptive_identity_includes_source_aware_ladder() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_adaptive_ladder_identity");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![MediaStreamInfo {
                    index: 0,
                    kind: MediaStreamKind::Video,
                    codec: Some("h264".to_owned()),
                    language: None,
                    duration_ms: None,
                    bit_rate: Some(4_000_000),
                    width: Some(1920),
                    height: Some(1080),
                    channels: None,
                    sample_rate: None,
                    technical: Default::default(),
                }],
            },
        )
        .await
        .unwrap();
    let client = ClientPlaybackCapabilities {
        hls_variant_policy: nako_playback::PlaybackHlsVariantPolicy::Adaptive,
        hls_segment_container: nako_playback::PlaybackHlsSegmentContainer::Fmp4,
        max_video_bitrate: Some(2_000_000),
        max_width: Some(1280),
        max_height: Some(720),
        ..ClientPlaybackCapabilities::default()
    };

    let output = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client,
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::default(),
        })
        .await
        .unwrap();

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert!(
        output
            .session
            .request_key
            .contains(";request_variant=hls-adaptive-ladder:v1%3Baudio%3Dfalse")
    );
    assert!(
        output
            .session
            .request_key
            .contains("0:1280x720@2000000+128000")
    );
    assert!(output.playlist_path.ends_with("master.m3u8"));
}

#[tokio::test]
async fn hls_service_degrades_unavailable_gpu_when_fallback_is_fail() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_cpu_only_hls_ffmpeg_script(script_root.path(), "hls_gpu_required");
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
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
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let app = NakoApp::new_with_store(config, store).await.unwrap();
    let diagnostics = app.playback().runtime_diagnostics();

    assert_eq!(
        diagnostics.hls_pipeline_readiness.status,
        nako_transcode::TranscodePipelineReadinessStatus::Unavailable
    );
    assert_eq!(
        diagnostics.hls_pipeline_readiness.reason,
        nako_transcode::TranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy
    );
    assert_eq!(diagnostics.selected_hls_slots, 0);
}

#[tokio::test]
async fn hls_source_rejects_persisted_active_duplicate() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let staging = HlsStagingPolicy::new(app.config().remux_staging_root.join("hls")).unwrap();
    let request_identity = local_hls_request_identity(&source, HardwareAcceleration::None);
    let layout = staging
        .single_variant_layout(
            source.id,
            &request_identity,
            nako_transcode::HlsOutputRequirement::default(),
        )
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
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::default(),
        })
        .await
        .unwrap_err();

    let NakoError::Conflict { message } = err else {
        panic!("expected hls duplicate conflict");
    };
    assert!(message.contains("already in progress"));
    assert!(message.contains(&active.id.to_string()));

    let segment_err = app
        .playback()
        .plan_hls_segment(active.id, "segment_00000.ts")
        .await
        .unwrap_err();
    let NakoError::Conflict { message } = segment_err else {
        panic!("expected hls segment readiness conflict");
    };
    assert!(message.contains("is not ready"));
}

#[tokio::test]
async fn hls_source_seek_generation_supersedes_active_prior_generation() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_seek_supersede");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let staging = HlsStagingPolicy::new(app.config().remux_staging_root.join("hls")).unwrap();
    let request_identity = local_hls_request_identity(&source, HardwareAcceleration::None);
    let layout = staging
        .single_variant_layout(
            source.id,
            &request_identity,
            nako_transcode::HlsOutputRequirement::default(),
        )
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

    let seeked = app
        .playback()
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::from_start_position_ms(45_000),
        })
        .await
        .unwrap();
    let superseded = store
        .get_transcode_session(active.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(seeked.disposition, HlsSourceDisposition::Finished);
    assert_ne!(seeked.session.id, active.id);
    assert_eq!(superseded.state, TranscodeSessionState::CancelRequested);
    assert_eq!(
        superseded.failure_category,
        Some(TranscodeFailureCategory::Cancelled)
    );
    assert!(
        superseded
            .failure_message
            .as_deref()
            .is_some_and(|message| message.contains("superseded by hls request"))
    );
    assert!(
        seeked
            .session
            .request_key
            .contains("hls-playback-generation:v1%3Bstart_ms%3D45000")
    );
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
            preferences: PlaybackPreferenceContext::default(),
            playback_generation: HlsPlaybackGeneration::default(),
        })
        .await
        .unwrap_err();

    let NakoError::Provider { provider, message } = err else {
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
    let range = RequestedByteRange {
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
                preset: nako_core::LibraryPreset::Movies,
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
        .single_variant_layout(
            MediaSourceId::new(),
            &request_identity,
            nako_transcode::HlsOutputRequirement::default(),
        )
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

#[test]
fn hls_staging_policy_uses_segment_container_in_layout() {
    let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
    let source = remote_media_source("local:///demo.mkv");
    let request_identity = local_hls_request_identity(&source, HardwareAcceleration::None);
    let fmp4 = nako_transcode::HlsOutputRequirement {
        variant_policy: nako_transcode::HlsVariantPolicy::SingleVariant,
        segment_container: nako_transcode::HlsSegmentContainer::Fmp4,
    };

    let layout = policy
        .single_variant_layout(source.id, &request_identity, fmp4)
        .unwrap();

    assert_eq!(layout.output, fmp4);
    assert!(layout.segment_pattern.ends_with("segment_%05d.m4s"));
}

#[test]
fn hls_staging_policy_uses_adaptive_fmp4_layout() {
    let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
    let source = remote_media_source("local:///demo.mkv");
    let request_identity = local_hls_request_identity(&source, HardwareAcceleration::None);
    let adaptive = nako_transcode::HlsOutputRequirement {
        variant_policy: nako_transcode::HlsVariantPolicy::Adaptive,
        segment_container: nako_transcode::HlsSegmentContainer::Fmp4,
    };

    let layout = policy
        .layout_for_output(source.id, &request_identity, adaptive)
        .unwrap();

    assert_eq!(layout.output, adaptive);
    assert!(layout.playlist_path.ends_with("master.m3u8"));
    assert!(
        layout
            .segment_pattern
            .ends_with("variant_%v_segment_%05d.m4s")
    );
    assert_eq!(layout.artifacts.renditions().len(), 2);
    assert!(
        layout
            .artifacts
            .variant_playlist_pattern()
            .unwrap()
            .ends_with("variant_%v.m3u8")
    );
}

#[test]
fn hls_staging_policy_uses_source_aware_adaptive_plan() {
    let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
    let source = remote_media_source("local:///demo.mkv");
    let plan = nako_transcode::HlsAdaptiveLadderPlan::from_source(
        nako_transcode::HlsAdaptiveLadderSource {
            width: Some(640),
            height: Some(360),
            video_bitrate: Some(700_000),
            has_audio: Some(false),
        },
        Default::default(),
    );
    let profile = local_hls_request_identity(&source, HardwareAcceleration::None);
    let request_identity = profile
        .profile_identity()
        .bind_source_with_request_variant(profile.source_identity(), plan.identity_key());
    let adaptive = nako_transcode::HlsOutputRequirement {
        variant_policy: nako_transcode::HlsVariantPolicy::Adaptive,
        segment_container: nako_transcode::HlsSegmentContainer::Fmp4,
    };

    let layout = policy
        .layout_for_output_with_adaptive_plan(source.id, &request_identity, adaptive, &plan)
        .unwrap();

    assert!(!layout.artifacts.has_audio());
    assert_eq!(layout.artifacts.renditions(), plan.renditions());
    assert!(layout.artifacts.artifact_for_name("variant_0.m3u8").is_ok());
    assert!(
        layout
            .artifacts
            .artifact_for_name("variant_1.m3u8")
            .is_err()
    );
}

#[test]
fn hls_staging_policy_carries_selected_subtitle_media_renditions() {
    let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
    let source = remote_media_source("local:///demo.mkv");
    let media = nako_transcode::HlsMediaRenditionPlan::from_subtitles(vec![
        nako_transcode::HlsSubtitleRendition::new(0, 2, Some("eng".to_owned())),
    ])
    .unwrap();
    let request_variant = nako_transcode::HlsRequestVariantPlan::new(None, media.clone());
    let profile = local_hls_request_identity(&source, HardwareAcceleration::None);
    let request_identity = profile.profile_identity().bind_source_with_request_variant(
        profile.source_identity(),
        request_variant.identity_key().unwrap(),
    );

    let layout = policy
        .layout_for_output_with_request_variant_plan(
            source.id,
            &request_identity,
            nako_transcode::HlsOutputRequirement::default(),
            &request_variant,
        )
        .unwrap();

    assert_eq!(layout.artifacts.media_renditions(), &media);
    assert!(
        layout
            .artifacts
            .artifact_for_name("subtitle_0.m3u8")
            .is_ok()
    );
    assert!(
        layout
            .artifacts
            .artifact_for_name("subtitle_0_00000.vtt")
            .is_ok()
    );
}

async fn local_playback_viewer(
    store: &NakoDatabase,
    library_id: LibraryId,
) -> AuthenticatedPrincipal {
    let user_id = UserId::new();
    let principal_id = UserPrincipalId::new(format!("local-user:{user_id}")).unwrap();
    let user = User {
        id: user_id,
        principal_id: principal_id.clone(),
        username: format!("viewer-{user_id}"),
        display_name: "Playback viewer".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1,
        updated_at_ms: 1,
    };

    store.upsert_user(&user).await.unwrap();
    store
        .replace_role_assignments(
            user_id,
            &[RoleAssignment {
                user_id,
                role: UserRole::Viewer,
                granted_at_ms: 1,
            }],
        )
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&LibraryAccessPolicy {
            scope: LibraryAccessPolicyScope::User(user_id),
            library_id,
            access: LibraryAccessLevel::Play,
            created_at_ms: 1,
            updated_at_ms: 1,
        })
        .await
        .unwrap();

    AuthenticatedPrincipal {
        user_id,
        principal_id,
        roles: vec![UserRole::Viewer],
        bootstrap: false,
    }
}

async fn wait_for_transcode_state(
    store: &NakoDatabase,
    session_id: TranscodeSessionId,
    expected: TranscodeSessionState,
) {
    tokio::time::timeout(std::time::Duration::from_millis(800), async {
        loop {
            let state = store
                .get_transcode_session(session_id)
                .await
                .unwrap()
                .unwrap()
                .state;
            if state == expected {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("expected transcode session {session_id} to reach {expected:?}"));
}
