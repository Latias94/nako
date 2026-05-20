use super::*;

#[tokio::test]
async fn metadata_refresh_route_queues_background_job() {
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
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: item.id,
            locator: "local:///The Matrix.mkv".to_owned(),
            file_name: "The Matrix.mkv".to_owned(),
            size_bytes: Some(1024),
            fingerprint: None,
        })
        .await
        .unwrap();
    let router = build_router(app);
    let path = format!("/items/{}/metadata/refresh", item.id);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let job = body_json::<JobResponse>(response).await;
    assert_eq!(job.kind, JobKind::MetadataRefresh);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert!(job.has_input);
    assert!(!job.has_summary);
    assert!(!job.has_error);
}

#[tokio::test]
async fn metadata_diagnostics_routes_expose_attempts_raw_and_provider_status_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.runtime = MetadataProviderRuntimeConfig {
        timeout_ms: 4_000,
        max_attempts: 3,
        min_interval_ms: 125,
        concurrency: 2,
        user_agent: "taru-test/metadata-diagnostics".to_owned(),
        proxy: Some("http://user:proxy-secret@127.0.0.1:10809".into()),
        circuit_breaker_failures: 4,
        circuit_breaker_backoff_ms: 12_345,
    };
    metadata.providers = vec![MetadataProviderConfig {
        provider: ExternalProvider::Douban,
        enabled: true,
        token_env: None,
        api_key_env: None,
        api_base_url: Some("https://api.douban.example.test".to_owned()),
        image_base_url: None,
        language: None,
        include_adult: false,
        headers: vec![MetadataProviderHeaderConfig {
            name: "X-Douban-Secret".to_owned(),
            value: Some("diagnostics-header-secret".into()),
            value_env: None,
        }],
        runtime: None,
    }];
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
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Diagnostics Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.douban".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store
        .insert_metadata_provider_attempt(NewMetadataProviderAttempt {
            id: MetadataProviderAttemptId::new(),
            job_id: job.id,
            item_id: item.id,
            provider: ExternalProvider::Douban,
            status: MetadataProviderAttemptStatus::Failed,
            provider_key: Some("douban-42".to_owned()),
            matched_by: Some(MetadataMatchKind::Search),
            started_at: "2026-05-16T00:00:00Z".to_owned(),
            finished_at: "2026-05-16T00:00:01Z".to_owned(),
            error_class: Some(MetadataProviderErrorClass::HttpStatus),
            message: Some("HTTP 503".to_owned()),
        })
        .await
        .unwrap();
    store
        .upsert_provider_raw_response(&ProviderRawResponse {
            item_id: item.id,
            provider: ExternalProvider::Douban,
            provider_key: "douban-42".to_owned(),
            fetched_at: "2026-05-16T00:00:02Z".to_owned(),
            body_json: r#"{"id":"douban-42","title":"Diagnostics Demo"}"#.to_owned(),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let attempts_path = format!(
        "/items/{}/metadata/attempts?provider=douban&status=failed",
        item.id
    );
    let raw_path = format!("/items/{}/metadata/raw?provider=douban", item.id);

    let attempts =
        request_json::<MetadataProviderAttemptsResponse>(&router, Method::GET, &attempts_path)
            .await;
    let raw = request_json::<MetadataRawResponsesResponse>(&router, Method::GET, &raw_path).await;

    assert_eq!(attempts.item_id, item.id);
    assert_eq!(attempts.page.returned, 1);
    assert_eq!(
        attempts.attempts[0].attempt.provider,
        ExternalProvider::Douban
    );
    assert_eq!(
        attempts.attempts[0].attempt.status,
        MetadataProviderAttemptStatus::Failed
    );
    assert!(attempts.attempts[0].retryable);
    assert_eq!(raw.item_id, item.id);
    assert_eq!(raw.page.returned, 1);
    assert_eq!(raw.responses[0].provider_key, "douban-42");

    let cleanup = request_json::<MetadataRawCleanupResponse>(
        &router,
        Method::POST,
        "/metadata/raw/cleanup?provider=douban&fetched_before=2026-05-17T00:00:00.000Z",
    )
    .await;
    assert_eq!(cleanup.cleanup.deleted, 1);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/metadata/providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(!body.contains("diagnostics-header-secret"));
    assert!(!body.contains("proxy-secret"));
    let providers: MetadataProviderDiagnosticsResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(providers.providers.len(), 1);
    assert_eq!(providers.providers[0].provider, ExternalProvider::Douban);
    assert_eq!(
        providers.providers[0].status,
        MetadataProviderDiagnosticStatus::Available
    );
    assert!(providers.providers[0].runtime.proxy_configured);
    assert_eq!(providers.providers[0].runtime.timeout_ms, 4_000);
    assert_eq!(providers.providers[0].runtime.max_attempts, 3);
    assert_eq!(
        providers.providers[0].runtime.circuit_breaker_backoff_ms,
        12_345
    );
    assert!(!providers.providers[0].runtime.circuit_open);
    assert_eq!(providers.providers[0].runtime.consecutive_failures, 0);
    assert_eq!(
        providers.providers[0].runtime.state_scope,
        taru_api::metadata_diagnostics::MetadataProviderRuntimeStateScope::ProcessLocal
    );
}

#[tokio::test]
async fn metadata_maintenance_route_enqueues_batch_job() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(
        TaruServerConfig {
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
        },
        store.clone(),
    )
    .await
    .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Route Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: item.id,
            locator: "local:///Route Demo.mkv".to_owned(),
            file_name: "Route Demo.mkv".to_owned(),
            size_bytes: Some(1024),
            fingerprint: None,
        })
        .await
        .unwrap();
    let router = build_router(app);
    let request = EnqueueMetadataMaintenanceRequest {
        library_id: Some(library_id),
        item_ids: Vec::new(),
        providers: Some(vec![ExternalProvider::Tmdb]),
        item_kinds: vec![MediaKind::Movie],
        profile: None,
        language: None,
        refresh_mode: None,
        force: false,
    };
    let plan = request_body_json::<MetadataMaintenancePlanResponse, _>(
        &router,
        Method::POST,
        "/metadata/maintenance/plan",
        &request,
    )
    .await;

    assert_eq!(plan.planned_items, 1);
    assert_eq!(plan.skipped_items, 0);
    assert_eq!(plan.items[0].item_id, item.id);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/metadata/maintenance/jobs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let response_body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&response_body);
    let job: JobResponse = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(job.kind, JobKind::MetadataMaintenance);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert!(job.has_input);
    assert!(!job.has_summary);
    assert!(!job.has_error);
    assert!(!body.contains("\"input\":"));
    assert!(!body.contains("\"summary\":"));
    assert!(!body.contains("\"error\":"));
    assert!(!body.contains("input_json"));
    assert!(!body.contains("summary_json"));
}
