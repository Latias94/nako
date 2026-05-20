use super::*;

async fn wait_for_runtime_jobs(
    app: &TaruApp,
    succeeded_jobs: u64,
    cancelled_jobs: u64,
    failed_jobs: u64,
) -> RuntimeSupervisorDiagnostics {
    for _ in 0..100 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == succeeded_jobs
            && diagnostics.cancelled_jobs == cancelled_jobs
            && diagnostics.failed_jobs == failed_jobs
        {
            return diagnostics;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "runtime job diagnostics did not reach expected state: {:?}",
        app.runtime_diagnostics()
    );
}

async fn upsert_item_with_source(
    store: &TaruDatabase,
    library_id: LibraryId,
    item: &MediaItem,
) -> MediaSource {
    let source = media_source_for_item(library_id, item);
    store.upsert_media_item(item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    source
}

fn media_source_for_item(library_id: LibraryId, item: &MediaItem) -> MediaSource {
    MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: format!("local:///{}.mkv", item.metadata.title),
        file_name: format!("{}.mkv", item.metadata.title),
        size_bytes: Some(1024),
        fingerprint: None,
    }
}

#[tokio::test]
async fn metadata_refresh_job_input_does_not_include_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.providers = vec![MetadataProviderConfig {
        provider: ExternalProvider::Tmdb,
        enabled: true,
        token_env: Some("TARU_TEST_MISSING_TMDB_TOKEN".to_owned()),
        api_key_env: None,
        api_base_url: None,
        image_base_url: None,
        language: None,
        include_adult: false,
        headers: Vec::new(),
        runtime: None,
    }];
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;

    let job = app
        .metadata()
        .create_metadata_refresh_job(item.id)
        .await
        .unwrap();
    let input = job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(job.kind, JobKind::MetadataRefresh);
    assert_eq!(job.resource_class, "metadata.tmdb");
    assert_eq!(job.library_id, Some(library_id));
    assert_eq!(
        input.get("item_id").and_then(serde_json::Value::as_str),
        Some(item.id.to_string().as_str())
    );
    assert_eq!(
        input.get("provider").and_then(serde_json::Value::as_str),
        Some("tmdb")
    );
    assert_eq!(
        input
            .get("refresh_mode")
            .and_then(serde_json::Value::as_str),
        Some("default")
    );
    assert!(input.get("access_token").is_none());
    assert!(input.get("api_key").is_none());
}

#[tokio::test]
async fn metadata_refresh_uses_reconciled_library_profile() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Configured Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Anime);
    options.metadata_profile.metadata_providers = vec![ExternalProvider::Bangumi];
    options.metadata_profile.refresh_mode = MetadataRefreshMode::MissingOnly;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Persisted Anime".to_owned(),
            roots: vec!["local:///".to_owned()],
            options,
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;

    let job = app
        .metadata()
        .create_metadata_refresh_job(item.id)
        .await
        .unwrap();
    let input = job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(job.resource_class, "metadata.bangumi");
    assert_eq!(
        input.get("provider").and_then(serde_json::Value::as_str),
        Some("bangumi")
    );
    assert_eq!(
        input
            .get("refresh_mode")
            .and_then(serde_json::Value::as_str),
        Some("missing_only")
    );
}

#[tokio::test]
async fn metadata_refresh_job_records_disabled_profile_provider_for_executor() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.providers = vec![MetadataProviderConfig {
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
    }];
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;

    let job = app
        .metadata()
        .create_metadata_refresh_job(item.id)
        .await
        .unwrap();
    let err = app
        .metadata()
        .run_metadata_refresh(job.id, item.id)
        .await
        .unwrap_err();

    assert_eq!(job.kind, JobKind::MetadataRefresh);
    assert_eq!(job.resource_class, "metadata.tmdb");
    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("tmdb=skipped_disabled"));
    assert!(message.contains("disabled in config"));
}

#[tokio::test]
async fn background_metadata_refresh_job_uses_runtime_job_supervision() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.providers = vec![MetadataProviderConfig {
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
    }];
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;

    let job = app
        .metadata()
        .enqueue_metadata_refresh(item.id)
        .await
        .unwrap();
    let diagnostics = wait_for_runtime_jobs(&app, 0, 0, 1).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();

    assert_eq!(persisted.status, JobStatus::Failed);
    assert_eq!(diagnostics.completed_tasks, 1);
    assert_eq!(diagnostics.failed_tasks, 0);
}

#[tokio::test]
async fn metadata_refresh_falls_back_from_unimplemented_bangumi_to_tmdb_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.providers = vec![MetadataProviderConfig {
        provider: ExternalProvider::Tmdb,
        enabled: true,
        token_env: Some("TARU_TEST_MISSING_TMDB_TOKEN".to_owned()),
        api_key_env: None,
        api_base_url: None,
        image_base_url: None,
        language: None,
        include_adult: false,
        headers: Vec::new(),
        runtime: None,
    }];
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Anime Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;
    let job = app
        .metadata()
        .create_metadata_refresh_job(item.id)
        .await
        .unwrap();
    let err = app
        .metadata()
        .run_metadata_refresh(job.id, item.id)
        .await
        .unwrap_err();

    assert_eq!(job.resource_class, "metadata.bangumi");
    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("bangumi=not_implemented"));
    assert!(message.contains("tmdb=skipped_unavailable"));
    assert_eq!(
        app.jobs().get_job(job.id).await.unwrap().status,
        JobStatus::Queued
    );
}

#[tokio::test]
async fn metadata_refresh_resolves_provider_order_from_library_profile() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Anime Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;
    let job = app
        .metadata()
        .create_metadata_refresh_job(item.id)
        .await
        .unwrap();
    let input = job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(job.resource_class, "metadata.bangumi");
    assert_eq!(
        input.get("provider").and_then(serde_json::Value::as_str),
        Some("bangumi")
    );
}

#[tokio::test]
async fn metadata_maintenance_job_refreshes_library_items_and_summarizes_attempts() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;

    let output = app
        .metadata()
        .run_metadata_maintenance(EnqueueMetadataMaintenanceRequest {
            library_id: Some(library_id),
            item_ids: Vec::new(),
            providers: Some(vec![ExternalProvider::Tmdb]),
            item_kinds: vec![MediaKind::Movie],
            profile: None,
            language: Some("en-US".to_owned()),
            refresh_mode: Some(MetadataRefreshMode::MissingOnly),
            force: false,
        })
        .await
        .unwrap();
    let input = output
        .job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(output.job.kind, JobKind::MetadataMaintenance);
    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(output.summary.requested_items, 1);
    assert_eq!(output.summary.attempted_items, 1);
    assert_eq!(output.summary.succeeded_items, 0);
    assert_eq!(output.summary.failed_items, 1);
    assert_eq!(output.summary.provider_attempts.len(), 1);
    assert_eq!(
        output.summary.provider_attempts[0].status,
        MetadataProviderAttemptStatus::NotImplemented
    );
    assert!(input.get("access_token").is_none());
    assert!(input.get("api_key").is_none());

    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::MetadataMaintenanceCompleted
            && event.subject == DomainEventSubject::Job(output.job.id)
    }));
}

#[tokio::test]
async fn metadata_maintenance_job_acknowledges_cancellation_before_next_item() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let control = BlockingBangumiControl::new();
    let provider_server = BlockingBangumiServer::start(control.clone()).await;
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata: MetadataConfig {
            providers: vec![MetadataProviderConfig {
                provider: ExternalProvider::Bangumi,
                enabled: true,
                token_env: None,
                api_key_env: None,
                api_base_url: Some(provider_server.base_url()),
                image_base_url: None,
                language: None,
                include_adult: false,
                headers: Vec::new(),
                runtime: None,
            }],
            ..MetadataConfig::default()
        },
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let first = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "First Anime".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let second = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Second Anime".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &first).await;
    upsert_item_with_source(&store, library_id, &second).await;

    let job = app
        .metadata()
        .enqueue_metadata_maintenance(EnqueueMetadataMaintenanceRequest {
            library_id: Some(library_id),
            item_ids: Vec::new(),
            providers: Some(vec![ExternalProvider::Bangumi]),
            item_kinds: vec![MediaKind::Movie],
            profile: None,
            language: Some("zh-CN".to_owned()),
            refresh_mode: Some(MetadataRefreshMode::MissingOnly),
            force: false,
        })
        .await
        .unwrap();
    provider_server.control().wait_for_first_search().await;

    let requested = app.jobs().request_job_cancellation(job.id).await.unwrap();
    assert!(requested.requested);
    assert!(!requested.terminal);
    assert_eq!(requested.job.status, JobStatus::Running);

    control.release_first_search();
    let diagnostics = wait_for_runtime_jobs(&app, 0, 1, 0).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();
    let attempts = store.list_metadata_provider_attempts(job.id).await.unwrap();
    let events = store
        .list_outbox_events(Default::default(), PageRequest::new(100, 0))
        .await
        .unwrap();

    assert_eq!(diagnostics.failed_jobs, 0);
    assert_eq!(diagnostics.cancelled_jobs, 1);
    assert_eq!(persisted.status, JobStatus::Cancelled);
    assert_eq!(persisted.summary_json, None);
    assert_eq!(persisted.error, None);
    assert_eq!(attempts.len(), 1);
    assert!(provider_server.control().requests() <= 2);
    assert!(!events.iter().any(|event| {
        event.kind == DomainEventKind::MetadataMaintenanceCompleted
            && event.subject == DomainEventSubject::Job(job.id)
    }));
}

#[tokio::test]
async fn metadata_lifecycle_config_maps_policy_and_cleans_raw_cache_on_startup() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let item_id = MediaItemId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata: MetadataConfig {
            raw_cache_retention_ms: 1,
            maintenance: MetadataMaintenanceConfig {
                raw_cache_cleanup_on_startup: true,
                raw_cache_cleanup_interval_ms: 0,
                policies: vec![MetadataMaintenancePolicyConfig {
                    id: "movies-nightly".to_owned(),
                    enabled: true,
                    library_id: Some(library_id),
                    item_ids: Vec::new(),
                    providers: Some(vec![ExternalProvider::Tmdb]),
                    item_kinds: vec![MediaKind::Movie],
                    profile: None,
                    language: Some("en-US".to_owned()),
                    refresh_mode: Some(MetadataRefreshMode::MissingOnly),
                    force: false,
                    interval_ms: 86_400_000,
                    initial_delay_ms: 86_400_000,
                }],
            },
            ..MetadataConfig::default()
        },
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = MediaItem {
        id: item_id,
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Lifecycle Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_provider_raw_response(&ProviderRawResponse {
            item_id,
            provider: ExternalProvider::Tmdb,
            provider_key: "1".to_owned(),
            fetched_at: "2020-01-01T00:00:00.000Z".to_owned(),
            body_json: "{}".to_owned(),
        })
        .await
        .unwrap();

    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let raw = store
        .list_provider_raw_responses(
            item_id,
            taru_core::ProviderRawResponseFilter::default(),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    let request = app
        .metadata()
        .metadata_maintenance_request_from_policy(&app.config().metadata.maintenance.policies[0]);
    let runtime = app.runtime_diagnostics();

    assert!(raw.is_empty());
    assert_eq!(runtime.active_tasks, 1);
    assert_eq!(runtime.tasks[0].name, "metadata_maintenance_policy");
    assert_eq!(
        runtime.tasks[0].resource_class,
        "metadata.maintenance.schedule"
    );
    assert_eq!(request.library_id, Some(library_id));
    assert_eq!(request.providers, Some(vec![ExternalProvider::Tmdb]));
    assert_eq!(request.item_kinds, vec![MediaKind::Movie]);
    assert_eq!(request.refresh_mode, Some(MetadataRefreshMode::MissingOnly));

    app.shutdown_runtime();
    assert!(app.runtime_diagnostics().shutdown_requested);
    assert_eq!(app.runtime_diagnostics().active_tasks, 0);
}

#[tokio::test]
async fn metadata_raw_cache_cleanup_worker_is_supervised_and_stops_on_shutdown() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        metadata: MetadataConfig {
            maintenance: MetadataMaintenanceConfig {
                raw_cache_cleanup_on_startup: false,
                raw_cache_cleanup_interval_ms: 60_000,
                policies: Vec::new(),
            },
            ..MetadataConfig::default()
        },
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store).await.unwrap();
    let runtime = app.runtime_diagnostics();

    assert_eq!(runtime.active_tasks, 1);
    assert_eq!(runtime.tasks[0].name, "metadata_raw_cache_cleanup");
    assert_eq!(
        runtime.tasks[0].resource_class,
        "metadata.raw_cache.cleanup"
    );

    app.shutdown_runtime();
    assert!(app.runtime_diagnostics().shutdown_requested);
    assert_eq!(app.runtime_diagnostics().active_tasks, 0);
}

#[tokio::test]
async fn metadata_refresh_event_payload_uses_ids_not_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    upsert_item_with_source(&store, library_id, &item).await;
    let job_id = JobId::new();
    let refresh = MetadataRefreshSummary {
        job_id,
        item_id: item.id,
        provider: ExternalProvider::Tmdb,
        selected_provider: ExternalProvider::Tmdb,
        provider_key: "603".to_owned(),
        matched_by: MetadataMatchKind::ExternalId,
        refresh_mode: MetadataRefreshMode::MissingOnly,
        updated: true,
        attempted_providers: vec![taru_metadata::MetadataProviderAttempt {
            provider: ExternalProvider::Tmdb,
            status: MetadataProviderAttemptStatus::Succeeded,
            message: None,
            provider_key: Some("603".to_owned()),
            matched_by: Some(MetadataMatchKind::ExternalId),
            error_class: None,
        }],
    };

    app.metadata()
        .record_metadata_refreshed_event(job_id, item.id, &refresh)
        .await;
    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, DomainEventKind::ItemMetadataRefreshed);
    assert_eq!(events[0].subject, DomainEventSubject::Item(item.id));
    assert_eq!(events[0].library_id, Some(library_id));
    assert!(!events[0].payload_json.contains("TMDB_READ_ACCESS_TOKEN"));
    assert!(
        !events[0]
            .payload_json
            .contains(&temp.path().display().to_string())
    );
}

#[tokio::test]
async fn metadata_refresh_requires_persisted_media_source_for_library_resolution() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
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
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "No Source".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();

    let err = app
        .metadata()
        .create_metadata_refresh_job(item.id)
        .await
        .unwrap_err();

    let TaruError::InvalidInput { message } = err else {
        panic!("expected missing media source validation error");
    };
    assert!(message.contains("has no persisted media source"));
    assert!(message.contains(&item.id.to_string()));
}
