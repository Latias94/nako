use super::super::storage::{
    EnqueueVfsCacheRepairTargetOutcome, RetryVfsCacheRepairJobRequest,
    VfsCacheRepairAutomationEnqueueOutcome, VfsCacheRepairJobSummary,
};
use super::*;
use crate::app::jobs::LibraryScanScheduleOutcome;
use nako_core::{
    JobKind, JobListFilter, JobPriority, JobRepository, JobStatus, NewVfsCacheFailure,
    StorageBackendHealthRecord, StorageBackendHealthRepository, StorageBackendHealthStatus,
    StorageCircuitBreakerState, StorageErrorKind, StorageFailureClass,
    VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS, VfsCacheFailure, VfsCacheFailureAuthority,
    VfsCacheOperation, VfsCacheRepairJobAction, VfsCacheRepairJobInput, VfsCacheRepository,
    VfsCachedObject, VfsCachedObjectKind, vfs_cache_repair_uri_digest,
};
use nako_library::SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS;
use nako_vfs::{
    CachedStorageBackend, ObjectCacheState, StorageApplyKind, StorageApplyRequest,
    StorageCleanupRequest, StorageLinkKind, StorageLinkPlanRequest, VfsCacheOptions,
};

#[tokio::test]
async fn webdav_preview_config_builds_scanner_backend() {
    let server = MockWebDavServer::start().await;
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
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: nako_core::LibraryPreset::Movies,
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let backend = app
        .storage()
        .backend_for_library_root(&library)
        .await
        .unwrap();
    let scanner = nako_library::VfsLibraryScanner::new(backend);
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
                    id: local_library_id,
                    name: "Local Movies".to_owned(),
                    root: temp.path().join("movies"),
                    preset: nako_core::LibraryPreset::Movies,
                    webdav: None,
                },
                LocalLibraryConfig {
                    id: remote_library_id,
                    name: "Remote Movies".to_owned(),
                    root: temp.path().join("unused-local-root"),
                    preset: nako_core::LibraryPreset::Movies,
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
    let libraries = app
        .library()
        .list_libraries(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(libraries.libraries.len(), 2);
    assert!(
        libraries
            .libraries
            .iter()
            .any(|library| library.id == local_library_id.to_string()
                && library.roots == vec!["local:///"])
    );
    assert!(libraries.libraries.iter().any(|library| {
        library.id == remote_library_id.to_string() && library.roots == vec!["webdav:///Movies"]
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
        .playback()
        .plan_direct_play(source.id, DirectPlayRangeRequest::None)
        .await
        .unwrap();

    assert_eq!(plan.response.total_len, 4);
    let DirectPlaySourceBody::Stream(_) = &plan.body else {
        panic!("expected remote direct play to use the configured WebDAV backend");
    };
}

#[tokio::test]
async fn storage_diagnostics_lists_reconciled_libraries_missing_from_config() {
    let temp = tempfile::tempdir().unwrap();
    let configured_root = temp.path().join("movies");
    fs::create_dir_all(&configured_root).unwrap();
    let configured_id = LibraryId::new();
    let retained_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store
        .upsert_library(&Library {
            id: retained_id,
            name: "Retained Historical Library".to_owned(),
            roots: vec!["local:///Retained".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
                id: configured_id,
                name: "Configured Movies".to_owned(),
                root: configured_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            }],
        },
        store.clone(),
    )
    .await
    .unwrap();

    let diagnostics = app.storage().list_storage_backend_diagnostics().await;

    assert_eq!(diagnostics.backends.len(), 2);
    assert!(diagnostics.backends.iter().any(|backend| {
        backend.library_id == configured_id
            && backend.status == nako_api::admin::StorageBackendStatus::Ready
    }));
    let retained = diagnostics
        .backends
        .iter()
        .find(|backend| backend.library_id == retained_id)
        .expect("retained library diagnostic");
    assert_eq!(retained.library_name, "Retained Historical Library");
    assert_eq!(retained.root_uri, "local:///Retained");
    assert_eq!(
        retained.status,
        nako_api::admin::StorageBackendStatus::Unavailable
    );
    assert_eq!(
        retained.reason.as_deref(),
        Some("configured library backend was not found")
    );
}

#[tokio::test]
async fn storage_health_records_runtime_updates_and_rejects_durable_circuit() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Remote-like Movies".to_owned(),
        root: root.clone(),
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let first_backend = Arc::new(StorageHealthCountingBackend::new(StorageErrorKind::Timeout));
    app.storage()
        .replace_backend_for_test(library_config.clone(), first_backend.clone())
        .await;
    let uri = StorageUri::parse("local:///Demo.mkv").unwrap();
    let backend = app
        .storage()
        .backend_for_library_root(&library)
        .await
        .unwrap();

    let first = backend.stat(&uri).await.unwrap_err();

    assert_eq!(
        first.storage_failure_class(),
        Some(StorageFailureClass::Timeout)
    );
    assert_eq!(first_backend.stat_calls.load(Ordering::SeqCst), 1);
    let backend_key = format!("library:{library_id}:local");
    let record = store
        .get_storage_backend_health(&backend_key)
        .await
        .unwrap()
        .expect("runtime storage failure should persist backend health");
    assert_eq!(record.backend_key, backend_key);
    assert_eq!(record.library_id, Some(library_id));
    assert_eq!(record.scheme, "local");
    assert_eq!(record.status, StorageBackendHealthStatus::Unavailable);
    assert_eq!(
        record.circuit_breaker_state,
        StorageCircuitBreakerState::Open
    );
    assert_eq!(record.consecutive_failures, 1);
    assert_eq!(
        record.last_failure_class,
        Some(StorageFailureClass::Timeout)
    );
    assert_eq!(
        record.last_failure_safe_message.as_deref(),
        Some("storage timeout")
    );
    assert!(record.last_failure_at_ms.is_some());
    assert!(record.circuit_opened_at_ms.is_some());
    assert!(record.backoff_until_ms.is_some());

    let restarted_backend = Arc::new(StorageHealthCountingBackend::new(StorageErrorKind::Timeout));
    app.storage()
        .replace_backend_for_test(library_config.clone(), restarted_backend.clone())
        .await;
    let restarted = app
        .storage()
        .backend_for_library_root(&library)
        .await
        .unwrap();

    let target_uri = StorageUri::parse("local:///Target.mkv").unwrap();
    let rejected = restarted.stat(&uri).await.unwrap_err();

    assert_storage_rate_limited(rejected);
    assert_eq!(restarted_backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_storage_rate_limited(restarted.write_string(&uri, "blocked").await.unwrap_err());
    assert_storage_rate_limited(
        restarted
            .write(StorageWriteRequest::direct(uri.clone(), "blocked"))
            .await
            .unwrap_err(),
    );
    assert_storage_rate_limited(
        restarted
            .plan_link(StorageLinkPlanRequest::new(
                uri.clone(),
                target_uri.clone(),
                StorageLinkKind::Hard,
            ))
            .await
            .unwrap_err(),
    );
    assert_storage_rate_limited(
        restarted
            .apply(StorageApplyRequest::new(
                uri.clone(),
                target_uri.clone(),
                StorageApplyKind::Copy,
            ))
            .await
            .unwrap_err(),
    );
    assert_storage_rate_limited(
        restarted
            .cleanup(StorageCleanupRequest::new(target_uri.clone()))
            .await
            .unwrap_err(),
    );
    assert_storage_rate_limited(
        restarted
            .restore(StorageRestoreRequest::new(uri.clone(), target_uri))
            .await
            .unwrap_err(),
    );
    assert_eq!(
        restarted_backend.write_string_calls.load(Ordering::SeqCst),
        0
    );
    assert_eq!(restarted_backend.write_calls.load(Ordering::SeqCst), 0);
    assert_eq!(restarted_backend.plan_link_calls.load(Ordering::SeqCst), 0);
    assert_eq!(restarted_backend.apply_calls.load(Ordering::SeqCst), 0);
    assert_eq!(restarted_backend.cleanup_calls.load(Ordering::SeqCst), 0);
    assert_eq!(restarted_backend.restore_calls.load(Ordering::SeqCst), 0);

    store
        .clear_storage_backend_health(&backend_key, record.updated_at_ms + 1)
        .await
        .unwrap();
    fs::write(root.join("Demo.mkv"), b"demo").unwrap();
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(LocalFsBackend::new(&root).unwrap()),
        )
        .await;
    let healthy = app
        .storage()
        .backend_for_library_root(&library)
        .await
        .unwrap();

    healthy.stat(&uri).await.unwrap();

    let recovered = store
        .get_storage_backend_health(&backend_key)
        .await
        .unwrap()
        .expect("successful storage operation should persist healthy state");
    assert_eq!(recovered.status, StorageBackendHealthStatus::Healthy);
    assert_eq!(
        recovered.circuit_breaker_state,
        StorageCircuitBreakerState::Closed
    );
    assert_eq!(recovered.consecutive_failures, 0);
    assert!(recovered.last_success_at_ms.is_some());
    assert_eq!(recovered.last_failure_at_ms, None);
    assert_eq!(recovered.last_failure_class, None);
    assert_eq!(recovered.last_failure_safe_message, None);
    assert_eq!(recovered.circuit_opened_at_ms, None);
    assert_eq!(recovered.backoff_until_ms, None);
}

#[tokio::test]
async fn vfs_cache_refresh_action_refreshes_retryable_stat_failure_and_resolves_preview() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Remote-like Movies".to_owned(),
        root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(CachedStorageBackend::with_options(
                backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    let failed_at_ms = 1_000;
    let backend_key = format!("library:{library_id}:local");
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(library_id, backend_key.clone()),
        })
        .await
        .unwrap();
    store
        .upsert_storage_backend_health(StorageBackendHealthRecord {
            backend_key: backend_key.clone(),
            library_id: Some(library_id),
            scheme: "local".to_owned(),
            status: StorageBackendHealthStatus::Unavailable,
            circuit_breaker_state: StorageCircuitBreakerState::Open,
            consecutive_failures: 1,
            last_success_at_ms: None,
            last_failure_at_ms: Some(failed_at_ms),
            last_failure_class: Some(StorageFailureClass::Timeout),
            last_failure_safe_message: Some("storage timeout".to_owned()),
            circuit_opened_at_ms: Some(failed_at_ms),
            backoff_until_ms: Some(i64::MAX),
            updated_at_ms: failed_at_ms,
        })
        .await
        .unwrap();

    let preview = app
        .storage()
        .latest_vfs_cache_repair_diagnostic()
        .await
        .unwrap()
        .expect("retryable preview");
    let report = app
        .storage()
        .refresh_latest_vfs_cache_repair()
        .await
        .unwrap();
    let refreshed = store
        .get_vfs_cache_object("local:///Movies/Demo.mkv")
        .await
        .unwrap()
        .expect("refreshed cache object");
    let resolved = app
        .storage()
        .latest_vfs_cache_repair_diagnostic()
        .await
        .unwrap();
    let health = store
        .get_storage_backend_health(&backend_key)
        .await
        .unwrap()
        .expect("refresh success should record backend health");

    assert_eq!(
        preview.recommended_action,
        nako_vfs::VfsCacheRepairAction::RefreshCache
    );
    assert_eq!(report.operation, VfsCacheOperation::Stat);
    assert_eq!(
        report.refresh.cache.as_ref().map(|cache| cache.state),
        Some(ObjectCacheState::Fresh)
    );
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 1);
    assert!(refreshed.fetched_at_ms >= failed_at_ms);
    assert!(resolved.is_none());
    assert_eq!(health.status, StorageBackendHealthStatus::Healthy);
    assert_eq!(
        health.circuit_breaker_state,
        StorageCircuitBreakerState::Closed
    );
}

#[tokio::test]
async fn vfs_cache_target_refresh_action_refreshes_selected_failure_not_latest() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Remote-like Movies".to_owned(),
        root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(CachedStorageBackend::with_options(
                backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    let backend_key = format!("library:{library_id}:local");
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Latest.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 2_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(library_id, backend_key.clone()),
        })
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Selected.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(library_id, backend_key),
        })
        .await
        .unwrap();

    let targets = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap();
    let selected = targets
        .iter()
        .find(|target| target.failed_at_ms == 1_000)
        .expect("selected target");
    let report = app
        .storage()
        .refresh_vfs_cache_repair_target(&selected.target_ref)
        .await
        .unwrap();

    assert_eq!(report.operation, VfsCacheOperation::Stat);
    assert_eq!(
        report.refresh.cache.as_ref().map(|cache| cache.state),
        Some(ObjectCacheState::Fresh)
    );
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 1);
    assert!(
        store
            .get_vfs_cache_object("local:///Movies/Selected.mkv")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        store
            .get_vfs_cache_object("local:///Movies/Latest.mkv")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn vfs_cache_refresh_action_rejects_non_refresh_recommendation_without_backend_call() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Remote-like Movies".to_owned(),
        root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(CachedStorageBackend::with_options(
                backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage permission failure".to_owned(),
            authority: VfsCacheFailureAuthority::default(),
        })
        .await
        .unwrap();

    let err = app
        .storage()
        .refresh_latest_vfs_cache_repair()
        .await
        .unwrap_err();

    assert!(matches!(err, NakoError::InvalidInput { .. }));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_target_refresh_action_rejects_non_refresh_recommendation_without_backend_call() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Movies".to_owned(),
        root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(CachedStorageBackend::with_options(
                backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage permission failure".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        })
        .await
        .unwrap();
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("target");

    let err = app
        .storage()
        .refresh_vfs_cache_repair_target(&target.target_ref)
        .await
        .unwrap_err();

    assert!(matches!(err, NakoError::InvalidInput { .. }));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_refresh_action_uses_authority_for_ambiguous_local_repair_target() {
    let temp = tempfile::tempdir().unwrap();
    let movies_root = temp.path().join("movies");
    let shows_root = temp.path().join("shows");
    fs::create_dir_all(&movies_root).unwrap();
    fs::create_dir_all(&shows_root).unwrap();
    let movies_library_id = LibraryId::new();
    let shows_library_id = LibraryId::new();
    let movies_config = LocalLibraryConfig {
        id: movies_library_id,
        name: "Movies".to_owned(),
        root: movies_root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let shows_config = LocalLibraryConfig {
        id: shows_library_id,
        name: "Shows".to_owned(),
        root: shows_root,
        preset: nako_core::LibraryPreset::Tv,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![movies_config.clone(), shows_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let movies_backend = Arc::new(CacheRefreshCountingBackend::new());
    let shows_backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            movies_config,
            Arc::new(CachedStorageBackend::with_options(
                movies_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    app.storage()
        .replace_backend_for_test(
            shows_config,
            Arc::new(CachedStorageBackend::with_options(
                shows_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                movies_library_id,
                format!("library:{movies_library_id}:local"),
            ),
        })
        .await
        .unwrap();

    let report = app
        .storage()
        .refresh_latest_vfs_cache_repair()
        .await
        .unwrap();

    assert_eq!(report.operation, VfsCacheOperation::Stat);
    assert_eq!(
        report.refresh.cache.as_ref().map(|cache| cache.state),
        Some(ObjectCacheState::Fresh)
    );
    assert_eq!(movies_backend.stat_calls.load(Ordering::SeqCst), 1);
    assert_eq!(movies_backend.list_calls.load(Ordering::SeqCst), 0);
    assert_eq!(shows_backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(shows_backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_refresh_action_rejects_mismatched_authority_without_backend_call() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Movies".to_owned(),
        root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(CachedStorageBackend::with_options(
                backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:webdav"),
            ),
        })
        .await
        .unwrap();

    let err = app
        .storage()
        .refresh_latest_vfs_cache_repair()
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        NakoError::Conflict { message } if message == "latest VFS cache repair target authority does not match configured storage backend"
    ));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_refresh_action_rejects_ambiguous_local_repair_target_without_backend_call() {
    let temp = tempfile::tempdir().unwrap();
    let movies_root = temp.path().join("movies");
    let shows_root = temp.path().join("shows");
    fs::create_dir_all(&movies_root).unwrap();
    fs::create_dir_all(&shows_root).unwrap();
    let movies_library_id = LibraryId::new();
    let shows_library_id = LibraryId::new();
    let movies_config = LocalLibraryConfig {
        id: movies_library_id,
        name: "Movies".to_owned(),
        root: movies_root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let shows_config = LocalLibraryConfig {
        id: shows_library_id,
        name: "Shows".to_owned(),
        root: shows_root,
        preset: nako_core::LibraryPreset::Tv,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![movies_config.clone(), shows_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let movies_backend = Arc::new(CacheRefreshCountingBackend::new());
    let shows_backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            movies_config,
            Arc::new(CachedStorageBackend::with_options(
                movies_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    app.storage()
        .replace_backend_for_test(
            shows_config,
            Arc::new(CachedStorageBackend::with_options(
                shows_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Movies/Demo.mkv".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::default(),
        })
        .await
        .unwrap();

    let err = app
        .storage()
        .refresh_latest_vfs_cache_repair()
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        NakoError::Conflict { message } if message == "latest VFS cache repair target matches multiple configured storage backends"
    ));
    assert_eq!(movies_backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(movies_backend.list_calls.load(Ordering::SeqCst), 0);
    assert_eq!(shows_backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(shows_backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_automation_policy_disabled_blocks_refreshable() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;

    let report = app
        .storage()
        .plan_vfs_cache_repair_automation(VfsCacheRepairAutomationPolicy { enabled: false })
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(!report.policy.enabled);
    assert_eq!(report.total_unresolved_targets, 1);
    assert!(report.eligible_targets.is_empty());
    assert_eq!(report.blocked_targets.len(), 1);
    assert_eq!(
        report.blocked_targets[0].reason,
        VfsCacheRepairAutomationBlockReason::PolicyDisabled
    );
    assert_eq!(
        report.blocked_targets[0].target.repair.recommended_action,
        nako_vfs::VfsCacheRepairAction::RefreshCache
    );
    assert!(report.boundary.reads_repair_targets);
    assert!(!report.boundary.may_start_durable_jobs);
    assert!(!report.boundary.refreshes_vfs_cache);
    assert!(!report.boundary.changes_backend_configuration);
    assert!(!report.boundary.deletes_cache_entries);
    assert!(!report.boundary.writes_library_files);
    assert!(jobs.is_empty());
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_automation_policy_enabled_allows_refreshable() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;

    let report = app
        .storage()
        .plan_vfs_cache_repair_automation(VfsCacheRepairAutomationPolicy { enabled: true })
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(report.policy.enabled);
    assert_eq!(report.total_unresolved_targets, 1);
    assert_eq!(report.eligible_targets.len(), 1);
    assert!(report.blocked_targets.is_empty());
    assert_eq!(
        report.eligible_targets[0].target.repair.recommended_action,
        nako_vfs::VfsCacheRepairAction::RefreshCache
    );
    assert_eq!(report.eligible_targets[0].target.scheme, failure.scheme);
    assert_eq!(
        report.eligible_targets[0].target.operation,
        failure.operation
    );
    assert_eq!(
        report.eligible_targets[0].target.failed_at_ms,
        failure.failed_at_ms
    );
    assert_eq!(
        report.eligible_targets[0].target.failure_count,
        failure.failure_count
    );
    assert!(report.boundary.reads_repair_targets);
    assert!(report.boundary.may_start_durable_jobs);
    assert!(!report.boundary.refreshes_vfs_cache);
    assert!(!report.boundary.changes_backend_configuration);
    assert!(!report.boundary.deletes_cache_entries);
    assert!(!report.boundary.writes_library_files);
    assert!(jobs.is_empty());
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_automation_policy_enabled_blocks_backend_config() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Permission.safe_message())
            .await;

    let report = app
        .storage()
        .plan_vfs_cache_repair_automation(VfsCacheRepairAutomationPolicy { enabled: true })
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(report.policy.enabled);
    assert_eq!(report.total_unresolved_targets, 1);
    assert!(report.eligible_targets.is_empty());
    assert_eq!(report.blocked_targets.len(), 1);
    assert_eq!(
        report.blocked_targets[0].reason,
        VfsCacheRepairAutomationBlockReason::BackendConfigurationRequired
    );
    assert_eq!(
        report.blocked_targets[0].target.repair.recommended_action,
        nako_vfs::VfsCacheRepairAction::FixBackendConfiguration
    );
    assert!(report.boundary.reads_repair_targets);
    assert!(report.boundary.may_start_durable_jobs);
    assert!(!report.boundary.refreshes_vfs_cache);
    assert!(!report.boundary.changes_backend_configuration);
    assert!(!report.boundary.deletes_cache_entries);
    assert!(!report.boundary.writes_library_files);
    assert!(jobs.is_empty());
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_planners_share_unresolved_target_collection() {
    let (_temp, app, store, backend, unresolved) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let resolved = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Users/ExampleUser/Secret Path/Already Fresh.mkv?token=secret".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 2_000,
            error: StorageFailureClass::Unavailable.safe_message().to_owned(),
            authority: unresolved.authority.clone(),
        })
        .await
        .unwrap();
    store
        .upsert_vfs_cache_object(&VfsCachedObject {
            uri: resolved.uri.clone(),
            scheme: resolved.scheme.clone(),
            kind: VfsCachedObjectKind::File,
            len: Some(42),
            modified_at: None,
            etag: Some("safe-test-etag".to_owned()),
            fingerprint: Some("safe-test-fingerprint".to_owned()),
            capabilities_bits: 0,
            fetched_at_ms: resolved.failed_at_ms + 1,
            fresh_until_ms: resolved.failed_at_ms + 60_000,
        })
        .await
        .unwrap();

    let targets = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap();
    let remediation = app
        .storage()
        .plan_vfs_cache_repair_remediation()
        .await
        .unwrap();
    let automation = app
        .storage()
        .plan_vfs_cache_repair_automation(VfsCacheRepairAutomationPolicy { enabled: true })
        .await
        .unwrap();
    let refresh_group = remediation
        .action_groups
        .iter()
        .find(|group| group.action == nako_vfs::VfsCacheRepairAction::RefreshCache)
        .expect("refresh cache group");

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].failed_at_ms, unresolved.failed_at_ms);
    assert_eq!(remediation.total_unresolved_targets, targets.len() as u32);
    assert_eq!(refresh_group.count, targets.len() as u32);
    assert_eq!(refresh_group.sample_targets, targets);
    assert_eq!(automation.total_unresolved_targets, targets.len() as u32);
    assert_eq!(automation.eligible_targets.len(), targets.len());
    assert!(automation.blocked_targets.is_empty());
    assert_eq!(automation.eligible_targets[0].target, targets[0]);
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_automation_enqueue_disabled_does_not_create_jobs() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;

    let report = app
        .storage()
        .enqueue_vfs_cache_repair_automation(
            VfsCacheRepairAutomationPolicy { enabled: false },
            Some(JobPriority::High),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(!report.policy_report.policy.enabled);
    assert_eq!(report.policy_report.total_unresolved_targets, 1);
    assert!(report.policy_report.eligible_targets.is_empty());
    assert_eq!(report.policy_report.blocked_targets.len(), 1);
    assert_eq!(
        report.policy_report.blocked_targets[0].reason,
        VfsCacheRepairAutomationBlockReason::PolicyDisabled
    );
    assert!(report.jobs.is_empty());
    assert_eq!(report.enqueued_count, 0);
    assert_eq!(report.already_queued_count, 0);
    assert!(jobs.is_empty());
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_automation_enqueue_creates_safe_repair_job() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;

    let report = app
        .storage()
        .enqueue_vfs_cache_repair_automation(
            VfsCacheRepairAutomationPolicy { enabled: true },
            Some(JobPriority::High),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();
    let persisted = jobs.first().expect("persisted repair job");
    let input_json = persisted.input_json.as_deref().expect("job input");

    assert!(report.policy_report.policy.enabled);
    assert_eq!(report.policy_report.total_unresolved_targets, 1);
    assert_eq!(report.policy_report.eligible_targets.len(), 1);
    assert!(report.policy_report.blocked_targets.is_empty());
    assert_eq!(report.enqueued_count, 1);
    assert_eq!(report.already_queued_count, 0);
    assert_eq!(report.jobs.len(), 1);
    assert_eq!(
        report.jobs[0].outcome,
        VfsCacheRepairAutomationEnqueueOutcome::Enqueued
    );
    assert_eq!(report.jobs[0].job_id, persisted.id);
    assert_eq!(report.jobs[0].status, JobStatus::Queued);
    assert_eq!(report.jobs[0].priority, JobPriority::High);
    assert_eq!(
        report.jobs[0].resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(report.jobs[0].library_id, failure.authority.library_id);
    assert_eq!(report.jobs[0].source_id, None);
    assert_eq!(jobs.len(), 1);
    assert_eq!(persisted.kind, JobKind::VfsCacheRepair);
    assert_eq!(
        persisted.resource_class,
        VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS
    );
    assert_eq!(persisted.priority, JobPriority::High);
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("Secret Path"));
    assert!(!input_json.contains("ExampleUser"));
    assert!(!input_json.contains("token=secret"));
    assert!(!input_json.contains("local:///"));
    assert!(!input_json.contains("storage backend unavailable"));
}

#[tokio::test]
async fn vfs_cache_repair_automation_enqueue_reuses_existing_incomplete_job() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;

    let first = app
        .storage()
        .enqueue_vfs_cache_repair_automation(
            VfsCacheRepairAutomationPolicy { enabled: true },
            Some(JobPriority::Low),
        )
        .await
        .unwrap();
    let second = app
        .storage()
        .enqueue_vfs_cache_repair_automation(
            VfsCacheRepairAutomationPolicy { enabled: true },
            Some(JobPriority::High),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(first.enqueued_count, 1);
    assert_eq!(first.already_queued_count, 0);
    assert_eq!(second.enqueued_count, 0);
    assert_eq!(second.already_queued_count, 1);
    assert_eq!(second.jobs.len(), 1);
    assert_eq!(
        second.jobs[0].outcome,
        VfsCacheRepairAutomationEnqueueOutcome::AlreadyQueued
    );
    assert_eq!(second.jobs[0].job_id, first.jobs[0].job_id);
    assert_eq!(second.jobs[0].priority, JobPriority::Low);
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].priority, JobPriority::Low);
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_target_enqueue_persists_safe_job_input_without_refreshing() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");

    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::High))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) = outcome else {
        panic!("expected new VFS cache repair job");
    };
    let input_json = job.input_json.as_deref().expect("job input json");
    let input: VfsCacheRepairJobInput = serde_json::from_str(input_json).unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job.id);
    assert_eq!(job.kind, JobKind::VfsCacheRepair);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.resource_class, VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS);
    assert_eq!(job.priority, JobPriority::High);
    assert_eq!(job.library_id, failure.authority.library_id);
    assert_eq!(job.source_id, None);
    assert_eq!(input.action, VfsCacheRepairJobAction::RefreshCache);
    assert_eq!(input.source_scheme, failure.scheme);
    assert_eq!(input.operation, failure.operation);
    assert_eq!(input.failed_at_ms, failure.failed_at_ms);
    assert_eq!(input.failure_count, failure.failure_count);
    assert_eq!(input.uri_digest, vfs_cache_repair_uri_digest(&failure.uri));
    assert_eq!(input.authority, failure.authority);
    assert!(input.matches_failure(&failure));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("Secret Path"));
    assert!(!input_json.contains("ExampleUser"));
    assert!(!input_json.contains("token=secret"));
    assert!(!input_json.contains("local:///"));
    assert!(!input_json.contains("storage backend unavailable"));
}

#[tokio::test]
async fn vfs_cache_repair_target_enqueue_is_idempotent_for_incomplete_jobs() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");

    let first = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, None)
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(first_job) = first else {
        panic!("expected initial enqueue");
    };
    let second = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::High))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::AlreadyQueued(second_job) = second else {
        panic!("expected queued job to block duplicate enqueue");
    };
    assert_eq!(first_job.id, second_job.id);

    store.start_job(first_job.id).await.unwrap();
    let third = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::High))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::AlreadyQueued(third_job) = third else {
        panic!("expected running job to block duplicate enqueue");
    };
    assert_eq!(first_job.id, third_job.id);

    store.succeed_job(first_job.id, None).await.unwrap();
    let fourth = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, None)
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(fourth_job) = fourth else {
        panic!("expected terminal job not to block future enqueue");
    };
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_ne!(first_job.id, fourth_job.id);
    assert_eq!(jobs.len(), 2);
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_target_enqueue_finds_duplicate_beyond_first_job_page() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let duplicate_input = VfsCacheRepairJobInput::from_failure(&failure).unwrap();
    let duplicate_job = store
        .enqueue_job(new_vfs_cache_repair_job(&failure, duplicate_input))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    for index in 0..=PageRequest::MAX_LIMIT {
        let decoy_input = VfsCacheRepairJobInput::new(
            VfsCacheRepairJobAction::RefreshCache,
            failure.scheme.clone(),
            failure.operation,
            failure.failed_at_ms + i64::from(index) + 1,
            failure.failure_count,
            vfs_cache_repair_uri_digest(&format!("local:///Movies/Decoy-{index}.mkv")),
            failure.authority.clone(),
        )
        .unwrap();
        store
            .enqueue_job(new_vfs_cache_repair_job(&failure, decoy_input))
            .await
            .unwrap();
    }

    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, None)
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::AlreadyQueued(existing) = outcome else {
        panic!("expected duplicate beyond first job page to block enqueue");
    };
    let first_page = store
        .list_jobs(
            JobListFilter {
                status: Some(JobStatus::Queued),
                kind: Some(JobKind::VfsCacheRepair),
                resource_class: Some(VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned()),
                library_id: failure.authority.library_id,
                source_id: None,
            },
            PageRequest::new(PageRequest::MAX_LIMIT, 0),
        )
        .await
        .unwrap();
    let second_page = store
        .list_jobs(
            JobListFilter {
                status: Some(JobStatus::Queued),
                kind: Some(JobKind::VfsCacheRepair),
                resource_class: Some(VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned()),
                library_id: failure.authority.library_id,
                source_id: None,
            },
            PageRequest::new(PageRequest::MAX_LIMIT, u64::from(PageRequest::MAX_LIMIT)),
        )
        .await
        .unwrap();

    assert_eq!(existing.id, duplicate_job.id);
    assert!(!first_page.iter().any(|job| job.id == duplicate_job.id));
    assert!(second_page.iter().any(|job| job.id == duplicate_job.id));
    assert_eq!(first_page.len(), PageRequest::MAX_LIMIT as usize);
    assert_eq!(second_page.len(), 2);
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_target_enqueue_rejects_non_refresh_target_without_backend_call() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Permission.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");

    let err = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::High))
        .await
        .unwrap_err();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(matches!(
        err,
        NakoError::InvalidInput { message }
            if message == "selected VFS cache repair target diagnostic does not recommend durable refresh_cache"
    ));
    assert_eq!(jobs.len(), 0);
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn vfs_cache_repair_job_executor_refreshes_target_and_persists_safe_summary() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::High))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) = outcome else {
        panic!("expected queued VFS cache repair job");
    };

    let output = app
        .storage()
        .execute_vfs_cache_repair_job(job.id)
        .await
        .unwrap();
    let loaded = store.get_job(job.id).await.unwrap().unwrap();
    let summary_json = loaded.summary_json.as_deref().expect("job summary");
    let summary: VfsCacheRepairJobSummary = serde_json::from_str(summary_json).unwrap();

    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(loaded.status, JobStatus::Succeeded);
    assert_eq!(summary, output.summary);
    assert_eq!(summary.action, nako_vfs::VfsCacheRepairAction::RefreshCache);
    assert_eq!(summary.source_scheme, "local");
    assert_eq!(summary.operation, VfsCacheOperation::Stat);
    assert_eq!(
        summary.classification,
        nako_vfs::VfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert_eq!(
        summary.failure_class,
        Some(StorageFailureClass::Unavailable)
    );
    assert_eq!(summary.failed_at_ms, failure.failed_at_ms);
    assert_eq!(summary.failure_count, failure.failure_count);
    assert_eq!(summary.refreshed_cache_state, Some(ObjectCacheState::Fresh));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 1);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
    assert!(
        store
            .get_vfs_cache_object(&failure.uri)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        nako_core::JobLeaseRepository::claim_next_job_lease(
            &store,
            nako_core::JobLeaseClaimRequest {
                worker_id: nako_core::JobWorkerId::new(),
                lease_duration_ms: 10_000,
                filter: nako_core::JobLeaseClaimFilter {
                    job_id: Some(job.id),
                    ..nako_core::JobLeaseClaimFilter::default()
                },
            },
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(!summary_json.contains("Hidden Movie"));
    assert!(!summary_json.contains("Secret Path"));
    assert!(!summary_json.contains("ExampleUser"));
    assert!(!summary_json.contains("token=secret"));
    assert!(!summary_json.contains("local:///"));
    assert!(!summary_json.contains("storage backend unavailable"));
    assert!(!summary_json.contains("safe-test-etag"));
    assert!(!summary_json.contains("safe-test-fingerprint"));
}

#[tokio::test]
async fn vfs_cache_repair_job_executor_rejects_stale_input_without_backend_call() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let input = VfsCacheRepairJobInput::from_failure(&failure).unwrap();
    let job = store
        .enqueue_job(new_vfs_cache_repair_job(&failure, input))
        .await
        .unwrap();
    store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: failure.uri.clone(),
            scheme: failure.scheme.clone(),
            operation: failure.operation,
            failed_at_ms: failure.failed_at_ms + 1_000,
            error: StorageFailureClass::Unavailable.safe_message().to_owned(),
            authority: failure.authority.clone(),
        })
        .await
        .unwrap();

    let err = app
        .storage()
        .execute_vfs_cache_repair_job(job.id)
        .await
        .unwrap_err();
    let loaded = store.get_job(job.id).await.unwrap().unwrap();

    assert!(matches!(
        err,
        NakoError::NotFound { entity, id }
            if entity == "vfs_cache_repair_target" && id == "job_input"
    ));
    assert_eq!(loaded.status, JobStatus::Failed);
    assert_eq!(
        loaded.error.as_deref(),
        Some("not found: vfs_cache_repair_target job_input")
    );
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);

    let error = loaded.error.unwrap_or_default();
    assert!(!error.contains("Hidden Movie"));
    assert!(!error.contains("Secret Path"));
    assert!(!error.contains("ExampleUser"));
    assert!(!error.contains("token=secret"));
    assert!(!error.contains("local:///"));
}

#[tokio::test]
async fn vfs_cache_repair_job_executor_redacts_backend_failure() {
    let (temp, app, store, _backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, None)
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) = outcome else {
        panic!("expected queued VFS cache repair job");
    };
    let failing_backend = Arc::new(StorageHealthCountingBackend::new(StorageErrorKind::Network));
    let library_id = failure.authority.library_id.expect("library authority");
    app.storage()
        .replace_backend_for_test(
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            Arc::new(CachedStorageBackend::with_options(
                failing_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;

    let err = app
        .storage()
        .execute_vfs_cache_repair_job(job.id)
        .await
        .unwrap_err();
    let loaded = store.get_job(job.id).await.unwrap().unwrap();
    let persisted_error = loaded.error.as_deref().expect("durable job error");

    assert!(matches!(err, NakoError::Storage { uri, kind, message }
        if uri == "local://<redacted>"
            && kind == StorageErrorKind::Network
            && message == StorageFailureClass::Unavailable.safe_message()
    ));
    assert_eq!(loaded.status, JobStatus::Failed);
    assert_eq!(
        persisted_error,
        "storage error at local://<redacted>: storage backend unavailable"
    );
    assert_eq!(failing_backend.stat_calls.load(Ordering::SeqCst), 1);
    assert!(!persisted_error.contains("Hidden Movie"));
    assert!(!persisted_error.contains("Secret Path"));
    assert!(!persisted_error.contains("ExampleUser"));
    assert!(!persisted_error.contains("token=secret"));
    assert!(!persisted_error.contains("local:///"));
    assert!(!persisted_error.contains("storage health test failure"));
}

#[tokio::test]
async fn vfs_cache_repair_retry_creates_safe_delayed_retry_job() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let input = VfsCacheRepairJobInput::from_failure(&failure).unwrap();
    let failed = fail_vfs_cache_repair_retry_source_job(
        &store,
        new_vfs_cache_repair_job(&failure, input.clone()),
    )
    .await;

    let retry = app
        .storage()
        .retry_vfs_cache_repair_job(RetryVfsCacheRepairJobRequest {
            job_id: failed.id,
            max_attempts: Some(3),
            next_attempt_at: Some("9999-01-01T08:00:00+08:00".to_owned()),
        })
        .await
        .unwrap();
    let retry_input_json = retry.input_json.as_deref().expect("retry input json");
    let retry_input: VfsCacheRepairJobInput = serde_json::from_str(retry_input_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(retry.id),
                kind: Some(JobKind::VfsCacheRepair),
                resource_class: Some(VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned()),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();

    assert_ne!(retry.id, failed.id);
    assert_eq!(retry.kind, JobKind::VfsCacheRepair);
    assert_eq!(retry.status, JobStatus::Queued);
    assert_eq!(retry.resource_class, VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS);
    assert_eq!(retry.priority, JobPriority::Normal);
    assert_eq!(retry.library_id, failure.authority.library_id);
    assert_eq!(retry.source_id, None);
    assert_eq!(retry.input_json, failed.input_json);
    assert_eq!(retry_input, input);
    assert_eq!(retry.attempt, 2);
    assert_eq!(retry.max_attempts, 3);
    assert_eq!(retry.retry_of_job_id, Some(failed.id));
    assert_eq!(
        retry.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    assert!(claim.is_none(), "future retry must not be claimable");
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
    assert_vfs_cache_repair_retry_payload_redacted(retry_input_json);
}

#[tokio::test]
async fn vfs_cache_repair_retry_scheduler_executes_due_job() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let input = VfsCacheRepairJobInput::from_failure(&failure).unwrap();
    let failed =
        fail_vfs_cache_repair_retry_source_job(&store, new_vfs_cache_repair_job(&failure, input))
            .await;

    let retry = app
        .storage()
        .retry_vfs_cache_repair_job(RetryVfsCacheRepairJobRequest {
            job_id: failed.id,
            max_attempts: Some(3),
            next_attempt_at: Some("0001-01-01T00:00:00Z".to_owned()),
        })
        .await
        .unwrap();

    let schedule = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_vfs_cache_repair_runtime_job(&app).await;
    let loaded = store.get_job(retry.id).await.unwrap().unwrap();
    let summary_json = loaded.summary_json.as_deref().expect("job summary");
    let summary: VfsCacheRepairJobSummary = serde_json::from_str(summary_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(retry.id),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();

    assert_eq!(schedule, LibraryScanScheduleOutcome::Scheduled(retry.id));
    assert_eq!(loaded.status, JobStatus::Succeeded);
    assert_eq!(loaded.retry_of_job_id, Some(failed.id));
    assert_eq!(loaded.next_attempt_at, None);
    assert_eq!(summary.action, nako_vfs::VfsCacheRepairAction::RefreshCache);
    assert_eq!(summary.source_scheme, "local");
    assert_eq!(summary.operation, VfsCacheOperation::Stat);
    assert_eq!(
        summary.classification,
        nako_vfs::VfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert_eq!(
        summary.failure_class,
        Some(StorageFailureClass::Unavailable)
    );
    assert_eq!(summary.failed_at_ms, failure.failed_at_ms);
    assert_eq!(summary.failure_count, failure.failure_count);
    assert_eq!(summary.refreshed_cache_state, Some(ObjectCacheState::Fresh));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 1);
    assert_eq!(backend.list_calls.load(Ordering::SeqCst), 0);
    assert!(claim.is_none());
    assert_vfs_cache_repair_retry_payload_redacted(summary_json);
    assert!(!summary_json.contains("safe-test-etag"));
    assert!(!summary_json.contains("safe-test-fingerprint"));
}

#[tokio::test]
async fn vfs_cache_repair_retry_rejects_invalid_states_without_retry_or_leak() {
    let (_temp, app, store, _backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let input = VfsCacheRepairJobInput::from_failure(&failure).unwrap();
    let queued = store
        .enqueue_job(new_vfs_cache_repair_job(&failure, input.clone()))
        .await
        .unwrap();
    let exhausted = fail_vfs_cache_repair_retry_source_job(
        &store,
        new_vfs_cache_repair_job(&failure, input.clone()),
    )
    .await;
    let wrong_kind = fail_vfs_cache_repair_retry_source_job(
        &store,
        NewJob {
            kind: JobKind::LibraryScan,
            ..new_vfs_cache_repair_job(&failure, input.clone())
        },
    )
    .await;
    let malformed_input = fail_vfs_cache_repair_retry_source_job(
        &store,
        NewJob {
            input_json: Some(
                "{\"uri\":\"local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv?token=secret\""
                    .to_owned(),
            ),
            ..new_vfs_cache_repair_job(&failure, input.clone())
        },
    )
    .await;
    let stale = fail_vfs_cache_repair_retry_source_job(
        &store,
        NewJob {
            input_json: Some(
                serde_json::to_string(
                    &VfsCacheRepairJobInput::new(
                        VfsCacheRepairJobAction::RefreshCache,
                        failure.scheme.clone(),
                        failure.operation,
                        failure.failed_at_ms + 10_000,
                        failure.failure_count,
                        vfs_cache_repair_uri_digest(
                            "local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv?token=secret",
                        ),
                        failure.authority.clone(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
            ..new_vfs_cache_repair_job(&failure, input)
        },
    )
    .await;

    let queued_message =
        retry_vfs_cache_repair_job_expect_err_without_retry(&app, &store, queued.id, None).await;
    let exhausted_message =
        retry_vfs_cache_repair_job_expect_err_without_retry(&app, &store, exhausted.id, Some(1))
            .await;
    let wrong_kind_message =
        retry_vfs_cache_repair_job_expect_err_without_retry(&app, &store, wrong_kind.id, None)
            .await;
    let malformed_input_message =
        retry_vfs_cache_repair_job_expect_err_without_retry(&app, &store, malformed_input.id, None)
            .await;
    let invalid_next_attempt_at_message =
        retry_vfs_cache_repair_job_request_expect_err_without_retry(
            &app,
            &store,
            RetryVfsCacheRepairJobRequest {
                job_id: stale.id,
                max_attempts: Some(3),
                next_attempt_at: Some(
                    "local:///Users/ExampleUser/Secret Path/not-a-time?token=secret".to_owned(),
                ),
            },
        )
        .await;
    let stale_message =
        retry_vfs_cache_repair_job_expect_err_without_retry(&app, &store, stale.id, None).await;

    assert_eq!(
        queued_message,
        "conflict: only failed VFS cache repair jobs can be retried"
    );
    assert_eq!(
        exhausted_message,
        "conflict: job retry attempts are exhausted"
    );
    assert_eq!(
        wrong_kind_message,
        "invalid input: job is not a VFS cache repair job"
    );
    assert_eq!(
        malformed_input_message,
        "invalid input: VFS cache repair job input is invalid"
    );
    assert_eq!(
        invalid_next_attempt_at_message,
        "invalid input: VFS cache repair retry next_attempt_at must be an RFC3339 timestamp"
    );
    assert_eq!(
        stale_message,
        "not found: vfs_cache_repair_target job_input"
    );
}

#[tokio::test]
async fn vfs_cache_repair_scheduler_executes_queued_job_and_persists_safe_summary() {
    let (_temp, app, store, backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let stat_started = Arc::new(Notify::new());
    let release_stat = Arc::new(Notify::new());
    let gated_backend = Arc::new(CacheRefreshCountingBackend::with_stat_gate(
        stat_started.clone(),
        release_stat.clone(),
    ));
    let library_id = failure.authority.library_id.expect("library authority");
    app.storage()
        .replace_backend_for_test(
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: _temp.path().join("movies"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            Arc::new(CachedStorageBackend::with_options(
                gated_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::High))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) = outcome else {
        panic!("expected queued VFS cache repair job");
    };

    let schedule = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), stat_started.notified())
        .await
        .unwrap();
    let disk_scan_budget = app
        .runtime_resource_class_diagnostics()
        .into_iter()
        .find(|class| class.name == "disk.scan")
        .expect("disk.scan runtime budget");
    let saturated = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();

    assert_eq!(disk_scan_budget.available_permits, 0);
    assert_eq!(saturated, LibraryScanScheduleOutcome::BudgetSaturated);

    release_stat.notify_one();
    wait_for_vfs_cache_repair_runtime_job(&app).await;
    let loaded = store.get_job(job.id).await.unwrap().unwrap();
    let summary_json = loaded.summary_json.as_deref().expect("job summary");
    let summary: VfsCacheRepairJobSummary = serde_json::from_str(summary_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(job.id),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();

    assert_eq!(schedule, LibraryScanScheduleOutcome::Scheduled(job.id));
    assert_eq!(loaded.status, JobStatus::Succeeded);
    assert_eq!(summary.action, nako_vfs::VfsCacheRepairAction::RefreshCache);
    assert_eq!(summary.source_scheme, "local");
    assert_eq!(summary.operation, VfsCacheOperation::Stat);
    assert_eq!(
        summary.classification,
        nako_vfs::VfsCacheRepairClassification::RetryableRefreshFailure
    );
    assert_eq!(
        summary.failure_class,
        Some(StorageFailureClass::Unavailable)
    );
    assert_eq!(summary.failed_at_ms, failure.failed_at_ms);
    assert_eq!(summary.failure_count, failure.failure_count);
    assert_eq!(summary.refreshed_cache_state, Some(ObjectCacheState::Fresh));
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 0);
    assert_eq!(gated_backend.stat_calls.load(Ordering::SeqCst), 1);
    assert_eq!(gated_backend.list_calls.load(Ordering::SeqCst), 0);
    assert!(claim.is_none());
    assert!(!summary_json.contains("Hidden Movie"));
    assert!(!summary_json.contains("Secret Path"));
    assert!(!summary_json.contains("ExampleUser"));
    assert!(!summary_json.contains("token=secret"));
    assert!(!summary_json.contains("local:///"));
    assert!(!summary_json.contains("storage backend unavailable"));
    assert!(!summary_json.contains("safe-test-etag"));
    assert!(!summary_json.contains("safe-test-fingerprint"));
}

#[tokio::test]
async fn vfs_cache_repair_scheduler_ignores_unrelated_claimable_job_window() {
    let (_temp, app, store, backend, _failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    for _ in 0..PageRequest::MAX_LIMIT {
        store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: "metadata.tmdb".to_owned(),
                priority: JobPriority::High,
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();
    }
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::Normal))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) = outcome else {
        panic!("expected queued VFS cache repair job");
    };

    let schedule = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_vfs_cache_repair_runtime_job(&app).await;
    let loaded = store.get_job(job.id).await.unwrap().unwrap();

    assert_eq!(schedule, LibraryScanScheduleOutcome::Scheduled(job.id));
    assert_eq!(loaded.status, JobStatus::Succeeded);
    assert!(loaded.summary_json.is_some());
    assert_eq!(backend.stat_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn vfs_cache_repair_scheduler_preserves_starved_disk_scan_order_across_job_kinds() {
    let (_temp, app, store, _backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let repair = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, Some(JobPriority::Low))
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(repair_job) = repair else {
        panic!("expected queued VFS cache repair job");
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    let library_id = failure.authority.library_id.expect("library authority");
    let scan_job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            priority: JobPriority::High,
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    let source_hash_job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::High,
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();

    let schedule = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();

    assert_eq!(
        schedule,
        LibraryScanScheduleOutcome::Scheduled(repair_job.id)
    );
    assert_ne!(scan_job.id, repair_job.id);
    assert_ne!(source_hash_job.id, repair_job.id);
}

#[tokio::test]
async fn vfs_cache_repair_scheduler_persists_redacted_backend_failure() {
    let (temp, app, store, _backend, failure) =
        vfs_cache_repair_enqueue_app_with_failure(StorageFailureClass::Unavailable.safe_message())
            .await;
    let target = app
        .storage()
        .list_vfs_cache_repair_targets(PageRequest::new(10, 0))
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("repair target");
    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target.target_ref, None)
        .await
        .unwrap();
    let EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) = outcome else {
        panic!("expected queued VFS cache repair job");
    };
    let failing_backend = Arc::new(StorageHealthCountingBackend::new(StorageErrorKind::Network));
    let library_id = failure.authority.library_id.expect("library authority");
    app.storage()
        .replace_backend_for_test(
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            Arc::new(CachedStorageBackend::with_options(
                failing_backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;

    let schedule = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_vfs_cache_repair_runtime_failure(&app).await;
    let loaded = store.get_job(job.id).await.unwrap().unwrap();
    let persisted_error = loaded.error.as_deref().expect("durable job error");

    assert_eq!(schedule, LibraryScanScheduleOutcome::Scheduled(job.id));
    assert_eq!(loaded.status, JobStatus::Failed);
    assert_eq!(loaded.summary_json, None);
    assert_eq!(
        persisted_error,
        "storage error at local://<redacted>: storage backend unavailable"
    );
    assert_eq!(failing_backend.stat_calls.load(Ordering::SeqCst), 1);
    assert!(!persisted_error.contains("Hidden Movie"));
    assert!(!persisted_error.contains("Secret Path"));
    assert!(!persisted_error.contains("ExampleUser"));
    assert!(!persisted_error.contains("token=secret"));
    assert!(!persisted_error.contains("local:///"));
    assert!(!persisted_error.contains("storage health test failure"));
}

async fn wait_for_vfs_cache_repair_runtime_job(app: &NakoApp) {
    for _ in 0..500 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == 1
            && diagnostics.cancelled_jobs == 0
            && diagnostics.failed_jobs == 0
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "VFS cache repair scheduler job did not finish successfully: {:?}",
        app.runtime_diagnostics()
    );
}

async fn wait_for_vfs_cache_repair_runtime_failure(app: &NakoApp) {
    for _ in 0..500 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == 0
            && diagnostics.cancelled_jobs == 0
            && diagnostics.failed_jobs == 1
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "VFS cache repair scheduler job did not fail as expected: {:?}",
        app.runtime_diagnostics()
    );
}

async fn fail_vfs_cache_repair_retry_source_job(
    store: &NakoDatabase,
    job: NewJob,
) -> nako_core::Job {
    let job = store.enqueue_job(job).await.unwrap();
    store.start_job(job.id).await.unwrap();
    store
        .fail_job(
            job.id,
            "VFS cache repair failed for local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv?token=secret input_json safe-test-etag safe-test-fingerprint".to_owned(),
        )
        .await
        .unwrap()
}

async fn retry_vfs_cache_repair_job_expect_err_without_retry(
    app: &NakoApp,
    store: &NakoDatabase,
    job_id: JobId,
    max_attempts: Option<u32>,
) -> String {
    retry_vfs_cache_repair_job_request_expect_err_without_retry(
        app,
        store,
        RetryVfsCacheRepairJobRequest {
            job_id,
            max_attempts,
            next_attempt_at: None,
        },
    )
    .await
}

async fn retry_vfs_cache_repair_job_request_expect_err_without_retry(
    app: &NakoApp,
    store: &NakoDatabase,
    request: RetryVfsCacheRepairJobRequest,
) -> String {
    let job_id = request.job_id;
    let err = app
        .storage()
        .retry_vfs_cache_repair_job(request)
        .await
        .unwrap_err();
    let message = err.to_string();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(
        jobs.iter().all(|job| job.retry_of_job_id != Some(job_id)),
        "VFS cache repair retry error must not create retry job: {message}"
    );
    assert_vfs_cache_repair_retry_payload_redacted(&message);

    message
}

fn assert_vfs_cache_repair_retry_payload_redacted(payload: &str) {
    for forbidden in [
        "Hidden Movie",
        "Secret Path",
        "ExampleUser",
        "token",
        "local:///",
        "webdav:///",
        "input_json",
        "safe-test-etag",
        "safe-test-fingerprint",
        "storage health test failure",
        "storage backend unavailable",
    ] {
        assert!(
            !payload.contains(forbidden),
            "VFS cache repair retry payload leaked {forbidden:?}: {payload}"
        );
    }
}

async fn vfs_cache_repair_enqueue_app_with_failure(
    error: &str,
) -> (
    tempfile::TempDir,
    NakoApp,
    NakoDatabase,
    Arc<CacheRefreshCountingBackend>,
    VfsCacheFailure,
) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("movies");
    fs::create_dir_all(&root).unwrap();
    let library_id = LibraryId::new();
    let library_config = LocalLibraryConfig {
        id: library_id,
        name: "Movies".to_owned(),
        root,
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        NakoServerConfig {
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
            libraries: vec![library_config.clone()],
        },
        store.clone(),
    )
    .await
    .unwrap();
    let backend = Arc::new(CacheRefreshCountingBackend::new());
    app.storage()
        .replace_backend_for_test(
            library_config,
            Arc::new(CachedStorageBackend::with_options(
                backend.clone(),
                store.clone(),
                VfsCacheOptions {
                    stat_ttl_ms: 60_000,
                    list_ttl_ms: 60_000,
                    serve_stale_on_error: true,
                    cache_local: true,
                },
            )),
        )
        .await;
    let backend_key = format!("library:{library_id}:local");
    let failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv?token=secret".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            error: error.to_owned(),
            authority: VfsCacheFailureAuthority::attributed(library_id, backend_key),
        })
        .await
        .unwrap();

    (temp, app, store, backend, failure)
}

fn new_vfs_cache_repair_job(failure: &VfsCacheFailure, input: VfsCacheRepairJobInput) -> NewJob {
    NewJob {
        id: JobId::new(),
        kind: JobKind::VfsCacheRepair,
        resource_class: VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned(),
        priority: JobPriority::Normal,
        library_id: failure.authority.library_id,
        source_id: None,
        input_json: Some(serde_json::to_string(&input).unwrap()),
    }
}

fn assert_storage_rate_limited(err: NakoError) {
    assert_eq!(
        err.storage_failure_class(),
        Some(StorageFailureClass::RateLimited)
    );
}

struct CacheRefreshCountingBackend {
    stat_calls: AtomicUsize,
    list_calls: AtomicUsize,
    stat_started: Option<Arc<Notify>>,
    release_stat: Option<Arc<Notify>>,
}

impl CacheRefreshCountingBackend {
    fn new() -> Self {
        Self {
            stat_calls: AtomicUsize::new(0),
            list_calls: AtomicUsize::new(0),
            stat_started: None,
            release_stat: None,
        }
    }

    fn with_stat_gate(stat_started: Arc<Notify>, release_stat: Arc<Notify>) -> Self {
        Self {
            stat_started: Some(stat_started),
            release_stat: Some(release_stat),
            ..Self::new()
        }
    }

    fn metadata(&self, uri: &StorageUri) -> ObjectMetadata {
        let kind = if uri.as_str().ends_with(".mkv") {
            ObjectKind::File
        } else {
            ObjectKind::Directory
        };
        ObjectMetadata {
            uri: uri.clone(),
            kind,
            len: (kind == ObjectKind::File).then_some(4),
            modified_at: Some("100".to_owned()),
            etag: Some("safe-test-etag".to_owned()),
            fingerprint: Some("safe-test-fingerprint".to_owned()),
            capabilities: StorageCapabilities::SEEKABLE
                | StorageCapabilities::RANGE_READABLE
                | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        }
    }
}

#[async_trait]
impl StorageBackend for CacheRefreshCountingBackend {
    fn scheme(&self) -> &'static str {
        "local"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        self.stat_calls.fetch_add(1, Ordering::SeqCst);
        if let Some(stat_started) = &self.stat_started {
            stat_started.notify_one();
        }
        if let Some(release_stat) = &self.release_stat {
            release_stat.notified().await;
        }
        Ok(self.metadata(uri))
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![self.metadata(
            &StorageUri::from_parts(uri.scheme(), "Movies/Demo.mkv").unwrap(),
        )])
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Err(NakoError::Unsupported("test backend does not read text"))
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(NakoError::Unsupported("test backend does not write text"))
    }
}

struct StorageHealthCountingBackend {
    kind: StorageErrorKind,
    stat_calls: AtomicUsize,
    write_string_calls: AtomicUsize,
    write_calls: AtomicUsize,
    plan_link_calls: AtomicUsize,
    apply_calls: AtomicUsize,
    cleanup_calls: AtomicUsize,
    restore_calls: AtomicUsize,
}

impl StorageHealthCountingBackend {
    fn new(kind: StorageErrorKind) -> Self {
        Self {
            kind,
            stat_calls: AtomicUsize::new(0),
            write_string_calls: AtomicUsize::new(0),
            write_calls: AtomicUsize::new(0),
            plan_link_calls: AtomicUsize::new(0),
            apply_calls: AtomicUsize::new(0),
            cleanup_calls: AtomicUsize::new(0),
            restore_calls: AtomicUsize::new(0),
        }
    }

    fn err<T>(&self, uri: &StorageUri) -> Result<T> {
        Err(NakoError::storage(
            uri.to_string(),
            self.kind,
            "storage health test failure",
        ))
    }
}

#[async_trait]
impl StorageBackend for StorageHealthCountingBackend {
    fn scheme(&self) -> &'static str {
        "local"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        self.stat_calls.fetch_add(1, Ordering::SeqCst);
        self.err(uri)
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        self.err(uri)
    }

    async fn open_range(&self, uri: &StorageUri, _range: Option<ByteRange>) -> Result<VirtualFile> {
        self.err(uri)
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        self.err(uri)
    }

    async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
        self.write_string_calls.fetch_add(1, Ordering::SeqCst);
        self.err(uri)
    }

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        self.write_calls.fetch_add(1, Ordering::SeqCst);
        self.err(&request.uri)
    }

    async fn plan_link(
        &self,
        request: StorageLinkPlanRequest,
    ) -> Result<nako_vfs::StorageLinkPlan> {
        self.plan_link_calls.fetch_add(1, Ordering::SeqCst);
        self.err(&request.source_uri)
    }

    async fn apply(&self, request: StorageApplyRequest) -> Result<nako_vfs::StorageApplyReport> {
        self.apply_calls.fetch_add(1, Ordering::SeqCst);
        self.err(&request.source_uri)
    }

    async fn cleanup(
        &self,
        request: StorageCleanupRequest,
    ) -> Result<nako_vfs::StorageCleanupReport> {
        self.cleanup_calls.fetch_add(1, Ordering::SeqCst);
        self.err(&request.target_uri)
    }

    async fn restore(&self, request: StorageRestoreRequest) -> Result<StorageRestoreReport> {
        self.restore_calls.fetch_add(1, Ordering::SeqCst);
        self.err(&request.backup_uri)
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        self.err(&request.uri)
    }
}
