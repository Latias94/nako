use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    Router,
    http::{Method, StatusCode as AxumStatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use taru_api::EnqueueMetadataMaintenanceRequest;
use taru_core::{
    CanonicalMetadata, DomainEventKind, DomainEventSubject, EventOutboxRepository, JobId, JobKind,
    JobStatus, LibraryId, MediaItem, MediaItemId, MediaKind, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, MetadataField, MetadataRefreshMode, MetadataRepository, MetadataSource,
    NewStagingManifestRecord, NewTranscodeSession, ProviderRawResponse, StagingManifestId,
    StagingManifestRepository, StagingPurpose, StagingState, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRepository, TranscodeSessionState,
};
use taru_core::{ExternalProvider, MetadataMatchKind, MetadataProviderAttemptStatus};
use taru_library::{LibraryScanRequest, LibraryScanner};
use taru_metadata::MetadataRefreshSummary;
use taru_streaming::{ClientPlaybackCapabilities, DirectPlayRangeRequest};
use taru_transcode::RemuxContainer;
use taru_vfs::{
    ByteRange, ObjectKind, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile,
    StorageBackend, StorageCapabilities, VirtualFile,
};
use tokio::{net::TcpListener, sync::Notify};

use super::playback::{
    HlsSourceDisposition, HlsStagingPolicy, RemuxRequestKey, RemuxSourceDisposition,
    RemuxStagingPolicy, source_path_for_ffmpeg_with_backend,
};
use super::*;
use crate::config::{
    LocalLibraryConfig, MetadataConfig, MetadataMaintenanceConfig, MetadataMaintenancePolicyConfig,
    PlaybackConfig, StagingConfig, TranscodeConfig, WebDavLibraryConfig,
};

#[tokio::test]
async fn scan_library_persists_job_success() {
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

    let output = app.scan_library(library_id).await.unwrap();
    let job = app.get_job(output.job.id).await.unwrap();
    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(job.status, JobStatus::Succeeded);
    assert_eq!(output.index.discovered_files, 0);
    assert_eq!(output.probe.total_sources, 0);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, DomainEventKind::LibraryScanned);
    assert_eq!(events[0].subject, DomainEventSubject::Library(library_id));
    assert!(events[0].payload_json.contains(&output.job.id.to_string()));
    assert!(
        !events[0]
            .payload_json
            .contains(&temp.path().display().to_string())
    );
}

#[tokio::test]
async fn app_startup_rejects_duplicate_configured_library_ids() {
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
        libraries: vec![
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: taru_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().join("anime"),
                preset: taru_core::LibraryPreset::Anime,
                webdav: None,
            },
        ],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();

    let err = TaruApp::new_with_store(config, store).await.unwrap_err();

    let TaruError::InvalidInput { message } = err else {
        panic!("expected duplicate library id validation error");
    };
    assert!(message.contains("duplicate configured library id"));
    assert!(message.contains(&library_id.to_string()));
}

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

#[tokio::test]
async fn nfo_import_uses_configured_webdav_backend() {
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
        remux_staging_root: temp.path().join("cache").join("remux"),
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

    let output = app.import_library_nfo(library_id).await.unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();

    assert_eq!(output.import.imported_items, 1);
    assert_eq!(loaded.metadata.title, "Remote NFO");
}

#[tokio::test]
async fn nfo_export_rejects_read_only_webdav_backend() {
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
        remux_staging_root: temp.path().join("cache").join("remux"),
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

    let err = app.export_library_nfo(library_id).await.unwrap_err();

    assert_eq!(
        err,
        TaruError::Unsupported("NFO export requires a writable storage backend")
    );
}

#[tokio::test]
async fn metadata_refresh_job_input_does_not_include_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.tmdb.enabled = true;
    metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
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
        metadata,
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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();

    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
    let input = job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(job.kind, JobKind::MetadataRefresh);
    assert_eq!(job.resource_class, "metadata.tmdb");
    assert_eq!(job.library_id, Some(library_id));
    assert_eq!(
        input.get("item_id").and_then(serde_json::Value::as_str),
        Some(item.id.to_string().as_str())
    );
    assert_eq!(
        input.get("provider").and_then(serde_json::Value::as_str),
        Some("tmdb")
    );
    assert_eq!(
        input
            .get("refresh_mode")
            .and_then(serde_json::Value::as_str),
        Some("default")
    );
    assert!(input.get("access_token").is_none());
    assert!(input.get("api_key").is_none());
}

#[tokio::test]
async fn metadata_refresh_job_records_disabled_profile_provider_for_executor() {
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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();

    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
    let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

    assert_eq!(job.kind, JobKind::MetadataRefresh);
    assert_eq!(job.resource_class, "metadata.tmdb");
    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("tmdb=skipped_disabled"));
    assert!(message.contains("disabled in config"));
}

#[tokio::test]
async fn metadata_refresh_falls_back_from_unimplemented_bangumi_to_tmdb_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.tmdb.enabled = true;
    metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Anime Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
    let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

    assert_eq!(job.resource_class, "metadata.bangumi");
    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("bangumi=not_implemented"));
    assert!(message.contains("tmdb=skipped_unavailable"));
    assert_eq!(app.get_job(job.id).await.unwrap().status, JobStatus::Queued);
}

#[tokio::test]
async fn metadata_refresh_resolves_provider_order_from_library_profile() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.tmdb.enabled = true;
    metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Anime".to_owned(),
            root: temp.path().to_path_buf(),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Anime Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    let job = app.create_metadata_refresh_job(item.id).await.unwrap();
    let input = job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(job.resource_class, "metadata.bangumi");
    assert_eq!(
        input.get("provider").and_then(serde_json::Value::as_str),
        Some("bangumi")
    );
}

#[tokio::test]
async fn metadata_maintenance_job_refreshes_library_items_and_summarizes_attempts() {
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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    let output = app
        .run_metadata_maintenance(EnqueueMetadataMaintenanceRequest {
            library_id: Some(library_id),
            item_ids: Vec::new(),
            providers: Some(vec![ExternalProvider::Tmdb]),
            item_kinds: vec![MediaKind::Movie],
            profile: None,
            language: Some("en-US".to_owned()),
            refresh_mode: Some(MetadataRefreshMode::MissingOnly),
            force: false,
        })
        .await
        .unwrap();
    let input = output
        .job
        .input_json
        .as_ref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .unwrap();

    assert_eq!(output.job.kind, JobKind::MetadataMaintenance);
    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(output.summary.requested_items, 1);
    assert_eq!(output.summary.attempted_items, 1);
    assert_eq!(output.summary.succeeded_items, 0);
    assert_eq!(output.summary.failed_items, 1);
    assert_eq!(output.summary.provider_attempts.len(), 1);
    assert_eq!(
        output.summary.provider_attempts[0].status,
        MetadataProviderAttemptStatus::SkippedDisabled
    );
    assert!(input.get("access_token").is_none());
    assert!(input.get("api_key").is_none());

    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::MetadataMaintenanceCompleted
            && event.subject == DomainEventSubject::Job(output.job.id)
    }));
}

#[tokio::test]
async fn metadata_lifecycle_config_maps_policy_and_cleans_raw_cache_on_startup() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let item_id = MediaItemId::new();
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
        metadata: MetadataConfig {
            raw_cache_retention_ms: 1,
            maintenance: MetadataMaintenanceConfig {
                raw_cache_cleanup_on_startup: true,
                raw_cache_cleanup_interval_ms: 0,
                policies: vec![MetadataMaintenancePolicyConfig {
                    id: "movies-nightly".to_owned(),
                    enabled: true,
                    library_id: Some(library_id),
                    item_ids: Vec::new(),
                    providers: Some(vec![ExternalProvider::Tmdb]),
                    item_kinds: vec![MediaKind::Movie],
                    profile: None,
                    language: Some("en-US".to_owned()),
                    refresh_mode: Some(MetadataRefreshMode::MissingOnly),
                    force: false,
                    interval_ms: 86_400_000,
                    initial_delay_ms: 86_400_000,
                }],
            },
            ..MetadataConfig::default()
        },
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
    store.migrate().await.unwrap();
    let item = MediaItem {
        id: item_id,
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Lifecycle Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_provider_raw_response(&ProviderRawResponse {
            item_id,
            provider: ExternalProvider::Tmdb,
            provider_key: "1".to_owned(),
            fetched_at: "2020-01-01T00:00:00.000Z".to_owned(),
            body_json: "{}".to_owned(),
        })
        .await
        .unwrap();

    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let raw = store
        .list_provider_raw_responses(
            item_id,
            taru_core::ProviderRawResponseFilter::default(),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    let request = app
        .metadata_maintenance_request_from_policy(&app.config().metadata.maintenance.policies[0]);

    assert!(raw.is_empty());
    assert_eq!(request.library_id, Some(library_id));
    assert_eq!(request.providers, Some(vec![ExternalProvider::Tmdb]));
    assert_eq!(request.item_kinds, vec![MediaKind::Movie]);
    assert_eq!(request.refresh_mode, Some(MetadataRefreshMode::MissingOnly));
}

#[tokio::test]
async fn metadata_refresh_event_payload_uses_ids_not_secrets() {
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
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    let job_id = JobId::new();
    let refresh = MetadataRefreshSummary {
        job_id,
        item_id: item.id,
        provider: ExternalProvider::Tmdb,
        selected_provider: ExternalProvider::Tmdb,
        provider_key: "603".to_owned(),
        matched_by: MetadataMatchKind::ExternalId,
        refresh_mode: MetadataRefreshMode::MissingOnly,
        updated: true,
        attempted_providers: vec![taru_metadata::MetadataProviderAttempt {
            provider: ExternalProvider::Tmdb,
            status: MetadataProviderAttemptStatus::Succeeded,
            message: None,
            provider_key: Some("603".to_owned()),
            matched_by: Some(MetadataMatchKind::ExternalId),
            error_class: None,
        }],
    };

    app.record_metadata_refreshed_event(job_id, item.id, &refresh)
        .await;
    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, DomainEventKind::ItemMetadataRefreshed);
    assert_eq!(events[0].subject, DomainEventSubject::Item(item.id));
    assert!(!events[0].payload_json.contains("TMDB_READ_ACCESS_TOKEN"));
    assert!(
        !events[0]
            .payload_json
            .contains(&temp.path().display().to_string())
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

    let output = app.import_library_nfo(library_id).await.unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let locks = store.list_field_locks(item.id).await.unwrap();
    let job = app.get_job(output.job.id).await.unwrap();
    let events = store
        .list_outbox_events(PageRequest::first_page())
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
async fn remux_source_runs_runner_and_reuses_completed_output() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
    let request = RemuxSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
        output_container: RemuxContainer::Mp4,
    };

    let output = app.remux_source(request.clone()).await.unwrap();
    let session = output.session.as_ref().unwrap();

    assert_eq!(output.disposition, RemuxSourceDisposition::Finished);
    assert_eq!(session.state, TranscodeSessionState::Finished);
    assert!(
        output
            .output_path
            .starts_with(&app.config().remux_staging_root)
    );
    assert_eq!(fs::read_to_string(&output.output_path).unwrap(), "remuxed");
    assert_eq!(
        app.get_transcode_session(session.id).await.unwrap().state,
        TranscodeSessionState::Finished
    );
    assert_eq!(
        store
            .find_latest_transcode_session(
                source.id,
                TranscodeSessionKind::Remux,
                &RemuxRequestKey {
                    source_id: source.id,
                    output_container: RemuxContainer::Mp4,
                }
                .persisted_request_key(),
            )
            .await
            .unwrap()
            .unwrap()
            .id,
        session.id
    );
    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::PlaybackSessionFinished
            && event.subject == DomainEventSubject::PlaybackSession(session.id)
            && event.source_id == Some(source.id)
            && !event
                .payload_json
                .contains(&app.config().remux_staging_root.display().to_string())
    }));

    let reused = app.remux_source(request.clone()).await.unwrap();

    assert_eq!(reused.disposition, RemuxSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.as_ref().unwrap().id, session.id);
    assert_eq!(reused.output_path, output.output_path);
    assert_eq!(fs::read_to_string(reused.output_path).unwrap(), "remuxed");

    let config = app.config().clone();
    drop(app);
    fs::remove_file(ffmpeg_path).unwrap();
    let restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let restarted_reused = restarted.remux_source(request).await.unwrap();

    assert_eq!(
        restarted_reused.disposition,
        RemuxSourceDisposition::ReusedExisting
    );
    assert_eq!(restarted_reused.session.as_ref().unwrap().id, session.id);
}

#[tokio::test]
async fn remux_source_rejects_persisted_active_duplicate() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let key = RemuxRequestKey {
        source_id: source.id,
        output_container: RemuxContainer::Mp4,
    };
    let staging = RemuxStagingPolicy::new(&app.config().remux_staging_root).unwrap();
    let active = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: key.persisted_request_key(),
            output_path: staging.output_path(source.id, RemuxContainer::Mp4).unwrap(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let err = app
        .remux_source(RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        })
        .await
        .unwrap_err();

    let TaruError::Conflict { message } = err else {
        panic!("expected remux duplicate conflict");
    };
    assert!(message.contains("already in progress"));
    assert!(message.contains(&active.id.to_string()));
}

#[tokio::test]
async fn remux_source_persists_runner_failure() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_failing_ffmpeg_script(script_root.path(), "failure");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let request_key = RemuxRequestKey {
        source_id: source.id,
        output_container: RemuxContainer::Mp4,
    }
    .persisted_request_key();

    let err = app
        .remux_source(RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected remux provider failure");
    };
    assert_eq!(provider, "ffmpeg_remux");
    assert_eq!(message, "remux runner failed");

    let session = store
        .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.state, TranscodeSessionState::Failed);
    assert_eq!(
        session.failure_category,
        Some(TranscodeFailureCategory::Runner)
    );
    assert_eq!(
        session.failure_message.as_deref(),
        Some("external provider error from ffmpeg_remux: remux runner failed")
    );
}

#[tokio::test]
async fn app_startup_marks_stale_transcode_sessions_failed() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let config = app.config().clone();
    let staging = RemuxStagingPolicy::new(&config.remux_staging_root).unwrap();
    let stale_id = TranscodeSessionId::new();

    store
        .create_transcode_session(NewTranscodeSession {
            id: stale_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: RemuxRequestKey {
                source_id: source.id,
                output_container: RemuxContainer::Mp4,
            }
            .persisted_request_key(),
            output_path: staging.output_path(source.id, RemuxContainer::Mp4).unwrap(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    drop(app);
    let _restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let stale = store
        .get_transcode_session(stale_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(stale.state, TranscodeSessionState::Failed);
    assert_eq!(
        stale.failure_category,
        Some(TranscodeFailureCategory::Stale)
    );
}

#[tokio::test]
async fn hls_source_runs_runner_and_reuses_completed_session() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
    let request = HlsSourceRequest {
        source_id: source.id,
        client: ClientPlaybackCapabilities::default(),
    };

    let output = app.hls_source(request.clone()).await.unwrap();
    let session_id = output.session.id;

    assert_eq!(output.disposition, HlsSourceDisposition::Finished);
    assert_eq!(output.session.kind, TranscodeSessionKind::HlsTranscode);
    assert_eq!(output.session.state, TranscodeSessionState::Finished);
    assert!(
        fs::read_to_string(&output.playlist_path)
            .unwrap()
            .contains("#EXTM3U")
    );
    assert_eq!(
        fs::read_to_string(output.segment_dir.join("segment_00000.ts")).unwrap(),
        "segment"
    );

    let playlist = app.hls_playlist(request.clone()).await.unwrap();
    assert!(playlist.body.contains(&format!(
        "/playback/sessions/{session_id}/hls/segments/segment_00000.ts"
    )));

    let segment = app
        .plan_hls_segment(session_id, "segment_00000.ts")
        .await
        .unwrap();
    assert_eq!(segment.content_type, "video/mp2t");
    assert!(segment.path.ends_with("segment_00000.ts"));
    assert!(
        app.plan_hls_segment(session_id, "../segment_00000.ts")
            .await
            .is_err()
    );
    let events = store
        .list_outbox_events(PageRequest::first_page())
        .await
        .unwrap();
    assert!(events.iter().any(|event| {
        event.kind == DomainEventKind::PlaybackSessionFinished
            && event.subject == DomainEventSubject::PlaybackSession(session_id)
            && event.source_id == Some(source.id)
            && !event
                .payload_json
                .contains(&app.config().remux_staging_root.display().to_string())
    }));

    fs::remove_file(ffmpeg_path).unwrap();
    let reused = app.hls_source(request.clone()).await.unwrap();
    assert_eq!(reused.disposition, HlsSourceDisposition::ReusedExisting);
    assert_eq!(reused.session.id, session_id);

    let config = app.config().clone();
    drop(app);
    let restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let restarted_reused = restarted.hls_source(request).await.unwrap();

    assert_eq!(
        restarted_reused.disposition,
        HlsSourceDisposition::ReusedExisting
    );
    assert_eq!(restarted_reused.session.id, session_id);
}

#[tokio::test]
async fn hls_source_rejects_persisted_active_duplicate() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
    let staging = HlsStagingPolicy::new(app.config().remux_staging_root.join("hls")).unwrap();
    let layout = staging.single_variant_layout(source.id).unwrap();
    let active = store
        .create_transcode_session(NewTranscodeSession {
            id: TranscodeSessionId::new(),
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: "hls:single".to_owned(),
            output_path: layout.playlist_path,
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let err = app
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
        })
        .await
        .unwrap_err();

    let TaruError::Conflict { message } = err else {
        panic!("expected hls duplicate conflict");
    };
    assert!(message.contains("already in progress"));
    assert!(message.contains(&active.id.to_string()));

    let segment_err = app
        .plan_hls_segment(active.id, "segment_00000.ts")
        .await
        .unwrap_err();
    let TaruError::Conflict { message } = segment_err else {
        panic!("expected hls segment readiness conflict");
    };
    assert!(message.contains("is not ready"));
}

#[tokio::test]
async fn hls_source_persists_runner_failure() {
    let script_root = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_failing_hls_ffmpeg_script(script_root.path(), "hls_failure");
    let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;

    let err = app
        .hls_source(HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected hls provider failure");
    };
    assert_eq!(provider, "ffmpeg_hls");
    assert_eq!(message, "hls runner failed");

    let session = store
        .find_latest_transcode_session(source.id, TranscodeSessionKind::HlsTranscode, "hls:single")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.state, TranscodeSessionState::Failed);
    assert_eq!(
        session.failure_category,
        Some(TranscodeFailureCategory::Runner)
    );
    assert_eq!(
        session.failure_message.as_deref(),
        Some("external provider error from ffmpeg_hls: hls runner failed")
    );
}

#[tokio::test]
async fn direct_play_uses_vfs_stream_when_backend_has_no_local_path() {
    let backend = RemotePlaybackBackend {
        bytes: b"remote-media".to_vec(),
        local_path_hint: None,
    };
    let source = remote_media_source("webdav:///Movies/Demo.mkv");
    let uri = StorageUri::parse(&source.locator).unwrap();
    let range = taru_streaming::RequestedByteRange {
        start: Some(2),
        end: Some(5),
    };

    let (response, body) = plan_direct_play_with_backend(
        &source,
        &uri,
        &backend,
        DirectPlayRangeRequest::Range(range),
    )
    .await
    .unwrap();

    assert_eq!(response.body_len, 4);
    assert_eq!(response.content_range.as_deref(), Some("bytes 2-5/12"));
    let DirectPlaySourceBody::Stream(stream) = body else {
        panic!("expected direct play to return a VFS stream");
    };
    assert_eq!(
        stream.stream.range,
        Some(ByteRange {
            offset: 2,
            length: Some(4)
        })
    );
}

#[tokio::test]
async fn ffmpeg_source_path_stages_remote_backend_without_local_path_hint() {
    let temp = tempfile::tempdir().unwrap();
    let backend = RemotePlaybackBackend {
        bytes: b"remote-media".to_vec(),
        local_path_hint: None,
    };
    let source = remote_media_source("webdav:///Movies/Demo.mkv");
    let uri = StorageUri::parse(&source.locator).unwrap();
    let staging_root = temp.path().join("remux").join("inputs");

    let input_path =
        source_path_for_ffmpeg_with_backend(&source, &uri, &backend, staging_root.clone())
            .await
            .unwrap();

    assert!(input_path.starts_with(&staging_root));
    assert_eq!(fs::read(&input_path).unwrap(), b"remote-media");
    assert!(!input_path.display().to_string().contains("webdav://"));
}

#[tokio::test]
async fn manifest_recording_backend_records_probe_staging() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: b"probe-media".to_vec(),
            local_path_hint: None,
        }),
        store.clone(),
        StagingPurpose::ProbeInput,
        StagingConfig::default().max_bytes,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(1)),
    );
    let uri = StorageUri::parse("webdav:///Movies/Demo.mkv").unwrap();

    let staged = backend
        .stage(StageRequest::new(
            uri.clone(),
            temp.path().join("probe-inputs"),
        ))
        .await
        .unwrap();

    assert_eq!(fs::read(&staged.path).unwrap(), b"probe-media");
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.source_uri, uri.to_string());
    assert_eq!(record.source_scheme, "webdav");
    assert_eq!(record.purpose, StagingPurpose::ProbeInput);
    assert_eq!(record.local_path, staged.path.display().to_string());
    assert_eq!(record.size_bytes, Some(11));
    assert_eq!(record.etag.as_deref(), Some("etag-remote"));
    assert_eq!(record.fingerprint.as_deref(), Some("remote-fingerprint"));
    assert!(record.expires_at_ms.unwrap() > record.created_at_ms);
}

#[tokio::test]
async fn manifest_recording_backend_rejects_staging_over_disk_budget() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: b"probe-media".to_vec(),
            local_path_hint: None,
        }),
        store.clone(),
        StagingPurpose::ProbeInput,
        5,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(1)),
    );
    let uri = StorageUri::parse("webdav:///Movies/Demo.mkv").unwrap();
    let staging_root = temp.path().join("probe-inputs");

    let err = backend
        .stage(StageRequest::new(uri.clone(), staging_root.clone()))
        .await
        .unwrap_err();

    let TaruError::Storage { message, .. } = err else {
        panic!("expected storage budget error");
    };
    assert!(message.contains("staging disk budget exhausted"));
    assert!(!staging_root.exists());
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert!(records.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manifest_recording_backend_serializes_budget_check_and_record() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = Arc::new(ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: vec![b'x'; 8],
            local_path_hint: None,
        }),
        store.clone(),
        StagingPurpose::ProbeInput,
        10,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(2)),
    ));
    let first_uri = StorageUri::parse("webdav:///Movies/First.mkv").unwrap();
    let second_uri = StorageUri::parse("webdav:///Movies/Second.mkv").unwrap();
    let staging_root = temp.path().join("probe-inputs");

    let first = {
        let backend = backend.clone();
        let staging_root = staging_root.clone();
        tokio::spawn(async move {
            backend
                .stage(StageRequest::new(first_uri, staging_root))
                .await
        })
    };
    let second = {
        let backend = backend.clone();
        let staging_root = staging_root.clone();
        tokio::spawn(async move {
            backend
                .stage(StageRequest::new(second_uri, staging_root))
                .await
        })
    };
    let first = first.await.unwrap();
    let second = second.await.unwrap();
    let successes = usize::from(first.is_ok()) + usize::from(second.is_ok());
    let failures = [first, second]
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();

    assert_eq!(successes, 1);
    assert_eq!(failures.len(), 1);
    let TaruError::Storage { message, .. } = &failures[0] else {
        panic!("expected storage budget error");
    };
    assert!(message.contains("staging disk budget exhausted"));
    assert!(store.sum_staging_manifest_bytes().await.unwrap() <= 10);
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].size_bytes, Some(8));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manifest_recording_backend_reserves_budget_without_serializing_downloads() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let control = ConcurrentStageControl::new();
    let backend = Arc::new(ManifestRecordingStorageBackend::new(
        Arc::new(ConcurrentStageBackend {
            bytes: vec![b'x'; 8],
            control: control.clone(),
        }),
        store.clone(),
        StagingPurpose::ProbeInput,
        32,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(2)),
    ));
    let first_uri = StorageUri::parse("webdav:///Movies/First.mkv").unwrap();
    let second_uri = StorageUri::parse("webdav:///Movies/Second.mkv").unwrap();
    let staging_root = temp.path().join("probe-inputs");

    let first = {
        let backend = backend.clone();
        let staging_root = staging_root.clone();
        tokio::spawn(async move {
            backend
                .stage(StageRequest::new(first_uri, staging_root))
                .await
        })
    };
    let second = {
        let backend = backend.clone();
        let staging_root = staging_root.clone();
        tokio::spawn(async move {
            backend
                .stage(StageRequest::new(second_uri, staging_root))
                .await
        })
    };

    let both_downloads_started =
        tokio::time::timeout(Duration::from_millis(200), control.both_entered.notified()).await;
    control.release();

    let first = first.await.unwrap();
    let second = second.await.unwrap();
    assert!(both_downloads_started.is_ok());
    assert!(first.is_ok());
    assert!(second.is_ok());
    assert_eq!(control.max_in_flight.load(Ordering::SeqCst), 2);
    assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 16);
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 2);
}

#[tokio::test]
async fn manifest_recording_backend_rolls_back_reservation_when_stage_fails() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(FailingStageBackend {
            len: 8,
            fingerprint: "failing-stage".to_owned(),
        }),
        store.clone(),
        StagingPurpose::ProbeInput,
        32,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(1)),
    );
    let uri = StorageUri::parse("webdav:///Movies/Failing.mkv").unwrap();

    let err = backend
        .stage(StageRequest::new(uri, temp.path().join("probe-inputs")))
        .await
        .unwrap_err();

    let TaruError::Storage { message, .. } = err else {
        panic!("expected storage failure");
    };
    assert!(message.contains("intentional staging failure"));
    assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 0);
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            None,
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].state, StagingState::Failed);
}

#[tokio::test]
async fn manifest_recording_backend_rejects_active_duplicate_path_reservation() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let uri = StorageUri::parse("webdav:///Movies/Demo.mkv").unwrap();
    let staging_root = temp.path().join("probe-inputs");
    let reserved_path =
        taru_vfs::deterministic_stage_path(&staging_root, &uri, Some("reserved")).unwrap();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: StagingManifestId::new(),
            source_uri: uri.to_string(),
            source_scheme: uri.scheme().to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: reserved_path.display().to_string(),
            size_bytes: Some(8),
            etag: Some("etag-reserved".to_owned()),
            fingerprint: Some("reserved".to_owned()),
            state: StagingState::Staging,
            created_at_ms: 1,
            updated_at_ms: 1,
            last_accessed_at_ms: 1,
            expires_at_ms: Some(i64::MAX),
            active_leases: 0,
            validation_error: None,
        })
        .await
        .unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(FailingStageBackend {
            len: 8,
            fingerprint: "reserved".to_owned(),
        }),
        store,
        StagingPurpose::ProbeInput,
        32,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(1)),
    );

    let err = backend
        .stage(StageRequest::new(uri, staging_root))
        .await
        .unwrap_err();

    let TaruError::Storage { message, .. } = err else {
        panic!("expected active reservation error");
    };
    assert!(message.contains("staging input is already reserved"));
    assert!(!message.contains("intentional staging failure"));
}

#[tokio::test]
async fn manifest_recording_backend_waits_for_stage_budget() {
    let temp = tempfile::tempdir().unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let stage_permits = Arc::new(Semaphore::new(1));
    let held_permit = stage_permits.clone().acquire_owned().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: b"probe-media".to_vec(),
            local_path_hint: None,
        }),
        store,
        StagingPurpose::ProbeInput,
        StagingConfig::default().max_bytes,
        StagingConfig::default().retention_ms,
        stage_permits,
    );
    let uri = StorageUri::parse("webdav:///Movies/Demo.mkv").unwrap();

    let blocked = tokio::time::timeout(
        Duration::from_millis(50),
        backend.stage(StageRequest::new(
            uri.clone(),
            temp.path().join("probe-inputs"),
        )),
    )
    .await;

    assert!(blocked.is_err());
    drop(held_permit);
    let staged = backend
        .stage(StageRequest::new(uri, temp.path().join("probe-inputs")))
        .await
        .unwrap();
    assert_eq!(fs::read(&staged.path).unwrap(), b"probe-media");
}

#[tokio::test]
async fn direct_play_holds_remote_stream_budget_until_body_is_dropped() {
    let server = MockWebDavServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
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
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig {
                remote_stream_concurrency: 1,
                remote_stage_concurrency: 1,
            },
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
        },
        store.clone(),
    )
    .await
    .unwrap();
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
        library_id,
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

    let DirectPlaySourceBody::Stream(_) = &plan.body else {
        panic!("expected remote direct play to hold a VFS stream");
    };
    let backend = app
        .storage_backend_for_media_source(&source)
        .await
        .unwrap()
        .1;
    assert_eq!(backend.available_stream_permits(), 0);
    drop(plan);
    assert_eq!(backend.available_stream_permits(), 1);
}

#[tokio::test]
async fn app_startup_cleans_expired_staging_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let staged_path = temp
        .path()
        .join("cache")
        .join("remux")
        .join("probe-inputs")
        .join("webdav")
        .join("old.mkv");
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&staged_path, b"old").unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let record_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(staging_manifest_record(
            record_id,
            &staged_path,
            Some(1),
            0,
        ))
        .await
        .unwrap();
    let library_id = LibraryId::new();

    let _app = TaruApp::new_with_store(
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
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            libraries: vec![LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("library"),
                preset: taru_core::LibraryPreset::Movies,
                webdav: None,
            }],
        },
        store.clone(),
    )
    .await
    .unwrap();

    assert!(!staged_path.exists());
    let record = store
        .get_staging_manifest_record(record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.state, StagingState::Deleted);
}

#[tokio::test]
async fn staging_cleanup_preserves_active_leases() {
    let temp = tempfile::tempdir().unwrap();
    let staged_path = temp.path().join("active.mkv");
    fs::write(&staged_path, b"active").unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let record_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(staging_manifest_record(
            record_id,
            &staged_path,
            Some(1),
            1,
        ))
        .await
        .unwrap();

    let cleanup = cleanup_expired_staging_inputs(&store, 2_000).await.unwrap();

    assert_eq!(cleanup.deleted_records, 0);
    assert_eq!(cleanup.deleted_files, 0);
    assert!(staged_path.exists());
    assert!(
        store
            .get_staging_manifest_record(record_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn staging_cleanup_removes_expired_pending_reservations() {
    let temp = tempfile::tempdir().unwrap();
    let staged_path = temp.path().join("pending.mkv");
    fs::write(&staged_path, b"partial").unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let record_id = StagingManifestId::new();
    let mut record = staging_manifest_record(record_id, &staged_path, Some(1), 0);
    record.state = StagingState::Staging;
    store.upsert_staging_manifest_record(record).await.unwrap();

    let cleanup = cleanup_expired_staging_inputs(&store, 2_000).await.unwrap();

    assert_eq!(cleanup.deleted_records, 1);
    assert_eq!(cleanup.deleted_files, 1);
    assert!(!staged_path.exists());
    assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 0);
    let record = store
        .get_staging_manifest_record(record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.state, StagingState::Deleted);
}

#[tokio::test]
async fn staging_lease_transitions_between_ready_and_leased() {
    let temp = tempfile::tempdir().unwrap();
    let staged_path = temp.path().join("leased.mkv");
    fs::write(&staged_path, b"leased").unwrap();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let record_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(staging_manifest_record(
            record_id,
            &staged_path,
            Some(1_000),
            0,
        ))
        .await
        .unwrap();

    let lease = super::staging::StagingLease::acquire(store.clone(), record_id)
        .await
        .unwrap();
    let leased = store
        .get_staging_manifest_record(record_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(leased.state, StagingState::Leased);
    assert_eq!(leased.active_leases, 1);

    let released = lease.release().await.unwrap();
    assert_eq!(released.state, StagingState::Ready);
    assert_eq!(released.active_leases, 0);
}

#[tokio::test]
async fn source_path_for_ffmpeg_records_manifest_for_remote_staging() {
    let server = MockWebDavServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let staging_root = temp.path().join("cache").join("remux");
    let library_id = LibraryId::new();
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
            remux_staging_root: staging_root.clone(),
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
        },
        store.clone(),
    )
    .await
    .unwrap();
    let source = remote_media_source("webdav:///Movies/Demo.mkv");

    let input_path = app.source_path_for_ffmpeg(&source).await.unwrap();

    assert!(input_path.starts_with(staging_root.join("inputs")));
    assert_eq!(fs::read(&input_path).unwrap(), b"demo");
    let records = store
        .list_staging_manifest_records(
            Some(StagingPurpose::FfmpegInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.source_uri, "webdav:///Movies/Demo.mkv");
    assert_eq!(record.source_scheme, "webdav");
    assert_eq!(record.local_path, input_path.display().to_string());
    assert_eq!(record.size_bytes, Some(4));
    assert_eq!(record.etag.as_deref(), Some("etag-demo"));
    assert_eq!(record.fingerprint.as_deref(), Some("webdav:etag=etag-demo"));
    assert!(record.expires_at_ms.unwrap() > record.created_at_ms);
}

#[tokio::test]
async fn ffmpeg_source_path_reuses_local_path_hint_without_staging() {
    let temp = tempfile::tempdir().unwrap();
    let local_path = temp.path().join("library").join("demo.mkv");
    fs::create_dir_all(local_path.parent().unwrap()).unwrap();
    fs::write(&local_path, b"local-media").unwrap();
    let backend = RemotePlaybackBackend {
        bytes: b"remote-media".to_vec(),
        local_path_hint: Some(local_path.clone()),
    };
    let source = remote_media_source("local:///demo.mkv");
    let uri = StorageUri::parse(&source.locator).unwrap();

    let input_path = source_path_for_ffmpeg_with_backend(
        &source,
        &uri,
        &backend,
        temp.path().join("remux").join("inputs"),
    )
    .await
    .unwrap();

    assert_eq!(input_path, local_path);
}

#[test]
fn remux_staging_policy_rejects_escaping_roots() {
    assert!(RemuxStagingPolicy::new(PathBuf::new()).is_err());
    assert!(RemuxStagingPolicy::new(PathBuf::from("cache/../outside")).is_err());

    let policy = RemuxStagingPolicy::new(PathBuf::from("cache/remux")).unwrap();
    let output = policy
        .output_path(MediaSourceId::new(), RemuxContainer::Mkv)
        .unwrap();

    assert!(output.starts_with(PathBuf::from("cache/remux")));
    assert_eq!(
        output.extension().and_then(|value| value.to_str()),
        Some("mkv")
    );
}

#[test]
fn hls_staging_policy_rejects_escaping_roots() {
    assert!(HlsStagingPolicy::new(PathBuf::new()).is_err());
    assert!(HlsStagingPolicy::new(PathBuf::from("cache/../outside")).is_err());

    let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
    let layout = policy.single_variant_layout(MediaSourceId::new()).unwrap();

    assert!(layout.output_dir.starts_with(PathBuf::from("cache/hls")));
    assert!(layout.playlist_path.starts_with(PathBuf::from("cache/hls")));
    assert!(
        layout
            .segment_pattern
            .starts_with(PathBuf::from("cache/hls"))
    );
    assert_eq!(
        layout
            .playlist_path
            .file_name()
            .and_then(|value| value.to_str()),
        Some("playlist.m3u8")
    );
}

fn remote_media_source(locator: &str) -> MediaSource {
    MediaSource {
        id: MediaSourceId::new(),
        library_id: LibraryId::new(),
        item_id: MediaItemId::new(),
        locator: locator.to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(12),
        fingerprint: Some("remote-fingerprint".to_owned()),
    }
}

fn staging_manifest_record(
    id: StagingManifestId,
    local_path: &Path,
    expires_at_ms: Option<i64>,
    active_leases: u32,
) -> NewStagingManifestRecord {
    NewStagingManifestRecord {
        id,
        source_uri: "webdav:///Movies/Demo.mkv".to_owned(),
        source_scheme: "webdav".to_owned(),
        purpose: StagingPurpose::ProbeInput,
        local_path: local_path.display().to_string(),
        size_bytes: Some(3),
        etag: Some("etag-staged".to_owned()),
        fingerprint: Some("fingerprint-staged".to_owned()),
        state: StagingState::Ready,
        created_at_ms: 1,
        updated_at_ms: 1,
        last_accessed_at_ms: 1,
        expires_at_ms,
        active_leases,
        validation_error: None,
    }
}

struct RemotePlaybackBackend {
    bytes: Vec<u8>,
    local_path_hint: Option<PathBuf>,
}

#[derive(Clone)]
struct ConcurrentStageControl {
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
    released: Arc<AtomicBool>,
    both_entered: Arc<Notify>,
    release_notify: Arc<Notify>,
}

impl ConcurrentStageControl {
    fn new() -> Self {
        Self {
            in_flight: Arc::new(AtomicUsize::new(0)),
            max_in_flight: Arc::new(AtomicUsize::new(0)),
            released: Arc::new(AtomicBool::new(false)),
            both_entered: Arc::new(Notify::new()),
            release_notify: Arc::new(Notify::new()),
        }
    }

    fn release(&self) {
        self.released.store(true, Ordering::SeqCst);
        self.release_notify.notify_waiters();
    }
}

struct ConcurrentStageBackend {
    bytes: Vec<u8>,
    control: ConcurrentStageControl,
}

struct FailingStageBackend {
    len: u64,
    fingerprint: String,
}

#[async_trait]
impl StorageBackend for FailingStageBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            uri: uri.clone(),
            kind: ObjectKind::File,
            len: Some(self.len),
            modified_at: None,
            etag: Some("etag-failing".to_owned()),
            fingerprint: Some(self.fingerprint.clone()),
            capabilities: StorageCapabilities::RANGE_READABLE | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        })
    }

    async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(Vec::new())
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Ok(String::new())
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(TaruError::Unsupported("test backend is read-only"))
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        Err(TaruError::Storage {
            uri: request.uri.to_string(),
            message: "intentional staging failure".to_owned(),
        })
    }
}

#[async_trait]
impl StorageBackend for ConcurrentStageBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            uri: uri.clone(),
            kind: ObjectKind::File,
            len: Some(self.bytes.len() as u64),
            modified_at: None,
            etag: Some("etag-concurrent".to_owned()),
            fingerprint: Some(format!("fingerprint-{}", uri.path_part())),
            capabilities: StorageCapabilities::RANGE_READABLE | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        })
    }

    async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(Vec::new())
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Ok(String::new())
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(TaruError::Unsupported("test backend is read-only"))
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let current = self.control.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.control
            .max_in_flight
            .fetch_max(current, Ordering::SeqCst);
        if current == 2 {
            self.control.both_entered.notify_waiters();
        }

        while !self.control.released.load(Ordering::SeqCst) {
            self.control.release_notify.notified().await;
        }

        self.control.in_flight.fetch_sub(1, Ordering::SeqCst);
        let path = taru_vfs::deterministic_stage_path(
            &request.root,
            &request.uri,
            Some(&format!("fingerprint-{}", request.uri.path_part())),
        )?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| TaruError::Storage {
                    uri: parent.display().to_string(),
                    message: format!("failed to create test staging directory: {err}"),
                })?;
        }
        tokio::fs::write(&path, &self.bytes)
            .await
            .map_err(|err| TaruError::Storage {
                uri: path.display().to_string(),
                message: format!("failed to write test staging file: {err}"),
            })?;

        Ok(StagedFile {
            uri: request.uri,
            path,
            len: Some(self.bytes.len() as u64),
            etag: Some("etag-concurrent".to_owned()),
            fingerprint: Some("fingerprint-concurrent".to_owned()),
            reused: false,
        })
    }
}

#[async_trait]
impl StorageBackend for RemotePlaybackBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            uri: uri.clone(),
            kind: ObjectKind::File,
            len: Some(self.bytes.len() as u64),
            modified_at: None,
            etag: Some("etag-remote".to_owned()),
            fingerprint: Some("remote-fingerprint".to_owned()),
            capabilities: StorageCapabilities::RANGE_READABLE | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        })
    }

    async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(Vec::new())
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: self.local_path_hint.clone(),
        })
    }

    async fn read_range(&self, _uri: &StorageUri, _range: Option<ByteRange>) -> Result<ReadRange> {
        panic!("direct play should use stream_range instead of read_range");
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        let bytes = match range {
            Some(range) => {
                let start = range.offset as usize;
                let end = range
                    .length
                    .map(|length| start + length as usize)
                    .unwrap_or(self.bytes.len());
                self.bytes[start..end].to_vec()
            }
            None => self.bytes.clone(),
        };

        Ok(ReadStream::from_bytes(uri.clone(), range, bytes))
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Ok(String::new())
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(TaruError::Unsupported("test backend is read-only"))
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let path = taru_vfs::deterministic_stage_path(
            &request.root,
            &request.uri,
            Some("remote-fingerprint"),
        )?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| TaruError::Storage {
                    uri: parent.display().to_string(),
                    message: format!("failed to create test staging directory: {err}"),
                })?;
        }
        tokio::fs::write(&path, &self.bytes)
            .await
            .map_err(|err| TaruError::Storage {
                uri: path.display().to_string(),
                message: format!("failed to write test staging file: {err}"),
            })?;

        Ok(StagedFile {
            uri: request.uri,
            path,
            len: Some(self.bytes.len() as u64),
            etag: Some("etag-remote".to_owned()),
            fingerprint: Some("remote-fingerprint".to_owned()),
            reused: false,
        })
    }
}

struct MockWebDavServer {
    addr: std::net::SocketAddr,
}

impl MockWebDavServer {
    async fn start() -> Self {
        let router = Router::new().route("/{*path}", any(mock_webdav_handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self { addr }
    }

    fn base_url(&self) -> String {
        format!("http://{}/dav", self.addr)
    }
}

async fn mock_webdav_handler(method: Method, uri: axum::http::Uri) -> Response {
    let path = uri.path();
    if method.as_str() == "PROPFIND" {
        if path.ends_with("/Movies/") || path.ends_with("/Movies") {
            return (
                AxumStatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                mock_multistatus(&[
                    MockWebDavFixture {
                        href: "/dav/Movies/",
                        collection: true,
                        len: None,
                        etag: None,
                    },
                    MockWebDavFixture {
                        href: "/dav/Movies/Demo.mkv",
                        collection: false,
                        len: Some(4),
                        etag: Some("etag-demo"),
                    },
                    MockWebDavFixture {
                        href: "/dav/Movies/Demo.nfo",
                        collection: false,
                        len: Some(40),
                        etag: Some("etag-demo-nfo"),
                    },
                ]),
            )
                .into_response();
        }

        if path.ends_with("/Movies/Demo.mkv") {
            return (
                AxumStatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                mock_multistatus(&[MockWebDavFixture {
                    href: "/dav/Movies/Demo.mkv",
                    collection: false,
                    len: Some(4),
                    etag: Some("etag-demo"),
                }]),
            )
                .into_response();
        }

        if path.ends_with("/Movies/Demo.nfo") {
            return (
                AxumStatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                mock_multistatus(&[MockWebDavFixture {
                    href: "/dav/Movies/Demo.nfo",
                    collection: false,
                    len: Some(40),
                    etag: Some("etag-demo-nfo"),
                }]),
            )
                .into_response();
        }
    }

    if method == Method::GET && path.ends_with("/Movies/Demo.mkv") {
        return (AxumStatusCode::OK, [(header::CONTENT_LENGTH, "4")], "demo").into_response();
    }
    if method == Method::GET && path.ends_with("/Movies/Demo.nfo") {
        return (
            AxumStatusCode::OK,
            [(header::CONTENT_LENGTH, "40")],
            "<movie><title>Remote NFO</title></movie>",
        )
            .into_response();
    }

    AxumStatusCode::NOT_FOUND.into_response()
}

struct MockWebDavFixture {
    href: &'static str,
    collection: bool,
    len: Option<u64>,
    etag: Option<&'static str>,
}

fn mock_multistatus(fixtures: &[MockWebDavFixture]) -> String {
    let responses = fixtures
            .iter()
            .map(|fixture| {
                let resourcetype = if fixture.collection {
                    "<D:resourcetype><D:collection/></D:resourcetype>"
                } else {
                    "<D:resourcetype/>"
                };
                let len = fixture
                    .len
                    .map(|len| format!("<D:getcontentlength>{len}</D:getcontentlength>"))
                    .unwrap_or_default();
                let etag = fixture
                    .etag
                    .map(|etag| format!("<D:getetag>\"{etag}\"</D:getetag>"))
                    .unwrap_or_default();
                format!(
                    r#"<D:response><D:href>{}</D:href><D:propstat><D:prop>{resourcetype}{len}{etag}</D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#,
                    fixture.href
                )
            })
            .collect::<String>();

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:">{responses}</D:multistatus>"#
    )
}

fn fake_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let content =
            "#!/bin/sh\nfor arg do out=\"$arg\"; done\nprintf remuxed > \"$out\"\nexit 0\n";
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
        content.push_str("exit /b 0\r\n");
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_failing_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let content = "#!/bin/sh\necho remux failed >&2\nexit 7\n";
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("echo remux failed 1>&2\r\n");
        content.push_str("exit /b 7\r\n");
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    hls_ffmpeg_script(root, name, true)
}

fn fake_failing_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    hls_ffmpeg_script(root, name, false)
}

fn hls_ffmpeg_script(root: &Path, name: &str, success: bool) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        content.push_str("for arg do out=\"$arg\"; done\n");
        content.push_str("dir=$(dirname \"$out\")\n");
        content.push_str("mkdir -p \"$dir\"\n");
        if success {
            content.push_str(
                "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
            );
            content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
            content.push_str("exit 0\n");
        } else {
            content.push_str("printf partial > \"$out\"\n");
            content.push_str("printf hls-failed >&2\n");
            content.push_str("exit 42\n");
        }
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
        content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
        if success {
            content.push_str(">\"%out%\" echo #EXTM3U\r\n");
            content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
            content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
            content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
            content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
            content.push_str("exit /b 0\r\n");
        } else {
            content.push_str("<nul set /p dummy=partial>\"%out%\"\r\n");
            content.push_str("echo hls-failed 1>&2\r\n");
            content.push_str("exit /b 42\r\n");
        }
        fs::write(&path, content).unwrap();
        path
    }
}

async fn remux_app_with_source(
    ffmpeg_path: PathBuf,
) -> (tempfile::TempDir, TaruApp, SqliteStore, MediaSource) {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    let staging_root = temp.path().join("cache").join("remux");
    fs::create_dir_all(&library_root).unwrap();
    fs::write(library_root.join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path,
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: staging_root,
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
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
        library_id,
        item_id: item.id,
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![
                    MediaStreamInfo {
                        index: 0,
                        kind: MediaStreamKind::Video,
                        codec: Some("h264".to_owned()),
                        language: None,
                        duration_ms: None,
                        bit_rate: None,
                        width: Some(1920),
                        height: Some(1080),
                        channels: None,
                        sample_rate: None,
                    },
                    MediaStreamInfo {
                        index: 1,
                        kind: MediaStreamKind::Audio,
                        codec: Some("aac".to_owned()),
                        language: None,
                        duration_ms: None,
                        bit_rate: None,
                        width: None,
                        height: None,
                        channels: Some(2),
                        sample_rate: Some(48_000),
                    },
                ],
            },
        )
        .await
        .unwrap();

    (temp, app, store, source)
}
