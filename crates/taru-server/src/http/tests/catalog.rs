use super::*;

#[tokio::test]
async fn empty_sources_and_items_routes_work() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let router = test_router(temp.path().to_path_buf(), library_id).await;
    let sources_path = format!("/libraries/{library_id}/sources");

    let sources =
        request_json::<taru_api::LibrarySourcesResponse>(&router, Method::GET, &sources_path).await;
    let items = request_json::<taru_api::ItemsResponse>(&router, Method::GET, "/items").await;

    assert_eq!(sources.library.id, library_id.to_string());
    assert_eq!(sources.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(sources.page.offset, 0);
    assert_eq!(sources.page.returned, 0);
    assert!(sources.sources.is_empty());
    assert_eq!(items.page.limit, taru_core::PageRequest::DEFAULT_LIMIT);
    assert_eq!(items.page.offset, 0);
    assert_eq!(items.page.returned, 0);
    assert!(items.items.is_empty());
}

#[tokio::test]
async fn search_route_returns_indexed_items() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        auth: crate::config::AuthConfig::disabled(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("taru-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: taru_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Search Route Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert(SearchDocument {
            item_id: item.id,
            title: item.metadata.title.clone(),
            body: "A route test fixture".to_owned(),
            facets: vec!["genre:test".to_owned()],
        })
        .await
        .unwrap();
    let router = build_router(app);

    let result = request_json::<taru_api::SearchResponse>(
        &router,
        Method::GET,
        "/search?q=route&facet=genre:test",
    )
    .await;

    assert_eq!(result.page.returned, 1);
    assert_eq!(result.hits[0].item.id, item.id.to_string());
}

#[tokio::test]
async fn browse_routes_return_catalog_graph() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        auth: crate::config::AuthConfig::disabled(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("taru-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: taru_core::MediaItemId::new(),
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
        provider: taru_core::ExternalProvider::Local,
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

    let detail = request_json::<taru_api::ItemDetailResponse>(
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
    let credits = request_json::<taru_api::ItemCreditsResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/credits", item.id),
    )
    .await;
    let images = request_json::<taru_api::ImagesResponse>(
        &router,
        Method::GET,
        &format!("/items/{}/images", item.id),
    )
    .await;
    let people = request_json::<taru_api::PeopleResponse>(&router, Method::GET, "/people").await;
    let person_items = request_json::<taru_api::PersonItemsResponse>(
        &router,
        Method::GET,
        &format!("/people/{}/items", person.id),
    )
    .await;
    let tags = request_json::<taru_api::TagsResponse>(&router, Method::GET, "/tags").await;
    let tag_items = request_json::<taru_api::TagItemsResponse>(
        &router,
        Method::GET,
        &format!("/tags/{}/items", tag.id),
    )
    .await;
    let genres = request_json::<taru_api::GenreListResponse>(&router, Method::GET, "/genres").await;
    let genre_items = request_json::<taru_api::GenreItemsResponse>(
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
    assert_eq!(images.images[0].id, image.id.to_string());
    assert_eq!(people.people[0].id, person.id.to_string());
    assert_eq!(person_items.items[0].id, item.id.to_string());
    assert_eq!(tags.tags[0].name, "favorite");
    assert_eq!(tag_items.items[0].id, item.id.to_string());
    assert_eq!(genres.genres[0].name, "Science Fiction");
    assert_eq!(genre_items.items[0].id, item.id.to_string());
}
