use super::*;

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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();

    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();

    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
    let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

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
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
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
    store.upsert_media_item(&item).await.unwrap();
    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
    let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

    assert_eq!(job.resource_class, "metadata.bangumi");
    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("bangumi=not_implemented"));
    assert!(message.contains("tmdb=skipped_unavailable"));
    assert_eq!(app.get_job(job.id).await.unwrap().status, JobStatus::Queued);
}

#[tokio::test]
async fn metadata_refresh_resolves_provider_order_from_library_profile() {
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
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
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
    store.upsert_media_item(&item).await.unwrap();
    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let output = app
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
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::MetadataMaintenanceCompleted
            && event.subject == DomainEventSubject::Job(output.job.id)
    }));
}

#[tokio::test]
async fn metadata_lifecycle_config_maps_policy_and_cleans_raw_cache_on_startup() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let item_id = MediaItemId::new();
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
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
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
        .metadata_maintenance_request_from_policy(&app.config().metadata.maintenance.policies[0]);

    assert!(raw.is_empty());
    assert_eq!(request.library_id, Some(library_id));
    assert_eq!(request.providers, Some(vec![ExternalProvider::Tmdb]));
    assert_eq!(request.item_kinds, vec![MediaKind::Movie]);
    assert_eq!(request.refresh_mode, Some(MetadataRefreshMode::MissingOnly));
}

#[tokio::test]
async fn metadata_refresh_event_payload_uses_ids_not_secrets() {
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
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

    app.record_metadata_refreshed_event(job_id, item.id, &refresh)
        .await;
    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, DomainEventKind::ItemMetadataRefreshed);
    assert_eq!(events[0].subject, DomainEventSubject::Item(item.id));
    assert!(!events[0].payload_json.contains("TMDB_READ_ACCESS_TOKEN"));
    assert!(
        !events[0]
            .payload_json
            .contains(&temp.path().display().to_string())
    );
}
