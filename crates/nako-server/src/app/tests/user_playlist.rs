use nako_core::{NakoError, UserPlaylistVisibility, UserPrincipalId};

use super::*;
use crate::app::user_playlist::{
    AddUserPlaylistItemRequest, CreateUserPlaylistRequest, RemoveUserPlaylistItemRequest,
    RenameUserPlaylistRequest, ReorderUserPlaylistItemsRequest, UserPlaylistAppService,
};

#[tokio::test]
async fn user_playlist_service_creates_orders_and_reorders_private_media_state() {
    let (service, _store, first, second) = user_playlist_service_with_items().await;
    let principal = UserPrincipalId::local_admin();

    let created = service
        .create_playlist(CreateUserPlaylistRequest {
            principal_id: principal.clone(),
            name: " Weekend queue ".to_owned(),
            created_at_ms: Some(1_000),
        })
        .await
        .unwrap();

    assert_eq!(created.principal_id, principal);
    assert_eq!(created.name, "Weekend queue");
    assert_eq!(created.visibility, UserPlaylistVisibility::Private);
    assert_eq!(created.item_count, 0);
    assert_eq!(created.version, 1);
    assert_eq!(
        service
            .list_playlists(&principal, PageRequest::first_page())
            .await
            .unwrap(),
        vec![created.clone()]
    );

    let first_add = service
        .add_item(AddUserPlaylistItemRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_id: first.item_id,
            position: None,
            expected_version: Some(created.version),
            added_at_ms: Some(1_100),
        })
        .await
        .unwrap();
    assert_eq!(first_add.version, 2);
    assert_eq!(first_add.item_count, 1);

    let duplicate_add = service
        .add_item(AddUserPlaylistItemRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_id: first.item_id,
            position: None,
            expected_version: Some(first_add.version),
            added_at_ms: Some(1_200),
        })
        .await
        .unwrap();
    assert_eq!(duplicate_add.version, first_add.version);
    assert_eq!(duplicate_add.item_count, 1);

    let second_add = service
        .add_item(AddUserPlaylistItemRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_id: second.item_id,
            position: Some(0),
            expected_version: Some(duplicate_add.version),
            added_at_ms: Some(1_300),
        })
        .await
        .unwrap();
    assert_eq!(second_add.version, 3);

    assert_eq!(
        service
            .list_items(&principal, created.id, PageRequest::first_page())
            .await
            .unwrap()
            .iter()
            .map(|item| (item.item_id, item.position))
            .collect::<Vec<_>>(),
        vec![(second.item_id, 0), (first.item_id, 1)]
    );

    let reordered = service
        .reorder_items(ReorderUserPlaylistItemsRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_ids: vec![first.item_id, second.item_id],
            expected_version: Some(second_add.version),
            updated_at_ms: Some(1_400),
        })
        .await
        .unwrap();
    assert_eq!(reordered.version, 4);
    assert_eq!(
        service
            .list_items(&principal, created.id, PageRequest::first_page())
            .await
            .unwrap()
            .iter()
            .map(|item| (item.item_id, item.position))
            .collect::<Vec<_>>(),
        vec![(first.item_id, 0), (second.item_id, 1)]
    );
}

#[tokio::test]
async fn user_playlist_service_rejects_invalid_or_stale_mutations() {
    let (service, _store, first, second) = user_playlist_service_with_items().await;
    let principal = UserPrincipalId::local_admin();
    let other_principal = UserPrincipalId::new("other-profile").unwrap();
    let created = service
        .create_playlist(CreateUserPlaylistRequest {
            principal_id: principal.clone(),
            name: "Queue".to_owned(),
            created_at_ms: Some(1_000),
        })
        .await
        .unwrap();
    let first_add = service
        .add_item(AddUserPlaylistItemRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_id: first.item_id,
            position: None,
            expected_version: Some(created.version),
            added_at_ms: Some(1_100),
        })
        .await
        .unwrap();
    let second_add = service
        .add_item(AddUserPlaylistItemRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_id: second.item_id,
            position: None,
            expected_version: Some(first_add.version),
            added_at_ms: Some(1_200),
        })
        .await
        .unwrap();

    assert!(matches!(
        service.get_playlist(&other_principal, created.id).await,
        Err(NakoError::NotFound {
            entity: "user_playlist",
            ..
        })
    ));
    assert!(matches!(
        service
            .create_playlist(CreateUserPlaylistRequest {
                principal_id: principal.clone(),
                name: "   ".to_owned(),
                created_at_ms: Some(1_300),
            })
            .await,
        Err(NakoError::InvalidInput { .. })
    ));
    assert!(matches!(
        service
            .rename_playlist(RenameUserPlaylistRequest {
                principal_id: principal.clone(),
                playlist_id: created.id,
                name: "Stale".to_owned(),
                expected_version: Some(1),
                updated_at_ms: Some(1_400),
            })
            .await,
        Err(NakoError::Conflict { .. })
    ));
    assert!(matches!(
        service
            .reorder_items(ReorderUserPlaylistItemsRequest {
                principal_id: principal.clone(),
                playlist_id: created.id,
                item_ids: vec![first.item_id, first.item_id],
                expected_version: Some(second_add.version),
                updated_at_ms: Some(1_500),
            })
            .await,
        Err(NakoError::InvalidInput { .. })
    ));

    let after_remove = service
        .remove_item(RemoveUserPlaylistItemRequest {
            principal_id: principal.clone(),
            playlist_id: created.id,
            item_id: second.item_id,
            expected_version: Some(second_add.version),
            updated_at_ms: Some(1_600),
        })
        .await
        .unwrap();
    assert_eq!(after_remove.item_count, 1);

    service
        .delete_playlist(&principal, created.id)
        .await
        .unwrap();
    assert!(matches!(
        service.get_playlist(&principal, created.id).await,
        Err(NakoError::NotFound {
            entity: "user_playlist",
            ..
        })
    ));
}

#[tokio::test]
async fn user_playlist_service_is_available_from_app_composition() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = UserPrincipalId::local_admin();
    let service = app.user_playlist();

    let playlist = service
        .create_playlist(CreateUserPlaylistRequest {
            principal_id: principal.clone(),
            name: "Composed queue".to_owned(),
            created_at_ms: Some(1_000),
        })
        .await
        .unwrap();
    let added = service
        .add_item(AddUserPlaylistItemRequest {
            principal_id: principal,
            playlist_id: playlist.id,
            item_id: source.item_id,
            position: None,
            expected_version: Some(playlist.version),
            added_at_ms: Some(1_100),
        })
        .await
        .unwrap();

    assert_eq!(added.item_count, 1);
}

async fn user_playlist_service_with_items() -> (
    UserPlaylistAppService,
    NakoDatabase,
    MediaSource,
    MediaSource,
) {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();
    let first = add_playlist_source(&store, library.id, "First").await;
    let second = add_playlist_source(&store, library.id, "Second").await;

    (
        UserPlaylistAppService::new(store.clone()),
        store,
        first,
        second,
    )
}

async fn add_playlist_source(
    store: &NakoDatabase,
    library_id: LibraryId,
    title: &str,
) -> MediaSource {
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: format!("local:///Movies/{title}.mkv"),
        file_name: format!("{title}.mkv"),
        size_bytes: Some(128),
        fingerprint: Some(title.to_owned()),
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    source
}
