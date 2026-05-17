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
