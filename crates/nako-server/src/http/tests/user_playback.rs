use super::*;

#[tokio::test]
async fn user_playback_routes_update_read_and_list_current_principal_state() {
    let (_temp, router, source, _store) = router_with_hls_source().await;
    let item_id = source.item_id;
    let progress_path = format!("/users/me/playback-state/items/{item_id}/progress");
    let state_path = format!("/users/me/playback-state/items/{item_id}");
    let continue_path = "/users/me/playback-state/continue-watching?limit=10&offset=0";
    let watched_path = format!("/users/me/playback-state/items/{item_id}/watched");

    let updated = request_body_json::<nako_api::public_client::UserPlaybackStateResponse, _>(
        &router,
        Method::PUT,
        &progress_path,
        &nako_api::public_client::UpdatePlaybackProgressRequest {
            source_id: Some(source.id.to_string()),
            position_ms: 120_000,
            duration_ms: Some(600_000),
            reported_at: Some("2026-05-19T00:00:00Z".to_owned()),
        },
    )
    .await;
    let read = request_json::<nako_api::public_client::UserPlaybackStateResponse>(
        &router,
        Method::GET,
        &state_path,
    )
    .await;
    let continue_watching = request_json::<nako_api::public_client::ContinueWatchingResponse>(
        &router,
        Method::GET,
        continue_path,
    )
    .await;
    let raw = request_json::<serde_json::Value>(&router, Method::GET, &state_path).await;
    let watched = request_body_json::<nako_api::public_client::UserPlaybackStateResponse, _>(
        &router,
        Method::PUT,
        &watched_path,
        &nako_api::public_client::SetWatchedStateRequest {
            watched: true,
            source_id: Some(source.id.to_string()),
            position_ms: Some(600_000),
            duration_ms: Some(600_000),
            marked_at: Some("2026-05-19T00:01:00Z".to_owned()),
        },
    )
    .await;

    assert_eq!(updated.state.item_id, item_id.to_string());
    assert_eq!(updated.state.source_id, Some(source.id.to_string()));
    assert_eq!(updated.state.resume_position_ms, Some(120_000));
    assert_eq!(updated.state.duration_ms, Some(600_000));
    assert_eq!(updated.state.progress_percent, Some(0.2));
    assert_eq!(
        updated.state.last_played_at.as_deref(),
        Some("2026-05-19T00:00:00Z")
    );
    assert_eq!(updated.state.version, 1);
    assert_eq!(read, updated);
    assert!(raw["state"].get("principal_id").is_none());
    assert!(raw["state"].get("user_id").is_none());

    assert_eq!(continue_watching.page.limit, 10);
    assert_eq!(continue_watching.page.offset, 0);
    assert_eq!(continue_watching.page.returned, 1);
    assert_eq!(continue_watching.items[0].item.id, item_id.to_string());
    assert_eq!(
        continue_watching.items[0].state.resume_position_ms,
        Some(120_000)
    );
    assert!(continue_watching.items[0].images.is_empty());

    assert!(watched.state.watched);
    assert_eq!(watched.state.resume_position_ms, None);
    assert_eq!(
        watched.state.watched_at.as_deref(),
        Some("2026-05-19T00:01:00Z")
    );
    assert!(watched.state.version > updated.state.version);
}

#[tokio::test]
async fn user_playback_route_rejects_source_from_another_item() {
    let (_temp, router, first, store) = router_with_hls_source().await;
    let second_item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Second".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&second_item).await.unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!(
                    "/users/me/playback-state/items/{}/progress",
                    second_item.id
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&nako_api::public_client::UpdatePlaybackProgressRequest {
                        source_id: Some(first.id.to_string()),
                        position_ms: 10_000,
                        duration_ms: Some(100_000),
                        reported_at: Some("2026-05-19T00:00:00Z".to_owned()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = body_json::<nako_api::public_client::ErrorResponse>(response).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body.code, "invalid_input");
    assert!(body.message.contains("does not belong to item"));
}
