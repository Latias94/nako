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
