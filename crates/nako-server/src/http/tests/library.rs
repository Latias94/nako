use super::*;

#[tokio::test]
async fn scan_route_queues_background_job() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let (router, store) = test_router_with_store(temp.path().to_path_buf(), library_id).await;
    let path = format!("/libraries/{library_id}/scan");
    let request_id = "REQ-LIBRARY-SCAN_123.Trace";
    let normalized_request_id = "req-library-scan_123.trace";

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(crate::http::trace_context::X_REQUEST_ID_HEADER, request_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(
        response
            .headers()
            .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(normalized_request_id)
    );

    let (job, response_text) = decode_job_response(response).await;
    assert_eq!(job.kind, nako_core::JobKind::LibraryScan);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert!(job.has_input);
    assert!(!job.has_summary);
    assert!(!job.has_error);
    assert_job_response_hides_scan_input(
        &response_text,
        request_id,
        normalized_request_id,
        temp.path(),
    );

    let persisted = store.get_job(job.id).await.unwrap().unwrap();
    let input_json = persisted
        .input_json
        .as_deref()
        .expect("scan job should persist input");
    assert_scan_input_contains_only_safe_trace_context(
        input_json,
        request_id,
        normalized_request_id,
        temp.path(),
    );

    let loaded_path = format!("/jobs/{}", job.id);
    let loaded_job = request_json::<JobResponse>(&router, Method::GET, &loaded_path).await;
    assert_eq!(loaded_job.id, job.id);
    assert!(loaded_job.has_input);

    let loaded_response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(loaded_path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let loaded_body = to_bytes(loaded_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let loaded_text = String::from_utf8_lossy(&loaded_body);
    assert_job_response_hides_scan_input(
        &loaded_text,
        request_id,
        normalized_request_id,
        temp.path(),
    );
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
    assert!(import_job.has_input);
    assert!(!import_job.has_summary);
    assert!(!import_job.has_error);
    assert_eq!(export_job.kind, JobKind::NfoExport);
    assert_eq!(export_job.resource_class, "metadata.nfo.export");
    assert_eq!(export_job.library_id, Some(library_id));
    assert!(export_job.has_input);
    assert!(!export_job.has_summary);
    assert!(!export_job.has_error);
}

#[tokio::test]
async fn admin_library_command_routes_queue_background_jobs() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;

    let cases = [
        (
            format!("/admin/v1/libraries/{library_id}/scan"),
            JobKind::LibraryScan,
            "disk.scan",
        ),
        (
            format!("/admin/v1/libraries/{library_id}/nfo/import"),
            JobKind::NfoImport,
            "metadata.nfo.import",
        ),
        (
            format!("/admin/v1/libraries/{library_id}/nfo/export"),
            JobKind::NfoExport,
            "metadata.nfo.export",
        ),
    ];

    for (path, kind, resource_class) in cases {
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
        assert_eq!(job.kind, kind);
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.resource_class, resource_class);
        assert_eq!(job.library_id, Some(library_id));
        assert!(job.has_input);
        assert!(!job.has_summary);
        assert!(!job.has_error);
    }
}

#[tokio::test]
async fn admin_scan_route_persists_safe_trace_context_without_exposing_input() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let (router, store) = test_router_with_store(temp.path().to_path_buf(), library_id).await;
    let path = format!("/admin/v1/libraries/{library_id}/scan");
    let request_id = "REQ-ADMIN-SCAN_456.Trace";
    let normalized_request_id = "req-admin-scan_456.trace";

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(crate::http::trace_context::X_REQUEST_ID_HEADER, request_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(
        response
            .headers()
            .get(&crate::http::trace_context::X_REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(normalized_request_id)
    );
    let (job, response_text) = decode_job_response(response).await;
    assert_eq!(job.kind, JobKind::LibraryScan);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.resource_class, "disk.scan");
    assert_eq!(job.library_id, Some(library_id));
    assert!(job.has_input);
    assert_job_response_hides_scan_input(
        &response_text,
        request_id,
        normalized_request_id,
        temp.path(),
    );

    let persisted = store.get_job(job.id).await.unwrap().unwrap();
    let input_json = persisted
        .input_json
        .as_deref()
        .expect("admin scan job should persist input");
    assert_scan_input_contains_only_safe_trace_context(
        input_json,
        request_id,
        normalized_request_id,
        temp.path(),
    );

    let loaded_response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/jobs/{}", job.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (_loaded_job, loaded_text) = decode_job_response(loaded_response).await;
    assert_job_response_hides_scan_input(
        &loaded_text,
        request_id,
        normalized_request_id,
        temp.path(),
    );
}

async fn decode_job_response(response: Response) -> (JobResponse, String) {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    let job = serde_json::from_str::<JobResponse>(&text).unwrap();
    (job, text)
}

fn assert_job_response_hides_scan_input(
    body: &str,
    raw_request_id: &str,
    normalized_request_id: &str,
    local_root: &std::path::Path,
) {
    assert!(!body.contains("\"input\":"));
    assert!(!body.contains("\"summary\":"));
    assert!(!body.contains("\"error\":"));
    assert!(!body.contains("input_json"));
    assert!(!body.contains("summary_json"));
    assert!(!body.contains("trace_context"));
    assert!(!body.contains(raw_request_id));
    assert!(!body.contains(normalized_request_id));
    assert!(!body.contains(&local_root.display().to_string()));
}

fn assert_scan_input_contains_only_safe_trace_context(
    input_json: &str,
    raw_request_id: &str,
    normalized_request_id: &str,
    local_root: &std::path::Path,
) {
    let input = serde_json::from_str::<serde_json::Value>(input_json).unwrap();
    let trace_context = input["trace_context"]
        .as_object()
        .expect("scan job input should include trace_context object");

    assert_eq!(trace_context.len(), 1);
    assert_eq!(
        trace_context
            .get("request_id")
            .and_then(serde_json::Value::as_str),
        Some(normalized_request_id)
    );
    assert!(!input_json.contains(raw_request_id));
    assert!(!input_json.contains(&local_root.display().to_string()));
}

#[tokio::test]
async fn admin_library_metadata_profile_route_reads_and_persists_updates() {
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
    let router = build_router(app);
    let path = format!("/admin/v1/libraries/{library_id}/metadata-profile");

    let current = request_json::<nako_api::admin::AdminLibraryMetadataProfileResponse>(
        &router,
        Method::GET,
        &path,
    )
    .await;
    assert_eq!(current.library_id, library_id);
    assert!(current.profile.scan.enabled);
    assert!(!current.profile.scan.addon_scrape);
    assert!(!current.profile.scan.addon_writeback);
    assert!(current.scan_acquisition_plan.local_nfo_import);
    assert!(!current.scan_acquisition_plan.addon_scrape);

    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.local_metadata_policy = LocalMetadataPolicy::Disabled;
    profile.scan.addon_scrape = true;
    profile.scan.addon_writeback = true;
    let request = nako_api::admin::AdminUpdateLibraryMetadataProfileRequest {
        profile: profile.clone(),
    };

    let updated = request_body_json::<nako_api::admin::AdminLibraryMetadataProfileResponse, _>(
        &router,
        Method::PUT,
        &path,
        &request,
    )
    .await;
    assert_eq!(updated.library_id, library_id);
    assert_eq!(updated.profile, profile);
    assert!(!updated.scan_acquisition_plan.local_nfo_import);
    assert!(updated.scan_acquisition_plan.addon_scrape);
    assert!(updated.scan_acquisition_plan.addon_writeback);

    let persisted = store.get_library(library_id).await.unwrap().unwrap();
    assert_eq!(persisted.options.metadata_profile, profile);

    let reread = request_json::<nako_api::admin::AdminLibraryMetadataProfileResponse>(
        &router,
        Method::GET,
        &path,
    )
    .await;
    assert_eq!(reread.profile, profile);
}

#[tokio::test]
async fn admin_library_metadata_profile_update_changes_next_scan_metadata_plan() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    write_admin_profile_fixture_mp4(&library_root.join("demo.mp4"));
    fs::write(
        library_root.join("demo.nfo"),
        r#"<movie><title>Should Not Import After Admin Update</title></movie>"#,
    )
    .unwrap();
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let router = build_router(app.clone());
    let path = format!("/admin/v1/libraries/{library_id}/metadata-profile");
    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.scan = nako_core::MetadataScanPolicy::disabled();

    let updated = request_body_json::<nako_api::admin::AdminLibraryMetadataProfileResponse, _>(
        &router,
        Method::PUT,
        &path,
        &nako_api::admin::AdminUpdateLibraryMetadataProfileRequest { profile },
    )
    .await;
    assert!(!updated.scan_acquisition_plan.local_nfo_import);

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    let item = store
        .get_media_item(sources[0].item_id)
        .await
        .unwrap()
        .unwrap();

    assert!(output.metadata.nfo_import.is_none());
    assert_eq!(item.metadata.title, "demo");
}

#[tokio::test]
async fn ingestion_failure_routes_list_and_ignore_failures() {
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
    store
        .record_ingestion_failure(NewIngestionFailure {
            library_id,
            job_id: None,
            scan_id: None,
            source_id: None,
            phase: IngestionFailurePhase::Scan,
            target_uri: "local:///Movies/Broken/".to_owned(),
            target_kind: "directory".to_owned(),
            failure_class: IngestionFailureClass::Storage,
            message: "failed to list local directory".to_owned(),
            retryable: true,
            failed_at_ms: 1,
        })
        .await
        .unwrap();
    let router = build_router(app);
    let path = format!("/libraries/{library_id}/ingestion/failures");

    let open = request_json::<IngestionFailuresResponse>(&router, Method::GET, &path).await;
    assert_eq!(open.failures.len(), 1);
    assert_eq!(
        open.failures[0].failure.status,
        IngestionFailureStatus::Open
    );
    assert!(open.failures[0].retryable_now);

    let ignored = request_body_json::<nako_api::admin::IngestionFailureDiagnostic, _>(
        &router,
        Method::POST,
        &path,
        &IgnoreIngestionFailureRequest {
            phase: IngestionFailurePhase::Scan,
            target_uri: "local:///Movies/Broken/".to_owned(),
        },
    )
    .await;
    assert_eq!(ignored.failure.status, IngestionFailureStatus::Ignored);
    assert!(!ignored.retryable_now);

    let open_after_ignore =
        request_json::<IngestionFailuresResponse>(&router, Method::GET, &path).await;
    assert!(open_after_ignore.failures.is_empty());
    let ignored_path = format!("{path}?status=ignored");
    let ignored_list =
        request_json::<IngestionFailuresResponse>(&router, Method::GET, &ignored_path).await;
    assert_eq!(ignored_list.failures.len(), 1);
}

fn write_admin_profile_fixture_mp4(path: &std::path::Path) {
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=16x16:d=0.1",
            "-an",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(path)
        .status()
        .expect("ffmpeg should be available for Nako HTTP scan tests");
    assert!(status.success(), "ffmpeg failed to create fixture mp4");
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
