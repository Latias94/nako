use super::*;

#[tokio::test]
async fn user_playback_profile_preference_routes_resolve_store_read_and_delete() {
    let (_temp, router, _source, _store) = router_with_hls_source().await;
    let path = "/users/me/playback-profile";

    let empty = request_json::<nako_api::public_client::UserPlaybackProfilePreferenceResponse>(
        &router,
        Method::GET,
        path,
    )
    .await;
    assert_eq!(empty.preference, None);

    let saved = request_body_json::<
        nako_api::public_client::UserPlaybackProfilePreferenceResponse,
        _,
    >(
        &router,
        Method::PUT,
        path,
        &nako_api::public_client::SetUserPlaybackProfilePreferenceRequest {
            direct_play: Some(true),
            device_family: Some("browser_chromium".to_owned()),
            profile_version: Some(1),
            containers: Some(vec!["mp4".to_owned()]),
            video_codecs: Some(vec!["h264".to_owned()]),
            audio_codecs: Some(vec!["aac".to_owned()]),
            hls_variant_policy: Some(nako_api::public_client::ClientHlsVariantPolicy::Adaptive),
            hls_segment_container: Some(nako_api::public_client::ClientHlsSegmentContainer::Fmp4),
            ..nako_api::public_client::SetUserPlaybackProfilePreferenceRequest::default()
        },
    )
    .await;
    let saved_preference = saved.preference.as_ref().expect("preference is saved");
    assert_eq!(
        saved_preference.capabilities.device_family.as_deref(),
        Some("browser_chromium")
    );
    assert_eq!(saved_preference.capabilities.profile_version, Some(1));
    assert!(
        saved_preference
            .capabilities
            .containers
            .contains(&"mp4".to_owned())
    );
    assert_eq!(saved_preference.version, 1);

    let read = request_json::<nako_api::public_client::UserPlaybackProfilePreferenceResponse>(
        &router,
        Method::GET,
        path,
    )
    .await;
    assert_eq!(read, saved);
    let raw = request_json::<serde_json::Value>(&router, Method::GET, path).await;
    assert!(raw["preference"].get("principal_id").is_none());
    assert!(raw["preference"].get("capabilities_json").is_none());

    let deleted = request_json::<
        nako_api::public_client::DeleteUserPlaybackProfilePreferenceResponse,
    >(&router, Method::DELETE, path)
    .await;
    assert!(deleted.deleted);

    let after_delete =
        request_json::<nako_api::public_client::UserPlaybackProfilePreferenceResponse>(
            &router,
            Method::GET,
            path,
        )
        .await;
    assert_eq!(after_delete.preference, None);
}

#[tokio::test]
async fn user_playback_profile_preference_route_rejects_unknown_hls_fields() {
    let (_temp, router, _source, _store) = router_with_hls_source().await;

    let response = response_body_json(
        &router,
        Method::PUT,
        "/users/me/playback-profile",
        &serde_json::json!({
            "device_family": "browser_chromium",
            "profile_version": 1,
            "hls_variant_policy": "future_policy"
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json::<nako_api::public_client::ErrorResponse>(response).await;
    assert_eq!(error.code, "invalid_input");
    assert!(
        error
            .message
            .contains("unsupported playback profile preference")
    );
}

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
async fn user_playback_write_routes_require_play_library_access() {
    let (_temp, app, source, store) =
        app_with_media_source_config("browse-only.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Browse)
            .await;
    let router = public_client_router_with_principal(app, principal);
    let item_id = source.item_id;
    let state_path = format!("/users/me/playback-state/items/{item_id}");
    let progress_path = format!("/users/me/playback-state/items/{item_id}/progress");
    let watched_path = format!("/users/me/playback-state/items/{item_id}/watched");

    let read = response_for(&router, Method::GET, &state_path).await;
    let progress = response_body_json(
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
    let watched = response_body_json(
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

    assert_eq!(read.status(), StatusCode::OK);
    assert_eq!(progress.status(), StatusCode::FORBIDDEN);
    assert_eq!(watched.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn user_playback_read_route_requires_browse_library_access() {
    let (_temp, app, source, store) =
        app_with_media_source_config("no-access-state.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::None).await;
    let router = public_client_router_with_principal(app, principal);
    let state_path = format!("/users/me/playback-state/items/{}", source.item_id);

    let response = response_for(&router, Method::GET, &state_path).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let error = body_json::<nako_api::public_client::ErrorResponse>(response).await;
    assert_eq!(error.code, "forbidden");
    assert!(
        error
            .message
            .contains("required Library Access level 'browse'"),
        "{}",
        error.message
    );
}

#[tokio::test]
async fn continue_watching_filters_items_without_current_library_access() {
    let (_temp, app, source, store) =
        app_with_media_source_config("hidden-continue.mkv", b"media", |_| {}).await;
    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Play).await;
    app.user_playback()
        .update_progress(
            crate::app::user_playback::UpdateUserPlaybackProgressRequest {
                principal: principal.clone(),
                item_id: source.item_id,
                source_id: Some(source.id),
                position_ms: 120_000,
                duration_ms: Some(600_000),
                reported_at_ms: Some(1_774_800_000_000),
            },
        )
        .await
        .unwrap();
    store
        .delete_library_access_policy(
            LibraryAccessPolicyScope::User(principal.user_id),
            source.library_id,
        )
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let continue_watching = request_json::<nako_api::public_client::ContinueWatchingResponse>(
        &router,
        Method::GET,
        "/users/me/playback-state/continue-watching?limit=10&offset=0",
    )
    .await;

    assert_eq!(continue_watching.page.returned, 0);
    assert!(continue_watching.items.is_empty());
}

#[tokio::test]
async fn continue_watching_backfills_visible_items_before_pagination() {
    let (_temp, app, source, store) =
        app_with_media_source_config("visible-continue.mkv", b"media", |_| {}).await;
    let inaccessible_library = Library {
        id: LibraryId::new(),
        name: "Hidden Continue".to_owned(),
        roots: vec!["local:///Hidden Continue".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let inaccessible_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Hidden Continue".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let inaccessible_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: inaccessible_library.id,
        item_id: inaccessible_item.id,
        locator: "local:///Hidden Continue/Hidden Continue.mkv".to_owned(),
        file_name: "Hidden Continue.mkv".to_owned(),
        size_bytes: Some(64),
        fingerprint: Some("hidden-continue".to_owned()),
    };

    store.upsert_library(&inaccessible_library).await.unwrap();
    store.upsert_media_item(&inaccessible_item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: inaccessible_library.id,
            item_id: inaccessible_item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_media_source(&inaccessible_source)
        .await
        .unwrap();

    let principal =
        local_viewer_with_library_access(&store, source.library_id, LibraryAccessLevel::Browse)
            .await;
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.principal_id.clone(),
            item_id: source.item_id,
            source_id: Some(source.id),
            resume_position_ms: Some(120_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(1_000),
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.principal_id.clone(),
            item_id: inaccessible_item.id,
            source_id: Some(inaccessible_source.id),
            resume_position_ms: Some(240_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(2_000),
            updated_at_ms: 2_000,
        })
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let continue_watching = request_json::<nako_api::public_client::ContinueWatchingResponse>(
        &router,
        Method::GET,
        "/users/me/playback-state/continue-watching?limit=1&offset=0",
    )
    .await;

    assert_eq!(continue_watching.page.returned, 1);
    assert_eq!(
        continue_watching.items[0].item.id,
        source.item_id.to_string()
    );
}

#[tokio::test]
async fn user_playback_route_rejects_source_from_another_item() {
    let (_temp, router, first, store) = router_with_hls_source().await;
    let second_library = Library {
        id: first.library_id,
        name: "Second".to_owned(),
        roots: vec!["local:///Second".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let second_item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Second".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let second_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: second_library.id,
        item_id: second_item.id,
        locator: "local:///Second/Second.mkv".to_owned(),
        file_name: "Second.mkv".to_owned(),
        size_bytes: Some(64),
        fingerprint: Some("second".to_owned()),
    };
    store.upsert_library(&second_library).await.unwrap();
    store.upsert_media_item(&second_item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: second_library.id,
            item_id: second_item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&second_source).await.unwrap();

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
