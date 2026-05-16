use super::*;

#[tokio::test]
async fn webdav_preview_config_builds_scanner_backend() {
    let server = MockWebDavServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
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
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: taru_core::LibraryPreset::Movies,
            webdav: Some(WebDavLibraryConfig {
                root: "webdav:///Movies".to_owned(),
                base_url: server.base_url(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 1,
            }),
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store).await.unwrap();
    let library = default_library_from_config(app.config()).unwrap();
    let backend = app
        .storage_backend_for_library_root(&library)
        .await
        .unwrap();
    let scanner = taru_library::VfsLibraryScanner::new(backend);
    let summary = scanner
        .scan(LibraryScanRequest {
            job_id: JobId::new(),
            library_id,
            root: StorageUri::parse("webdav:///Movies").unwrap(),
            force: false,
        })
        .await
        .unwrap();

    assert_eq!(library.roots, vec!["webdav:///Movies"]);
    assert_eq!(summary.discovered_files, 1);
    assert_eq!(
        summary.media_sources[0].uri.as_str(),
        "webdav:///Movies/Demo.mkv"
    );
}

#[tokio::test]
async fn multi_library_config_registers_libraries_and_resolves_source_backend() {
    let server = MockWebDavServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let local_library_id = LibraryId::new();
    let remote_library_id = LibraryId::new();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(
        TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
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
            libraries: vec![
                LocalLibraryConfig {
                    id: local_library_id,
                    name: "Local Movies".to_owned(),
                    root: temp.path().join("movies"),
                    preset: taru_core::LibraryPreset::Movies,
                    webdav: None,
                },
                LocalLibraryConfig {
                    id: remote_library_id,
                    name: "Remote Movies".to_owned(),
                    root: temp.path().join("unused-local-root"),
                    preset: taru_core::LibraryPreset::Movies,
                    webdav: Some(WebDavLibraryConfig {
                        root: "webdav:///Movies".to_owned(),
                        base_url: server.base_url(),
                        username: None,
                        password_env: None,
                        timeout_ms: 5_000,
                        max_attempts: 1,
                    }),
                },
            ],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let libraries = app.list_libraries(PageRequest::first_page()).await.unwrap();

    assert_eq!(libraries.libraries.len(), 2);
    assert!(
        libraries
            .libraries
            .iter()
            .any(|library| library.id == local_library_id && library.roots == vec!["local:///"])
    );
    assert!(libraries.libraries.iter().any(|library| {
        library.id == remote_library_id && library.roots == vec!["webdav:///Movies"]
    }));

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
        library_id: remote_library_id,
        item_id: item.id,
        locator: "webdav:///Movies/Demo.mkv".to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let plan = app
        .plan_direct_play(source.id, DirectPlayRangeRequest::None)
        .await
        .unwrap();

    assert_eq!(plan.response.total_len, 4);
    let DirectPlaySourceBody::Stream(_) = &plan.body else {
        panic!("expected remote direct play to use the configured WebDAV backend");
    };
}
