use super::*;

async fn wait_for_runtime_jobs(
    app: &TaruApp,
    succeeded_jobs: u64,
    cancelled_jobs: u64,
    failed_jobs: u64,
) -> RuntimeSupervisorDiagnostics {
    for _ in 0..100 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == succeeded_jobs
            && diagnostics.cancelled_jobs == cancelled_jobs
            && diagnostics.failed_jobs == failed_jobs
        {
            return diagnostics;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "runtime jobs did not reach expected counts: {:?}",
        app.runtime_diagnostics()
    );
}

#[tokio::test]
async fn nfo_import_uses_configured_webdav_backend() {
    let server = MockWebDavServer::start().await;
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
        remux_staging_root: temp.path().join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
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
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Original".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "webdav:///Movies/Demo.mkv".to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let output = app.nfo().import_library_nfo(library_id).await.unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();

    assert_eq!(output.import.imported_items, 1);
    assert_eq!(loaded.metadata.title, "Remote NFO");
}

#[tokio::test]
async fn nfo_import_job_acknowledges_cancellation_before_next_sidecar() {
    let server = BlockingNfoWebDavServer::start(BlockingNfoWebDavControl::new()).await;
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
        remux_staging_root: temp.path().join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
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
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let first = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "First Original".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let second = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Second Original".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let first_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: first.id,
        locator: "webdav:///Movies/First.mkv".to_owned(),
        file_name: "First.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    let second_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: second.id,
        locator: "webdav:///Movies/Second.mkv".to_owned(),
        file_name: "Second.mkv".to_owned(),
        size_bytes: Some(4),
        fingerprint: None,
    };
    store.upsert_media_item(&first).await.unwrap();
    store.upsert_media_item(&second).await.unwrap();
    store.upsert_media_source(&first_source).await.unwrap();
    store.upsert_media_source(&second_source).await.unwrap();

    let job = app.nfo().enqueue_nfo_import(library_id).await.unwrap();
    tokio::time::timeout(
        Duration::from_secs(5),
        server.control().wait_for_first_get(),
    )
    .await
    .unwrap();

    let requested = app.jobs().request_job_cancellation(job.id).await.unwrap();
    assert!(requested.requested);
    assert!(!requested.terminal);
    assert_eq!(requested.job.status, JobStatus::Running);

    server.control().release_first_get();
    let diagnostics = wait_for_runtime_jobs(&app, 0, 1, 0).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();
    let first_loaded = store.get_media_item(first.id).await.unwrap().unwrap();
    let second_loaded = store.get_media_item(second.id).await.unwrap().unwrap();
    let changed = [&first_loaded, &second_loaded]
        .into_iter()
        .filter(|item| item.metadata.title.ends_with("Remote NFO"))
        .count();
    let events = store
        .list_outbox_events(Default::default(), PageRequest::new(100, 0))
        .await
        .unwrap();

    assert_eq!(diagnostics.failed_jobs, 0);
    assert_eq!(diagnostics.cancelled_jobs, 1);
    assert_eq!(persisted.status, JobStatus::Cancelled);
    assert_eq!(persisted.summary_json, None);
    assert_eq!(persisted.error, None);
    assert_eq!(server.control().nfo_gets(), 1);
    assert_eq!(changed, 1);
    assert!(!events.iter().any(|event| {
        event.kind == DomainEventKind::NfoImported
            && event.subject == DomainEventSubject::Library(library_id)
    }));
}

#[tokio::test]
async fn nfo_export_job_acknowledges_cancellation_before_next_sidecar() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("Movies")).unwrap();
    fs::write(temp.path().join("Movies").join("First.mkv"), b"media").unwrap();
    fs::write(temp.path().join("Movies").join("Second.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Movies".to_owned(),
        root: temp.path().to_path_buf(),
        preset: taru_core::LibraryPreset::Movies,
        webdav: None,
    };
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
        remux_staging_root: temp.path().join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![library_config.clone()],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Movies);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::WriteSidecar;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options,
        })
        .await
        .unwrap();
    let first = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "First Export".to_owned(),
            overview: Some("First overview".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let second = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Second Export".to_owned(),
            overview: Some("Second overview".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let first_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: first.id,
        locator: "local:///Movies/First.mkv".to_owned(),
        file_name: "First.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    let second_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: second.id,
        locator: "local:///Movies/Second.mkv".to_owned(),
        file_name: "Second.mkv".to_owned(),
        size_bytes: Some(6),
        fingerprint: None,
    };
    store.upsert_media_item(&first).await.unwrap();
    store.upsert_media_item(&second).await.unwrap();
    store.upsert_media_source(&first_source).await.unwrap();
    store.upsert_media_source(&second_source).await.unwrap();
    let control = BlockingNfoExportControl::new();
    let backend = BlockingNfoExportBackend::new(temp.path(), control.clone()).unwrap();
    app.storage()
        .replace_backend_for_test(library_config, Arc::new(backend))
        .await;

    let job = app.nfo().enqueue_nfo_export(library_id).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), control.wait_for_first_write())
        .await
        .unwrap();

    let requested = app.jobs().request_job_cancellation(job.id).await.unwrap();
    assert!(requested.requested);
    assert!(!requested.terminal);
    assert_eq!(requested.job.status, JobStatus::Running);

    control.release_first_write();
    let diagnostics = wait_for_runtime_jobs(&app, 0, 1, 0).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();
    let first_exists = temp.path().join("Movies").join("First.nfo").exists();
    let second_exists = temp.path().join("Movies").join("Second.nfo").exists();
    let events = store
        .list_outbox_events(Default::default(), PageRequest::new(100, 0))
        .await
        .unwrap();

    assert_eq!(diagnostics.failed_jobs, 0);
    assert_eq!(diagnostics.cancelled_jobs, 1);
    assert_eq!(persisted.status, JobStatus::Cancelled);
    assert_eq!(persisted.summary_json, None);
    assert_eq!(persisted.error, None);
    assert_eq!(control.nfo_writes(), 1);
    assert_eq!(
        [first_exists, second_exists]
            .into_iter()
            .filter(|exists| *exists)
            .count(),
        1
    );
    assert!(!events.iter().any(|event| {
        event.kind == DomainEventKind::NfoExported
            && event.subject == DomainEventSubject::Library(library_id)
    }));
}

#[tokio::test]
async fn nfo_import_uses_reconciled_library_policy() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie><title>NFO Title</title></movie>"#,
    )
    .unwrap();
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
        remux_staging_root: temp.path().join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Configured Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Movies);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::Disabled;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Persisted Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options,
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Original".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let err = app.nfo().import_library_nfo(library_id).await.unwrap_err();

    assert_eq!(
        err,
        TaruError::Unsupported(
            "NFO import requires read-only, local-first, or remote-first local metadata policy",
        )
    );
}

#[tokio::test]
async fn nfo_export_rejects_read_only_webdav_backend() {
    let server = MockWebDavServer::start().await;
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
        remux_staging_root: temp.path().join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
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
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store).await.unwrap();

    let err = app.nfo().export_library_nfo(library_id).await.unwrap_err();

    assert_eq!(
        err,
        TaruError::Unsupported("NFO export requires a writable storage backend")
    );
}

#[tokio::test]
async fn nfo_import_job_imports_sidecar_and_persists_summary() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
</movie>
"#,
    )
    .unwrap();
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
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "File Title".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let output = app.nfo().import_library_nfo(library_id).await.unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();
    let job = app.jobs().get_job(output.job.id).await.unwrap();
    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(output.job.kind, JobKind::NfoImport);
    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(output.import.imported_items, 1);
    assert_eq!(loaded.metadata.title, "NFO Title");
    assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
    assert!(locks.iter().any(|lock| {
        lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
    }));
    assert_eq!(job.status, JobStatus::Succeeded);
    assert!(job.summary_json.unwrap().contains("\"imported_items\":1"));
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::NfoImported
            && event.subject == DomainEventSubject::Library(library_id)
            && !event.payload_json.contains("demo.nfo")
            && !event
                .payload_json
                .contains(&temp.path().display().to_string())
    }));
}
