use super::*;
use nako_core::{UserPlaylistItemWrite, UserPlaylistRepository};

#[tokio::test]
async fn user_playlist_routes_mutate_current_principal_private_state() {
    let (_temp, app, source, _store) =
        app_with_media_source_config("playlist-route.mkv", b"media", |_| {}).await;
    let router = build_router(app);

    let created = request_body_json::<nako_api::public_client::UserPlaylistResponse, _>(
        &router,
        Method::POST,
        "/users/me/playlists",
        &nako_api::public_client::CreateUserPlaylistRequest {
            name: " Watch Later ".to_owned(),
        },
    )
    .await;
    let playlist_id = created.playlist.id.clone();

    assert_eq!(created.playlist.name, "Watch Later");
    assert_eq!(created.playlist.visibility.wire_value(), "private");
    assert_eq!(created.playlist.item_count, 0);
    assert!(created.playlist.created_at.ends_with('Z'));
    assert!(created.playlist.updated_at.ends_with('Z'));

    let list = request_json::<nako_api::public_client::UserPlaylistsResponse>(
        &router,
        Method::GET,
        "/users/me/playlists?limit=10&offset=0",
    )
    .await;
    assert_eq!(list.page.returned, 1);
    assert_eq!(list.playlists[0].id, playlist_id);

    let add_path = format!("/users/me/playlists/{playlist_id}/items/{}", source.item_id);
    let added = request_body_json::<nako_api::public_client::UserPlaylistResponse, _>(
        &router,
        Method::PUT,
        &add_path,
        &nako_api::public_client::AddUserPlaylistItemRequest {
            position: None,
            expected_version: Some(created.playlist.version),
        },
    )
    .await;
    assert_eq!(added.playlist.item_count, 1);
    assert!(added.playlist.version > created.playlist.version);

    let detail_path = format!("/users/me/playlists/{playlist_id}");
    let detail = request_json::<nako_api::public_client::UserPlaylistResponse>(
        &router,
        Method::GET,
        &detail_path,
    )
    .await;
    assert_eq!(detail.playlist.item_count, 1);

    let items_path = format!("/users/me/playlists/{playlist_id}/items?limit=10&offset=0");
    let items = request_json::<nako_api::public_client::UserPlaylistItemsResponse>(
        &router,
        Method::GET,
        &items_path,
    )
    .await;
    assert_eq!(items.playlist.id, playlist_id);
    assert_eq!(items.page.returned, 1);
    assert_eq!(items.items[0].playlist_id, playlist_id);
    assert_eq!(items.items[0].item_id, source.item_id.to_string());
    assert_eq!(items.items[0].position, 0);
    assert_eq!(items.items[0].item.id, source.item_id.to_string());
    assert!(items.items[0].images.is_empty());

    let reordered = request_body_json::<nako_api::public_client::UserPlaylistResponse, _>(
        &router,
        Method::PUT,
        &format!("/users/me/playlists/{playlist_id}/items/reorder"),
        &nako_api::public_client::ReorderUserPlaylistItemsRequest {
            item_ids: vec![source.item_id.to_string()],
            expected_version: Some(added.playlist.version),
        },
    )
    .await;
    assert_eq!(reordered.playlist.item_count, 1);

    let raw_items = request_json::<serde_json::Value>(&router, Method::GET, &items_path).await;
    let raw = serde_json::to_string(&raw_items).unwrap();
    assert!(!raw.contains("principal_id"));
    assert!(!raw.contains("user_id"));
    assert!(!raw.contains("locator"));
    assert!(!raw.contains("playlist.m3u8"));

    let renamed = request_body_json::<nako_api::public_client::UserPlaylistResponse, _>(
        &router,
        Method::PATCH,
        &detail_path,
        &nako_api::public_client::UpdateUserPlaylistRequest {
            name: "Queue".to_owned(),
            expected_version: Some(reordered.playlist.version),
        },
    )
    .await;
    assert_eq!(renamed.playlist.name, "Queue");

    let removed = request_json::<nako_api::public_client::UserPlaylistResponse>(
        &router,
        Method::DELETE,
        &add_path,
    )
    .await;
    assert_eq!(removed.playlist.item_count, 0);

    let deleted = request_json::<nako_api::public_client::UserPlaylistDeleteResponse>(
        &router,
        Method::DELETE,
        &detail_path,
    )
    .await;
    assert!(deleted.deleted);
    assert_eq!(deleted.playlist_id, playlist_id);
}

#[tokio::test]
async fn user_playlist_item_routes_filter_and_enforce_effective_library_access() {
    let (_temp, app, visible, store) =
        app_with_media_source_config("visible-playlist.mkv", b"visible", |_| {}).await;
    let hidden = add_catalog_source(&store, LibraryId::new(), "hidden-playlist.mkv").await;
    let principal =
        local_viewer_with_library_access(&store, visible.library_id, LibraryAccessLevel::Browse)
            .await;
    let playlist = app
        .user_playlist()
        .create_playlist(crate::app::user_playlist::CreateUserPlaylistRequest {
            principal_id: principal.principal_id.clone(),
            name: "Filtered".to_owned(),
            created_at_ms: Some(1_774_800_000_000),
        })
        .await
        .unwrap();
    app.user_playlist()
        .add_item(crate::app::user_playlist::AddUserPlaylistItemRequest {
            principal: principal.clone(),
            playlist_id: playlist.id,
            item_id: visible.item_id,
            position: None,
            expected_version: None,
            added_at_ms: Some(1_774_800_001_000),
        })
        .await
        .unwrap();
    UserPlaylistRepository::add_user_playlist_item(
        &store,
        UserPlaylistItemWrite {
            playlist_id: playlist.id,
            principal_id: principal.principal_id.clone(),
            item_id: hidden.item_id,
            position: None,
            expected_version: None,
            added_at_ms: 1_774_800_002_000,
            updated_at_ms: 1_774_800_002_000,
        },
    )
    .await
    .unwrap()
    .unwrap();
    let router = public_client_router_with_principal(app, principal);
    let playlist_id = playlist.id.to_string();

    let items = request_json::<nako_api::public_client::UserPlaylistItemsResponse>(
        &router,
        Method::GET,
        &format!("/users/me/playlists/{playlist_id}/items?limit=10&offset=0"),
    )
    .await;

    assert_eq!(items.playlist.item_count, 1);
    assert_eq!(items.page.returned, 1);
    assert_eq!(items.items.len(), 1);
    assert_eq!(items.items[0].item_id, visible.item_id.to_string());
    assert_eq!(items.items[0].position, 0);

    let detail = request_json::<nako_api::public_client::UserPlaylistResponse>(
        &router,
        Method::GET,
        &format!("/users/me/playlists/{playlist_id}"),
    )
    .await;
    assert_eq!(detail.playlist.item_count, 1);

    let hidden_add = response_body_json(
        &router,
        Method::PUT,
        &format!("/users/me/playlists/{playlist_id}/items/{}", hidden.item_id),
        &nako_api::public_client::AddUserPlaylistItemRequest {
            position: None,
            expected_version: None,
        },
    )
    .await;
    assert_eq!(hidden_add.status(), StatusCode::FORBIDDEN);

    let hidden_remove = response_body_json(
        &router,
        Method::DELETE,
        &format!("/users/me/playlists/{playlist_id}/items/{}", hidden.item_id),
        &serde_json::json!({}),
    )
    .await;
    assert_eq!(hidden_remove.status(), StatusCode::FORBIDDEN);

    let hidden_reorder = response_body_json(
        &router,
        Method::PUT,
        &format!("/users/me/playlists/{playlist_id}/items/reorder"),
        &nako_api::public_client::ReorderUserPlaylistItemsRequest {
            item_ids: vec![hidden.item_id.to_string(), visible.item_id.to_string()],
            expected_version: None,
        },
    )
    .await;
    assert_eq!(hidden_reorder.status(), StatusCode::FORBIDDEN);
}

async fn add_catalog_source(
    store: &NakoDatabase,
    library_id: LibraryId,
    file_name: &str,
) -> MediaSource {
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: file_name.to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: format!("local:///{file_name}"),
        file_name: file_name.to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };

    store
        .upsert_library(&Library {
            id: library_id,
            name: file_name.to_owned(),
            roots: Vec::new(),
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
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

    source
}
