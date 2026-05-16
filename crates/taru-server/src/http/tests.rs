use std::{
    fs,
    path::{Path as FsPath, PathBuf},
    time::Duration,
};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use taru_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonManifest, AddonResource, AddonResourceDeclaration,
    AddonScope, ReqwestAddonTransport, call_addon_resource,
};
use taru_api::{
    AddonRegistrationResponse, AddonRegistrationsResponse, AutomationArtifactsResponse,
    AutomationProviderResponse, AutomationProvidersResponse, EnqueueAutomationJobRequest,
    EnqueueMetadataMaintenanceRequest, ErrorResponse, HealthResponse, JobResponse,
    LibraryListResponse, MetadataProviderAttemptsResponse, MetadataProviderDiagnosticStatus,
    MetadataProviderDiagnosticsResponse, MetadataRawCleanupResponse, MetadataRawResponsesResponse,
    RegisterAddonRequest, TranscodeSessionResponse, UpsertAutomationProviderRequest,
    UpsertWebhookEndpointRequest, WebhookDeliveryAttemptsResponse, WebhookEndpointResponse,
    WebhookEndpointsResponse,
};
use taru_core::{
    AddonStatus, AutomationCapability, AutomationProviderStatus, CanonicalMetadata,
    CatalogRepository, CreditRole, DomainEventKind, DomainEventSubject, EventId,
    EventOutboxRepository, ExternalProvider, Genre, GenreId, ImageAsset, ImageAssetId, ImageKind,
    ImageOwner, ItemCredit, ItemGenre, ItemTag, JobId, JobKind, JobRepository, JobStatus,
    LibraryId, MediaItem, MediaItemId, MediaKind, MediaProbeRepository, MediaProbeResult,
    MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind,
    MetadataMatchKind, MetadataProviderAttemptId, MetadataProviderAttemptStatus,
    MetadataProviderErrorClass, MetadataRepository, MetadataSource, NewJob,
    NewMetadataProviderAttempt, NewOutboxEvent, NewTranscodeSession, Person, PersonId,
    ProviderRawResponse, Tag, TagId, TaruError, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRepository, TranscodeSessionState, WebhookEndpointStatus,
};
use taru_db::SqliteStore;
use taru_search::{SearchDocument, SearchIndex};
use taru_streaming::{
    DirectPlayRangeRequest, PlaybackMode, RequestedByteRange, plan_direct_play_response,
};
use taru_vfs::{ByteRange, ReadStream, StorageUri};
use tokio::{net::TcpListener, task::yield_now, time::sleep};
use tower::ServiceExt;

use super::error::ApiError;
use super::*;
use crate::config::{
    LocalLibraryConfig, MetadataConfig, MetadataProviderConfig, MetadataProviderHeaderConfig,
    MetadataProviderRuntimeConfig, PlaybackConfig, StagingConfig, TaruServerConfig,
    TranscodeConfig,
};
use crate::http::playback::stream_direct_play_response;

#[tokio::test]
async fn health_and_libraries_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let health = request_json::<HealthResponse>(&router, Method::GET, "/health").await;
    let libraries = request_json::<LibraryListResponse>(&router, Method::GET, "/libraries").await;

    assert_eq!(health.status, "ok");
    assert_eq!(libraries.libraries.len(), 1);
    assert_eq!(libraries.libraries[0].id, library_id);
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
            "staging_budget_exhausted",
            "staging disk budget exhausted",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "staged WebDAV file did not match expected size".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            "staging_validation_mismatch",
            "staged input validation failed",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "WebDAV request failed: operation timed out".to_owned(),
            },
            StatusCode::GATEWAY_TIMEOUT,
            "storage_timeout",
            "storage backend timed out",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "WebDAV GET returned 401 Unauthorized".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            "storage_unauthorized",
            "storage backend rejected credentials",
        ),
        (
            TaruError::Storage {
                uri: "webdav:///Movies/Demo.mkv".to_owned(),
                message: "WebDAV GET returned 429 Too Many Requests".to_owned(),
            },
            StatusCode::SERVICE_UNAVAILABLE,
            "storage_rate_limited",
            "storage backend rate limited the request",
        ),
        (
            TaruError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message: "hls runner failed".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            "ffmpeg_error",
            "ffmpeg operation failed",
        ),
    ];

    for (error, status, code, message) in cases {
        let response = ApiError(error).into_response();

        assert_eq!(response.status(), status);
        let body = body_json::<ErrorResponse>(response).await;
        assert_eq!(body.code, code);
        assert_eq!(body.message, message);
    }
}

#[tokio::test]
async fn webhook_endpoint_routes_validate_and_list_enabled_endpoints() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let response = request_body_json::<WebhookEndpointResponse, _>(
        &router,
        Method::POST,
        "/webhooks/endpoints",
        &UpsertWebhookEndpointRequest {
            id: None,
            name: "receiver".to_owned(),
            url: "https://example.test/taru-webhook".to_owned(),
            secret_env: Some("TARU_WEBHOOK_SECRET".to_owned()),
            subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
            timeout_ms: Some(5_000),
            max_attempts: Some(3),
            status: WebhookEndpointStatus::Enabled,
        },
    )
    .await;

    assert_eq!(response.endpoint.name, "receiver");
    assert_eq!(
        response.endpoint.secret_env,
        Some("TARU_WEBHOOK_SECRET".to_owned())
    );

    let list =
        request_json::<WebhookEndpointsResponse>(&router, Method::GET, "/webhooks/endpoints").await;
    assert_eq!(list.endpoints, vec![response.endpoint.clone()]);

    let detail_path = format!("/webhooks/endpoints/{}", response.endpoint.id);
    let detail = request_json::<WebhookEndpointResponse>(&router, Method::GET, &detail_path).await;
    assert_eq!(detail, response);

    let invalid = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/webhooks/endpoints")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&UpsertWebhookEndpointRequest {
                        id: None,
                        name: "bad".to_owned(),
                        url: "file:///tmp/webhook".to_owned(),
                        secret_env: None,
                        subscribed_event_kinds: vec![
                            DomainEventKind::LibraryScanned.as_str().to_owned(),
                        ],
                        timeout_ms: Some(5_000),
                        max_attempts: Some(3),
                        status: WebhookEndpointStatus::Enabled,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn webhook_attempt_route_lists_attempts_for_existing_event() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(
        TaruServerConfig {
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
        },
        store.clone(),
    )
    .await
    .unwrap();
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("library.scanned:{library_id}"),
            payload_json: format!(r#"{{"library_id":"{library_id}"}}"#),
        })
        .await
        .unwrap();
    let router = build_router(app);
    let path = format!("/events/{}/webhook-attempts", event.id);

    let attempts =
        request_json::<WebhookDeliveryAttemptsResponse>(&router, Method::GET, &path).await;

    assert_eq!(attempts.event_id, event.id);
    assert!(attempts.attempts.is_empty());
}

#[tokio::test]
async fn automation_routes_configure_provider_and_enqueue_jobs_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let provider = request_body_json::<AutomationProviderResponse, _>(
        &router,
        Method::POST,
        "/automation/providers",
        &UpsertAutomationProviderRequest {
            id: None,
            name: "gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("TARU_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![
                AutomationCapability::Recommendation,
                AutomationCapability::Summary,
            ],
            timeout_ms: Some(10_000),
            max_attempts: Some(2),
            status: AutomationProviderStatus::Enabled,
        },
    )
    .await;

    assert_eq!(provider.provider.name, "gateway");
    assert_eq!(
        provider.provider.secret_env,
        Some("TARU_AUTOMATION_SECRET".to_owned())
    );

    let providers =
        request_json::<AutomationProvidersResponse>(&router, Method::GET, "/automation/providers")
            .await;
    assert_eq!(providers.providers, vec![provider.provider.clone()]);

    let job_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/automation/jobs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&EnqueueAutomationJobRequest {
                        provider_id: provider.provider.id,
                        capability: AutomationCapability::Summary,
                        library_id: None,
                        item_id: None,
                        source_id: None,
                        prompt: serde_json::json!({"title":"The Matrix"}),
                        idempotency_key: "summary:matrix".to_owned(),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(job_response.status(), StatusCode::ACCEPTED);
    let job = body_json::<JobResponse>(job_response).await;
    assert_eq!(job.kind, JobKind::Automation);
    assert_eq!(job.resource_class, "automation.external_api");
    let input = job.input.unwrap();
    assert_eq!(input["capability"], "summary");
    assert!(!input.to_string().contains("TARU_AUTOMATION_SECRET"));

    let artifacts_path = format!("/automation/jobs/{}/artifacts", job.id);
    let artifacts =
        request_json::<AutomationArtifactsResponse>(&router, Method::GET, &artifacts_path).await;
    assert!(artifacts.artifacts.is_empty());
}

#[tokio::test]
async fn addon_routes_register_disabled_by_default_and_validate_contract() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = addon_manifest();

    let response = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
        &RegisterAddonRequest {
            id: None,
            manifest: manifest.clone(),
            granted_scopes: vec![
                AddonScope::ItemMetadataSuggest,
                AddonScope::ItemMetadataRead,
            ],
            status: None,
        },
    )
    .await;

    assert_eq!(response.addon.manifest_id, manifest.id);
    assert_eq!(response.addon.status, AddonStatus::Disabled);
    assert_eq!(
        response.addon.granted_scopes,
        vec!["item_metadata_suggest", "item_metadata_read"]
    );
    assert!(!response.addon.manifest_json.contains("token"));

    let disabled =
        request_json::<AddonRegistrationsResponse>(&router, Method::GET, "/addons?status=disabled")
            .await;
    assert_eq!(disabled.addons, vec![response.addon.clone()]);

    let enabled =
        request_json::<AddonRegistrationsResponse>(&router, Method::GET, "/addons?status=enabled")
            .await;
    assert!(enabled.addons.is_empty());

    let detail_path = format!("/addons/{}", response.addon.id);
    let detail =
        request_json::<AddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    assert_eq!(detail, response);

    let mut invalid_manifest = addon_manifest();
    invalid_manifest.resources[0].path = "metadata".to_owned();
    let invalid = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: invalid_manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let invalid_error = body_json::<ErrorResponse>(invalid).await;
    assert_eq!(invalid_error.code, "invalid_input");

    let missing_scope = post_addon_registration(
        &router,
        RegisterAddonRequest {
            id: None,
            manifest: addon_manifest(),
            granted_scopes: vec![AddonScope::ItemMetadataRead],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(missing_scope.status(), StatusCode::BAD_REQUEST);
    let missing_scope_error = body_json::<ErrorResponse>(missing_scope).await;
    assert_eq!(missing_scope_error.code, "invalid_input");
}

#[tokio::test]
async fn reference_addon_registers_queries_and_handles_resource_call() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addon_base_url = format!("http://{}", listener.local_addr().unwrap());
    let addon_server = tokio::spawn(async move {
        axum::serve(listener, taru_reference_addon::build_router())
            .await
            .unwrap();
    });
    yield_now().await;

    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let manifest = taru_reference_addon::reference_manifest(addon_base_url);

    let registered = request_body_json::<AddonRegistrationResponse, _>(
        &router,
        Method::POST,
        "/addons",
        &RegisterAddonRequest {
            id: None,
            manifest,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            status: Some(AddonStatus::Enabled),
        },
    )
    .await;
    assert_eq!(registered.addon.status, AddonStatus::Enabled);
    assert_eq!(
        registered.addon.manifest_id,
        taru_reference_addon::REFERENCE_ADDON_ID
    );

    let detail_path = format!("/addons/{}", registered.addon.id);
    let detail =
        request_json::<AddonRegistrationResponse>(&router, Method::GET, &detail_path).await;
    let stored_manifest =
        serde_json::from_str::<AddonManifest>(&detail.addon.manifest_json).unwrap();
    let granted_scopes = [
        AddonScope::ItemMetadataRead,
        AddonScope::ItemMetadataSuggest,
    ];

    let response = call_addon_resource(
        &ReqwestAddonTransport::default(),
        &stored_manifest,
        AddonResource::Metadata,
        &granted_scopes,
        "reference-addon-e2e-1",
        serde_json::json!({"title":"The Matrix"}),
        None,
    )
    .await
    .unwrap();

    assert_eq!(response.payload["title"], "The Matrix");
    assert_eq!(
        response.payload["source"],
        taru_reference_addon::REFERENCE_ADDON_ID
    );
    assert_eq!(response.artifacts[0].kind, "metadata_suggestion");

    addon_server.abort();
}

#[tokio::test]
async fn scan_route_queues_background_job() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let path = format!("/libraries/{library_id}/scan");

    let response = router
        .clone()
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
    assert_eq!(job.kind, taru_core::JobKind::LibraryScan);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert_eq!(
        job.input
            .as_ref()
            .and_then(|input| input.get("library_id"))
            .and_then(serde_json::Value::as_str),
        Some(library_id.to_string().as_str())
    );

    let loaded_path = format!("/jobs/{}", job.id);
    let loaded_job = request_json::<JobResponse>(&router, Method::GET, &loaded_path).await;
    assert_eq!(loaded_job.id, job.id);
}

#[tokio::test]
async fn nfo_routes_queue_background_jobs() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let import_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/libraries/{library_id}/nfo/import"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let export_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/libraries/{library_id}/nfo/export"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(import_response.status(), StatusCode::ACCEPTED);
    assert_eq!(export_response.status(), StatusCode::ACCEPTED);
    let import_job = body_json::<JobResponse>(import_response).await;
    let export_job = body_json::<JobResponse>(export_response).await;

    assert_eq!(import_job.kind, JobKind::NfoImport);
    assert_eq!(import_job.resource_class, "metadata.nfo.import");
    assert_eq!(import_job.library_id, Some(library_id));
    assert_eq!(
        import_job
            .input
            .as_ref()
            .and_then(|input| input.get("policy"))
            .and_then(serde_json::Value::as_str),
        Some("local_first")
    );
    assert_eq!(export_job.kind, JobKind::NfoExport);
    assert_eq!(export_job.resource_class, "metadata.nfo.export");
    assert_eq!(export_job.library_id, Some(library_id));
}

#[tokio::test]
async fn missing_job_returns_404() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let missing = JobId::new();
    let path = format!("/jobs/{missing}");

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
}

#[tokio::test]
async fn metadata_refresh_route_queues_background_job() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.tmdb.enabled = true;
    metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
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
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
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
    assert_eq!(
        job.input
            .as_ref()
            .and_then(|input| input.get("item_id"))
            .and_then(serde_json::Value::as_str),
        Some(item.id.to_string().as_str())
    );
    assert_eq!(
        job.input
            .as_ref()
            .and_then(|input| input.get("provider"))
            .and_then(serde_json::Value::as_str),
        Some("tmdb")
    );
    assert_eq!(
        job.input
            .as_ref()
            .and_then(|input| input.get("refresh_mode"))
            .and_then(serde_json::Value::as_str),
        Some("default")
    );
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
        proxy: Some("http://user:proxy-secret@127.0.0.1:10809".to_owned()),
        circuit_breaker_failures: 4,
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
            value: Some("diagnostics-header-secret".to_owned()),
            value_env: None,
        }],
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
    assert!(!providers.providers[0].runtime.circuit_open);
    assert_eq!(providers.providers[0].runtime.consecutive_failures, 0);
    assert_eq!(
        providers.providers[0].runtime.state_scope,
        taru_api::MetadataProviderRuntimeStateScope::ProcessLocal
    );
}

#[tokio::test]
async fn metadata_maintenance_route_enqueues_batch_job() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(
        TaruServerConfig {
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

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/metadata/maintenance/jobs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&EnqueueMetadataMaintenanceRequest {
                        library_id: Some(library_id),
                        item_ids: Vec::new(),
                        providers: Some(vec![ExternalProvider::Tmdb]),
                        item_kinds: vec![MediaKind::Movie],
                        profile: None,
                        language: None,
                        refresh_mode: None,
                        force: false,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let job = body_json::<JobResponse>(response).await;
    assert_eq!(job.kind, JobKind::MetadataMaintenance);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert!(job.input.unwrap().get("access_token").is_none());
}

#[tokio::test]
async fn empty_sources_and_items_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let sources_path = format!("/libraries/{library_id}/sources");

    let sources =
        request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path).await;
    let items = request_json::<taru_api::ItemsResponse>(&router, Method::GET, "/items").await;

    assert_eq!(sources.library.id, library_id);
    assert_eq!(sources.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(sources.page.offset, 0);
    assert!(sources.sources.is_empty());
    assert_eq!(items.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
    assert!(items.items.is_empty());
}

#[tokio::test]
async fn search_route_returns_indexed_items() {
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
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Search Route Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert(SearchDocument {
            item_id: item.id,
            title: item.metadata.title.clone(),
            body: "A route test fixture".to_owned(),
            facets: vec!["genre:test".to_owned()],
        })
        .await
        .unwrap();
    let router = build_router(app);

    let result = request_json::<taru_api::SearchResponse>(
        &router,
        Method::GET,
        "/search?q=route&facet=genre:test",
    )
    .await;

    assert_eq!(result.page.returned, 1);
    assert_eq!(result.hits[0].item.id, item.id);
}

#[tokio::test]
async fn browse_routes_return_catalog_graph() {
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
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Browse Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Browse Demo.mkv".to_owned(),
        file_name: "Browse Demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    let person = Person {
        id: PersonId::new(),
        name: "Demo Actor".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let genre = Genre {
        id: GenreId::new(),
        name: "Science Fiction".to_owned(),
        source: MetadataSource::Nfo,
    };
    let tag = Tag {
        id: TagId::new(),
        name: "favorite".to_owned(),
        source: MetadataSource::User,
    };
    let image = ImageAsset {
        id: ImageAssetId::new(),
        owner: ImageOwner::Item(item.id),
        kind: ImageKind::Poster,
        source_uri: "local:///poster.jpg".to_owned(),
        provider: taru_core::ExternalProvider::Local,
        cache_uri: None,
        width: None,
        height: None,
        language: None,
        selected: true,
        content_hash: None,
        etag: None,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store.upsert_person(&person).await.unwrap();
    store
        .upsert_item_credit(&ItemCredit {
            item_id: item.id,
            person_id: person.id,
            role: CreditRole::Actor,
            character: Some("Lead".to_owned()),
            sort_order: Some(0),
        })
        .await
        .unwrap();
    store.upsert_genre(&genre).await.unwrap();
    store
        .upsert_item_genre(&ItemGenre {
            item_id: item.id,
            genre_id: genre.id,
        })
        .await
        .unwrap();
    store.upsert_tag(&tag).await.unwrap();
    store
        .upsert_item_tag(&ItemTag {
            item_id: item.id,
            tag_id: tag.id,
        })
        .await
        .unwrap();
    store.upsert_image_asset(&image).await.unwrap();
    let router = build_router(app);

    let detail = request_json::<taru_api::ItemDetailResponse>(
        &router,
        Method::GET,
        &format!("/items/{}", item.id),
    )
    .await;
    let credits = request_json::<taru_api::ItemCreditsResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/credits", item.id),
    )
    .await;
    let images = request_json::<taru_api::ImagesResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/images", item.id),
    )
    .await;
    let people = request_json::<taru_api::PeopleResponse>(&router, Method::GET, "/people").await;
    let person_items = request_json::<taru_api::PersonItemsResponse>(
        &router,
        Method::GET,
        &format!("/people/{}/items", person.id),
    )
    .await;
    let tags = request_json::<taru_api::TagsResponse>(&router, Method::GET, "/tags").await;
    let tag_items = request_json::<taru_api::TagItemsResponse>(
        &router,
        Method::GET,
        &format!("/tags/{}/items", tag.id),
    )
    .await;
    let genres = request_json::<taru_api::GenreListResponse>(&router, Method::GET, "/genres").await;
    let genre_items = request_json::<taru_api::GenreItemsResponse>(
        &router,
        Method::GET,
        &format!("/genres/{}/items", genre.id),
    )
    .await;

    assert_eq!(detail.item.id, item.id);
    assert_eq!(detail.sources[0].id, source.id);
    assert_eq!(detail.credits.len(), 1);
    assert_eq!(credits.people[0].name, "Demo Actor");
    assert_eq!(images.images[0].id, image.id);
    assert_eq!(people.people[0].id, person.id);
    assert_eq!(person_items.items[0].id, item.id);
    assert_eq!(tags.tags[0].name, "favorite");
    assert_eq!(tag_items.items[0].id, item.id);
    assert_eq!(genres.genres[0].name, "Science Fiction");
    assert_eq!(genre_items.items[0].id, item.id);
}

#[tokio::test]
async fn playback_decision_and_direct_stream_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mp4"), b"0123456789").unwrap();
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

    assert_eq!(decision.decision.mode, PlaybackMode::DirectPlay);
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

    assert_eq!(session_response.session.id, session.id);
    assert_eq!(
        session_response.session.state,
        TranscodeSessionState::Finished
    );
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

    for _ in 0..50 {
        if marker.exists() {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    assert!(marker.exists());

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
}

async fn router_with_media_source(
    file_name: &str,
    content: &[u8],
) -> (tempfile::TempDir, Router, MediaSource) {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join(file_name), content).unwrap();
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
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: file_name.to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: format!("local:///{file_name}"),
        file_name: file_name.to_owned(),
        size_bytes: Some(content.len() as u64),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let router = build_router(app);

    (temp, router, source)
}

async fn router_with_remux_source(
    slow: bool,
) -> (
    tempfile::TempDir,
    Router,
    MediaSource,
    PathBuf,
    PathBuf,
    PathBuf,
    SqliteStore,
) {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("remux.started");
    let ffmpeg_path = fake_ffmpeg_script(temp.path(), "remux", slow, &marker);
    let library_root = temp.path().join("library");
    let staging_root = temp.path().join("cache").join("remux");
    fs::create_dir_all(&library_root).unwrap();
    fs::write(library_root.join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
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
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
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
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let router = build_router(app);

    (
        temp,
        router,
        source,
        staging_root,
        ffmpeg_path,
        marker,
        store,
    )
}

async fn router_with_hls_source() -> (tempfile::TempDir, Router, MediaSource, SqliteStore) {
    let temp = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(temp.path(), "hls");
    let library_root = temp.path().join("library");
    let staging_root = temp.path().join("cache").join("remux");
    fs::create_dir_all(&library_root).unwrap();
    fs::write(library_root.join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path,
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: staging_root,
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
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
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let router = build_router(app);

    (temp, router, source, store)
}

fn compatible_probe() -> MediaProbeResult {
    MediaProbeResult {
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
    }
}

fn fake_ffmpeg_script(root: &FsPath, name: &str, slow: bool, marker: &FsPath) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        content.push_str("for arg do out=\"$arg\"; done\n");
        if slow {
            content.push_str(&format!("printf started > \"{}\"\n", marker.display()));
        }
        content.push_str("printf remuxed > \"$out\"\n");
        if slow {
            content.push_str("sleep 1\n");
        }
        content.push_str("exit 0\n");
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        if slow {
            content.push_str(&format!(
                "<nul set /p dummy=started>\"{}\"\r\n",
                marker.display()
            ));
        }
        content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
        if slow {
            content.push_str("ping -n 3 127.0.0.1 > nul\r\n");
        }
        content.push_str("exit /b 0\r\n");
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_hls_ffmpeg_script(root: &FsPath, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        content.push_str("for arg do out=\"$arg\"; done\n");
        content.push_str("dir=$(dirname \"$out\")\n");
        content.push_str("mkdir -p \"$dir\"\n");
        content.push_str(
            "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
        );
        content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
        content.push_str("exit 0\n");
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
        content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
        content.push_str(">\"%out%\" echo #EXTM3U\r\n");
        content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
        content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
        content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
        content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
        content.push_str("exit /b 0\r\n");
        fs::write(&path, content).unwrap();
        path
    }
}

async fn test_router(root: PathBuf, library_id: LibraryId) -> Router {
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
        remux_staging_root: root.join("taru-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root,
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store).await.unwrap();
    build_router(app)
}

fn addon_manifest() -> AddonManifest {
    AddonManifest {
        id: "example.metadata".to_owned(),
        name: "Example Metadata".to_owned(),
        version: "0.1.0".to_owned(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: "https://example.test/addon".to_owned(),
        description: Some("Metadata suggestion addon".to_owned()),
        resources: vec![AddonResourceDeclaration {
            kind: AddonResource::Metadata,
            path: "/metadata".to_owned(),
            input_schema: Some("taru.metadata.request.v1".to_owned()),
            output_schema: Some("taru.metadata.response.v1".to_owned()),
            required_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            timeout_ms: Some(5_000),
            max_attempts: Some(2),
        }],
        auth: AddonAuth::Bearer,
        default_timeout_ms: Some(10_000),
        default_max_attempts: Some(2),
        scopes: vec![
            AddonScope::ItemMetadataRead,
            AddonScope::ItemMetadataSuggest,
        ],
    }
}

async fn post_addon_registration(
    router: &Router,
    request: RegisterAddonRequest,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/addons")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn request_json<T>(router: &Router, method: Method, uri: &str) -> T
where
    T: DeserializeOwned,
{
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn request_body_json<T, B>(router: &Router, method: Method, uri: &str, body: &B) -> T
where
    T: DeserializeOwned,
    B: Serialize,
{
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn body_json<T>(response: axum::response::Response) -> T
where
    T: DeserializeOwned,
{
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

struct MockWebDavServer {
    addr: std::net::SocketAddr,
}

impl MockWebDavServer {
    async fn start() -> Self {
        let router = Router::new().route("/{*path}", axum::routing::any(mock_webdav_handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self { addr }
    }

    fn base_url(&self) -> String {
        format!("http://{}/dav", self.addr)
    }
}

async fn mock_webdav_handler(method: Method, uri: axum::http::Uri) -> Response {
    let path = uri.path();
    if method.as_str() == "PROPFIND" && path.ends_with("/Movies/Demo.mkv") {
        return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:"><D:response><D:href>/dav/Movies/Demo.mkv</D:href><D:propstat><D:prop><D:getcontentlength>4</D:getcontentlength><D:getetag>"etag-demo"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response></D:multistatus>"#,
            )
                .into_response();
    }

    if method == Method::GET && path.ends_with("/Movies/Demo.mkv") {
        return (StatusCode::OK, [(header::CONTENT_LENGTH, "4")], "demo").into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}
