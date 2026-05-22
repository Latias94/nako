use super::*;

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
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
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
        Some("NAKO_AUTOMATION_SECRET".to_owned())
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
    assert!(job.has_input);
    assert!(!job.has_summary);
    assert!(!job.has_error);

    let artifacts_path = format!("/automation/jobs/{}/artifacts", job.id);
    let artifacts =
        request_json::<AutomationArtifactsResponse>(&router, Method::GET, &artifacts_path).await;
    assert!(artifacts.artifacts.is_empty());
}
