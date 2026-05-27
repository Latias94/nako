use super::*;

#[tokio::test]
async fn manifest_recording_backend_records_probe_staging() {
    let temp = tempfile::tempdir().unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: b"probe-media".to_vec(),
            local_path_hint: None,
        }),
        Arc::new(store.clone()),
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
    let reserved = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Reserved),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    let staging = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Staging),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert!(reserved.is_empty());
    assert!(staging.is_empty());
}

#[tokio::test]
async fn manifest_recording_backend_rejects_staging_over_disk_budget() {
    let temp = tempfile::tempdir().unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: b"probe-media".to_vec(),
            local_path_hint: None,
        }),
        Arc::new(store.clone()),
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

    let NakoError::Storage { message, .. } = err else {
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = Arc::new(ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: vec![b'x'; 8],
            local_path_hint: None,
        }),
        Arc::new(store.clone()),
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
    let NakoError::Storage { message, .. } = &failures[0] else {
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let control = ConcurrentStageControl::new();
    let backend = Arc::new(ManifestRecordingStorageBackend::new(
        Arc::new(ConcurrentStageBackend {
            bytes: vec![b'x'; 8],
            control: control.clone(),
        }),
        Arc::new(store.clone()),
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(FailingStageBackend {
            len: 8,
            fingerprint: "failing-stage".to_owned(),
        }),
        Arc::new(store.clone()),
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

    let NakoError::Storage { message, .. } = err else {
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let uri = StorageUri::parse("webdav:///Movies/Demo.mkv").unwrap();
    let staging_root = temp.path().join("probe-inputs");
    let reserved_path =
        nako_vfs::deterministic_stage_path(&staging_root, &uri, Some("reserved")).unwrap();
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
            state: StagingState::Reserved,
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
        Arc::new(store.clone()),
        StagingPurpose::ProbeInput,
        32,
        StagingConfig::default().retention_ms,
        Arc::new(Semaphore::new(1)),
    );

    let err = backend
        .stage(StageRequest::new(uri, staging_root))
        .await
        .unwrap_err();

    let NakoError::Storage { message, .. } = err else {
        panic!("expected active reservation error");
    };
    assert!(message.contains("staging input is already reserved"));
    assert!(!message.contains("intentional staging failure"));
}

#[tokio::test]
async fn manifest_recording_backend_waits_for_stage_budget() {
    let temp = tempfile::tempdir().unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let stage_permits = Arc::new(Semaphore::new(1));
    let held_permit = stage_permits.clone().acquire_owned().await.unwrap();
    let backend = ManifestRecordingStorageBackend::new(
        Arc::new(RemotePlaybackBackend {
            bytes: b"probe-media".to_vec(),
            local_path_hint: None,
        }),
        Arc::new(store.clone()),
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
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig {
                remote_stream_concurrency: 1,
                remote_stage_concurrency: 1,
                ..PlaybackConfig::default()
            },
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
        .playback()
        .plan_direct_play(source.id, DirectPlayRangeRequest::None)
        .await
        .unwrap();

    let DirectPlaySourceBody::Stream(_) = &plan.body else {
        panic!("expected remote direct play to hold a VFS stream");
    };
    let backend = app
        .playback()
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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

    let _app = NakoApp::new_with_store(
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
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("library"),
                preset: nako_core::LibraryPreset::Movies,
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let record_id = StagingManifestId::new();
    let mut record = staging_manifest_record(record_id, &staged_path, Some(1), 0);
    record.state = StagingState::Reserved;
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
async fn staging_cleanup_retries_expired_manifest_records() {
    let temp = tempfile::tempdir().unwrap();
    let staged_path = temp.path().join("expired.mkv");
    fs::write(&staged_path, b"expired").unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let record_id = StagingManifestId::new();
    let mut record = staging_manifest_record(record_id, &staged_path, Some(1), 0);
    record.state = StagingState::Expired;
    store.upsert_staging_manifest_record(record).await.unwrap();

    let cleanup = cleanup_expired_staging_inputs(&store, 2_000).await.unwrap();

    assert_eq!(cleanup.deleted_records, 1);
    assert_eq!(cleanup.deleted_files, 1);
    assert!(!staged_path.exists());
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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

    let runtime = crate::app::runtime::RuntimeSupervisor::new();
    let lease = crate::app::staging::StagingLease::acquire(
        Arc::new(store.clone()),
        record_id,
        runtime.clone(),
    )
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
async fn dropped_staging_lease_releases_manifest_record() {
    let temp = tempfile::tempdir().unwrap();
    let staged_path = temp.path().join("dropped-lease.mkv");
    fs::write(&staged_path, b"leased").unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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

    let runtime = crate::app::runtime::RuntimeSupervisor::new();
    let lease = crate::app::staging::StagingLease::acquire(
        Arc::new(store.clone()),
        record_id,
        runtime.clone(),
    )
    .await
    .unwrap();
    drop(lease);

    for _ in 0..50 {
        let diagnostics = runtime.diagnostics();
        if diagnostics.completed_tasks > 0 {
            assert_eq!(diagnostics.failed_tasks, 0);
        }
        let record = store
            .get_staging_manifest_record(record_id)
            .await
            .unwrap()
            .unwrap();
        if record.active_leases == 0 {
            assert_eq!(record.state, StagingState::Ready);
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let record = store
        .get_staging_manifest_record(record_id)
        .await
        .unwrap()
        .unwrap();
    panic!(
        "dropped lease was not released: state={:?}, active_leases={}",
        record.state, record.active_leases
    );
}
