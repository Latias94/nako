use super::*;

#[tokio::test]
async fn scan_library_persists_job_success() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
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

    let output = app.scan_library(library_id).await.unwrap();
    let job = app.get_job(output.job.id).await.unwrap();
    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(job.status, JobStatus::Succeeded);
    assert_eq!(output.index.discovered_files, 0);
    assert_eq!(output.probe.total_sources, 0);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, DomainEventKind::LibraryScanned);
    assert_eq!(events[0].subject, DomainEventSubject::Library(library_id));
    assert!(events[0].payload_json.contains(&output.job.id.to_string()));
    assert!(
        !events[0]
            .payload_json
            .contains(&temp.path().display().to_string())
    );
}

#[tokio::test]
async fn app_startup_rejects_duplicate_configured_library_ids() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
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
        libraries: vec![
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: taru_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().join("anime"),
                preset: taru_core::LibraryPreset::Anime,
                webdav: None,
            },
        ],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();

    let err = TaruApp::new_with_store(config, store).await.unwrap_err();

    let TaruError::InvalidInput { message } = err else {
        panic!("expected duplicate library id validation error");
    };
    assert!(message.contains("duplicate configured library id"));
    assert!(message.contains(&library_id.to_string()));
}

#[tokio::test]
async fn app_startup_rejects_duplicate_metadata_provider_configs() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.providers = vec![
        MetadataProviderConfig {
            provider: ExternalProvider::Tmdb,
            enabled: false,
            token_env: None,
            api_key_env: None,
            api_base_url: None,
            image_base_url: None,
            language: None,
            include_adult: false,
            headers: Vec::new(),
            runtime: None,
        },
        MetadataProviderConfig {
            provider: ExternalProvider::Tmdb,
            enabled: false,
            token_env: None,
            api_key_env: None,
            api_base_url: None,
            image_base_url: None,
            language: None,
            include_adult: false,
            headers: Vec::new(),
            runtime: None,
        },
    ];
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("taru-cache").join("remux"),
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();

    let err = TaruApp::new_with_store(config, store).await.unwrap_err();

    let TaruError::InvalidInput { message } = err else {
        panic!("expected duplicate provider validation error");
    };
    assert_eq!(message, "duplicate metadata provider config: tmdb");
}

#[tokio::test]
async fn app_startup_marks_stale_transcode_sessions_failed() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let config = app.config().clone();
    let staging = RemuxStagingPolicy::new(&config.remux_staging_root).unwrap();
    let stale_id = TranscodeSessionId::new();

    store
        .create_transcode_session(NewTranscodeSession {
            id: stale_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: RemuxRequestKey {
                source_id: source.id,
                output_container: RemuxContainer::Mp4,
            }
            .persisted_request_key(),
            output_path: staging.output_path(source.id, RemuxContainer::Mp4).unwrap(),
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
