use taru_core::*;
use taru_search::{SearchDocument, SearchIndex, SearchQuery};

use crate::{
    DatabaseBackendCapabilities, DatabaseBackendKind, DatabaseConnectOptions, TaruDatabase,
};

#[tokio::test]
async fn taru_database_connect_options_select_sqlite_explicitly() {
    let store =
        TaruDatabase::connect_with_options(DatabaseConnectOptions::sqlite("sqlite::memory:"))
            .await
            .unwrap();

    assert_eq!(store.backend_kind(), DatabaseBackendKind::Sqlite);
    assert_eq!(
        store.capabilities(),
        DatabaseBackendCapabilities::production_ready()
    );
}

#[tokio::test]
async fn taru_database_postgres_runtime_capabilities_include_managed_artwork() {
    assert_eq!(
        DatabaseBackendCapabilities::postgres_supported_scope(),
        DatabaseBackendCapabilities {
            lifecycle: true,
            libraries: true,
            jobs: true,
            job_leases: true,
            media: true,
            scan_commits: true,
            metadata: true,
            catalog: true,
            playback_state: true,
            transcode_sessions: true,
            event_outbox: true,
            addons: true,
            automation: true,
            managed_artwork: true,
            vfs_cache: true,
            webhooks: true,
            search_index: true,
        }
    );
}

#[tokio::test]
async fn taru_database_postgres_selection_enters_runtime_connection_path() {
    let err =
        TaruDatabase::connect_with_options(DatabaseConnectOptions::postgres("not-a-postgres-url"))
            .await
            .unwrap_err();

    assert!(matches!(err, TaruError::Database { .. }));
    assert!(
        !err.to_string()
            .contains("PostgreSQL runtime backend is gated")
    );
}

#[tokio::test]
async fn taru_database_sqlite_persists_libraries() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };

    store.upsert_library(&library).await.unwrap();
    let loaded = store.get_library(library.id).await.unwrap();

    assert_eq!(loaded, Some(library));
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_library_profiles() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let mut options = LibraryOptions::from_preset(LibraryPreset::Anime);
    options.metadata_profile.refresh_mode = MetadataRefreshMode::MissingOnly;
    options.metadata_profile.metadata_providers = vec![
        ExternalProvider::Bangumi,
        ExternalProvider::Tmdb,
        ExternalProvider::Douban,
    ];
    let library = Library {
        id: LibraryId::new(),
        name: "Anime".to_owned(),
        roots: vec!["local:///Anime".to_owned()],
        options,
    };

    store.upsert_library(&library).await.unwrap();

    assert_eq!(store.get_library(library.id).await.unwrap(), Some(library));
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_media_items_and_sources() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            original_title: None,
            sort_title: Some("Matrix, The".to_owned()),
            overview: Some("A hacker discovers the nature of reality.".to_owned()),
            release_date: Some("1999-03-31".to_owned()),
            runtime_minutes: Some(136),
            tagline: Some("Welcome to the Real World".to_owned()),
            genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
            tags: vec!["cyberpunk".to_owned()],
            ratings: vec![ContentRating {
                source: "MPAA".to_owned(),
                value: "R".to_owned(),
            }],
            images: vec![ImageRef {
                kind: ImageKind::Poster,
                uri: "https://image.example/poster.jpg".to_owned(),
                provider: ExternalProvider::Tmdb,
                width: Some(1000),
                height: Some(1500),
                language: Some("en".to_owned()),
            }],
            credits: vec![Credit {
                name: "Keanu Reeves".to_owned(),
                role: CreditRole::Actor,
                character: Some("Neo".to_owned()),
                order: Some(0),
                external_ids: Vec::new(),
            }],
            collections: Vec::new(),
            studios: Vec::new(),
            external_ids: vec![
                ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "603".to_owned(),
                },
                ExternalId {
                    provider: ExternalProvider::Other("custom".to_owned()),
                    value: "matrix-local".to_owned(),
                },
            ],
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/The Matrix (1999).mkv".to_owned(),
        file_name: "The Matrix (1999).mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("fingerprint".to_owned()),
    };

    let mut expected_item = item.clone();
    expected_item
        .metadata
        .external_ids
        .sort_by_key(|external_id| format!("{:?}\0{}", external_id.provider, external_id.value));

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    assert_eq!(
        store.get_media_item(item.id).await.unwrap(),
        Some(expected_item)
    );
    assert_eq!(
        store.get_media_source(source.id).await.unwrap(),
        Some(source.clone())
    );
    assert_eq!(
        store
            .get_media_source(source.id)
            .await
            .unwrap()
            .map(|source| source.library_id),
        Some(library.id)
    );
    assert_eq!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![source]
    );
}

#[tokio::test]
async fn taru_database_sqlite_persists_user_playback_state_by_principal() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
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
    let principal = UserPrincipalId::local_admin();
    let other_principal = UserPrincipalId::new("second-profile").unwrap();

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let state = store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.clone(),
            item_id: item.id,
            source_id: Some(source.id),
            resume_position_ms: Some(42_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(1_000),
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();

    assert_eq!(state.principal_id, principal);
    assert_eq!(state.item_id, item.id);
    assert_eq!(state.source_id, Some(source.id));
    assert_eq!(state.resume_position_ms, Some(42_000));
    assert_eq!(state.duration_ms, Some(600_000));
    assert!(!state.watched);
    assert_eq!(state.version, 1);
    assert_eq!(
        store
            .get_user_playback_state(&state.principal_id, item.id)
            .await
            .unwrap(),
        Some(state.clone())
    );
    assert_eq!(
        store
            .get_user_playback_state(&other_principal, item.id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .list_continue_watching_states(&state.principal_id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![state]
    );
}

#[tokio::test]
async fn taru_database_sqlite_continue_watching_filters_watched_zero_and_other_principals() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let principal = UserPrincipalId::local_admin();
    let other_principal = UserPrincipalId::new("second-profile").unwrap();
    store.upsert_library(&library).await.unwrap();

    let first = seed_media_item_with_source(&store, library.id, "First").await;
    let second = seed_media_item_with_source(&store, library.id, "Second").await;
    let watched = seed_media_item_with_source(&store, library.id, "Watched").await;
    let zero = seed_media_item_with_source(&store, library.id, "Zero").await;
    let other = seed_media_item_with_source(&store, library.id, "Other").await;

    for (source, principal_id, resume_position_ms, watched, last_played_at_ms) in [
        (&first, principal.clone(), Some(60_000), false, Some(1_000)),
        (
            &second,
            principal.clone(),
            Some(120_000),
            false,
            Some(2_000),
        ),
        (&watched, principal.clone(), None, true, Some(3_000)),
        (&zero, principal.clone(), None, false, Some(4_000)),
        (&other, other_principal, Some(180_000), false, Some(5_000)),
    ] {
        store
            .upsert_user_playback_state(UserPlaybackStateWrite {
                principal_id,
                item_id: source.item_id,
                source_id: Some(source.id),
                resume_position_ms,
                duration_ms: Some(600_000),
                watched,
                watched_at_ms: watched.then_some(last_played_at_ms.unwrap()),
                last_played_at_ms,
                updated_at_ms: last_played_at_ms.unwrap(),
            })
            .await
            .unwrap();
    }

    let states = store
        .list_continue_watching_states(&principal, PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        states.iter().map(|state| state.item_id).collect::<Vec<_>>(),
        vec![second.item_id, first.item_id]
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_video_item_hierarchy_and_multiple_sources() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "TV".to_owned(),
        roots: vec!["local:///TV".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Tv),
    };
    let series = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Example Series".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let season = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Season,
        parent_id: Some(series.id),
        metadata: CanonicalMetadata {
            title: "Season 1".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let episode = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Episode,
        parent_id: Some(season.id),
        metadata: CanonicalMetadata {
            title: "Episode 1".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let extra = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Extra,
        parent_id: Some(series.id),
        metadata: CanonicalMetadata {
            title: "Behind the Scenes".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let unknown = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Unknown,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Unclassified Clip".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let episode_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: episode.id,
        locator: "local:///TV/Example Series/S01E01.mkv".to_owned(),
        file_name: "S01E01.mkv".to_owned(),
        size_bytes: Some(100),
        fingerprint: Some("episode-fingerprint".to_owned()),
    };
    let alternate_episode_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: episode.id,
        locator: "local:///TV/Example Series/S01E01.z-remux.mkv".to_owned(),
        file_name: "S01E01.z-remux.mkv".to_owned(),
        size_bytes: Some(200),
        fingerprint: Some("episode-remux-fingerprint".to_owned()),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&series).await.unwrap();
    store.upsert_media_item(&season).await.unwrap();
    store.upsert_media_item(&episode).await.unwrap();
    store.upsert_media_item(&extra).await.unwrap();
    store.upsert_media_item(&unknown).await.unwrap();
    store.upsert_media_source(&episode_source).await.unwrap();
    store
        .upsert_media_source(&alternate_episode_source)
        .await
        .unwrap();

    assert_eq!(store.get_media_item(series.id).await.unwrap(), Some(series));
    assert_eq!(store.get_media_item(season.id).await.unwrap(), Some(season));
    assert_eq!(
        store.get_media_item(episode.id).await.unwrap(),
        Some(episode.clone())
    );
    assert_eq!(store.get_media_item(extra.id).await.unwrap(), Some(extra));
    assert_eq!(
        store.get_media_item(unknown.id).await.unwrap(),
        Some(unknown)
    );
    assert_eq!(
        store
            .list_item_sources(episode.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![episode_source, alternate_episode_source]
    );
}

#[tokio::test]
async fn taru_database_sqlite_tracks_source_less_library_items() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "TV".to_owned(),
        roots: vec!["local:///TV".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Tv),
    };
    let series = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Example Series".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let state = LibraryItemState {
        library_id: library.id,
        item_id: series.id,
        provisional: true,
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&series).await.unwrap();
    store.upsert_library_item_state(&state).await.unwrap();

    assert_eq!(
        store
            .get_library_item_state(library.id, series.id)
            .await
            .unwrap(),
        Some(state)
    );
    assert_eq!(
        store
            .find_library_item_by_kind_parent_title(
                library.id,
                MediaKind::Series,
                None,
                "Example Series",
            )
            .await
            .unwrap(),
        Some(series.clone())
    );
    assert_eq!(
        store
            .list_media_items_for_library(library.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![series]
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_transcode_sessions() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Session Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Session Demo.mkv".to_owned(),
        file_name: "Session Demo.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let session_id = TranscodeSessionId::new();
    let request_key = "test-transcode-profile:remux-active".to_owned();

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let planned = store
        .create_transcode_session(NewTranscodeSession {
            id: session_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: request_key.clone(),
            output_path: "cache/remux/stream.mp4".into(),
            state: TranscodeSessionState::Planned,
        })
        .await
        .unwrap();

    assert_eq!(planned.id, session_id);
    assert_eq!(planned.state, TranscodeSessionState::Planned);
    assert!(planned.started_at.is_none());
    assert!(planned.completed_at.is_none());
    assert_eq!(
        store
            .find_active_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key,)
            .await
            .unwrap()
            .unwrap()
            .id,
        session_id
    );

    let running = store
        .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
        .await
        .unwrap();

    assert_eq!(running.state, TranscodeSessionState::Running);
    assert!(running.started_at.is_some());

    let finished = store
        .set_transcode_session_state(session_id, TranscodeSessionState::Finished, None, None)
        .await
        .unwrap();

    assert_eq!(finished.state, TranscodeSessionState::Finished);
    assert!(finished.completed_at.is_some());
    assert!(
        store
            .find_active_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key,)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key,)
            .await
            .unwrap()
            .unwrap()
            .id,
        session_id
    );
}

#[tokio::test]
async fn taru_database_sqlite_lists_transcode_sessions_with_filters_and_pagination() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Session List Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Session List Demo.mkv".to_owned(),
        file_name: "Session List Demo.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let other_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Other Session List Demo.mkv".to_owned(),
        file_name: "Other Session List Demo.mkv".to_owned(),
        size_bytes: Some(24),
        fingerprint: None,
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store.upsert_media_source(&other_source).await.unwrap();

    let remux_id = TranscodeSessionId::new();
    let hls_id = TranscodeSessionId::new();
    let other_id = TranscodeSessionId::new();

    store
        .create_transcode_session(NewTranscodeSession {
            id: remux_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "test-transcode-profile:remux-primary".to_owned(),
            output_path: "cache/remux/stream.mp4".into(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .create_transcode_session(NewTranscodeSession {
            id: hls_id,
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: "test-transcode-profile:hls-primary".to_owned(),
            output_path: "cache/hls/playlist.m3u8".into(),
            state: TranscodeSessionState::Planned,
        })
        .await
        .unwrap();
    store
        .create_transcode_session(NewTranscodeSession {
            id: other_id,
            source_id: other_source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "test-transcode-profile:remux-other".to_owned(),
            output_path: "cache/remux/other.mkv".into(),
            state: TranscodeSessionState::Planned,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            hls_id,
            TranscodeSessionState::Failed,
            Some(TranscodeFailureCategory::Runner),
            Some("ffmpeg failed".to_owned()),
        )
        .await
        .unwrap();

    let filtered = store
        .list_transcode_sessions(
            TranscodeSessionListFilter {
                source_id: Some(source.id),
                kind: Some(TranscodeSessionKind::HlsTranscode),
                state: Some(TranscodeSessionState::Failed),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, hls_id);
    assert_eq!(
        filtered[0].failure_category,
        Some(TranscodeFailureCategory::Runner)
    );

    let source_sessions = store
        .list_transcode_sessions(
            TranscodeSessionListFilter {
                source_id: Some(source.id),
                ..TranscodeSessionListFilter::default()
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(source_sessions.len(), 2);

    let second_page = store
        .list_transcode_sessions(
            TranscodeSessionListFilter::default(),
            PageRequest::new(1, 1),
        )
        .await
        .unwrap();
    assert_eq!(second_page.len(), 1);
}

#[tokio::test]
async fn taru_database_sqlite_marks_stale_transcode_sessions_failed() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Stale Session Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Stale Session Demo.mkv".to_owned(),
        file_name: "Stale Session Demo.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let stale_id = TranscodeSessionId::new();
    let finished_id = TranscodeSessionId::new();

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .create_transcode_session(NewTranscodeSession {
            id: stale_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "test-transcode-profile:remux-stale".to_owned(),
            output_path: "cache/remux/stale.mp4".into(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();
    store
        .create_transcode_session(NewTranscodeSession {
            id: finished_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "test-transcode-profile:remux-finished".to_owned(),
            output_path: "cache/remux/finished.mkv".into(),
            state: TranscodeSessionState::Planned,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(finished_id, TranscodeSessionState::Finished, None, None)
        .await
        .unwrap();

    let recovered = store
        .fail_stale_transcode_sessions(
            TranscodeFailureCategory::Stale,
            "session was active during server startup".to_owned(),
        )
        .await
        .unwrap();

    let stale = store
        .get_transcode_session(stale_id)
        .await
        .unwrap()
        .unwrap();
    let finished = store
        .get_transcode_session(finished_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(recovered, 1);
    assert_eq!(stale.state, TranscodeSessionState::Failed);
    assert_eq!(
        stale.failure_category,
        Some(TranscodeFailureCategory::Stale)
    );
    assert_eq!(finished.state, TranscodeSessionState::Finished);
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_metadata_policy_records() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Policy Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let lock = MetadataFieldLock {
        item_id: item.id,
        field: MetadataField::Title,
        locked: true,
        source: MetadataSource::Addon(AddonId::new()),
    };
    let raw = ProviderRawResponse {
        item_id: item.id,
        provider: ExternalProvider::Tmdb,
        provider_key: "603".to_owned(),
        fetched_at: "2026-05-14T00:00:00.000Z".to_owned(),
        body_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
    };
    let raw_douban = ProviderRawResponse {
        item_id: item.id,
        provider: ExternalProvider::Douban,
        provider_key: "1291843".to_owned(),
        fetched_at: "2026-05-16T00:00:00.000Z".to_owned(),
        body_json: r#"{"id":"1291843","title":"The Matrix"}"#.to_owned(),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_field_lock(&lock).await.unwrap();
    store.upsert_provider_raw_response(&raw).await.unwrap();
    store
        .upsert_provider_raw_response(&raw_douban)
        .await
        .unwrap();

    assert_eq!(store.list_field_locks(item.id).await.unwrap(), vec![lock]);
    assert_eq!(
        store
            .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "603")
            .await
            .unwrap(),
        Some(raw.clone())
    );
    assert_eq!(
        store
            .list_provider_raw_responses(
                item.id,
                ProviderRawResponseFilter::default(),
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![
            ProviderRawResponse {
                item_id: item.id,
                provider: ExternalProvider::Douban,
                provider_key: "1291843".to_owned(),
                fetched_at: "2026-05-16T00:00:00.000Z".to_owned(),
                body_json: r#"{"id":"1291843","title":"The Matrix"}"#.to_owned(),
            },
            ProviderRawResponse {
                item_id: item.id,
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                fetched_at: "2026-05-14T00:00:00.000Z".to_owned(),
                body_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
            }
        ]
    );
    assert_eq!(
        store
            .list_provider_raw_responses(
                item.id,
                ProviderRawResponseFilter {
                    provider: Some(ExternalProvider::Tmdb),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![raw.clone()]
    );

    let cleanup = store
        .cleanup_provider_raw_responses(
            ProviderRawResponseFilter {
                provider: Some(ExternalProvider::Tmdb),
            },
            "2026-05-15T00:00:00.000Z",
        )
        .await
        .unwrap();
    assert_eq!(cleanup.deleted, 1);
    assert_eq!(
        store
            .list_provider_raw_responses(
                item.id,
                ProviderRawResponseFilter::default(),
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![raw_douban]
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_metadata_provider_attempts() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Attempt Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.tmdb".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    let attempt = NewMetadataProviderAttempt {
        id: taru_core::MetadataProviderAttemptId::new(),
        job_id: job.id,
        item_id: item.id,
        provider: ExternalProvider::Tmdb,
        status: MetadataProviderAttemptStatus::Succeeded,
        provider_key: Some("603".to_owned()),
        matched_by: Some(MetadataMatchKind::Search),
        started_at: "2026-05-14T00:00:00Z".to_owned(),
        finished_at: "2026-05-14T00:00:01Z".to_owned(),
        error_class: None,
        message: None,
    };
    let failed_attempt = NewMetadataProviderAttempt {
        id: taru_core::MetadataProviderAttemptId::new(),
        job_id: job.id,
        item_id: item.id,
        provider: ExternalProvider::Douban,
        status: MetadataProviderAttemptStatus::NoMatch,
        provider_key: None,
        matched_by: None,
        started_at: "2026-05-14T00:00:02Z".to_owned(),
        finished_at: "2026-05-14T00:00:03Z".to_owned(),
        error_class: Some(MetadataProviderErrorClass::NoMatch),
        message: Some("no match".to_owned()),
    };

    store
        .insert_metadata_provider_attempt(attempt.clone())
        .await
        .unwrap();
    store
        .insert_metadata_provider_attempt(failed_attempt.clone())
        .await
        .unwrap();

    let expected = MetadataProviderAttemptRecord {
        id: attempt.id,
        job_id: attempt.job_id,
        item_id: attempt.item_id,
        provider: attempt.provider,
        status: attempt.status,
        provider_key: attempt.provider_key,
        matched_by: attempt.matched_by,
        started_at: attempt.started_at,
        finished_at: attempt.finished_at,
        error_class: attempt.error_class,
        message: attempt.message,
    };
    let failed_expected = MetadataProviderAttemptRecord {
        id: failed_attempt.id,
        job_id: failed_attempt.job_id,
        item_id: failed_attempt.item_id,
        provider: failed_attempt.provider,
        status: failed_attempt.status,
        provider_key: failed_attempt.provider_key,
        matched_by: failed_attempt.matched_by,
        started_at: failed_attempt.started_at,
        finished_at: failed_attempt.finished_at,
        error_class: failed_attempt.error_class,
        message: failed_attempt.message,
    };

    assert_eq!(
        store.list_metadata_provider_attempts(job.id).await.unwrap(),
        vec![expected.clone(), failed_expected.clone()]
    );
    assert_eq!(
        store
            .list_metadata_provider_attempts_for_item(
                item.id,
                MetadataAttemptFilter::default(),
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![failed_expected.clone(), expected.clone()]
    );
    assert_eq!(
        store
            .list_metadata_provider_attempts_for_item(
                item.id,
                MetadataAttemptFilter {
                    provider: Some(ExternalProvider::Douban),
                    status: Some(MetadataProviderAttemptStatus::NoMatch),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![failed_expected]
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_provider_subjects_and_mappings() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Anime".to_owned(),
        roots: vec!["local:///Anime".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Anime),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Local Title".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let tmdb_movie = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "603".to_owned(),
        title: Some("The Matrix".to_owned()),
        release_year: Some(1999),
        locale: Some("en-US".to_owned()),
    };
    let bangumi_subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Subject,
        subject_key: "265".to_owned(),
        title: Some("Cowboy Bebop".to_owned()),
        release_year: Some(1998),
        locale: Some("zh-CN".to_owned()),
    };
    let tmdb_mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: item.id,
        subject_id: tmdb_movie.id,
        status: ProviderMappingStatus::Accepted,
        confidence_milli: Some(940),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    let bangumi_mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: item.id,
        subject_id: bangumi_subject.id,
        status: ProviderMappingStatus::Candidate,
        confidence_milli: Some(720),
        source: MetadataSource::Provider(ExternalProvider::Bangumi),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_provider_subject(&tmdb_movie).await.unwrap();
    store
        .upsert_provider_subject(&bangumi_subject)
        .await
        .unwrap();
    store.upsert_provider_mapping(&tmdb_mapping).await.unwrap();
    store
        .upsert_provider_mapping(&bangumi_mapping)
        .await
        .unwrap();

    assert_eq!(
        store
            .find_provider_subject(&ExternalProvider::Tmdb, &ProviderSubjectKind::Movie, "603",)
            .await
            .unwrap(),
        Some(tmdb_movie.clone())
    );
    assert_eq!(
        store
            .list_provider_subjects_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![bangumi_subject, tmdb_movie]
    );
    assert_eq!(
        store
            .list_provider_mappings_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![bangumi_mapping, tmdb_mapping]
    );
    assert_eq!(store.get_media_item(item.id).await.unwrap(), Some(item));
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_source_duplicate_relationships_without_merging_items() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let first_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Movie A".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let second_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Movie A Remux".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let first_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: first_item.id,
        locator: "local:///Movies/Movie A.mkv".to_owned(),
        file_name: "Movie A.mkv".to_owned(),
        size_bytes: Some(100),
        fingerprint: Some("sha256:movie-a".to_owned()),
    };
    let second_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: second_item.id,
        locator: "local:///Movies/Movie A Remux.mkv".to_owned(),
        file_name: "Movie A Remux.mkv".to_owned(),
        size_bytes: Some(100),
        fingerprint: Some("sha256:movie-a".to_owned()),
    };
    let relationship = SourceDuplicateRelationship {
        id: SourceDuplicateRelationshipId::new(),
        source_id: second_source.id,
        duplicate_source_id: first_source.id,
        evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
        evidence_value: Some("sha256:movie-a".to_owned()),
        status: SourceDuplicateRelationshipStatus::Suggested,
        confidence_milli: Some(990),
    };
    let expected_relationship = relationship.canonicalized();

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&first_item).await.unwrap();
    store.upsert_media_item(&second_item).await.unwrap();
    store.upsert_media_source(&first_source).await.unwrap();
    store.upsert_media_source(&second_source).await.unwrap();
    store
        .upsert_source_duplicate_relationship(&relationship)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_source_duplicate_relationship(relationship.id)
            .await
            .unwrap(),
        Some(expected_relationship.clone())
    );
    assert_eq!(
        store
            .list_source_duplicate_relationships(first_source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected_relationship.clone()]
    );
    assert_eq!(
        store
            .list_source_duplicate_relationships(second_source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected_relationship]
    );
    assert_eq!(
        store
            .get_media_source(first_source.id)
            .await
            .unwrap()
            .unwrap()
            .item_id,
        first_item.id
    );
    assert_eq!(
        store
            .get_media_source(second_source.id)
            .await
            .unwrap()
            .unwrap()
            .item_id,
        second_item.id
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_local_inference_evidence_without_confirming_metadata() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Mixed Video".to_owned(),
        roots: vec!["local:///Videos".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::MixedVideo),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Unknown,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Unmatched Local File".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Videos/Show.Name.S01E02.mkv".to_owned(),
        file_name: "Show.Name.S01E02.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let evidence = LocalInferenceEvidence {
        id: LocalInferenceEvidenceId::new(),
        source_id: source.id,
        inferred_kind: MediaKind::Episode,
        inferred_title: Some("Show Name".to_owned()),
        inferred_year: None,
        inferred_season: Some(1),
        inferred_episode: Some(2),
        confidence_milli: Some(650),
        evidence_source: LocalInferenceEvidenceSource::FileName,
        evidence_value: "Show.Name.S01E02.mkv".to_owned(),
        inference_version: "local-path-parser:v1".to_owned(),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_local_inference_evidence(&evidence)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_local_inference_evidence(evidence.id)
            .await
            .unwrap(),
        Some(evidence.clone())
    );
    assert_eq!(
        store
            .list_local_inference_evidence_for_source(source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![evidence.clone()]
    );

    let mut updated_evidence = evidence.clone();
    updated_evidence.id = LocalInferenceEvidenceId::new();
    updated_evidence.inferred_title = Some("Show Name Revised".to_owned());
    updated_evidence.confidence_milli = Some(900);
    updated_evidence.evidence_value = "Show.Name.S01E02.1080p.mkv".to_owned();
    store
        .upsert_local_inference_evidence(&updated_evidence)
        .await
        .unwrap();

    let mut expected_snapshot = updated_evidence;
    expected_snapshot.id = evidence.id;
    assert_eq!(
        store
            .list_local_inference_evidence_for_source(source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected_snapshot.clone()]
    );
    assert_eq!(
        store
            .get_local_inference_evidence(evidence.id)
            .await
            .unwrap(),
        Some(expected_snapshot)
    );
    assert_eq!(store.get_media_item(item.id).await.unwrap(), Some(item));
}

#[tokio::test]
async fn taru_database_sqlite_lists_catalog_governance_items_for_unknown_and_low_confidence() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let unknown = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Unknown,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Unmatched Local File".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let low_confidence = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Weak Match".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let high_confidence = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Confident Match".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let unknown_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: unknown.id,
        locator: "local:///Movies/Private/Unknown.Local.File.mkv".to_owned(),
        file_name: "Unknown.Local.File.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    let low_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: low_confidence.id,
        locator: "local:///Movies/Weak.Match.mkv".to_owned(),
        file_name: "Weak.Match.mkv".to_owned(),
        size_bytes: Some(84),
        fingerprint: Some("sha256:weak".to_owned()),
    };
    let high_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: high_confidence.id,
        locator: "local:///Movies/Confident.Match.mkv".to_owned(),
        file_name: "Confident.Match.mkv".to_owned(),
        size_bytes: Some(168),
        fingerprint: Some("sha256:confident".to_owned()),
    };
    let low_subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "100".to_owned(),
        title: Some("Weak Match".to_owned()),
        release_year: None,
        locale: Some("en-US".to_owned()),
    };
    let low_mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: low_confidence.id,
        subject_id: low_subject.id,
        status: ProviderMappingStatus::Accepted,
        confidence_milli: Some(900),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    let duplicate = SourceDuplicateRelationship {
        id: SourceDuplicateRelationshipId::new(),
        source_id: low_source.id,
        duplicate_source_id: unknown_source.id,
        evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
        evidence_value: Some("sha256:weak".to_owned()),
        status: SourceDuplicateRelationshipStatus::Suggested,
        confidence_milli: Some(880),
    };

    store.upsert_library(&library).await.unwrap();
    for item in [&unknown, &low_confidence, &high_confidence] {
        store.upsert_media_item(item).await.unwrap();
    }
    for source in [&unknown_source, &low_source, &high_source] {
        store.upsert_media_source(source).await.unwrap();
    }
    store.upsert_provider_subject(&low_subject).await.unwrap();
    store.upsert_provider_mapping(&low_mapping).await.unwrap();
    store
        .upsert_source_duplicate_relationship(&duplicate)
        .await
        .unwrap();
    for (source, confidence) in [
        (&unknown_source, 350),
        (&low_source, 640),
        (&high_source, 920),
    ] {
        store
            .upsert_local_inference_evidence(&LocalInferenceEvidence {
                id: LocalInferenceEvidenceId::new(),
                source_id: source.id,
                inferred_kind: if source.id == unknown_source.id {
                    MediaKind::Unknown
                } else {
                    MediaKind::Movie
                },
                inferred_title: Some(source.file_name.trim_end_matches(".mkv").replace('.', " ")),
                inferred_year: None,
                inferred_season: None,
                inferred_episode: None,
                confidence_milli: Some(confidence),
                evidence_source: LocalInferenceEvidenceSource::Path,
                evidence_value: source.locator.clone(),
                inference_version: "taru-naming:1".to_owned(),
            })
            .await
            .unwrap();
    }

    let records = store
        .list_catalog_governance_items(
            CatalogGovernanceItemListFilter {
                library_id: Some(library.id),
                max_confidence_milli: 700,
            },
            PageRequest::new(10, 0),
        )
        .await
        .unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].item.id, unknown.id);
    assert_eq!(records[0].item.kind, MediaKind::Unknown);
    assert_eq!(records[0].source_count, 1);
    assert_eq!(records[0].representative_source_id, Some(unknown_source.id));
    assert_eq!(
        records[0]
            .best_local_inference
            .as_ref()
            .unwrap()
            .confidence_milli,
        Some(350)
    );
    assert_eq!(records[0].provider_mapping_count, 0);
    assert_eq!(records[0].duplicate_relationship_count, 1);

    assert_eq!(records[1].item.id, low_confidence.id);
    assert_eq!(records[1].item.kind, MediaKind::Movie);
    assert_eq!(
        records[1]
            .best_local_inference
            .as_ref()
            .unwrap()
            .confidence_milli,
        Some(640)
    );
    assert_eq!(records[1].provider_mapping_count, 1);
    assert_eq!(records[1].accepted_provider_mapping_count, 1);
    assert_eq!(records[1].duplicate_relationship_count, 1);
    assert!(
        !records
            .iter()
            .any(|record| record.item.id == high_confidence.id)
    );
}

#[tokio::test]
async fn taru_database_sqlite_rolls_back_catalog_graph_when_search_projection_commit_fails() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let item_id = MediaItemId::new();
    let person = Person {
        id: PersonId::new(),
        name: "Rollback Actor".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let commit = CatalogItemProjectionCommit {
        graph: CatalogItemGraphReplacement {
            people: vec![person.clone()],
            credits: vec![ItemCredit {
                item_id,
                person_id: person.id,
                role: CreditRole::Actor,
                character: Some("Failure Path".to_owned()),
                sort_order: Some(1),
            }],
            ..CatalogItemGraphReplacement::default()
        },
        search: CatalogSearchProjection::try_from_facet_labels(
            item_id,
            "Missing Item",
            "should not be committed",
            vec!["genre:rollback".to_owned()],
        )
        .unwrap(),
    };

    let err = store.commit_item_projection(&commit).await.unwrap_err();
    let people = store.list_people(PageRequest::first_page()).await.unwrap();
    let credits = store.list_item_credits(item_id).await.unwrap();
    let hits = store
        .search(SearchQuery::from_facet_labels("missing", Vec::new(), 10, 0).unwrap())
        .await
        .unwrap();

    assert!(matches!(err, TaruError::Database { .. }));
    assert!(people.is_empty());
    assert!(credits.is_empty());
    assert!(hits.is_empty());
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_media_probe_results() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Probe Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Probe Demo.mkv".to_owned(),
        file_name: "Probe Demo.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: None,
    };
    let result = MediaProbeResult {
        duration_ms: Some(120_253),
        container: Some("matroska,webm".to_owned()),
        bit_rate: Some(4_200_000),
        streams: vec![
            MediaStreamInfo {
                index: 0,
                kind: MediaStreamKind::Video,
                codec: Some("h264".to_owned()),
                language: Some("und".to_owned()),
                duration_ms: Some(120_250),
                bit_rate: Some(4_000_000),
                width: Some(1920),
                height: Some(1080),
                channels: None,
                sample_rate: None,
            },
            MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: Some("eng".to_owned()),
                duration_ms: Some(120_240),
                bit_rate: Some(128_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
            },
            MediaStreamInfo {
                index: 2,
                kind: MediaStreamKind::Other("timed_id3".to_owned()),
                codec: None,
                language: None,
                duration_ms: None,
                bit_rate: None,
                width: None,
                height: None,
                channels: None,
                sample_rate: None,
            },
        ],
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store.upsert_media_probe(source.id, &result).await.unwrap();

    assert_eq!(
        store.get_media_probe(source.id).await.unwrap(),
        Some(result)
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_vfs_cache_records_and_failures() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let directory = VfsCachedObject {
        uri: "webdav:///Movies/".to_owned(),
        scheme: "webdav".to_owned(),
        kind: VfsCachedObjectKind::Directory,
        len: None,
        modified_at: Some("2026-05-15T00:00:00.000Z".to_owned()),
        etag: Some("movies".to_owned()),
        fingerprint: Some("webdav:etag=movies".to_owned()),
        capabilities_bits: 0b111,
        fetched_at_ms: 100,
        fresh_until_ms: 200,
    };
    let movie = VfsCachedObject {
        uri: "webdav:///Movies/Demo.mkv".to_owned(),
        scheme: "webdav".to_owned(),
        kind: VfsCachedObjectKind::File,
        len: Some(4),
        modified_at: Some("2026-05-15T00:00:01.000Z".to_owned()),
        etag: Some("demo".to_owned()),
        fingerprint: Some("webdav:etag=demo".to_owned()),
        capabilities_bits: 0b101,
        fetched_at_ms: 100,
        fresh_until_ms: 200,
    };
    let listing = VfsCachedListing {
        directory: directory.clone(),
        entries: vec![movie.clone()],
        fetched_at_ms: 100,
        fresh_until_ms: 200,
    };

    store.upsert_vfs_cache_listing(&listing).await.unwrap();
    let loaded_object = store
        .get_vfs_cache_object("webdav:///Movies/Demo.mkv")
        .await
        .unwrap();
    let loaded_listing = store
        .get_vfs_cache_listing("webdav:///Movies/")
        .await
        .unwrap();

    assert_eq!(loaded_object, Some(movie));
    assert_eq!(loaded_listing, Some(listing));

    let first_failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Movies/".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 300,
            error: "timeout".to_owned(),
        })
        .await
        .unwrap();
    let second_failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Movies/".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 400,
            error: "rate limited".to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(first_failure.failure_count, 1);
    assert_eq!(second_failure.failure_count, 2);
    assert_eq!(second_failure.failed_at_ms, 400);
    assert_eq!(second_failure.error, "rate limited");

    let summary = store.summarize_vfs_cache(300).await.unwrap();
    assert_eq!(summary.object_count, 2);
    assert_eq!(summary.listing_count, 1);
    assert_eq!(summary.failure_count, 1);
    assert_eq!(summary.stale_object_count, 2);
    assert_eq!(summary.stale_listing_count, 1);
    assert_eq!(summary.last_failure_at_ms, Some(400));
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_catalog_graph_records() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Graph Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let person = Person {
        id: PersonId::new(),
        name: "Keanu Reeves".to_owned(),
        sort_name: Some("Reeves, Keanu".to_owned()),
        overview: Some("Actor".to_owned()),
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: "6384".to_owned(),
        }],
    };
    let credit = ItemCredit {
        item_id: item.id,
        person_id: person.id,
        role: CreditRole::Actor,
        character: Some("Neo".to_owned()),
        sort_order: Some(0),
    };
    let genre = Genre {
        id: GenreId::new(),
        name: "Science Fiction".to_owned(),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    let tag = Tag {
        id: TagId::new(),
        name: "Watchlist".to_owned(),
        source: MetadataSource::User,
    };
    let collection = Collection {
        id: CollectionId::new(),
        name: "Matrix Collection".to_owned(),
        overview: Some("Franchise".to_owned()),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: "2344".to_owned(),
        }],
    };
    let studio = Studio {
        id: StudioId::new(),
        name: "Warner Bros.".to_owned(),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: "174".to_owned(),
        }],
    };
    let image = ImageAsset {
        id: ImageAssetId::new(),
        owner: ImageOwner::Item(item.id),
        kind: ImageKind::Poster,
        source_uri: "https://image.example/poster.jpg".to_owned(),
        provider: ExternalProvider::Tmdb,
        cache_uri: Some("local:///cache/poster.webp".to_owned()),
        width: Some(1000),
        height: Some(1500),
        language: Some("en".to_owned()),
        selected: true,
        content_hash: Some("hash".to_owned()),
        etag: Some("etag".to_owned()),
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_person(&person).await.unwrap();
    store.upsert_item_credit(&credit).await.unwrap();
    store.upsert_genre(&genre).await.unwrap();
    store
        .upsert_item_genre(&ItemGenre {
            item_id: item.id,
            genre_id: genre.id,
        })
        .await
        .unwrap();
    store.upsert_tag(&tag).await.unwrap();
    store
        .upsert_item_tag(&ItemTag {
            item_id: item.id,
            tag_id: tag.id,
        })
        .await
        .unwrap();
    store.upsert_collection(&collection).await.unwrap();
    store
        .upsert_collection_item(&CollectionItem {
            collection_id: collection.id,
            item_id: item.id,
            sort_order: Some(1),
        })
        .await
        .unwrap();
    store.upsert_studio(&studio).await.unwrap();
    store
        .upsert_item_studio(&ItemStudio {
            item_id: item.id,
            studio_id: studio.id,
        })
        .await
        .unwrap();
    store.upsert_image_asset(&image).await.unwrap();

    assert_eq!(store.get_person(person.id).await.unwrap(), Some(person));
    assert_eq!(
        store.list_item_credits(item.id).await.unwrap(),
        vec![credit]
    );
    assert_eq!(store.get_genre(genre.id).await.unwrap(), Some(genre));
    assert_eq!(store.list_item_genres(item.id).await.unwrap().len(), 1);
    assert_eq!(store.get_tag(tag.id).await.unwrap(), Some(tag));
    assert_eq!(store.list_item_tags(item.id).await.unwrap().len(), 1);
    assert_eq!(
        store.get_collection(collection.id).await.unwrap(),
        Some(collection.clone())
    );
    assert_eq!(
        store.list_collection_items(collection.id).await.unwrap(),
        vec![CollectionItem {
            collection_id: collection.id,
            item_id: item.id,
            sort_order: Some(1)
        }]
    );
    assert_eq!(store.get_studio(studio.id).await.unwrap(), Some(studio));
    assert_eq!(store.list_item_studios(item.id).await.unwrap().len(), 1);
    assert_eq!(store.get_image_asset(image.id).await.unwrap(), Some(image));
    assert_eq!(store.list_item_images(item.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_scan_state_search_and_artwork_tasks() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Searchable Demo".to_owned(),
            overview: Some("A searchable graph fixture.".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Searchable Demo.mkv".to_owned(),
        file_name: "Searchable Demo.mkv".to_owned(),
        size_bytes: Some(10),
        fingerprint: Some("fingerprint".to_owned()),
    };
    let scan_id = ScanSnapshotId::new();
    let image = ImageAsset {
        id: ImageAssetId::new(),
        owner: ImageOwner::Item(item.id),
        kind: ImageKind::Thumbnail,
        source_uri: "local:///Movies/Searchable Demo.mkv#preview=10".to_owned(),
        provider: ExternalProvider::Local,
        cache_uri: None,
        width: Some(320),
        height: Some(180),
        language: None,
        selected: false,
        content_hash: None,
        etag: None,
    };
    let task = ArtworkTask {
        id: ArtworkTaskId::new(),
        image_id: image.id,
        kind: ArtworkTaskKind::Preview,
        status: JobStatus::Queued,
        resource_class: ArtworkTaskKind::Preview.resource_class().to_owned(),
        attempts: 0,
        max_attempts: 3,
        error: None,
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let running = store
        .begin_scan_snapshot(scan_id, library.id, "local:///Movies")
        .await
        .unwrap();
    store
        .upsert_directory_snapshot(&DirectorySnapshot {
            scan_id,
            uri: "local:///Movies".to_owned(),
            etag: Some("dir-etag".to_owned()),
            modified_at: Some("1".to_owned()),
            child_count: 1,
        })
        .await
        .unwrap();
    store
        .upsert_source_state(&SourceState {
            library_id: library.id,
            source_id: Some(source.id),
            uri: source.locator.clone(),
            size_bytes: source.size_bytes,
            modified_at: Some("1".to_owned()),
            etag: None,
            fingerprint: source.fingerprint.clone(),
            last_seen_scan_id: scan_id,
            tombstoned: false,
        })
        .await
        .unwrap();
    let completed = store
        .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
        .await
        .unwrap();
    let failed_scan_id = ScanSnapshotId::new();
    store
        .begin_scan_snapshot(failed_scan_id, library.id, "local:///Broken")
        .await
        .unwrap();
    let failed = store
        .complete_scan_snapshot(
            failed_scan_id,
            ScanStatus::Failed,
            Some("scan failed".to_owned()),
        )
        .await
        .unwrap();
    store
        .upsert(
            SearchDocument::from_facet_labels(
                item.id,
                item.metadata.title.clone(),
                item.metadata.overview.clone().unwrap(),
                vec!["genre:sci-fi".to_owned()],
            )
            .unwrap(),
        )
        .await
        .unwrap();
    store.upsert_image_asset(&image).await.unwrap();
    store.enqueue_artwork_task(&task).await.unwrap();

    assert_eq!(running.status, ScanStatus::Running);
    assert_eq!(completed.status, ScanStatus::Succeeded);
    assert!(completed.completed_at.is_some());
    assert_eq!(failed.status, ScanStatus::Failed);
    assert_eq!(failed.error, Some("scan failed".to_owned()));
    assert_eq!(
        store.list_directory_snapshots(scan_id).await.unwrap().len(),
        1
    );
    assert_eq!(
        store
            .get_source_state(library.id, &source.locator)
            .await
            .unwrap()
            .unwrap()
            .fingerprint,
        Some("fingerprint".to_owned())
    );
    assert_eq!(
        store
            .search(SearchQuery::from_facet_labels("searchable", Vec::new(), 10, 0,).unwrap())
            .await
            .unwrap()[0]
            .item_id,
        item.id
    );
    assert_eq!(store.get_artwork_task(task.id).await.unwrap(), Some(task));
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_job_lifecycle() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();

    let id = JobId::new();
    let queued = store
        .enqueue_job(NewJob {
            id,
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: Some(r#"{"library_id":"demo"}"#.to_owned()),
        })
        .await
        .unwrap();
    let running = store.start_job(id).await.unwrap();
    let succeeded = store
        .succeed_job(id, Some(r#"{"discovered_files":1}"#.to_owned()))
        .await
        .unwrap();

    assert_eq!(queued.status, JobStatus::Queued);
    assert_eq!(
        queued.input_json,
        Some(r#"{"library_id":"demo"}"#.to_owned())
    );
    assert_eq!(running.status, JobStatus::Running);
    assert!(running.started_at.is_some());
    assert_eq!(succeeded.status, JobStatus::Succeeded);
    assert_eq!(
        succeeded.summary_json,
        Some(r#"{"discovered_files":1}"#.to_owned())
    );
    assert!(succeeded.completed_at.is_some());
    assert_eq!(store.get_job(id).await.unwrap(), Some(succeeded));

    let failed_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: failed_id,
            kind: JobKind::LibraryProbe,
            resource_class: "media.probe".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(failed_id).await.unwrap();
    let failed = store
        .fail_job(failed_id, "probe failed".to_owned())
        .await
        .unwrap();

    assert_eq!(failed.status, JobStatus::Failed);
    assert_eq!(failed.error, Some("probe failed".to_owned()));
    assert!(failed.completed_at.is_some());
}

#[tokio::test]
async fn taru_database_sqlite_lists_jobs_with_filters_and_pagination() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let other_library = Library {
        id: LibraryId::new(),
        name: "Anime".to_owned(),
        roots: vec!["local:///Anime".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Anime),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_library(&other_library).await.unwrap();

    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: item.id,
        locator: "local:///Movies/Demo.mkv".to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: Some("fingerprint".to_owned()),
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let scan = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            library_id: Some(library.id),
            source_id: Some(source.id),
            input_json: Some(r#"{"library_id":"movies"}"#.to_owned()),
        })
        .await
        .unwrap();
    store.start_job(scan.id).await.unwrap();
    store
        .succeed_job(scan.id, Some(r#"{"discovered_files":1}"#.to_owned()))
        .await
        .unwrap();

    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.tmdb".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();

    let failed_scan = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            library_id: Some(other_library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(failed_scan.id).await.unwrap();
    store
        .fail_job(failed_scan.id, "scan failed".to_owned())
        .await
        .unwrap();

    let filtered = store
        .list_jobs(
            JobListFilter {
                status: Some(JobStatus::Succeeded),
                kind: Some(JobKind::LibraryScan),
                resource_class: Some("disk.scan".to_owned()),
                library_id: Some(library.id),
                source_id: Some(source.id),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, scan.id);
    assert_eq!(
        filtered[0].summary_json.as_deref(),
        Some(r#"{"discovered_files":1}"#)
    );

    let disk_scan_jobs = store
        .list_jobs(
            JobListFilter {
                resource_class: Some("disk.scan".to_owned()),
                ..JobListFilter::default()
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(disk_scan_jobs.len(), 2);

    let second_page = store
        .list_jobs(JobListFilter::default(), PageRequest::new(1, 1))
        .await
        .unwrap();
    assert_eq!(second_page.len(), 1);
}

#[tokio::test]
async fn taru_database_sqlite_marks_running_jobs_failed_on_startup_and_preserves_queued_jobs() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();

    let queued_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: queued_id,
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.refresh".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();

    let running_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: running_id,
            kind: JobKind::LibraryScan,
            resource_class: "library.scan".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(running_id).await.unwrap();

    let leased_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: leased_id,
            kind: JobKind::NfoExport,
            resource_class: "nfo.export".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    let leased = store
        .claim_next_job_lease(JobLeaseClaimRequest {
            worker_id: JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: JobLeaseClaimFilter {
                kind: Some(JobKind::NfoExport),
                ..JobLeaseClaimFilter::default()
            },
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(leased.job.id, leased_id);

    let succeeded_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: succeeded_id,
            kind: JobKind::NfoImport,
            resource_class: "nfo.import".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(succeeded_id).await.unwrap();
    store
        .succeed_job(succeeded_id, Some(r#"{"imported":1}"#.to_owned()))
        .await
        .unwrap();

    let failed_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: failed_id,
            kind: JobKind::LibraryProbe,
            resource_class: "library.probe".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(failed_id).await.unwrap();
    store
        .fail_job(failed_id, "probe failed".to_owned())
        .await
        .unwrap();

    let managed_artwork_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: managed_artwork_id,
            kind: JobKind::ManagedArtworkIngest,
            resource_class: "artwork.ingest".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(managed_artwork_id).await.unwrap();

    let error = "job was unfinished during server startup".to_owned();
    let recovered = store.fail_unfinished_jobs(error.clone()).await.unwrap();

    let queued = store.get_job(queued_id).await.unwrap().unwrap();
    let running = store.get_job(running_id).await.unwrap().unwrap();
    let leased = store.get_job(leased_id).await.unwrap().unwrap();
    let succeeded = store.get_job(succeeded_id).await.unwrap().unwrap();
    let failed = store.get_job(failed_id).await.unwrap().unwrap();
    let managed_artwork = store.get_job(managed_artwork_id).await.unwrap().unwrap();

    assert_eq!(recovered, 2);
    assert_eq!(queued.status, JobStatus::Queued);
    assert_eq!(queued.error, None);
    assert_eq!(queued.completed_at, None);
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(running.error, Some(error));
    assert!(running.completed_at.is_some());
    assert_eq!(leased.status, JobStatus::Failed);
    assert_eq!(succeeded.status, JobStatus::Succeeded);
    assert_eq!(succeeded.summary_json, Some(r#"{"imported":1}"#.to_owned()));
    assert_eq!(failed.status, JobStatus::Failed);
    assert_eq!(failed.error, Some("probe failed".to_owned()));
    assert_eq!(managed_artwork.status, JobStatus::Running);
    assert_eq!(managed_artwork.error, None);
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_outbox_events_idempotently() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();

    let event = NewOutboxEvent {
        id: EventId::new(),
        kind: DomainEventKind::LibraryScanned,
        subject: DomainEventSubject::Library(library.id),
        library_id: Some(library.id),
        source_id: None,
        idempotency_key: format!("library_scan:{}", library.id),
        payload_json: format!(r#"{{"library_id":"{}","indexed_items":1}}"#, library.id),
    };

    let first = store.enqueue_outbox_event(event.clone()).await.unwrap();
    let duplicate = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            ..event.clone()
        })
        .await
        .unwrap();

    assert_eq!(first, duplicate);
    assert_eq!(first.kind, DomainEventKind::LibraryScanned);
    assert_eq!(first.subject, DomainEventSubject::Library(library.id));
    assert_eq!(first.status, OutboxEventStatus::Pending);
    assert_eq!(first.attempts, 0);
    assert!(first.occurred_at.ends_with('Z'));
    assert_eq!(store.get_outbox_event(first.id).await.unwrap(), Some(first));

    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(!events[0].payload_json.contains("TMDB_READ_ACCESS_TOKEN"));
    assert!(!events[0].payload_json.contains("F:/"));
}

#[tokio::test]
async fn taru_database_sqlite_lists_outbox_events_with_filters_and_pagination() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let source_id = MediaSourceId::new();
    let other_source_id = MediaSourceId::new();
    let item_id = MediaItemId::new();
    let other_item_id = MediaItemId::new();

    for (id, name) in [(library_id, "Movies"), (other_library_id, "Anime")] {
        store
            .upsert_library(&Library {
                id,
                name: name.to_owned(),
                roots: vec!["local:///".to_owned()],
                options: LibraryOptions::from_preset(LibraryPreset::Movies),
            })
            .await
            .unwrap();
    }
    for (id, title) in [(item_id, "Demo"), (other_item_id, "Other")] {
        store
            .upsert_media_item(&MediaItem {
                id,
                kind: MediaKind::Movie,
                parent_id: None,
                metadata: CanonicalMetadata {
                    title: title.to_owned(),
                    ..CanonicalMetadata::default()
                },
            })
            .await
            .unwrap();
    }
    store
        .upsert_media_source(&MediaSource {
            id: source_id,
            library_id,
            item_id,
            locator: "local:///Movies/Demo.mkv".to_owned(),
            file_name: "Demo.mkv".to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
        })
        .await
        .unwrap();
    store
        .upsert_media_source(&MediaSource {
            id: other_source_id,
            library_id: other_library_id,
            item_id: other_item_id,
            locator: "local:///Anime/Other.mkv".to_owned(),
            file_name: "Other.mkv".to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
        })
        .await
        .unwrap();

    let scan = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("library_scan:{library_id}"),
            payload_json: "{}".to_owned(),
        })
        .await
        .unwrap();
    let metadata = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::ItemMetadataRefreshed,
            subject: DomainEventSubject::Source(source_id),
            library_id: Some(library_id),
            source_id: Some(source_id),
            idempotency_key: format!("metadata:{source_id}"),
            payload_json: "{}".to_owned(),
        })
        .await
        .unwrap();
    let other = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::PlaybackSessionFinished,
            subject: DomainEventSubject::Source(other_source_id),
            library_id: Some(other_library_id),
            source_id: Some(other_source_id),
            idempotency_key: format!("playback:{other_source_id}"),
            payload_json: "{}".to_owned(),
        })
        .await
        .unwrap();

    let by_kind = store
        .list_outbox_events(
            OutboxEventListFilter {
                kind: Some(DomainEventKind::ItemMetadataRefreshed),
                ..Default::default()
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(by_kind, vec![metadata.clone()]);

    let by_status = store
        .list_outbox_events(
            OutboxEventListFilter {
                status: Some(OutboxEventStatus::Pending),
                ..Default::default()
            },
            PageRequest::new(2, 0),
        )
        .await
        .unwrap();
    assert_eq!(by_status.len(), 2);
    assert!(
        by_status
            .iter()
            .all(|event| event.status == OutboxEventStatus::Pending)
    );

    let by_library = store
        .list_outbox_events(
            OutboxEventListFilter {
                library_id: Some(library_id),
                ..Default::default()
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(by_library.len(), 2);
    assert!(by_library.iter().any(|event| event.id == scan.id));
    assert!(by_library.iter().any(|event| event.id == metadata.id));
    assert!(!by_library.iter().any(|event| event.id == other.id));

    let by_source = store
        .list_outbox_events(
            OutboxEventListFilter {
                source_id: Some(source_id),
                ..Default::default()
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(by_source, vec![metadata]);
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_webhook_endpoint_and_attempts() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();

    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: None,
            idempotency_key: format!("library_scan:{}", library.id),
            payload_json: format!(r#"{{"library_id":"{}"}}"#, library.id),
        })
        .await
        .unwrap();
    let endpoint_id = WebhookEndpointId::new();
    let endpoint = store
        .upsert_webhook_endpoint(NewWebhookEndpoint {
            id: endpoint_id,
            name: "Local Receiver".to_owned(),
            url: "https://example.test/taru-webhook".to_owned(),
            secret_env: Some("TARU_TEST_WEBHOOK_SECRET".to_owned()),
            subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
            timeout_ms: 5_000,
            max_attempts: 3,
            status: WebhookEndpointStatus::Enabled,
        })
        .await
        .unwrap();

    assert_eq!(endpoint.id, endpoint_id);
    assert_eq!(
        store.list_enabled_webhook_endpoints().await.unwrap().len(),
        1
    );
    assert_eq!(
        store.get_webhook_endpoint(endpoint_id).await.unwrap(),
        Some(endpoint.clone())
    );

    let attempt = store
        .create_webhook_delivery_attempt(NewWebhookDeliveryAttempt {
            id: WebhookDeliveryAttemptId::new(),
            endpoint_id,
            event_id: event.id,
            attempt_number: 1,
        })
        .await
        .unwrap();
    assert_eq!(attempt.status, WebhookDeliveryStatus::Pending);

    let failed = store
        .set_webhook_delivery_attempt_result(
            attempt.id,
            WebhookDeliveryStatus::Failed,
            Some(503),
            Some("receiver returned 503".to_owned()),
            Some("2026-05-15T00:00:10Z".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(failed.status, WebhookDeliveryStatus::Failed);
    assert_eq!(failed.http_status, Some(503));
    assert_eq!(
        store
            .list_webhook_delivery_attempts(event.id)
            .await
            .unwrap(),
        vec![failed]
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_automation_provider_and_artifacts() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();

    let provider_id = AutomationProviderId::new();
    let provider = store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("TARU_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![
                AutomationCapability::Summary,
                AutomationCapability::TitleMatch,
            ],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();

    assert_eq!(provider.id, provider_id);
    assert_eq!(
        store.list_enabled_automation_providers().await.unwrap(),
        vec![provider.clone()]
    );
    assert_eq!(
        store.get_automation_provider(provider_id).await.unwrap(),
        Some(provider)
    );

    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            library_id: Some(library.id),
            source_id: None,
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::Summary,
                    library_id: Some(library.id),
                    item_id: Some(item.id),
                    source_id: None,
                    prompt_json: r#"{"title":"The Matrix"}"#.to_owned(),
                    idempotency_key: format!("summary:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::Summary,
            kind: AutomationArtifactKind::Summary,
            library_id: Some(library.id),
            item_id: Some(item.id),
            source_id: None,
            artifact_json: r#"{"summary":"A generated summary."}"#.to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(artifact.status, AutomationArtifactStatus::Proposed);
    assert!(artifact.accepted_at.is_none());
    assert_eq!(
        store
            .list_automation_artifacts_for_job(job.id)
            .await
            .unwrap(),
        vec![artifact.clone()]
    );
    assert_eq!(
        store
            .list_automation_artifacts_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![artifact.clone()]
    );

    let accepted = store
        .set_automation_artifact_status(artifact.id, AutomationArtifactStatus::Accepted)
        .await
        .unwrap();
    assert_eq!(accepted.status, AutomationArtifactStatus::Accepted);
    assert!(accepted.accepted_at.is_some());
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_addon_registration() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let addon_id = AddonId::new();
    let manifest_json = r#"{
            "id":"example.metadata",
            "name":"Example Metadata",
            "version":"0.1.0",
            "protocol_version":"2026-05-15",
            "base_url":"https://example.test/addon"
        }"#
    .to_owned();
    let registration = store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.metadata".to_owned(),
            name: "Example Metadata".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json,
            granted_scopes: vec!["item_metadata_read".to_owned()],
            status: AddonStatus::Disabled,
        })
        .await
        .unwrap();

    assert_eq!(registration.id, addon_id);
    assert_eq!(registration.status, AddonStatus::Disabled);
    assert_eq!(registration.granted_scopes, vec!["item_metadata_read"]);
    assert_eq!(
        store.get_addon_registration(addon_id).await.unwrap(),
        Some(registration.clone())
    );
    assert_eq!(
        store
            .find_addon_registration_by_manifest_id("example.metadata")
            .await
            .unwrap(),
        Some(registration.clone())
    );
    assert_eq!(
        store
            .list_addon_registrations(Some(AddonStatus::Disabled))
            .await
            .unwrap(),
        vec![registration]
    );
    assert!(
        store
            .list_addon_registrations(Some(AddonStatus::Enabled))
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_addon_tokens_and_grants() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.metadata".to_owned(),
            name: "Example Metadata".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["item_metadata_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let first_id = AddonTokenId::new();
    let first = store
        .create_addon_token(NewAddonToken {
            id: first_id,
            addon_id,
            label: "initial".to_owned(),
            token_prefix: "taru_at_initial".to_owned(),
            token_hash: "sha256:first".to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(first.id, first_id);
    assert_eq!(first.addon_id, addon_id);
    assert_eq!(first.status, AddonTokenStatus::Active);
    assert_eq!(first.token_hash, "sha256:first");
    assert_eq!(
        store.list_addon_tokens(addon_id).await.unwrap(),
        vec![first.clone()]
    );

    let replacement_id = AddonTokenId::new();
    let (rotated, replacement) = store
        .rotate_addon_token(
            first_id,
            NewAddonToken {
                id: replacement_id,
                addon_id,
                label: "rotated".to_owned(),
                token_prefix: "taru_at_rotated".to_owned(),
                token_hash: "sha256:second".to_owned(),
            },
        )
        .await
        .unwrap();

    assert_eq!(rotated.status, AddonTokenStatus::Rotated);
    assert!(rotated.rotated_at.is_some());
    assert_eq!(replacement.status, AddonTokenStatus::Active);
    assert_eq!(replacement.token_hash, "sha256:second");

    assert_eq!(
        store
            .find_addon_token_by_hash("sha256:second")
            .await
            .unwrap()
            .unwrap()
            .id,
        replacement_id
    );

    let used = store
        .mark_addon_token_used(replacement_id)
        .await
        .unwrap()
        .unwrap();
    assert!(used.last_used_at.is_some());

    let revoked = store
        .revoke_addon_token(replacement_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(revoked.status, AddonTokenStatus::Revoked);
    assert!(revoked.revoked_at.is_some());

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let grants = store
        .replace_addon_grants(
            addon_id,
            vec![
                NewAddonGrant {
                    id: AddonGrantId::new(),
                    addon_id,
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library_id),
                },
                NewAddonGrant {
                    id: AddonGrantId::new(),
                    addon_id,
                    permission: AddonPermission::ArtworkWrite,
                    library_id: None,
                },
            ],
        )
        .await
        .unwrap();

    assert_eq!(grants.len(), 2);
    assert!(
        grants
            .iter()
            .any(|grant| grant.permission == AddonPermission::MetadataWrite
                && grant.library_id == Some(library_id))
    );
    assert!(grants.iter().any(
        |grant| grant.permission == AddonPermission::ArtworkWrite && grant.library_id.is_none()
    ));

    let replaced = store
        .replace_addon_grants(
            addon_id,
            vec![NewAddonGrant {
                id: AddonGrantId::new(),
                addon_id,
                permission: AddonPermission::SubtitleWrite,
                library_id: Some(library_id),
            }],
        )
        .await
        .unwrap();

    assert_eq!(replaced.len(), 1);
    assert_eq!(replaced[0].permission, AddonPermission::SubtitleWrite);
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_addon_side_effects_idempotently() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.metadata".to_owned(),
            name: "Example Metadata".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["item_metadata_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "runtime".to_owned(),
            token_prefix: "taru_at_runtime".to_owned(),
            token_hash: "sha256:runtime".to_owned(),
        })
        .await
        .unwrap();

    let source_id = MediaSourceId::new();
    let side_effect_id = AddonSideEffectId::new();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: side_effect_id,
            addon_id,
            token_id,
            permission: AddonPermission::MetadataWrite,
            library_id,
            target: AddonSideEffectTarget::media_source(source_id),
            idempotency_key: "metadata-demo".to_owned(),
            provenance_json: r#"{"origin":"reference-addon"}"#.to_owned(),
            payload_json: r#"{"title":"Demo"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();

    assert_eq!(side_effect.id, side_effect_id);
    assert_eq!(side_effect.addon_id, addon_id);
    assert_eq!(side_effect.token_id, token_id);
    assert_eq!(side_effect.library_id, library_id);
    assert_eq!(
        side_effect.target,
        AddonSideEffectTarget::media_source(source_id)
    );
    assert_eq!(
        side_effect.validation_status,
        AddonSideEffectValidationStatus::Accepted
    );
    assert_eq!(
        side_effect.apply_status,
        AddonSideEffectApplyStatus::Pending
    );
    assert_eq!(side_effect.apply_error_code, None);
    assert_eq!(side_effect.applied_item_id, None);
    assert_eq!(side_effect.applied_source, None);
    assert_eq!(side_effect.apply_report_json, None);
    assert_eq!(side_effect.applied_at, None);
    assert_eq!(
        side_effect.provenance_json,
        r#"{"origin":"reference-addon"}"#
    );
    assert_eq!(side_effect.payload_json, r#"{"title":"Demo"}"#);

    let duplicate = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_source(MediaSourceId::new()),
            idempotency_key: "metadata-demo".to_owned(),
            provenance_json: r#"{"origin":"duplicate"}"#.to_owned(),
            payload_json: r#"{"title":"Duplicate"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Rejected,
            safe_error_code: Some("forbidden".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(duplicate, side_effect);
    assert_eq!(
        store
            .find_addon_side_effect_by_idempotency_key(addon_id, "metadata-demo")
            .await
            .unwrap(),
        Some(side_effect)
    );
}

#[tokio::test]
async fn taru_database_sqlite_records_addon_side_effect_apply_outcome() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.metadata".to_owned(),
            name: "Example Metadata".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["item_metadata_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "runtime".to_owned(),
            token_prefix: "taru_at_runtime".to_owned(),
            token_hash: "sha256:runtime".to_owned(),
        })
        .await
        .unwrap();

    let item_id = MediaItemId::new();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::MetadataWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: "metadata-apply-demo".to_owned(),
            provenance_json: r#"{"origin":"reference-addon"}"#.to_owned(),
            payload_json: r#"{"title":"Demo"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();

    let applied = store
        .set_addon_side_effect_apply_outcome(
            side_effect.id,
            AddonSideEffectApplyOutcome {
                status: AddonSideEffectApplyStatus::Applied,
                error_code: None,
                item_id: Some(item_id),
                source: Some(format!("addon:{addon_id}")),
                report_json: Some(r#"{"kind":"metadata_write"}"#.to_owned()),
            },
        )
        .await
        .unwrap();

    assert_eq!(applied.apply_status, AddonSideEffectApplyStatus::Applied);
    assert_eq!(applied.apply_error_code, None);
    assert_eq!(applied.applied_item_id, Some(item_id));
    assert_eq!(applied.applied_source, Some(format!("addon:{addon_id}")));
    assert_eq!(
        applied.apply_report_json.as_deref(),
        Some(r#"{"kind":"metadata_write"}"#)
    );
    assert!(applied.applied_at.is_some());
    assert_eq!(
        store
            .find_addon_side_effect_by_idempotency_key(addon_id, "metadata-apply-demo")
            .await
            .unwrap()
            .unwrap()
            .apply_status,
        AddonSideEffectApplyStatus::Applied
    );
}

#[tokio::test]
async fn taru_database_sqlite_round_trips_addon_artwork_candidates_idempotently() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Artwork Candidate Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["item_artwork_suggest".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork runtime".to_owned(),
            token_prefix: "taru_at_artwork".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();

    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "artwork-candidate-demo".to_owned(),
            provenance_json: r#"{"origin":"reference-addon"}"#.to_owned(),
            payload_json: r#"{"intent":"propose_artwork"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();

    let source_uri = "https://cdn.example.test/poster.jpg";
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: source_uri.to_owned(),
            width: Some(1000),
            height: Some(1500),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(candidate.addon_id, addon_id);
    assert_eq!(candidate.side_effect_id, side_effect.id);
    assert_eq!(candidate.library_id, library_id);
    assert_eq!(candidate.item_id, item.id);
    assert_eq!(candidate.kind, ImageKind::Poster);
    assert_eq!(candidate.source_kind, ArtworkCandidateSourceKind::RemoteUrl);
    assert_eq!(candidate.source_uri, source_uri);
    assert_eq!(candidate.width, Some(1000));
    assert_eq!(candidate.height, Some(1500));
    assert_eq!(candidate.language.as_deref(), Some("en"));
    assert_eq!(candidate.status, ArtworkCandidateStatus::Proposed);
    assert!(!candidate.created_at.is_empty());
    assert!(!candidate.updated_at.is_empty());

    let duplicate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: source_uri.to_owned(),
            width: Some(999),
            height: Some(1499),
            language: Some("fr".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(duplicate, candidate);
    assert_eq!(
        store
            .find_artwork_candidate_by_source(
                addon_id,
                library_id,
                item.id,
                &ImageKind::Poster,
                ArtworkCandidateSourceKind::RemoteUrl,
                source_uri,
            )
            .await
            .unwrap(),
        Some(candidate.clone())
    );
    assert_eq!(
        store
            .list_artwork_candidates_for_item(item.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![candidate]
    );
}

#[tokio::test]
async fn taru_database_sqlite_accepts_artwork_candidate_into_managed_ingest_atomically() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Artwork Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["item_artwork_suggest".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork runtime".to_owned(),
            token_prefix: "taru_at_artwork".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "artwork-candidate-managed-ingest".to_owned(),
            provenance_json: r#"{"origin":"reference-addon"}"#.to_owned(),
            payload_json: r#"{"intent":"propose_artwork"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/poster.jpg?token=secret".to_owned(),
            width: Some(1000),
            height: Some(1500),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let ingest_id = ManagedArtworkIngestId::new();
    let job_id = JobId::new();

    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ingest_id,
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(
                    serde_json::json!({
                        "candidate_id": candidate.id,
                        "library_id": library_id,
                        "item_id": item.id,
                        "image_kind": "poster"
                    })
                    .to_string(),
                ),
            },
        )
        .await
        .unwrap();

    assert_eq!(accepted.candidate.status, ArtworkCandidateStatus::Accepted);
    assert_eq!(accepted.ingest.id, ingest_id);
    assert_eq!(accepted.ingest.candidate_id, candidate.id);
    assert_eq!(accepted.ingest.job_id, job_id);
    assert_eq!(accepted.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(accepted.job.kind, JobKind::ManagedArtworkIngest);
    assert_eq!(accepted.job.status, JobStatus::Queued);
    assert_eq!(accepted.job.resource_class, "artwork.ingest");
    assert!(
        !accepted
            .job
            .input_json
            .as_ref()
            .unwrap()
            .contains("token=secret")
    );
    assert_eq!(
        store
            .find_managed_artwork_ingest_by_candidate(candidate.id)
            .await
            .unwrap()
            .unwrap(),
        accepted.ingest
    );

    let replay = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ManagedArtworkIngestId::new(),
                candidate_id: candidate.id,
                job_id: JobId::new(),
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: JobId::new(),
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some("{}".to_owned()),
            },
        )
        .await
        .unwrap();
    assert_eq!(replay.ingest.id, accepted.ingest.id);
    assert_eq!(replay.job.id, accepted.job.id);

    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claim.candidate.id, candidate.id);
    assert_eq!(claim.candidate.source_uri, candidate.source_uri);
    assert_eq!(claim.ingest.id, accepted.ingest.id);
    assert_eq!(claim.ingest.status, ManagedArtworkIngestStatus::Fetching);
    assert_eq!(claim.job.id, accepted.job.id);
    assert_eq!(claim.job.status, JobStatus::Running);
    assert!(
        store
            .claim_next_queued_managed_artwork_ingest()
            .await
            .unwrap()
            .is_none()
    );

    let artifact_id = ManagedArtworkArtifactId::new();
    let committed = store
        .commit_managed_artwork_artifact(
            claim.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-demo".to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored","byte_len":68}"#.to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(committed.ingest.status, ManagedArtworkIngestStatus::Stored);
    assert_eq!(committed.ingest.artifact_id, Some(artifact_id));
    assert_eq!(committed.artifact.as_ref().unwrap().id, artifact_id);
    assert_eq!(committed.job.status, JobStatus::Succeeded);
    assert!(
        !committed
            .job
            .summary_json
            .as_ref()
            .unwrap()
            .contains("token=secret")
    );

    let stored_artifact = store
        .get_managed_artwork_artifact(artifact_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_artifact.ingest_id, claim.ingest.id);
    assert_eq!(
        stored_artifact.storage_uri,
        format!("managed-artwork://artifact/{artifact_id}")
    );
}

#[tokio::test]
async fn taru_database_sqlite_managed_artwork_ingest_requeue_resets_failed_ingest_and_job() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Managed Artwork Requeue Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork.requeue".to_owned(),
            name: "Example Artwork Requeue".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork requeue".to_owned(),
            token_prefix: "taru_at_requeue".to_owned(),
            token_hash: "sha256:artwork-requeue".to_owned(),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "managed-artwork-ingest-requeue".to_owned(),
            provenance_json: r#"{"origin":"reference-addon","token":"private"}"#.to_owned(),
            payload_json: r#"{"intent":"propose_artwork","url":"https://private.test"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/poster.png?token=secret".to_owned(),
            width: Some(1),
            height: Some(1),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();

    let job_id = JobId::new();
    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ManagedArtworkIngestId::new(),
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(r#"{"candidate_id":"private"}"#.to_owned()),
            },
        )
        .await
        .unwrap();
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    let failed = store
        .fail_managed_artwork_ingest(
            claim.ingest.id,
            "invalid_image".to_owned(),
            "private failure for https://cdn.example.test/poster.png?token=secret".to_owned(),
            Some(r#"{"private_path":"C:\\media\\poster.png"}"#.to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(failed.ingest.status, ManagedArtworkIngestStatus::Failed);
    assert_eq!(failed.job.status, JobStatus::Failed);
    assert!(failed.job.started_at.is_some());
    assert!(failed.job.completed_at.is_some());
    assert!(failed.job.error.is_some());
    assert!(failed.job.summary_json.is_some());

    let requeued = store
        .requeue_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap();
    assert!(requeued.requeued);
    assert!(requeued.had_failure);
    assert_eq!(requeued.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(requeued.ingest.failure_code, None);
    assert_eq!(requeued.ingest.artifact_id, None);
    assert_eq!(requeued.job.status, JobStatus::Queued);
    assert_eq!(requeued.job.error, None);
    assert_eq!(requeued.job.summary_json, None);
    assert_eq!(requeued.job.started_at, None);
    assert_eq!(requeued.job.completed_at, None);
    assert_eq!(requeued.job.input_json, accepted.job.input_json);

    let replay = store
        .requeue_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap();
    assert!(!replay.requeued);
    assert!(!replay.had_failure);
    assert_eq!(replay.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(replay.job.status, JobStatus::Queued);

    let retry_claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retry_claim.ingest.id, accepted.ingest.id);
    assert_eq!(
        retry_claim.ingest.status,
        ManagedArtworkIngestStatus::Fetching
    );
    assert_eq!(retry_claim.job.id, accepted.job.id);
    assert_eq!(retry_claim.job.status, JobStatus::Running);

    let running_requeue = store
        .requeue_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap_err();
    assert!(matches!(running_requeue, TaruError::Conflict { .. }));

    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            retry_claim.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: retry_claim.ingest.id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-requeue-stored".to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();
    let stored_requeue = store
        .requeue_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap_err();
    assert!(matches!(stored_requeue, TaruError::Conflict { .. }));
}

#[tokio::test]
async fn taru_database_sqlite_managed_artwork_startup_recovery_fails_only_claimed_ingests() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Managed Artwork Startup Recovery".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork.startup-recovery".to_owned(),
            name: "Example Artwork Startup Recovery".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork startup recovery".to_owned(),
            token_prefix: "taru_at_recovery".to_owned(),
            token_hash: "sha256:artwork-startup-recovery".to_owned(),
        })
        .await
        .unwrap();

    async fn accepted_ingest(
        store: &TaruDatabase,
        addon_id: AddonId,
        token_id: AddonTokenId,
        library_id: LibraryId,
        item_id: MediaItemId,
        key: &str,
    ) -> ManagedArtworkAcceptanceRecord {
        let side_effect = store
            .create_addon_side_effect(NewAddonSideEffect {
                id: AddonSideEffectId::new(),
                addon_id,
                token_id,
                permission: AddonPermission::ArtworkWrite,
                library_id,
                target: AddonSideEffectTarget::media_item(item_id),
                idempotency_key: key.to_owned(),
                provenance_json: r#"{"origin":"reference-addon","token":"private"}"#.to_owned(),
                payload_json: r#"{"intent":"propose_artwork","url":"https://private.test"}"#
                    .to_owned(),
                validation_status: AddonSideEffectValidationStatus::Accepted,
                safe_error_code: None,
            })
            .await
            .unwrap();
        let candidate = store
            .create_artwork_candidate(NewArtworkCandidate {
                id: ArtworkCandidateId::new(),
                addon_id,
                side_effect_id: side_effect.id,
                library_id,
                item_id,
                kind: ImageKind::Poster,
                source_kind: ArtworkCandidateSourceKind::RemoteUrl,
                source_uri: format!("https://cdn.example.test/{key}.png?token=secret"),
                width: Some(1),
                height: Some(1),
                language: Some("en".to_owned()),
            })
            .await
            .unwrap();
        let job_id = JobId::new();
        store
            .accept_managed_artwork_candidate_ingest(
                candidate.id,
                NewManagedArtworkIngest {
                    id: ManagedArtworkIngestId::new(),
                    candidate_id: candidate.id,
                    job_id,
                    library_id,
                    item_id,
                    kind: ImageKind::Poster,
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                },
                NewJob {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    resource_class: "artwork.ingest".to_owned(),
                    library_id: Some(library_id),
                    source_id: None,
                    input_json: Some(r#"{"candidate_id":"redacted"}"#.to_owned()),
                },
            )
            .await
            .unwrap()
    }

    let fetching = accepted_ingest(
        &store,
        addon_id,
        token_id,
        library_id,
        item.id,
        "startup-fetching",
    )
    .await;
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claim.ingest.id, fetching.ingest.id);
    let queued = accepted_ingest(
        &store,
        addon_id,
        token_id,
        library_id,
        item.id,
        "startup-queued",
    )
    .await;

    let recovered = store
        .fail_unfinished_managed_artwork_ingests(
            "startup_recovery".to_owned(),
            "managed artwork ingest was unfinished during server startup".to_owned(),
            Some(r#"{"status":"failed","failure_code":"startup_recovery"}"#.to_owned()),
        )
        .await
        .unwrap();

    let fetching_ingest = store
        .get_managed_artwork_ingest(fetching.ingest.id)
        .await
        .unwrap()
        .unwrap();
    let fetching_job = store.get_job(fetching.job.id).await.unwrap().unwrap();
    let queued_ingest = store
        .get_managed_artwork_ingest(queued.ingest.id)
        .await
        .unwrap()
        .unwrap();
    let queued_job = store.get_job(queued.job.id).await.unwrap().unwrap();

    assert_eq!(recovered, 1);
    assert_eq!(fetching_ingest.status, ManagedArtworkIngestStatus::Failed);
    assert_eq!(
        fetching_ingest.failure_code.as_deref(),
        Some("startup_recovery")
    );
    assert_eq!(fetching_job.status, JobStatus::Failed);
    assert_eq!(
        fetching_job.error.as_deref(),
        Some("managed artwork ingest was unfinished during server startup")
    );
    assert!(fetching_job.summary_json.is_some());
    assert_eq!(queued_ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(queued_ingest.failure_code, None);
    assert_eq!(queued_job.status, JobStatus::Queued);
    assert_eq!(queued_job.error, None);

    let requeued = store
        .requeue_managed_artwork_ingest(fetching.ingest.id)
        .await
        .unwrap();
    assert!(requeued.requeued);
    assert_eq!(requeued.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(requeued.job.status, JobStatus::Queued);
}

#[tokio::test]
async fn taru_database_sqlite_publishes_stored_managed_artifact_as_selected_artwork_idempotently() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Selected Artwork Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork".to_owned(),
            token_prefix: "taru_at_art".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "selected-artwork-demo".to_owned(),
            provenance_json: "{}".to_owned(),
            payload_json: "{}".to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/poster.png".to_owned(),
            width: Some(1),
            height: Some(1),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let ingest_id = ManagedArtworkIngestId::new();
    let job_id = JobId::new();
    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ingest_id,
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some("{}".to_owned()),
            },
        )
        .await
        .unwrap();
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            accepted.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-selected-artwork".to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();

    let published = store.publish_selected_artwork(artifact_id).await.unwrap();

    assert_eq!(published.selected_artwork.library_id, library_id);
    assert_eq!(published.selected_artwork.item_id, item.id);
    assert_eq!(published.selected_artwork.kind, ImageKind::Poster);
    assert_eq!(published.selected_artwork.artifact_id, artifact_id);
    assert_eq!(published.artifact.id, artifact_id);
    assert!(published.changed);

    let replay = store.publish_selected_artwork(artifact_id).await.unwrap();
    assert_eq!(replay.selected_artwork.id, published.selected_artwork.id);
    assert_eq!(replay.selected_artwork.artifact_id, artifact_id);
    assert!(!replay.changed);

    let listed = store.list_selected_artwork_for_item(item.id).await.unwrap();
    assert_eq!(listed, vec![published.selected_artwork.clone()]);
    assert_eq!(
        store
            .get_selected_artwork(published.selected_artwork.id)
            .await
            .unwrap(),
        Some(published.selected_artwork)
    );
}

#[tokio::test]
async fn taru_database_sqlite_publishes_selected_artwork_with_item_kind_guard() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Guarded Selection Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let other_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Other Item".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_item(&other_item).await.unwrap();
    for item_id in [item.id, other_item.id] {
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id,
                item_id,
                provisional: false,
            })
            .await
            .unwrap();
    }

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork".to_owned(),
            token_prefix: "taru_at_art".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "guarded-selection-demo".to_owned(),
            provenance_json: "{}".to_owned(),
            payload_json: "{}".to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/guarded-poster.png".to_owned(),
            width: Some(1),
            height: Some(1),
            language: None,
        })
        .await
        .unwrap();
    let ingest_id = ManagedArtworkIngestId::new();
    let job_id = JobId::new();
    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ingest_id,
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some("{}".to_owned()),
            },
        )
        .await
        .unwrap();
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            accepted.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-guarded-selection".to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();

    let wrong_item = store
        .publish_selected_artwork_for_item_kind(other_item.id, ImageKind::Poster, artifact_id)
        .await
        .unwrap_err();
    assert!(matches!(wrong_item, TaruError::Conflict { .. }));
    let wrong_kind = store
        .publish_selected_artwork_for_item_kind(item.id, ImageKind::Backdrop, artifact_id)
        .await
        .unwrap_err();
    assert!(matches!(wrong_kind, TaruError::Conflict { .. }));

    let published = store
        .publish_selected_artwork_for_item_kind(item.id, ImageKind::Poster, artifact_id)
        .await
        .unwrap();
    assert_eq!(published.selected_artwork.item_id, item.id);
    assert_eq!(published.selected_artwork.kind, ImageKind::Poster);
    assert_eq!(published.selected_artwork.artifact_id, artifact_id);
    assert!(published.changed);

    let replay = store
        .publish_selected_artwork_for_item_kind(item.id, ImageKind::Poster, artifact_id)
        .await
        .unwrap();
    assert_eq!(replay.selected_artwork.id, published.selected_artwork.id);
    assert!(!replay.changed);
}

#[tokio::test]
async fn taru_database_sqlite_selected_artwork_unpublish_without_deleting_artifact() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Unpublish Artwork Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork".to_owned(),
            token_prefix: "taru_at_art".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "unpublish-artwork-demo".to_owned(),
            provenance_json: "{}".to_owned(),
            payload_json: "{}".to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/unpublish-poster.png".to_owned(),
            width: Some(1),
            height: Some(1),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let ingest_id = ManagedArtworkIngestId::new();
    let job_id = JobId::new();
    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ingest_id,
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some("{}".to_owned()),
            },
        )
        .await
        .unwrap();
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            accepted.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-unpublish-demo".to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();
    let published = store.publish_selected_artwork(artifact_id).await.unwrap();

    let unpublished = store
        .unpublish_selected_artwork_for_item_kind(item.id, ImageKind::Poster)
        .await
        .unwrap();

    assert!(unpublished.changed);
    assert_eq!(unpublished.item_id, item.id);
    assert_eq!(unpublished.kind, ImageKind::Poster);
    assert_eq!(
        unpublished.unpublished.as_ref().unwrap().id,
        published.selected_artwork.id
    );
    assert_eq!(unpublished.artifact.as_ref().unwrap().id, artifact_id);
    assert_eq!(
        store
            .get_selected_artwork(published.selected_artwork.id)
            .await
            .unwrap(),
        None
    );
    assert!(
        store
            .list_selected_artwork_for_item(item.id)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .get_managed_artwork_artifact(artifact_id)
            .await
            .unwrap()
            .unwrap()
            .id,
        artifact_id
    );

    let lifecycle = store
        .list_managed_artwork_artifact_lifecycle(
            ManagedArtworkArtifactLifecycleFilter::CleanupCandidates,
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(lifecycle.artifacts.len(), 1);
    assert_eq!(lifecycle.artifacts[0].artifact.id, artifact_id);
    assert!(lifecycle.artifacts[0].cleanup_candidate());

    let replay = store
        .unpublish_selected_artwork_for_item_kind(item.id, ImageKind::Poster)
        .await
        .unwrap();
    assert!(!replay.changed);
    assert!(replay.unpublished.is_none());
    assert!(replay.artifact.is_none());
}

#[tokio::test]
async fn taru_database_sqlite_lists_managed_artwork_gallery_for_item_without_raw_locators() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Gallery Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork".to_owned(),
            token_prefix: "taru_at_art".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "gallery-demo".to_owned(),
            provenance_json: r#"{"token":"addon-secret"}"#.to_owned(),
            payload_json: r#"{"source_uri":"https://cdn.example.test/poster.png?token=secret"}"#
                .to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/poster.png?token=secret".to_owned(),
            width: Some(1000),
            height: Some(1500),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let ingest_id = ManagedArtworkIngestId::new();
    let job_id = JobId::new();
    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ingest_id,
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some("{}".to_owned()),
            },
        )
        .await
        .unwrap();
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            accepted.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id: item.id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-private-gallery-hash".to_owned()),
                width: Some(1000),
                height: Some(1500),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();
    let published = store.publish_selected_artwork(artifact_id).await.unwrap();

    let gallery = store
        .get_managed_artwork_gallery_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let body = serde_json::to_string(&gallery).unwrap();

    assert_eq!(gallery.item_id, item.id);
    assert_eq!(gallery.summary.candidates, 1);
    assert_eq!(gallery.summary.artifacts, 1);
    assert_eq!(gallery.summary.selected, 1);
    assert_eq!(gallery.candidates[0].id, candidate.id);
    assert_eq!(
        gallery.candidates[0].source_kind,
        ArtworkCandidateSourceKind::RemoteUrl
    );
    assert_eq!(
        gallery.candidates[0].status,
        ArtworkCandidateStatus::Accepted
    );
    assert_eq!(gallery.candidates[0].artifact_id, Some(artifact_id));
    assert!(gallery.candidates[0].has_stored_artifact());
    assert!(gallery.candidates[0].selected());
    assert_eq!(gallery.artifacts[0].id, artifact_id);
    assert_eq!(gallery.artifacts[0].candidate_id, candidate.id);
    assert!(gallery.artifacts[0].has_content_hash);
    assert!(gallery.artifacts[0].selected());
    assert_eq!(
        gallery.selected[0].selected_artwork.id,
        published.selected_artwork.id
    );
    assert_eq!(gallery.selected[0].artifact.id, artifact_id);
    assert!(!body.contains("https://cdn.example.test"));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("source_uri"));
    assert!(!body.contains("storage_uri"));
    assert!(!body.contains("managed-artwork://"));
    assert!(!body.contains("sha256-private-gallery-hash"));
    assert!(!body.contains("\"content_hash\""));
}

#[tokio::test]
async fn taru_database_sqlite_lists_managed_artwork_lifecycle_with_selected_artwork_protection() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Lifecycle Artwork Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork".to_owned(),
            name: "Example Artwork".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork".to_owned(),
            token_prefix: "taru_at_art".to_owned(),
            token_hash: "sha256:artwork".to_owned(),
        })
        .await
        .unwrap();
    let first_side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "lifecycle-artwork-demo-selected".to_owned(),
            provenance_json: "{}".to_owned(),
            payload_json: "{}".to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let second_side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item.id),
            idempotency_key: "lifecycle-artwork-demo-orphan".to_owned(),
            provenance_json: "{}".to_owned(),
            payload_json: "{}".to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();

    let first_candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: first_side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/poster-one.png".to_owned(),
            width: Some(1),
            height: Some(1),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let second_candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id: second_side_effect.id,
            library_id,
            item_id: item.id,
            kind: ImageKind::Backdrop,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/backdrop-one.png".to_owned(),
            width: Some(1),
            height: Some(1),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();

    let mut artifact_ids = Vec::new();
    for (candidate, kind, byte_len) in [
        (first_candidate, ImageKind::Poster, 68_u64),
        (second_candidate, ImageKind::Backdrop, 102_u64),
    ] {
        let ingest_id = ManagedArtworkIngestId::new();
        let job_id = JobId::new();
        let accepted = store
            .accept_managed_artwork_candidate_ingest(
                candidate.id,
                NewManagedArtworkIngest {
                    id: ingest_id,
                    candidate_id: candidate.id,
                    job_id,
                    library_id,
                    item_id: item.id,
                    kind: kind.clone(),
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                },
                NewJob {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    resource_class: "artwork.ingest".to_owned(),
                    library_id: Some(library_id),
                    source_id: None,
                    input_json: Some("{}".to_owned()),
                },
            )
            .await
            .unwrap();
        let claim = store
            .claim_next_queued_managed_artwork_ingest()
            .await
            .unwrap()
            .unwrap();
        let artifact_id = ManagedArtworkArtifactId::new();
        store
            .commit_managed_artwork_artifact(
                accepted.ingest.id,
                NewManagedArtworkArtifact {
                    id: artifact_id,
                    ingest_id: claim.ingest.id,
                    library_id,
                    item_id: item.id,
                    kind,
                    storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                    content_hash: Some(format!("sha256-{artifact_id}")),
                    width: Some(1),
                    height: Some(1),
                    byte_len: Some(byte_len),
                    media_type: Some("image/png".to_owned()),
                },
                Some(r#"{"status":"stored"}"#.to_owned()),
            )
            .await
            .unwrap();
        artifact_ids.push(artifact_id);
    }

    store
        .publish_selected_artwork(artifact_ids[0])
        .await
        .unwrap();

    let snapshot = store
        .list_managed_artwork_artifact_lifecycle(
            ManagedArtworkArtifactLifecycleFilter::All,
            PageRequest::first_page(),
        )
        .await
        .unwrap();

    assert_eq!(snapshot.summary.total_artifacts, 2);
    assert_eq!(snapshot.summary.protected_artifacts, 1);
    assert_eq!(snapshot.summary.cleanup_candidate_artifacts, 1);
    assert_eq!(snapshot.summary.known_total_bytes, 170);
    assert_eq!(snapshot.summary.known_protected_bytes, 68);
    assert_eq!(snapshot.summary.known_cleanup_candidate_bytes, 102);
    assert_eq!(snapshot.artifacts.len(), 2);
    assert!(
        snapshot
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact.id == artifact_ids[0]
                && artifact.selected_artwork_count == 1
                && !artifact.cleanup_candidate())
    );
    assert!(
        snapshot
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact.id == artifact_ids[1]
                && artifact.selected_artwork_count == 0
                && artifact.cleanup_candidate())
    );

    let cleanup = store
        .list_managed_artwork_artifact_lifecycle(
            ManagedArtworkArtifactLifecycleFilter::CleanupCandidates,
            PageRequest::first_page(),
        )
        .await
        .unwrap();

    assert_eq!(cleanup.artifacts.len(), 1);
    assert_eq!(cleanup.artifacts[0].artifact.id, artifact_ids[1]);
    assert!(cleanup.artifacts[0].cleanup_candidate());
}

#[tokio::test]
async fn taru_database_sqlite_cleanup_marks_only_unselected_managed_artwork_artifacts_deleted() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let library_id = LibraryId::new();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Cleanup Artwork Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "example.artwork.cleanup".to_owned(),
            name: "Example Artwork Cleanup".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "artwork".to_owned(),
            token_prefix: "taru_at_art".to_owned(),
            token_hash: "sha256:artwork-cleanup".to_owned(),
        })
        .await
        .unwrap();

    let mut artifact_ids = Vec::new();
    for (index, kind) in [ImageKind::Poster, ImageKind::Backdrop]
        .into_iter()
        .enumerate()
    {
        let side_effect = store
            .create_addon_side_effect(NewAddonSideEffect {
                id: AddonSideEffectId::new(),
                addon_id,
                token_id,
                permission: AddonPermission::ArtworkWrite,
                library_id,
                target: AddonSideEffectTarget::media_item(item.id),
                idempotency_key: format!("cleanup-artwork-demo-{index}"),
                provenance_json: "{}".to_owned(),
                payload_json: "{}".to_owned(),
                validation_status: AddonSideEffectValidationStatus::Accepted,
                safe_error_code: None,
            })
            .await
            .unwrap();
        let candidate = store
            .create_artwork_candidate(NewArtworkCandidate {
                id: ArtworkCandidateId::new(),
                addon_id,
                side_effect_id: side_effect.id,
                library_id,
                item_id: item.id,
                kind: kind.clone(),
                source_kind: ArtworkCandidateSourceKind::RemoteUrl,
                source_uri: format!("https://cdn.example.test/cleanup-{index}.png"),
                width: Some(1),
                height: Some(1),
                language: Some("en".to_owned()),
            })
            .await
            .unwrap();
        let ingest_id = ManagedArtworkIngestId::new();
        let job_id = JobId::new();
        let accepted = store
            .accept_managed_artwork_candidate_ingest(
                candidate.id,
                NewManagedArtworkIngest {
                    id: ingest_id,
                    candidate_id: candidate.id,
                    job_id,
                    library_id,
                    item_id: item.id,
                    kind: kind.clone(),
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                },
                NewJob {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    resource_class: "artwork.ingest".to_owned(),
                    library_id: Some(library_id),
                    source_id: None,
                    input_json: Some("{}".to_owned()),
                },
            )
            .await
            .unwrap();
        let claim = store
            .claim_next_queued_managed_artwork_ingest()
            .await
            .unwrap()
            .unwrap();
        let artifact_id = ManagedArtworkArtifactId::new();
        store
            .commit_managed_artwork_artifact(
                accepted.ingest.id,
                NewManagedArtworkArtifact {
                    id: artifact_id,
                    ingest_id: claim.ingest.id,
                    library_id,
                    item_id: item.id,
                    kind,
                    storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                    content_hash: Some(format!("sha256-cleanup-{index}")),
                    width: Some(1),
                    height: Some(1),
                    byte_len: Some(68),
                    media_type: Some("image/png".to_owned()),
                },
                Some(r#"{"status":"stored"}"#.to_owned()),
            )
            .await
            .unwrap();
        artifact_ids.push(artifact_id);
    }

    store
        .publish_selected_artwork(artifact_ids[0])
        .await
        .unwrap();
    let report = store
        .cleanup_unselected_managed_artwork_artifacts(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(report.examined_artifacts, 1);
    assert_eq!(report.cleanup_candidate_artifacts, 1);
    assert_eq!(report.cleaned_artifacts.len(), 1);
    assert_eq!(report.cleaned_artifacts[0].id, artifact_ids[1]);
    assert!(
        store
            .get_managed_artwork_artifact(artifact_ids[0])
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        store
            .get_managed_artwork_artifact(artifact_ids[1])
            .await
            .unwrap(),
        None
    );

    let lifecycle = store
        .list_managed_artwork_artifact_lifecycle(
            ManagedArtworkArtifactLifecycleFilter::All,
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(lifecycle.summary.total_artifacts, 1);
    assert_eq!(lifecycle.summary.protected_artifacts, 1);
    assert_eq!(lifecycle.summary.cleanup_candidate_artifacts, 0);
}

#[tokio::test]
async fn taru_database_sqlite_rejects_addon_token_rotation_across_addons() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let first_addon_id = AddonId::new();
    let second_addon_id = AddonId::new();

    for (addon_id, manifest_id) in [
        (first_addon_id, "example.first"),
        (second_addon_id, "example.second"),
    ] {
        store
            .upsert_addon_registration(NewAddonRegistration {
                id: addon_id,
                manifest_id: manifest_id.to_owned(),
                name: manifest_id.to_owned(),
                version: "0.1.0".to_owned(),
                protocol_version: "2026-05-15".to_owned(),
                base_url: "https://example.test/addon".to_owned(),
                manifest_json: "{}".to_owned(),
                granted_scopes: vec!["item_metadata_read".to_owned()],
                status: AddonStatus::Enabled,
            })
            .await
            .unwrap();
    }

    let first_token_id = AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: first_token_id,
            addon_id: first_addon_id,
            label: "first".to_owned(),
            token_prefix: "taru_at_first".to_owned(),
            token_hash: "sha256:first-addon".to_owned(),
        })
        .await
        .unwrap();

    let error = store
        .rotate_addon_token(
            first_token_id,
            NewAddonToken {
                id: AddonTokenId::new(),
                addon_id: second_addon_id,
                label: "wrong aggregate".to_owned(),
                token_prefix: "taru_at_wrong".to_owned(),
                token_hash: "sha256:wrong-addon".to_owned(),
            },
        )
        .await
        .unwrap_err();

    assert!(matches!(error, TaruError::Conflict { .. }));
    assert_eq!(
        store
            .get_addon_token(first_token_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        AddonTokenStatus::Active
    );
    assert!(
        store
            .list_addon_tokens(second_addon_id)
            .await
            .unwrap()
            .is_empty()
    );
}

async fn seed_media_item_with_source(
    store: &TaruDatabase,
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
