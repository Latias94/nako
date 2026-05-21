use super::*;
use crate::app::nfo::{
    AcceptNfoSidecarApplyRequest, ApplyNfoSidecarApplyRequest, NfoSidecarApplyAuditFailurePoint,
};
use taru_core::{
    LibraryItemRepository, LibraryItemState, NfoSidecarApplyOperationKind,
    NfoSidecarApplyRepository, NfoSidecarApplyState, UserPrincipalId,
};
use taru_nfo::{
    NfoAuthorityPreviewAction, NfoAuthorityPreviewOperation, NfoAuthorityPreviewReason,
    NfoAuthorityPreviewSummary,
};

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
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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

#[tokio::test]
async fn nfo_authority_preview_explains_export_create_skip_update_and_policy_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("fresh.mkv"), b"media").unwrap();
    fs::write(temp.path().join("existing.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("existing.nfo"),
        r#"<movie><title>Existing Sidecar</title><custom>keep</custom></movie>"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let fresh = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Fresh Export".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let existing = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Existing Export".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let fresh_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: fresh.id,
        locator: "local:///fresh.mkv".to_owned(),
        file_name: "fresh.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    let existing_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: existing.id,
        locator: "local:///existing.mkv".to_owned(),
        file_name: "existing.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&fresh).await.unwrap();
    store.upsert_media_item(&existing).await.unwrap();
    store.upsert_media_source(&fresh_source).await.unwrap();
    store.upsert_media_source(&existing_source).await.unwrap();

    let create_and_skip = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    let forced = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, true)
        .await
        .unwrap();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions::from_preset(taru_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let rejected = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();

    assert_eq!(create_and_skip.scanned_sources, 2);
    assert_eq!(create_and_skip.create_items, 1);
    assert_eq!(create_and_skip.skip_items, 1);
    assert_eq!(create_and_skip.backup_required_items, 0);
    assert!(
        create_and_skip
            .decisions
            .iter()
            .any(|decision| decision.source_id == fresh_source.id
                && decision.action == NfoAuthorityPreviewAction::Create
                && decision.reason == NfoAuthorityPreviewReason::ExportWouldCreateSidecar)
    );
    assert!(
        create_and_skip
            .decisions
            .iter()
            .any(|decision| decision.source_id == existing_source.id
                && decision.action == NfoAuthorityPreviewAction::Skip
                && decision.reason == NfoAuthorityPreviewReason::ExportWouldSkipExistingSidecar)
    );
    assert_eq!(forced.create_items, 1);
    assert_eq!(forced.update_items, 1);
    assert_eq!(forced.backup_required_items, 1);
    assert!(
        forced
            .decisions
            .iter()
            .any(|decision| decision.source_id == existing_source.id
                && decision.action == NfoAuthorityPreviewAction::Update
                && decision.backup_required
                && decision.reason == NfoAuthorityPreviewReason::ExportWouldUpdateExistingSidecar)
    );
    assert_eq!(rejected.policy_rejected_items, 2);
    assert!(rejected.decisions.iter().all(|decision| {
        decision.action == NfoAuthorityPreviewAction::PolicyRejected
            && decision.reason == NfoAuthorityPreviewReason::PolicyDoesNotAllowOperation
    }));
    assert!(!temp.path().join("fresh.nfo").exists());
    assert_eq!(
        fs::read_to_string(temp.path().join("existing.nfo")).unwrap(),
        r#"<movie><title>Existing Sidecar</title><custom>keep</custom></movie>"#
    );
}

#[tokio::test]
async fn nfo_authority_preview_explains_import_without_mutating_metadata() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie><title>NFO Preview Title</title></movie>"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
            title: "Original Title".to_owned(),
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

    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Import, false)
        .await
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();

    assert_eq!(preview.scanned_sources, 1);
    assert_eq!(preview.update_items, 1);
    assert_eq!(preview.backup_required_items, 0);
    assert_eq!(
        preview.decisions[0].action,
        NfoAuthorityPreviewAction::Update
    );
    assert_eq!(
        preview.decisions[0].reason,
        NfoAuthorityPreviewReason::ImportWouldReadSidecar
    );
    assert_eq!(loaded.metadata.title, "Original Title");
    assert!(locks.is_empty());
}

#[tokio::test]
async fn nfo_sidecar_apply_accepts_current_preview_with_idempotent_replay_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("fresh.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Fresh Export".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///fresh.mkv".to_owned(),
        file_name: "fresh.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    let mut request =
        accept_current_export_preview_request(&preview, item.id, source.id, "nfo-sidecar-accept-1");

    let accepted = app
        .nfo()
        .accept_sidecar_apply(request.clone())
        .await
        .unwrap();
    let replayed = app
        .nfo()
        .accept_sidecar_apply(request.clone())
        .await
        .unwrap();
    request.sidecar_locator = "local:///other.nfo".to_owned();
    let mismatch = app.nfo().accept_sidecar_apply(request).await.unwrap_err();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();

    assert_eq!(accepted.id, replayed.id);
    assert!(replayed.replayed);
    assert_eq!(accepted.state, NfoSidecarApplyState::Accepted);
    assert_eq!(
        accepted.operation_kind,
        NfoSidecarApplyOperationKind::ExportSidecar
    );
    assert_eq!(accepted.target_library_id, library_id);
    assert_eq!(accepted.media_item_id, item.id);
    assert_eq!(accepted.media_source_id, Some(source.id));
    assert_eq!(accepted.sidecar_scheme.as_deref(), Some("local"));
    assert_eq!(
        accepted.sidecar_locator.as_deref(),
        Some("local:///fresh.nfo")
    );
    assert!(accepted.accepted_preview_snapshot);
    assert!(!accepted.accepted_warnings_snapshot);
    assert!(accepted.has_outcome);
    assert!(!accepted.has_raw_storage_path);
    assert!(matches!(
        mismatch,
        TaruError::Conflict { message }
            if message.contains("idempotency key was already used")
    ));
    assert_eq!(stored.state, NfoSidecarApplyState::Accepted);
    assert_eq!(stored.accepted_warnings_json, None);
    assert!(
        stored
            .accepted_preview_json
            .contains(r#""operation":"export""#)
    );
    assert!(
        stored
            .accepted_preview_json
            .contains(r#""action":"create""#)
    );
    assert!(
        !stored
            .accepted_preview_json
            .contains(&temp.path().display().to_string())
    );
    assert!(!temp.path().join("fresh.nfo").exists());
    assert_eq!(loaded.metadata.title, "Fresh Export");
    assert!(locks.is_empty());
}

#[tokio::test]
async fn nfo_sidecar_apply_rejects_stale_preview_without_persistence_or_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    let sidecar_path = temp.path().join("demo.nfo");
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Original Title".to_owned(),
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
    let stale_preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    fs::write(
        &sidecar_path,
        r#"<movie><title>Already Created</title></movie>"#,
    )
    .unwrap();

    let err = app
        .nfo()
        .accept_sidecar_apply(accept_current_export_preview_request(
            &stale_preview,
            item.id,
            source.id,
            "nfo-sidecar-stale-1",
        ))
        .await
        .unwrap_err();
    let attempts = store
        .list_nfo_sidecar_applies_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();

    assert!(matches!(
        err,
        TaruError::Conflict { message } if message.contains("preview is stale")
    ));
    assert!(attempts.is_empty());
    assert_eq!(
        fs::read_to_string(sidecar_path).unwrap(),
        r#"<movie><title>Already Created</title></movie>"#
    );
    assert_eq!(loaded.metadata.title, "Original Title");
    assert!(locks.is_empty());
}

#[tokio::test]
async fn nfo_sidecar_apply_exports_accepted_create_preview_and_commits_audit() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Applied Export".to_owned(),
            overview: Some("Applied overview".to_owned()),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_export_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-export-apply-1",
        ))
        .await
        .unwrap();

    let applied = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let replayed = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();
    let sidecar_xml = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    let outcome = stored.outcome_json.as_deref().unwrap_or_default();

    assert_eq!(applied.state, NfoSidecarApplyState::Committed);
    assert_eq!(applied.id, replayed.id);
    assert!(replayed.replayed);
    assert_eq!(stored.state, NfoSidecarApplyState::Committed);
    assert!(sidecar_xml.contains("<title>Applied Export</title>"));
    assert!(sidecar_xml.contains("<plot>Applied overview</plot>"));
    assert_eq!(loaded.metadata.title, "Applied Export");
    assert!(locks.is_empty());
    assert!(outcome.contains(r#""committed":true"#));
    assert!(outcome.contains(r#""storage_mutation":true"#));
    assert!(!outcome.contains(&temp.path().display().to_string()));
    assert!(!outcome.contains("<movie"));
}

#[tokio::test]
async fn nfo_sidecar_apply_export_audit_failure_records_repair_pending() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Repair Pending Export".to_owned(),
            overview: Some("Sidecar write succeeds before audit failure".to_owned()),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_export_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-export-audit-failure-1",
        ))
        .await
        .unwrap();
    let service = app
        .nfo()
        .with_audit_failure_for_test(NfoSidecarApplyAuditFailurePoint::BeforeCommittedState);

    let err = service
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();
    let repair_pending = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let replayed = service
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let outcome: serde_json::Value =
        serde_json::from_str(repair_pending.outcome_json.as_deref().unwrap()).unwrap();
    let sidecar_xml = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();

    assert!(
        err.to_string()
            .contains("nfo_sidecar_apply_audit_commit_failed")
    );
    assert_eq!(repair_pending.state, NfoSidecarApplyState::RepairPending);
    assert_eq!(
        repair_pending.safe_error_code.as_deref(),
        Some("nfo_sidecar_apply_audit_commit_failed")
    );
    assert_eq!(replayed.state, NfoSidecarApplyState::RepairPending);
    assert!(replayed.replayed);
    assert!(sidecar_xml.contains("<title>Repair Pending Export</title>"));
    assert_eq!(outcome["committed"], false);
    assert_eq!(outcome["writes_library"], true);
    assert_eq!(outcome["storage_mutation"], true);
    assert_eq!(outcome["metadata_mutation"], false);
    assert_eq!(outcome["repair_required"], true);
    assert_eq!(outcome["audit_commit_completed"], false);
    assert!(
        !repair_pending
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains(&temp.path().display().to_string())
    );
    assert!(
        !repair_pending
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains("<movie")
    );
}

#[tokio::test]
async fn nfo_sidecar_apply_export_write_failure_records_failed_before_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Movies".to_owned(),
        root: temp.path().to_path_buf(),
        preset: taru_core::LibraryPreset::Movies,
        webdav: None,
    };
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
        libraries: vec![library_config.clone()],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(FailingNfoWriteBackend::new(temp.path()).unwrap()),
        )
        .await;
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Write Failure Export".to_owned(),
            overview: Some("Write fails before sidecar mutation".to_owned()),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_export_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-export-write-failure-1",
        ))
        .await
        .unwrap();

    let err = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let outcome: serde_json::Value =
        serde_json::from_str(stored.outcome_json.as_deref().unwrap()).unwrap();

    assert!(err.to_string().contains("nfo_sidecar_export_not_committed"));
    assert_eq!(stored.state, NfoSidecarApplyState::FailedBeforeMutation);
    assert_eq!(
        stored.safe_error_code.as_deref(),
        Some("nfo_sidecar_export_not_committed")
    );
    assert!(!temp.path().join("demo.nfo").exists());
    assert_eq!(outcome["committed"], false);
    assert_eq!(outcome["writes_library"], false);
    assert_eq!(outcome["storage_mutation"], false);
    assert_eq!(outcome["metadata_mutation"], false);
    assert_eq!(outcome["failed_items"], 1);
    assert_eq!(outcome["failure_count"], 1);
    assert!(
        !stored
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains(&temp.path().display().to_string())
    );
    assert!(
        !stored
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains("<movie")
    );
}

#[tokio::test]
async fn nfo_sidecar_apply_rejects_stale_export_apply_before_overwrite() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    let sidecar_path = temp.path().join("demo.nfo");
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Accepted Before Drift".to_owned(),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_export_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-export-stale-apply-1",
        ))
        .await
        .unwrap();
    fs::write(
        &sidecar_path,
        r#"<movie><title>Operator Created</title></movie>"#,
    )
    .unwrap();

    let err = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();

    assert!(matches!(
        err,
        TaruError::Conflict { message } if message.contains("preview is stale")
    ));
    assert_eq!(stored.state, NfoSidecarApplyState::FailedBeforeMutation);
    assert_eq!(
        stored.safe_error_code.as_deref(),
        Some("nfo_sidecar_apply_preview_stale")
    );
    assert_eq!(
        fs::read_to_string(sidecar_path).unwrap(),
        r#"<movie><title>Operator Created</title></movie>"#
    );
}

#[tokio::test]
async fn nfo_sidecar_apply_exports_forced_update_with_backup_and_retention_diagnostics() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie><title>Old Sidecar</title><custom>keep</custom></movie>"#,
    )
    .unwrap();
    fs::write(temp.path().join("demo.nfo.taru-backup-0000"), "old backup").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Updated Sidecar".to_owned(),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Export, true)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_export_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-export-forced-update-1",
        ))
        .await
        .unwrap();

    let applied = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let sidecar_xml = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    let backup_files = fs::read_dir(temp.path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| name.contains(".taru-backup-"))
        .collect::<Vec<_>>();
    let outcome = stored.outcome_json.as_deref().unwrap_or_default();

    assert_eq!(applied.state, NfoSidecarApplyState::Committed);
    assert!(sidecar_xml.contains("<title>Updated Sidecar</title>"));
    assert!(sidecar_xml.contains("<custom>keep</custom>"));
    assert!(
        backup_files
            .iter()
            .any(|name| name.starts_with("demo.nfo.taru-backup-"))
    );
    assert!(outcome.contains(r#""backed_up_items":1"#));
    assert!(outcome.contains(r#""backup_count":1"#));
    assert!(outcome.contains(r#""pruned_backups":"#));
    assert!(!outcome.contains(&temp.path().display().to_string()));
    assert!(!outcome.contains("<movie"));
}

#[tokio::test]
async fn nfo_sidecar_apply_imports_accepted_sidecar_into_metadata_and_locks() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>Imported NFO Title</title>
  <plot>Imported overview</plot>
</movie>
"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Movies);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::LocalFirst;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
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
            title: "Original Title".to_owned(),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Import, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_import_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-import-apply-1",
        ))
        .await
        .unwrap();

    let applied = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let replayed = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();
    let sidecar_xml = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    let outcome = stored.outcome_json.as_deref().unwrap_or_default();

    assert_eq!(applied.state, NfoSidecarApplyState::Committed);
    assert_eq!(applied.id, replayed.id);
    assert!(replayed.replayed);
    assert_eq!(loaded.metadata.title, "Imported NFO Title");
    assert_eq!(
        loaded.metadata.overview,
        Some("Imported overview".to_owned())
    );
    assert!(locks.iter().any(|lock| {
        lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
    }));
    assert_eq!(
        sidecar_xml,
        r#"<movie>
  <title>Imported NFO Title</title>
  <plot>Imported overview</plot>
</movie>
"#
    );
    assert!(outcome.contains(r#""metadata_mutation":true"#));
    assert!(outcome.contains(r#""storage_mutation":false"#));
    assert!(outcome.contains(r#""imported_items":1"#));
    assert!(!outcome.contains(&temp.path().display().to_string()));
    assert!(!outcome.contains("<movie"));
}

#[tokio::test]
async fn nfo_sidecar_apply_import_audit_failure_records_repair_pending() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>Repair Pending Import</title>
  <plot>Metadata mutation succeeds before audit failure</plot>
</movie>
"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Movies);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::LocalFirst;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
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
            title: "Original Title".to_owned(),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Import, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_import_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-import-audit-failure-1",
        ))
        .await
        .unwrap();
    let service = app
        .nfo()
        .with_audit_failure_for_test(NfoSidecarApplyAuditFailurePoint::BeforeCommittedState);

    let err = service
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();
    let repair_pending = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let replayed = service
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();
    let outcome: serde_json::Value =
        serde_json::from_str(repair_pending.outcome_json.as_deref().unwrap()).unwrap();

    assert!(
        err.to_string()
            .contains("nfo_sidecar_apply_audit_commit_failed")
    );
    assert_eq!(repair_pending.state, NfoSidecarApplyState::RepairPending);
    assert_eq!(
        repair_pending.safe_error_code.as_deref(),
        Some("nfo_sidecar_apply_audit_commit_failed")
    );
    assert_eq!(replayed.state, NfoSidecarApplyState::RepairPending);
    assert!(replayed.replayed);
    assert_eq!(loaded.metadata.title, "Repair Pending Import");
    assert_eq!(
        loaded.metadata.overview,
        Some("Metadata mutation succeeds before audit failure".to_owned())
    );
    assert!(locks.iter().any(|lock| {
        lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
    }));
    assert_eq!(outcome["committed"], false);
    assert_eq!(outcome["writes_library"], true);
    assert_eq!(outcome["storage_mutation"], false);
    assert_eq!(outcome["metadata_mutation"], true);
    assert_eq!(outcome["repair_required"], true);
    assert_eq!(outcome["audit_commit_completed"], false);
    assert!(
        !repair_pending
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains(&temp.path().display().to_string())
    );
    assert!(
        !repair_pending
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains("<movie")
    );
}

#[tokio::test]
async fn nfo_sidecar_apply_import_metadata_commit_failure_records_failed_before_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>Rejected Metadata Commit</title>
  <plot>Metadata commit failure should not mutate canonical state</plot>
</movie>
"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Movies);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::LocalFirst;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
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
            title: "Original Title".to_owned(),
            overview: Some("Original overview".to_owned()),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Import, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_import_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-import-metadata-failure-1",
        ))
        .await
        .unwrap();
    let service = app
        .nfo()
        .with_audit_failure_for_test(NfoSidecarApplyAuditFailurePoint::BeforeMetadataCommit);

    let err = service
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();
    let outcome: serde_json::Value =
        serde_json::from_str(stored.outcome_json.as_deref().unwrap()).unwrap();

    assert!(
        err.to_string()
            .contains("injected NFO sidecar metadata commit failure")
    );
    assert_eq!(stored.state, NfoSidecarApplyState::FailedBeforeMutation);
    assert_eq!(
        stored.safe_error_code.as_deref(),
        Some("nfo_sidecar_import_metadata_commit_failed")
    );
    assert_eq!(loaded.metadata.title, "Original Title");
    assert_eq!(
        loaded.metadata.overview,
        Some("Original overview".to_owned())
    );
    assert!(locks.is_empty());
    assert_eq!(outcome["committed"], false);
    assert_eq!(outcome["writes_library"], false);
    assert_eq!(outcome["storage_mutation"], false);
    assert_eq!(outcome["metadata_mutation"], false);
    assert_eq!(
        outcome["safe_error_code"],
        "nfo_sidecar_import_metadata_commit_failed"
    );
    assert!(
        !stored
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains(&temp.path().display().to_string())
    );
    assert!(
        !stored
            .outcome_json
            .as_deref()
            .unwrap_or_default()
            .contains("<movie")
    );
}

#[tokio::test]
async fn nfo_sidecar_apply_rejects_stale_import_content_without_metadata_or_sidecar_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>Accepted NFO Title</title>
  <plot>Accepted overview</plot>
</movie>
"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Movies);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::LocalFirst;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Movies".to_owned(),
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
            title: "Original Title".to_owned(),
            overview: Some("Original overview".to_owned()),
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
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Import, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_import_preview_request(
            &preview,
            item.id,
            source.id,
            "nfo-sidecar-import-stale-content-1",
        ))
        .await
        .unwrap();
    fs::write(
        temp.path().join("demo.nfo"),
        r#"<movie>
  <title>Changed NFO Title</title>
  <plot>Changed overview</plot>
</movie>
"#,
    )
    .unwrap();

    let err = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();
    let stored = store
        .get_nfo_sidecar_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let sidecar_xml = fs::read_to_string(temp.path().join("demo.nfo")).unwrap();
    let outcome = stored.outcome_json.as_deref().unwrap_or_default();

    assert!(matches!(err, TaruError::Conflict { .. }));
    assert_eq!(stored.state, NfoSidecarApplyState::FailedBeforeMutation);
    assert_eq!(loaded.metadata.title, "Original Title");
    assert_eq!(
        loaded.metadata.overview,
        Some("Original overview".to_owned())
    );
    assert!(sidecar_xml.contains("Changed NFO Title"));
    assert!(outcome.contains(r#""safe_error_code":"nfo_sidecar_apply_preview_stale""#));
    assert!(!outcome.contains(&temp.path().display().to_string()));
    assert!(!outcome.contains("<movie"));
}

#[tokio::test]
async fn nfo_sidecar_apply_import_respects_user_locked_fields_and_confirms_hierarchy() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("TV").join("Firefly")).unwrap();
    fs::write(
        temp.path()
            .join("TV")
            .join("Firefly")
            .join("Firefly.S01E02.mkv"),
        b"media",
    )
    .unwrap();
    fs::write(
        temp.path()
            .join("TV")
            .join("Firefly")
            .join("Firefly.S01E02.nfo"),
        r#"<episodedetails>
  <title>The Train Job</title>
  <showtitle>Firefly</showtitle>
  <season>1</season>
  <episode>2</episode>
  <aired>2002-09-20</aired>
  <plot>The crew takes a train heist job.</plot>
</episodedetails>
"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
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
            name: "TV".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Tv,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let mut options = LibraryOptions::from_preset(taru_core::LibraryPreset::Tv);
    options.metadata_profile.local_metadata_policy = LocalMetadataPolicy::LocalFirst;
    store
        .upsert_library(&Library {
            id: library_id,
            name: "TV".to_owned(),
            roots: vec!["local:///TV".to_owned()],
            options,
        })
        .await
        .unwrap();
    let series = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Fireflie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let season = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Season,
        parent_id: Some(series.id),
        metadata: CanonicalMetadata {
            title: "Season 01".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let episode = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Episode,
        parent_id: Some(season.id),
        metadata: CanonicalMetadata {
            title: "User Locked Episode Title".to_owned(),
            overview: Some("Original overview".to_owned()),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: episode.id,
        locator: "local:///TV/Firefly/Firefly.S01E02.mkv".to_owned(),
        file_name: "Firefly.S01E02.mkv".to_owned(),
        size_bytes: Some(1),
        fingerprint: None,
    };
    for item in [&series, &season, &episode] {
        store.upsert_media_item(item).await.unwrap();
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id,
                item_id: item.id,
                provisional: true,
            })
            .await
            .unwrap();
    }
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_field_lock(&taru_core::MetadataFieldLock {
            item_id: episode.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();
    let preview = app
        .nfo()
        .preview_library_nfo_authority(library_id, NfoAuthorityPreviewOperation::Import, false)
        .await
        .unwrap();
    let accepted = app
        .nfo()
        .accept_sidecar_apply(accept_current_import_preview_request(
            &preview,
            episode.id,
            source.id,
            "nfo-sidecar-import-hierarchy-lock-1",
        ))
        .await
        .unwrap();

    let applied = app
        .nfo()
        .apply_sidecar_apply(ApplyNfoSidecarApplyRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let loaded_series = store.get_media_item(series.id).await.unwrap().unwrap();
    let loaded_season = store.get_media_item(season.id).await.unwrap().unwrap();
    let loaded_episode = store.get_media_item(episode.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(episode.id).await.unwrap();
    let sidecar_xml = fs::read_to_string(
        temp.path()
            .join("TV")
            .join("Firefly")
            .join("Firefly.S01E02.nfo"),
    )
    .unwrap();

    assert_eq!(applied.state, NfoSidecarApplyState::Committed);
    assert_eq!(loaded_series.metadata.title, "Firefly");
    assert_eq!(loaded_season.metadata.title, "Season 1");
    assert_eq!(loaded_episode.metadata.title, "User Locked Episode Title");
    assert_eq!(
        loaded_episode.metadata.overview,
        Some("The crew takes a train heist job.".to_owned())
    );
    for item_id in [series.id, season.id, episode.id] {
        assert!(
            !store
                .get_library_item_state(library_id, item_id)
                .await
                .unwrap()
                .unwrap()
                .provisional
        );
    }
    assert!(locks.iter().any(|lock| {
        lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::User
    }));
    assert!(sidecar_xml.contains("<title>The Train Job</title>"));
}

fn accept_current_export_preview_request(
    preview: &NfoAuthorityPreviewSummary,
    item_id: MediaItemId,
    source_id: MediaSourceId,
    idempotency_key: &str,
) -> AcceptNfoSidecarApplyRequest {
    let decision = preview
        .decisions
        .iter()
        .find(|decision| decision.item_id == item_id && decision.source_id == source_id)
        .expect("preview should contain source decision");
    AcceptNfoSidecarApplyRequest {
        target_library_id: preview.library_id,
        media_item_id: item_id,
        media_source_id: Some(source_id),
        requested_by: UserPrincipalId::local_admin(),
        idempotency_key: idempotency_key.to_owned(),
        operation_kind: NfoSidecarApplyOperationKind::ExportSidecar,
        sidecar_locator: decision
            .nfo_uri
            .as_ref()
            .expect("export preview should include sidecar locator")
            .to_string(),
        accepted_preview: preview.clone(),
        accepted_warning_codes: Vec::new(),
    }
}

fn accept_current_import_preview_request(
    preview: &NfoAuthorityPreviewSummary,
    item_id: MediaItemId,
    source_id: MediaSourceId,
    idempotency_key: &str,
) -> AcceptNfoSidecarApplyRequest {
    let decision = preview
        .decisions
        .iter()
        .find(|decision| decision.item_id == item_id && decision.source_id == source_id)
        .expect("preview should contain source decision");
    AcceptNfoSidecarApplyRequest {
        target_library_id: preview.library_id,
        media_item_id: item_id,
        media_source_id: Some(source_id),
        requested_by: UserPrincipalId::local_admin(),
        idempotency_key: idempotency_key.to_owned(),
        operation_kind: NfoSidecarApplyOperationKind::ImportSidecar,
        sidecar_locator: decision
            .nfo_uri
            .as_ref()
            .expect("import preview should include sidecar locator")
            .to_string(),
        accepted_preview: preview.clone(),
        accepted_warning_codes: Vec::new(),
    }
}

struct FailingNfoWriteBackend {
    inner: LocalFsBackend,
}

impl FailingNfoWriteBackend {
    fn new(root: impl Into<PathBuf>) -> taru_core::Result<Self> {
        Ok(Self {
            inner: LocalFsBackend::new(root)?,
        })
    }

    fn fail_if_nfo_write(&self, uri: &StorageUri) -> taru_core::Result<()> {
        if uri.as_str().ends_with(".nfo") {
            return Err(TaruError::storage_io(
                uri.to_string(),
                "injected NFO sidecar write failure before storage mutation",
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl StorageBackend for FailingNfoWriteBackend {
    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> taru_core::Result<ObjectMetadata> {
        self.inner.stat(uri).await
    }

    async fn list(&self, uri: &StorageUri) -> taru_core::Result<Vec<ObjectMetadata>> {
        self.inner.list(uri).await
    }

    async fn open_range(
        &self,
        uri: &StorageUri,
        range: Option<ByteRange>,
    ) -> taru_core::Result<VirtualFile> {
        self.inner.open_range(uri, range).await
    }

    async fn read_range(
        &self,
        uri: &StorageUri,
        range: Option<ByteRange>,
    ) -> taru_core::Result<ReadRange> {
        self.inner.read_range(uri, range).await
    }

    async fn stream_range(
        &self,
        uri: &StorageUri,
        range: Option<ByteRange>,
    ) -> taru_core::Result<ReadStream> {
        self.inner.stream_range(uri, range).await
    }

    async fn read_to_string(&self, uri: &StorageUri) -> taru_core::Result<String> {
        self.inner.read_to_string(uri).await
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> taru_core::Result<()> {
        self.fail_if_nfo_write(uri)?;
        self.inner.write_string(uri, content).await
    }

    async fn write(&self, request: StorageWriteRequest) -> taru_core::Result<StorageWriteReport> {
        self.fail_if_nfo_write(&request.uri)?;
        self.inner.write(request).await
    }

    async fn stage(&self, request: StageRequest) -> taru_core::Result<StagedFile> {
        self.inner.stage(request).await
    }
}
