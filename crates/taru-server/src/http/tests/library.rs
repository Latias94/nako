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
async fn ingestion_failure_routes_list_and_ignore_failures() {
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

    let ignored = request_body_json::<taru_api::IngestionFailureDiagnostic, _>(
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
