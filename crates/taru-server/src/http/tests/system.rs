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

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, taru_api::API_VERSION);
    assert_eq!(libraries.libraries.len(), 1);
    assert_eq!(libraries.libraries[0].id, library_id.to_string());
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
async fn api_errors_map_playback_storage_categories() {
    let cases = [
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "staging disk budget exhausted: used=10, additional=4, max=12".to_owned(),
            },
            StatusCode::INSUFFICIENT_STORAGE,
            taru_api::ClientErrorCode::StagingBudgetExhausted,
            "staging disk budget exhausted",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "staged WebDAV file did not match expected size".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            taru_api::ClientErrorCode::StagingValidationMismatch,
            "staged input validation failed",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "WebDAV request failed: operation timed out".to_owned(),
            },
            StatusCode::GATEWAY_TIMEOUT,
            taru_api::ClientErrorCode::StorageTimeout,
            "storage backend timed out",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "WebDAV GET returned 401 Unauthorized".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            taru_api::ClientErrorCode::StorageUnauthorized,
            "storage backend rejected credentials",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "WebDAV GET returned 429 Too Many Requests".to_owned(),
            },
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
