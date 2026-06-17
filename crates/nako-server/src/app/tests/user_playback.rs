use tokio::sync::Mutex;

use nako_core::{
    AuthenticatedPrincipal, ContinueWatchingEntry, DatabaseLifecycle, LibraryItemRepository,
    LibraryItemState, UserPlaybackProfilePreference, UserPlaybackProfilePreferenceWrite,
    UserPlaybackState, UserPlaybackStateRepository, UserPlaybackStateWrite, UserPrincipalId,
};

use super::*;
use crate::app::user_playback::{
    SetUserWatchedStateRequest, UpdateUserPlaybackProgressRequest, UserPlaybackAppService,
    UserPlaybackStore,
};

#[tokio::test]
async fn user_playback_get_state_returns_default_for_existing_item_without_state() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, _store, source) = remux_app_with_source(ffmpeg_path).await;
    let principal = AuthenticatedPrincipal::bootstrap_admin();

    let state = app
        .user_playback()
        .get_state(&principal, source.item_id)
        .await
        .unwrap();

    assert_eq!(state.principal_id, principal.principal_id);
    assert_eq!(state.item_id, source.item_id);
    assert_eq!(state.source_id, None);
    assert_eq!(state.resume_position_ms, None);
    assert!(!state.watched);
    assert_eq!(state.updated_at_ms, 0);
    assert_eq!(state.version, 0);
}

#[tokio::test]
async fn user_playback_service_uses_the_focused_store_port() {
    let store = FakeUserPlaybackStore::new();
    let service = UserPlaybackAppService::new(store.clone());
    let principal = AuthenticatedPrincipal::bootstrap_admin();

    let updated = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal: principal.clone(),
            item_id: store.item.id,
            source_id: Some(store.source.id),
            position_ms: 120_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(1_000),
        })
        .await
        .unwrap();

    let loaded = service.get_state(&principal, store.item.id).await.unwrap();

    assert_eq!(loaded, updated);
    assert_eq!(store.state.lock().await.clone().unwrap(), updated);
}

#[tokio::test]
async fn user_playback_progress_persists_resume_until_watched_threshold() {
    let (service, store, source) = user_playback_service_with_source().await;
    let principal = AuthenticatedPrincipal::bootstrap_admin();

    let resume = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal: principal.clone(),
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 120_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(1_000),
        })
        .await
        .unwrap();

    assert_eq!(resume.principal_id, principal.principal_id);
    assert_eq!(resume.item_id, source.item_id);
    assert_eq!(resume.source_id, Some(source.id));
    assert_eq!(resume.resume_position_ms, Some(120_000));
    assert_eq!(resume.duration_ms, Some(600_000));
    assert_eq!(resume.last_played_at_ms, Some(1_000));
    assert!(!resume.watched);
    assert_eq!(resume.watched_at_ms, None);

    let watched = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal: principal.clone(),
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
        UserPlaybackStateRepository::list_continue_watching_states(
            &store,
            &watched.principal_id,
            PageRequest::first_page(),
        )
        .await
        .unwrap(),
        Vec::new()
    );
}

#[tokio::test]
async fn user_playback_explicit_unwatch_does_not_invent_resume() {
    let (service, _store, source) = user_playback_service_with_source().await;
    let principal = AuthenticatedPrincipal::bootstrap_admin();

    let watched = service
        .set_watched_state(SetUserWatchedStateRequest {
            principal: principal.clone(),
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
            principal,
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
    let principal = AuthenticatedPrincipal::bootstrap_admin();

    let current = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal: principal.clone(),
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
            principal,
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
    let principal = AuthenticatedPrincipal::bootstrap_admin();
    let other_principal = AuthenticatedPrincipal {
        user_id: UserId::new(),
        principal_id: UserPrincipalId::new("second-profile").unwrap(),
        roles: vec![UserRole::Administrator],
        bootstrap: false,
    };

    service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal: principal.clone(),
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
            principal: principal.clone(),
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
            principal: other_principal,
            item_id: first.item_id,
            source_id: Some(first.id),
            position_ms: 180_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(3_000),
        })
        .await
        .unwrap();

    let states = service
        .list_continue_watching(&principal.principal_id, PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        states.iter().map(|state| state.item_id).collect::<Vec<_>>(),
        vec![second.item_id, first.item_id]
    );
    assert!(
        states
            .iter()
            .all(|state| state.principal_id == principal.principal_id)
    );
}

#[tokio::test]
async fn user_playback_progress_does_not_auto_unwatch_existing_watched_state() {
    let (service, _store, source) = user_playback_service_with_source().await;
    let principal = AuthenticatedPrincipal::bootstrap_admin();

    let watched = service
        .set_watched_state(SetUserWatchedStateRequest {
            principal: principal.clone(),
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
            principal,
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
    let principal = AuthenticatedPrincipal::bootstrap_admin();
    let request = UpdateUserPlaybackProgressRequest {
        principal,
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

#[tokio::test]
async fn user_playback_service_enforces_library_access_for_reads_and_writes() {
    let (service, store, source) = user_playback_service_with_source().await;
    let no_access = local_principal_with_library_access(
        &store,
        source.library_id,
        UserRole::Viewer,
        LibraryAccessLevel::None,
    )
    .await;
    let browse_only = local_principal_with_library_access(
        &store,
        source.library_id,
        UserRole::Viewer,
        LibraryAccessLevel::Browse,
    )
    .await;
    let playable = local_principal_with_library_access(
        &store,
        source.library_id,
        UserRole::Viewer,
        LibraryAccessLevel::Play,
    )
    .await;

    assert!(matches!(
        service.get_state(&no_access, source.item_id).await,
        Err(NakoError::Forbidden { .. })
    ));

    let read = service
        .get_state(&browse_only, source.item_id)
        .await
        .unwrap();
    assert_eq!(read.principal_id, browse_only.principal_id);
    assert!(matches!(
        service
            .update_progress(UpdateUserPlaybackProgressRequest {
                principal: browse_only.clone(),
                item_id: source.item_id,
                source_id: Some(source.id),
                position_ms: 120_000,
                duration_ms: Some(600_000),
                reported_at_ms: Some(1_000),
            })
            .await,
        Err(NakoError::Forbidden { .. })
    ));
    assert!(matches!(
        service
            .set_watched_state(SetUserWatchedStateRequest {
                principal: browse_only,
                item_id: source.item_id,
                watched: true,
                source_id: Some(source.id),
                position_ms: Some(600_000),
                duration_ms: Some(600_000),
                marked_at_ms: Some(2_000),
            })
            .await,
        Err(NakoError::Forbidden { .. })
    ));

    let updated = service
        .update_progress(UpdateUserPlaybackProgressRequest {
            principal: playable,
            item_id: source.item_id,
            source_id: Some(source.id),
            position_ms: 120_000,
            duration_ms: Some(600_000),
            reported_at_ms: Some(3_000),
        })
        .await
        .unwrap();
    assert_eq!(updated.resume_position_ms, Some(120_000));
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
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
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
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();

    source
}

async fn local_principal_with_library_access(
    store: &NakoDatabase,
    library_id: LibraryId,
    role: UserRole,
    access: LibraryAccessLevel,
) -> AuthenticatedPrincipal {
    let user_id = UserId::new();
    let principal_id = UserPrincipalId::new(format!("local-user:{user_id}")).unwrap();
    let user = User {
        id: user_id,
        principal_id: principal_id.clone(),
        username: format!("{}-{}", role.as_str(), user_id),
        display_name: "Library principal".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1,
        updated_at_ms: 1,
    };

    store.upsert_user(&user).await.unwrap();
    store
        .replace_role_assignments(
            user_id,
            &[RoleAssignment {
                user_id,
                role,
                granted_at_ms: 1,
            }],
        )
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&LibraryAccessPolicy {
            scope: LibraryAccessPolicyScope::User(user_id),
            library_id,
            access,
            created_at_ms: 1,
            updated_at_ms: 1,
        })
        .await
        .unwrap();

    AuthenticatedPrincipal {
        user_id,
        principal_id,
        roles: vec![role],
        bootstrap: false,
    }
}

#[derive(Clone, Debug)]
struct FakeUserPlaybackStore {
    item: MediaItem,
    source: MediaSource,
    state: Arc<Mutex<Option<UserPlaybackState>>>,
    profile_preference: Arc<Mutex<Option<UserPlaybackProfilePreference>>>,
}

impl FakeUserPlaybackStore {
    fn new() -> Self {
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

        Self {
            item,
            source,
            state: Arc::new(Mutex::new(None)),
            profile_preference: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl UserPlaybackStore for FakeUserPlaybackStore {
    async fn load_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> nako_core::Result<Option<UserPlaybackState>> {
        let state = self.state.lock().await;
        Ok(state
            .clone()
            .filter(|state| state.principal_id == *principal_id && state.item_id == item_id))
    }

    async fn store_user_playback_state(
        &self,
        write: UserPlaybackStateWrite,
    ) -> nako_core::Result<UserPlaybackState> {
        let state = UserPlaybackState {
            principal_id: write.principal_id.clone(),
            item_id: write.item_id,
            source_id: write.source_id,
            resume_position_ms: write.resume_position_ms,
            duration_ms: write.duration_ms,
            watched: write.watched,
            watched_at_ms: write.watched_at_ms,
            last_played_at_ms: write.last_played_at_ms,
            updated_at_ms: write.updated_at_ms,
            version: 1,
        };
        *self.state.lock().await = Some(state.clone());
        Ok(state)
    }

    async fn load_user_playback_profile_preference(
        &self,
        principal_id: &UserPrincipalId,
    ) -> nako_core::Result<Option<UserPlaybackProfilePreference>> {
        let preference = self.profile_preference.lock().await;
        Ok(preference
            .clone()
            .filter(|preference| preference.principal_id == *principal_id))
    }

    async fn store_user_playback_profile_preference(
        &self,
        write: UserPlaybackProfilePreferenceWrite,
    ) -> nako_core::Result<UserPlaybackProfilePreference> {
        let current_version = self
            .profile_preference
            .lock()
            .await
            .as_ref()
            .filter(|preference| preference.principal_id == write.principal_id)
            .map_or(0, |preference| preference.version);
        let preference = UserPlaybackProfilePreference {
            principal_id: write.principal_id,
            capabilities_json: write.capabilities_json,
            updated_at_ms: write.updated_at_ms,
            version: current_version + 1,
        };
        *self.profile_preference.lock().await = Some(preference.clone());
        Ok(preference)
    }

    async fn delete_user_playback_profile_preference(
        &self,
        principal_id: &UserPrincipalId,
    ) -> nako_core::Result<bool> {
        let mut preference = self.profile_preference.lock().await;
        if preference
            .as_ref()
            .is_some_and(|preference| preference.principal_id == *principal_id)
        {
            *preference = None;
            return Ok(true);
        }
        Ok(false)
    }

    async fn list_continue_watching_user_playback_states(
        &self,
        principal_id: &UserPrincipalId,
        page: nako_core::PageRequest,
    ) -> nako_core::Result<Vec<UserPlaybackState>> {
        let state = self.state.lock().await.clone();
        let mut states = state
            .into_iter()
            .filter(|state| state.principal_id == *principal_id && !state.watched)
            .collect::<Vec<_>>();
        states.truncate(page.limit as usize);
        Ok(states)
    }

    async fn list_continue_watching_entries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: nako_core::PageRequest,
    ) -> nako_core::Result<Vec<ContinueWatchingEntry>> {
        let states = self
            .list_continue_watching_user_playback_states(&principal.principal_id, page)
            .await?;

        Ok(states
            .into_iter()
            .filter(|state| state.item_id == self.item.id)
            .map(|state| ContinueWatchingEntry {
                state,
                item: self.item.clone(),
                images: Vec::new(),
            })
            .collect())
    }

    async fn load_media_item(&self, item_id: MediaItemId) -> nako_core::Result<Option<MediaItem>> {
        Ok((self.item.id == item_id).then_some(self.item.clone()))
    }

    async fn load_media_source(
        &self,
        source_id: MediaSourceId,
    ) -> nako_core::Result<Option<MediaSource>> {
        Ok((self.source.id == source_id).then_some(self.source.clone()))
    }

    async fn list_item_sources(
        &self,
        item_id: MediaItemId,
        page: nako_core::PageRequest,
    ) -> nako_core::Result<Vec<MediaSource>> {
        if self.source.item_id == item_id && page.offset == 0 && page.limit > 0 {
            Ok(vec![self.source.clone()])
        } else {
            Ok(Vec::new())
        }
    }

    async fn resolve_library_access_level(
        &self,
        _user_id: UserId,
        _library_id: LibraryId,
    ) -> nako_core::Result<LibraryAccessLevel> {
        Ok(LibraryAccessLevel::Manage)
    }
}
