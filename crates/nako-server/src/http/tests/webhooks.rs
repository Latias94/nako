use super::*;

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
            url: "https://example.test/nako-webhook".to_owned(),
            secret_env: Some("NAKO_WEBHOOK_SECRET".to_owned()),
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
        Some("NAKO_WEBHOOK_SECRET".to_owned())
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
