use super::*;

#[tokio::test]
async fn empty_sources_and_items_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let sources_path = format!("/libraries/{library_id}/sources");

    let sources = request_json::<nako_api::public_client::LibrarySourcesResponse>(
        &router,
        Method::GET,
        &sources_path,
    )
    .await;
    let items =
        request_json::<nako_api::public_client::ItemsResponse>(&router, Method::GET, "/items")
            .await;

    assert_eq!(sources.library.id, library_id.to_string());
    assert_eq!(sources.page.limit, nako_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(sources.page.offset, 0);
    assert_eq!(sources.page.returned, 0);
    assert!(sources.sources.is_empty());
    assert_eq!(items.page.limit, nako_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(items.page.offset, 0);
    assert_eq!(items.page.returned, 0);
    assert!(items.items.is_empty());
}

#[tokio::test]
async fn search_route_returns_indexed_items() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Search Route Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert(
            SearchDocument::from_facet_labels(
                item.id,
                item.metadata.title.clone(),
                "A route test fixture",
                vec!["genre:test".to_owned()],
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let router = build_router(app);

    let result = request_json::<nako_api::public_client::SearchResponse>(
        &router,
        Method::GET,
        "/search?q=route&facet=genre:test&limit=12&offset=0",
    )
    .await;

    assert_eq!(result.page.limit, 12);
    assert_eq!(result.page.offset, 0);
    assert_eq!(result.page.returned, 1);
    assert_eq!(result.hits[0].item.id, item.id.to_string());
}

#[tokio::test]
async fn browse_routes_return_catalog_graph() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Browse Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Browse Demo.mkv".to_owned(),
        file_name: "Browse Demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    let person = Person {
        id: PersonId::new(),
        name: "Demo Actor".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let genre = Genre {
        id: GenreId::new(),
        name: "Science Fiction".to_owned(),
        source: MetadataSource::Nfo,
    };
    let tag = Tag {
        id: TagId::new(),
        name: "favorite".to_owned(),
        source: MetadataSource::User,
    };
    let image = ImageAsset {
        id: ImageAssetId::new(),
        owner: ImageOwner::Item(item.id),
        kind: ImageKind::Poster,
        source_uri: "local:///poster.jpg".to_owned(),
        provider: nako_core::ExternalProvider::Local,
        cache_uri: None,
        width: None,
        height: None,
        language: None,
        selected: true,
        content_hash: None,
        etag: None,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store.upsert_person(&person).await.unwrap();
    store
        .upsert_item_credit(&ItemCredit {
            item_id: item.id,
            person_id: person.id,
            role: CreditRole::Actor,
            character: Some("Lead".to_owned()),
            sort_order: Some(0),
        })
        .await
        .unwrap();
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
    store.upsert_image_asset(&image).await.unwrap();
    let router = build_router(app);

    let detail = request_json::<nako_api::public_client::ItemDetailResponse>(
        &router,
        Method::GET,
        &format!("/items/{}", item.id),
    )
    .await;
    let detail_json =
        request_json::<serde_json::Value>(&router, Method::GET, &format!("/items/{}", item.id))
            .await;
    let sources_json = request_json::<serde_json::Value>(
        &router,
        Method::GET,
        &format!("/libraries/{library_id}/sources"),
    )
    .await;
    let credits = request_json::<nako_api::public_client::ItemCreditsResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/credits", item.id),
    )
    .await;
    let images = request_json::<nako_api::public_client::ImagesResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/images", item.id),
    )
    .await;
    let people =
        request_json::<nako_api::public_client::PeopleResponse>(&router, Method::GET, "/people")
            .await;
    let person_items = request_json::<nako_api::public_client::PersonItemsResponse>(
        &router,
        Method::GET,
        &format!("/people/{}/items", person.id),
    )
    .await;
    let tags =
        request_json::<nako_api::public_client::TagsResponse>(&router, Method::GET, "/tags").await;
    let tag_items = request_json::<nako_api::public_client::TagItemsResponse>(
        &router,
        Method::GET,
        &format!("/tags/{}/items", tag.id),
    )
    .await;
    let genres =
        request_json::<nako_api::public_client::GenreListResponse>(&router, Method::GET, "/genres")
            .await;
    let genre_items = request_json::<nako_api::public_client::GenreItemsResponse>(
        &router,
        Method::GET,
        &format!("/genres/{}/items", genre.id),
    )
    .await;

    assert_eq!(detail.item.id, item.id.to_string());
    assert_eq!(detail.sources[0].id, source.id.to_string());
    assert!(detail_json["sources"][0].get("locator").is_none());
    assert!(
        sources_json["sources"][0]["source"]
            .get("locator")
            .is_none()
    );
    assert_eq!(detail.credits.len(), 1);
    assert_eq!(credits.people[0].name, "Demo Actor");
    assert!(detail.images.is_empty());
    assert!(images.images.is_empty());
    assert_eq!(people.people[0].id, person.id.to_string());
    assert_eq!(person_items.items[0].id, item.id.to_string());
    assert_eq!(tags.tags[0].name, "favorite");
    assert_eq!(tag_items.items[0].id, item.id.to_string());
    assert_eq!(genres.genres[0].name, "Science Fiction");
    assert_eq!(genre_items.items[0].id, item.id.to_string());
}

#[tokio::test]
async fn public_browse_routes_filter_libraries_and_items_by_effective_access() {
    let temp = tempfile::tempdir().unwrap();
    let allowed_root = temp.path().join("allowed");
    let blocked_root = temp.path().join("blocked");
    fs::create_dir_all(&allowed_root).unwrap();
    fs::create_dir_all(&blocked_root).unwrap();
    let allowed_library_id = LibraryId::new();
    let blocked_library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        libraries: vec![
            LocalLibraryConfig {
                id: allowed_library_id,
                name: "Allowed".to_owned(),
                root: allowed_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: blocked_library_id,
                name: "Blocked".to_owned(),
                root: blocked_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
        ],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let allowed_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Allowed Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let blocked_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Blocked Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let allowed_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: allowed_library_id,
        item_id: allowed_item.id,
        locator: "local:///Allowed Movie.mkv".to_owned(),
        file_name: "Allowed Movie.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    let blocked_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: blocked_library_id,
        item_id: blocked_item.id,
        locator: "local:///Blocked Movie.mkv".to_owned(),
        file_name: "Blocked Movie.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };

    store.upsert_media_item(&allowed_item).await.unwrap();
    store.upsert_media_item(&blocked_item).await.unwrap();
    store.upsert_media_source(&allowed_source).await.unwrap();
    store.upsert_media_source(&blocked_source).await.unwrap();
    let principal =
        local_viewer_with_library_access(&store, allowed_library_id, LibraryAccessLevel::Play)
            .await;
    let router = public_client_router_with_principal(app, principal);

    let libraries = request_json::<nako_api::public_client::LibraryListResponse>(
        &router,
        Method::GET,
        "/libraries",
    )
    .await;
    let items =
        request_json::<nako_api::public_client::ItemsResponse>(&router, Method::GET, "/items")
            .await;
    let blocked_detail =
        response_for(&router, Method::GET, &format!("/items/{}", blocked_item.id)).await;
    let blocked_sources = response_for(
        &router,
        Method::GET,
        &format!("/libraries/{blocked_library_id}/sources"),
    )
    .await;

    assert_eq!(libraries.page.returned, 1);
    assert_eq!(libraries.libraries[0].id, allowed_library_id.to_string());
    assert_eq!(items.page.returned, 1);
    assert_eq!(items.items[0].id, allowed_item.id.to_string());
    assert_eq!(blocked_detail.status(), StatusCode::FORBIDDEN);
    assert_eq!(blocked_sources.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn library_items_route_returns_scoped_items_and_hides_inaccessible_libraries() {
    let temp = tempfile::tempdir().unwrap();
    let allowed_root = temp.path().join("allowed");
    let blocked_root = temp.path().join("blocked");
    fs::create_dir_all(&allowed_root).unwrap();
    fs::create_dir_all(&blocked_root).unwrap();
    let allowed_library_id = LibraryId::new();
    let blocked_library_id = LibraryId::new();
    let config = NakoServerConfig {
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
        libraries: vec![
            LocalLibraryConfig {
                id: allowed_library_id,
                name: "Allowed".to_owned(),
                root: allowed_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: blocked_library_id,
                name: "Blocked".to_owned(),
                root: blocked_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
        ],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let allowed_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Allowed Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let blocked_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Blocked Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let allowed_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: allowed_library_id,
        item_id: allowed_item.id,
        locator: "local:///Allowed Movie.mkv".to_owned(),
        file_name: "Allowed Movie.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    let blocked_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: blocked_library_id,
        item_id: blocked_item.id,
        locator: "local:///Blocked Movie.mkv".to_owned(),
        file_name: "Blocked Movie.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };

    store.upsert_media_item(&allowed_item).await.unwrap();
    store.upsert_media_item(&blocked_item).await.unwrap();
    store.upsert_media_source(&allowed_source).await.unwrap();
    store.upsert_media_source(&blocked_source).await.unwrap();
    let principal =
        local_viewer_with_library_access(&store, allowed_library_id, LibraryAccessLevel::Browse)
            .await;
    let router = public_client_router_with_principal(app, principal);

    let allowed_items = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!("/libraries/{allowed_library_id}/items?limit=25&offset=0"),
    )
    .await;
    let hidden_library = response_for(
        &router,
        Method::GET,
        &format!("/libraries/{blocked_library_id}/items"),
    )
    .await;
    let hidden_body = body_json::<nako_api::public_client::ErrorResponse>(hidden_library).await;

    assert_eq!(allowed_items.library.id, allowed_library_id.to_string());
    assert_eq!(allowed_items.page.limit, 25);
    assert_eq!(allowed_items.page.offset, 0);
    assert_eq!(allowed_items.page.returned, 1);
    assert_eq!(allowed_items.items[0].id, allowed_item.id.to_string());
    assert_eq!(
        nako_api::public_client::ClientErrorCode::from_code(&hidden_body.code),
        Some(nako_api::public_client::ClientErrorCode::NotFound)
    );
}

#[tokio::test]
async fn library_items_route_applies_kind_watch_state_and_last_played_query() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
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
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let in_progress_movie = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "In Progress Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let watched_movie = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Watched Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let in_progress_episode = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Episode,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "In Progress Episode".to_owned(),
            ..CanonicalMetadata::default()
        },
    };

    for item in [&in_progress_movie, &watched_movie, &in_progress_episode] {
        store.upsert_media_item(item).await.unwrap();
        store
            .upsert_media_source(&MediaSource {
                id: MediaSourceId::new(),
                library_id,
                item_id: item.id,
                locator: format!("local:///{}.mkv", item.metadata.title),
                file_name: format!("{}.mkv", item.metadata.title),
                size_bytes: Some(5),
                fingerprint: None,
            })
            .await
            .unwrap();
    }

    let principal =
        local_viewer_with_library_access(&store, library_id, LibraryAccessLevel::Browse).await;
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.principal_id.clone(),
            item_id: in_progress_movie.id,
            source_id: None,
            resume_position_ms: Some(10_000),
            duration_ms: Some(100_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(200),
            updated_at_ms: 200,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.principal_id.clone(),
            item_id: watched_movie.id,
            source_id: None,
            resume_position_ms: Some(90_000),
            duration_ms: Some(100_000),
            watched: true,
            watched_at_ms: Some(300),
            last_played_at_ms: Some(300),
            updated_at_ms: 300,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.principal_id.clone(),
            item_id: in_progress_episode.id,
            source_id: None,
            resume_position_ms: Some(5_000),
            duration_ms: Some(20_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(400),
            updated_at_ms: 400,
        })
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let response = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!(
            "/libraries/{library_id}/items?facet=kind:movie&watch_state=in_progress&sort=last_played&order=desc"
        ),
    )
    .await;

    assert_eq!(response.page.returned, 1);
    assert_eq!(response.items[0].id, in_progress_movie.id.to_string());

    let repeated_facet_response = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!("/libraries/{library_id}/items?facet=&facet=kind:movie&watch_state=in_progress"),
    )
    .await;

    assert_eq!(repeated_facet_response.page.returned, 1);
    assert_eq!(
        repeated_facet_response.items[0].id,
        in_progress_movie.id.to_string()
    );
}
