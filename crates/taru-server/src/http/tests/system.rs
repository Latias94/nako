use super::*;

#[tokio::test]
async fn health_and_libraries_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let health_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health_response.status(), StatusCode::OK);
    assert_eq!(
        health_response.headers()[taru_api::API_VERSION_HEADER],
        taru_api::API_VERSION
    );
    let health = body_json::<HealthResponse>(health_response).await;
    let libraries = request_json::<LibraryListResponse>(&router, Method::GET, "/libraries").await;
    let library =
        request_json::<LibraryResponse>(&router, Method::GET, &format!("/libraries/{library_id}"))
            .await;

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, taru_api::API_VERSION);
    assert_eq!(libraries.libraries.len(), 1);
    assert_eq!(libraries.libraries[0].id, library_id.to_string());
    assert_eq!(library.library.id, library_id.to_string());
    assert_eq!(libraries.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(libraries.page.offset, 0);
    assert_eq!(libraries.page.returned, 1);
}

#[tokio::test]
async fn storage_backend_diagnostics_route_exposes_registry_state_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let diagnostics = request_json::<StorageBackendDiagnosticsResponse>(
        &router,
        Method::GET,
        "/storage/backends",
    )
    .await;

    assert_eq!(diagnostics.backends.len(), 1);
    let backend = &diagnostics.backends[0];
    assert_eq!(backend.library_id, library_id);
    assert_eq!(backend.library_name, "Movies");
    assert_eq!(backend.root_uri, "local:///");
    assert_eq!(backend.backend_kind, StorageBackendKind::Local);
    assert_eq!(backend.scheme, "local");
    assert_eq!(backend.status, StorageBackendStatus::Ready);
    assert_eq!(backend.reason, None);
    assert!(backend.registry.cached);
    assert_eq!(backend.registry.stream_permits_max, 8);
    assert_eq!(backend.registry.stream_permits_available, 8);
    assert_eq!(backend.registry.stage_permits_max, 2);
    assert_eq!(backend.registry.stage_permits_available, 2);
    assert_eq!(
        backend.registry.state_scope,
        StorageBackendRuntimeStateScope::ProcessLocal
    );
    assert_eq!(backend.health.consecutive_errors, 0);
    assert_eq!(backend.health.last_success_at_ms, None);
    assert_eq!(backend.health.last_error_at_ms, None);

    let body = serde_json::to_string(&diagnostics).unwrap();
    assert!(!body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_overview_composes_safe_read_only_diagnostics() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[taru_api::API_VERSION_HEADER],
        taru_api::API_VERSION
    );

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    let overview: AdminOverviewResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(overview.admin_api_version, taru_api::ADMIN_API_VERSION);
    assert_eq!(overview.public_api_version, taru_api::API_VERSION);
    assert_eq!(overview.status, AdminOverviewStatus::Healthy);
    assert_eq!(overview.storage.total_backends, 1);
    assert_eq!(overview.storage.ready_backends, 1);
    assert_eq!(overview.storage.backends.len(), 1);
    assert_eq!(overview.storage.backends[0].library_id, library_id);
    assert_eq!(overview.storage.backends[0].library_name, "Movies");
    assert_eq!(
        overview.storage.backends[0].backend_kind,
        StorageBackendKind::Local
    );
    assert_eq!(
        overview.storage.backends[0].status,
        StorageBackendStatus::Ready
    );
    assert_eq!(overview.metadata.total_providers, 0);
    assert_eq!(overview.runtime.failed_tasks, 0);
    assert_eq!(overview.runtime.failed_jobs, 0);
    assert!(!overview.runtime.shutdown_requested);
    assert_eq!(overview.startup.configured_libraries, 1);
    assert_eq!(overview.startup.recovered_jobs, 0);
    assert_eq!(overview.startup.recovered_transcode_sessions, 0);

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("secret"));
    assert!(!body.contains("token"));
    assert!(!body.contains("root_uri"));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("ProviderRawResponse"));

    let health = request_json::<HealthResponse>(&router, Method::GET, "/health").await;
    let libraries = request_json::<LibraryListResponse>(&router, Method::GET, "/libraries").await;
    let storage = request_json::<StorageBackendDiagnosticsResponse>(
        &router,
        Method::GET,
        "/storage/backends",
    )
    .await;

    assert_eq!(health.status, "ok");
    assert_eq!(libraries.libraries[0].id, library_id.to_string());
    assert_eq!(storage.backends[0].library_id, library_id);
}

#[tokio::test]
async fn admin_v1_jobs_lists_filters_and_redacts_raw_payloads() {
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

    let scan = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
        })
        .await
        .unwrap();
    store.start_job(scan.id).await.unwrap();
    store
        .succeed_job(
            scan.id,
            Some(format!(
                r#"{{"output_path":"{}","discovered_files":1}}"#,
                temp.path().join("private.nfo").display()
            )),
        )
        .await
        .unwrap();
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.tmdb".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/jobs?status=succeeded&kind=library_scan&resource_class=disk.scan&library_id={library_id}&limit=5"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let jobs: AdminJobListResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(jobs.jobs.len(), 1);
    assert_eq!(jobs.jobs[0].id, scan.id);
    assert_eq!(jobs.jobs[0].kind, JobKind::LibraryScan);
    assert_eq!(jobs.jobs[0].status, JobStatus::Succeeded);
    assert!(jobs.jobs[0].has_input);
    assert!(jobs.jobs[0].has_summary);
    assert!(!jobs.jobs[0].has_error);
    assert_eq!(jobs.page.limit, 5);
    assert_eq!(jobs.page.returned, 1);
    assert!(!body.contains("admin-token"));
    assert!(!body.contains("private.nfo"));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("secret"));
}

#[tokio::test]
async fn admin_v1_events_lists_filters_and_redacts_payloads() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Event Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: source_id,
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Event Demo.mkv".to_owned(),
        file_name: "Event Demo.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let selected = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::ItemMetadataRefreshed,
            subject: DomainEventSubject::Source(source_id),
            library_id: Some(library_id),
            source_id: Some(source_id),
            idempotency_key: format!("metadata:{source_id}:secret-key"),
            payload_json: format!(
                r#"{{"token":"admin-token","local_path":"{}"}}"#,
                temp.path().join("private.nfo").display()
            ),
        })
        .await
        .unwrap();
    store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("library_scan:{library_id}"),
            payload_json: "{}".to_owned(),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/events?kind=item.metadata_refreshed&status=pending&library_id={library_id}&source_id={source_id}&limit=5"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let events: AdminOutboxEventListResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(events.events.len(), 1);
    assert_eq!(events.events[0].id, selected.id);
    assert_eq!(
        events.events[0].kind,
        DomainEventKind::ItemMetadataRefreshed
    );
    assert_eq!(events.events[0].status, OutboxEventStatus::Pending);
    assert_eq!(events.events[0].library_id, Some(library_id));
    assert_eq!(events.events[0].source_id, Some(source_id));
    assert_eq!(events.events[0].attempts, 0);
    assert!(events.events[0].has_payload);
    assert!(!events.events[0].has_error);
    assert_eq!(events.page.limit, 5);
    assert_eq!(events.page.returned, 1);
    assert!(!body.contains("payload_json"));
    assert!(!body.contains("idempotency_key"));
    assert!(!body.contains("secret-key"));
    assert!(!body.contains("admin-token"));
    assert!(!body.contains("private.nfo"));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("local_path"));
}

#[tokio::test]
async fn admin_v1_storage_staging_lists_filters_and_redacts_paths() {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig {
            max_bytes: 9_999,
            retention_ms: 8_888,
            cleanup_on_startup: true,
        },
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
    let staging_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: staging_id,
            source_uri: "webdav:///Movies/Private/Demo.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::FfmpegInput,
            local_path: temp
                .path()
                .join("secret-cache")
                .join("inputs")
                .join("Demo.mkv")
                .display()
                .to_string(),
            size_bytes: Some(42),
            etag: Some("etag-secret".to_owned()),
            fingerprint: Some("fingerprint-secret".to_owned()),
            state: StagingState::Ready,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(1_300),
            active_leases: 0,
            validation_error: Some("failed at local secret path".to_owned()),
        })
        .await
        .unwrap();
    store
        .upsert_vfs_cache_object(&VfsCachedObject {
            uri: "webdav:///Movies/Private/Demo.mkv".to_owned(),
            scheme: "webdav".to_owned(),
            kind: VfsCachedObjectKind::File,
            len: Some(42),
            modified_at: None,
            etag: Some("cache-etag-secret".to_owned()),
            fingerprint: Some("cache-fingerprint-secret".to_owned()),
            capabilities_bits: 0,
            fetched_at_ms: 1_000,
            fresh_until_ms: 1_000,
        })
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Movies/Private/Demo.mkv".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 2_000,
            error: "cache failed at secret path".to_owned(),
        })
        .await
        .unwrap();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: StagingManifestId::new(),
            source_uri: "webdav:///Movies/Other.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: temp
                .path()
                .join("secret-cache")
                .join("probe")
                .join("Other.mkv")
                .display()
                .to_string(),
            size_bytes: Some(10),
            etag: None,
            fingerprint: None,
            state: StagingState::Reserved,
            created_at_ms: 2_000,
            updated_at_ms: 2_100,
            last_accessed_at_ms: 2_200,
            expires_at_ms: Some(2_300),
            active_leases: 0,
            validation_error: None,
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/staging?purpose=ffmpeg_input&state=ready&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let diagnostics: AdminStorageStagingDiagnosticsResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(diagnostics.admin_api_version, taru_api::ADMIN_API_VERSION);
    assert_eq!(diagnostics.summary.configured_max_bytes, 9_999);
    assert_eq!(diagnostics.summary.used_manifest_bytes, 52);
    assert!(diagnostics.summary.cleanup_on_startup);
    assert_eq!(diagnostics.summary.retention_ms, 8_888);
    assert_eq!(diagnostics.summary.vfs_cache.object_count, 1);
    assert_eq!(diagnostics.summary.vfs_cache.failure_count, 1);
    assert_eq!(
        diagnostics.summary.vfs_cache.last_failure_at_ms,
        Some(2_000)
    );
    assert_eq!(diagnostics.records.len(), 1);
    assert_eq!(diagnostics.records[0].id, staging_id);
    assert_eq!(diagnostics.records[0].source_scheme, "webdav");
    assert_eq!(diagnostics.records[0].purpose, StagingPurpose::FfmpegInput);
    assert_eq!(diagnostics.records[0].state, StagingState::Ready);
    assert_eq!(diagnostics.records[0].size_bytes, Some(42));
    assert!(diagnostics.records[0].has_etag);
    assert!(diagnostics.records[0].has_fingerprint);
    assert!(diagnostics.records[0].has_validation_error);
    assert_eq!(diagnostics.page.limit, 5);
    assert_eq!(diagnostics.page.returned, 1);
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("Private"));
    assert!(!body.contains("Demo.mkv"));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("etag-secret"));
    assert!(!body.contains("fingerprint-secret"));
    assert!(!body.contains("cache-etag-secret"));
    assert!(!body.contains("cache-fingerprint-secret"));
    assert!(!body.contains("cache failed at secret path"));
    assert!(!body.contains("failed at local secret path"));
}

#[tokio::test]
async fn admin_v1_system_config_reports_sanitized_configuration() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.runtime = MetadataProviderRuntimeConfig {
        timeout_ms: 7_000,
        max_attempts: 3,
        min_interval_ms: 500,
        concurrency: 2,
        user_agent: "taru-test/1".to_owned(),
        proxy: Some("http://user:proxy-secret@127.0.0.1:10809".into()),
        circuit_breaker_failures: 4,
        circuit_breaker_backoff_ms: 12_345,
    };
    metadata.providers = vec![MetadataProviderConfig {
        provider: ExternalProvider::Bangumi,
        enabled: true,
        token_env: Some("BANGUMI_TOKEN".to_owned()),
        api_key_env: None,
        api_base_url: Some("https://api.bgm.tv/private".to_owned()),
        image_base_url: Some("https://lain.bgm.tv/private".to_owned()),
        language: Some("zh-CN".to_owned()),
        include_adult: true,
        headers: vec![MetadataProviderHeaderConfig {
            name: "X-Secret".to_owned(),
            value: Some("literal-header-secret".into()),
            value_env: Some("BANGUMI_HEADER".to_owned()),
        }],
        runtime: Some(MetadataProviderRuntimeConfig::default()),
    }];
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite://F:/secret/taru.db".to_owned(),
        auth: crate::config::AuthConfig {
            enabled: false,
            token_env: Some("TARU_ADMIN_TOKEN".to_owned()),
        },
        ffprobe_path: temp.path().join("private").join("ffprobe"),
        ffmpeg_path: temp.path().join("private").join("ffmpeg"),
        scan_concurrency: 2,
        probe_concurrency: 3,
        metadata_concurrency: 4,
        remux_concurrency: 5,
        webhook_concurrency: 6,
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata,
        transcode: TranscodeConfig {
            hardware_acceleration: taru_transcode::HardwareAcceleration::Nvenc,
            hardware_fallback: taru_transcode::HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 7,
            gpu_concurrency: 8,
        },
        staging: StagingConfig {
            max_bytes: 9_999,
            retention_ms: 8_888,
            cleanup_on_startup: false,
        },
        playback: PlaybackConfig {
            remote_stream_concurrency: 9,
            remote_stage_concurrency: 10,
        },
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Remote Anime".to_owned(),
            root: temp.path().join("local-root-secret"),
            preset: taru_core::LibraryPreset::Anime,
            webdav: Some(crate::config::WebDavLibraryConfig {
                root: "webdav:///PrivateAnime".to_owned(),
                base_url: "https://user:webdav-secret@example.test/dav".to_owned(),
                username: Some("webdav-user".to_owned()),
                password_env: Some("TARU_WEBDAV_PASSWORD".to_owned()),
                timeout_ms: 11_000,
                max_attempts: 3,
            }),
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store).await.unwrap();
    let router = build_router(app);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/system/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let diagnostics: AdminServerConfigDiagnosticsResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(diagnostics.admin_api_version, taru_api::ADMIN_API_VERSION);
    assert!(!diagnostics.auth.enabled);
    assert_eq!(
        diagnostics.auth.token_env.as_deref(),
        Some("TARU_ADMIN_TOKEN")
    );
    assert_eq!(diagnostics.runtime.scan_concurrency, 2);
    assert_eq!(diagnostics.runtime.webhook_concurrency, 6);
    assert_eq!(diagnostics.libraries.len(), 1);
    assert_eq!(diagnostics.libraries[0].id, library_id);
    assert_eq!(diagnostics.libraries[0].name, "Remote Anime");
    assert_eq!(
        diagnostics.libraries[0].backend_kind,
        StorageBackendKind::WebDav
    );
    assert_eq!(diagnostics.libraries[0].root_scheme, "webdav");
    assert!(diagnostics.libraries[0].has_webdav_password_env);
    assert_eq!(diagnostics.libraries[0].webdav_timeout_ms, Some(11_000));
    assert_eq!(diagnostics.metadata.runtime.timeout_ms, 7_000);
    assert!(diagnostics.metadata.runtime.has_proxy);
    assert_eq!(diagnostics.metadata.providers.len(), 1);
    assert_eq!(
        diagnostics.metadata.providers[0].provider,
        ExternalProvider::Bangumi
    );
    assert_eq!(
        diagnostics.metadata.providers[0].token_env.as_deref(),
        Some("BANGUMI_TOKEN")
    );
    assert!(diagnostics.metadata.providers[0].has_api_base_url);
    assert!(diagnostics.metadata.providers[0].has_image_base_url);
    assert_eq!(diagnostics.metadata.providers[0].header_count, 1);
    assert_eq!(diagnostics.metadata.providers[0].secret_header_count, 1);
    assert!(diagnostics.metadata.providers[0].has_provider_runtime_override);
    assert_eq!(diagnostics.transcode.cpu_concurrency, 7);
    assert_eq!(diagnostics.transcode.gpu_concurrency, 8);
    assert_eq!(diagnostics.staging.max_bytes, 9_999);
    assert!(!diagnostics.staging.cleanup_on_startup);
    assert_eq!(diagnostics.playback.remote_stream_concurrency, 9);
    assert_eq!(diagnostics.playback.remote_stage_concurrency, 10);

    assert!(!body.contains("database_url"));
    assert!(!body.contains("secret/taru.db"));
    assert!(!body.contains("ffmpeg_path"));
    assert!(!body.contains("ffprobe_path"));
    assert!(!body.contains("private/ffmpeg"));
    assert!(!body.contains("remux_staging_root"));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("local-root-secret"));
    assert!(!body.contains("PrivateAnime"));
    assert!(!body.contains("https://user:webdav-secret@example.test/dav"));
    assert!(!body.contains("webdav-secret"));
    assert!(!body.contains("webdav-user"));
    assert!(!body.contains("TARU_WEBDAV_PASSWORD"));
    assert!(!body.contains("proxy-secret"));
    assert!(!body.contains("api.bgm.tv"));
    assert!(!body.contains("lain.bgm.tv"));
    assert!(!body.contains("literal-header-secret"));
    assert!(!body.contains("BANGUMI_HEADER"));
}

#[tokio::test]
async fn admin_v1_playback_sessions_lists_filters_and_redacts_output_paths() {
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
            title: "Playback Session Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Playback Session Demo.mkv".to_owned(),
        file_name: "Playback Session Demo.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let session = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: "hls:single".to_owned(),
            output_path: temp
                .path()
                .join("taru-cache")
                .join("hls")
                .join("secret")
                .join("playlist.m3u8"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            session.id,
            TranscodeSessionState::Failed,
            Some(taru_core::TranscodeFailureCategory::Runner),
            Some(format!(
                "ffmpeg failed while writing {}",
                temp.path().join("taru-cache").join("hls").display()
            )),
        )
        .await
        .unwrap();
    store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "remux:mp4".to_owned(),
            output_path: temp
                .path()
                .join("taru-cache")
                .join("remux")
                .join("stream.mp4"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/playback/sessions?source_id={}&kind=hls_transcode&state=failed&limit=5",
                    source.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let sessions: AdminPlaybackSessionListResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(sessions.sessions.len(), 1);
    assert_eq!(sessions.sessions[0].id, session.id);
    assert_eq!(sessions.sessions[0].source_id, source.id);
    assert_eq!(
        sessions.sessions[0].kind,
        TranscodeSessionKind::HlsTranscode
    );
    assert_eq!(sessions.sessions[0].state, TranscodeSessionState::Failed);
    assert!(sessions.sessions[0].has_failure_message);
    assert!(!sessions.sessions[0].active);
    assert!(sessions.sessions[0].terminal);
    assert_eq!(sessions.page.limit, 5);
    assert_eq!(sessions.page.returned, 1);
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("playlist.m3u8"));
    assert!(!body.contains("ffmpeg failed while writing"));
}

#[tokio::test]
async fn admin_v1_playback_runtime_reports_safe_diagnostics() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let marker = temp.path().join("unused.marker");
    let ffmpeg_path = fake_ffmpeg_script(temp.path(), "runtime", false, &marker);
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        auth: crate::config::AuthConfig::disabled(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 3,
        webhook_concurrency: 2,
        remux_timeout_ms: 90_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig {
            hardware_acceleration: taru_transcode::HardwareAcceleration::Nvenc,
            hardware_fallback: taru_transcode::HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 2,
            gpu_concurrency: 4,
        },
        staging: StagingConfig {
            max_bytes: 123_456,
            retention_ms: 654_321,
            cleanup_on_startup: true,
        },
        playback: PlaybackConfig {
            remote_stream_concurrency: 7,
            remote_stage_concurrency: 3,
        },
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store).await.unwrap();
    let router = build_router(app);

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/playback/runtime")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let diagnostics: AdminPlaybackRuntimeDiagnosticsResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(diagnostics.admin_api_version, taru_api::ADMIN_API_VERSION);
    assert_eq!(diagnostics.public_api_version, taru_api::API_VERSION);
    assert_eq!(
        diagnostics.ffmpeg.probe_status,
        AdminPlaybackRuntimeStatus::Ready
    );
    assert!(!diagnostics.ffmpeg.has_probe_error);
    assert_eq!(diagnostics.ffmpeg.hardware_capability_count, 4);
    assert_eq!(diagnostics.ffmpeg.available_gpu_capabilities, 3);
    assert_eq!(
        diagnostics.hardware.policy.requested,
        taru_transcode::HardwareAcceleration::Nvenc
    );
    assert_eq!(
        diagnostics.hardware.selection.acceleration,
        taru_transcode::HardwareAcceleration::Nvenc
    );
    assert!(!diagnostics.hardware.selection.fallback_used);
    assert_eq!(diagnostics.transcode.configured_cpu_slots, 2);
    assert_eq!(diagnostics.transcode.configured_gpu_slots, 4);
    assert_eq!(diagnostics.transcode.effective_cpu_slots, 2);
    assert_eq!(diagnostics.transcode.effective_gpu_slots, 4);
    assert_eq!(diagnostics.transcode.selected_hls_slots, 4);
    assert_eq!(diagnostics.remux.max_concurrent_sessions, 3);
    assert_eq!(diagnostics.remux.timeout_ms, 90_000);
    assert_eq!(diagnostics.remote_playback.backend_count, 1);
    assert_eq!(diagnostics.remote_playback.stream_permits_max, 7);
    assert_eq!(diagnostics.remote_playback.stage_permits_max, 3);
    assert_eq!(diagnostics.staging.max_bytes, 123_456);
    assert_eq!(diagnostics.staging.retention_ms, 654_321);
    assert!(diagnostics.staging.cleanup_on_startup);

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains(&ffmpeg_path.display().to_string()));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("ffmpeg_path"));
    assert!(!body.contains("remux_staging_root"));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("token"));
}

#[tokio::test]
async fn bearer_auth_protects_non_health_routes_and_keeps_health_public() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let health_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health_response.status(), StatusCode::OK);

    let missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        missing.headers()[taru_api::API_VERSION_HEADER],
        taru_api::API_VERSION
    );
    assert_eq!(missing.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let missing_error = body_json::<ErrorResponse>(missing).await;
    assert_eq!(
        missing_error.code,
        taru_api::ClientErrorCode::Unauthorized.as_str()
    );
    assert_eq!(missing_error.message, "authentication required");

    let wrong = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(header::AUTHORIZATION, "Bearer wrong-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    let wrong_error = body_json::<ErrorResponse>(wrong).await;
    let wrong_error_json = serde_json::to_string(&wrong_error).unwrap();
    assert_eq!(
        wrong_error.code,
        taru_api::ClientErrorCode::Unauthorized.as_str()
    );
    assert!(!wrong_error_json.contains("wrong-token"));
    assert!(!wrong_error_json.contains(token));

    let ok = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let libraries = body_json::<LibraryListResponse>(ok).await;
    assert_eq!(libraries.libraries[0].id, library_id.to_string());

    let admin_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_missing.status(), StatusCode::UNAUTHORIZED);

    let admin_jobs_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_jobs_missing.status(), StatusCode::UNAUTHORIZED);

    let admin_events_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_events_missing.status(), StatusCode::UNAUTHORIZED);

    let admin_staging_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/staging")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_staging_missing.status(), StatusCode::UNAUTHORIZED);

    let admin_config_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/system/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_config_missing.status(), StatusCode::UNAUTHORIZED);

    let admin_sessions_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/playback/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_sessions_missing.status(), StatusCode::UNAUTHORIZED);

    let admin_playback_runtime_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/playback/runtime")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        admin_playback_runtime_missing.status(),
        StatusCode::UNAUTHORIZED
    );

    let admin_ok = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/overview")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_ok.status(), StatusCode::OK);
    let overview = body_json::<AdminOverviewResponse>(admin_ok).await;
    assert_eq!(overview.admin_api_version, taru_api::ADMIN_API_VERSION);
}

#[tokio::test]
async fn api_errors_map_playback_storage_categories() {
    let cases = [
        (
            TaruError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::StagingBudgetExhausted,
                "used=10, additional=4, max=12",
            ),
            StatusCode::INSUFFICIENT_STORAGE,
            taru_api::ClientErrorCode::StagingBudgetExhausted,
            "staging disk budget exhausted",
        ),
        (
            TaruError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::StagingValidationMismatch,
                "staged WebDAV file did not match expected size",
            ),
            StatusCode::BAD_GATEWAY,
            taru_api::ClientErrorCode::StagingValidationMismatch,
            "staged input validation failed",
        ),
        (
            TaruError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::Timeout,
                "WebDAV request failed: operation timed out",
            ),
            StatusCode::GATEWAY_TIMEOUT,
            taru_api::ClientErrorCode::StorageTimeout,
            "storage backend timed out",
        ),
        (
            TaruError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::Unauthorized,
                "WebDAV GET returned 401 Unauthorized",
            ),
            StatusCode::BAD_GATEWAY,
            taru_api::ClientErrorCode::StorageUnauthorized,
            "storage backend rejected credentials",
        ),
        (
            TaruError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::RateLimited,
                "WebDAV GET returned 429 Too Many Requests",
            ),
            StatusCode::SERVICE_UNAVAILABLE,
            taru_api::ClientErrorCode::StorageRateLimited,
            "storage backend rate limited the request",
        ),
        (
            TaruError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message: "hls runner failed".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            taru_api::ClientErrorCode::FfmpegError,
            "ffmpeg operation failed",
        ),
        (
            TaruError::Database {
                message: "raw sqlite path F:\\secret\\taru.db failed".to_owned(),
            },
            StatusCode::INTERNAL_SERVER_ERROR,
            taru_api::ClientErrorCode::DatabaseError,
            "database operation failed",
        ),
    ];

    for (error, status, code, message) in cases {
        let response = ApiError(error).into_response();

        assert_eq!(response.status(), status);
        let body = body_json::<ErrorResponse>(response).await;
        assert_eq!(body.code, code.as_str());
        assert_eq!(taru_api::ClientErrorCode::from_code(&body.code), Some(code));
        assert_eq!(body.message, message);
    }
}
