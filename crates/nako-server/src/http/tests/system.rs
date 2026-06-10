use std::sync::Arc;

use super::*;
use nako_api::admin::{
    AdminGeneratedArtifactMetadataApplyRecoveryResponse, AdminJobListItem, AdminJobPriority,
    AdminMetadataCandidateReviewApplicationAction, AdminMetadataCandidateReviewApplicationReason,
    AdminMetadataCandidateReviewApplyRequest, AdminMetadataCandidateReviewApplyResponse,
    AdminMetadataCandidateReviewAuditEventKind, AdminMetadataCandidateReviewBatchApplyItemRequest,
    AdminMetadataCandidateReviewBatchApplyRequest, AdminMetadataCandidateReviewBatchApplyResponse,
    AdminMetadataCandidateReviewBatchApplyResultStatus,
    AdminMetadataCandidateReviewBatchCreateRequest, AdminMetadataCandidateReviewBatchPlanRequest,
    AdminMetadataCandidateReviewBatchPlanResponse, AdminMetadataCandidateReviewBatchResponse,
    AdminMetadataCandidateReviewListResponse, AdminMetadataCandidateReviewQueueResponse,
    AdminMetadataCandidateReviewRelatedHierarchyApplicationAction,
    AdminMetadataCandidateReviewRelatedHierarchyApplyRequest,
    AdminMetadataCandidateReviewRelatedHierarchyApplyResponse,
    AdminMetadataCandidateReviewRelatedHierarchyPlanRequest,
    AdminMetadataCandidateReviewRelatedHierarchyPlanResponse, AdminMetadataCandidateReviewResponse,
    AdminMetadataCandidateReviewUndoMode, AdminMetadataCandidateReviewUndoReason,
    AdminOperatorReadinessArea, AdminOperatorReadinessReason, AdminOperatorReadinessStatus,
    AdminSourceDuplicateReconciliationApplyExpectedAction,
    AdminSourceDuplicateReconciliationApplyRequest,
    AdminSourceDuplicateReconciliationApplyResponse,
    AdminSourceDuplicateReconciliationPlanResponse, AdminSourceFingerprintHashEnqueueRequest,
    AdminSourceFingerprintHashMode, AdminSourceFingerprintHashRetryRequest,
    AdminStorageBackendHealthDiagnosticsResponse, AdminStorageBackendHealthResetResponse,
    AdminStorageStagingPressureStatus, AdminVfsCacheRefreshResponse,
    AdminVfsCacheRepairActionPlanReason, AdminVfsCacheRepairActionPlanResponse,
    AdminVfsCacheRepairActionPlanStatus, AdminVfsCacheRepairAutomationEnqueueRequest,
    AdminVfsCacheRepairAutomationEnqueueResponse, AdminVfsCacheRepairAutomationPlanResponse,
    AdminVfsCacheRepairAutomationPolicyRequest, AdminVfsCacheRepairCacheState,
    AdminVfsCacheRepairEnqueueOutcome, AdminVfsCacheRepairEnqueueRequest,
    AdminVfsCacheRepairEnqueueResponse, AdminVfsCacheRepairExecuteResponse,
    AdminVfsCacheRepairJobDiagnosticStatus, AdminVfsCacheRepairRemediationPlanResponse,
    AdminVfsCacheRepairRetryRequest, AdminVfsCacheRepairTargetListResponse,
    AdminVfsCacheRepairTargetPreviewResponse, AdminWatchFolderRuntimeCoverageStatus,
};
use nako_core::{
    JobKind, JobPriority, JobRepository, JobStatus,
    METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS, MetadataCandidateRecord,
    MetadataCandidateRelationshipKind, MetadataCandidateReviewBatchStatus,
    MetadataCandidateReviewId, MetadataCandidateReviewNode, MetadataCandidateReviewPlan,
    MetadataCandidateReviewRelationship, MetadataCandidateReviewRepository,
    MetadataCandidateReviewStatus as DurableMetadataCandidateReviewStatus, MetadataCandidateSource,
    MetadataCandidateSubject, NewJob, NewMetadataCandidateReview, ProviderMappingStatus,
    StorageBackendHealthRecord, StorageBackendHealthRepository, StorageBackendHealthStatus,
    StorageCircuitBreakerState, StorageFailureClass, VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS,
    VfsCacheFailureAuthority, VfsCacheRepairJobInput, VfsCachedObject, VfsCachedObjectKind,
};
use nako_library::{
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, SourceFingerprintHashJobInput,
    SourceFingerprintHashMode,
};

fn system_process_backed_hls_playlist_readiness_timeout() -> Duration {
    // Full-suite HLS gates on Windows start many fake FFmpeg processes at once.
    // Keep this guard above that startup tail while still bounding a hang.
    let seconds = if cfg!(windows) { 180 } else { 60 };
    Duration::from_secs(seconds)
}

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
        health_response.headers()[nako_api::public_client::API_VERSION_HEADER],
        nako_api::public_client::API_VERSION
    );
    let request_id = health_response
        .headers()
        .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(request_id.starts_with("req_"));
    let health = body_json::<HealthResponse>(health_response).await;
    let libraries = request_json::<LibraryListResponse>(&router, Method::GET, "/libraries").await;
    let library =
        request_json::<LibraryResponse>(&router, Method::GET, &format!("/libraries/{library_id}"))
            .await;

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, nako_api::public_client::API_VERSION);
    assert_eq!(libraries.libraries.len(), 1);
    assert_eq!(libraries.libraries[0].id, library_id.to_string());
    assert_eq!(library.library.id, library_id.to_string());
    assert_eq!(libraries.page.limit, nako_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(libraries.page.offset, 0);
    assert_eq!(libraries.page.returned, 1);
}

#[tokio::test]
async fn http_trace_context_echoes_safe_request_id_and_replaces_unsafe_input() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let echoed = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header(
                    &crate::http::trace_context::X_REQUEST_ID_HEADER,
                    "REQ-ABC_123.trace",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        echoed.headers()[crate::http::trace_context::X_REQUEST_ID_HEADER],
        "req-abc_123.trace"
    );

    let replaced = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header(
                    &crate::http::trace_context::X_REQUEST_ID_HEADER,
                    "https://secret.example/path?token=private",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let request_id = replaced.headers()[crate::http::trace_context::X_REQUEST_ID_HEADER]
        .to_str()
        .unwrap();
    assert!(request_id.starts_with("req_"));
    assert!(!request_id.contains("secret"));
    assert!(!request_id.contains("token"));
    assert_ne!(request_id, "https://secret.example/path?token=private");
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
    assert_eq!(backend.health.last_error_class, None);
    assert_eq!(backend.health.backoff_until_ms, None);

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
        response.headers()[nako_api::public_client::API_VERSION_HEADER],
        nako_api::public_client::API_VERSION
    );

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    let overview: AdminOverviewResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        overview.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        overview.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(overview.status, AdminOverviewStatus::Healthy);
    assert_eq!(
        overview.operator_readiness.status,
        AdminOperatorReadinessStatus::Degraded
    );
    assert_eq!(overview.operator_readiness.checks.len(), 6);
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Setup,
        AdminOperatorReadinessStatus::Degraded,
        AdminOperatorReadinessReason::AuthDisabledLocalOnly,
        Some("systemConfig"),
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::MediaLibraryScan,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::MediaLibraryConfigured,
        None,
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Playback,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::PlaybackReady,
        None,
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Storage,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::StorageReady,
        None,
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Network,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::NetworkReady,
        None,
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Backup,
        AdminOperatorReadinessStatus::Degraded,
        AdminOperatorReadinessReason::BackupNeedsDurableDatabase,
        Some("systemConfig"),
    );
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
    assert_eq!(overview.catalog.governed_items, 0);
    assert_eq!(overview.catalog.unknown_kind_items, 0);
    assert_eq!(overview.catalog.low_confidence_items, 0);
    assert_eq!(overview.catalog.items_with_duplicate_relationships, 0);
    assert_eq!(overview.catalog.items_missing_accepted_provider_mapping, 0);
    assert_eq!(overview.metadata.total_providers, 0);
    assert_eq!(overview.runtime.failed_tasks, 0);
    assert_eq!(overview.runtime.cancelled_jobs, 0);
    assert_eq!(overview.runtime.failed_jobs, 0);
    assert!(!overview.runtime.shutdown_requested);
    assert_eq!(overview.source_fingerprint_hash.total_sources, 0);
    assert_eq!(overview.source_fingerprint_hash.fingerprinted_sources, 0);
    assert_eq!(overview.source_fingerprint_hash.content_hash_sources, 0);
    assert_eq!(overview.source_fingerprint_hash.queued_jobs, 0);
    assert_eq!(overview.source_fingerprint_hash.claimable_jobs, 0);
    assert_eq!(overview.source_fingerprint_hash.delayed_retry_jobs, 0);
    assert_eq!(overview.source_fingerprint_hash.failed_jobs, 0);
    assert_eq!(overview.source_fingerprint_hash.oldest_queued_at, None);
    assert_eq!(overview.source_fingerprint_hash.next_retry_at, None);
    assert_eq!(overview.startup.configured_libraries, 1);
    assert_eq!(overview.startup.recovered_jobs, 0);
    assert_eq!(overview.startup.recovered_transcode_sessions, 0);
    assert_eq!(
        overview.startup.watch_folder_runtime.configured_libraries,
        1
    );
    assert_eq!(
        overview
            .startup
            .watch_folder_runtime
            .realtime_enabled_libraries,
        0
    );
    assert_eq!(overview.startup.watch_folder_runtime.started_libraries, 0);
    assert_eq!(overview.startup.watch_folder_runtime.skipped_libraries, 1);
    assert_eq!(
        overview.startup.watch_folder_runtime.diagnostics[0].status,
        AdminWatchFolderRuntimeCoverageStatus::Disabled
    );
    assert_eq!(
        overview.startup.watch_folder_runtime.diagnostics[0].root_ref_redacted,
        "local://<redacted>"
    );

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

fn assert_operator_readiness_check(
    overview: &AdminOverviewResponse,
    area: AdminOperatorReadinessArea,
    status: AdminOperatorReadinessStatus,
    reason: AdminOperatorReadinessReason,
    action_route_key: Option<&str>,
) {
    let check = overview
        .operator_readiness
        .checks
        .iter()
        .find(|check| check.area == area)
        .expect("operator readiness check");

    assert_eq!(check.status, status);
    assert_eq!(check.reason, reason);

    match action_route_key {
        Some(route_key) => {
            let action = check.action.as_ref().expect("operator action");
            assert_eq!(action.route_key, route_key);
            assert!(action.route_path.starts_with("/admin/v1/"));
        }
        None => assert!(check.action.is_none()),
    }
}

#[tokio::test]
async fn admin_v1_overview_reports_operator_readiness_for_configured_local_install() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: temp.path().join("nako.db").display().to_string(),
        database_url_env: None,
        auth: crate::config::AuthConfig {
            enabled: true,
            token_env: Some("NAKO_ADMIN_TOKEN".to_owned()),
        },
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    let router = build_router_with_auth(app, auth::InboundAuthState::bearer_token("test-token"));

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/overview")
                .header(header::AUTHORIZATION, "Bearer test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    let overview: AdminOverviewResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        overview.operator_readiness.status,
        AdminOperatorReadinessStatus::Ready
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Setup,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::AuthConfigured,
        None,
    );
    assert_operator_readiness_check(
        &overview,
        AdminOperatorReadinessArea::Backup,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::BackupRunbookAvailable,
        None,
    );

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("test-token"));
    assert!(!body.contains("NAKO_ADMIN_TOKEN"));
    assert!(!body.contains("ffmpeg -"));
    assert!(!body.contains("root_uri"));
}

#[tokio::test]
async fn admin_v1_catalog_governance_lists_unknown_low_confidence_and_redacts_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let unknown = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Unknown,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Unmatched Local File".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let weak = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Weak Match".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let confident = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Confident Match".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let unknown_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: unknown.id,
        locator: "local:///Movies/Private/Unmatched.Local.File.mkv".to_owned(),
        file_name: "Unmatched.Local.File.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let weak_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: weak.id,
        locator: "local:///Movies/Weak.Match.mkv".to_owned(),
        file_name: "Weak.Match.mkv".to_owned(),
        size_bytes: Some(84),
        fingerprint: None,
    };
    let confident_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: confident.id,
        locator: "local:///Movies/Confident.Match.mkv".to_owned(),
        file_name: "Confident.Match.mkv".to_owned(),
        size_bytes: Some(168),
        fingerprint: None,
    };
    let weak_subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "100".to_owned(),
        title: Some("Weak Match".to_owned()),
        release_year: None,
        locale: Some("en-US".to_owned()),
    };

    for item in [&unknown, &weak, &confident] {
        store.upsert_media_item(item).await.unwrap();
    }
    for source in [&unknown_source, &weak_source, &confident_source] {
        store.upsert_media_source(source).await.unwrap();
    }
    store.upsert_provider_subject(&weak_subject).await.unwrap();
    store
        .upsert_provider_mapping(&ProviderMapping {
            id: ProviderMappingId::new(),
            item_id: weak.id,
            subject_id: weak_subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: Some(900),
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
        })
        .await
        .unwrap();
    store
        .upsert_source_duplicate_relationship(&nako_core::SourceDuplicateRelationship {
            id: nako_core::SourceDuplicateRelationshipId::new(),
            source_id: confident_source.id,
            duplicate_source_id: weak_source.id,
            evidence_kind: nako_core::SourceDuplicateEvidenceKind::StrongFingerprint,
            evidence_value: Some("sha256:confident".to_owned()),
            status: nako_core::SourceDuplicateRelationshipStatus::Suggested,
            confidence_milli: Some(930),
        })
        .await
        .unwrap();
    for (source, kind, confidence, evidence_value) in [
        (
            &unknown_source,
            MediaKind::Unknown,
            350,
            "local:///Movies/Private/secret-evidence.mkv",
        ),
        (
            &weak_source,
            MediaKind::Movie,
            640,
            "local:///Movies/Weak.Match.mkv",
        ),
        (
            &confident_source,
            MediaKind::Movie,
            920,
            "local:///Movies/Confident.Match.mkv",
        ),
    ] {
        store
            .upsert_local_inference_evidence(&LocalInferenceEvidence {
                id: LocalInferenceEvidenceId::new(),
                source_id: source.id,
                inferred_kind: kind,
                inferred_title: Some(source.file_name.trim_end_matches(".mkv").replace('.', " ")),
                inferred_year: None,
                inferred_season: None,
                inferred_episode: None,
                confidence_milli: Some(confidence),
                evidence_source: LocalInferenceEvidenceSource::Path,
                evidence_value: evidence_value.to_owned(),
                inference_version: "nako-naming:1".to_owned(),
            })
            .await
            .unwrap();
    }

    let router = build_router(app.clone());
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/catalog/governance/items?library_id={library_id}&max_confidence_milli=700&limit=10"
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
    let response: AdminCatalogGovernanceItemListResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(response.items.len(), 3);
    assert_eq!(response.items[0].item_id, unknown.id);
    assert_eq!(response.items[0].kind, MediaKind::Unknown);
    assert_eq!(response.items[0].source_count, 1);
    assert_eq!(
        response.items[0]
            .local_inference
            .as_ref()
            .unwrap()
            .confidence_milli,
        Some(350)
    );
    assert_eq!(response.items[0].provider_mapping_count, 0);
    assert_eq!(response.items[1].item_id, weak.id);
    assert_eq!(response.items[1].kind, MediaKind::Movie);
    assert_eq!(
        response.items[1]
            .local_inference
            .as_ref()
            .unwrap()
            .confidence_milli,
        Some(640)
    );
    assert_eq!(response.items[1].provider_mapping_count, 1);
    assert_eq!(response.items[1].accepted_provider_mapping_count, 1);
    assert_eq!(response.items[1].duplicate_relationship_count, 1);
    assert_eq!(response.items[2].item_id, confident.id);
    assert_eq!(response.items[2].kind, MediaKind::Movie);
    assert_eq!(
        response.items[2]
            .local_inference
            .as_ref()
            .unwrap()
            .confidence_milli,
        Some(920)
    );
    assert_eq!(response.items[2].duplicate_relationship_count, 1);
    assert_eq!(response.page.limit, 10);
    assert_eq!(response.page.returned, 3);
    assert!(!body.contains("evidence_value"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("secret-evidence"));
    assert!(!body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_catalog_governance_provider_mapping_review_plan_is_redacted() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Candidate Mapping".to_owned(),
            release_date: Some("2026-05-25".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Private/Candidate.Mapping.mkv?token=secret".to_owned(),
        file_name: "Candidate.Mapping.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    let subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "603".to_owned(),
        title: Some("The Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("en-US".to_owned()),
    };
    let mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: item.id,
        subject_id: subject.id,
        status: ProviderMappingStatus::Candidate,
        confidence_milli: Some(820),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_local_inference_evidence(&LocalInferenceEvidence {
            id: LocalInferenceEvidenceId::new(),
            source_id: source.id,
            inferred_kind: MediaKind::Movie,
            inferred_title: Some("Candidate Mapping".to_owned()),
            inferred_year: Some(2026),
            inferred_season: None,
            inferred_episode: None,
            confidence_milli: Some(510),
            evidence_source: LocalInferenceEvidenceSource::Path,
            evidence_value: "local:///Movies/Private/raw-evidence-token.mkv".to_owned(),
            inference_version: "nako-naming:1".to_owned(),
        })
        .await
        .unwrap();
    store.upsert_provider_subject(&subject).await.unwrap();
    store.upsert_provider_mapping(&mapping).await.unwrap();

    let router = build_router(app.clone());
    let detail_path = format!("/admin/v1/catalog/governance/items/{}", item.id);
    let detail_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&detail_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let detail_status = detail_response.status();
    let detail_body = String::from_utf8(
        to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    let detail: AdminCatalogGovernanceItemDetailResponse =
        serde_json::from_str(&detail_body).unwrap();

    assert_eq!(detail.item.item_id, item.id);
    assert_eq!(detail.provider_mappings.len(), 1);
    assert_eq!(detail.provider_mappings[0].mapping_id, mapping.id);
    assert_eq!(
        detail.provider_mappings[0].status,
        ProviderMappingStatus::Candidate
    );
    assert_eq!(detail.provider_mappings[0].subject.subject_key, "603");
    assert_eq!(
        detail.repair_actions[0],
        AdminCatalogGovernanceRepairAction::ProviderMappingReview
    );
    assert!(!detail_body.contains("evidence_value"));
    assert!(!detail_body.contains("local:///"));
    assert!(!detail_body.contains("raw-evidence-token"));
    assert!(!detail_body.contains("sha256-private-fingerprint"));
    assert!(!detail_body.contains(&temp.path().display().to_string()));

    let review_plan_path = format!(
        "/admin/v1/catalog/governance/items/{}/provider-mappings/{}/review-plan",
        item.id, mapping.id
    );
    let review_plan_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&review_plan_path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&AdminCatalogGovernanceProviderMappingReviewRequest {
                        decision: AdminCatalogGovernanceProviderMappingReviewDecision::Accept,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let review_plan_status = review_plan_response.status();
    let review_plan_body = String::from_utf8(
        to_bytes(review_plan_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(review_plan_status, StatusCode::OK, "{review_plan_body}");
    let plan: AdminCatalogGovernanceProviderMappingReviewPlanResponse =
        serde_json::from_str(&review_plan_body).unwrap();

    assert_eq!(plan.plan.item.item_id, item.id);
    assert_eq!(plan.plan.mapping.mapping_id, mapping.id);
    assert_eq!(plan.plan.current_status, ProviderMappingStatus::Candidate);
    assert_eq!(plan.plan.target_status, ProviderMappingStatus::Accepted);
    assert_eq!(
        plan.plan.status,
        AdminCatalogGovernanceRepairPlanStatus::Ready
    );
    assert!(plan.plan.readiness.actionable);
    assert!(plan.plan.boundary.updates_provider_mapping_status);
    assert!(!plan.plan.boundary.updates_canonical_metadata);
    assert!(!plan.plan.boundary.writes_nfo);
    assert!(!plan.plan.boundary.writes_library_files);
    assert!(!review_plan_body.contains("evidence_value"));
    assert!(!review_plan_body.contains("local:///"));
    assert!(!review_plan_body.contains("raw-evidence-token"));
    assert!(!review_plan_body.contains("sha256-private-fingerprint"));
    assert!(!review_plan_body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_catalog_governance_provider_mapping_review_mutates_idempotently() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Candidate Mapping".to_owned(),
            release_date: Some("2026-05-25".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Private/Candidate.Mapping.mkv?token=secret".to_owned(),
        file_name: "Candidate.Mapping.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    let subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "603".to_owned(),
        title: Some("The Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("en-US".to_owned()),
    };
    let mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: item.id,
        subject_id: subject.id,
        status: ProviderMappingStatus::Candidate,
        confidence_milli: Some(820),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_local_inference_evidence(&LocalInferenceEvidence {
            id: LocalInferenceEvidenceId::new(),
            source_id: source.id,
            inferred_kind: MediaKind::Movie,
            inferred_title: Some("Candidate Mapping".to_owned()),
            inferred_year: Some(2026),
            inferred_season: None,
            inferred_episode: None,
            confidence_milli: Some(510),
            evidence_source: LocalInferenceEvidenceSource::Path,
            evidence_value: "local:///Movies/Private/raw-evidence-token.mkv".to_owned(),
            inference_version: "nako-naming:1".to_owned(),
        })
        .await
        .unwrap();
    store.upsert_provider_subject(&subject).await.unwrap();
    store.upsert_provider_mapping(&mapping).await.unwrap();

    let router = build_router(app);
    let review_path = format!(
        "/admin/v1/catalog/governance/items/{}/provider-mappings/{}/review",
        item.id, mapping.id
    );
    let request_body = serde_json::to_vec(&AdminCatalogGovernanceProviderMappingReviewRequest {
        decision: AdminCatalogGovernanceProviderMappingReviewDecision::Accept,
    })
    .unwrap();
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&review_path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(request_body.clone()))
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
    let review: AdminCatalogGovernanceProviderMappingReviewResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(review.item_id, item.id);
    assert_eq!(review.mapping_id, mapping.id);
    assert_eq!(
        review.decision,
        AdminCatalogGovernanceProviderMappingReviewDecision::Accept
    );
    assert_eq!(review.previous_status, ProviderMappingStatus::Candidate);
    assert_eq!(review.current_status, ProviderMappingStatus::Accepted);
    assert!(review.changed);
    assert!(!review.idempotent_replay);
    assert!(review.plan.boundary.updates_provider_mapping_status);
    assert!(!review.plan.boundary.updates_canonical_metadata);
    assert!(!body.contains("evidence_value"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("raw-evidence-token"));
    assert!(!body.contains("sha256-private-fingerprint"));
    assert!(!body.contains(&temp.path().display().to_string()));

    let stored = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(stored[0].status, ProviderMappingStatus::Accepted);

    let replay_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(&review_path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();
    let replay_status = replay_response.status();
    let replay_body = String::from_utf8(
        to_bytes(replay_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replay: AdminCatalogGovernanceProviderMappingReviewResponse =
        serde_json::from_str(&replay_body).unwrap();

    assert_eq!(replay.previous_status, ProviderMappingStatus::Accepted);
    assert_eq!(replay.current_status, ProviderMappingStatus::Accepted);
    assert!(!replay.changed);
    assert!(replay.idempotent_replay);
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_list_is_item_scoped_redacted_and_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "List Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let other_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Other Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Private/List.Candidate.S01E01.mkv?token=secret".to_owned(),
        file_name: "List.Candidate.S01E01.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-list-review".to_owned()),
    };
    let newer_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "newer".to_owned(),
        title: Some("Newer Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let older_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "older".to_owned(),
        title: Some("Older Candidate".to_owned()),
        release_year: Some(2025),
        locale: Some("zh-CN".to_owned()),
    };
    let related_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "newer/1".to_owned(),
        title: Some("Episode One".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let other_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "other-secret-subject".to_owned(),
        title: Some("Other Candidate".to_owned()),
        release_year: Some(2024),
        locale: Some("en-US".to_owned()),
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_item(&other_item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: other_item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let newer_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:newer".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(newer_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Newer Candidate".to_owned()),
                        overview: Some("newer secret overview".to_owned()),
                        release_date: Some("2026-06-02".to_owned()),
                        tags: vec!["newer-secret-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Episode,
                    subject: Some(related_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Episode One".to_owned()),
                        overview: Some("related secret overview".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                }],
                relationships: vec![MetadataCandidateReviewRelationship {
                    parent_subject: newer_subject.clone(),
                    child_subject: related_subject,
                    kind: MetadataCandidateRelationshipKind::Contains,
                }],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let older_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:older".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(older_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Older Candidate".to_owned()),
                        overview: Some("older secret overview".to_owned()),
                        release_date: Some("2025-01-01".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 90,
            updated_at_ms: 100,
        })
        .await
        .unwrap();
    let other_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: other_item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
            source_key: "tmdb:other".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                    kind: MediaKind::Movie,
                    subject: Some(other_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Other Candidate".to_owned()),
                        overview: Some("other item secret overview".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 80,
            updated_at_ms: 700,
        })
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            newer_review.id,
            DurableMetadataCandidateReviewStatus::Pending,
            500,
        )
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            older_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            other_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            700,
        )
        .await
        .unwrap();

    let router = build_router(app);
    let path = format!(
        "/admin/v1/metadata/items/{}/candidate-reviews?limit=2",
        item.id
    );
    let response = router
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
    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let list: AdminMetadataCandidateReviewListResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(list.item_id, item.id);
    assert_eq!(list.page.limit, 2);
    assert_eq!(list.page.offset, 0);
    assert_eq!(list.page.returned, 2);
    assert_eq!(list.reviews.len(), 2);
    assert_eq!(list.reviews[0].review_id, newer_review.id);
    assert_eq!(
        list.reviews[0].status,
        DurableMetadataCandidateReviewStatus::Pending
    );
    assert_eq!(
        list.reviews[0].root.metadata.title.as_deref(),
        Some("Newer Candidate")
    );
    assert_eq!(list.reviews[0].related_count, 1);
    assert_eq!(list.reviews[0].relationship_count, 1);
    assert_eq!(
        list.reviews[0].application_plan.action,
        AdminMetadataCandidateReviewApplicationAction::Skip
    );
    assert!(!list.reviews[0].boundary.apply_mutation_required);
    assert!(list.reviews[0].governance.audit_timeline.read_only);
    assert!(list.reviews[0].governance.audit_timeline.replay_safe);
    assert_eq!(
        list.reviews[0].governance.audit_timeline.events[0].kind,
        AdminMetadataCandidateReviewAuditEventKind::ReviewCreated
    );
    assert_eq!(
        list.reviews[0].governance.undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::NoMutationObserved
    );
    assert!(
        list.reviews[0]
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::NoProviderMappingMutationObserved)
    );
    assert_eq!(list.reviews[1].review_id, older_review.id);
    assert_eq!(
        list.reviews[1].application_plan.action,
        AdminMetadataCandidateReviewApplicationAction::Apply
    );
    assert_eq!(
        list.reviews[1].application_plan.reasons,
        vec![AdminMetadataCandidateReviewApplicationReason::Ready]
    );
    assert!(list.reviews[1].boundary.read_only);
    assert!(!list.reviews[1].boundary.applies_on_read);
    assert!(list.reviews[1].boundary.apply_mutation_required);
    assert!(!list.reviews[1].boundary.updates_hierarchy);
    assert_eq!(
        list.reviews[1].governance.audit_timeline.events[1].kind,
        AdminMetadataCandidateReviewAuditEventKind::ReviewStatusCurrent
    );
    assert_eq!(
        list.reviews[1].governance.undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::DeferredUntilApplyOutcomeAudit
    );
    assert_eq!(
        list.reviews[1]
            .governance
            .undo_plan
            .stale_state_guard_updated_at_ms,
        Some(300)
    );
    assert!(
        list.reviews[1]
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::ApplyOutcomeAuditRequired)
    );
    assert!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "newer"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!body.contains(&other_review.id.to_string()));
    assert!(!body.contains("other-secret-subject"));
    assert!(!body.contains("overview"));
    assert!(!body.contains("newer secret overview"));
    assert!(!body.contains("older secret overview"));
    assert!(!body.contains("related secret overview"));
    assert!(!body.contains("newer-secret-tag"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("sha256-private-list-review"));
    assert!(!body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_queue_filters_global_rows_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Queue Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let other_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Other Queue Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_item(&other_item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: other_item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: item.id,
            locator: "local:///Private/Queue.Candidate.S01E01.mkv?token=secret".to_owned(),
            file_name: "Queue.Candidate.S01E01.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("sha256-private-queue-review".to_owned()),
        })
        .await
        .unwrap();

    let pending_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "pending".to_owned(),
        title: Some("Pending Queue Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let accepted_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "accepted".to_owned(),
        title: Some("Accepted Queue Candidate".to_owned()),
        release_year: Some(2025),
        locale: Some("zh-CN".to_owned()),
    };
    let other_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "queue-other".to_owned(),
        title: Some("Other Queue Candidate".to_owned()),
        release_year: Some(2024),
        locale: Some("en-US".to_owned()),
    };

    let pending_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:pending".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(pending_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Pending Queue Candidate".to_owned()),
                        overview: Some("pending queue secret overview".to_owned()),
                        tags: vec!["pending-secret-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let accepted_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:accepted".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(accepted_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Accepted Queue Candidate".to_owned()),
                        overview: Some("accepted queue secret overview".to_owned()),
                        release_date: Some("2025-01-01".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 90,
            updated_at_ms: 100,
        })
        .await
        .unwrap();
    let other_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: other_item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
            source_key: "tmdb:queue-other".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                    kind: MediaKind::Movie,
                    subject: Some(other_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Other Queue Candidate".to_owned()),
                        overview: Some("other queue secret overview".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 80,
            updated_at_ms: 700,
        })
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            pending_review.id,
            DurableMetadataCandidateReviewStatus::Pending,
            500,
        )
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            accepted_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            other_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            700,
        )
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/metadata/candidate-reviews?status=accepted&limit=2")
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
    let queue: AdminMetadataCandidateReviewQueueResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(queue.page.limit, 2);
    assert_eq!(queue.page.offset, 0);
    assert_eq!(queue.page.returned, 2);
    assert_eq!(queue.reviews.len(), 2);
    assert_eq!(queue.reviews[0].review_id, other_review.id);
    assert_eq!(queue.reviews[0].item_id, other_item.id);
    assert_eq!(queue.reviews[1].review_id, accepted_review.id);
    assert_eq!(queue.reviews[1].item_id, item.id);
    assert_eq!(
        queue.reviews[1].application_plan.action,
        AdminMetadataCandidateReviewApplicationAction::Apply
    );
    assert!(queue.reviews[1].boundary.read_only);
    assert!(!queue.reviews[1].boundary.updates_hierarchy);
    assert!(!body.contains(&pending_review.id.to_string()));
    assert!(!body.contains("overview"));
    assert!(!body.contains("pending queue secret overview"));
    assert!(!body.contains("accepted queue secret overview"));
    assert!(!body.contains("other queue secret overview"));
    assert!(!body.contains("pending-secret-tag"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("sha256-private-queue-review"));
    assert!(!body.contains(&temp.path().display().to_string()));

    let provider_response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(
                    "/admin/v1/metadata/candidate-reviews?status=accepted&provider=bangumi&limit=10",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let provider_status = provider_response.status();
    let provider_body = String::from_utf8(
        to_bytes(provider_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(provider_status, StatusCode::OK, "{provider_body}");
    let provider_queue: AdminMetadataCandidateReviewQueueResponse =
        serde_json::from_str(&provider_body).unwrap();
    assert_eq!(provider_queue.page.returned, 1);
    assert_eq!(provider_queue.reviews[0].review_id, accepted_review.id);
    assert!(!provider_body.contains(&other_review.id.to_string()));

    assert!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_provider_mappings_for_item(other_item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "accepted"
            )
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_batch_plan_is_bounded_redacted_and_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Batch Plan Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let accepted_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "batch-accepted".to_owned(),
        title: Some("Batch Accepted".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let pending_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "batch-pending".to_owned(),
        title: Some("Batch Pending".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let accepted_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-accepted".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(accepted_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Batch Accepted".to_owned()),
                        overview: Some("batch secret accepted overview".to_owned()),
                        tags: vec!["batch-secret-accepted-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let pending_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-pending".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(pending_subject),
                    metadata: MetadataCandidateRecord {
                        title: Some("Batch Pending".to_owned()),
                        overview: Some("batch secret pending overview".to_owned()),
                        tags: vec!["batch-secret-pending-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 90,
            updated_at_ms: 300,
        })
        .await
        .unwrap();
    let accepted_review = store
        .set_metadata_candidate_review_status(
            accepted_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            400,
        )
        .await
        .unwrap()
        .unwrap();

    let router = build_router(app);
    let path = "/admin/v1/metadata/candidate-reviews/batch-application-plan";
    let request = AdminMetadataCandidateReviewBatchPlanRequest {
        review_ids: vec![accepted_review.id, pending_review.id],
    };
    let response = response_body_json(&router, Method::POST, path, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let plan: AdminMetadataCandidateReviewBatchPlanResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(plan.summary.requested_count, 2);
    assert_eq!(plan.summary.returned_count, 2);
    assert_eq!(plan.summary.max_review_count, 50);
    assert_eq!(plan.summary.apply_count, 1);
    assert_eq!(plan.summary.skip_count, 1);
    assert_eq!(plan.reviews[0].review_id, accepted_review.id);
    assert_eq!(plan.reviews[1].review_id, pending_review.id);
    assert_eq!(
        plan.reviews[0].application_plan.action,
        AdminMetadataCandidateReviewApplicationAction::Apply
    );
    assert_eq!(
        plan.reviews[1].application_plan.action,
        AdminMetadataCandidateReviewApplicationAction::Skip
    );
    assert_eq!(
        plan.reviews[1].application_plan.reasons,
        vec![AdminMetadataCandidateReviewApplicationReason::ReviewNotAccepted]
    );
    assert!(plan.reviews.iter().all(|review| review.boundary.read_only));
    assert_eq!(
        plan.reviews[0].governance.audit_timeline.events[0].kind,
        AdminMetadataCandidateReviewAuditEventKind::ReviewCreated
    );
    assert_eq!(
        plan.reviews[0].governance.undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::DeferredUntilApplyOutcomeAudit
    );
    assert!(
        plan.reviews[1]
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::NoProviderMappingMutationObserved)
    );
    assert!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "batch-accepted"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!body.contains("overview"));
    assert!(!body.contains("batch secret accepted overview"));
    assert!(!body.contains("batch secret pending overview"));
    assert!(!body.contains("batch-secret-accepted-tag"));
    assert!(!body.contains("batch-secret-pending-tag"));

    let duplicate = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchPlanRequest {
            review_ids: vec![accepted_review.id, accepted_review.id],
        },
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);
    let duplicate_body = response_text(duplicate).await;
    let duplicate_error: ErrorResponse = serde_json::from_str(&duplicate_body).unwrap();
    assert_eq!(duplicate_error.code, "invalid_input");

    let too_many_review_ids = (0..=plan.summary.max_review_count)
        .map(|_| MetadataCandidateReviewId::new())
        .collect();
    let too_many = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchPlanRequest {
            review_ids: too_many_review_ids,
        },
    )
    .await;
    assert_eq!(too_many.status(), StatusCode::BAD_REQUEST);
    let too_many_body = response_text(too_many).await;
    let too_many_error: ErrorResponse = serde_json::from_str(&too_many_body).unwrap();
    assert_eq!(too_many_error.code, "invalid_input");
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_batch_durable_create_replays_and_reports_status() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Durable Batch Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let make_subject = |subject_key: &str, title: &str| MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: subject_key.to_owned(),
        title: Some(title.to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let make_review_node =
        |subject: MetadataCandidateSubject, title: &str| MetadataCandidateReviewNode {
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            kind: MediaKind::Series,
            subject: Some(subject),
            metadata: MetadataCandidateRecord {
                title: Some(title.to_owned()),
                overview: Some(format!("{title} private durable overview")),
                tags: vec![format!("{title} private durable tag")],
                ..MetadataCandidateRecord::default()
            },
        };

    let accepted_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:durable-batch-accepted".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(
                    make_subject("durable-batch-accepted", "Durable Batch Accepted"),
                    "Durable Batch Accepted",
                ),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let accepted_review = store
        .set_metadata_candidate_review_status(
            accepted_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap()
        .unwrap();
    let pending_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:durable-batch-pending".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(
                    make_subject("durable-batch-pending", "Durable Batch Pending"),
                    "Durable Batch Pending",
                ),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 110,
            updated_at_ms: 210,
        })
        .await
        .unwrap();

    let router = build_router(app.clone());
    let path = "/admin/v1/metadata/candidate-reviews/batches";
    let request = AdminMetadataCandidateReviewBatchCreateRequest {
        idempotency_key: "candidate-review:durable-secret".to_owned(),
        reviews: vec![
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: accepted_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(accepted_review.updated_at_ms),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: pending_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(pending_review.updated_at_ms),
            },
        ],
    };

    let response = response_body_json(&router, Method::POST, path, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let created: AdminMetadataCandidateReviewBatchResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        created.batch.status,
        MetadataCandidateReviewBatchStatus::Queued
    );
    assert_eq!(created.batch.selection.requested_review_count, 2);
    assert_eq!(created.batch.selection.selected_review_count, 2);
    assert_eq!(created.batch.summary.apply_count, 1);
    assert_eq!(created.batch.summary.skip_count, 1);
    assert_eq!(created.batch.execution_summary.pending_item_count, 1);
    assert_eq!(created.batch.execution_summary.skipped_item_count, 1);
    assert_eq!(created.batch.items.len(), 2);
    assert_eq!(created.batch.items[0].review_id, accepted_review.id);
    assert_eq!(
        created.batch.items[0].plan.action,
        AdminMetadataCandidateReviewApplicationAction::Apply
    );
    assert_eq!(
        created.batch.items[1].plan.action,
        AdminMetadataCandidateReviewApplicationAction::Skip
    );
    assert!(!created.batch.idempotency_key_fingerprint.is_empty());
    assert!(
        !created.batch.items[0]
            .idempotency_key_fingerprint
            .is_empty()
    );

    let job = store.get_job(created.batch.job_id).await.unwrap().unwrap();
    assert_eq!(job.kind, JobKind::MetadataCandidateReviewBatchApply);
    assert_eq!(
        job.resource_class,
        METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS
    );
    assert!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "durable-batch-accepted"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!body.contains("candidate-review:durable-secret"));
    assert!(!body.contains("private durable overview"));
    assert!(!body.contains("private durable tag"));

    let replay = response_body_json(&router, Method::POST, path, &request).await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminMetadataCandidateReviewBatchResponse =
        serde_json::from_str(&replay_body).unwrap();
    assert_eq!(replayed.batch.id, created.batch.id);
    assert_eq!(replayed.batch.job_id, created.batch.job_id);
    assert_eq!(replayed.batch.items, created.batch.items);
    assert!(!replay_body.contains("candidate-review:durable-secret"));

    let status_uri = format!(
        "/admin/v1/metadata/candidate-reviews/batches/{}",
        created.batch.id
    );
    let status_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&status_uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_code = status_response.status();
    let status_body = response_text(status_response).await;
    assert_eq!(status_code, StatusCode::OK, "{status_body}");
    let status_batch: AdminMetadataCandidateReviewBatchResponse =
        serde_json::from_str(&status_body).unwrap();
    assert_eq!(status_batch.batch.id, created.batch.id);
    assert_eq!(
        status_batch.batch.status,
        MetadataCandidateReviewBatchStatus::Queued
    );
    assert!(!status_body.contains("candidate-review:durable-secret"));

    let executed = app
        .metadata()
        .execute_admin_metadata_candidate_review_batch(created.batch.id)
        .await
        .unwrap();
    assert_eq!(
        executed.status,
        MetadataCandidateReviewBatchStatus::Completed
    );
    assert_eq!(executed.execution_summary.applied_item_count, 1);
    assert_eq!(executed.execution_summary.skipped_item_count, 1);
    assert_eq!(executed.execution_summary.pending_item_count, 0);
    assert_eq!(
        executed.items[0].status,
        nako_core::MetadataCandidateReviewBatchItemStatus::Applied
    );
    assert_eq!(
        executed.items[1].status,
        nako_core::MetadataCandidateReviewBatchItemStatus::Skipped
    );

    let executed_job = store.get_job(created.batch.job_id).await.unwrap().unwrap();
    assert_eq!(executed_job.status, JobStatus::Succeeded);
    assert!(executed_job.summary_json.is_some());
    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(provider_mappings.len(), 1);
    assert_eq!(provider_mappings[0].status, ProviderMappingStatus::Accepted);
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "durable-batch-accepted"
            )
            .await
            .unwrap()
            .is_some()
    );

    let completed_status_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&status_uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let completed_status_code = completed_status_response.status();
    let completed_status_body = response_text(completed_status_response).await;
    assert_eq!(
        completed_status_code,
        StatusCode::OK,
        "{completed_status_body}"
    );
    let completed_status: AdminMetadataCandidateReviewBatchResponse =
        serde_json::from_str(&completed_status_body).unwrap();
    assert_eq!(
        completed_status.batch.status,
        MetadataCandidateReviewBatchStatus::Completed
    );
    assert_eq!(
        completed_status.batch.execution_summary.applied_item_count,
        1
    );
    assert!(!completed_status_body.contains("candidate-review:durable-secret"));

    let cancel_response = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchCreateRequest {
            idempotency_key: "candidate-review:durable-cancel".to_owned(),
            reviews: vec![AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: accepted_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(accepted_review.updated_at_ms),
            }],
        },
    )
    .await;
    let cancel_status = cancel_response.status();
    let cancel_body = response_text(cancel_response).await;
    assert_eq!(cancel_status, StatusCode::OK, "{cancel_body}");
    let cancel_created: AdminMetadataCandidateReviewBatchResponse =
        serde_json::from_str(&cancel_body).unwrap();
    let cancellation = app
        .jobs()
        .request_job_cancellation(cancel_created.batch.job_id)
        .await
        .unwrap();
    assert!(cancellation.requested);
    assert!(cancellation.terminal);
    assert_eq!(cancellation.job.status, JobStatus::Cancelled);
    let cancelled_batch = app
        .metadata()
        .execute_admin_metadata_candidate_review_batch(cancel_created.batch.id)
        .await
        .unwrap();
    assert_eq!(
        cancelled_batch.status,
        MetadataCandidateReviewBatchStatus::Cancelled
    );

    let duplicate = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchCreateRequest {
            idempotency_key: "candidate-review:durable-duplicate".to_owned(),
            reviews: vec![
                AdminMetadataCandidateReviewBatchApplyItemRequest {
                    review_id: accepted_review.id,
                    item_id: item.id,
                    expected_updated_at_ms: Some(accepted_review.updated_at_ms),
                },
                AdminMetadataCandidateReviewBatchApplyItemRequest {
                    review_id: accepted_review.id,
                    item_id: item.id,
                    expected_updated_at_ms: Some(accepted_review.updated_at_ms),
                },
            ],
        },
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);
    let duplicate_body = response_text(duplicate).await;
    let duplicate_error: ErrorResponse = serde_json::from_str(&duplicate_body).unwrap();
    assert_eq!(duplicate_error.code, "invalid_input");
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_batch_apply_reports_partial_results_redacted() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Batch Apply Candidate".to_owned(),
            release_date: Some("2026-06-02".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Private/Batch.Apply.Candidate.S01E01.mkv?token=batch-secret".to_owned(),
        file_name: "Batch.Apply.Candidate.S01E01.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-batch-apply".to_owned()),
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let make_subject = |subject_key: &str, title: &str| MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: subject_key.to_owned(),
        title: Some(title.to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let make_review_node =
        |subject: Option<MetadataCandidateSubject>, title: &str| MetadataCandidateReviewNode {
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            kind: MediaKind::Series,
            subject,
            metadata: MetadataCandidateRecord {
                title: Some(title.to_owned()),
                overview: Some(format!("{title} private batch overview")),
                tags: vec![format!("{title} private batch tag")],
                ..MetadataCandidateRecord::default()
            },
        };

    let apply_subject = make_subject("batch-apply", "Batch Apply");
    let apply_related_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "batch-apply/1".to_owned(),
        title: Some("Batch Apply Episode".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let apply_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-apply".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(Some(apply_subject.clone()), "Batch Apply"),
                related: vec![MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Episode,
                    subject: Some(apply_related_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Batch Apply Episode".to_owned()),
                        overview: Some("related private batch overview".to_owned()),
                        tags: vec!["related-private-batch-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                }],
                relationships: vec![MetadataCandidateReviewRelationship {
                    parent_subject: apply_subject,
                    child_subject: apply_related_subject,
                    kind: MetadataCandidateRelationshipKind::Contains,
                }],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let apply_review = store
        .set_metadata_candidate_review_status(
            apply_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap()
        .unwrap();

    let noop_subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "batch-noop".to_owned(),
        title: Some("Batch Noop".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    store.upsert_provider_subject(&noop_subject).await.unwrap();
    store
        .upsert_provider_mapping(&ProviderMapping {
            id: ProviderMappingId::new(),
            item_id: item.id,
            subject_id: noop_subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: None,
            source: MetadataSource::Provider(ExternalProvider::Bangumi),
        })
        .await
        .unwrap();
    let noop_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-noop".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(
                    Some(make_subject("batch-noop", "Batch Noop")),
                    "Batch Noop",
                ),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 110,
            updated_at_ms: 210,
        })
        .await
        .unwrap();
    let noop_review = store
        .set_metadata_candidate_review_status(
            noop_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            310,
        )
        .await
        .unwrap()
        .unwrap();

    let pending_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-pending".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(
                    Some(make_subject("batch-pending", "Batch Pending")),
                    "Batch Pending",
                ),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 120,
            updated_at_ms: 220,
        })
        .await
        .unwrap();

    let blocked_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-blocked".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(None, "Batch Blocked"),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 130,
            updated_at_ms: 230,
        })
        .await
        .unwrap();
    let blocked_review = store
        .set_metadata_candidate_review_status(
            blocked_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            330,
        )
        .await
        .unwrap()
        .unwrap();

    let stale_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-stale".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(
                    Some(make_subject("batch-stale", "Batch Stale")),
                    "Batch Stale",
                ),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 140,
            updated_at_ms: 240,
        })
        .await
        .unwrap();
    let stale_review = store
        .set_metadata_candidate_review_status(
            stale_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            340,
        )
        .await
        .unwrap()
        .unwrap();

    let conflict_review = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:batch-conflict".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: make_review_node(
                    Some(make_subject("batch-conflict", "Batch Conflict")),
                    "Batch Conflict",
                ),
                related: vec![],
                relationships: vec![],
            },
            expires_at_ms: None,
            created_at_ms: 150,
            updated_at_ms: 250,
        })
        .await
        .unwrap();
    let conflict_review = store
        .set_metadata_candidate_review_status(
            conflict_review.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            350,
        )
        .await
        .unwrap()
        .unwrap();
    let missing_review_id = MetadataCandidateReviewId::new();

    let router = build_router(app);
    let path = "/admin/v1/metadata/candidate-reviews/batch-apply";
    let request = AdminMetadataCandidateReviewBatchApplyRequest {
        idempotency_key: "candidate-review:batch-secret".to_owned(),
        reviews: vec![
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: apply_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(apply_review.updated_at_ms),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: noop_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(noop_review.updated_at_ms),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: pending_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(pending_review.updated_at_ms),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: blocked_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(blocked_review.updated_at_ms),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: stale_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(stale_review.updated_at_ms - 1),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: conflict_review.id,
                item_id: MediaItemId::new(),
                expected_updated_at_ms: Some(conflict_review.updated_at_ms),
            },
            AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: missing_review_id,
                item_id: item.id,
                expected_updated_at_ms: None,
            },
        ],
    };

    let response = response_body_json(&router, Method::POST, path, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let batch: AdminMetadataCandidateReviewBatchApplyResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(batch.summary.requested_count, 7);
    assert_eq!(batch.summary.returned_count, 7);
    assert_eq!(batch.summary.max_review_count, 50);
    assert_eq!(batch.summary.applied_count, 1);
    assert_eq!(batch.summary.changed_count, 1);
    assert_eq!(batch.summary.noop_count, 1);
    assert_eq!(batch.summary.replay_count, 1);
    assert_eq!(batch.summary.skipped_count, 1);
    assert_eq!(batch.summary.blocked_count, 1);
    assert_eq!(batch.summary.stale_count, 1);
    assert_eq!(batch.summary.conflict_count, 1);
    assert_eq!(batch.summary.failed_count, 1);
    assert_eq!(
        batch.results[0].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Applied
    );
    assert_eq!(
        batch.results[1].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Noop
    );
    assert_eq!(
        batch.results[2].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Skipped
    );
    assert_eq!(
        batch.results[3].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Blocked
    );
    assert_eq!(
        batch.results[4].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Stale
    );
    assert_eq!(
        batch.results[5].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Conflict
    );
    assert_eq!(
        batch.results[6].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Failed
    );
    assert!(batch.results[0].provider_mapping.is_some());
    assert!(batch.results[0].governance.is_some());
    assert!(batch.results[1].idempotent_replay);
    assert!(batch.results[1].governance.is_some());
    assert!(batch.results[2].plan.is_some());
    assert!(batch.results[2].governance.is_some());
    assert!(batch.results[3].plan.is_some());
    assert!(batch.results[3].governance.is_some());
    assert_eq!(
        batch.results[0]
            .governance
            .as_ref()
            .unwrap()
            .audit_timeline
            .events
            .last()
            .unwrap()
            .kind,
        AdminMetadataCandidateReviewAuditEventKind::ApplicationResult
    );
    assert_eq!(
        batch.results[0]
            .governance
            .as_ref()
            .unwrap()
            .audit_timeline
            .events
            .last()
            .unwrap()
            .changed,
        Some(true)
    );
    assert_eq!(
        batch.results[1].governance.as_ref().unwrap().undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::ManualRootProviderMappingReview
    );
    assert_eq!(batch.results[4].governance, None);
    assert_eq!(batch.results[5].governance, None);
    assert_eq!(batch.results[6].governance, None);
    assert!(
        batch.results[4]
            .error
            .as_ref()
            .is_some_and(|error| error.code == "conflict")
    );
    assert!(
        batch.results[6]
            .error
            .as_ref()
            .is_some_and(|error| error.code == "not_found")
    );
    assert!(!batch.idempotency_key_fingerprint.is_empty());

    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(provider_mappings.len(), 2);
    assert!(
        provider_mappings
            .iter()
            .all(|mapping| mapping.status == ProviderMappingStatus::Accepted)
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Episode,
                "batch-apply/1"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!body.contains("candidate-review:batch-secret"));
    assert!(!body.contains("private batch overview"));
    assert!(!body.contains("private batch tag"));
    assert!(!body.contains("related private batch overview"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("sha256-private-batch-apply"));
    assert!(!body.contains(&temp.path().display().to_string()));

    let replay = response_body_json(&router, Method::POST, path, &request).await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminMetadataCandidateReviewBatchApplyResponse =
        serde_json::from_str(&replay_body).unwrap();
    assert_eq!(replayed.summary.applied_count, 0);
    assert_eq!(replayed.summary.changed_count, 0);
    assert_eq!(replayed.summary.noop_count, 2);
    assert_eq!(replayed.summary.replay_count, 2);
    assert_eq!(
        replayed.results[0].status,
        AdminMetadataCandidateReviewBatchApplyResultStatus::Noop
    );
    assert_eq!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        2
    );
    assert!(!replay_body.contains("candidate-review:batch-secret"));
    assert!(!replay_body.contains("private batch overview"));
    assert!(!replay_body.contains("local:///"));

    let duplicate = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchApplyRequest {
            idempotency_key: "candidate-review:duplicate".to_owned(),
            reviews: vec![
                AdminMetadataCandidateReviewBatchApplyItemRequest {
                    review_id: apply_review.id,
                    item_id: item.id,
                    expected_updated_at_ms: Some(apply_review.updated_at_ms),
                },
                AdminMetadataCandidateReviewBatchApplyItemRequest {
                    review_id: apply_review.id,
                    item_id: item.id,
                    expected_updated_at_ms: Some(apply_review.updated_at_ms),
                },
            ],
        },
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);
    let duplicate_body = response_text(duplicate).await;
    let duplicate_error: ErrorResponse = serde_json::from_str(&duplicate_body).unwrap();
    assert_eq!(duplicate_error.code, "invalid_input");

    let empty_key = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchApplyRequest {
            idempotency_key: "  ".to_owned(),
            reviews: vec![AdminMetadataCandidateReviewBatchApplyItemRequest {
                review_id: noop_review.id,
                item_id: item.id,
                expected_updated_at_ms: Some(noop_review.updated_at_ms),
            }],
        },
    )
    .await;
    assert_eq!(empty_key.status(), StatusCode::BAD_REQUEST);
    let empty_key_body = response_text(empty_key).await;
    let empty_key_error: ErrorResponse = serde_json::from_str(&empty_key_body).unwrap();
    assert_eq!(empty_key_error.code, "invalid_input");

    let too_many_review_ids = (0..=batch.summary.max_review_count)
        .map(|_| AdminMetadataCandidateReviewBatchApplyItemRequest {
            review_id: MetadataCandidateReviewId::new(),
            item_id: item.id,
            expected_updated_at_ms: None,
        })
        .collect();
    let too_many = response_body_json(
        &router,
        Method::POST,
        path,
        &AdminMetadataCandidateReviewBatchApplyRequest {
            idempotency_key: "candidate-review:too-many".to_owned(),
            reviews: too_many_review_ids,
        },
    )
    .await;
    assert_eq!(too_many.status(), StatusCode::BAD_REQUEST);
    let too_many_body = response_text(too_many).await;
    let too_many_error: ErrorResponse = serde_json::from_str(&too_many_body).unwrap();
    assert_eq!(too_many_error.code, "invalid_input");
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_detail_is_redacted_and_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Depth Candidate".to_owned(),
            release_date: Some("2026-05-30".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Private/Depth.Candidate.S01E01.mkv?token=secret".to_owned(),
        file_name: "Depth.Candidate.S01E01.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-candidate-review".to_owned()),
    };
    let root_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "1437".to_owned(),
        title: Some("Depth Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let related_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "1437/1".to_owned(),
        title: Some("Episode One".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let review_id = MetadataCandidateReviewId::new();
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let inserted = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: review_id,
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:1437".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(root_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Depth Candidate".to_owned()),
                        overview: Some(
                            "raw provider payload should not leak secret-overview".to_owned(),
                        ),
                        release_date: Some("2026-05-30".to_owned()),
                        runtime_minutes: Some(24),
                        tags: vec!["secret-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Episode,
                    subject: Some(related_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Episode One".to_owned()),
                        overview: Some("secret-related-overview".to_owned()),
                        release_date: Some("2026-06-01".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                }],
                relationships: vec![MetadataCandidateReviewRelationship {
                    parent_subject: root_subject,
                    child_subject: related_subject,
                    kind: MetadataCandidateRelationshipKind::Contains,
                }],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    store
        .set_metadata_candidate_review_status(
            inserted.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap();

    let router = build_router(app);
    let path = format!("/admin/v1/metadata/candidate-reviews/{review_id}");
    let response = router
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
    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let detail: AdminMetadataCandidateReviewResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(detail.review.review_id, review_id);
    assert_eq!(detail.review.item_id, item.id);
    assert_eq!(
        detail.review.status,
        DurableMetadataCandidateReviewStatus::Accepted
    );
    assert_eq!(
        detail.review.root.subject.as_ref().unwrap().subject_key,
        "1437"
    );
    assert_eq!(detail.review.related.len(), 1);
    assert_eq!(
        detail.review.related[0]
            .subject
            .as_ref()
            .unwrap()
            .subject_key,
        "1437/1"
    );
    assert_eq!(detail.review.relationship_count, 1);
    assert_eq!(
        detail.application_plan.action,
        AdminMetadataCandidateReviewApplicationAction::Apply
    );
    assert_eq!(
        detail.application_plan.reasons,
        vec![AdminMetadataCandidateReviewApplicationReason::Ready]
    );
    assert!(detail.boundary.read_only);
    assert!(!detail.boundary.applies_on_read);
    assert!(detail.boundary.apply_mutation_required);
    assert!(detail.boundary.apply_updates_root_provider_subject);
    assert!(detail.boundary.apply_updates_root_provider_mapping);
    assert!(!detail.boundary.apply_updates_related_provider_subjects);
    assert!(!detail.boundary.apply_updates_related_provider_mappings);
    assert!(!detail.boundary.updates_canonical_metadata);
    assert!(!detail.boundary.updates_hierarchy);
    assert!(!detail.boundary.writes_nfo);
    assert!(!detail.boundary.writes_library_files);
    assert!(detail.governance.audit_timeline.read_only);
    assert!(detail.governance.audit_timeline.replay_safe);
    assert_eq!(
        detail.governance.audit_timeline.events[0].kind,
        AdminMetadataCandidateReviewAuditEventKind::ReviewCreated
    );
    assert_eq!(
        detail.governance.audit_timeline.events[1].kind,
        AdminMetadataCandidateReviewAuditEventKind::ReviewStatusCurrent
    );
    assert_eq!(
        detail.governance.audit_timeline.events[1].status,
        Some(DurableMetadataCandidateReviewStatus::Accepted)
    );
    assert_eq!(
        detail.governance.undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::DeferredUntilApplyOutcomeAudit
    );
    assert_eq!(
        detail.governance.undo_plan.stale_state_guard_updated_at_ms,
        Some(300)
    );
    assert!(
        detail
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::ApplyOutcomeAuditRequired)
    );
    assert!(
        detail
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::StaleStateGuardRequired)
    );
    assert!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "1437"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Episode,
                "1437/1"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!body.contains("overview"));
    assert!(!body.contains("secret-overview"));
    assert!(!body.contains("secret-related-overview"));
    assert!(!body.contains("secret-tag"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("sha256-private-candidate-review"));
    assert!(!body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_apply_commits_root_mapping_and_replays() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Apply Candidate".to_owned(),
            release_date: Some("2026-05-30".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Private/Apply.Candidate.S01E01.mkv?token=secret".to_owned(),
        file_name: "Apply.Candidate.S01E01.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-candidate-apply".to_owned()),
    };
    let root_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "1437".to_owned(),
        title: Some("Apply Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let related_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "1437/1".to_owned(),
        title: Some("Episode One".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let review_id = MetadataCandidateReviewId::new();
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let inserted = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: review_id,
            item_id: item.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:1437".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(root_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Apply Candidate".to_owned()),
                        overview: Some("secret apply overview".to_owned()),
                        release_date: Some("2026-05-30".to_owned()),
                        tags: vec!["secret-apply-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Episode,
                    subject: Some(related_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Episode One".to_owned()),
                        overview: Some("secret related apply overview".to_owned()),
                        release_date: Some("2026-06-01".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                }],
                relationships: vec![MetadataCandidateReviewRelationship {
                    parent_subject: root_subject,
                    child_subject: related_subject,
                    kind: MetadataCandidateRelationshipKind::Contains,
                }],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let accepted = store
        .set_metadata_candidate_review_status(
            inserted.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap()
        .unwrap();

    let router = build_router(app);
    let path = format!("/admin/v1/metadata/candidate-reviews/{review_id}/apply");
    let request = AdminMetadataCandidateReviewApplyRequest {
        item_id: item.id,
        expected_updated_at_ms: Some(accepted.updated_at_ms),
        idempotency_key: "candidate-review:operator-confirmation:secret".to_owned(),
    };
    let stale_response = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminMetadataCandidateReviewApplyRequest {
            item_id: item.id,
            expected_updated_at_ms: Some(accepted.updated_at_ms - 1),
            idempotency_key: "candidate-review:stale-secret".to_owned(),
        },
    )
    .await;
    assert_eq!(stale_response.status(), StatusCode::CONFLICT);
    let stale_body = response_text(stale_response).await;
    let stale_error: ErrorResponse = serde_json::from_str(&stale_body).unwrap();
    assert_eq!(stale_error.code, "conflict");
    assert!(!stale_body.contains("candidate-review:stale-secret"));
    assert!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );

    let empty_key_response = response_body_json(
        &router,
        Method::POST,
        &path,
        &AdminMetadataCandidateReviewApplyRequest {
            item_id: item.id,
            expected_updated_at_ms: Some(accepted.updated_at_ms),
            idempotency_key: "  ".to_owned(),
        },
    )
    .await;
    assert_eq!(empty_key_response.status(), StatusCode::BAD_REQUEST);
    let empty_key_body = response_text(empty_key_response).await;
    let empty_key_error: ErrorResponse = serde_json::from_str(&empty_key_body).unwrap();
    assert_eq!(empty_key_error.code, "invalid_input");

    let response = response_body_json(&router, Method::POST, &path, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let applied: AdminMetadataCandidateReviewApplyResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(applied.review_id, review_id);
    assert_eq!(applied.item_id, item.id);
    assert!(applied.applied);
    assert!(applied.changed);
    assert!(!applied.idempotent_replay);
    assert!(!applied.idempotency_key_fingerprint.is_empty());
    assert_eq!(
        applied.plan.action,
        AdminMetadataCandidateReviewApplicationAction::Apply
    );
    assert_eq!(
        applied.provider_subject.as_ref().unwrap().subject_key,
        "1437"
    );
    assert_eq!(
        applied.provider_mapping.as_ref().unwrap().status,
        ProviderMappingStatus::Accepted
    );
    assert!(applied.boundary.apply_updates_root_provider_subject);
    assert!(applied.boundary.apply_updates_root_provider_mapping);
    assert!(!applied.boundary.apply_updates_related_provider_subjects);
    assert!(!applied.boundary.updates_hierarchy);
    assert!(applied.governance.audit_timeline.read_only);
    assert_eq!(
        applied
            .governance
            .audit_timeline
            .events
            .last()
            .unwrap()
            .kind,
        AdminMetadataCandidateReviewAuditEventKind::ApplicationResult
    );
    assert_eq!(
        applied
            .governance
            .audit_timeline
            .events
            .last()
            .unwrap()
            .changed,
        Some(true)
    );
    assert_eq!(
        applied
            .governance
            .audit_timeline
            .events
            .last()
            .unwrap()
            .idempotent_replay,
        Some(false)
    );
    assert_eq!(
        applied.governance.undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::ManualRootProviderMappingReview
    );
    assert_eq!(
        applied.governance.undo_plan.target_mapping_status,
        Some(ProviderMappingStatus::Accepted)
    );
    assert_eq!(
        applied.governance.undo_plan.stale_state_guard_updated_at_ms,
        Some(300)
    );
    assert!(
        applied
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::MissingPersistedPreApplySnapshot)
    );
    assert!(
        applied
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::ProviderMappingMayPreexistReview)
    );
    assert!(
        applied
            .governance
            .undo_plan
            .reasons
            .contains(&AdminMetadataCandidateReviewUndoReason::StaleStateGuardRequired)
    );

    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(provider_mappings.len(), 1);
    assert_eq!(provider_mappings[0].status, ProviderMappingStatus::Accepted);
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Episode,
                "1437/1"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!body.contains("candidate-review:operator-confirmation:secret"));
    assert!(!body.contains("secret apply overview"));
    assert!(!body.contains("secret related apply overview"));
    assert!(!body.contains("secret-apply-tag"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("sha256-private-candidate-apply"));
    assert!(!body.contains(&temp.path().display().to_string()));

    let replay = response_body_json(&router, Method::POST, &path, &request).await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminMetadataCandidateReviewApplyResponse =
        serde_json::from_str(&replay_body).unwrap();

    assert!(replayed.applied);
    assert!(!replayed.changed);
    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.provider_mapping, applied.provider_mapping);
    assert_eq!(
        replayed.plan.action,
        AdminMetadataCandidateReviewApplicationAction::Noop
    );
    assert_eq!(
        replayed
            .governance
            .audit_timeline
            .events
            .last()
            .unwrap()
            .kind,
        AdminMetadataCandidateReviewAuditEventKind::ApplicationResult
    );
    assert_eq!(
        replayed
            .governance
            .audit_timeline
            .events
            .last()
            .unwrap()
            .changed,
        Some(false)
    );
    assert_eq!(
        replayed
            .governance
            .audit_timeline
            .events
            .last()
            .unwrap()
            .idempotent_replay,
        Some(true)
    );
    assert_eq!(
        replayed.governance.undo_plan.mode,
        AdminMetadataCandidateReviewUndoMode::ManualRootProviderMappingReview
    );
    assert_eq!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(!replay_body.contains("candidate-review:operator-confirmation:secret"));
    assert!(!replay_body.contains("secret apply overview"));
    assert!(!replay_body.contains("secret related apply overview"));
    assert!(!replay_body.contains("local:///"));
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_related_hierarchy_plan_apply_and_replay_are_redacted() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "TV".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Tv,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let root = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Related Candidate".to_owned(),
            release_date: Some("2026-05-30".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let child = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Season,
        parent_id: Some(root.id),
        metadata: CanonicalMetadata {
            title: "Season 1".to_owned(),
            release_date: None,
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: root.id,
        locator: "local:///Private/Related.Candidate.S01E01.mkv?token=secret".to_owned(),
        file_name: "Related.Candidate.S01E01.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256-private-related-hierarchy".to_owned()),
    };
    let root_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "2437".to_owned(),
        title: Some("Related Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let related_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "2437/1".to_owned(),
        title: Some("Season 1".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let review_id = MetadataCandidateReviewId::new();
    store.upsert_media_item(&root).await.unwrap();
    store.upsert_media_item(&child).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: root.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: child.id,
            provisional: true,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let root_provider_subject = root_subject
        .clone()
        .into_provider_subject(ProviderSubjectId::new());
    store
        .upsert_provider_subject(&root_provider_subject)
        .await
        .unwrap();
    store
        .upsert_provider_mapping(&ProviderMapping {
            id: ProviderMappingId::new(),
            item_id: root.id,
            subject_id: root_provider_subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: None,
            source: MetadataSource::Provider(ExternalProvider::Bangumi),
        })
        .await
        .unwrap();
    let inserted = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: review_id,
            item_id: root.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:2437".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(root_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Related Candidate".to_owned()),
                        overview: Some("secret related root overview".to_owned()),
                        release_date: Some("2026-05-30".to_owned()),
                        tags: vec!["secret-related-root-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Season,
                    subject: Some(related_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Season 1".to_owned()),
                        overview: Some("secret related child overview".to_owned()),
                        release_date: Some("2026-06-01".to_owned()),
                        tags: vec!["secret-related-child-tag".to_owned()],
                        ..MetadataCandidateRecord::default()
                    },
                }],
                relationships: vec![MetadataCandidateReviewRelationship {
                    parent_subject: root_subject,
                    child_subject: related_subject,
                    kind: MetadataCandidateRelationshipKind::Contains,
                }],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    let accepted = store
        .set_metadata_candidate_review_status(
            inserted.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap()
        .unwrap();

    let router = build_router(app);
    let plan_path = format!(
        "/admin/v1/metadata/candidate-reviews/{review_id}/related-hierarchy/application-plan"
    );
    let plan_request = AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
        item_id: root.id,
        expected_updated_at_ms: Some(accepted.updated_at_ms),
    };
    let stale_plan = response_body_json(
        &router,
        Method::POST,
        &plan_path,
        &AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
            item_id: root.id,
            expected_updated_at_ms: Some(accepted.updated_at_ms - 1),
        },
    )
    .await;
    assert_eq!(stale_plan.status(), StatusCode::CONFLICT);

    let plan_response = response_body_json(&router, Method::POST, &plan_path, &plan_request).await;
    let plan_status = plan_response.status();
    let plan_body = response_text(plan_response).await;
    assert_eq!(plan_status, StatusCode::OK, "{plan_body}");
    let planned: AdminMetadataCandidateReviewRelatedHierarchyPlanResponse =
        serde_json::from_str(&plan_body).unwrap();

    assert_eq!(planned.review_id, review_id);
    assert_eq!(planned.item_id, root.id);
    assert_eq!(
        planned.plan.action,
        AdminMetadataCandidateReviewRelatedHierarchyApplicationAction::Apply
    );
    assert_eq!(planned.plan.target_count, 1);
    assert_eq!(planned.plan.mapping_change_count, 1);
    assert_eq!(planned.plan.provisional_state_change_count, 1);
    assert_eq!(planned.plan.targets[0].item_id, child.id);
    assert!(planned.plan.targets[0].mapping_change_required);
    assert!(planned.boundary.apply_mutation_required);
    assert!(!planned.boundary.apply_updates_root_provider_subject);
    assert!(!planned.boundary.apply_updates_root_provider_mapping);
    assert!(planned.boundary.apply_updates_related_provider_subjects);
    assert!(planned.boundary.apply_updates_related_provider_mappings);
    assert!(planned.boundary.apply_confirms_related_library_item_state);
    assert!(!planned.boundary.updates_parent_hierarchy);
    assert!(!planned.boundary.updates_canonical_metadata);
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Episode,
                "2437/1"
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(!plan_body.contains("secret related root overview"));
    assert!(!plan_body.contains("secret related child overview"));
    assert!(!plan_body.contains("secret-related-root-tag"));
    assert!(!plan_body.contains("secret-related-child-tag"));
    assert!(!plan_body.contains("local:///"));
    assert!(!plan_body.contains("token=secret"));
    assert!(!plan_body.contains("sha256-private-related-hierarchy"));
    assert!(!plan_body.contains(&temp.path().display().to_string()));

    let apply_path =
        format!("/admin/v1/metadata/candidate-reviews/{review_id}/related-hierarchy/apply");
    let apply_request = AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
        item_id: root.id,
        expected_updated_at_ms: Some(accepted.updated_at_ms),
        idempotency_key: "candidate-review:related-hierarchy:secret".to_owned(),
    };
    let empty_key = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
            item_id: root.id,
            expected_updated_at_ms: Some(accepted.updated_at_ms),
            idempotency_key: "  ".to_owned(),
        },
    )
    .await;
    assert_eq!(empty_key.status(), StatusCode::BAD_REQUEST);

    let apply_response =
        response_body_json(&router, Method::POST, &apply_path, &apply_request).await;
    let apply_status = apply_response.status();
    let apply_body = response_text(apply_response).await;
    assert_eq!(apply_status, StatusCode::OK, "{apply_body}");
    let applied: AdminMetadataCandidateReviewRelatedHierarchyApplyResponse =
        serde_json::from_str(&apply_body).unwrap();

    assert!(applied.applied);
    assert!(applied.changed);
    assert!(!applied.idempotent_replay);
    assert_eq!(
        applied.plan.action,
        AdminMetadataCandidateReviewRelatedHierarchyApplicationAction::Apply
    );
    assert_eq!(applied.provider_subjects.len(), 1);
    assert_eq!(applied.provider_subjects[0].subject_key, "2437/1");
    assert_eq!(applied.provider_mappings.len(), 1);
    assert_eq!(applied.provider_mappings[0].item_id, child.id);
    assert_eq!(
        applied.provider_mappings[0].status,
        ProviderMappingStatus::Accepted
    );
    assert_eq!(applied.confirmed_item_ids, vec![child.id]);
    assert!(
        !store
            .get_library_item_state(library_id, child.id)
            .await
            .unwrap()
            .unwrap()
            .provisional
    );
    let loaded_child = store.get_media_item(child.id).await.unwrap().unwrap();
    assert_eq!(loaded_child.parent_id, Some(root.id));
    assert_eq!(loaded_child.metadata.release_date, None);
    assert!(!apply_body.contains("candidate-review:related-hierarchy:secret"));
    assert!(!apply_body.contains("secret related child overview"));
    assert!(!apply_body.contains("local:///"));

    let replay = response_body_json(&router, Method::POST, &apply_path, &apply_request).await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminMetadataCandidateReviewRelatedHierarchyApplyResponse =
        serde_json::from_str(&replay_body).unwrap();

    assert!(replayed.applied);
    assert!(!replayed.changed);
    assert!(replayed.idempotent_replay);
    assert_eq!(
        replayed.plan.action,
        AdminMetadataCandidateReviewRelatedHierarchyApplicationAction::Noop
    );
    assert_eq!(replayed.plan.mapping_change_count, 0);
    assert_eq!(replayed.plan.provisional_state_change_count, 0);
    assert_eq!(
        store
            .list_provider_mappings_for_item(child.id, PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(!replay_body.contains("candidate-review:related-hierarchy:secret"));
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_related_hierarchy_plan_and_apply_reject_pending_and_missing_root_mapping()
 {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "TV".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Tv,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let root = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Related Reject Candidate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let child = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Season,
        parent_id: Some(root.id),
        metadata: CanonicalMetadata {
            title: "Season 1".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let root_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "related-reject-root".to_owned(),
        title: Some("Related Reject Candidate".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };
    let related_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "related-reject-root/1".to_owned(),
        title: Some("Season 1".to_owned()),
        release_year: Some(2026),
        locale: Some("zh-CN".to_owned()),
    };

    store.upsert_media_item(&root).await.unwrap();
    store.upsert_media_item(&child).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: root.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: child.id,
            provisional: true,
        })
        .await
        .unwrap();
    let inserted = store
        .upsert_metadata_candidate_review(NewMetadataCandidateReview {
            id: MetadataCandidateReviewId::new(),
            item_id: root.id,
            source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
            source_key: "bangumi:related-reject-root".to_owned(),
            plan: MetadataCandidateReviewPlan {
                root: MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Series,
                    subject: Some(root_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Related Reject Candidate".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                },
                related: vec![MetadataCandidateReviewNode {
                    source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                    kind: MediaKind::Season,
                    subject: Some(related_subject.clone()),
                    metadata: MetadataCandidateRecord {
                        title: Some("Season 1".to_owned()),
                        ..MetadataCandidateRecord::default()
                    },
                }],
                relationships: vec![MetadataCandidateReviewRelationship {
                    parent_subject: root_subject,
                    child_subject: related_subject,
                    kind: MetadataCandidateRelationshipKind::Contains,
                }],
            },
            expires_at_ms: None,
            created_at_ms: 100,
            updated_at_ms: 200,
        })
        .await
        .unwrap();

    let router = build_router(app);
    let plan_path = format!(
        "/admin/v1/metadata/candidate-reviews/{}/related-hierarchy/application-plan",
        inserted.id
    );
    let apply_path = format!(
        "/admin/v1/metadata/candidate-reviews/{}/related-hierarchy/apply",
        inserted.id
    );

    let pending_plan = response_body_json(
        &router,
        Method::POST,
        &plan_path,
        &AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
            item_id: root.id,
            expected_updated_at_ms: Some(inserted.updated_at_ms),
        },
    )
    .await;
    let pending_plan_status = pending_plan.status();
    let pending_plan_body = response_text(pending_plan).await;
    assert_eq!(pending_plan_status, StatusCode::CONFLICT);
    let pending_plan_error: ErrorResponse = serde_json::from_str(&pending_plan_body).unwrap();
    assert_eq!(pending_plan_error.code, "conflict");
    assert!(pending_plan_body.contains("before it is accepted"));

    let pending_apply = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
            item_id: root.id,
            expected_updated_at_ms: Some(inserted.updated_at_ms),
            idempotency_key: "candidate-review-related-reject-secret".to_owned(),
        },
    )
    .await;
    let pending_apply_status = pending_apply.status();
    let pending_apply_body = response_text(pending_apply).await;
    assert_eq!(pending_apply_status, StatusCode::CONFLICT);
    let pending_apply_error: ErrorResponse = serde_json::from_str(&pending_apply_body).unwrap();
    assert_eq!(pending_apply_error.code, "conflict");
    assert!(pending_apply_body.contains("before it is accepted"));
    assert!(!pending_apply_body.contains("candidate-review-related-reject-secret"));

    let accepted = store
        .set_metadata_candidate_review_status(
            inserted.id,
            DurableMetadataCandidateReviewStatus::Accepted,
            300,
        )
        .await
        .unwrap()
        .unwrap();

    let missing_root_plan = response_body_json(
        &router,
        Method::POST,
        &plan_path,
        &AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
            item_id: root.id,
            expected_updated_at_ms: Some(accepted.updated_at_ms),
        },
    )
    .await;
    let missing_root_plan_status = missing_root_plan.status();
    let missing_root_plan_body = response_text(missing_root_plan).await;
    assert_eq!(missing_root_plan_status, StatusCode::CONFLICT);
    let missing_root_plan_error: ErrorResponse =
        serde_json::from_str(&missing_root_plan_body).unwrap();
    assert_eq!(missing_root_plan_error.code, "conflict");
    assert!(missing_root_plan_body.contains("requires an accepted root provider mapping"));

    let missing_root_apply = response_body_json(
        &router,
        Method::POST,
        &apply_path,
        &AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
            item_id: root.id,
            expected_updated_at_ms: Some(accepted.updated_at_ms),
            idempotency_key: "candidate-review-related-missing-root-secret".to_owned(),
        },
    )
    .await;
    let missing_root_apply_status = missing_root_apply.status();
    let missing_root_apply_body = response_text(missing_root_apply).await;
    assert_eq!(missing_root_apply_status, StatusCode::CONFLICT);
    let missing_root_apply_error: ErrorResponse =
        serde_json::from_str(&missing_root_apply_body).unwrap();
    assert_eq!(missing_root_apply_error.code, "conflict");
    assert!(missing_root_apply_body.contains("requires an accepted root provider mapping"));
    assert!(!missing_root_apply_body.contains("candidate-review-related-missing-root-secret"));

    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Subject,
                "related-reject-root",
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .find_provider_subject(
                &ExternalProvider::Bangumi,
                &ProviderSubjectKind::Episode,
                "related-reject-root/1",
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .list_provider_mappings_for_item(root.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_provider_mappings_for_item(child.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .get_library_item_state(library_id, child.id)
            .await
            .unwrap()
            .unwrap()
            .provisional
    );
}

#[tokio::test]
async fn admin_v1_metadata_candidate_review_related_hierarchy_routes_reject_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "candidate-review-related-viewer".to_owned(),
            display_name: "Candidate Review Related Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "candidate-review-related-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    let review_id = MetadataCandidateReviewId::new();
    let item_id = MediaItemId::new();
    let plan_body = serde_json::to_vec(&AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
        item_id,
        expected_updated_at_ms: None,
    })
    .unwrap();
    let plan_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/metadata/candidate-reviews/{review_id}/related-hierarchy/application-plan"
                ))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(plan_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(plan_response.status(), StatusCode::FORBIDDEN);
    let plan_error = body_json::<ErrorResponse>(plan_response).await;
    assert_eq!(
        plan_error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(plan_error.message, "administrator role is required");

    let apply_body =
        serde_json::to_vec(&AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
            item_id,
            expected_updated_at_ms: None,
            idempotency_key: "candidate-review-related-secret".to_owned(),
        })
        .unwrap();
    let apply_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/metadata/candidate-reviews/{review_id}/related-hierarchy/apply"
                ))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(apply_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(apply_response.status(), StatusCode::FORBIDDEN);
    let apply_error = body_json::<ErrorResponse>(apply_response).await;
    assert_eq!(
        apply_error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(apply_error.message, "administrator role is required");
}

#[tokio::test]
async fn admin_v1_generated_artifact_proposals_are_admin_only_redacted_and_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
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
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning"}"#.to_owned(),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/automation/generated-artifacts/proposals?limit=5")
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
    let diagnostics: AdminGeneratedArtifactProposalListResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        diagnostics.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(diagnostics.proposals.len(), 1);
    assert_eq!(
        diagnostics.proposals[0].capability,
        AutomationCapability::MetadataCleanup
    );
    assert_eq!(diagnostics.proposals[0].target.source_id, Some(source.id));
    assert_eq!(diagnostics.proposals[0].payload.confidence_milli, Some(810));
    assert!(
        diagnostics.proposals[0]
            .payload
            .payload_fingerprint
            .starts_with("sha256:")
    );
    assert_eq!(
        diagnostics.proposals[0].readiness.status,
        nako_core::GeneratedArtifactReadinessStatus::Ready
    );
    assert!(diagnostics.proposals[0].readiness.actionable);
    assert_eq!(diagnostics.page.limit, 5);
    assert_eq!(diagnostics.page.returned, 1);
    let item_after = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    assert!(!body.contains("prompt_json"));
    assert!(!body.contains("artifact_json"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn admin_v1_generated_artifact_review_accepts_without_autonomous_metadata_writes() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
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
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning"}"#.to_owned(),
        })
        .await
        .unwrap();
    let router = build_router(app);
    let request = AdminGeneratedArtifactReviewRequest {
        decision: GeneratedArtifactReviewDecision::Accept,
    };

    let plan_body = serde_json::to_vec(&request).unwrap();
    let plan_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/automation/generated-artifacts/{}/review-plan",
                    artifact.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(plan_body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = plan_response.status();
    let plan_body = String::from_utf8(
        to_bytes(plan_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{plan_body}");
    let plan: AdminGeneratedArtifactReviewPlanResponse = serde_json::from_str(&plan_body).unwrap();
    assert_eq!(
        plan.plan.action,
        nako_core::GeneratedArtifactAcceptanceActionKind::StageMetadataAuthorityReview
    );
    assert!(plan.plan.boundary.requires_metadata_authority_apply);
    assert!(!plan.plan.boundary.accepted_into_canonical_metadata);
    assert!(!plan.plan.boundary.applies_immediately);

    let review_body = serde_json::to_vec(&request).unwrap();
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/automation/generated-artifacts/{}/review",
                    artifact.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(review_body))
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
    let review: AdminGeneratedArtifactReviewResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(review.artifact_status, AutomationArtifactStatus::Accepted);
    assert!(!review.idempotent_replay);
    assert!(!review.plan.boundary.accepted_into_canonical_metadata);
    assert!(!review.plan.boundary.writes_sidecar);
    assert!(!review.plan.boundary.writes_library_files);
    assert!(!review.plan.boundary.applies_immediately);
    let item_after = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    assert!(!body.contains("prompt_json"));
    assert!(!body.contains("artifact_json"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn admin_v1_generated_artifact_metadata_apply_plan_is_redacted_and_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
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
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning","provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#.to_owned(),
        })
        .await
        .unwrap();
    app.automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();
    let router = build_router(app);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/automation/generated-artifacts/{}/metadata-apply-plan",
                    artifact.id
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
    let plan: AdminGeneratedArtifactMetadataApplyPlanResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(
        plan.plan.status,
        nako_core::GeneratedArtifactMetadataApplyPlanStatus::Ready
    );
    assert!(plan.plan.executable);
    assert_eq!(plan.plan.apply_field_count, 1);
    assert_eq!(plan.plan.apply_provider_mapping_count, 1);
    assert_eq!(plan.plan.fields[0].field, MetadataField::Overview);
    assert_eq!(
        plan.plan.fields[0].action,
        nako_core::GeneratedArtifactMetadataFieldAction::Apply
    );
    assert_eq!(
        plan.plan.provider_mappings[0].action,
        nako_core::GeneratedArtifactProviderMappingAction::Apply
    );
    let item_after = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    assert!(!body.contains("prompt_json"));
    assert!(!body.contains("artifact_json"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn admin_v1_generated_artifact_metadata_apply_plan_bulk_is_redacted_read_only_and_bounded() {
    let fixture = admin_generated_artifact_metadata_apply_http_fixture().await;
    let missing_artifact_id = AutomationArtifactId::new();
    let uri = "/admin/v1/automation/generated-artifacts/metadata-apply-plan";
    let request = AdminGeneratedArtifactMetadataBulkApplyPlanRequest {
        artifact_ids: vec![
            fixture.artifact_id,
            fixture.artifact_id,
            missing_artifact_id,
        ],
    };

    let response = response_body_json(&fixture.router, Method::POST, uri, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let plan: AdminGeneratedArtifactMetadataBulkApplyPlanResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(plan.plan.selection.requested_artifact_count, 3);
    assert_eq!(plan.plan.selection.selected_artifact_count, 2);
    assert_eq!(plan.plan.selection.duplicate_artifact_count, 1);
    assert_eq!(plan.plan.summary.planned_artifact_count, 1);
    assert_eq!(plan.plan.summary.missing_artifact_count, 1);
    assert_eq!(plan.plan.summary.ready_artifact_count, 1);
    assert_eq!(plan.plan.summary.executable_artifact_count, 1);
    assert_eq!(plan.plan.summary.apply_field_count, 1);
    assert_eq!(plan.plan.summary.apply_provider_mapping_count, 1);
    assert_eq!(plan.plan.summary.skipped_provider_mapping_count, 0);
    assert_eq!(plan.plan.summary.noop_provider_mapping_count, 0);
    assert_eq!(plan.plan.items.len(), 2);
    assert_eq!(
        plan.plan.items[0].status,
        nako_core::GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned
    );
    assert!(plan.plan.items[0].plan.is_some());
    assert_eq!(
        plan.plan
            .items
            .first()
            .and_then(|item| item.plan.as_ref())
            .map(|item_plan| item_plan.apply_provider_mapping_count),
        Some(1)
    );
    assert_eq!(
        plan.plan.items[1].status,
        nako_core::GeneratedArtifactMetadataBulkApplyPlanItemStatus::Missing
    );
    assert!(plan.plan.items[1].plan.is_none());

    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    assert!(!body.contains("prompt_json"));
    assert!(!body.contains("artifact_json"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));

    let too_many = AdminGeneratedArtifactMetadataBulkApplyPlanRequest {
        artifact_ids: (0..=nako_core::GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS)
            .map(|_| AutomationArtifactId::new())
            .collect(),
    };
    let rejected = response_body_json(&fixture.router, Method::POST, uri, &too_many).await;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    let rejected_body = response_text(rejected).await;
    let error: ErrorResponse = serde_json::from_str(&rejected_body).unwrap();
    assert_eq!(error.code, "invalid_input");
    assert!(error.message.contains("at most"));
    assert!(!rejected_body.contains("prompt_json"));
    assert!(!rejected_body.contains("artifact_json"));
    assert!(!rejected_body.contains("local:///Movies/private"));
    assert!(!rejected_body.contains("private generated overview"));
    assert!(!rejected_body.contains("secret"));
}

#[tokio::test]
async fn admin_generated_artifact_bulk_metadata_apply_v1_confirms_replays_and_reports_status() {
    let fixture = admin_generated_artifact_metadata_apply_http_fixture().await;
    let missing_artifact_id = AutomationArtifactId::new();
    let uri = "/admin/v1/automation/generated-artifacts/metadata-apply-batches";
    let request = AdminGeneratedArtifactMetadataBulkApplyRequest {
        artifact_ids: vec![fixture.artifact_id, missing_artifact_id],
        idempotency_key: "bulk-metadata-apply:operator-confirmation".to_owned(),
    };

    let response = response_body_json(&fixture.router, Method::POST, uri, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let queued: AdminGeneratedArtifactMetadataBulkApplyBatchResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(
        queued.batch.status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchStatus::Queued
    );
    assert_eq!(queued.batch.selection.requested_artifact_count, 2);
    assert_eq!(queued.batch.summary.executable_artifact_count, 1);
    assert_eq!(queued.batch.summary.missing_artifact_count, 1);
    assert_eq!(queued.batch.summary.apply_provider_mapping_count, 1);
    assert_eq!(queued.batch.summary.skipped_provider_mapping_count, 0);
    assert_eq!(queued.batch.summary.noop_provider_mapping_count, 0);
    assert_eq!(queued.batch.execution_summary.total_item_count, 2);
    assert_eq!(queued.batch.execution_summary.pending_item_count, 1);
    assert_eq!(queued.batch.execution_summary.skipped_item_count, 1);
    assert_eq!(queued.batch.items.len(), 2);
    assert_eq!(
        queued.batch.items[0].status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending
    );
    assert_eq!(
        queued.batch.items[1].status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped
    );
    assert!(queued.batch.items[0].outcome_id.is_none());
    assert!(queued.batch.items[0].plan_item.plan.is_some());
    assert!(queued.batch.items[1].plan_item.plan.is_none());
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&body);

    let replay = response_body_json(&fixture.router, Method::POST, uri, &request).await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminGeneratedArtifactMetadataBulkApplyBatchResponse =
        serde_json::from_str(&replay_body).unwrap();
    assert_eq!(replayed.batch.id, queued.batch.id);
    assert_eq!(replayed.batch.job_id, queued.batch.job_id);
    assert_eq!(replayed.batch.status, queued.batch.status);
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&replay_body);

    let status_uri = format!(
        "/admin/v1/automation/generated-artifacts/metadata-apply-batches/{}",
        queued.batch.id
    );
    let status_response = fixture
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&status_uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status_code = status_response.status();
    let status_body = response_text(status_response).await;
    assert_eq!(status_code, StatusCode::OK, "{status_body}");
    let status_batch: AdminGeneratedArtifactMetadataBulkApplyBatchResponse =
        serde_json::from_str(&status_body).unwrap();
    assert_eq!(status_batch.batch.id, queued.batch.id);
    assert_eq!(
        status_batch.batch.status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchStatus::Queued
    );
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&status_body);

    let executed = fixture
        .app
        .automation()
        .execute_generated_artifact_metadata_bulk_apply_batch(queued.batch.id)
        .await
        .unwrap();
    assert_eq!(
        executed.status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchStatus::Completed
    );

    let result_response = fixture
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(status_uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let result_status = result_response.status();
    let result_body = response_text(result_response).await;
    assert_eq!(result_status, StatusCode::OK, "{result_body}");
    let result: AdminGeneratedArtifactMetadataBulkApplyBatchResponse =
        serde_json::from_str(&result_body).unwrap();

    assert_eq!(result.batch.id, queued.batch.id);
    assert_eq!(
        result.batch.status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchStatus::Completed
    );
    assert_eq!(result.batch.execution_summary.pending_item_count, 0);
    assert_eq!(result.batch.execution_summary.applied_item_count, 1);
    assert_eq!(result.batch.execution_summary.skipped_item_count, 1);
    assert_eq!(result.batch.summary.apply_provider_mapping_count, 1);
    assert_eq!(result.batch.summary.skipped_provider_mapping_count, 0);
    assert_eq!(result.batch.summary.noop_provider_mapping_count, 0);
    assert_eq!(
        result.batch.items[0].status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied
    );
    assert!(result.batch.items[0].outcome_id.is_some());
    assert_eq!(
        result.batch.items[1].status,
        nako_core::GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped
    );
    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        item_after.metadata.overview.as_deref(),
        Some("private generated overview")
    );
    let mappings = fixture
        .store
        .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(
        mappings[0].status,
        nako_core::ProviderMappingStatus::Accepted
    );
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&result_body);
}

#[tokio::test]
async fn admin_generated_artifact_bulk_metadata_apply_v1_maps_errors_without_sensitive_body() {
    let fixture = admin_generated_artifact_metadata_apply_http_fixture().await;
    let uri = "/admin/v1/automation/generated-artifacts/metadata-apply-batches";

    let empty_key = response_body_json(
        &fixture.router,
        Method::POST,
        uri,
        &AdminGeneratedArtifactMetadataBulkApplyRequest {
            artifact_ids: vec![fixture.artifact_id],
            idempotency_key: "  ".to_owned(),
        },
    )
    .await;
    assert_eq!(empty_key.status(), StatusCode::BAD_REQUEST);
    let empty_key_body = response_text(empty_key).await;
    let empty_key_error: ErrorResponse = serde_json::from_str(&empty_key_body).unwrap();
    assert_eq!(empty_key_error.code, "invalid_input");
    assert!(empty_key_error.message.contains("idempotency_key"));
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&empty_key_body);

    let no_executable = response_body_json(
        &fixture.router,
        Method::POST,
        uri,
        &AdminGeneratedArtifactMetadataBulkApplyRequest {
            artifact_ids: vec![AutomationArtifactId::new()],
            idempotency_key: "bulk-metadata-apply:no-executable".to_owned(),
        },
    )
    .await;
    assert_eq!(no_executable.status(), StatusCode::BAD_REQUEST);
    let no_executable_body = response_text(no_executable).await;
    let no_executable_error: ErrorResponse = serde_json::from_str(&no_executable_body).unwrap();
    assert_eq!(no_executable_error.code, "invalid_input");
    assert!(no_executable_error.message.contains("executable"));
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&no_executable_body);

    let missing_batch_id = GeneratedArtifactMetadataBulkApplyBatchId::new();
    let missing = fixture
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/automation/generated-artifacts/metadata-apply-batches/{missing_batch_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    let missing_body = response_text(missing).await;
    let missing_error: ErrorResponse = serde_json::from_str(&missing_body).unwrap();
    assert_eq!(missing_error.code, "not_found");
    assert_generated_artifact_bulk_metadata_apply_body_redacted(&missing_body);
}

#[tokio::test]
async fn admin_generated_artifact_bulk_metadata_apply_v1_requires_admin_auth() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "bulk-metadata-apply-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;
    let post_uri = "/admin/v1/automation/generated-artifacts/metadata-apply-batches";
    let request = AdminGeneratedArtifactMetadataBulkApplyRequest {
        artifact_ids: vec![AutomationArtifactId::new()],
        idempotency_key: "bulk-metadata-apply:auth".to_owned(),
    };

    let missing_post = response_body_json(&router, Method::POST, post_uri, &request).await;
    assert_eq!(missing_post.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(missing_post.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let missing_post_error = body_json::<ErrorResponse>(missing_post).await;
    assert_eq!(missing_post_error.code, "unauthorized");

    let authorized_post = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(post_uri)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized_post.status(), StatusCode::BAD_REQUEST);
    let authorized_post_error = body_json::<ErrorResponse>(authorized_post).await;
    assert_eq!(authorized_post_error.code, "invalid_input");

    let batch_id = GeneratedArtifactMetadataBulkApplyBatchId::new();
    let get_uri =
        format!("/admin/v1/automation/generated-artifacts/metadata-apply-batches/{batch_id}");
    let missing_get = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&get_uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_get.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(missing_get.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let missing_get_error = body_json::<ErrorResponse>(missing_get).await;
    assert_eq!(missing_get_error.code, "unauthorized");

    let authorized_get = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(get_uri)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized_get.status(), StatusCode::NOT_FOUND);
    let authorized_get_error = body_json::<ErrorResponse>(authorized_get).await;
    assert_eq!(authorized_get_error.code, "not_found");
}

#[tokio::test]
async fn admin_generated_artifact_metadata_apply_v1_commits_and_replays_redacted_result() {
    let fixture = admin_generated_artifact_metadata_apply_http_fixture().await;
    let uri = format!(
        "/admin/v1/automation/generated-artifacts/{}/metadata-apply",
        fixture.artifact_id
    );
    let request = AdminGeneratedArtifactMetadataApplyRequest {
        idempotency_key: "metadata-apply:operator-confirmation".to_owned(),
    };

    let response = response_body_json(&fixture.router, Method::POST, &uri, &request).await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let applied: AdminGeneratedArtifactMetadataApplyResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        applied.status,
        nako_core::GeneratedArtifactMetadataApplyResultStatus::Applied
    );
    assert!(applied.applied);
    assert!(applied.changed);
    assert!(!applied.idempotent_replay);
    assert!(applied.outcome_id.is_some());
    assert_eq!(applied.plan.apply_field_count, 1);
    assert_eq!(applied.plan.fields[0].field, MetadataField::Overview);
    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        item_after.metadata.overview.as_deref(),
        Some("private generated overview")
    );
    assert!(!body.contains("prompt_json"));
    assert!(!body.contains("artifact_json"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));

    let replay = response_body_json(&fixture.router, Method::POST, &uri, &request).await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminGeneratedArtifactMetadataApplyResponse =
        serde_json::from_str(&replay_body).unwrap();

    assert_eq!(replayed.status, applied.status);
    assert_eq!(replayed.outcome_id, applied.outcome_id);
    assert!(replayed.applied);
    assert!(replayed.changed);
    assert!(replayed.idempotent_replay);
    assert!(!replay_body.contains("prompt_json"));
    assert!(!replay_body.contains("artifact_json"));
    assert!(!replay_body.contains("local:///Movies/private"));
    assert!(!replay_body.contains("private generated overview"));
    assert!(!replay_body.contains("private reasoning"));
    assert!(!replay_body.contains("secret"));
    assert!(!replay_body.contains("sha256-private-fingerprint"));

    let recovery_response = fixture
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/automation/generated-artifact-apply-recovery?attention=resolved")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let recovery_status = recovery_response.status();
    let recovery_body = response_text(recovery_response).await;
    assert_eq!(recovery_status, StatusCode::OK, "{recovery_body}");
    let recovery: AdminGeneratedArtifactMetadataApplyRecoveryResponse =
        serde_json::from_str(&recovery_body).unwrap();
    assert_eq!(recovery.summary.resolved_count, 1);
    assert_eq!(
        recovery.entries[0].attention,
        nako_core::GeneratedArtifactMetadataApplyRecoveryAttention::Resolved
    );
    assert_eq!(recovery.entries[0].outcome_id, applied.outcome_id);
    assert!(!recovery_body.contains("prompt_json"));
    assert!(!recovery_body.contains("artifact_json"));
    assert!(!recovery_body.contains("local:///Movies/private"));
    assert!(!recovery_body.contains("private generated overview"));
    assert!(!recovery_body.contains("private reasoning"));
    assert!(!recovery_body.contains("secret"));
    assert!(!recovery_body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn admin_generated_artifact_metadata_apply_v1_maps_errors_without_sensitive_body() {
    let fixture = admin_generated_artifact_metadata_apply_http_fixture().await;
    let uri = format!(
        "/admin/v1/automation/generated-artifacts/{}/metadata-apply",
        fixture.artifact_id
    );
    let empty_key = response_body_json(
        &fixture.router,
        Method::POST,
        &uri,
        &AdminGeneratedArtifactMetadataApplyRequest {
            idempotency_key: "  ".to_owned(),
        },
    )
    .await;
    assert_eq!(empty_key.status(), StatusCode::BAD_REQUEST);
    let empty_key_body = response_text(empty_key).await;
    let empty_key_error: ErrorResponse = serde_json::from_str(&empty_key_body).unwrap();
    assert_eq!(empty_key_error.code, "invalid_input");
    assert!(empty_key_error.message.contains("idempotency_key"));
    assert!(!empty_key_body.contains("prompt_json"));
    assert!(!empty_key_body.contains("artifact_json"));
    assert!(!empty_key_body.contains("local:///Movies/private"));
    assert!(!empty_key_body.contains("private generated overview"));
    assert!(!empty_key_body.contains("secret"));

    let missing_artifact_id = AutomationArtifactId::new();
    let missing = response_body_json(
        &fixture.router,
        Method::POST,
        &format!("/admin/v1/automation/generated-artifacts/{missing_artifact_id}/metadata-apply"),
        &AdminGeneratedArtifactMetadataApplyRequest {
            idempotency_key: "metadata-apply:missing".to_owned(),
        },
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    let missing_body = response_text(missing).await;
    let missing_error: ErrorResponse = serde_json::from_str(&missing_body).unwrap();
    assert_eq!(missing_error.code, "not_found");
    assert!(!missing_body.contains("prompt_json"));
    assert!(!missing_body.contains("artifact_json"));
    assert!(!missing_body.contains("local:///Movies/private"));
    assert!(!missing_body.contains("private generated overview"));
    assert!(!missing_body.contains("secret"));
}

#[tokio::test]
async fn admin_generated_artifact_metadata_apply_v1_requires_admin_auth() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "metadata-apply-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;
    let artifact_id = AutomationArtifactId::new();
    let uri = format!("/admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply");
    let request = AdminGeneratedArtifactMetadataApplyRequest {
        idempotency_key: "metadata-apply:auth".to_owned(),
    };

    let missing = response_body_json(&router, Method::POST, &uri, &request).await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(missing.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let missing_error = body_json::<ErrorResponse>(missing).await;
    assert_eq!(missing_error.code, "unauthorized");

    let authorized = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized.status(), StatusCode::NOT_FOUND);
    let authorized_error = body_json::<ErrorResponse>(authorized).await;
    assert_eq!(authorized_error.code, "not_found");
}

fn assert_generated_artifact_bulk_metadata_apply_body_redacted(body: &str) {
    assert!(!body.contains("prompt_json"));
    assert!(!body.contains("artifact_json"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
    assert!(!body.contains("bulk-metadata-apply:operator-confirmation"));
}

struct AdminGeneratedArtifactMetadataApplyHttpFixture {
    _temp: tempfile::TempDir,
    app: NakoApp,
    router: Router,
    store: NakoDatabase,
    artifact_id: AutomationArtifactId,
    item_id: MediaItemId,
}

async fn admin_generated_artifact_metadata_apply_http_fixture()
-> AdminGeneratedArtifactMetadataApplyHttpFixture {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
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
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning","provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#.to_owned(),
        })
        .await
        .unwrap();
    app.automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();
    let router = build_router(app.clone());

    AdminGeneratedArtifactMetadataApplyHttpFixture {
        _temp: temp,
        app,
        router,
        store,
        artifact_id: artifact.id,
        item_id: item.id,
    }
}

#[tokio::test]
async fn admin_v1_jobs_lists_filters_and_redacts_raw_payloads() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    let scan = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            priority: nako_core::JobPriority::Normal,
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
            priority: nako_core::JobPriority::Normal,
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
    assert_eq!(jobs.queue_pressure.len(), 2);
    let scan_pressure = jobs
        .queue_pressure
        .iter()
        .find(|pressure| {
            pressure.kind == JobKind::LibraryScan
                && pressure.status == JobStatus::Succeeded
                && pressure.resource_class == "disk.scan"
        })
        .expect("library scan queue pressure");
    assert_eq!(scan_pressure.count, 1);
    assert_eq!(scan_pressure.claimable_count, 0);
    assert_eq!(scan_pressure.delayed_retry_count, 0);
    assert_eq!(scan_pressure.oldest_queued_at, None);
    assert_eq!(scan_pressure.next_attempt_at, None);
    let metadata_pressure = jobs
        .queue_pressure
        .iter()
        .find(|pressure| {
            pressure.kind == JobKind::MetadataRefresh
                && pressure.status == JobStatus::Queued
                && pressure.resource_class == "metadata.tmdb"
        })
        .expect("metadata queue pressure");
    assert_eq!(metadata_pressure.count, 1);
    assert_eq!(metadata_pressure.claimable_count, 1);
    assert_eq!(metadata_pressure.delayed_retry_count, 0);
    assert!(metadata_pressure.oldest_queued_at.is_some());
    assert_eq!(metadata_pressure.next_attempt_at, None);
    assert!(!body.contains("admin-token"));
    assert!(!body.contains("private.nfo"));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("secret"));
}

#[tokio::test]
async fn admin_v1_jobs_lists_source_fingerprint_hash_filters_without_payload_leaks() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let other_source_id = MediaSourceId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Private Source Hash".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: source_id,
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Private/source_hash_secret_locator.mkv".to_owned(),
        file_name: "source_hash_secret_locator.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-source-hash".to_owned()),
    };
    let other_source = MediaSource {
        id: other_source_id,
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Other/source_hash_other_locator.mkv".to_owned(),
        file_name: "source_hash_other_locator.mkv".to_owned(),
        size_bytes: Some(2048),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store.upsert_media_source(&other_source).await.unwrap();

    let source_hash = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source_id),
            input_json: Some(
                r#"{"source_uri":"local:///Movies/Private/source_hash_secret_locator.mkv?token=source-hash-token","fingerprint":"sha256-private-source-hash"}"#.to_owned(),
            ),
        })
        .await
        .unwrap();
    store.start_job(source_hash.id).await.unwrap();
    store
        .fail_job(
            source_hash.id,
            "source hash failed for local:///Movies/Private/source_hash_secret_locator.mkv sha256-private-source-hash".to_owned(),
        )
        .await
        .unwrap();
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(other_source_id),
            input_json: None,
        })
        .await
        .unwrap();
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source_id),
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
                    "/admin/v1/jobs?kind=source_fingerprint_hash&resource_class={SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS}&source_id={source_id}&limit=10"
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
    assert_eq!(jobs.jobs[0].id, source_hash.id);
    assert_eq!(jobs.jobs[0].kind, JobKind::SourceFingerprintHash);
    assert_eq!(jobs.jobs[0].status, JobStatus::Failed);
    assert_eq!(
        jobs.jobs[0].resource_class,
        SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
    );
    assert_eq!(jobs.jobs[0].library_id, Some(library_id));
    assert_eq!(jobs.jobs[0].source_id, Some(source_id));
    assert!(jobs.jobs[0].has_input);
    assert!(jobs.jobs[0].has_error);
    assert_eq!(jobs.page.limit, 10);
    assert_eq!(jobs.page.returned, 1);
    assert!(!body.contains("source_hash_secret_locator"));
    assert!(!body.contains("source-hash-token"));
    assert!(!body.contains("sha256-private-source-hash"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("input_json"));
    assert!(!body.contains("summary_json"));
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_enqueue_queues_full_and_partial_jobs_without_payload_leaks()
 {
    let (_temp, router, source, store) =
        router_with_media_source("source_hash_secret_locator.mkv", b"0123456789abcdef").await;

    let full_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Full,
            partial_prefix_bytes: None,
            priority: Some(AdminJobPriority::High),
        },
    )
    .await;
    let full_status = full_response.status();
    let full_body = response_text(full_response).await;
    assert_eq!(full_status, StatusCode::ACCEPTED, "{full_body}");
    let full_job: AdminJobListItem = serde_json::from_str(&full_body).unwrap();
    let persisted_full = store.get_job(full_job.id).await.unwrap().unwrap();
    let full_input: SourceFingerprintHashJobInput =
        serde_json::from_str(persisted_full.input_json.as_deref().unwrap()).unwrap();

    assert_eq!(full_job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(full_job.status, JobStatus::Queued);
    assert_eq!(
        full_job.resource_class,
        SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
    );
    assert_eq!(full_job.library_id, Some(source.library_id));
    assert_eq!(full_job.source_id, Some(source.id));
    assert!(full_job.has_input);
    assert!(!full_job.has_summary);
    assert!(!full_job.has_error);
    assert_eq!(persisted_full.priority, JobPriority::High);
    assert_eq!(persisted_full.library_id, Some(source.library_id));
    assert_eq!(persisted_full.source_id, Some(source.id));
    assert_eq!(full_input.library_id, source.library_id);
    assert_eq!(full_input.source_id, source.id);
    assert_eq!(full_input.source_scheme, "local");
    assert_eq!(full_input.mode, SourceFingerprintHashMode::Full);
    assert_source_hash_admin_body_redacted(&full_body);
    assert_source_hash_job_input_redacted(persisted_full.input_json.as_deref().unwrap());

    let partial_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Partial,
            partial_prefix_bytes: Some(7),
            priority: None,
        },
    )
    .await;
    let partial_status = partial_response.status();
    let partial_body = response_text(partial_response).await;
    assert_eq!(partial_status, StatusCode::ACCEPTED, "{partial_body}");
    let partial_job: AdminJobListItem = serde_json::from_str(&partial_body).unwrap();
    let persisted_partial = store.get_job(partial_job.id).await.unwrap().unwrap();
    let partial_input: SourceFingerprintHashJobInput =
        serde_json::from_str(persisted_partial.input_json.as_deref().unwrap()).unwrap();

    assert_ne!(partial_job.id, full_job.id);
    assert_eq!(partial_job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(partial_job.status, JobStatus::Queued);
    assert_eq!(
        partial_job.resource_class,
        SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
    );
    assert_eq!(partial_job.library_id, Some(source.library_id));
    assert_eq!(partial_job.source_id, Some(source.id));
    assert_eq!(persisted_partial.priority, JobPriority::Normal);
    assert_eq!(
        partial_input.mode,
        SourceFingerprintHashMode::Partial { prefix_bytes: 7 }
    );
    assert_source_hash_admin_body_redacted(&partial_body);
    assert_source_hash_job_input_redacted(persisted_partial.input_json.as_deref().unwrap());
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_enqueue_propagates_request_id_into_durable_input() {
    let (_temp, router, source, store) =
        router_with_media_source("source_hash_secret_locator.mkv", b"0123456789abcdef").await;
    let request_id = "REQ-SOURCE_123.Trace";
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/source-fingerprint-hashes")
                .header(crate::http::trace_context::X_REQUEST_ID_HEADER, request_id)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&AdminSourceFingerprintHashEnqueueRequest {
                        library_id: source.library_id,
                        source_id: source.id,
                        mode: AdminSourceFingerprintHashMode::Full,
                        partial_prefix_bytes: None,
                        priority: Some(AdminJobPriority::High),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let echoed_request_id = response
        .headers()
        .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let body = response_text(response).await;
    assert_eq!(echoed_request_id, "req-source_123.trace");
    assert_eq!(
        serde_json::from_str::<AdminJobListItem>(&body)
            .unwrap()
            .status,
        JobStatus::Queued
    );
    let job: AdminJobListItem = serde_json::from_str(&body).unwrap();
    let persisted = store.get_job(job.id).await.unwrap().unwrap();
    let input: SourceFingerprintHashJobInput =
        serde_json::from_str(persisted.input_json.as_deref().unwrap()).unwrap();

    assert_eq!(job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(input.request_id.as_deref(), Some("req-source_123.trace"));
    assert_source_hash_admin_body_redacted(&body);
    assert_source_hash_job_input_redacted(persisted.input_json.as_deref().unwrap());
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_retry_requeues_failed_job_without_payload_leaks() {
    let (_temp, router, source, store) =
        router_with_media_source("source_hash_secret_locator.mkv", b"0123456789abcdef").await;
    let enqueue_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Full,
            partial_prefix_bytes: None,
            priority: Some(AdminJobPriority::High),
        },
    )
    .await;
    assert_eq!(enqueue_response.status(), StatusCode::ACCEPTED);
    let source_job = body_json::<AdminJobListItem>(enqueue_response).await;
    store.start_job(source_job.id).await.unwrap();
    let failed = store
        .fail_job(
            source_job.id,
            "source hash failed for local:///Movies/Private/source_hash_secret_locator.mkv sha256-private-source-hash".to_owned(),
        )
        .await
        .unwrap();

    let retry_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/source-fingerprint-hashes/jobs/{}/retry",
            source_job.id
        ),
        &AdminSourceFingerprintHashRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: Some("9999-01-01T00:00:00Z".to_owned()),
        },
    )
    .await;
    let retry_status = retry_response.status();
    let retry_body = response_text(retry_response).await;
    assert_eq!(retry_status, StatusCode::ACCEPTED, "{retry_body}");
    let retry_job: AdminJobListItem = serde_json::from_str(&retry_body).unwrap();
    let persisted_retry = store.get_job(retry_job.id).await.unwrap().unwrap();
    let retry_input: SourceFingerprintHashJobInput =
        serde_json::from_str(persisted_retry.input_json.as_deref().unwrap()).unwrap();

    assert_ne!(retry_job.id, source_job.id);
    assert_eq!(retry_job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(retry_job.status, JobStatus::Queued);
    assert_eq!(
        retry_job.resource_class,
        SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
    );
    assert_eq!(retry_job.library_id, Some(source.library_id));
    assert_eq!(retry_job.source_id, Some(source.id));
    assert!(retry_job.has_input);
    assert!(!retry_job.has_summary);
    assert!(!retry_job.has_error);
    assert_eq!(retry_job.priority, JobPriority::High);
    assert_eq!(retry_job.retry_of_job_id, Some(failed.id));
    assert_eq!(retry_job.attempt, 2);
    assert_eq!(retry_job.max_attempts, 3);
    assert_eq!(
        retry_job.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    assert_eq!(persisted_retry.retry_of_job_id, Some(failed.id));
    assert_eq!(persisted_retry.attempt, 2);
    assert_eq!(persisted_retry.max_attempts, 3);
    assert_eq!(
        persisted_retry.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    assert_eq!(persisted_retry.input_json, failed.input_json);
    assert_eq!(retry_input.library_id, source.library_id);
    assert_eq!(retry_input.source_id, source.id);
    assert_eq!(retry_input.source_scheme, "local");
    assert_eq!(retry_input.mode, SourceFingerprintHashMode::Full);
    assert_source_hash_admin_body_redacted(&retry_body);
    assert_source_hash_job_input_redacted(persisted_retry.input_json.as_deref().unwrap());
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_retry_rejects_invalid_states_without_leaks() {
    let (_temp, router, mut source, store) =
        router_with_media_source("source_hash_secret_locator.mkv", b"0123456789abcdef").await;
    let enqueue_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Full,
            partial_prefix_bytes: None,
            priority: None,
        },
    )
    .await;
    assert_eq!(enqueue_response.status(), StatusCode::ACCEPTED);
    let source_job = body_json::<AdminJobListItem>(enqueue_response).await;

    let not_failed_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/source-fingerprint-hashes/jobs/{}/retry",
            source_job.id
        ),
        &AdminSourceFingerprintHashRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: None,
        },
    )
    .await;
    let not_failed_status = not_failed_response.status();
    let not_failed_body = response_text(not_failed_response).await;
    assert_eq!(not_failed_status, StatusCode::CONFLICT, "{not_failed_body}");
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&not_failed_body)
            .unwrap()
            .message,
        "conflict: only failed jobs can be retried"
    );
    assert_source_hash_admin_body_redacted(&not_failed_body);

    let invalid_timestamp_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/source-fingerprint-hashes/jobs/{}/retry",
            source_job.id
        ),
        &AdminSourceFingerprintHashRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: Some("local:///Secret Path/not-a-timestamp?token=secret".to_owned()),
        },
    )
    .await;
    let invalid_timestamp_status = invalid_timestamp_response.status();
    let invalid_timestamp_body = response_text(invalid_timestamp_response).await;
    assert_eq!(
        invalid_timestamp_status,
        StatusCode::BAD_REQUEST,
        "{invalid_timestamp_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&invalid_timestamp_body)
            .unwrap()
            .message,
        "invalid input: source fingerprint hash retry next_attempt_at must be an RFC3339 timestamp"
    );
    assert_source_hash_admin_body_redacted(&invalid_timestamp_body);
    assert!(!invalid_timestamp_body.contains("not-a-timestamp"));

    store.start_job(source_job.id).await.unwrap();
    let failed = store
        .fail_job(source_job.id, "source hash failed".to_owned())
        .await
        .unwrap();
    source.locator =
        "webdav:///Users/Frankorz/Secret Path/source_hash_secret_locator.mkv?token=secret"
            .to_owned();
    store.upsert_media_source(&source).await.unwrap();

    let scheme_drift_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/source-fingerprint-hashes/jobs/{}/retry",
            failed.id
        ),
        &AdminSourceFingerprintHashRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: None,
        },
    )
    .await;
    let scheme_drift_status = scheme_drift_response.status();
    let scheme_drift_body = response_text(scheme_drift_response).await;
    assert_eq!(
        scheme_drift_status,
        StatusCode::CONFLICT,
        "{scheme_drift_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&scheme_drift_body)
            .unwrap()
            .message,
        "conflict: source fingerprint hash retry source locator scheme changed since enqueue"
    );
    assert_source_hash_admin_body_redacted(&scheme_drift_body);
    assert!(!scheme_drift_body.contains("webdav:///"));

    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(jobs.len(), 1);
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_enqueue_rejects_invalid_requests_without_leaks() {
    let (_temp, router, mut source, store) =
        router_with_media_source("Hidden Movie.mkv", b"secret source").await;

    let missing_source_id = MediaSourceId::new();
    let missing_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: missing_source_id,
            mode: AdminSourceFingerprintHashMode::Full,
            partial_prefix_bytes: None,
            priority: None,
        },
    )
    .await;
    let missing_status = missing_response.status();
    let missing_body = response_text(missing_response).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND, "{missing_body}");
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&missing_body)
            .unwrap()
            .code,
        nako_api::public_client::ClientErrorCode::NotFound.as_str()
    );

    let cross_library_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: LibraryId::new(),
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Full,
            partial_prefix_bytes: None,
            priority: None,
        },
    )
    .await;
    let cross_library_status = cross_library_response.status();
    let cross_library_body = response_text(cross_library_response).await;
    assert_eq!(
        cross_library_status,
        StatusCode::BAD_REQUEST,
        "{cross_library_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&cross_library_body)
            .unwrap()
            .message,
        "invalid input: source fingerprint hash job source does not belong to requested library"
    );
    assert_source_hash_admin_body_redacted(&cross_library_body);

    source.locator = "Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret".to_owned();
    store.upsert_media_source(&source).await.unwrap();
    let invalid_locator_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Full,
            partial_prefix_bytes: None,
            priority: None,
        },
    )
    .await;
    let invalid_locator_status = invalid_locator_response.status();
    let invalid_locator_body = response_text(invalid_locator_response).await;
    assert_eq!(
        invalid_locator_status,
        StatusCode::BAD_REQUEST,
        "{invalid_locator_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&invalid_locator_body)
            .unwrap()
            .message,
        "invalid input: source fingerprint hash job source locator is not a valid storage URI"
    );
    assert_source_hash_admin_body_redacted(&invalid_locator_body);

    let invalid_prefix_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/source-fingerprint-hashes",
        &AdminSourceFingerprintHashEnqueueRequest {
            library_id: source.library_id,
            source_id: source.id,
            mode: AdminSourceFingerprintHashMode::Partial,
            partial_prefix_bytes: Some(0),
            priority: None,
        },
    )
    .await;
    let invalid_prefix_status = invalid_prefix_response.status();
    let invalid_prefix_body = response_text(invalid_prefix_response).await;
    assert_eq!(
        invalid_prefix_status,
        StatusCode::BAD_REQUEST,
        "{invalid_prefix_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&invalid_prefix_body)
            .unwrap()
            .message,
        "invalid input: partial source fingerprint hash prefix must be greater than zero"
    );
    assert_source_hash_admin_body_redacted(&invalid_prefix_body);

    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert!(jobs.is_empty());
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_enqueue_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "source-hash-viewer".to_owned(),
            display_name: "Source Hash Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "source-hash-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/source-fingerprint-hashes")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&AdminSourceFingerprintHashEnqueueRequest {
                        library_id,
                        source_id,
                        mode: AdminSourceFingerprintHashMode::Full,
                        partial_prefix_bytes: None,
                        priority: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(error.message, "administrator role is required");
}

#[tokio::test]
async fn admin_v1_source_fingerprint_hash_retry_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let job_id = JobId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "source-hash-retry-viewer".to_owned(),
            display_name: "Source Hash Retry Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "source-hash-retry-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/source-fingerprint-hashes/jobs/{job_id}/retry"
                ))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&AdminSourceFingerprintHashRetryRequest {
                        max_attempts: Some(3),
                        next_attempt_at: None,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(error.message, "administrator role is required");
}

const ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT: &str = "source:v1:content_hash:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_plan_returns_read_only_safe_plan() {
    let (_temp, router, mut target, store) =
        router_with_media_source("source_duplicate_secret_locator.mkv", b"media").await;
    target.locator =
        "local:///Users/Frankorz/Secret Target.mkv?token=source-duplicate-token".to_owned();
    target.fingerprint = Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT.to_owned());
    store.upsert_media_source(&target).await.unwrap();

    let suggested = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Suggested",
        "local:///Users/Frankorz/Suggested.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let confirmed = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Confirmed",
        "local:///Users/Frankorz/Confirmed.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let rejected = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Rejected",
        "local:///Users/Frankorz/Rejected.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let stale = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Stale",
        "local:///Users/Frankorz/Stale.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let fresh = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Fresh",
        "local:///Users/Frankorz/Fresh.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;

    seed_admin_source_duplicate_relationship(
        &store,
        target.id,
        suggested.id,
        nako_core::SourceDuplicateRelationshipStatus::Suggested,
    )
    .await;
    seed_admin_source_duplicate_relationship(
        &store,
        target.id,
        confirmed.id,
        nako_core::SourceDuplicateRelationshipStatus::Confirmed,
    )
    .await;
    seed_admin_source_duplicate_relationship(
        &store,
        target.id,
        rejected.id,
        nako_core::SourceDuplicateRelationshipStatus::Rejected,
    )
    .await;
    seed_admin_source_duplicate_stale_state(&store, target.library_id, &stale).await;

    let before = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();
    let response = get_admin_source_duplicate_reconciliation_plan(
        &router,
        target.library_id,
        target.id,
        "?limit=10",
    )
    .await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let plan: AdminSourceDuplicateReconciliationPlanResponse = serde_json::from_str(&body).unwrap();
    let after = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();

    assert_eq!(before, after);
    assert_eq!(before.len(), 3);
    assert_eq!(plan.admin_api_version, nako_api::admin::ADMIN_API_VERSION);
    assert_eq!(plan.library_id, target.library_id);
    assert_eq!(plan.source_id, target.id);
    assert_eq!(
        plan.fingerprint_evidence_kind,
        nako_core::SourceFingerprintEvidenceKind::ContentHash
    );
    assert_eq!(plan.confidence_milli, 1_000);
    assert!(!plan.stale);
    assert_eq!(plan.page.limit, 10);
    assert_eq!(plan.page.offset, 0);
    assert_eq!(plan.page.returned, 5);
    assert_eq!(plan.candidates.len(), 5);

    let assert_candidate =
        |duplicate_source_id: MediaSourceId,
         expected_action: nako_core::SourceDuplicateReconciliationAction,
         expected_status: Option<nako_core::SourceDuplicateRelationshipStatus>,
         stale: bool,
         confidence_milli: Option<u16>| {
            let candidate = plan
                .candidates
                .iter()
                .find(|candidate| candidate.duplicate_source_id == duplicate_source_id)
                .expect("candidate should be returned");

            assert_eq!(candidate.source_id, target.id);
            assert_eq!(
                candidate.evidence_kind,
                nako_core::SourceDuplicateEvidenceKind::StrongFingerprint
            );
            assert_eq!(candidate.recommended_action, expected_action);
            assert_eq!(candidate.existing_status, expected_status);
            assert_eq!(candidate.stale, stale);
            assert_eq!(candidate.confidence_milli, confidence_milli);
            if expected_status.is_some() {
                assert!(candidate.relationship_id.is_some());
            } else {
                assert_eq!(candidate.relationship_id, None);
            }
        };

    assert_candidate(
        suggested.id,
        nako_core::SourceDuplicateReconciliationAction::PreserveSuggested,
        Some(nako_core::SourceDuplicateRelationshipStatus::Suggested),
        false,
        Some(1_000),
    );
    assert_candidate(
        confirmed.id,
        nako_core::SourceDuplicateReconciliationAction::PreserveConfirmed,
        Some(nako_core::SourceDuplicateRelationshipStatus::Confirmed),
        false,
        Some(1_000),
    );
    assert_candidate(
        rejected.id,
        nako_core::SourceDuplicateReconciliationAction::PreserveRejected,
        Some(nako_core::SourceDuplicateRelationshipStatus::Rejected),
        false,
        Some(1_000),
    );
    assert_candidate(
        stale.id,
        nako_core::SourceDuplicateReconciliationAction::RefreshSourceFingerprint,
        None,
        true,
        Some(800),
    );
    assert_candidate(
        fresh.id,
        nako_core::SourceDuplicateReconciliationAction::SuggestRelationship,
        None,
        false,
        Some(1_000),
    );
    assert_source_duplicate_plan_body_redacted(&body);
}

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_apply_creates_and_replays_safely() {
    let (_temp, router, mut target, store) =
        router_with_media_source("source_duplicate_apply_secret_locator.mkv", b"media").await;
    target.locator =
        "local:///Users/Frankorz/Secret Target.mkv?token=source-duplicate-token".to_owned();
    target.fingerprint = Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT.to_owned());
    store.upsert_media_source(&target).await.unwrap();
    let duplicate = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Apply Duplicate",
        "local:///Users/Frankorz/Apply Duplicate.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let request = AdminSourceDuplicateReconciliationApplyRequest {
        duplicate_source_id: duplicate.id,
        expected_action: AdminSourceDuplicateReconciliationApplyExpectedAction::SuggestRelationship,
    };

    let response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request,
    )
    .await;
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let applied: AdminSourceDuplicateReconciliationApplyResponse =
        serde_json::from_str(&body).unwrap();
    let relationship = store
        .get_source_duplicate_relationship_by_pair(target.id, duplicate.id)
        .await
        .unwrap()
        .expect("relationship should be persisted");

    assert!(applied.created);
    assert_eq!(
        applied.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(applied.library_id, target.library_id);
    assert_eq!(applied.source_id, target.id);
    assert_eq!(applied.duplicate_source_id, duplicate.id);
    assert_eq!(applied.relationship_id, relationship.id);
    assert_eq!(
        applied.relationship_status,
        nako_core::SourceDuplicateRelationshipStatus::Suggested
    );
    assert_eq!(
        applied.applied_action,
        nako_core::SourceDuplicateReconciliationAction::SuggestRelationship
    );
    assert_eq!(
        (relationship.source_id, relationship.duplicate_source_id),
        nako_core::SourceDuplicateRelationship::canonical_pair(target.id, duplicate.id)
    );
    assert_eq!(
        relationship.status,
        nako_core::SourceDuplicateRelationshipStatus::Suggested
    );
    assert_source_duplicate_plan_body_redacted(&body);

    let replay = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request,
    )
    .await;
    let replay_status = replay.status();
    let replay_body = response_text(replay).await;
    assert_eq!(replay_status, StatusCode::OK, "{replay_body}");
    let replayed: AdminSourceDuplicateReconciliationApplyResponse =
        serde_json::from_str(&replay_body).unwrap();
    let relationships = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();

    assert!(!replayed.created);
    assert_eq!(replayed.relationship_id, relationship.id);
    assert_eq!(
        replayed.relationship_status,
        nako_core::SourceDuplicateRelationshipStatus::Suggested
    );
    assert_eq!(
        replayed.applied_action,
        nako_core::SourceDuplicateReconciliationAction::PreserveSuggested
    );
    assert_eq!(relationships.len(), 1);
    assert_source_duplicate_plan_body_redacted(&replay_body);
}

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_apply_rejects_unsafe_states_without_writes() {
    let (_temp, router, mut target, store) =
        router_with_media_source("source_duplicate_apply_reject_target.mkv", b"media").await;
    target.locator =
        "local:///Users/Frankorz/Secret Target.mkv?token=source-duplicate-token".to_owned();
    target.fingerprint = Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT.to_owned());
    store.upsert_media_source(&target).await.unwrap();
    let confirmed = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Confirmed",
        "local:///Users/Frankorz/Confirmed.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let rejected = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Rejected",
        "local:///Users/Frankorz/Rejected.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let stale = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Stale",
        "local:///Users/Frankorz/Stale.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let mismatch = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Mismatch",
        "local:///Users/Frankorz/Mismatch.mkv",
        Some(
            "source:v1:content_hash:sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ),
    )
    .await;
    let missing_fingerprint = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Missing Fingerprint",
        "local:///Users/Frankorz/Missing Fingerprint.mkv?token=secret",
        None,
    )
    .await;
    let raw_fingerprint = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Raw Fingerprint",
        "local:///Users/Frankorz/Raw Fingerprint.mkv?token=secret",
        Some("sha256:private-raw-fingerprint"),
    )
    .await;
    let other_library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: other_library_id,
            name: "Other Movies".to_owned(),
            roots: vec!["local:///Other Movies".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let other_library = seed_admin_source_duplicate_source(
        &store,
        other_library_id,
        "Other Library",
        "local:///Users/Frankorz/Other Library.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let stale_target = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Stale Target",
        "local:///Users/Frankorz/Stale Target.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let stale_target_duplicate = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Stale Target Duplicate",
        "local:///Users/Frankorz/Stale Target Duplicate.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;

    seed_admin_source_duplicate_relationship(
        &store,
        target.id,
        confirmed.id,
        nako_core::SourceDuplicateRelationshipStatus::Confirmed,
    )
    .await;
    seed_admin_source_duplicate_relationship(
        &store,
        target.id,
        rejected.id,
        nako_core::SourceDuplicateRelationshipStatus::Rejected,
    )
    .await;
    seed_admin_source_duplicate_stale_state(&store, target.library_id, &stale).await;
    seed_admin_source_duplicate_stale_state(&store, target.library_id, &stale_target).await;

    let before = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();
    let request_for = |duplicate_source_id| AdminSourceDuplicateReconciliationApplyRequest {
        duplicate_source_id,
        expected_action: AdminSourceDuplicateReconciliationApplyExpectedAction::SuggestRelationship,
    };

    let confirmed_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(confirmed.id),
    )
    .await;
    let confirmed_body = assert_error_response(
        confirmed_response,
        StatusCode::CONFLICT,
        "conflict",
        "conflict: source duplicate reconciliation apply expected suggest_relationship but current recommendation is preserve_confirmed",
    )
    .await;
    let rejected_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(rejected.id),
    )
    .await;
    let rejected_body = assert_error_response(
        rejected_response,
        StatusCode::CONFLICT,
        "conflict",
        "conflict: source duplicate reconciliation apply expected suggest_relationship but current recommendation is preserve_rejected",
    )
    .await;
    let stale_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(stale.id),
    )
    .await;
    let stale_body = assert_error_response(
        stale_response,
        StatusCode::CONFLICT,
        "conflict",
        "conflict: source duplicate reconciliation apply expected suggest_relationship but current recommendation is refresh_source_fingerprint",
    )
    .await;
    let stale_target_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        stale_target.id,
        &request_for(stale_target_duplicate.id),
    )
    .await;
    let stale_target_body = assert_error_response(
        stale_target_response,
        StatusCode::CONFLICT,
        "conflict",
        "conflict: source duplicate reconciliation apply expected suggest_relationship but current recommendation is refresh_source_fingerprint",
    )
    .await;
    let mismatch_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(mismatch.id),
    )
    .await;
    let mismatch_body = assert_error_response(
        mismatch_response,
        StatusCode::BAD_REQUEST,
        "invalid_input",
        "invalid input: source duplicate reconciliation candidate fingerprint does not match source fingerprint evidence",
    )
    .await;
    let missing_fingerprint_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(missing_fingerprint.id),
    )
    .await;
    let missing_fingerprint_body = assert_error_response(
        missing_fingerprint_response,
        StatusCode::BAD_REQUEST,
        "invalid_input",
        "invalid input: source duplicate reconciliation requires source fingerprint evidence",
    )
    .await;
    let raw_fingerprint_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(raw_fingerprint.id),
    )
    .await;
    let raw_fingerprint_body = assert_error_response(
        raw_fingerprint_response,
        StatusCode::BAD_REQUEST,
        "invalid_input",
        "invalid input: source duplicate reconciliation requires redacted source fingerprint evidence",
    )
    .await;
    let cross_library_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(other_library.id),
    )
    .await;
    let cross_library_body = assert_error_response(
        cross_library_response,
        StatusCode::BAD_REQUEST,
        "invalid_input",
        "invalid input: source duplicate reconciliation candidate does not belong to requested library",
    )
    .await;
    let missing_candidate_response = post_admin_source_duplicate_reconciliation_apply(
        &router,
        target.library_id,
        target.id,
        &request_for(MediaSourceId::new()),
    )
    .await;
    let missing_candidate_body = assert_error_response(
        missing_candidate_response,
        StatusCode::NOT_FOUND,
        "not_found",
        "not found:",
    )
    .await;
    let after = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();
    let stale_target_after = store
        .list_source_duplicate_relationships(stale_target.id, PageRequest::new(20, 0))
        .await
        .unwrap();

    assert_eq!(before, after);
    assert_eq!(before.len(), 2);
    assert!(stale_target_after.is_empty());
    assert_eq!(
        store
            .get_source_duplicate_relationship_by_pair(target.id, confirmed.id)
            .await
            .unwrap()
            .unwrap()
            .status,
        nako_core::SourceDuplicateRelationshipStatus::Confirmed
    );
    assert_eq!(
        store
            .get_source_duplicate_relationship_by_pair(target.id, rejected.id)
            .await
            .unwrap()
            .unwrap()
            .status,
        nako_core::SourceDuplicateRelationshipStatus::Rejected
    );

    for body in [
        confirmed_body,
        rejected_body,
        stale_body,
        stale_target_body,
        mismatch_body,
        missing_fingerprint_body,
        raw_fingerprint_body,
        cross_library_body,
        missing_candidate_body,
    ] {
        assert_source_duplicate_plan_body_redacted(&body);
    }
}

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_plan_paginates_after_excluding_target() {
    let (_temp, router, mut target, store) =
        router_with_media_source("source_duplicate_pagination_target.mkv", b"media").await;
    target.fingerprint = Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT.to_owned());
    store.upsert_media_source(&target).await.unwrap();
    let first = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "First",
        "local:///Users/Frankorz/First.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;
    let second = seed_admin_source_duplicate_source(
        &store,
        target.library_id,
        "Second",
        "local:///Users/Frankorz/Second.mkv",
        Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT),
    )
    .await;

    let first_page_response = get_admin_source_duplicate_reconciliation_plan(
        &router,
        target.library_id,
        target.id,
        "?limit=1",
    )
    .await;
    let first_status = first_page_response.status();
    let first_body = response_text(first_page_response).await;
    assert_eq!(first_status, StatusCode::OK, "{first_body}");
    let first_page: AdminSourceDuplicateReconciliationPlanResponse =
        serde_json::from_str(&first_body).unwrap();

    let second_page_response = get_admin_source_duplicate_reconciliation_plan(
        &router,
        target.library_id,
        target.id,
        "?limit=1&offset=1",
    )
    .await;
    let second_status = second_page_response.status();
    let second_body = response_text(second_page_response).await;
    assert_eq!(second_status, StatusCode::OK, "{second_body}");
    let second_page: AdminSourceDuplicateReconciliationPlanResponse =
        serde_json::from_str(&second_body).unwrap();

    assert_eq!(first_page.page.limit, 1);
    assert_eq!(first_page.page.offset, 0);
    assert_eq!(first_page.page.returned, 1);
    assert_eq!(second_page.page.limit, 1);
    assert_eq!(second_page.page.offset, 1);
    assert_eq!(second_page.page.returned, 1);
    assert_eq!(first_page.candidates.len(), 1);
    assert_eq!(second_page.candidates.len(), 1);
    assert_ne!(
        first_page.candidates[0].duplicate_source_id,
        second_page.candidates[0].duplicate_source_id
    );
    assert_ne!(first_page.candidates[0].duplicate_source_id, target.id);
    assert_ne!(second_page.candidates[0].duplicate_source_id, target.id);
    assert!([first.id, second.id].contains(&first_page.candidates[0].duplicate_source_id));
    assert!([first.id, second.id].contains(&second_page.candidates[0].duplicate_source_id));
    assert_source_duplicate_plan_body_redacted(&first_body);
    assert_source_duplicate_plan_body_redacted(&second_body);
}

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_plan_rejects_unsafe_inputs_without_leaks() {
    let (_temp, router, mut source, store) =
        router_with_media_source("Hidden Duplicate.mkv", b"secret source").await;
    source.locator =
        "local:///Users/Frankorz/Secret Path/Hidden Duplicate.mkv?token=secret".to_owned();
    source.fingerprint = None;
    store.upsert_media_source(&source).await.unwrap();

    let missing_response = get_admin_source_duplicate_reconciliation_plan(
        &router,
        source.library_id,
        MediaSourceId::new(),
        "",
    )
    .await;
    let missing_status = missing_response.status();
    let missing_body = response_text(missing_response).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND, "{missing_body}");
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&missing_body)
            .unwrap()
            .code,
        nako_api::public_client::ClientErrorCode::NotFound.as_str()
    );
    assert_source_duplicate_plan_body_redacted(&missing_body);

    let missing_fingerprint_response =
        get_admin_source_duplicate_reconciliation_plan(&router, source.library_id, source.id, "")
            .await;
    let missing_fingerprint_status = missing_fingerprint_response.status();
    let missing_fingerprint_body = response_text(missing_fingerprint_response).await;
    assert_eq!(
        missing_fingerprint_status,
        StatusCode::BAD_REQUEST,
        "{missing_fingerprint_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&missing_fingerprint_body)
            .unwrap()
            .message,
        "invalid input: source duplicate reconciliation requires source fingerprint evidence"
    );
    assert_source_duplicate_plan_body_redacted(&missing_fingerprint_body);

    source.fingerprint = Some("sha256:private-raw-fingerprint".to_owned());
    store.upsert_media_source(&source).await.unwrap();
    let raw_fingerprint_response =
        get_admin_source_duplicate_reconciliation_plan(&router, source.library_id, source.id, "")
            .await;
    let raw_fingerprint_status = raw_fingerprint_response.status();
    let raw_fingerprint_body = response_text(raw_fingerprint_response).await;
    assert_eq!(
        raw_fingerprint_status,
        StatusCode::BAD_REQUEST,
        "{raw_fingerprint_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&raw_fingerprint_body)
            .unwrap()
            .message,
        "invalid input: source duplicate reconciliation requires redacted source fingerprint evidence"
    );
    assert_source_duplicate_plan_body_redacted(&raw_fingerprint_body);

    source.fingerprint = Some(ADMIN_SOURCE_DUPLICATE_CONTENT_FINGERPRINT.to_owned());
    store.upsert_media_source(&source).await.unwrap();
    let cross_library_response =
        get_admin_source_duplicate_reconciliation_plan(&router, LibraryId::new(), source.id, "")
            .await;
    let cross_library_status = cross_library_response.status();
    let cross_library_body = response_text(cross_library_response).await;
    assert_eq!(
        cross_library_status,
        StatusCode::BAD_REQUEST,
        "{cross_library_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&cross_library_body)
            .unwrap()
            .message,
        "invalid input: source duplicate reconciliation source does not belong to requested library"
    );
    assert_source_duplicate_plan_body_redacted(&cross_library_body);
}

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_plan_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "source-duplicate-viewer".to_owned(),
            display_name: "Source Duplicate Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "source-duplicate-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan"
                ))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(error.message, "administrator role is required");
}

#[tokio::test]
async fn admin_v1_source_duplicate_reconciliation_apply_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let duplicate_source_id = MediaSourceId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "source-duplicate-applier".to_owned(),
            display_name: "Source Duplicate Applier".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "source-duplicate-applier".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply"
                ))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&AdminSourceDuplicateReconciliationApplyRequest {
                        duplicate_source_id,
                        expected_action:
                            AdminSourceDuplicateReconciliationApplyExpectedAction::SuggestRelationship,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(error.message, "administrator role is required");
}

async fn get_admin_source_duplicate_reconciliation_plan(
    router: &Router,
    library_id: LibraryId,
    source_id: MediaSourceId,
    query: &str,
) -> Response {
    response_for(
        router,
        Method::GET,
        &format!(
            "/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan{query}"
        ),
    )
    .await
}

async fn post_admin_source_duplicate_reconciliation_apply(
    router: &Router,
    library_id: LibraryId,
    source_id: MediaSourceId,
    request: &AdminSourceDuplicateReconciliationApplyRequest,
) -> Response {
    response_body_json(
        router,
        Method::POST,
        &format!(
            "/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply"
        ),
        request,
    )
    .await
}

async fn assert_error_response(
    response: Response,
    expected_status: StatusCode,
    expected_code: &str,
    expected_message_fragment: &str,
) -> String {
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, expected_status, "{body}");
    let error: ErrorResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(error.code, expected_code);
    assert!(
        error.message.contains(expected_message_fragment),
        "expected error message to contain {expected_message_fragment:?}, got {:?}",
        error.message
    );
    body
}

async fn seed_admin_source_duplicate_source(
    store: &NakoDatabase,
    library_id: LibraryId,
    title: &str,
    locator: &str,
    fingerprint: Option<&str>,
) -> MediaSource {
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: locator.to_owned(),
        file_name: format!("{title}.mkv"),
        size_bytes: Some(42),
        fingerprint: fingerprint.map(ToOwned::to_owned),
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    source
}

async fn seed_admin_source_duplicate_relationship(
    store: &NakoDatabase,
    source_id: MediaSourceId,
    duplicate_source_id: MediaSourceId,
    status: nako_core::SourceDuplicateRelationshipStatus,
) {
    store
        .upsert_source_duplicate_relationship(&nako_core::SourceDuplicateRelationship {
            id: nako_core::SourceDuplicateRelationshipId::new(),
            source_id,
            duplicate_source_id,
            evidence_kind: nako_core::SourceDuplicateEvidenceKind::StrongFingerprint,
            evidence_value: Some("redacted-existing-evidence".to_owned()),
            status,
            confidence_milli: Some(1_000),
        })
        .await
        .unwrap();
}

async fn seed_admin_source_duplicate_stale_state(
    store: &NakoDatabase,
    library_id: LibraryId,
    source: &MediaSource,
) {
    let scan_id = ScanSnapshotId::new();
    store
        .begin_scan_snapshot(scan_id, library_id, "local:///Users/Frankorz")
        .await
        .unwrap();
    store
        .upsert_source_state(&SourceState {
            library_id,
            source_id: Some(source.id),
            uri: source.locator.clone(),
            size_bytes: source.size_bytes,
            modified_at: Some("2026-06-06T00:00:00Z".to_owned()),
            etag: Some("private-etag".to_owned()),
            fingerprint: source.fingerprint.clone(),
            last_seen_scan_id: scan_id,
            tombstoned: true,
        })
        .await
        .unwrap();
    store
        .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
        .await
        .unwrap();
}

fn assert_source_duplicate_plan_body_redacted(body: &str) {
    for forbidden in [
        "source_duplicate_secret_locator",
        "Hidden Duplicate",
        "Secret Target",
        "Secret Path",
        "Frankorz",
        "local:///",
        "source-duplicate-token",
        "private-raw-fingerprint",
        "private-etag",
        "source_uri",
        "source_locator",
        "input_json",
        "summary_json",
        "evidence_value",
        "sha256:",
    ] {
        assert!(
            !body.contains(forbidden),
            "source duplicate Admin response leaked forbidden term: {forbidden}"
        );
    }
}

fn assert_source_hash_admin_body_redacted(body: &str) {
    for forbidden in [
        "source_hash_secret_locator",
        "Hidden Movie",
        "Secret Path",
        "Frankorz",
        "local:///",
        "source_uri",
        "source_scheme",
        "input_json",
        "summary_json",
        "partial_prefix_bytes",
        "token",
        "sha256",
        "fingerprint\":\"",
    ] {
        assert!(
            !body.contains(forbidden),
            "source hash Admin response leaked forbidden term: {forbidden}"
        );
    }
}

fn assert_source_hash_job_input_redacted(input_json: &str) {
    for forbidden in [
        "source_hash_secret_locator",
        "Hidden Movie",
        "Secret Path",
        "Frankorz",
        "local:///",
        "source_uri",
        "token",
        "sha256",
        "fingerprint",
        "etag",
    ] {
        assert!(
            !input_json.contains(forbidden),
            "source hash job input leaked forbidden term: {forbidden}"
        );
    }
}

fn assert_vfs_cache_repair_admin_response_redacted(body: &str) {
    for forbidden in [
        "source_uri",
        "source_locator",
        "cache_uri",
        "storage_uri",
        "local_path",
        "local:///",
        "Private",
        "AdminJob.mkv",
        "Config.mkv",
        "backend.example",
        "secret-cache",
        "token=secret",
        "input_json",
        "summary_json",
        "uri_digest",
        "raw backend",
        "etag",
        "fingerprint",
    ] {
        assert!(
            !body.contains(forbidden),
            "VFS cache repair Admin response leaked forbidden term: {forbidden}"
        );
    }
}

fn assert_vfs_cache_repair_job_input_redacted(input_json: &str) {
    for forbidden in [
        "source_uri",
        "source_locator",
        "cache_uri",
        "storage_uri",
        "local_path",
        "local:///",
        "Private",
        "AdminJob.mkv",
        "backend.example",
        "secret-cache",
        "token=secret",
        "raw backend",
        "etag",
        "fingerprint",
    ] {
        assert!(
            !input_json.contains(forbidden),
            "VFS cache repair job input leaked forbidden term: {forbidden}"
        );
    }
}

fn assert_vfs_cache_repair_job_summary_redacted(summary_json: &str) {
    for forbidden in [
        "source_uri",
        "source_locator",
        "cache_uri",
        "storage_uri",
        "local_path",
        "local:///",
        "Private",
        "AdminJob.mkv",
        "backend.example",
        "secret-cache",
        "token=secret",
        "input_json",
        "uri_digest",
        "raw backend",
        "etag",
        "fingerprint",
    ] {
        assert!(
            !summary_json.contains(forbidden),
            "VFS cache repair job summary leaked forbidden term: {forbidden}"
        );
    }
}

async fn vfs_cache_repair_retry_http_fixture()
-> (tempfile::TempDir, Router, NakoDatabase, LibraryId, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    fs::create_dir_all(root.join("Movies").join("Private")).unwrap();
    fs::write(
        root.join("Movies").join("Private").join("AdminJob.mkv"),
        b"repair-job",
    )
    .unwrap();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            config.libraries[0].clone(),
            Arc::new(nako_vfs::CachedStorageBackend::with_options(
                nako_vfs::LocalFsBackend::new(&root).unwrap(),
                store.clone(),
                nako_vfs::VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/AdminJob.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: StorageFailureClass::Unavailable.safe_message().to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    (temp, build_router(app), store, library_id, root)
}

#[tokio::test]
async fn admin_v1_job_cancel_requests_are_truthful_and_redacted() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let queued = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.tmdb".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: None,
            input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
        })
        .await
        .unwrap();
    let running = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: None,
            input_json: Some(r#"{"path":"private.nfo"}"#.to_owned()),
        })
        .await
        .unwrap();
    store.start_job(running.id).await.unwrap();
    let succeeded = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::NfoExport,
            resource_class: "metadata.nfo.export".to_owned(),
            priority: nako_core::JobPriority::Normal,
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(succeeded.id).await.unwrap();
    store
        .succeed_job(succeeded.id, Some(r#"{"private":"summary"}"#.to_owned()))
        .await
        .unwrap();

    let router = build_router(app);
    let queued_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/admin/v1/jobs/{}/cancel", queued.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(queued_response.status(), StatusCode::OK);
    let queued_body = String::from_utf8(
        to_bytes(queued_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let queued_cancel: AdminJobCancelRequestResponse = serde_json::from_str(&queued_body).unwrap();
    assert!(queued_cancel.requested);
    assert!(queued_cancel.terminal);
    assert_eq!(queued_cancel.job.status, JobStatus::Cancelled);
    assert!(queued_cancel.cancel_requested_at.is_some());
    assert!(!queued_body.contains("admin-token"));
    assert!(!queued_body.contains("secret"));
    assert!(!queued_body.contains("input_json"));

    let running_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/admin/v1/jobs/{}/cancel", running.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(running_response.status(), StatusCode::OK);
    let running_body = String::from_utf8(
        to_bytes(running_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let running_cancel: AdminJobCancelRequestResponse =
        serde_json::from_str(&running_body).unwrap();
    assert!(running_cancel.requested);
    assert!(!running_cancel.terminal);
    assert_eq!(running_cancel.job.status, JobStatus::Running);
    assert!(running_cancel.cancel_requested_at.is_some());
    assert!(!running_body.contains("private.nfo"));
    assert!(!running_body.contains("input_json"));

    let terminal_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/admin/v1/jobs/{}/cancel", succeeded.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(terminal_response.status(), StatusCode::CONFLICT);
    let terminal_error = body_json::<ErrorResponse>(terminal_response).await;
    assert_eq!(
        terminal_error.code,
        nako_api::public_client::ClientErrorCode::Conflict.as_str()
    );
}

#[tokio::test]
async fn admin_v1_events_lists_filters_and_redacts_payloads() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
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
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig {
            max_bytes: 50,
            retention_ms: 8_888,
            cleanup_on_startup: true,
        },
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let staging_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: staging_id,
            attribution: StagingAttribution::unknown(),
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
            authority: VfsCacheFailureAuthority::default(),
        })
        .await
        .unwrap();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: StagingManifestId::new(),
            attribution: StagingAttribution::unknown(),
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
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: StagingManifestId::new(),
            attribution: StagingAttribution::unknown(),
            source_uri: "webdav:///Movies/Private/Failed.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: temp
                .path()
                .join("secret-cache")
                .join("probe")
                .join("Failed.mkv")
                .display()
                .to_string(),
            size_bytes: None,
            etag: Some("failed-etag-secret".to_owned()),
            fingerprint: Some("failed-fingerprint-secret".to_owned()),
            state: StagingState::Failed,
            created_at_ms: 3_000,
            updated_at_ms: 3_100,
            last_accessed_at_ms: 3_200,
            expires_at_ms: Some(3_300),
            active_leases: 2,
            validation_error: Some(
                "backend raw error: F:\\Nako\\secret-cache\\token=secret".to_owned(),
            ),
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

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(diagnostics.summary.configured_max_bytes, 50);
    assert_eq!(diagnostics.summary.used_manifest_bytes, 52);
    assert_eq!(
        diagnostics.summary.pressure.status,
        AdminStorageStagingPressureStatus::Exhausted
    );
    assert_eq!(diagnostics.summary.pressure.used_ratio_milli, Some(1_040));
    assert_eq!(diagnostics.summary.pressure.total_records, 3);
    assert_eq!(diagnostics.summary.pressure.in_flight_records, 1);
    assert_eq!(diagnostics.summary.pressure.failed_records, 1);
    assert_eq!(diagnostics.summary.pressure.unknown_size_records, 1);
    assert_eq!(diagnostics.summary.pressure.active_leases, 2);
    assert_eq!(diagnostics.summary.pressure.ffmpeg_input_records, 1);
    assert_eq!(diagnostics.summary.pressure.probe_input_records, 2);
    assert_eq!(diagnostics.summary.purpose_state_summaries.len(), 3);
    let ffmpeg_ready = diagnostics
        .summary
        .purpose_state_summaries
        .iter()
        .find(|summary| {
            summary.purpose == StagingPurpose::FfmpegInput && summary.state == StagingState::Ready
        })
        .expect("ffmpeg ready staging summary");
    assert_eq!(ffmpeg_ready.record_count, 1);
    assert_eq!(ffmpeg_ready.used_manifest_bytes, 42);
    assert_eq!(ffmpeg_ready.active_leases, 0);
    assert_eq!(ffmpeg_ready.unknown_size_records, 0);
    let probe_reserved = diagnostics
        .summary
        .purpose_state_summaries
        .iter()
        .find(|summary| {
            summary.purpose == StagingPurpose::ProbeInput && summary.state == StagingState::Reserved
        })
        .expect("probe reserved staging summary");
    assert_eq!(probe_reserved.record_count, 1);
    assert_eq!(probe_reserved.used_manifest_bytes, 10);
    assert_eq!(probe_reserved.active_leases, 0);
    assert_eq!(probe_reserved.unknown_size_records, 0);
    let probe_failed = diagnostics
        .summary
        .purpose_state_summaries
        .iter()
        .find(|summary| {
            summary.purpose == StagingPurpose::ProbeInput && summary.state == StagingState::Failed
        })
        .expect("probe failed staging summary");
    assert_eq!(probe_failed.record_count, 1);
    assert_eq!(probe_failed.used_manifest_bytes, 0);
    assert_eq!(probe_failed.active_leases, 2);
    assert_eq!(probe_failed.unknown_size_records, 1);
    assert!(diagnostics.summary.cleanup_on_startup);
    assert_eq!(diagnostics.summary.retention_ms, 8_888);
    assert_eq!(diagnostics.summary.cleanup_candidate_records, 2);
    assert_eq!(diagnostics.summary.cleanup_candidate_bytes, 52);
    assert_eq!(diagnostics.summary.cleanup_purpose_state_summaries.len(), 2);
    let cleanup_ffmpeg_ready = diagnostics
        .summary
        .cleanup_purpose_state_summaries
        .iter()
        .find(|summary| {
            summary.purpose == StagingPurpose::FfmpegInput && summary.state == StagingState::Ready
        })
        .expect("ffmpeg ready cleanup candidate summary");
    assert_eq!(cleanup_ffmpeg_ready.record_count, 1);
    assert_eq!(cleanup_ffmpeg_ready.cleanup_candidate_bytes, 42);
    assert_eq!(cleanup_ffmpeg_ready.active_leases, 0);
    assert_eq!(cleanup_ffmpeg_ready.unknown_size_records, 0);
    let cleanup_probe_reserved = diagnostics
        .summary
        .cleanup_purpose_state_summaries
        .iter()
        .find(|summary| {
            summary.purpose == StagingPurpose::ProbeInput && summary.state == StagingState::Reserved
        })
        .expect("probe reserved cleanup candidate summary");
    assert_eq!(cleanup_probe_reserved.record_count, 1);
    assert_eq!(cleanup_probe_reserved.cleanup_candidate_bytes, 10);
    assert_eq!(cleanup_probe_reserved.active_leases, 0);
    assert_eq!(cleanup_probe_reserved.unknown_size_records, 0);
    assert!(
        !diagnostics
            .summary
            .cleanup_purpose_state_summaries
            .iter()
            .any(|summary| {
                summary.purpose == StagingPurpose::ProbeInput
                    && summary.state == StagingState::Failed
            })
    );
    assert_eq!(diagnostics.summary.vfs_cache.object_count, 1);
    assert_eq!(diagnostics.summary.vfs_cache.failure_count, 1);
    assert_eq!(
        diagnostics.summary.vfs_cache.last_failure_at_ms,
        Some(2_000)
    );
    let repair = diagnostics
        .summary
        .vfs_cache
        .repair
        .as_ref()
        .expect("vfs cache repair preview");
    assert_eq!(
        repair.classification,
        nako_api::admin::AdminVfsCacheRepairClassification::UnknownFailure
    );
    assert_eq!(
        repair.recommended_action,
        nako_api::admin::AdminVfsCacheRepairAction::InspectFailure
    );
    assert_eq!(repair.operation, Some(VfsCacheOperation::Stat));
    assert_eq!(repair.failure_class, None);
    assert!(!repair.retryable);
    assert_eq!(repair.failed_at_ms, Some(2_000));
    assert_eq!(repair.failure_count, Some(1));
    assert_eq!(repair.safe_message.as_deref(), Some("storage failure"));
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
    assert!(!body.contains("failed-etag-secret"));
    assert!(!body.contains("failed-fingerprint-secret"));
    assert!(!body.contains("cache-etag-secret"));
    assert!(!body.contains("cache-fingerprint-secret"));
    assert!(!body.contains("cache failed at secret path"));
    assert!(!body.contains("failed at local secret path"));
    assert!(!body.contains("backend raw error"));
    assert!(!body.contains("token=secret"));
}

#[tokio::test]
async fn admin_v1_vfs_cache_refresh_action_refreshes_latest_failure_and_redacts_target() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    fs::create_dir_all(root.join("Movies")).unwrap();
    fs::write(root.join("Movies").join("Demo.mkv"), b"demo").unwrap();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            config.libraries[0].clone(),
            Arc::new(nako_vfs::CachedStorageBackend::with_options(
                nako_vfs::LocalFsBackend::new(&root).unwrap(),
                store.clone(),
                nako_vfs::VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    let backend_key = format!("library:{library_id}:local");
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(library_id, backend_key),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/storage/vfs-cache/repair/refresh-cache")
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
    let refresh: AdminVfsCacheRefreshResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(
        refresh.action,
        nako_api::admin::AdminVfsCacheRepairAction::RefreshCache
    );
    assert_eq!(refresh.operation, VfsCacheOperation::Stat);
    assert!(refresh.refreshed);
    assert_eq!(
        refresh.repair.classification,
        nako_api::admin::AdminVfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert_eq!(
        refresh.repair.safe_message.as_deref(),
        Some("storage backend unavailable")
    );
    let cached = store
        .get_vfs_cache_object("local:///Movies/Demo.mkv")
        .await
        .unwrap()
        .expect("refresh should update object cache");
    assert!(cached.fetched_at_ms >= 1_000);

    let staging_response = router
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
    let diagnostics = body_json::<AdminStorageStagingDiagnosticsResponse>(staging_response).await;

    assert!(diagnostics.summary.vfs_cache.repair.is_none());
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("Movies"));
    assert!(!body.contains("Demo.mkv"));
    assert!(!body.contains(&root.display().to_string()));
    assert!(!body.contains("secret-cache"));
}

#[tokio::test]
async fn admin_v1_vfs_cache_target_refresh_action_refreshes_selected_target_and_redacts_target() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    fs::create_dir_all(root.join("Movies").join("Private")).unwrap();
    fs::write(
        root.join("Movies").join("Private").join("Selected.mkv"),
        b"selected",
    )
    .unwrap();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            config.libraries[0].clone(),
            Arc::new(nako_vfs::CachedStorageBackend::with_options(
                nako_vfs::LocalFsBackend::new(&root).unwrap(),
                store.clone(),
                nako_vfs::VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Selected.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let targets: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    let target_ref = targets.targets[0].target_ref.clone();
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let refresh: AdminVfsCacheRefreshResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(
        refresh.action,
        nako_api::admin::AdminVfsCacheRepairAction::RefreshCache
    );
    assert_eq!(refresh.operation, VfsCacheOperation::Stat);
    assert!(refresh.refreshed);
    assert_eq!(
        refresh.repair.classification,
        nako_api::admin::AdminVfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert!(
        store
            .get_vfs_cache_object("local:///Movies/Private/Selected.mkv")
            .await
            .unwrap()
            .is_some()
    );
    let remaining: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    assert!(remaining.targets.is_empty());

    assert!(!body.contains("source_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("Private"));
    assert!(!body.contains("Selected.mkv"));
    assert!(!body.contains(&root.display().to_string()));
    assert!(!body.contains("secret-cache"));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_job_routes_enqueue_and_execute_without_payload_leaks() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    fs::create_dir_all(root.join("Movies").join("Private")).unwrap();
    fs::write(
        root.join("Movies").join("Private").join("AdminJob.mkv"),
        b"repair-job",
    )
    .unwrap();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            config.libraries[0].clone(),
            Arc::new(nako_vfs::CachedStorageBackend::with_options(
                nako_vfs::LocalFsBackend::new(&root).unwrap(),
                store.clone(),
                nako_vfs::VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/AdminJob.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let targets: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    let target_ref = targets.targets[0].target_ref.clone();

    let enqueue_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest {
            priority: Some(AdminJobPriority::High),
        },
    )
    .await;
    let enqueue_status = enqueue_response.status();
    let enqueue_body = response_text(enqueue_response).await;
    assert_eq!(enqueue_status, StatusCode::ACCEPTED, "{enqueue_body}");
    let enqueue: AdminVfsCacheRepairEnqueueResponse = serde_json::from_str(&enqueue_body).unwrap();
    let persisted = store
        .get_job(enqueue.job.id)
        .await
        .unwrap()
        .expect("enqueued repair job");

    assert_eq!(enqueue.outcome, AdminVfsCacheRepairEnqueueOutcome::Enqueued);
    assert_eq!(enqueue.job.kind, JobKind::VfsCacheRepair);
    assert_eq!(enqueue.job.status, JobStatus::Queued);
    assert_eq!(
        enqueue.job.resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(enqueue.job.library_id, Some(library_id));
    assert_eq!(enqueue.job.source_id, None);
    assert!(enqueue.job.has_input);
    assert!(!enqueue.job.has_summary);
    assert!(!enqueue.job.has_error);
    let enqueue_diagnostics = enqueue
        .job
        .diagnostics
        .as_ref()
        .and_then(|diagnostics| diagnostics.vfs_cache_repair.as_ref())
        .expect("queued repair job diagnostics");
    assert_eq!(
        enqueue_diagnostics.status,
        AdminVfsCacheRepairJobDiagnosticStatus::Pending
    );
    assert!(enqueue_diagnostics.summary.is_none());
    assert!(enqueue_diagnostics.failure.is_none());
    assert_eq!(persisted.kind, JobKind::VfsCacheRepair);
    assert_eq!(
        persisted.resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(persisted.priority, JobPriority::High);
    assert_eq!(persisted.library_id, Some(library_id));
    assert_eq!(persisted.source_id, None);
    assert_vfs_cache_repair_admin_response_redacted(&enqueue_body);
    assert_vfs_cache_repair_job_input_redacted(persisted.input_json.as_deref().unwrap());

    let duplicate_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest { priority: None },
    )
    .await;
    let duplicate_status = duplicate_response.status();
    let duplicate_body = response_text(duplicate_response).await;
    assert_eq!(duplicate_status, StatusCode::ACCEPTED, "{duplicate_body}");
    let duplicate: AdminVfsCacheRepairEnqueueResponse =
        serde_json::from_str(&duplicate_body).unwrap();
    assert_eq!(
        duplicate.outcome,
        AdminVfsCacheRepairEnqueueOutcome::AlreadyQueued
    );
    assert_eq!(duplicate.job.id, enqueue.job.id);
    assert_vfs_cache_repair_admin_response_redacted(&duplicate_body);

    let execute_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/storage/vfs-cache/repair/jobs/{}/execute",
                    enqueue.job.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let execute_status = execute_response.status();
    let execute_body = response_text(execute_response).await;
    assert_eq!(execute_status, StatusCode::OK, "{execute_body}");
    let execute: AdminVfsCacheRepairExecuteResponse = serde_json::from_str(&execute_body).unwrap();

    assert_eq!(execute.job.id, enqueue.job.id);
    assert_eq!(execute.job.kind, JobKind::VfsCacheRepair);
    assert_eq!(execute.job.status, JobStatus::Succeeded);
    assert_eq!(
        execute.job.resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(execute.job.library_id, Some(library_id));
    assert_eq!(execute.job.source_id, None);
    assert!(execute.job.has_input);
    assert!(execute.job.has_summary);
    assert!(!execute.job.has_error);
    let execute_diagnostics = execute
        .job
        .diagnostics
        .as_ref()
        .and_then(|diagnostics| diagnostics.vfs_cache_repair.as_ref())
        .expect("completed repair job diagnostics");
    assert_eq!(
        execute_diagnostics.status,
        AdminVfsCacheRepairJobDiagnosticStatus::SummaryAvailable
    );
    assert_eq!(execute_diagnostics.summary.as_ref(), Some(&execute.summary));
    assert!(execute_diagnostics.failure.is_none());
    assert_eq!(
        execute.summary.action,
        nako_api::admin::AdminVfsCacheRepairAction::RefreshCache
    );
    assert_eq!(execute.summary.source_scheme, "local");
    assert_eq!(execute.summary.operation, VfsCacheOperation::Stat);
    assert_eq!(
        execute.summary.classification,
        nako_api::admin::AdminVfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert_eq!(
        execute.summary.failure_class,
        Some(StorageFailureClass::Unavailable)
    );
    assert_eq!(execute.summary.failed_at_ms, 1_000);
    assert_eq!(execute.summary.failure_count, 1);
    assert_eq!(
        execute.summary.refreshed_cache_state,
        Some(AdminVfsCacheRepairCacheState::Fresh)
    );
    assert!(
        store
            .get_vfs_cache_object("local:///Movies/Private/AdminJob.mkv")
            .await
            .unwrap()
            .is_some()
    );
    let completed = store
        .get_job(enqueue.job.id)
        .await
        .unwrap()
        .expect("completed repair job");
    assert_eq!(completed.status, JobStatus::Succeeded);
    assert!(completed.summary_json.is_some());
    assert_vfs_cache_repair_admin_response_redacted(&execute_body);
    assert_vfs_cache_repair_job_summary_redacted(completed.summary_json.as_deref().unwrap());
    assert!(!enqueue_body.contains(&root.display().to_string()));
    assert!(!duplicate_body.contains(&root.display().to_string()));
    assert!(!execute_body.contains(&root.display().to_string()));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_automation_plan_and_enqueue_without_payload_leaks() {
    let (_temp, router, store, library_id, _root) = vfs_cache_repair_retry_http_fixture().await;

    let plan_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/storage/vfs-cache/repair/automation/plan",
        &AdminVfsCacheRepairAutomationPolicyRequest { enabled: true },
    )
    .await;
    let plan_status = plan_response.status();
    let plan_body = response_text(plan_response).await;
    assert_eq!(plan_status, StatusCode::OK, "{plan_body}");
    let plan: AdminVfsCacheRepairAutomationPlanResponse = serde_json::from_str(&plan_body).unwrap();
    let jobs_before = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(plan.policy.enabled);
    assert_eq!(plan.policy.total_unresolved_targets, 1);
    assert_eq!(plan.policy.eligible_targets.len(), 1);
    assert!(plan.policy.blocked_targets.is_empty());
    assert!(plan.policy.boundary.reads_repair_targets);
    assert!(plan.policy.boundary.may_start_durable_jobs);
    assert!(!plan.policy.boundary.refreshes_vfs_cache);
    assert!(!plan.policy.boundary.changes_backend_configuration);
    assert!(!plan.policy.boundary.deletes_cache_entries);
    assert!(!plan.policy.boundary.writes_library_files);
    assert!(jobs_before.is_empty());
    assert_vfs_cache_repair_admin_response_redacted(&plan_body);

    let enqueue_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/storage/vfs-cache/repair/automation/jobs",
        &AdminVfsCacheRepairAutomationEnqueueRequest {
            enabled: true,
            priority: Some(AdminJobPriority::High),
        },
    )
    .await;
    let enqueue_status = enqueue_response.status();
    let enqueue_body = response_text(enqueue_response).await;
    assert_eq!(enqueue_status, StatusCode::ACCEPTED, "{enqueue_body}");
    let enqueue: AdminVfsCacheRepairAutomationEnqueueResponse =
        serde_json::from_str(&enqueue_body).unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(enqueue.enqueued_count, 1);
    assert_eq!(enqueue.already_queued_count, 0);
    assert_eq!(enqueue.jobs.len(), 1);
    assert_eq!(
        enqueue.jobs[0].outcome,
        AdminVfsCacheRepairEnqueueOutcome::Enqueued
    );
    assert_eq!(enqueue.jobs[0].status, JobStatus::Queued);
    assert_eq!(enqueue.jobs[0].priority, JobPriority::High);
    assert_eq!(
        enqueue.jobs[0].resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(enqueue.jobs[0].library_id, Some(library_id));
    assert_eq!(enqueue.jobs[0].source_id, None);
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, enqueue.jobs[0].job_id);
    assert_eq!(jobs[0].kind, JobKind::VfsCacheRepair);
    assert_eq!(jobs[0].priority, JobPriority::High);
    assert_vfs_cache_repair_admin_response_redacted(&enqueue_body);
    assert_vfs_cache_repair_job_input_redacted(jobs[0].input_json.as_deref().unwrap());

    let duplicate_response = response_body_json(
        &router,
        Method::POST,
        "/admin/v1/storage/vfs-cache/repair/automation/jobs",
        &AdminVfsCacheRepairAutomationEnqueueRequest {
            enabled: true,
            priority: None,
        },
    )
    .await;
    let duplicate_status = duplicate_response.status();
    let duplicate_body = response_text(duplicate_response).await;
    assert_eq!(duplicate_status, StatusCode::ACCEPTED, "{duplicate_body}");
    let duplicate: AdminVfsCacheRepairAutomationEnqueueResponse =
        serde_json::from_str(&duplicate_body).unwrap();
    assert_eq!(duplicate.enqueued_count, 0);
    assert_eq!(duplicate.already_queued_count, 1);
    assert_eq!(
        duplicate.jobs[0].outcome,
        AdminVfsCacheRepairEnqueueOutcome::AlreadyQueued
    );
    assert_eq!(duplicate.jobs[0].job_id, enqueue.jobs[0].job_id);
    assert_vfs_cache_repair_admin_response_redacted(&duplicate_body);
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_retry_requeues_failed_job_without_payload_leaks() {
    let (_temp, router, store, library_id, root) = vfs_cache_repair_retry_http_fixture().await;
    let targets: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    let target_ref = targets.targets[0].target_ref.clone();
    let enqueue_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest {
            priority: Some(AdminJobPriority::High),
        },
    )
    .await;
    let enqueue_body = response_text(enqueue_response).await;
    let source_job: AdminVfsCacheRepairEnqueueResponse =
        serde_json::from_str(&enqueue_body).unwrap();
    let source_persisted = store
        .get_job(source_job.job.id)
        .await
        .unwrap()
        .expect("enqueued source repair job");
    let source_input: VfsCacheRepairJobInput =
        serde_json::from_str(source_persisted.input_json.as_deref().unwrap()).unwrap();
    store.start_job(source_job.job.id).await.unwrap();
    let failed = store
        .fail_job(
            source_job.job.id,
            "VFS repair failed at local:///Movies/Private/AdminJob.mkv with token=secret etag cache-etag-secret fingerprint cache-fingerprint-secret"
                .to_owned(),
        )
        .await
        .unwrap();

    let failed_jobs_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/jobs?kind=vfs_cache_repair&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(failed_jobs_response.status(), StatusCode::OK);
    let failed_jobs_body = response_text(failed_jobs_response).await;
    let failed_jobs: AdminJobListResponse = serde_json::from_str(&failed_jobs_body).unwrap();
    let failed_job = failed_jobs
        .jobs
        .iter()
        .find(|job| job.id == failed.id)
        .expect("failed repair job in Admin Jobs list");
    assert_eq!(failed_job.kind, JobKind::VfsCacheRepair);
    assert_eq!(failed_job.status, JobStatus::Failed);
    assert!(failed_job.has_error);
    let failed_diagnostics = failed_job
        .diagnostics
        .as_ref()
        .and_then(|diagnostics| diagnostics.vfs_cache_repair.as_ref())
        .expect("failed repair job diagnostics");
    assert_eq!(
        failed_diagnostics.status,
        AdminVfsCacheRepairJobDiagnosticStatus::Failed
    );
    assert!(failed_diagnostics.summary.is_none());
    let failed_diagnostic = failed_diagnostics
        .failure
        .as_ref()
        .expect("failed repair job failure diagnostic");
    assert_eq!(failed_diagnostic.status, JobStatus::Failed);
    assert_eq!(
        failed_diagnostic.failure_class,
        StorageFailureClass::Unknown
    );
    assert_eq!(failed_diagnostic.safe_message, "storage failure");
    assert!(!failed_diagnostic.retryable);
    assert_vfs_cache_repair_admin_response_redacted(&failed_jobs_body);

    let retry_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/storage/vfs-cache/repair/jobs/{}/retry",
            failed.id
        ),
        &AdminVfsCacheRepairRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: Some("9999-01-01T08:00:00+08:00".to_owned()),
        },
    )
    .await;
    let retry_status = retry_response.status();
    let retry_body = response_text(retry_response).await;
    assert_eq!(retry_status, StatusCode::ACCEPTED, "{retry_body}");
    let retry_job: AdminJobListItem = serde_json::from_str(&retry_body).unwrap();
    let persisted_retry = store
        .get_job(retry_job.id)
        .await
        .unwrap()
        .expect("persisted retry job");
    let retry_input: VfsCacheRepairJobInput =
        serde_json::from_str(persisted_retry.input_json.as_deref().unwrap()).unwrap();

    assert_ne!(retry_job.id, failed.id);
    assert_eq!(retry_job.kind, JobKind::VfsCacheRepair);
    assert_eq!(retry_job.status, JobStatus::Queued);
    assert_eq!(
        retry_job.resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(retry_job.library_id, Some(library_id));
    assert_eq!(retry_job.source_id, None);
    assert!(retry_job.has_input);
    assert!(!retry_job.has_summary);
    assert!(!retry_job.has_error);
    assert_eq!(retry_job.priority, JobPriority::High);
    assert_eq!(retry_job.retry_of_job_id, Some(failed.id));
    assert_eq!(retry_job.attempt, 2);
    assert_eq!(retry_job.max_attempts, 3);
    assert_eq!(
        retry_job.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    let retry_diagnostics = retry_job
        .diagnostics
        .as_ref()
        .and_then(|diagnostics| diagnostics.vfs_cache_repair.as_ref())
        .expect("retry repair job diagnostics");
    assert_eq!(
        retry_diagnostics.status,
        AdminVfsCacheRepairJobDiagnosticStatus::Pending
    );
    assert!(retry_diagnostics.summary.is_none());
    assert!(retry_diagnostics.failure.is_none());
    assert_eq!(persisted_retry.retry_of_job_id, Some(failed.id));
    assert_eq!(persisted_retry.attempt, 2);
    assert_eq!(persisted_retry.max_attempts, 3);
    assert_eq!(
        persisted_retry.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    assert_eq!(persisted_retry.priority, JobPriority::High);
    assert_eq!(persisted_retry.library_id, Some(library_id));
    assert_eq!(persisted_retry.source_id, None);
    assert_eq!(persisted_retry.input_json, failed.input_json);
    assert_eq!(retry_input, source_input);
    assert_vfs_cache_repair_admin_response_redacted(&retry_body);
    assert_vfs_cache_repair_job_input_redacted(persisted_retry.input_json.as_deref().unwrap());
    assert!(!retry_body.contains(&root.display().to_string()));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_retry_rejects_invalid_states_without_leaks() {
    let (_temp, router, store, library_id, _root) = vfs_cache_repair_retry_http_fixture().await;
    let targets: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    let target_ref = targets.targets[0].target_ref.clone();
    let enqueue_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest { priority: None },
    )
    .await;
    let enqueue_body = response_text(enqueue_response).await;
    let source_job: AdminVfsCacheRepairEnqueueResponse =
        serde_json::from_str(&enqueue_body).unwrap();
    let source_persisted = store
        .get_job(source_job.job.id)
        .await
        .unwrap()
        .expect("enqueued repair job");
    let input: VfsCacheRepairJobInput =
        serde_json::from_str(source_persisted.input_json.as_deref().unwrap()).unwrap();

    let not_failed_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/storage/vfs-cache/repair/jobs/{}/retry",
            source_job.job.id
        ),
        &AdminVfsCacheRepairRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: None,
        },
    )
    .await;
    let not_failed_status = not_failed_response.status();
    let not_failed_body = response_text(not_failed_response).await;
    assert_eq!(not_failed_status, StatusCode::CONFLICT, "{not_failed_body}");
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&not_failed_body)
            .unwrap()
            .message,
        "conflict: only failed VFS cache repair jobs can be retried"
    );
    assert_vfs_cache_repair_admin_response_redacted(&not_failed_body);

    let invalid_timestamp_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/storage/vfs-cache/repair/jobs/{}/retry",
            source_job.job.id
        ),
        &AdminVfsCacheRepairRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: Some("local:///Movies/Private/not-a-time?token=secret".to_owned()),
        },
    )
    .await;
    let invalid_timestamp_status = invalid_timestamp_response.status();
    let invalid_timestamp_body = response_text(invalid_timestamp_response).await;
    assert_eq!(
        invalid_timestamp_status,
        StatusCode::BAD_REQUEST,
        "{invalid_timestamp_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&invalid_timestamp_body)
            .unwrap()
            .message,
        "invalid input: VFS cache repair retry next_attempt_at must be an RFC3339 timestamp"
    );
    assert_vfs_cache_repair_admin_response_redacted(&invalid_timestamp_body);
    assert!(!invalid_timestamp_body.contains("not-a-time"));

    let wrong_kind = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: None,
            input_json: source_persisted.input_json.clone(),
        })
        .await
        .unwrap();
    store.start_job(wrong_kind.id).await.unwrap();
    let wrong_kind = store
        .fail_job(
            wrong_kind.id,
            "wrong kind failed at local:///Movies/Private/AdminJob.mkv token=secret".to_owned(),
        )
        .await
        .unwrap();
    let wrong_kind_response = response_body_json(
        &router,
        Method::POST,
        &format!(
            "/admin/v1/storage/vfs-cache/repair/jobs/{}/retry",
            wrong_kind.id
        ),
        &AdminVfsCacheRepairRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: None,
        },
    )
    .await;
    let wrong_kind_status = wrong_kind_response.status();
    let wrong_kind_body = response_text(wrong_kind_response).await;
    assert_eq!(
        wrong_kind_status,
        StatusCode::BAD_REQUEST,
        "{wrong_kind_body}"
    );
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&wrong_kind_body)
            .unwrap()
            .message,
        "invalid input: job is not a VFS cache repair job"
    );
    assert_vfs_cache_repair_admin_response_redacted(&wrong_kind_body);

    let stale_input = VfsCacheRepairJobInput::new(
        input.action,
        input.source_scheme.clone(),
        input.operation,
        input.failed_at_ms + 1,
        input.failure_count,
        input.uri_digest.clone(),
        input.authority.clone(),
    )
    .unwrap();
    let stale = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::VfsCacheRepair,
            resource_class: VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: None,
            input_json: Some(serde_json::to_string(&stale_input).unwrap()),
        })
        .await
        .unwrap();
    store.start_job(stale.id).await.unwrap();
    let stale = store
        .fail_job(
            stale.id,
            "stale repair failed at local:///Movies/Private/AdminJob.mkv token=secret".to_owned(),
        )
        .await
        .unwrap();
    let stale_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/jobs/{}/retry", stale.id),
        &AdminVfsCacheRepairRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: None,
        },
    )
    .await;
    let stale_status = stale_response.status();
    let stale_body = response_text(stale_response).await;
    assert_eq!(stale_status, StatusCode::NOT_FOUND, "{stale_body}");
    assert_eq!(
        serde_json::from_str::<ErrorResponse>(&stale_body)
            .unwrap()
            .message,
        "not found: vfs_cache_repair_target job_input"
    );
    assert_vfs_cache_repair_admin_response_redacted(&stale_body);

    let retry_jobs = store
        .list_jobs(
            nako_core::JobListFilter {
                status: Some(JobStatus::Queued),
                kind: Some(JobKind::VfsCacheRepair),
                resource_class: Some(VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned()),
                library_id: Some(library_id),
                source_id: None,
            },
            PageRequest::new(PageRequest::MAX_LIMIT, 0),
        )
        .await
        .unwrap()
        .into_iter()
        .filter(|job| job.retry_of_job_id.is_some())
        .collect::<Vec<_>>();
    assert!(retry_jobs.is_empty());
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_target_enqueue_rejects_non_refresh_and_unknown_refs_safely() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Config.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage permission failure at local:///Movies/Private/Config.mkv".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let targets: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    let target_ref = targets.targets[0].target_ref.clone();
    let non_refresh_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest {
            priority: Some(AdminJobPriority::High),
        },
    )
    .await;
    let non_refresh_status = non_refresh_response.status();
    let non_refresh_body = response_text(non_refresh_response).await;
    assert_eq!(
        non_refresh_status,
        StatusCode::BAD_REQUEST,
        "{non_refresh_body}"
    );
    let error: ErrorResponse = serde_json::from_str(&non_refresh_body).unwrap();
    assert_eq!(
        error.message,
        "invalid input: selected VFS cache repair target diagnostic does not recommend durable refresh_cache"
    );
    assert_vfs_cache_repair_admin_response_redacted(&non_refresh_body);
    assert!(!non_refresh_body.contains(&target_ref));

    let unsafe_ref = "not_a_target_ref_token_secret";
    let unsafe_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{unsafe_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest { priority: None },
    )
    .await;
    let unsafe_status = unsafe_response.status();
    let unsafe_body = response_text(unsafe_response).await;
    assert_eq!(unsafe_status, StatusCode::NOT_FOUND, "{unsafe_body}");
    assert!(!unsafe_body.contains(unsafe_ref));
    assert!(!unsafe_body.contains("token_secret"));

    let unknown_ref = "vfsrt_00000000000000000000000000000000";
    let unknown_response = response_body_json(
        &router,
        Method::POST,
        &format!("/admin/v1/storage/vfs-cache/repair/targets/{unknown_ref}/jobs"),
        &AdminVfsCacheRepairEnqueueRequest { priority: None },
    )
    .await;
    let unknown_status = unknown_response.status();
    let unknown_body = response_text(unknown_response).await;
    assert_eq!(unknown_status, StatusCode::NOT_FOUND, "{unknown_body}");
    assert!(!unknown_body.contains(unknown_ref));
    assert_vfs_cache_repair_admin_response_redacted(&unknown_body);

    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert!(jobs.is_empty());
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_action_plan_reports_executable_refresh_and_redacts_target() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    fs::create_dir_all(root.join("Movies")).unwrap();
    fs::write(root.join("Movies").join("Demo.mkv"), b"demo").unwrap();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            config.libraries[0].clone(),
            Arc::new(nako_vfs::CachedStorageBackend::with_options(
                nako_vfs::LocalFsBackend::new(&root).unwrap(),
                store.clone(),
                nako_vfs::VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/vfs-cache/repair/action-plan")
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
    let plan: AdminVfsCacheRepairActionPlanResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(
        plan.plan.status,
        AdminVfsCacheRepairActionPlanStatus::Executable
    );
    assert_eq!(
        plan.plan.action,
        nako_api::admin::AdminVfsCacheRepairAction::RefreshCache
    );
    assert_eq!(
        plan.plan.readiness.status,
        AdminVfsCacheRepairActionPlanStatus::Executable
    );
    assert!(plan.plan.readiness.api_executable);
    assert_eq!(
        plan.plan.readiness.reasons,
        vec![AdminVfsCacheRepairActionPlanReason::RefreshCacheExecutable]
    );
    assert!(plan.plan.boundary.refreshes_vfs_cache);
    assert!(!plan.plan.boundary.changes_backend_configuration);
    assert!(!plan.plan.boundary.requires_manual_failure_inspection);
    assert!(!plan.plan.boundary.deletes_cache_entries);
    assert!(!plan.plan.boundary.writes_library_files);
    assert!(!plan.plan.boundary.starts_durable_job);
    let executable = plan.plan.executable_action.as_ref().expect("refresh route");
    assert_eq!(executable.method, "POST");
    assert_eq!(executable.route_key, "storageVfsCacheRepairRefreshCache");
    assert_eq!(
        executable.route_path,
        "/admin/v1/storage/vfs-cache/repair/refresh-cache"
    );
    let repair = plan.plan.repair.as_ref().expect("repair diagnostic");
    assert_eq!(
        repair.classification,
        nako_api::admin::AdminVfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert_eq!(repair.operation, Some(VfsCacheOperation::Stat));
    assert_eq!(
        repair.safe_message.as_deref(),
        Some("storage backend unavailable")
    );
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("Movies"));
    assert!(!body.contains("Demo.mkv"));
    assert!(!body.contains(&root.display().to_string()));
    assert!(!body.contains("secret-cache"));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_targets_list_and_preview_redact_targets_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 3_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Config.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 2_000,
            error: "storage permission failure".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Scheme.mkv".to_owned(),
            scheme: "local:///scheme-secret".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let list_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/vfs-cache/repair/targets?limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = list_response.status();
    let list_body = response_text(list_response).await;
    assert_eq!(status, StatusCode::OK, "{list_body}");
    let list: AdminVfsCacheRepairTargetListResponse = serde_json::from_str(&list_body).unwrap();

    assert_eq!(list.page.limit, 5);
    assert_eq!(list.page.offset, 0);
    assert_eq!(list.page.returned, 3);
    assert_eq!(list.targets.len(), 3);
    assert_eq!(list.targets[0].scheme, "local");
    assert_eq!(list.targets[0].operation, VfsCacheOperation::Stat);
    assert_eq!(list.targets[0].failed_at_ms, 3_000);
    assert_eq!(list.targets[0].failure_count, 1);
    assert!(list.targets[0].target_ref.starts_with("vfsrt_"));
    assert_ne!(list.targets[0].target_ref, list.targets[1].target_ref);
    assert_eq!(list.targets[2].scheme, "unknown");
    assert_eq!(
        list.targets[0].failure_class,
        Some(StorageFailureClass::Unavailable)
    );
    assert_eq!(
        list.targets[0].recommended_action,
        nako_api::admin::AdminVfsCacheRepairAction::RefreshCache
    );

    let plan_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/vfs-cache/repair/remediation-plan")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = plan_response.status();
    let plan_body = response_text(plan_response).await;
    assert_eq!(status, StatusCode::OK, "{plan_body}");
    let remediation: AdminVfsCacheRepairRemediationPlanResponse =
        serde_json::from_str(&plan_body).unwrap();

    assert_eq!(remediation.total_unresolved_targets, 3);
    assert!(remediation.boundary.read_only);
    assert!(!remediation.boundary.refreshes_vfs_cache);
    assert!(!remediation.boundary.changes_backend_configuration);
    assert!(!remediation.boundary.deletes_cache_entries);
    assert!(!remediation.boundary.writes_library_files);
    assert!(!remediation.boundary.starts_durable_job);
    let refresh_group = remediation
        .action_groups
        .iter()
        .find(|group| group.action == nako_api::admin::AdminVfsCacheRepairAction::RefreshCache)
        .expect("refresh group");
    assert_eq!(refresh_group.count, 2);
    assert_eq!(
        refresh_group.status,
        AdminVfsCacheRepairActionPlanStatus::Executable
    );
    assert!(refresh_group.readiness.api_executable);
    assert!(refresh_group.boundary.refreshes_vfs_cache);
    assert_eq!(refresh_group.sample_targets.len(), 2);
    let executable = refresh_group
        .executable_action
        .as_ref()
        .expect("target refresh executable route");
    assert_eq!(executable.method, "POST");
    assert_eq!(
        executable.route_key,
        "storageVfsCacheRepairTargetRefreshCache"
    );
    assert_eq!(
        executable.route_path,
        "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
    );
    let config_group = remediation
        .action_groups
        .iter()
        .find(|group| {
            group.action == nako_api::admin::AdminVfsCacheRepairAction::FixBackendConfiguration
        })
        .expect("backend configuration group");
    assert_eq!(config_group.count, 1);
    assert_eq!(
        config_group.status,
        AdminVfsCacheRepairActionPlanStatus::PlanOnly
    );
    assert!(!config_group.readiness.api_executable);
    assert!(config_group.boundary.changes_backend_configuration);
    assert!(config_group.executable_action.is_none());
    assert_eq!(
        remediation
            .classification_counts
            .iter()
            .find(|count| {
                count.classification
                    == nako_api::admin::AdminVfsCacheRepairClassification::RetryableRefreshFailure
            })
            .expect("retryable count")
            .count,
        2
    );
    assert_eq!(
        remediation
            .classification_counts
            .iter()
            .find(|count| {
                count.classification
                    == nako_api::admin::AdminVfsCacheRepairClassification::OperatorActionRequired
            })
            .expect("operator count")
            .count,
        1
    );
    assert_eq!(
        store
            .list_vfs_cache_failures(PageRequest::new(10, 0))
            .await
            .unwrap()
            .len(),
        3
    );

    let preview_path = format!(
        "/admin/v1/storage/vfs-cache/repair/targets/{}/preview",
        list.targets[0].target_ref
    );
    let preview_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(preview_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = preview_response.status();
    let preview_body = response_text(preview_response).await;
    assert_eq!(status, StatusCode::OK, "{preview_body}");
    let preview: AdminVfsCacheRepairTargetPreviewResponse =
        serde_json::from_str(&preview_body).unwrap();

    assert_eq!(preview.target, list.targets[0]);
    assert_eq!(
        preview.plan.status,
        AdminVfsCacheRepairActionPlanStatus::Executable
    );
    assert_eq!(
        preview.plan.action,
        nako_api::admin::AdminVfsCacheRepairAction::RefreshCache
    );
    assert!(preview.plan.readiness.api_executable);
    assert_eq!(
        preview.plan.readiness.reasons,
        vec![nako_api::admin::AdminVfsCacheRepairActionPlanReason::RefreshCacheExecutable]
    );
    assert!(preview.plan.boundary.refreshes_vfs_cache);
    let executable = preview
        .plan
        .executable_action
        .as_ref()
        .expect("target refresh route");
    assert_eq!(executable.method, "POST");
    assert_eq!(
        executable.route_key,
        "storageVfsCacheRepairTargetRefreshCache"
    );
    assert_eq!(
        executable.route_path,
        "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
    );
    assert!(preview.plan.repair.is_some());
    assert!(
        store
            .get_vfs_cache_object("local:///Movies/Private/Demo.mkv")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .list_vfs_cache_failures(PageRequest::new(10, 0))
            .await
            .unwrap()
            .len(),
        3
    );

    let body = format!("{list_body}{plan_body}{preview_body}");
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("source_locator"));
    assert!(!body.contains("cache_uri"));
    assert!(!body.contains("storage_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("Private"));
    assert!(!body.contains("Demo.mkv"));
    assert!(!body.contains("Config.mkv"));
    assert!(!body.contains("Scheme.mkv"));
    assert!(!body.contains("scheme-secret"));
    assert!(!body.contains(&root.display().to_string()));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("token=secret"));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_targets_page_over_unresolved_targets() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.clone(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Resolved.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 4_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();
    store
        .upsert_vfs_cache_object(&VfsCachedObject {
            uri: "local:///Movies/Private/Resolved.mkv".to_owned(),
            scheme: "local".to_owned(),
            kind: VfsCachedObjectKind::File,
            len: Some(42),
            modified_at: None,
            etag: Some("resolved-etag-secret".to_owned()),
            fingerprint: Some("resolved-fingerprint-secret".to_owned()),
            capabilities_bits: 0,
            fetched_at_ms: 5_000,
            fresh_until_ms: 6_000,
        })
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Unresolved.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 3_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/vfs-cache/repair/targets?limit=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = response_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list: AdminVfsCacheRepairTargetListResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(list.page.limit, 1);
    assert_eq!(list.page.offset, 0);
    assert_eq!(list.page.returned, 1);
    assert_eq!(list.targets.len(), 1);
    assert_eq!(list.targets[0].failed_at_ms, 3_000);

    let preview_path = format!(
        "/admin/v1/storage/vfs-cache/repair/targets/{}/preview",
        list.targets[0].target_ref
    );
    let preview_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(preview_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let preview_status = preview_response.status();
    let preview_body = response_text(preview_response).await;
    assert_eq!(preview_status, StatusCode::OK, "{preview_body}");
    let preview: AdminVfsCacheRepairTargetPreviewResponse =
        serde_json::from_str(&preview_body).unwrap();
    assert_eq!(preview.target, list.targets[0]);

    let combined_body = format!("{body}{preview_body}");
    assert!(!combined_body.contains("local:///"));
    assert!(!combined_body.contains("Resolved.mkv"));
    assert!(!combined_body.contains("Unresolved.mkv"));
    assert!(!combined_body.contains("resolved-etag-secret"));
    assert!(!combined_body.contains("resolved-fingerprint-secret"));
    assert!(!combined_body.contains(&root.display().to_string()));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_target_preview_rejects_stale_and_unknown_refs_safely() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let root = temp.path().join("movies");
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Private/Stale.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let router = build_router(app);
    let list: AdminVfsCacheRepairTargetListResponse = request_json(
        &router,
        Method::GET,
        "/admin/v1/storage/vfs-cache/repair/targets",
    )
    .await;
    let stale_ref = list.targets[0].target_ref.clone();

    store
        .upsert_vfs_cache_object(&VfsCachedObject {
            uri: "local:///Movies/Private/Stale.mkv".to_owned(),
            scheme: "local".to_owned(),
            kind: VfsCachedObjectKind::File,
            len: Some(42),
            modified_at: None,
            etag: Some("cache-etag-secret".to_owned()),
            fingerprint: Some("cache-fingerprint-secret".to_owned()),
            capabilities_bits: 0,
            fetched_at_ms: 2_000,
            fresh_until_ms: 3_000,
        })
        .await
        .unwrap();

    let stale_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/storage/vfs-cache/repair/targets/{stale_ref}/preview"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stale_response.status(), StatusCode::NOT_FOUND);
    let stale_body = response_text(stale_response).await;
    assert!(!stale_body.contains(&stale_ref));
    assert!(!stale_body.contains("local:///"));
    assert!(!stale_body.contains("Private"));
    assert!(!stale_body.contains("Stale.mkv"));
    assert!(!stale_body.contains("cache-etag-secret"));
    assert!(!stale_body.contains("cache-fingerprint-secret"));

    let stale_refresh_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/storage/vfs-cache/repair/targets/{stale_ref}/refresh-cache"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stale_refresh_response.status(), StatusCode::NOT_FOUND);
    let stale_refresh_body = response_text(stale_refresh_response).await;
    assert!(!stale_refresh_body.contains(&stale_ref));
    assert!(!stale_refresh_body.contains("local:///"));
    assert!(!stale_refresh_body.contains("Private"));
    assert!(!stale_refresh_body.contains("Stale.mkv"));
    assert!(!stale_refresh_body.contains("cache-etag-secret"));
    assert!(!stale_refresh_body.contains("cache-fingerprint-secret"));

    let unsafe_ref = "not_a_target_ref_token_secret";
    let unknown_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/storage/vfs-cache/repair/targets/{unsafe_ref}/preview"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unknown_response.status(), StatusCode::NOT_FOUND);
    let unknown_body = response_text(unknown_response).await;
    assert!(!unknown_body.contains(unsafe_ref));
    assert!(!unknown_body.contains("token_secret"));

    let unknown_refresh_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/storage/vfs-cache/repair/targets/{unsafe_ref}/refresh-cache"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unknown_refresh_response.status(), StatusCode::NOT_FOUND);
    let unknown_refresh_body = response_text(unknown_refresh_response).await;
    assert!(!unknown_refresh_body.contains(unsafe_ref));
    assert!(!unknown_refresh_body.contains("token_secret"));
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_targets_reject_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "repair-target-viewer".to_owned(),
            display_name: "Repair Target Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "repair-target-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;

    for uri in [
        "/admin/v1/storage/vfs-cache/repair/remediation-plan",
        "/admin/v1/storage/vfs-cache/repair/targets",
        "/admin/v1/storage/vfs-cache/repair/targets/vfsrt_00000000000000000000000000000000/preview",
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", login.session.token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let error = body_json::<ErrorResponse>(response).await;
        assert_eq!(
            error.code,
            nako_api::public_client::ClientErrorCode::Forbidden.as_str()
        );
        assert_eq!(error.message, "administrator role is required");
    }
}

#[tokio::test]
async fn admin_v1_vfs_cache_refresh_action_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "refresh-viewer".to_owned(),
            display_name: "Refresh Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "refresh-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;

    let job_id = nako_core::JobId::new();
    let uris = [
        "/admin/v1/storage/vfs-cache/repair/refresh-cache".to_owned(),
        "/admin/v1/storage/vfs-cache/repair/targets/vfsrt_00000000000000000000000000000000/refresh-cache".to_owned(),
        "/admin/v1/storage/vfs-cache/repair/targets/vfsrt_00000000000000000000000000000000/jobs".to_owned(),
        format!("/admin/v1/storage/vfs-cache/repair/jobs/{job_id}/execute"),
        format!("/admin/v1/storage/vfs-cache/repair/jobs/{job_id}/retry"),
    ];

    for uri in uris {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(uri)
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", login.session.token),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let error = body_json::<ErrorResponse>(response).await;
        assert_eq!(
            error.code,
            nako_api::public_client::ClientErrorCode::Forbidden.as_str()
        );
        assert_eq!(error.message, "administrator role is required");
    }
}

#[tokio::test]
async fn admin_v1_vfs_cache_repair_action_plan_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "plan-viewer".to_owned(),
            display_name: "Plan Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "plan-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/vfs-cache/repair/action-plan")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(error.message, "administrator role is required");
}

#[tokio::test]
async fn admin_v1_storage_staging_attributes_policy_slices_without_raw_backend_data() {
    let temp = tempfile::tempdir().unwrap();
    let local_library_id = LibraryId::new();
    let remote_library_id = LibraryId::new();
    let local_root = temp.path().join("private-local-root");
    fs::create_dir_all(&local_root).unwrap();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig {
            max_bytes: 100,
            retention_ms: 8_888,
            cleanup_on_startup: true,
        },
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![
            LocalLibraryConfig {
                id: local_library_id,
                name: "Local Slice".to_owned(),
                root: local_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: remote_library_id,
                name: "Remote Slice".to_owned(),
                root: temp.path().join("unused-webdav-local-root"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: Some(crate::config::WebDavLibraryConfig {
                    root: "webdav:///Movies/PrivateRoot".to_owned(),
                    base_url: "https://dav.example.test/token-secret".to_owned(),
                    username: Some("webdav-user-secret".to_owned()),
                    password_env: Some("NAKO_WEBDAV_PASSWORD_SECRET".to_owned()),
                    timeout_ms: 5_000,
                    max_attempts: 1,
                }),
            },
        ],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: StagingManifestId::new(),
            attribution: StagingAttribution::attributed(local_library_id),
            source_uri: "local:///PrivateLocal/LocalSecret.mkv".to_owned(),
            source_scheme: "local".to_owned(),
            purpose: StagingPurpose::FfmpegInput,
            local_path: temp
                .path()
                .join("secret-cache")
                .join("local")
                .join("LocalSecret.mkv")
                .display()
                .to_string(),
            size_bytes: Some(20),
            etag: Some("local-etag-secret".to_owned()),
            fingerprint: Some("local-fingerprint-secret".to_owned()),
            state: StagingState::Ready,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(9_000),
            active_leases: 0,
            validation_error: None,
        })
        .await
        .unwrap();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: StagingManifestId::new(),
            attribution: StagingAttribution::attributed(remote_library_id),
            source_uri: "webdav:///Movies/PrivateRoot/RemoteSecret.mkv?token=raw-secret".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: temp
                .path()
                .join("secret-cache")
                .join("webdav")
                .join("RemoteSecret.mkv")
                .display()
                .to_string(),
            size_bytes: Some(95),
            etag: Some("webdav-etag-secret".to_owned()),
            fingerprint: Some("webdav-fingerprint-secret".to_owned()),
            state: StagingState::Reserved,
            created_at_ms: 2_000,
            updated_at_ms: 2_100,
            last_accessed_at_ms: 2_200,
            expires_at_ms: Some(10_000),
            active_leases: 1,
            validation_error: Some(
                "raw backend failure at webdav:///Movies/PrivateRoot/RemoteSecret.mkv".to_owned(),
            ),
        })
        .await
        .unwrap();

    let response = build_router(app)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/staging")
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

    assert_eq!(diagnostics.summary.used_manifest_bytes, 115);
    assert_eq!(
        diagnostics.summary.pressure.status,
        AdminStorageStagingPressureStatus::Exhausted
    );
    let local = diagnostics
        .summary
        .policy_slices
        .iter()
        .find(|slice| slice.library_id == Some(local_library_id))
        .expect("local staging policy slice");
    assert_eq!(
        local.backend_key,
        format!("library:{local_library_id}:local")
    );
    assert_eq!(
        local.backend_kind,
        Some(nako_api::admin::StorageBackendKind::Local)
    );
    assert_eq!(local.source_scheme, "local");
    assert_eq!(local.configured_max_bytes, 100);
    assert_eq!(local.used_manifest_bytes, 20);
    assert_eq!(
        local.pressure.status,
        AdminStorageStagingPressureStatus::Healthy
    );
    assert_eq!(local.pressure.ffmpeg_input_records, 1);
    let webdav = diagnostics
        .summary
        .policy_slices
        .iter()
        .find(|slice| slice.library_id == Some(remote_library_id))
        .expect("WebDAV staging policy slice");
    assert_eq!(
        webdav.backend_key,
        format!("library:{remote_library_id}:webdav")
    );
    assert_eq!(
        webdav.backend_kind,
        Some(nako_api::admin::StorageBackendKind::WebDav)
    );
    assert_eq!(webdav.source_scheme, "webdav");
    assert_eq!(webdav.configured_max_bytes, 100);
    assert_eq!(webdav.used_manifest_bytes, 95);
    assert_eq!(
        webdav.pressure.status,
        AdminStorageStagingPressureStatus::Critical
    );
    assert_eq!(webdav.pressure.in_flight_records, 1);
    assert_eq!(webdav.pressure.active_leases, 1);
    assert_eq!(webdav.pressure.probe_input_records, 1);

    assert!(!body.contains("source_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("webdav:///"));
    assert!(!body.contains("PrivateRoot"));
    assert!(!body.contains("LocalSecret"));
    assert!(!body.contains("RemoteSecret"));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("token-secret"));
    assert!(!body.contains("raw-secret"));
    assert!(!body.contains("webdav-user-secret"));
    assert!(!body.contains("NAKO_WEBDAV_PASSWORD_SECRET"));
    assert!(!body.contains("local-etag-secret"));
    assert!(!body.contains("local-fingerprint-secret"));
    assert!(!body.contains("webdav-etag-secret"));
    assert!(!body.contains("webdav-fingerprint-secret"));
    assert!(!body.contains("raw backend failure"));
}

#[tokio::test]
async fn admin_v1_storage_backends_lists_durable_health_and_resets_circuit() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let backend_key = format!("library:{library_id}:webdav");
    store
        .upsert_storage_backend_health(StorageBackendHealthRecord {
            backend_key: backend_key.clone(),
            library_id: Some(library_id),
            scheme: "webdav".to_owned(),
            status: StorageBackendHealthStatus::Unavailable,
            circuit_breaker_state: StorageCircuitBreakerState::Open,
            consecutive_failures: 2,
            last_success_at_ms: Some(500),
            last_failure_at_ms: Some(1_000),
            last_failure_class: Some(StorageFailureClass::Timeout),
            last_failure_safe_message: Some("storage backend timed out".to_owned()),
            circuit_opened_at_ms: Some(1_000),
            backoff_until_ms: Some(2_000),
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/storage/backends?limit=5")
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
    let diagnostics: AdminStorageBackendHealthDiagnosticsResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        diagnostics.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(diagnostics.backends.len(), 1);
    assert_eq!(diagnostics.page.limit, 5);
    assert_eq!(diagnostics.page.returned, 1);
    assert_eq!(diagnostics.backends[0].backend_key, backend_key);
    assert_eq!(diagnostics.backends[0].library_id, Some(library_id));
    assert_eq!(diagnostics.backends[0].scheme, "webdav");
    assert_eq!(
        diagnostics.backends[0].status,
        StorageBackendHealthStatus::Unavailable
    );
    assert_eq!(
        diagnostics.backends[0].circuit_breaker_state,
        StorageCircuitBreakerState::Open
    );
    assert_eq!(
        diagnostics.backends[0].last_failure_safe_message.as_deref(),
        Some("storage backend timed out")
    );
    assert!(!body.contains("root_uri"));
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("local_path"));
    assert!(!body.contains("webdav:///"));
    assert!(!body.contains("Private"));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains(&temp.path().display().to_string()));

    let reset_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/admin/v1/storage/backends/{backend_key}/circuit-breaker/reset"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let reset_status = reset_response.status();
    let reset_body = String::from_utf8(
        to_bytes(reset_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(reset_status, StatusCode::OK, "{reset_body}");
    let reset: AdminStorageBackendHealthResetResponse = serde_json::from_str(&reset_body).unwrap();

    assert_eq!(reset.backend.backend_key, backend_key);
    assert_eq!(reset.backend.status, StorageBackendHealthStatus::Healthy);
    assert_eq!(
        reset.backend.circuit_breaker_state,
        StorageCircuitBreakerState::Closed
    );
    assert_eq!(reset.backend.consecutive_failures, 0);
    assert_eq!(reset.backend.last_success_at_ms, Some(500));
    assert_eq!(reset.backend.last_failure_at_ms, None);
    assert_eq!(reset.backend.last_failure_class, None);
    assert_eq!(reset.backend.last_failure_safe_message, None);
    assert_eq!(reset.backend.circuit_opened_at_ms, None);
    assert_eq!(reset.backend.backoff_until_ms, None);
    assert_eq!(reset.reset_at_ms, reset.backend.updated_at_ms);

    let saved = store
        .get_storage_backend_health(&backend_key)
        .await
        .unwrap()
        .expect("reset should preserve the durable backend health row");
    assert_eq!(saved.status, StorageBackendHealthStatus::Healthy);
    assert_eq!(
        saved.circuit_breaker_state,
        StorageCircuitBreakerState::Closed
    );
    assert_eq!(saved.consecutive_failures, 0);
    assert_eq!(saved.last_failure_at_ms, None);
    assert_eq!(saved.backoff_until_ms, None);
}

#[tokio::test]
async fn admin_v1_acquisition_intake_exposes_redacted_diagnostics_and_watch_folder_discovery() {
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    fs::create_dir_all(watch.join("Season 01")).unwrap();
    fs::write(watch.join("Ready Movie.mkv"), b"ready").unwrap();
    fs::write(watch.join("Season 01").join("Episode 01.mp4"), b"episode").unwrap();
    fs::write(watch.join("Downloading.part"), b"partial").unwrap();
    fs::write(watch.join("Notes.txt"), b"notes").unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let discovery_request = AdminWatchFolderDiscoveryRequest {
        target_library_id: library_id,
        root_uri: Some("local:///watch".to_owned()),
        max_depth: Some(4),
    };
    let first_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/acquisition/intake/watch-folder-discovery")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&discovery_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = first_response.status();
    let body = String::from_utf8(
        to_bytes(first_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let first_discovery: AdminWatchFolderDiscoveryResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(first_discovery.target_library_id, library_id);
    assert_eq!(first_discovery.root_scheme.as_deref(), Some("local"));
    assert_eq!(first_discovery.root_ref_redacted, "local://<redacted>");
    assert_eq!(first_discovery.ready_candidates, 0);
    assert_eq!(first_discovery.inspecting_candidates, 2);
    assert_eq!(first_discovery.blocked_candidates, 2);
    assert_eq!(first_discovery.incomplete_candidates, 1);
    assert_eq!(first_discovery.unsupported_candidates, 1);
    assert_eq!(first_discovery.recorded_candidates, 4);
    assert_eq!(first_discovery.newly_ready_candidates, 0);
    assert!(first_discovery.failures.is_empty());

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/acquisition/intake/watch-folder-discovery")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&discovery_request).unwrap()))
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
    let discovery: AdminWatchFolderDiscoveryResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(discovery.target_library_id, library_id);
    assert_eq!(discovery.root_scheme.as_deref(), Some("local"));
    assert_eq!(discovery.root_ref_redacted, "local://<redacted>");
    assert_eq!(discovery.ready_candidates, 2);
    assert_eq!(discovery.inspecting_candidates, 0);
    assert_eq!(discovery.blocked_candidates, 2);
    assert_eq!(discovery.incomplete_candidates, 1);
    assert_eq!(discovery.unsupported_candidates, 1);
    assert_eq!(discovery.recorded_candidates, 4);
    assert_eq!(discovery.newly_ready_candidates, 2);
    assert!(discovery.failures.is_empty());
    assert!(!discovery.writes_library);
    assert!(!discovery.managed_import_artifacts_created);
    assert!(!discovery.promotion_apply);

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Ready Movie"));
    assert!(!body.contains("Episode 01"));
    assert!(!body.contains("Downloading"));
    assert!(!body.contains("Notes.txt"));
    assert!(!body.contains("admin-token"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("root_uri"));

    let list_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/acquisition/intake/candidates?library_id={library_id}&source_kind=watch_folder&state=ready&limit=10"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = list_response.status();
    let body = String::from_utf8(
        to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "{body}");
    let diagnostics: AdminAcquisitionIntakeCandidateListResponse =
        serde_json::from_str(&body).unwrap();

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(diagnostics.candidates.len(), 2);
    assert_eq!(diagnostics.page.limit, 10);
    assert_eq!(diagnostics.page.returned, 2);
    for candidate in &diagnostics.candidates {
        assert_eq!(candidate.target_library_id, library_id);
        assert_eq!(candidate.source_kind, "watch_folder");
        assert_eq!(candidate.source_scheme.as_deref(), Some("local"));
        assert_eq!(candidate.source_ref_redacted, "local://<redacted>");
        assert!(candidate.source_key_fingerprint.starts_with("sha256:"));
        assert_eq!(candidate.state, AcquisitionIntakeCandidateState::Ready);
        assert!(candidate.has_display_name);
        assert!(candidate.has_diagnostics);
    }

    assert!(!body.contains("source_uri"));
    assert!(!body.contains("\"display_name\""));
    assert!(!body.contains("\"intended_locator\""));
    assert!(!body.contains("diagnostics_json"));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Ready Movie"));
    assert!(!body.contains("Episode 01"));
    assert!(!body.contains("admin-token"));
    assert!(!body.contains("local:///"));
}

#[tokio::test]
async fn admin_v1_acquisition_intake_rejects_raw_root_uri_without_echoing_it() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let discovery_request = AdminWatchFolderDiscoveryRequest {
        target_library_id: library_id,
        root_uri: Some("C:\\private\\watch?token=admin-token".to_owned()),
        max_depth: Some(1),
    };

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/acquisition/intake/watch-folder-discovery")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&discovery_request).unwrap()))
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

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(body.contains("invalid root_uri"));
    assert!(!body.contains("C:\\"));
    assert!(!body.contains("private"));
    assert!(!body.contains("admin-token"));
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
        user_agent: "nako-test/1".to_owned(),
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
    let network = crate::config::NetworkAccessConfig {
        exposure_mode: crate::config::NetworkExposureMode::TunnelProvider,
        external_base_url: Some(
            "https://user:network-secret@nako.example.test/path?token=url-secret".to_owned(),
        ),
        trusted_proxy_headers: true,
        trusted_proxy_sources: vec!["127.0.0.1".to_owned(), "10.0.0.0/8".to_owned()],
        allowed_origins: vec!["https://operator-secret.example.test".to_owned()],
        tunnel_providers: vec![crate::config::TunnelProviderConfig {
            id: "cloudflared".to_owned(),
            kind: crate::config::TunnelProviderKind::CloudflareTunnel,
            public_url: Some(
                "https://user:tunnel-url-secret@tunnel.example.test/path?token=secret".to_owned(),
            ),
            token_env: None,
        }],
    };
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite://F:/secret/nako.db".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig {
            enabled: false,
            token_env: Some("NAKO_ADMIN_TOKEN".to_owned()),
        },
        network,
        ffprobe_path: temp.path().join("private").join("ffprobe"),
        ffmpeg_path: temp.path().join("private").join("ffmpeg"),
        scan_concurrency: 2,
        probe_concurrency: 3,
        metadata_concurrency: 4,
        remux_concurrency: 5,
        webhook_concurrency: 6,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata,
        transcode: TranscodeConfig {
            hardware_acceleration: nako_transcode::HardwareAcceleration::Nvenc,
            hardware_fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
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
            ..PlaybackConfig::default()
        },
        artwork: crate::config::ArtworkConfig {
            artifact_root: temp.path().join("artwork-secret-root"),
            fetch_timeout_ms: 6_000,
            fetch_max_attempts: 4,
            fetch_max_bytes: 12_345,
            fetch_concurrency: 3,
            ingest_worker_enabled: true,
            ingest_worker_idle_ms: 250,
            fetch_user_agent: "nako-artwork-test/1".to_owned(),
            fetch_proxy: Some("http://user:artwork-proxy-secret@127.0.0.1:10809".into()),
            max_width: 4_000,
            max_height: 5_000,
        },
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Remote Anime".to_owned(),
            root: temp.path().join("local-root-secret"),
            preset: nako_core::LibraryPreset::Anime,
            webdav: Some(crate::config::WebDavLibraryConfig {
                root: "webdav:///PrivateAnime".to_owned(),
                base_url: "https://user:webdav-secret@example.test/dav".to_owned(),
                username: Some("webdav-user".to_owned()),
                password_env: Some("NAKO_WEBDAV_PASSWORD".to_owned()),
                timeout_ms: 11_000,
                max_attempts: 3,
            }),
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
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

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert!(!diagnostics.auth.enabled);
    assert_eq!(
        diagnostics.auth.token_env.as_deref(),
        Some("NAKO_ADMIN_TOKEN")
    );
    assert_eq!(
        diagnostics.network.exposure_mode,
        nako_api::admin::AdminNetworkExposureMode::TunnelProvider
    );
    assert_eq!(
        diagnostics.network.readiness.status,
        nako_api::admin::AdminNetworkReadinessStatus::Unavailable
    );
    assert!(diagnostics.network.external_endpoint.configured);
    assert_eq!(
        diagnostics.network.external_endpoint.scheme.as_deref(),
        Some("https")
    );
    assert!(
        diagnostics
            .network
            .external_endpoint
            .host_fingerprint
            .as_deref()
            .is_some_and(|fingerprint| fingerprint.starts_with("sha256:"))
    );
    assert!(diagnostics.network.trusted_proxy.headers_enabled);
    assert_eq!(diagnostics.network.trusted_proxy.source_count, 2);
    assert_eq!(diagnostics.network.origins.allowed_origin_count, 1);
    assert!(diagnostics.network.origins.configured);
    assert_eq!(diagnostics.network.tunnel_providers.len(), 1);
    assert_eq!(diagnostics.network.tunnel_providers[0].id, "cloudflared");
    assert_eq!(
        diagnostics.network.tunnel_providers[0].kind,
        nako_api::admin::AdminTunnelProviderKind::CloudflareTunnel
    );
    assert!(diagnostics.network.tunnel_providers[0].endpoint_configured);
    assert_eq!(
        diagnostics.network.tunnel_providers[0]
            .endpoint_scheme
            .as_deref(),
        Some("https")
    );
    assert_eq!(diagnostics.network.tunnel_providers[0].token_env, None);
    assert!(!diagnostics.network.tunnel_providers[0].token_present);
    assert_eq!(
        diagnostics.database.configured_backend_kind.as_str(),
        "sqlite"
    );
    assert_eq!(diagnostics.database.active_backend_kind.as_str(), "sqlite");
    assert_eq!(diagnostics.database.url_scheme.as_str(), "sqlite");
    assert!(diagnostics.database.runtime_supported);
    assert!(diagnostics.database.migrated_on_startup);
    assert!(diagnostics.database.capabilities.lifecycle);
    assert!(diagnostics.database.capabilities.libraries);
    assert!(diagnostics.database.capabilities.jobs);
    assert!(diagnostics.database.capabilities.job_leases);
    assert!(diagnostics.database.capabilities.media);
    assert!(diagnostics.database.capabilities.scan_commits);
    assert!(diagnostics.database.capabilities.metadata);
    assert!(diagnostics.database.capabilities.catalog);
    assert!(diagnostics.database.capabilities.playback_state);
    assert!(diagnostics.database.capabilities.transcode_sessions);
    assert!(diagnostics.database.capabilities.event_outbox);
    assert!(diagnostics.database.capabilities.addons);
    assert!(diagnostics.database.capabilities.automation);
    assert!(diagnostics.database.capabilities.managed_artwork);
    assert!(diagnostics.database.capabilities.vfs_cache);
    assert!(diagnostics.database.capabilities.webhooks);
    assert!(diagnostics.database.capabilities.search_index);
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
    assert!(diagnostics.artwork.artifact_root_configured);
    assert_eq!(diagnostics.artwork.fetch_timeout_ms, 6_000);
    assert_eq!(diagnostics.artwork.fetch_max_attempts, 4);
    assert_eq!(diagnostics.artwork.fetch_max_bytes, 12_345);
    assert_eq!(diagnostics.artwork.fetch_concurrency, 3);
    assert!(diagnostics.artwork.ingest_worker_enabled);
    assert_eq!(diagnostics.artwork.ingest_worker_idle_ms, 250);
    assert_eq!(diagnostics.artwork.fetch_user_agent, "nako-artwork-test/1");
    assert!(diagnostics.artwork.has_fetch_proxy);
    assert_eq!(diagnostics.artwork.max_width, 4_000);
    assert_eq!(diagnostics.artwork.max_height, 5_000);

    assert!(!body.contains("database_url"));
    assert!(!body.contains("secret/nako.db"));
    assert!(!body.contains("F:/secret"));
    assert!(!body.contains("ffmpeg_path"));
    assert!(!body.contains("ffprobe_path"));
    assert!(!body.contains("private/ffmpeg"));
    assert!(!body.contains("remux_staging_root"));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("\"artifact_root\""));
    assert!(!body.contains("artwork-secret-root"));
    assert!(!body.contains("artwork-proxy-secret"));
    assert!(!body.contains("local-root-secret"));
    assert!(!body.contains("PrivateAnime"));
    assert!(!body.contains("https://user:webdav-secret@example.test/dav"));
    assert!(!body.contains("webdav-secret"));
    assert!(!body.contains("webdav-user"));
    assert!(!body.contains("NAKO_WEBDAV_PASSWORD"));
    assert!(!body.contains("proxy-secret"));
    assert!(!body.contains("api.bgm.tv"));
    assert!(!body.contains("lain.bgm.tv"));
    assert!(!body.contains("literal-header-secret"));
    assert!(!body.contains("BANGUMI_HEADER"));
    assert!(!body.contains("external_base_url"));
    assert!(!body.contains("trusted_proxy_sources"));
    assert!(!body.contains("allowed_origins"));
    assert!(!body.contains("network-secret"));
    assert!(!body.contains("url-secret"));
    assert!(!body.contains("operator-secret"));
    assert!(!body.contains("tunnel-url-secret"));
    assert!(!body.contains("nako.example.test"));
    assert!(!body.contains("tunnel.example.test"));
    assert!(!body.contains("x-forwarded"));
}

#[tokio::test]
async fn admin_v1_system_config_reports_postgres_capability_gaps_for_injected_store() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: nako_db::DatabaseBackendKind::Postgres,
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "postgres://user:secret@db.example.test/nako?sslmode=require".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 1,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig {
            cleanup_on_startup: false,
            ..StagingConfig::default()
        },
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    let router = build_router(app);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/system/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let diagnostics: AdminServerConfigDiagnosticsResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        diagnostics.database.configured_backend_kind.as_str(),
        "postgres"
    );
    assert_eq!(diagnostics.database.active_backend_kind.as_str(), "sqlite");
    assert_eq!(diagnostics.database.url_scheme.as_str(), "postgres");
    assert!(!diagnostics.database.runtime_supported);
    assert!(diagnostics.database.capabilities.managed_artwork);
    assert!(diagnostics.database.capabilities.vfs_cache);
    assert!(!body.contains("database_url"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("db.example.test"));
    assert!(!body.contains("sslmode"));
}

#[tokio::test]
async fn admin_v1_access_management_round_trips_users_roles_and_library_policies() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        webhook_concurrency: 1,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Managed Movies".to_owned(),
            root: temp.path().join("managed"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    let router = build_router(app);

    let initial =
        request_json::<AdminAccessUserListResponse>(&router, Method::GET, "/admin/v1/access/users")
            .await;
    let bootstrap = initial
        .users
        .iter()
        .find(|user| user.bootstrap)
        .expect("bootstrap administrator should be listed");
    assert_eq!(bootstrap.principal_id, nako_core::LOCAL_ADMIN_PRINCIPAL_ID);
    assert!(bootstrap.roles.contains(&UserRole::Administrator));

    let created = request_body_json::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "viewer".to_owned(),
            display_name: "Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
    )
    .await;
    assert_eq!(created.user.username, "viewer");
    assert_eq!(created.user.status, UserStatus::Active);
    assert!(created.user.roles.contains(&UserRole::Viewer));
    assert!(!created.user.principal_id.contains("token"));

    let roles_path = format!("/admin/v1/access/users/{}/roles", created.user.user_id);
    let rerolled = request_body_json::<AdminAccessUserResponse, _>(
        &router,
        Method::PUT,
        &roles_path,
        &AdminReplaceUserRolesRequest {
            roles: vec![UserRole::Viewer, UserRole::LibraryManager],
        },
    )
    .await;
    assert!(rerolled.user.roles.contains(&UserRole::Viewer));
    assert!(rerolled.user.roles.contains(&UserRole::LibraryManager));

    let status_path = format!("/admin/v1/access/users/{}/status", created.user.user_id);
    let disabled = request_body_json::<AdminAccessUserResponse, _>(
        &router,
        Method::PATCH,
        &status_path,
        &AdminUpdateUserStatusRequest {
            status: UserStatus::Disabled,
        },
    )
    .await;
    assert_eq!(disabled.user.status, UserStatus::Disabled);

    let policy = request_body_json::<nako_api::admin::AdminLibraryAccessPolicyResponse, _>(
        &router,
        Method::PUT,
        "/admin/v1/access/library-policies",
        &AdminUpsertLibraryAccessPolicyRequest {
            scope: AdminLibraryAccessPolicyScope::Role {
                role: UserRole::Viewer,
            },
            library_id,
            access: LibraryAccessLevel::Play,
        },
    )
    .await;
    assert_eq!(policy.policy.library_id, library_id);
    assert_eq!(policy.policy.access, LibraryAccessLevel::Play);

    let list_path =
        format!("/admin/v1/access/library-policies?role=viewer&library_id={library_id}");
    let listed =
        request_json::<AdminLibraryAccessPolicyListResponse>(&router, Method::GET, &list_path)
            .await;
    assert_eq!(listed.policies.len(), 1);
    assert_eq!(listed.policies[0].access, LibraryAccessLevel::Play);

    let deleted =
        request_json::<AdminLibraryAccessPolicyDeleteResponse>(&router, Method::DELETE, &list_path)
            .await;
    assert!(deleted.deleted);

    let listed =
        request_json::<AdminLibraryAccessPolicyListResponse>(&router, Method::GET, &list_path)
            .await;
    assert!(listed.policies.is_empty());
}

#[tokio::test]
async fn local_session_auth_login_me_and_logout_use_real_user_principal() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "viewer".to_owned(),
            display_name: "Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    assert!(!created.user.local_password_configured);

    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    let password_response =
        request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
            &router,
            Method::PUT,
            &password_path,
            &nako_api::admin::AdminSetLocalPasswordRequest {
                password: "correct horse battery staple".to_owned(),
            },
            token,
        )
        .await;
    assert_eq!(password_response.user_id, created.user.user_id);
    assert!(password_response.local_password_configured);

    let users = request_json_with_bearer::<AdminAccessUserListResponse>(
        &router,
        Method::GET,
        "/admin/v1/access/users",
        token,
    )
    .await;
    let viewer = users
        .users
        .iter()
        .find(|user| user.user_id == created.user.user_id)
        .expect("created user should be listed");
    assert!(viewer.local_password_configured);

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: " VIEWER ".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    assert!(login.session.token.starts_with("nako_sess_"));
    assert_eq!(login.account.user.id, created.user.user_id.to_string());
    assert_eq!(login.account.user.username, "viewer");
    assert!(!login.account.user.bootstrap);
    assert!(login.account.user.roles.contains(&"viewer".to_owned()));
    let login_json = serde_json::to_string(&login).unwrap();
    assert!(!login_json.contains("correct horse"));
    assert!(!login_json.contains("password_hash"));

    let me = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/users/me")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);
    let me = body_json::<CurrentUserResponse>(me).await;
    assert_eq!(me.user.id, created.user.user_id.to_string());

    let forbidden_library = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden_library.status(), StatusCode::OK);
    let hidden_libraries = body_json::<LibraryListResponse>(forbidden_library).await;
    assert!(hidden_libraries.libraries.is_empty());

    let policy =
        request_body_json_with_bearer::<nako_api::admin::AdminLibraryAccessPolicyResponse, _>(
            &router,
            Method::PUT,
            "/admin/v1/access/library-policies",
            &AdminUpsertLibraryAccessPolicyRequest {
                scope: AdminLibraryAccessPolicyScope::User {
                    user_id: created.user.user_id,
                },
                library_id,
                access: LibraryAccessLevel::Browse,
            },
            token,
        )
        .await;
    assert_eq!(policy.policy.access, LibraryAccessLevel::Browse);

    let allowed_library = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(allowed_library.status(), StatusCode::OK);
    let libraries = body_json::<LibraryListResponse>(allowed_library).await;
    assert_eq!(libraries.libraries[0].id, library_id.to_string());

    let logout = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/logout")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::OK);
    assert!(body_json::<LogoutResponse>(logout).await.revoked);

    let after_logout = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/users/me")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(after_logout.status(), StatusCode::UNAUTHORIZED);

    let bad_login = response_body_json(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "viewer".to_owned(),
            password: "wrong password".to_owned(),
        },
    )
    .await;
    assert_eq!(bad_login.status(), StatusCode::UNAUTHORIZED);
    let bad_login_json =
        serde_json::to_string(&body_json::<ErrorResponse>(bad_login).await).unwrap();
    assert!(!bad_login_json.contains("wrong password"));
    assert!(!bad_login_json.contains("correct horse"));
}

#[tokio::test]
async fn invitation_registration_redeems_once_and_does_not_list_raw_tokens() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let issued = request_body_json_with_bearer::<AdminCreateInvitationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/invitations",
        &AdminCreateInvitationRequest {
            email_or_username: Some("invitee@example.test".to_owned()),
            roles: vec![UserRole::Viewer],
            expires_in_ms: Some(60 * 60 * 1_000),
        },
        token,
    )
    .await;
    assert!(issued.token.starts_with("nako_inv_"));
    assert_eq!(
        issued.invitation.email_or_username.as_deref(),
        Some("invitee@example.test")
    );
    assert_eq!(issued.invitation.roles, vec![UserRole::Viewer]);

    let listed = request_json_with_bearer::<AdminInvitationListResponse>(
        &router,
        Method::GET,
        "/admin/v1/access/invitations",
        token,
    )
    .await;
    assert_eq!(listed.invitations.len(), 1);
    assert_eq!(
        listed.invitations[0].invitation_id,
        issued.invitation.invitation_id
    );
    let listed_json = serde_json::to_string(&listed).unwrap();
    assert!(!listed_json.contains(&issued.token));
    assert!(!listed_json.contains("token_hash"));

    let redeemed = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/invitations/redeem",
        &RedeemInvitationRequest {
            token: issued.token.clone(),
            username: "invited-viewer".to_owned(),
            display_name: "Invited Viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    assert!(redeemed.session.token.starts_with("nako_sess_"));
    assert_eq!(redeemed.account.user.username, "invited-viewer");
    assert!(redeemed.account.user.roles.contains(&"viewer".to_owned()));

    let duplicate = response_body_json(
        &router,
        Method::POST,
        "/auth/invitations/redeem",
        &RedeemInvitationRequest {
            token: issued.token.clone(),
            username: "second-user".to_owned(),
            display_name: "Second User".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::UNAUTHORIZED);

    let revoked_issue = request_body_json_with_bearer::<AdminCreateInvitationResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/invitations",
        &AdminCreateInvitationRequest {
            email_or_username: None,
            roles: vec![UserRole::Viewer],
            expires_in_ms: Some(60 * 60 * 1_000),
        },
        token,
    )
    .await;
    let revoke_path = format!(
        "/admin/v1/access/invitations/{}/revoke",
        revoked_issue.invitation.invitation_id
    );
    let revoked = request_json_with_bearer::<AdminInvitationResponse>(
        &router,
        Method::POST,
        &revoke_path,
        token,
    )
    .await;
    assert_eq!(
        revoked.invitation.status,
        nako_core::UserInvitationStatus::Revoked
    );

    let revoked_redeem = response_body_json(
        &router,
        Method::POST,
        "/auth/invitations/redeem",
        &RedeemInvitationRequest {
            token: revoked_issue.token,
            username: "revoked-user".to_owned(),
            display_name: "Revoked User".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    assert_eq!(revoked_redeem.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn local_session_auth_rejects_disabled_users() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "disabled-viewer".to_owned(),
            display_name: "Disabled Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "disabled-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;

    let status_path = format!("/admin/v1/access/users/{}/status", created.user.user_id);
    request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::PATCH,
        &status_path,
        &AdminUpdateUserStatusRequest {
            status: UserStatus::Disabled,
        },
        token,
    )
    .await;

    let login_after_disable = response_body_json(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "disabled-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;
    assert_eq!(login_after_disable.status(), StatusCode::UNAUTHORIZED);

    let me_after_disable = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/users/me")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me_after_disable.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_v1_access_summary_reports_single_admin_effective_library_access_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let local_library_id = LibraryId::new();
    let remote_library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite://F:/secret/access.db".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig {
            enabled: true,
            token_env: Some("NAKO_ADMIN_TOKEN".to_owned()),
        },
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 1,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("private-remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![
            LocalLibraryConfig {
                id: local_library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("local-root-secret"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: remote_library_id,
                name: "Remote Anime".to_owned(),
                root: temp.path().join("remote-root-secret"),
                preset: nako_core::LibraryPreset::Anime,
                webdav: Some(crate::config::WebDavLibraryConfig {
                    root: "webdav:///PrivateAnime".to_owned(),
                    base_url: "https://user:webdav-secret@example.test/dav".to_owned(),
                    username: Some("webdav-user".to_owned()),
                    password_env: Some("NAKO_WEBDAV_PASSWORD".to_owned()),
                    timeout_ms: 11_000,
                    max_attempts: 3,
                }),
            },
        ],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    let router = build_router_with_auth(
        app,
        auth::InboundAuthState::bearer_token("redacted-admin-token"),
    );

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/access/summary")
                .header(header::AUTHORIZATION, "Bearer redacted-admin-token")
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
    let summary: AdminAccessSummaryResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        summary.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        summary.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(summary.mode, AdminAccessMode::SingleAdmin);
    assert_eq!(
        summary.principal.principal_id,
        nako_core::LOCAL_ADMIN_PRINCIPAL_ID
    );
    assert_eq!(
        summary.principal.principal_kind,
        AdminAccessPrincipalKind::LocalAdmin
    );
    assert_eq!(summary.principal.display_name, "Local administrator");
    assert!(summary.auth.enabled);
    assert!(summary.auth.token_reference_configured);
    assert_eq!(
        summary.readiness.single_admin_mode,
        AdminAccessCapabilityState::Active
    );
    assert_eq!(
        summary.readiness.user_accounts,
        AdminAccessCapabilityState::Active
    );
    assert_eq!(summary.readiness.roles, AdminAccessCapabilityState::Active);
    assert_eq!(
        summary.readiness.library_access_policy,
        AdminAccessCapabilityState::Active
    );
    assert_eq!(summary.library_access.configured_libraries, 2);
    assert_eq!(summary.library_access.libraries.len(), 2);
    assert!(summary.library_access.libraries.iter().any(|library| {
        library.library_id == local_library_id
            && library.library_name == "Movies"
            && library.backend_kind == StorageBackendKind::Local
            && library.access == AdminLibraryAccessLevel::Manage
            && library.reason == AdminLibraryAccessReason::SingleAdminMode
    }));
    assert!(summary.library_access.libraries.iter().any(|library| {
        library.library_id == remote_library_id
            && library.library_name == "Remote Anime"
            && library.backend_kind == StorageBackendKind::WebDav
            && library.access == AdminLibraryAccessLevel::Manage
            && library.reason == AdminLibraryAccessReason::SingleAdminMode
    }));

    assert!(!body.contains("NAKO_ADMIN_TOKEN"));
    assert!(!body.contains("redacted-admin-token"));
    assert!(!body.contains("F:/secret"));
    assert!(!body.contains("local-root-secret"));
    assert!(!body.contains("remote-root-secret"));
    assert!(!body.contains("PrivateAnime"));
    assert!(!body.contains("https://user:webdav-secret@example.test/dav"));
    assert!(!body.contains("webdav-secret"));
    assert!(!body.contains("webdav-user"));
    assert!(!body.contains("NAKO_WEBDAV_PASSWORD"));
    assert!(!body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_metadata_raw_cache_settings_round_trips_persisted_override() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        webhook_concurrency: 1,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig {
            cleanup_on_startup: false,
            ..StagingConfig::default()
        },
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    let router = build_router(app);

    let configured = request_json::<AdminMetadataRawCacheSettingsResponse>(
        &router,
        Method::GET,
        "/admin/v1/settings/metadata/raw-cache",
    )
    .await;
    assert_eq!(
        configured.source,
        nako_core::AdminSettingsSource::Configured
    );
    assert_eq!(configured.effect, nako_core::AdminSettingsEffect::Active);
    assert_eq!(configured.updated_at_ms, None);

    let update = request_body_json::<
        AdminMetadataRawCacheSettingsResponse,
        AdminUpdateMetadataRawCacheSettingsRequest,
    >(
        &router,
        Method::PUT,
        "/admin/v1/settings/metadata/raw-cache",
        &AdminUpdateMetadataRawCacheSettingsRequest {
            retention_ms: 3_600_000,
            cleanup_on_startup: false,
        },
    )
    .await;
    assert_eq!(update.retention_ms, 3_600_000);
    assert!(!update.cleanup_on_startup);
    assert_eq!(update.source, nako_core::AdminSettingsSource::Admin);
    assert_eq!(
        update.effect,
        nako_core::AdminSettingsEffect::RequiresRestart
    );
    assert!(update.updated_at_ms.is_some());

    let persisted = request_json::<AdminMetadataRawCacheSettingsResponse>(
        &router,
        Method::GET,
        "/admin/v1/settings/metadata/raw-cache",
    )
    .await;
    assert_eq!(persisted.retention_ms, 3_600_000);
    assert_eq!(
        persisted.effect,
        nako_core::AdminSettingsEffect::RequiresRestart
    );

    drop(router);
    let restarted = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let restarted_router = build_router(restarted);
    let active = request_json::<AdminMetadataRawCacheSettingsResponse>(
        &restarted_router,
        Method::GET,
        "/admin/v1/settings/metadata/raw-cache",
    )
    .await;
    assert_eq!(active.source, nako_core::AdminSettingsSource::Admin);
    assert_eq!(active.effect, nako_core::AdminSettingsEffect::Active);
    assert_eq!(active.retention_ms, 3_600_000);
    assert!(!active.cleanup_on_startup);

    let diagnostics = request_json::<AdminServerConfigDiagnosticsResponse>(
        &restarted_router,
        Method::GET,
        "/admin/v1/system/config",
    )
    .await;
    assert_eq!(diagnostics.metadata.raw_cache_retention_ms, 3_600_000);
    assert!(!diagnostics.metadata.raw_cache_cleanup_on_startup);
}

#[tokio::test]
async fn admin_v1_metadata_raw_cache_settings_rejects_zero_retention() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;

    let response = response_body_json(
        &router,
        Method::PUT,
        "/admin/v1/settings/metadata/raw-cache",
        &AdminUpdateMetadataRawCacheSettingsRequest {
            retention_ms: 0,
            cleanup_on_startup: true,
        },
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::InvalidInput.as_str()
    );
}

#[tokio::test]
async fn admin_v1_metadata_raw_cache_settings_rejects_non_admin_session() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let router = test_router_with_bearer_auth(temp.path().to_path_buf(), library_id, token).await;

    let created = request_body_json_with_bearer::<AdminAccessUserResponse, _>(
        &router,
        Method::POST,
        "/admin/v1/access/users",
        &AdminCreateUserRequest {
            username: "settings-viewer".to_owned(),
            display_name: "Settings Viewer".to_owned(),
            roles: vec![UserRole::Viewer],
        },
        token,
    )
    .await;
    let password_path = format!(
        "/admin/v1/access/users/{}/local-password",
        created.user.user_id
    );
    request_body_json_with_bearer::<nako_api::admin::AdminLocalPasswordResponse, _>(
        &router,
        Method::PUT,
        &password_path,
        &nako_api::admin::AdminSetLocalPasswordRequest {
            password: "correct horse battery staple".to_owned(),
        },
        token,
    )
    .await;

    let login = request_body_json::<LoginResponse, _>(
        &router,
        Method::POST,
        "/auth/login",
        &LoginRequest {
            username: "settings-viewer".to_owned(),
            password: "correct horse battery staple".to_owned(),
        },
    )
    .await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/admin/v1/settings/metadata/raw-cache")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", login.session.token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&AdminUpdateMetadataRawCacheSettingsRequest {
                        retention_ms: 3_600_000,
                        cleanup_on_startup: true,
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::Forbidden.as_str()
    );
    assert_eq!(error.message, "administrator role is required");
}

#[tokio::test]
async fn admin_v1_playback_runtime_settings_round_trips_persisted_override() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        webhook_concurrency: 1,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 60_000,
        remux_staging_root: temp.path().join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig {
            cleanup_on_startup: false,
            ..StagingConfig::default()
        },
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    let router = build_router(app);

    let configured = request_json::<AdminPlaybackRuntimeSettingsResponse>(
        &router,
        Method::GET,
        "/admin/v1/settings/playback/runtime",
    )
    .await;
    assert_eq!(
        configured.source,
        nako_core::AdminSettingsSource::Configured
    );
    assert_eq!(configured.effect, nako_core::AdminSettingsEffect::Active);
    assert_eq!(configured.updated_at_ms, None);
    assert_eq!(configured.settings.remux_timeout_ms, 60_000);

    let override_settings = AdminPlaybackRuntimeSettingsPayload {
        hardware_acceleration: AdminHardwareAcceleration::None,
        hardware_fallback: AdminHardwareAccelerationFallback::Cpu,
        cpu_concurrency: 2,
        gpu_concurrency: 3,
        remux_concurrency: 4,
        remux_timeout_ms: 45_000,
        remote_stream_concurrency: 5,
        remote_stage_concurrency: 6,
        staging_max_bytes: 7_000,
        staging_retention_ms: 8_000,
        staging_cleanup_on_startup: false,
        transcode_artifact_retention_ms: 9_000,
        transcode_artifact_cleanup_on_startup: false,
        hls_segment_cleanup_enabled: true,
        hls_segment_keep_ms: 10_000,
        transcode_throttle_enabled: true,
        transcode_throttle_delay_ms: 11_000,
    };
    let update = request_body_json::<
        AdminPlaybackRuntimeSettingsResponse,
        AdminUpdatePlaybackRuntimeSettingsRequest,
    >(
        &router,
        Method::PUT,
        "/admin/v1/settings/playback/runtime",
        &AdminUpdatePlaybackRuntimeSettingsRequest {
            settings: override_settings.clone(),
        },
    )
    .await;
    assert_eq!(update.settings, override_settings);
    assert_eq!(update.source, nako_core::AdminSettingsSource::Admin);
    assert_eq!(
        update.effect,
        nako_core::AdminSettingsEffect::RequiresRestart
    );
    assert!(update.updated_at_ms.is_some());

    let persisted = request_json::<AdminPlaybackRuntimeSettingsResponse>(
        &router,
        Method::GET,
        "/admin/v1/settings/playback/runtime",
    )
    .await;
    assert_eq!(persisted.settings, override_settings);
    assert_eq!(
        persisted.effect,
        nako_core::AdminSettingsEffect::RequiresRestart
    );

    drop(router);
    let restarted = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let restarted_router = build_router(restarted);
    let active = request_json::<AdminPlaybackRuntimeSettingsResponse>(
        &restarted_router,
        Method::GET,
        "/admin/v1/settings/playback/runtime",
    )
    .await;
    assert_eq!(active.source, nako_core::AdminSettingsSource::Admin);
    assert_eq!(active.effect, nako_core::AdminSettingsEffect::Active);
    assert_eq!(active.settings, override_settings);

    let runtime = request_json::<AdminPlaybackRuntimeDiagnosticsResponse>(
        &restarted_router,
        Method::GET,
        "/admin/v1/playback/runtime",
    )
    .await;
    assert_eq!(runtime.transcode.configured_cpu_slots, 2);
    assert_eq!(runtime.transcode.configured_gpu_slots, 3);
    assert_eq!(runtime.remux.max_concurrent_sessions, 4);
    assert_eq!(runtime.remux.timeout_ms, 45_000);
    assert_eq!(runtime.remote_playback.stream_permits_max, 5);
    assert_eq!(runtime.remote_playback.stage_permits_max, 6);
    assert_eq!(runtime.staging.max_bytes, 7_000);
    assert_eq!(runtime.staging.retention_ms, 8_000);
    assert_eq!(
        runtime.artifact_lifecycle.transcode_artifact_retention_ms,
        9_000
    );
    assert!(runtime.artifact_lifecycle.hls_segment_cleanup_enabled);
    assert_eq!(runtime.artifact_lifecycle.hls_segment_keep_ms, 10_000);
    assert!(runtime.throttle.enabled);
    assert_eq!(runtime.throttle.delay_ms, 11_000);

    let diagnostics = request_json::<AdminServerConfigDiagnosticsResponse>(
        &restarted_router,
        Method::GET,
        "/admin/v1/system/config",
    )
    .await;
    assert_eq!(diagnostics.transcode.cpu_concurrency, 2);
    assert_eq!(diagnostics.transcode.gpu_concurrency, 3);
    assert_eq!(diagnostics.runtime.remux_concurrency, 4);
    assert_eq!(diagnostics.runtime.remux_timeout_ms, 45_000);
    assert_eq!(diagnostics.playback.remote_stream_concurrency, 5);
    assert_eq!(diagnostics.playback.remote_stage_concurrency, 6);
    assert_eq!(diagnostics.staging.max_bytes, 7_000);
    assert_eq!(diagnostics.staging.retention_ms, 8_000);
    assert_eq!(diagnostics.playback.transcode_artifact_retention_ms, 9_000);
    assert!(diagnostics.playback.hls_segment_cleanup_enabled);
    assert_eq!(diagnostics.playback.hls_segment_keep_ms, 10_000);
    assert!(diagnostics.playback.transcode_throttle_enabled);
    assert_eq!(diagnostics.playback.transcode_throttle_delay_ms, 11_000);
}

#[tokio::test]
async fn admin_v1_playback_runtime_settings_rejects_invalid_policy_values() {
    let temp = tempfile::tempdir().unwrap();
    let router = test_router(temp.path().to_path_buf(), LibraryId::new()).await;
    let configured = request_json::<AdminPlaybackRuntimeSettingsResponse>(
        &router,
        Method::GET,
        "/admin/v1/settings/playback/runtime",
    )
    .await;

    let mut zero_cpu = configured.settings.clone();
    zero_cpu.cpu_concurrency = 0;
    let response = response_body_json(
        &router,
        Method::PUT,
        "/admin/v1/settings/playback/runtime",
        &AdminUpdatePlaybackRuntimeSettingsRequest { settings: zero_cpu },
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::InvalidInput.as_str()
    );

    let mut zero_throttle_delay = configured.settings;
    zero_throttle_delay.transcode_throttle_enabled = true;
    zero_throttle_delay.transcode_throttle_delay_ms = 0;
    let response = response_body_json(
        &router,
        Method::PUT,
        "/admin/v1/settings/playback/runtime",
        &AdminUpdatePlaybackRuntimeSettingsRequest {
            settings: zero_throttle_delay,
        },
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<ErrorResponse>(response).await;
    assert_eq!(
        error.code,
        nako_api::public_client::ClientErrorCode::InvalidInput.as_str()
    );
}

#[tokio::test]
async fn admin_v1_playback_sessions_lists_filters_and_redacts_output_paths() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
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

    let transcode_session = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: local_hls_request_key(&source, nako_transcode::HardwareAcceleration::None),
            output_path: temp
                .path()
                .join("nako-cache")
                .join("hls")
                .join("secret")
                .join("playlist.m3u8"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            transcode_session.id,
            TranscodeSessionState::Failed,
            Some(nako_core::TranscodeFailureCategory::Runner),
            Some(format!(
                "ffmpeg failed while writing {}",
                temp.path().join("nako-cache").join("hls").display()
            )),
        )
        .await
        .unwrap();
    let playback_session = store
        .create_playback_session(NewPlaybackSession {
            id: PlaybackSessionId::new(),
            principal_id: UserPrincipalId::local_admin(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Hls,
            state: PlaybackSessionState::Failed,
            client_capabilities_json: Some(
                r#"{"direct_play":true,"containers":["mp4"],"video_codecs":["h264"],"audio_codecs":["aac"]}"#
                    .to_owned(),
            ),
            started_at_ms: 1_779_814_400_000,
            updated_at_ms: 1_779_814_401_000,
        })
        .await
        .unwrap();
    store
        .link_playback_session_transcode(playback_session.id, transcode_session.id)
        .await
        .unwrap();
    store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: local_remux_request_key(&source, nako_transcode::RemuxContainer::Mp4),
            output_path: temp
                .path()
                .join("nako-cache")
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
                    "/admin/v1/playback/sessions?source_id={}&state=failed&limit=5",
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
    assert_eq!(sessions.sessions[0].id, playback_session.id);
    assert_eq!(sessions.sessions[0].source_id, source.id);
    assert_eq!(sessions.sessions[0].item_id, source.item_id);
    assert_eq!(sessions.sessions[0].mode, PlaybackSessionMode::Hls);
    assert_eq!(sessions.sessions[0].state, PlaybackSessionState::Failed);
    assert_eq!(
        sessions.sessions[0].transcode_session_id,
        Some(transcode_session.id)
    );
    assert!(sessions.sessions[0].has_client_capabilities);
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
async fn admin_v1_playback_support_evidence_is_bounded_and_redacted() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let ffmpeg_path = fake_ffmpeg_encoder_script(
        temp.path(),
        "support-evidence",
        &[" V..... libx264", " A..... aac"],
    );
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 2,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 90_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig {
            hardware_acceleration: nako_transcode::HardwareAcceleration::Nvenc,
            hardware_fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 0,
            gpu_concurrency: 0,
        },
        staging: StagingConfig {
            max_bytes: 123_456,
            retention_ms: 654_321,
            cleanup_on_startup: true,
        },
        playback: PlaybackConfig {
            remote_stream_concurrency: 0,
            remote_stage_concurrency: 0,
            ..PlaybackConfig::default()
        },
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Support Evidence Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Private/Support Evidence Demo.mkv?token=admin-token".to_owned(),
        file_name: "Support Evidence Demo.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("sha256:private-fingerprint".to_owned()),
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let session = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: local_hls_request_key(&source, nako_transcode::HardwareAcceleration::None),
            output_path: temp
                .path()
                .join("secret-cache")
                .join("hls")
                .join("private")
                .join("playlist.m3u8"),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            session.id,
            TranscodeSessionState::Failed,
            Some(nako_core::TranscodeFailureCategory::Runner),
            Some(format!(
                "ffmpeg failed while writing {} with argv -i local:///Movies/Private/Support Evidence Demo.mkv token=admin-token",
                temp.path()
                    .join("secret-cache")
                    .join("hls")
                    .join("private")
                    .join("playlist.m3u8")
                    .display()
            )),
        )
        .await
        .unwrap();

    let router = build_router(app);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/playback/support?session_id={}",
                    session.id
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
    let evidence: AdminPlaybackSupportEvidenceResponse = serde_json::from_str(&body).unwrap();

    assert_eq!(
        evidence.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        evidence.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(evidence.subject.session_id, Some(session.id));
    assert_eq!(evidence.subject.source_id, Some(source.id));
    assert_eq!(evidence.session.as_ref().unwrap().id, session.id);
    assert_eq!(
        evidence.session.as_ref().unwrap().failure_category,
        Some(nako_core::TranscodeFailureCategory::Runner)
    );
    assert!(evidence.session.as_ref().unwrap().has_failure_message);
    assert_eq!(evidence.source.as_ref().unwrap().source_id, source.id);
    assert_eq!(evidence.source.as_ref().unwrap().source_scheme, "local");
    assert_eq!(
        evidence.runtime.readiness.status,
        AdminPlaybackReadinessStatus::Degraded
    );
    assert_eq!(
        evidence.runtime.hardware.selected_acceleration,
        AdminHardwareAcceleration::None
    );
    assert!(evidence.runtime.hardware.fallback_used);
    assert_eq!(evidence.runtime.staging.max_bytes, 123_456);
    assert_eq!(evidence.runtime.remote_playback.stream_permits_max, 1);
    assert!(evidence.redaction.paths_redacted);
    assert!(evidence.redaction.source_references_redacted);
    assert!(evidence.redaction.ffmpeg_commands_redacted);
    assert!(evidence.redaction.stderr_redacted);
    assert!(evidence.redaction.credentials_redacted);

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains(&ffmpeg_path.display().to_string()));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("Private"));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("playlist.m3u8"));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("ffmpeg failed while writing"));
    assert!(!body.contains("argv"));
    assert!(!body.contains("admin-token"));
    assert!(!body.contains("private-fingerprint"));
    assert!(!body.contains("ffmpeg_path"));
    assert!(!body.contains("remux_staging_root"));
}

#[tokio::test]
async fn admin_v1_playback_support_evidence_rejects_mismatched_source_context() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Support Evidence Mismatch".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Support Evidence Mismatch.mkv".to_owned(),
        file_name: "Support Evidence Mismatch.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let other_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/Other Source.mkv".to_owned(),
        file_name: "Other Source.mkv".to_owned(),
        size_bytes: Some(84),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store.upsert_media_source(&other_source).await.unwrap();
    let session = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: local_remux_request_key(&source, nako_transcode::RemuxContainer::Mp4),
            output_path: temp
                .path()
                .join("nako-cache")
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
                    "/admin/v1/playback/support?session_id={}&source_id={}",
                    session.id, other_source.id
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

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(!body.contains("local:///"));
    assert!(!body.contains(&temp.path().display().to_string()));
}

#[tokio::test]
async fn admin_v1_playback_runtime_reports_safe_diagnostics() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let marker = temp.path().join("unused.marker");
    let ffmpeg_path = fake_ffmpeg_script(temp.path(), "runtime", false, &marker);
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 3,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 90_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig {
            hardware_acceleration: nako_transcode::HardwareAcceleration::Nvenc,
            hardware_fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
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
            ..PlaybackConfig::default()
        },
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
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

    assert_eq!(
        diagnostics.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
    assert_eq!(
        diagnostics.public_api_version,
        nako_api::public_client::API_VERSION
    );
    assert_eq!(
        diagnostics.ffmpeg.probe_status,
        AdminPlaybackRuntimeStatus::Ready
    );
    assert_eq!(
        diagnostics.readiness.status,
        AdminPlaybackReadinessStatus::Ready
    );
    assert_eq!(
        diagnostics.readiness.reason,
        AdminPlaybackReadinessReason::FfmpegProbeReady
    );
    assert_eq!(diagnostics.readiness.checks.len(), 9);
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::HardwareAcceleration
        && check.reason == AdminPlaybackReadinessReason::RequestedAcceleratorReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::TranscodeBudget
        && check.reason == AdminPlaybackReadinessReason::TranscodeBudgetReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::RemotePlaybackBudget
        && check.reason == AdminPlaybackReadinessReason::RemotePlaybackBudgetReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::PlaybackPolicy
        && check.reason == AdminPlaybackReadinessReason::PlaybackPolicyReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::Staging
        && check.reason == AdminPlaybackReadinessReason::StagingReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::ArtifactLifecycle
        && check.reason == AdminPlaybackReadinessReason::ArtifactLifecycleReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::TranscodeThrottle
        && check.reason == AdminPlaybackReadinessReason::TranscodeThrottleReady));
    assert!(!diagnostics.ffmpeg.has_probe_error);
    assert_eq!(diagnostics.ffmpeg.hardware_capability_count, 6);
    assert_eq!(diagnostics.ffmpeg.available_gpu_capabilities, 3);
    assert_eq!(
        diagnostics.hardware.policy.requested,
        AdminHardwareAcceleration::Nvenc
    );
    assert_eq!(
        diagnostics.hardware.pipeline.selected,
        AdminHardwareAcceleration::Nvenc
    );
    assert!(!diagnostics.hardware.pipeline.fallback_used);
    let nvenc_capability = diagnostics
        .hardware
        .capabilities
        .iter()
        .find(|capability| capability.accelerator == AdminHardwareAcceleration::Nvenc)
        .unwrap();
    assert_eq!(
        nvenc_capability.encoder_discovery.status,
        nako_api::admin::AdminPlaybackHardwareEncoderDiscoveryStatus::Listed
    );
    assert_eq!(
        nvenc_capability.encoder_discovery.encoder.as_deref(),
        Some("h264_nvenc")
    );
    assert_eq!(
        nvenc_capability.device_initialization.status,
        nako_api::admin::AdminPlaybackHardwareDeviceInitializationStatus::NotRun
    );
    assert!(
        nvenc_capability
            .device_initialization
            .operator_check
            .contains("NVENC")
    );
    assert_eq!(
        nvenc_capability.smoke_probe.status,
        nako_api::admin::AdminPlaybackHardwareSmokeProbeStatus::NotRun
    );
    assert!(
        nvenc_capability
            .smoke_probe
            .operator_check
            .contains("NVENC")
    );
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
    let remote_stream_pressure = diagnostics
        .resource_pressure
        .classes
        .iter()
        .find(|pressure| pressure.class == AdminPlaybackResourceClass::RemoteStream)
        .expect("remote stream resource pressure should be reported");
    assert_eq!(remote_stream_pressure.configured_capacity, Some(7));
    assert_eq!(remote_stream_pressure.available_permits, Some(7));
    assert_eq!(remote_stream_pressure.in_use_permits, Some(0));
    assert!(diagnostics.policy.user_policy_rows_supported);
    assert!(diagnostics.policy.role_policy_rows_supported);
    assert!(diagnostics.policy.effective_resolution_supported);
    assert!(diagnostics.policy.library_access_required);
    assert!(diagnostics.policy.user_policy_overrides_role_policy);
    assert!(
        diagnostics
            .policy
            .permissions
            .contains(&nako_core::PlaybackPermission::Remux)
    );
    assert_eq!(diagnostics.staging.max_bytes, 123_456);
    assert_eq!(diagnostics.staging.retention_ms, 654_321);
    assert!(diagnostics.staging.cleanup_on_startup);
    assert!(
        !diagnostics
            .artifact_lifecycle
            .transcode_artifact_cleanup_on_startup
    );
    assert_eq!(diagnostics.artifact_lifecycle.startup_deleted_artifacts, 0);
    assert_eq!(diagnostics.artifact_lifecycle.hls_segment_keep_ms, 60_000);
    assert!(!diagnostics.throttle.enabled);
    assert_eq!(diagnostics.throttle.delay_ms, 3_000);

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains(&ffmpeg_path.display().to_string()));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("ffmpeg_path"));
    assert!(!body.contains("remux_staging_root"));
    assert!(!body.contains("output_path"));
    assert!(!body.contains("token"));
}

#[tokio::test]
async fn admin_v1_playback_runtime_reports_active_resource_pressure() {
    let (temp, router, source, store) = router_with_running_hls_source().await;
    let playlist_path = format!("/sources/{}/stream/hls/playlist.m3u8", source.id);

    let playlist_response = tokio::time::timeout(
        system_process_backed_hls_playlist_readiness_timeout(),
        router.clone().oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&playlist_path)
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await
    .expect("hls playlist route should return while ffmpeg remains running")
    .unwrap();

    assert_eq!(playlist_response.status(), StatusCode::OK);
    let playback_session_id: PlaybackSessionId = playlist_response
        .headers()
        .get(PLAYBACK_SESSION_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .expect("hls playlist should expose playback session id")
        .parse()
        .unwrap();
    let _playlist_body = to_bytes(playlist_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let playback_session = store
        .get_playback_session(playback_session_id)
        .await
        .unwrap()
        .unwrap();
    let transcode_session_id = playback_session
        .transcode_session_id
        .expect("hls playback session should link transcode session");
    let transcode = store
        .get_transcode_session(transcode_session_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(transcode.state, TranscodeSessionState::Running);

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

    let cpu = diagnostics
        .resource_pressure
        .classes
        .iter()
        .find(|class| class.class == AdminPlaybackResourceClass::CpuTranscode)
        .expect("cpu transcode pressure should be reported");
    assert_eq!(cpu.configured_capacity, Some(1));
    assert_eq!(cpu.available_permits, Some(0));
    assert_eq!(cpu.in_use_permits, Some(1));
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("demo.mkv"));

    let cancel_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/playback/sessions/{playback_session_id}/cancel"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    wait_for_system_transcode_state(
        &store,
        transcode_session_id,
        TranscodeSessionState::Cancelled,
    )
    .await;
}

async fn wait_for_system_transcode_state(
    store: &NakoDatabase,
    session_id: TranscodeSessionId,
    expected: TranscodeSessionState,
) {
    let mut last_state = None;
    for _ in 0..80 {
        if let Some(session) = store.get_transcode_session(session_id).await.unwrap() {
            last_state = Some(session.state);
            if session.state == expected {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("transcode session {session_id} did not reach {expected:?}; last state: {last_state:?}");
}

#[tokio::test]
async fn admin_v1_playback_runtime_reports_unavailable_cpu_pipeline_without_blocking_startup() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let ffmpeg_path =
        fake_ffmpeg_encoder_script(temp.path(), "runtime-missing-cpu-aac", &[" V..... libx264"]);
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 3,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 90_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig {
            hardware_acceleration: nako_transcode::HardwareAcceleration::None,
            hardware_fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 2,
            gpu_concurrency: 4,
        },
        staging: StagingConfig {
            max_bytes: 123_456,
            retention_ms: 654_321,
            cleanup_on_startup: true,
        },
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
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

    assert_eq!(
        diagnostics.readiness.status,
        AdminPlaybackReadinessStatus::Unavailable
    );
    assert_eq!(
        diagnostics.readiness.reason,
        AdminPlaybackReadinessReason::SoftwarePipelineUnavailable
    );
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::HardwareAcceleration
        && check.reason == AdminPlaybackReadinessReason::SoftwarePipelineUnavailable));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::SelectedFallback
        && check.status == AdminPlaybackReadinessStatus::Unavailable
        && check.reason == AdminPlaybackReadinessReason::SoftwarePipelineUnavailable));
    assert_eq!(
        diagnostics.hardware.pipeline.status,
        AdminTranscodePipelineReadinessStatus::Unavailable
    );
    assert_eq!(
        diagnostics.hardware.pipeline.reason,
        AdminTranscodePipelineReadinessReason::SoftwarePipelineUnavailable
    );
    assert_eq!(
        diagnostics.hardware.pipeline.selected,
        AdminHardwareAcceleration::None
    );
    assert!(!diagnostics.hardware.pipeline.fallback_used);
    assert_eq!(diagnostics.transcode.selected_hls_slots, 0);

    let cpu_capability = diagnostics
        .hardware
        .capabilities
        .iter()
        .find(|capability| capability.accelerator == AdminHardwareAcceleration::None)
        .unwrap();
    assert!(!cpu_capability.available);
    assert!(cpu_capability.stage_capabilities.iter().any(|stage| {
        stage.stage == AdminHardwarePipelineStage::Encode
            && stage.required
            && !stage.available
            && stage.feature.as_deref() == Some("aac")
    }));

    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains(&ffmpeg_path.display().to_string()));
    assert!(!body.contains("secret-cache"));
    assert!(!body.contains("ffmpeg_path"));
    assert!(!body.contains("remux_staging_root"));

    let overview_response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(overview_response.status(), StatusCode::OK);
    let overview_body = String::from_utf8(
        to_bytes(overview_response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let overview: AdminOverviewResponse = serde_json::from_str(&overview_body).unwrap();
    assert_eq!(
        overview.operator_readiness.status,
        AdminOperatorReadinessStatus::Unavailable
    );
    let playback_check = overview
        .operator_readiness
        .checks
        .iter()
        .find(|check| check.area == AdminOperatorReadinessArea::Playback)
        .expect("playback readiness check");
    assert_eq!(
        playback_check.status,
        AdminOperatorReadinessStatus::Unavailable
    );
    assert_eq!(
        playback_check.reason,
        AdminOperatorReadinessReason::PlaybackUnavailable
    );
    assert_eq!(
        playback_check.source_reason.as_deref(),
        Some("software_pipeline_unavailable")
    );
    assert_eq!(
        playback_check.action.as_ref().unwrap().route_key,
        "playbackRuntime"
    );
    assert!(!overview_body.contains(&temp.path().display().to_string()));
    assert!(!overview_body.contains(&ffmpeg_path.display().to_string()));
    assert!(!overview_body.contains("secret-cache"));
    assert!(!overview_body.contains("ffmpeg_path"));
    assert!(!overview_body.contains("remux_staging_root"));
    assert!(!overview_body.contains("ffmpeg -"));
}

#[tokio::test]
async fn admin_v1_playback_runtime_reports_typed_readiness_for_cpu_fallback() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let ffmpeg_path = fake_ffmpeg_encoder_script(
        temp.path(),
        "runtime-cpu-fallback",
        &[" V..... libx264", " A..... aac"],
    );
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 3,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 90_000,
        remux_staging_root: temp.path().join("secret-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig {
            hardware_acceleration: nako_transcode::HardwareAcceleration::Nvenc,
            hardware_fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 0,
            gpu_concurrency: 0,
        },
        staging: StagingConfig {
            max_bytes: 123_456,
            retention_ms: 654_321,
            cleanup_on_startup: true,
        },
        playback: PlaybackConfig {
            remote_stream_concurrency: 0,
            remote_stage_concurrency: 0,
            ..PlaybackConfig::default()
        },
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    let router = build_router(app);

    let response = router
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

    assert_eq!(
        diagnostics.readiness.status,
        AdminPlaybackReadinessStatus::Degraded
    );
    assert_eq!(
        diagnostics.readiness.reason,
        AdminPlaybackReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu
    );
    assert_eq!(diagnostics.readiness.checks.len(), 9);
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::HardwareAcceleration
        && check.reason
            == AdminPlaybackReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::SelectedFallback
        && check.reason == AdminPlaybackReadinessReason::CpuFallbackActive));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::TranscodeBudget
        && check.reason == AdminPlaybackReadinessReason::TranscodeBudgetClamped));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::ArtifactLifecycle
        && check.reason == AdminPlaybackReadinessReason::ArtifactLifecycleReady));
    assert!(diagnostics.readiness.checks.iter().any(|check| check.name
        == AdminPlaybackReadinessCheckName::TranscodeThrottle
        && check.reason == AdminPlaybackReadinessReason::TranscodeThrottleReady));
    assert_eq!(
        diagnostics.hardware.pipeline.selected,
        AdminHardwareAcceleration::None
    );
    assert!(diagnostics.hardware.pipeline.fallback_used);
    assert_eq!(diagnostics.transcode.effective_cpu_slots, 1);
    assert_eq!(diagnostics.transcode.effective_gpu_slots, 1);
    assert_eq!(diagnostics.remote_playback.stream_permits_max, 1);
    assert_eq!(diagnostics.remote_playback.stage_permits_max, 1);

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
        missing.headers()[nako_api::public_client::API_VERSION_HEADER],
        nako_api::public_client::API_VERSION
    );
    let missing_request_id = missing
        .headers()
        .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(missing_request_id.starts_with("req_"));
    assert_eq!(missing.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let missing_error = body_json::<ErrorResponse>(missing).await;
    assert_eq!(
        missing_error.code,
        nako_api::public_client::ClientErrorCode::Unauthorized.as_str()
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
        nako_api::public_client::ClientErrorCode::Unauthorized.as_str()
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

    let admin_acquisition_intake_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/acquisition/intake/candidates")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        admin_acquisition_intake_missing.status(),
        StatusCode::UNAUTHORIZED
    );

    let admin_acquisition_discovery_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/acquisition/intake/watch-folder-discovery")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        admin_acquisition_discovery_missing.status(),
        StatusCode::UNAUTHORIZED
    );

    let admin_catalog_governance_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/catalog/governance/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        admin_catalog_governance_missing.status(),
        StatusCode::UNAUTHORIZED
    );

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

    let admin_playback_support_missing = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/admin/v1/playback/support")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        admin_playback_support_missing.status(),
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
    assert_eq!(
        overview.admin_api_version,
        nako_api::admin::ADMIN_API_VERSION
    );
}

#[tokio::test]
async fn network_boundary_enforces_origin_policy_and_preserves_auth_order() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let mut network = NetworkAccessConfig::default();
    network.exposure_mode = NetworkExposureMode::ReverseProxy;
    network.external_base_url = Some("https://nako.example.test".to_owned());
    network.allowed_origins = vec!["https://app.example.test".to_owned()];
    let router = test_router_with_bearer_auth_and_network(
        temp.path().to_path_buf(),
        library_id,
        token,
        network,
    )
    .await;

    let rejected = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(header::ORIGIN, "https://evil.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
    let rejected_body = serde_json::to_string(&body_json::<ErrorResponse>(rejected).await).unwrap();
    assert!(!rejected_body.contains("evil.example.test"));
    assert!(!rejected_body.contains(token));

    let forbidden = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(header::ORIGIN, "https://evil.example.test")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let forbidden_body =
        serde_json::to_string(&body_json::<ErrorResponse>(forbidden).await).unwrap();
    assert!(!forbidden_body.contains("evil.example.test"));
    assert!(!forbidden_body.contains(token));

    let allowed = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/libraries")
                .header(header::ORIGIN, "https://app.example.test")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(allowed.status(), StatusCode::OK);
    assert_eq!(
        allowed.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://app.example.test"
    );

    let health = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header(header::ORIGIN, "https://evil.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let preflight = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/libraries")
                .header(header::ORIGIN, "https://app.example.test")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(preflight.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        preflight.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://app.example.test"
    );
    assert_eq!(
        preflight.headers()[header::ACCESS_CONTROL_ALLOW_HEADERS],
        "authorization,content-type,range,x-request-id"
    );
    let preflight_request_id = preflight
        .headers()
        .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(preflight_request_id.starts_with("req_"));

    let rejected_preflight = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/libraries")
                .header(header::ORIGIN, "https://evil.example.test")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected_preflight.status(), StatusCode::FORBIDDEN);
    let rejected_preflight_request_id = rejected_preflight
        .headers()
        .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(rejected_preflight_request_id.starts_with("req_"));
    let rejected_preflight_body =
        serde_json::to_string(&body_json::<ErrorResponse>(rejected_preflight).await).unwrap();
    assert!(!rejected_preflight_body.contains("evil.example.test"));
}

#[tokio::test]
async fn network_boundary_trusts_forwarded_host_only_when_proxy_policy_allows_it() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let token = "test-admin-token";
    let mut untrusted_network = NetworkAccessConfig::default();
    untrusted_network.exposure_mode = NetworkExposureMode::ReverseProxy;
    untrusted_network.external_base_url = Some("https://nako.example.test".to_owned());
    let untrusted_router = test_router_with_bearer_auth_and_network(
        temp.path().to_path_buf(),
        library_id,
        token,
        untrusted_network,
    )
    .await;

    let untrusted = untrusted_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header("x-forwarded-host", "evil.example.test")
                .header("x-forwarded-proto", "http")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(untrusted.status(), StatusCode::OK);
    assert!(!untrusted.headers().contains_key("x-nako-external-origin"));

    let mut trusted_network = NetworkAccessConfig::default();
    trusted_network.exposure_mode = NetworkExposureMode::ReverseProxy;
    trusted_network.external_base_url = Some("https://nako.example.test".to_owned());
    trusted_network.trusted_proxy_headers = true;
    trusted_network.trusted_proxy_sources = vec!["127.0.0.1".to_owned(), "10.10.0.0/16".to_owned()];
    let trusted_router = test_router_with_bearer_auth_and_network(
        temp.path().to_path_buf(),
        library_id,
        token,
        trusted_network,
    )
    .await;

    let trusted = trusted_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header("x-forwarded-host", "nako.example.test")
                .header("x-forwarded-proto", "https")
                .extension(axum::extract::connect_info::ConnectInfo(
                    "127.0.0.1:4000".parse::<std::net::SocketAddr>().unwrap(),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(trusted.status(), StatusCode::OK);
    assert_eq!(
        trusted.headers()["x-nako-external-origin"],
        "https://nako.example.test"
    );

    let cidr_trusted = trusted_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header("x-forwarded-host", "nako-lan.example.test")
                .header("x-forwarded-proto", "https")
                .extension(axum::extract::connect_info::ConnectInfo(
                    "10.10.4.20:4000".parse::<std::net::SocketAddr>().unwrap(),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cidr_trusted.status(), StatusCode::OK);
    assert_eq!(
        cidr_trusted.headers()["x-nako-external-origin"],
        "https://nako-lan.example.test"
    );

    let malformed = trusted_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header("x-forwarded-host", "nako.example.test, evil.example.test")
                .header("x-forwarded-proto", "https")
                .extension(axum::extract::connect_info::ConnectInfo(
                    "127.0.0.1:4000".parse::<std::net::SocketAddr>().unwrap(),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::OK);
    assert!(!malformed.headers().contains_key("x-nako-external-origin"));

    let spoofed = trusted_router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .header("x-forwarded-host", "evil.example.test")
                .header("x-forwarded-proto", "https")
                .extension(axum::extract::connect_info::ConnectInfo(
                    "198.51.100.10:4000"
                        .parse::<std::net::SocketAddr>()
                        .unwrap(),
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(spoofed.status(), StatusCode::OK);
    assert!(!spoofed.headers().contains_key("x-nako-external-origin"));
}

#[tokio::test]
async fn api_errors_map_playback_storage_categories() {
    let cases = [
        (
            NakoError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::StagingBudgetExhausted,
                "used=10, additional=4, max=12",
            ),
            StatusCode::INSUFFICIENT_STORAGE,
            nako_api::public_client::ClientErrorCode::StagingBudgetExhausted,
            "staging disk budget exhausted",
        ),
        (
            NakoError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::StagingValidationMismatch,
                "staged WebDAV file did not match expected size",
            ),
            StatusCode::BAD_GATEWAY,
            nako_api::public_client::ClientErrorCode::StagingValidationMismatch,
            "staged input validation failed",
        ),
        (
            NakoError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::Timeout,
                "WebDAV request failed: operation timed out",
            ),
            StatusCode::GATEWAY_TIMEOUT,
            nako_api::public_client::ClientErrorCode::StorageTimeout,
            "storage backend timed out",
        ),
        (
            NakoError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::Unauthorized,
                "WebDAV GET returned 401 Unauthorized",
            ),
            StatusCode::BAD_GATEWAY,
            nako_api::public_client::ClientErrorCode::StorageUnauthorized,
            "storage backend rejected credentials",
        ),
        (
            NakoError::storage(
                "webdav:///Movies/Demo.mkv",
                StorageErrorKind::RateLimited,
                "WebDAV GET returned 429 Too Many Requests",
            ),
            StatusCode::SERVICE_UNAVAILABLE,
            nako_api::public_client::ClientErrorCode::StorageRateLimited,
            "storage backend rate limited the request",
        ),
        (
            NakoError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message: "hls runner failed".to_owned(),
            },
            StatusCode::BAD_GATEWAY,
            nako_api::public_client::ClientErrorCode::FfmpegError,
            "ffmpeg operation failed",
        ),
        (
            NakoError::Database {
                message: "raw sqlite path F:\\secret\\nako.db failed".to_owned(),
            },
            StatusCode::INTERNAL_SERVER_ERROR,
            nako_api::public_client::ClientErrorCode::DatabaseError,
            "database operation failed",
        ),
    ];

    for (error, status, code, message) in cases {
        let response = ApiError(error).into_response();

        assert_eq!(response.status(), status);
        let body = body_json::<ErrorResponse>(response).await;
        assert_eq!(body.code, code.as_str());
        assert_eq!(
            nako_api::public_client::ClientErrorCode::from_code(&body.code),
            Some(code)
        );
        assert_eq!(body.message, message);
    }
}
