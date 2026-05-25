use super::*;

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
    assert_eq!(job.kind, nako_core::JobKind::LibraryScan);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert!(job.has_input);
    assert!(!job.has_summary);
    assert!(!job.has_error);

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
    assert!(!loaded_text.contains("\"input\":"));
    assert!(!loaded_text.contains("\"summary\":"));
    assert!(!loaded_text.contains("\"error\":"));
    assert!(!loaded_text.contains("input_json"));
    assert!(!loaded_text.contains("summary_json"));
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
