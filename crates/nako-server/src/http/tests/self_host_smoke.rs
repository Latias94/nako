use super::*;

async fn smoke_response_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

async fn smoke_request_json_with_status<T>(
    router: &Router,
    method: Method,
    uri: &str,
    expected_status: StatusCode,
) -> T
where
    T: serde::de::DeserializeOwned,
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
    assert_eq!(response.status(), expected_status);
    body_json(response).await
}

#[tokio::test]
async fn self_host_smoke_sqlite_operator_flow_redacts_sensitive_boundaries() {
    let (temp, router, source, store) =
        router_with_media_source("self-host-demo.mp4", b"0123456789").await;
    let library_id = source.library_id;

    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
                bit_rate: Some(80_000),
                streams: vec![
                    MediaStreamInfo {
                        index: 0,
                        kind: MediaStreamKind::Video,
                        codec: Some("h264".to_owned()),
                        language: None,
                        duration_ms: Some(1_000),
                        bit_rate: Some(64_000),
                        width: Some(1920),
                        height: Some(1080),
                        channels: None,
                        sample_rate: None,
                        technical: Default::default(),
                    },
                    MediaStreamInfo {
                        index: 1,
                        kind: MediaStreamKind::Audio,
                        codec: Some("aac".to_owned()),
                        language: Some("eng".to_owned()),
                        duration_ms: Some(1_000),
                        bit_rate: Some(16_000),
                        width: None,
                        height: None,
                        channels: Some(2),
                        sample_rate: Some(48_000),
                        technical: Default::default(),
                    },
                ],
            },
        )
        .await
        .unwrap();

    let health = request_json::<HealthResponse>(&router, Method::GET, "/health").await;
    assert_eq!(health.status, "ok");

    let scan_job = smoke_request_json_with_status::<JobResponse>(
        &router,
        Method::POST,
        &format!("/libraries/{library_id}/scan"),
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(scan_job.kind, JobKind::LibraryScan);
    assert_eq!(scan_job.resource_class, "disk.scan");
    assert!(scan_job.has_input);

    let nfo_export_job = smoke_request_json_with_status::<JobResponse>(
        &router,
        Method::POST,
        &format!("/libraries/{library_id}/nfo/export"),
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(nfo_export_job.kind, JobKind::NfoExport);
    assert_eq!(nfo_export_job.resource_class, "metadata.nfo.export");
    assert!(nfo_export_job.has_input);

    let metadata_refresh = smoke_request_json_with_status::<JobResponse>(
        &router,
        Method::POST,
        &format!("/items/{}/metadata/refresh", source.item_id),
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(metadata_refresh.kind, JobKind::MetadataRefresh);
    assert!(metadata_refresh.resource_class.starts_with("metadata."));
    assert!(metadata_refresh.has_input);

    let (remote_url, expected_byte_len) = super::addons::tiny_artwork_server().await;
    let (raw_token, _candidate_id, accepted) = super::addons::propose_and_accept_remote_artwork(
        &router,
        library_id,
        source.item_id,
        &remote_url,
        "self-host-smoke-artwork",
    )
    .await;
    assert_eq!(accepted.ingest.status, ManagedArtworkIngestStatus::Queued);

    let processed = request_json::<ProcessManagedArtworkIngestResponse>(
        &router,
        Method::POST,
        "/admin/v1/artwork/ingests/process-next",
    )
    .await;
    assert!(processed.processed);
    let artifact = processed.artifact.as_ref().unwrap();
    assert_eq!(artifact.byte_len, Some(expected_byte_len));
    assert_eq!(artifact.media_type.as_deref(), Some("image/png"));
    assert!(artifact.has_content_hash);

    let published = request_json::<PublishSelectedArtworkResponse>(
        &router,
        Method::POST,
        &format!("/admin/v1/artwork/artifacts/{}/publish", artifact.id),
    )
    .await;
    assert_eq!(published.selected_artwork.item_id, source.item_id);
    assert_eq!(
        published.image.url,
        format!("/images/{}", published.image.id)
    );

    let images = request_json::<nako_api::public_client::ImagesResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/images", source.item_id),
    )
    .await;
    assert_eq!(images.images.len(), 1);
    assert_eq!(images.images[0].id, published.image.id);

    let image_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&published.image.url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(image_response.status(), StatusCode::OK);
    assert_eq!(
        image_response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    let image_bytes = to_bytes(image_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(image_bytes.len() as u64, expected_byte_len);

    let decision = request_json::<nako_api::public_client::PlaybackDecisionResponse>(
        &router,
        Method::GET,
        &format!("/sources/{}/playback/decision", source.id),
    )
    .await;
    assert_eq!(
        decision.decision.mode,
        nako_api::public_client::ClientPlaybackMode::DirectPlay
    );

    let stream_response = router
        .clone()
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
    assert_eq!(stream_response.status(), StatusCode::PARTIAL_CONTENT);
    let stream_bytes = to_bytes(stream_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&stream_bytes[..], b"2345");

    let overview =
        request_json::<AdminOverviewResponse>(&router, Method::GET, "/admin/v1/overview").await;
    assert!(matches!(
        overview.status,
        AdminOverviewStatus::Healthy | AdminOverviewStatus::Degraded
    ));
    assert_eq!(overview.startup.configured_libraries, 1);

    let config_response = router
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
    assert_eq!(config_response.status(), StatusCode::OK);
    let config_text = smoke_response_text(config_response).await;

    let decision_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/sources/{}/playback/decision", source.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let decision_text = smoke_response_text(decision_response).await;

    let support_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/admin/v1/playback/support?source_id={}",
                    source.id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(support_response.status(), StatusCode::OK);
    let support_text = smoke_response_text(support_response).await;
    let support: AdminPlaybackSupportEvidenceResponse =
        serde_json::from_str(&support_text).unwrap();
    assert_eq!(support.subject.source_id, Some(source.id));
    assert_eq!(support.source.as_ref().unwrap().source_scheme, "local");
    assert_eq!(
        support.runtime.hardware.selected_acceleration,
        AdminHardwareAcceleration::None
    );
    assert_eq!(
        support.runtime.readiness.status,
        AdminPlaybackReadinessStatus::Ready
    );
    assert!(support.redaction.paths_redacted);
    assert!(support.redaction.source_references_redacted);
    assert!(support.redaction.ffmpeg_commands_redacted);
    assert!(support.redaction.stderr_redacted);
    assert!(support.redaction.credentials_redacted);

    for text in [&config_text, &decision_text, &support_text] {
        assert!(!text.contains("storage_uri"));
        assert!(!text.contains("source_uri"));
        assert!(!text.contains("cache_uri"));
        assert!(!text.contains("managed-artwork://"));
        assert!(!text.contains("content_hash"));
        assert!(!text.contains("database_url"));
        assert!(!text.contains("token=secret"));
        assert!(!text.contains(&source.locator));
        assert!(!text.contains(&remote_url));
        assert!(!text.contains(&raw_token));
        assert!(!text.contains(temp.path().to_string_lossy().as_ref()));
    }
}
