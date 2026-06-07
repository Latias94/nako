use super::super::storage::EnqueueVfsCacheRepairTargetOutcome;
use super::*;
use nako_core::{
    JobKind, JobListFilter, JobPriority, JobRepository, JobStatus, NewVfsCacheFailure,
    StorageBackendHealthRecord, StorageBackendHealthRepository, StorageBackendHealthStatus,
    StorageCircuitBreakerState, StorageErrorKind, StorageFailureClass,
    VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS, VfsCacheFailure, VfsCacheFailureAuthority,
    VfsCacheOperation, VfsCacheRepairJobAction, VfsCacheRepairJobInput, VfsCacheRepository,
    vfs_cache_repair_uri_digest,
};
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
}

impl CacheRefreshCountingBackend {
    fn new() -> Self {
        Self {
            stat_calls: AtomicUsize::new(0),
            list_calls: AtomicUsize::new(0),
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
