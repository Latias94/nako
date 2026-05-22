use nako_core::{DatabaseLifecycle, UserPlaybackStateRepository, UserPrincipalId};

use super::*;
use crate::app::user_playback::{
    SetUserWatchedStateRequest, UpdateUserPlaybackProgressRequest, UserPlaybackAppService,
};

#[tokio::test]
async fn user_playback_get_state_returns_default_for_existing_item_without_state() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = UserPrincipalId::local_admin();

    let state = app
        .user_playback()
        .get_state(&principal, source.item_id)
        .await
        .unwrap();

    assert_eq!(state.principal_id, principal);
    assert_eq!(state.item_id, source.item_id);
    assert_eq!(state.source_id, None);
    assert_eq!(state.resume_position_ms, None);
    assert!(!state.watched);
    assert_eq!(state.updated_at_ms, 0);
    assert_eq!(state.version, 0);
}

#[tokio::test]
async fn user_playback_progress_persists_resume_until_watched_threshold() {
    let (service, store, source) = user_playback_service_with_source().await;
    let principal = UserPrincipalId::local_admin();

    let resume = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: principal.clone(),
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 120_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(1_000),
        })
        .await
        .unwrap();

    assert_eq!(resume.principal_id, principal);
    assert_eq!(resume.item_id, source.item_id);
    assert_eq!(resume.source_id, Some(source.id));
    assert_eq!(resume.resume_position_ms, Some(120_000));
    assert_eq!(resume.duration_ms, Some(600_000));
    assert_eq!(resume.last_played_at_ms, Some(1_000));
    assert!(!resume.watched);
    assert_eq!(resume.watched_at_ms, None);

    let watched = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: resume.principal_id.clone(),
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 540_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(2_000),
        })
        .await
        .unwrap();

    assert!(watched.watched);
    assert_eq!(watched.watched_at_ms, Some(2_000));
    assert_eq!(watched.resume_position_ms, None);
    assert_eq!(
        store
            .list_continue_watching_states(&watched.principal_id, PageRequest::first_page())
            .await
            .unwrap(),
        Vec::new()
    );
}

#[tokio::test]
async fn user_playback_explicit_unwatch_does_not_invent_resume() {
    let (service, _store, source) = user_playback_service_with_source().await;
    let principal = UserPrincipalId::local_admin();

    let watched = service
        .set_watched_state(SetUserWatchedStateRequest {
            principal_id: principal,
            item_id: source.item_id,
            watched: true,
            source_id: Some(source.id),
            position_ms: Some(600_000),
            duration_ms: Some(600_000),
            marked_at_ms: Some(1_000),
        })
        .await
        .unwrap();

    assert!(watched.watched);
    assert_eq!(watched.resume_position_ms, None);

    let unwatched = service
        .set_watched_state(SetUserWatchedStateRequest {
            principal_id: watched.principal_id.clone(),
            item_id: source.item_id,
            watched: false,
            source_id: Some(source.id),
            position_ms: None,
            duration_ms: Some(600_000),
            marked_at_ms: Some(2_000),
        })
        .await
        .unwrap();

    assert!(!unwatched.watched);
    assert_eq!(unwatched.watched_at_ms, None);
    assert_eq!(unwatched.resume_position_ms, None);
}

#[tokio::test]
async fn user_playback_ignores_older_progress_reports() {
    let (service, _store, source) = user_playback_service_with_source().await;
    let principal = UserPrincipalId::local_admin();

    let current = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: principal,
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 300_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(2_000),
        })
        .await
        .unwrap();
    let stale = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: current.principal_id.clone(),
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 100_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(1_000),
        })
        .await
        .unwrap();

    assert_eq!(stale, current);
}

#[tokio::test]
async fn user_playback_continue_watching_is_principal_scoped_and_sorted() {
    let (service, store, first) = user_playback_service_with_source().await;
    let second = add_source(
        &store,
        "Evening Current",
        "local:///Movies/Evening Current.mkv",
    )
    .await;
    let principal = UserPrincipalId::local_admin();
    let other_principal = UserPrincipalId::new("second-profile").unwrap();

    service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: principal.clone(),
            item_id: first.item_id,
            source_id: Some(first.id),
            position_ms: 60_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(1_000),
        })
        .await
        .unwrap();
    service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: principal.clone(),
            item_id: second.item_id,
            source_id: Some(second.id),
            position_ms: 120_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(2_000),
        })
        .await
        .unwrap();
    service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: other_principal.clone(),
            item_id: first.item_id,
            source_id: Some(first.id),
            position_ms: 180_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(3_000),
        })
        .await
        .unwrap();

    let states = service
        .list_continue_watching(&principal, PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        states.iter().map(|state| state.item_id).collect::<Vec<_>>(),
        vec![second.item_id, first.item_id]
    );
    assert!(states.iter().all(|state| state.principal_id == principal));
}

#[tokio::test]
async fn user_playback_progress_does_not_auto_unwatch_existing_watched_state() {
    let (service, _store, source) = user_playback_service_with_source().await;
    let principal = UserPrincipalId::local_admin();

    let watched = service
        .set_watched_state(SetUserWatchedStateRequest {
            principal_id: principal,
            item_id: source.item_id,
            watched: true,
            source_id: Some(source.id),
            position_ms: Some(600_000),
            duration_ms: Some(600_000),
            marked_at_ms: Some(1_000),
        })
        .await
        .unwrap();
    let progress = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal_id: watched.principal_id.clone(),
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 60_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(2_000),
        })
        .await
        .unwrap();

    assert!(progress.watched);
    assert_eq!(progress.watched_at_ms, watched.watched_at_ms);
    assert_eq!(progress.resume_position_ms, None);
}

#[tokio::test]
async fn user_playback_state_progress_is_idempotent_for_identical_event() {
    let (service, _store, source) = user_playback_service_with_source().await;
    let principal = UserPrincipalId::local_admin();
    let request = UpdateUserPlaybackProgressRequest {
        principal_id: principal,
        item_id: source.item_id,
        source_id: Some(source.id),
        position_ms: 120_000,
        duration_ms: Some(600_000),
        reported_at_ms: Some(1_000),
    };

    let first = service.update_progress(request.clone()).await.unwrap();
    let second = service.update_progress(request).await.unwrap();

    assert_eq!(second, first);
}

async fn user_playback_service_with_source() -> (UserPlaybackAppService, NakoDatabase, MediaSource)
{
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Night Harbor".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Night Harbor.mkv".to_owned(),
        file_name: "Night Harbor.mkv".to_owned(),
        size_bytes: Some(128),
        fingerprint: Some("night-harbor".to_owned()),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    (UserPlaybackAppService::new(store.clone()), store, source)
}

async fn add_source(store: &NakoDatabase, title: &str, locator: &str) -> MediaSource {
    let library = Library {
        id: LibraryId::new(),
        name: title.to_owned(),
        roots: vec![locator.to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
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
        library_id: library.id,
        item_id: item.id,
        locator: locator.to_owned(),
        file_name: format!("{title}.mkv"),
        size_bytes: Some(128),
        fingerprint: Some(title.to_owned()),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    source
}
