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
async fn public_json_browse_routes_use_no_store_cache_policy() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let paths = [
        "/items".to_owned(),
        "/search?q=route&limit=1&offset=0".to_owned(),
        "/libraries?limit=20&offset=0".to_owned(),
        format!("/libraries/{library_id}/sources?limit=20&offset=0"),
        format!("/libraries/{library_id}/items?limit=20&offset=0"),
    ];

    for path in paths {
        let response = response_for(&router, Method::GET, &path).await;

        assert_eq!(response.status(), StatusCode::OK, "path: {path}");
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("no-store")),
            "path: {path}"
        );
    }
}

#[tokio::test]
async fn public_json_browse_routes_reject_limits_above_the_response_budget() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let paths = [
        "/items?limit=501".to_owned(),
        "/search?q=route&limit=501&offset=0".to_owned(),
        "/libraries?limit=501&offset=0".to_owned(),
        format!("/libraries/{library_id}/sources?limit=501&offset=0"),
        format!("/libraries/{library_id}/items?limit=501&offset=0"),
    ];

    for path in paths {
        let response = response_for(&router, Method::GET, &path).await;
        let error = body_json::<nako_api::public_client::ErrorResponse>(response).await;

        assert_eq!(error.code, "invalid_input", "path: {path}");
        assert!(
            error
                .message
                .contains("limit must be less than or equal to"),
            "path: {path}"
        );
    }
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
async fn search_route_supports_repeated_and_comma_separated_facets() {
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
            title: "Facet Route Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let mismatch = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Facet Route Mismatch".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let genre_only = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Facet Route Genre Only".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_item(&mismatch).await.unwrap();
    store.upsert_media_item(&genre_only).await.unwrap();
    store
        .upsert(
            SearchDocument::from_facet_labels(
                item.id,
                item.metadata.title.clone(),
                "A route test fixture",
                vec!["genre:test".to_owned(), "tag:demo".to_owned()],
            )
            .unwrap(),
        )
        .await
        .unwrap();
    store
        .upsert(
            SearchDocument::from_facet_labels(
                mismatch.id,
                mismatch.metadata.title.clone(),
                "A route test fixture",
                vec!["genre:test".to_owned()],
            )
            .unwrap(),
        )
        .await
        .unwrap();
    store
        .upsert(
            SearchDocument::from_facet_labels(
                genre_only.id,
                genre_only.metadata.title.clone(),
                "A route test fixture",
                vec!["tag:demo".to_owned()],
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let router = build_router(app);

    let repeated = request_json::<nako_api::public_client::SearchResponse>(
        &router,
        Method::GET,
        "/search?q=facet&facet=genre:test&facet=tag:demo&limit=12&offset=0",
    )
    .await;
    let comma_separated = request_json::<nako_api::public_client::SearchResponse>(
        &router,
        Method::GET,
        "/search?q=facet&facet=genre:test,tag:demo&limit=12&offset=0",
    )
    .await;

    assert_eq!(repeated.page.returned, 1);
    assert_eq!(repeated.hits[0].item.id, item.id.to_string());
    assert_eq!(comma_separated.page.returned, 1);
    assert_eq!(comma_separated.hits[0].item.id, item.id.to_string());
    assert!(
        repeated
            .hits
            .iter()
            .all(|hit| hit.item.id != mismatch.id.to_string()
                && hit.item.id != genre_only.id.to_string())
    );
    assert!(
        comma_separated
            .hits
            .iter()
            .all(|hit| hit.item.id != mismatch.id.to_string()
                && hit.item.id != genre_only.id.to_string())
    );
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig {
            artifact_root: temp.path().join("nako-cache").join("artwork"),
            ..crate::config::ArtworkConfig::default()
        },
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
async fn catalog_items_route_filters_access_before_pagination() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "A Hidden Catalog Route",
    )
    .await;
    let visible_item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "B Visible Catalog Route",
    )
    .await;

    let response = request_json::<nako_api::public_client::ItemsResponse>(
        &fixture.router,
        Method::GET,
        "/items?limit=1&offset=0",
    )
    .await;

    assert_eq!(response.page.limit, 1);
    assert_eq!(response.page.offset, 0);
    assert_eq!(response.page.returned, 1);
    assert_eq!(response.items[0].id, visible_item.id.to_string());
    assert_ne!(response.items[0].id, hidden_item.id.to_string());
}

#[tokio::test]
async fn catalog_item_detail_credits_and_images_require_browse_access() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "Hidden Catalog Detail",
    )
    .await;

    let detail = response_for(
        &fixture.router,
        Method::GET,
        &format!("/items/{}", hidden_item.id),
    )
    .await;
    let credits = response_for(
        &fixture.router,
        Method::GET,
        &format!("/items/{}/credits", hidden_item.id),
    )
    .await;
    let images = response_for(
        &fixture.router,
        Method::GET,
        &format!("/items/{}/images", hidden_item.id),
    )
    .await;

    assert_eq!(detail.status(), StatusCode::FORBIDDEN);
    assert_eq!(credits.status(), StatusCode::FORBIDDEN);
    assert_eq!(images.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn catalog_selected_artwork_image_route_uses_private_cache_validators() {
    let fixture = catalog_access_route_fixture().await;
    let item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Visible Selected Artwork Cache",
    )
    .await;
    let selected = seed_catalog_route_selected_artwork(
        &fixture.store,
        &fixture.artwork_root,
        fixture.allowed_library_id,
        item.id,
        68,
        "sha256-visible-selected-artwork-cache",
    )
    .await;
    let path = format!("/images/{}", selected.id);

    let get = response_for(&fixture.router, Method::GET, &path).await;
    assert_eq!(get.status(), StatusCode::OK);
    assert_eq!(
        get.headers().get(header::CACHE_CONTROL).unwrap(),
        "private, max-age=86400"
    );
    let etag = get
        .headers()
        .get(header::ETAG)
        .expect("selected artwork response has an ETag")
        .to_str()
        .unwrap()
        .to_owned();
    assert!(etag.contains("nako-img-v1-"));
    to_bytes(get.into_body(), usize::MAX).await.unwrap();

    let not_modified = fixture
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&path)
                .header(header::IF_NONE_MATCH, &etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(not_modified.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(
        not_modified.headers().get(header::CACHE_CONTROL).unwrap(),
        "private, max-age=86400"
    );
    assert_eq!(
        not_modified.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&etag).unwrap()
    );
    let not_modified_body = to_bytes(not_modified.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(not_modified_body.is_empty());

    let head = response_for(&fixture.router, Method::HEAD, &path).await;
    assert_eq!(head.status(), StatusCode::OK);
    assert_eq!(
        head.headers().get(header::CACHE_CONTROL).unwrap(),
        "private, max-age=86400"
    );
    assert_eq!(
        head.headers().get(header::ETAG).unwrap(),
        HeaderValue::from_str(&etag).unwrap()
    );
    let head_body = to_bytes(head.into_body(), usize::MAX).await.unwrap();
    assert!(head_body.is_empty());
}

#[tokio::test]
async fn catalog_item_detail_filters_sources_inside_app_service() {
    let fixture = catalog_access_route_fixture().await;
    let item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Visible Multi Source Detail",
    )
    .await;
    let visible_source = fixture
        .store
        .list_item_sources(item.id, PageRequest::first_page())
        .await
        .unwrap()
        .into_iter()
        .find(|source| source.library_id == fixture.allowed_library_id)
        .unwrap();
    let hidden_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: fixture.blocked_library_id,
        item_id: item.id,
        locator: "local:///Visible Multi Source Detail Hidden.mkv".to_owned(),
        file_name: "Visible Multi Source Detail Hidden.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    fixture
        .store
        .upsert_media_source(&hidden_source)
        .await
        .unwrap();

    let detail = request_json::<nako_api::public_client::ItemDetailResponse>(
        &fixture.router,
        Method::GET,
        &format!("/items/{}", item.id),
    )
    .await;

    assert_eq!(detail.item.id, item.id.to_string());
    assert_eq!(detail.sources.len(), 1);
    assert_eq!(detail.sources[0].id, visible_source.id.to_string());
    assert_ne!(detail.sources[0].id, hidden_source.id.to_string());
}

#[tokio::test]
async fn catalog_source_probe_route_enforces_access_inside_app_service() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "Hidden Source Probe",
    )
    .await;
    let visible_item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Visible Source Probe",
    )
    .await;
    let hidden_source = catalog_route_item_source(&fixture.store, hidden_item.id).await;
    let visible_source = catalog_route_item_source(&fixture.store, visible_item.id).await;
    fixture
        .store
        .upsert_media_probe(visible_source.id, &catalog_route_probe())
        .await
        .unwrap();
    fixture
        .store
        .upsert_media_probe(hidden_source.id, &catalog_route_probe())
        .await
        .unwrap();

    let visible = request_json::<nako_api::public_client::SourceProbeResponse>(
        &fixture.router,
        Method::GET,
        &format!("/sources/{}/probe", visible_source.id),
    )
    .await;
    let hidden = response_for(
        &fixture.router,
        Method::GET,
        &format!("/sources/{}/probe", hidden_source.id),
    )
    .await;

    assert_eq!(visible.source_id, visible_source.id.to_string());
    assert_eq!(visible.probe.duration_ms, Some(90_000));
    assert_eq!(visible.probe.container.as_deref(), Some("matroska,webm"));
    assert_eq!(hidden.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn catalog_relation_item_routes_filter_access_before_pagination() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "A Hidden Catalog Relation",
    )
    .await;
    let visible_item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "B Visible Catalog Relation",
    )
    .await;
    let person = Person {
        id: PersonId::new(),
        name: "Catalog Route Person".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let genre = Genre {
        id: GenreId::new(),
        name: "Catalog Route Genre".to_owned(),
        source: MetadataSource::User,
    };
    let tag = Tag {
        id: TagId::new(),
        name: "catalog-route".to_owned(),
        source: MetadataSource::User,
    };
    fixture.store.upsert_person(&person).await.unwrap();
    fixture.store.upsert_genre(&genre).await.unwrap();
    fixture.store.upsert_tag(&tag).await.unwrap();

    for item_id in [hidden_item.id, visible_item.id] {
        fixture
            .store
            .upsert_item_credit(&ItemCredit {
                item_id,
                person_id: person.id,
                role: CreditRole::Actor,
                character: None,
                sort_order: None,
            })
            .await
            .unwrap();
        fixture
            .store
            .upsert_item_genre(&ItemGenre {
                item_id,
                genre_id: genre.id,
            })
            .await
            .unwrap();
        fixture
            .store
            .upsert_item_tag(&ItemTag {
                item_id,
                tag_id: tag.id,
            })
            .await
            .unwrap();
    }

    let person_items = request_json::<nako_api::public_client::PersonItemsResponse>(
        &fixture.router,
        Method::GET,
        &format!("/people/{}/items?limit=1&offset=0", person.id),
    )
    .await;
    let genre_items = request_json::<nako_api::public_client::GenreItemsResponse>(
        &fixture.router,
        Method::GET,
        &format!("/genres/{}/items?limit=1&offset=0", genre.id),
    )
    .await;
    let tag_items = request_json::<nako_api::public_client::TagItemsResponse>(
        &fixture.router,
        Method::GET,
        &format!("/tags/{}/items?limit=1&offset=0", tag.id),
    )
    .await;

    assert_eq!(person_items.page.returned, 1);
    assert_eq!(person_items.items[0].id, visible_item.id.to_string());
    assert_eq!(genre_items.page.returned, 1);
    assert_eq!(genre_items.items[0].id, visible_item.id.to_string());
    assert_eq!(tag_items.page.returned, 1);
    assert_eq!(tag_items.items[0].id, visible_item.id.to_string());
}

#[tokio::test]
async fn catalog_root_aggregate_routes_filter_access_before_pagination() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "A Hidden Catalog Root Aggregate",
    )
    .await;
    let visible_item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "B Visible Catalog Root Aggregate",
    )
    .await;
    let hidden_person = Person {
        id: PersonId::new(),
        name: "A Hidden Catalog Root Person".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let orphan_person = Person {
        id: PersonId::new(),
        name: "B Orphan Catalog Root Person".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let visible_person = Person {
        id: PersonId::new(),
        name: "C Visible Catalog Root Person".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let hidden_genre = Genre {
        id: GenreId::new(),
        name: "A Hidden Catalog Root Genre".to_owned(),
        source: MetadataSource::User,
    };
    let orphan_genre = Genre {
        id: GenreId::new(),
        name: "B Orphan Catalog Root Genre".to_owned(),
        source: MetadataSource::User,
    };
    let visible_genre = Genre {
        id: GenreId::new(),
        name: "C Visible Catalog Root Genre".to_owned(),
        source: MetadataSource::User,
    };
    let hidden_tag = Tag {
        id: TagId::new(),
        name: "a-hidden-catalog-root-tag".to_owned(),
        source: MetadataSource::User,
    };
    let orphan_tag = Tag {
        id: TagId::new(),
        name: "b-orphan-catalog-root-tag".to_owned(),
        source: MetadataSource::User,
    };
    let visible_tag = Tag {
        id: TagId::new(),
        name: "c-visible-catalog-root-tag".to_owned(),
        source: MetadataSource::User,
    };

    for person in [&hidden_person, &orphan_person, &visible_person] {
        fixture.store.upsert_person(person).await.unwrap();
    }
    for genre in [&hidden_genre, &orphan_genre, &visible_genre] {
        fixture.store.upsert_genre(genre).await.unwrap();
    }
    for tag in [&hidden_tag, &orphan_tag, &visible_tag] {
        fixture.store.upsert_tag(tag).await.unwrap();
    }

    link_catalog_route_root_aggregates(
        &fixture.store,
        hidden_item.id,
        hidden_person.id,
        hidden_genre.id,
        hidden_tag.id,
    )
    .await;
    link_catalog_route_root_aggregates(
        &fixture.store,
        visible_item.id,
        visible_person.id,
        visible_genre.id,
        visible_tag.id,
    )
    .await;

    let people = request_json::<nako_api::public_client::PeopleResponse>(
        &fixture.router,
        Method::GET,
        "/people?limit=1&offset=0",
    )
    .await;
    let genres = request_json::<nako_api::public_client::GenreListResponse>(
        &fixture.router,
        Method::GET,
        "/genres?limit=1&offset=0",
    )
    .await;
    let tags = request_json::<nako_api::public_client::TagsResponse>(
        &fixture.router,
        Method::GET,
        "/tags?limit=1&offset=0",
    )
    .await;

    assert_eq!(people.page.returned, 1);
    assert_eq!(people.people[0].id, visible_person.id.to_string());
    assert_ne!(people.people[0].id, hidden_person.id.to_string());
    assert_ne!(people.people[0].id, orphan_person.id.to_string());
    assert_eq!(genres.page.returned, 1);
    assert_eq!(genres.genres[0].id, visible_genre.id.to_string());
    assert_ne!(genres.genres[0].id, hidden_genre.id.to_string());
    assert_ne!(genres.genres[0].id, orphan_genre.id.to_string());
    assert_eq!(tags.page.returned, 1);
    assert_eq!(tags.tags[0].id, visible_tag.id.to_string());
    assert_ne!(tags.tags[0].id, hidden_tag.id.to_string());
    assert_ne!(tags.tags[0].id, orphan_tag.id.to_string());
}

#[tokio::test]
async fn catalog_person_detail_route_requires_accessible_related_item() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "A Hidden Person Detail",
    )
    .await;
    let visible_item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "B Visible Person Detail",
    )
    .await;
    let hidden_person = Person {
        id: PersonId::new(),
        name: "A Hidden Person Detail".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    let visible_person = Person {
        id: PersonId::new(),
        name: "B Visible Person Detail".to_owned(),
        sort_name: None,
        overview: None,
        external_ids: Vec::new(),
    };
    fixture.store.upsert_person(&hidden_person).await.unwrap();
    fixture.store.upsert_person(&visible_person).await.unwrap();
    fixture
        .store
        .upsert_item_credit(&ItemCredit {
            item_id: hidden_item.id,
            person_id: hidden_person.id,
            role: CreditRole::Actor,
            character: None,
            sort_order: None,
        })
        .await
        .unwrap();
    fixture
        .store
        .upsert_item_credit(&ItemCredit {
            item_id: visible_item.id,
            person_id: visible_person.id,
            role: CreditRole::Actor,
            character: None,
            sort_order: None,
        })
        .await
        .unwrap();

    let hidden_response = response_for(
        &fixture.router,
        Method::GET,
        &format!("/people/{}", hidden_person.id),
    )
    .await;
    let visible_response = request_json::<nako_api::public_client::PersonResponse>(
        &fixture.router,
        Method::GET,
        &format!("/people/{}", visible_person.id),
    )
    .await;

    assert_eq!(hidden_response.status(), StatusCode::FORBIDDEN);
    assert_eq!(visible_response.person.id, visible_person.id.to_string());
}

#[tokio::test]
async fn catalog_search_route_filters_accessible_batch_without_leaking_hidden_hits() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_item = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "Needle Hidden Catalog Search",
    )
    .await;
    let visible_item = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Visible Catalog Search",
    )
    .await;
    fixture
        .store
        .upsert(
            SearchDocument::from_facet_labels(
                hidden_item.id,
                hidden_item.metadata.title.clone(),
                "hidden route fixture",
                Vec::new(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    fixture
        .store
        .upsert(
            SearchDocument::from_facet_labels(
                visible_item.id,
                visible_item.metadata.title.clone(),
                "needle visible route fixture",
                Vec::new(),
            )
            .unwrap(),
        )
        .await
        .unwrap();

    let response = request_json::<nako_api::public_client::SearchResponse>(
        &fixture.router,
        Method::GET,
        "/search?q=needle&limit=1&offset=0",
    )
    .await;

    assert_eq!(response.page.limit, 1);
    assert_eq!(response.page.offset, 0);
    assert_eq!(response.page.returned, 1);
    assert_eq!(response.hits[0].item.id, visible_item.id.to_string());
    assert_ne!(response.hits[0].item.id, hidden_item.id.to_string());
}

#[tokio::test]
async fn catalog_search_route_combines_facets_and_access_filtering_before_pagination() {
    let fixture = catalog_access_route_fixture().await;
    let hidden_match = seed_catalog_route_item(
        &fixture.store,
        fixture.blocked_library_id,
        "Needle Hidden Facet Search",
    )
    .await;
    let visible_match = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Needle Visible Facet Search",
    )
    .await;
    let visible_genre_only = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Needle Visible Genre Only",
    )
    .await;
    let visible_tag_only = seed_catalog_route_item(
        &fixture.store,
        fixture.allowed_library_id,
        "Needle Visible Tag Only",
    )
    .await;
    for (item, facets) in [
        (
            &hidden_match,
            vec!["genre:test".to_owned(), "tag:demo".to_owned()],
        ),
        (
            &visible_match,
            vec!["genre:test".to_owned(), "tag:demo".to_owned()],
        ),
        (&visible_genre_only, vec!["genre:test".to_owned()]),
        (&visible_tag_only, vec!["tag:demo".to_owned()]),
    ] {
        fixture
            .store
            .upsert(
                SearchDocument::from_facet_labels(
                    item.id,
                    item.metadata.title.clone(),
                    "needle route fixture",
                    facets,
                )
                .unwrap(),
            )
            .await
            .unwrap();
    }

    let repeated = request_json::<nako_api::public_client::SearchResponse>(
        &fixture.router,
        Method::GET,
        "/search?q=needle&facet=genre:test&facet=tag:demo&limit=1&offset=0",
    )
    .await;
    let comma_separated = request_json::<nako_api::public_client::SearchResponse>(
        &fixture.router,
        Method::GET,
        "/search?q=needle&facet=genre:test,tag:demo&limit=1&offset=0",
    )
    .await;
    let second_page = request_json::<nako_api::public_client::SearchResponse>(
        &fixture.router,
        Method::GET,
        "/search?q=needle&facet=genre:test,tag:demo&limit=1&offset=1",
    )
    .await;

    for response in [&repeated, &comma_separated] {
        assert_eq!(response.page.limit, 1);
        assert_eq!(response.page.offset, 0);
        assert_eq!(response.page.returned, 1);
        assert_eq!(response.hits[0].item.id, visible_match.id.to_string());
        assert!(
            response
                .hits
                .iter()
                .all(|hit| hit.item.id != hidden_match.id.to_string()
                    && hit.item.id != visible_genre_only.id.to_string()
                    && hit.item.id != visible_tag_only.id.to_string())
        );
    }
    assert_eq!(second_page.page.limit, 1);
    assert_eq!(second_page.page.offset, 1);
    assert_eq!(second_page.page.returned, 0);
    assert!(second_page.hits.is_empty());
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig {
            artifact_root: temp.path().join("nako-cache").join("artwork"),
            ..crate::config::ArtworkConfig::default()
        },
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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

    let csv_facet_response = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!(
            "/libraries/{library_id}/items?facet=kind:movie,,&watch_state=in_progress&sort=last_played&order=desc&limit=1&offset=0"
        ),
    )
    .await;

    assert_eq!(csv_facet_response.page.limit, 1);
    assert_eq!(csv_facet_response.page.offset, 0);
    assert_eq!(csv_facet_response.page.returned, 1);
    assert_eq!(
        csv_facet_response.items[0].id,
        in_progress_movie.id.to_string()
    );

    let impossible_csv_facet_response =
        request_json::<nako_api::public_client::LibraryItemsResponse>(
            &router,
            Method::GET,
            &format!(
                "/libraries/{library_id}/items?facet=kind:movie,kind:episode&watch_state=in_progress"
            ),
        )
        .await;

    assert_eq!(impossible_csv_facet_response.page.returned, 0);
    assert!(impossible_csv_facet_response.items.is_empty());
}

#[tokio::test]
async fn library_items_route_uses_stable_sort_keys_with_pagination() {
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig {
            artifact_root: temp.path().join("nako-cache").join("artwork"),
            ..crate::config::ArtworkConfig::default()
        },
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
    let alpha = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Alpha Sort".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let beta = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Beta Sort".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    for item in [&alpha, &beta] {
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
            item_id: alpha.id,
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
            item_id: beta.id,
            source_id: None,
            resume_position_ms: Some(10_000),
            duration_ms: Some(100_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(300),
            updated_at_ms: 300,
        })
        .await
        .unwrap();
    let router = public_client_router_with_principal(app, principal);

    let title_asc = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!("/libraries/{library_id}/items?sort=title&order=asc&limit=1&offset=0"),
    )
    .await;
    let title_asc_page_2 = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!("/libraries/{library_id}/items?sort=title&order=asc&limit=1&offset=1"),
    )
    .await;
    let last_played_desc = request_json::<nako_api::public_client::LibraryItemsResponse>(
        &router,
        Method::GET,
        &format!("/libraries/{library_id}/items?sort=last_played&order=desc&limit=1&offset=0"),
    )
    .await;

    assert_eq!(title_asc.page.limit, 1);
    assert_eq!(title_asc.page.offset, 0);
    assert_eq!(title_asc.page.returned, 1);
    assert_eq!(title_asc.items[0].id, alpha.id.to_string());
    assert_eq!(title_asc_page_2.page.limit, 1);
    assert_eq!(title_asc_page_2.page.offset, 1);
    assert_eq!(title_asc_page_2.page.returned, 1);
    assert_eq!(title_asc_page_2.items[0].id, beta.id.to_string());
    assert_eq!(last_played_desc.page.limit, 1);
    assert_eq!(last_played_desc.page.offset, 0);
    assert_eq!(last_played_desc.page.returned, 1);
    assert_eq!(last_played_desc.items[0].id, beta.id.to_string());
}

struct CatalogAccessRouteFixture {
    _temp: tempfile::TempDir,
    artwork_root: std::path::PathBuf,
    router: Router,
    store: NakoDatabase,
    allowed_library_id: LibraryId,
    blocked_library_id: LibraryId,
}

async fn catalog_access_route_fixture() -> CatalogAccessRouteFixture {
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
    let artwork_root = app.config().artwork.artifact_root.clone();
    let principal =
        local_viewer_with_library_access(&store, allowed_library_id, LibraryAccessLevel::Browse)
            .await;
    let router = public_client_router_with_principal(app, principal);

    CatalogAccessRouteFixture {
        _temp: temp,
        artwork_root,
        router,
        store,
        allowed_library_id,
        blocked_library_id,
    }
}

async fn seed_catalog_route_item(
    store: &NakoDatabase,
    library_id: LibraryId,
    title: &str,
) -> MediaItem {
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
        locator: format!("local:///{title}.mkv"),
        file_name: format!("{title}.mkv"),
        size_bytes: Some(5),
        fingerprint: None,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    item
}

async fn catalog_route_item_source(store: &NakoDatabase, item_id: MediaItemId) -> MediaSource {
    store
        .list_item_sources(item_id, PageRequest::first_page())
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

async fn seed_catalog_route_selected_artwork(
    store: &NakoDatabase,
    artwork_root: &std::path::Path,
    library_id: LibraryId,
    item_id: MediaItemId,
    byte_len: u64,
    content_hash: &str,
) -> nako_core::SelectedArtworkRecord {
    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "catalog.artwork.cache".to_owned(),
            name: "Catalog Artwork Cache".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            outbound_task_dispatch_secret_env: None,
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
            label: "catalog-artwork".to_owned(),
            token_prefix: "nako_at_catalog".to_owned(),
            token_hash: "sha256:catalog-artwork".to_owned(),
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
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: format!("catalog-artwork-cache-{item_id}"),
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
            item_id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: "https://cdn.example.test/poster.png".to_owned(),
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
                priority: JobPriority::Normal,
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
        .expect("expected queued managed artwork ingest");
    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            accepted.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some(content_hash.to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(byte_len),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();
    let shard = artifact_id.to_string()[0..2].to_owned();
    let artifact_path = artwork_root.join(shard).join(format!("{artifact_id}.png"));
    fs::create_dir_all(artifact_path.parent().unwrap()).unwrap();
    fs::write(artifact_path, catalog_route_png()).unwrap();

    store
        .publish_selected_artwork(artifact_id)
        .await
        .unwrap()
        .selected_artwork
}

fn catalog_route_png() -> Vec<u8> {
    let image = image::RgbaImage::from_fn(1, 1, |_, _| image::Rgba([31, 119, 180, 255]));
    let mut cursor = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    cursor.into_inner()
}

fn catalog_route_probe() -> MediaProbeResult {
    MediaProbeResult {
        duration_ms: Some(90_000),
        container: Some("matroska,webm".to_owned()),
        bit_rate: Some(8_000_000),
        streams: vec![MediaStreamInfo {
            index: 0,
            kind: MediaStreamKind::Video,
            codec: Some("h264".to_owned()),
            language: None,
            duration_ms: Some(90_000),
            bit_rate: Some(8_000_000),
            width: Some(1920),
            height: Some(1080),
            channels: None,
            sample_rate: None,
            technical: Default::default(),
        }],
    }
}

async fn link_catalog_route_root_aggregates(
    store: &NakoDatabase,
    item_id: MediaItemId,
    person_id: PersonId,
    genre_id: GenreId,
    tag_id: TagId,
) {
    store
        .upsert_item_credit(&ItemCredit {
            item_id,
            person_id,
            role: CreditRole::Actor,
            character: None,
            sort_order: None,
        })
        .await
        .unwrap();
    store
        .upsert_item_genre(&ItemGenre { item_id, genre_id })
        .await
        .unwrap();
    store
        .upsert_item_tag(&ItemTag { item_id, tag_id })
        .await
        .unwrap();
}
