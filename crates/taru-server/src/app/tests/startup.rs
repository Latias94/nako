use super::*;
use taru_core::{Library, LibraryOptions, LibraryRepository};

fn startup_config(root: &Path, libraries: Vec<LocalLibraryConfig>) -> TaruServerConfig {
    TaruServerConfig {
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
        remux_staging_root: root.join("taru-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries,
    }
}

async fn wait_for_runtime_jobs(
    app: &TaruApp,
    succeeded_jobs: u64,
    failed_jobs: u64,
) -> RuntimeSupervisorDiagnostics {
    for _ in 0..100 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == succeeded_jobs && diagnostics.failed_jobs == failed_jobs {
            return diagnostics;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "runtime job diagnostics did not reach expected state: {:?}",
        app.runtime_diagnostics()
    );
}

#[tokio::test]
async fn scan_library_persists_job_success() {
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

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let job = app.jobs().get_job(output.job.id).await.unwrap();
    let events = store
        .list_outbox_events(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(app.startup_report().configured_libraries, 1);
    assert_eq!(app.startup_report().recovered_transcode_sessions, 0);
    assert_eq!(
        app.startup_report()
            .staging_cleanup
            .expect("staging cleanup report")
            .deleted_records,
        0
    );
    assert_eq!(app.startup_report().metadata_raw_cache_deleted, 0);
    assert_eq!(app.startup_report().metadata_lifecycle_tasks_started, 0);
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
async fn background_scan_job_uses_runtime_job_supervision() {
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
    let app = TaruApp::new_with_store(config, store).await.unwrap();

    let job = app
        .library_scan()
        .enqueue_library_scan(library_id)
        .await
        .unwrap();
    let diagnostics = wait_for_runtime_jobs(&app, 1, 0).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();

    assert_eq!(persisted.status, JobStatus::Succeeded);
    assert_eq!(diagnostics.completed_tasks, 1);
    assert_eq!(diagnostics.failed_tasks, 0);
}

#[tokio::test]
async fn app_startup_persists_all_configured_libraries_with_library_scoped_roots() {
    let temp = tempfile::tempdir().unwrap();
    let movies_id = LibraryId::new();
    let anime_id = LibraryId::new();
    let config = startup_config(
        temp.path(),
        vec![
            LocalLibraryConfig {
                id: movies_id,
                name: "Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: taru_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: anime_id,
                name: "Remote Anime".to_owned(),
                root: temp.path().join("unused-local-root"),
                preset: taru_core::LibraryPreset::Anime,
                webdav: Some(WebDavLibraryConfig {
                    root: "webdav:///Anime".to_owned(),
                    base_url: "https://webdav.example.test/dav".to_owned(),
                    username: None,
                    password_env: None,
                    timeout_ms: 15_000,
                    max_attempts: 4,
                }),
            },
        ],
    );
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    let movies = store.get_library(movies_id).await.unwrap().unwrap();
    let anime = store.get_library(anime_id).await.unwrap().unwrap();
    let libraries = store
        .list_libraries(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(app.startup_report().configured_libraries, 2);
    assert_eq!(libraries.len(), 2);
    assert!(libraries.iter().any(|library| library.id == movies_id));
    assert!(libraries.iter().any(|library| library.id == anime_id));
    assert_eq!(movies.name, "Movies");
    assert_eq!(movies.roots, vec!["local:///".to_owned()]);
    assert_eq!(movies.options.preset, taru_core::LibraryPreset::Movies);
    assert_eq!(anime.name, "Remote Anime");
    assert_eq!(anime.roots, vec!["webdav:///Anime".to_owned()]);
    assert_eq!(anime.options.preset, taru_core::LibraryPreset::Anime);
}

#[tokio::test]
async fn app_startup_overwrites_persisted_library_with_configured_desired_state() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Old Movies".to_owned(),
            roots: vec!["local:///OldMovies".to_owned()],
            options: LibraryOptions::from_preset(taru_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Remote Anime".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: taru_core::LibraryPreset::Anime,
            webdav: Some(WebDavLibraryConfig {
                root: "webdav:///Anime".to_owned(),
                base_url: "https://webdav.example.test/dav".to_owned(),
                username: None,
                password_env: None,
                timeout_ms: 15_000,
                max_attempts: 4,
            }),
        }],
    );
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let loaded = store.get_library(library_id).await.unwrap().unwrap();

    assert_eq!(app.startup_report().configured_libraries, 1);
    assert_eq!(loaded.name, "Remote Anime");
    assert_eq!(loaded.roots, vec!["webdav:///Anime".to_owned()]);
    assert_eq!(loaded.options.preset, taru_core::LibraryPreset::Anime);
}

#[tokio::test]
async fn app_startup_retains_persisted_library_missing_from_config() {
    let temp = tempfile::tempdir().unwrap();
    let retained_id = LibraryId::new();
    let configured_id = LibraryId::new();
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store
        .upsert_library(&Library {
            id: retained_id,
            name: "Retained Historical Library".to_owned(),
            roots: vec!["local:///Retained".to_owned()],
            options: LibraryOptions::from_preset(taru_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: configured_id,
            name: "Configured Library".to_owned(),
            root: temp.path().join("configured"),
            preset: taru_core::LibraryPreset::Anime,
            webdav: None,
        }],
    );
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let retained = store.get_library(retained_id).await.unwrap().unwrap();
    let configured = store.get_library(configured_id).await.unwrap().unwrap();
    let libraries = store
        .list_libraries(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(app.startup_report().configured_libraries, 1);
    assert_eq!(libraries.len(), 2);
    assert_eq!(retained.name, "Retained Historical Library");
    assert_eq!(retained.roots, vec!["local:///Retained".to_owned()]);
    assert_eq!(retained.options.preset, taru_core::LibraryPreset::Movies);
    assert_eq!(configured.name, "Configured Library");
    assert_eq!(configured.roots, vec!["local:///".to_owned()]);
    assert_eq!(configured.options.preset, taru_core::LibraryPreset::Anime);
}

#[tokio::test]
async fn app_startup_reports_configured_library_reconciliation() {
    let temp = tempfile::tempdir().unwrap();
    let unchanged_id = LibraryId::new();
    let updated_id = LibraryId::new();
    let added_id = LibraryId::new();
    let retained_id = LibraryId::new();
    let unchanged = Library {
        id: unchanged_id,
        name: "Unchanged Movies".to_owned(),
        roots: vec!["local:///".to_owned()],
        options: LibraryOptions::from_preset(taru_core::LibraryPreset::Movies),
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store.upsert_library(&unchanged).await.unwrap();
    store
        .upsert_library(&Library {
            id: updated_id,
            name: "Old Anime".to_owned(),
            roots: vec!["webdav:///OldAnime".to_owned()],
            options: LibraryOptions::from_preset(taru_core::LibraryPreset::Anime),
        })
        .await
        .unwrap();
    store
        .upsert_library(&Library {
            id: retained_id,
            name: "Retained Historical Library".to_owned(),
            roots: vec!["local:///Retained".to_owned()],
            options: LibraryOptions::from_preset(taru_core::LibraryPreset::MixedVideo),
        })
        .await
        .unwrap();

    let config = startup_config(
        temp.path(),
        vec![
            LocalLibraryConfig {
                id: unchanged_id,
                name: "Unchanged Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: taru_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: updated_id,
                name: "Updated Anime".to_owned(),
                root: temp.path().join("unused-local-root"),
                preset: taru_core::LibraryPreset::Anime,
                webdav: Some(WebDavLibraryConfig {
                    root: "webdav:///Anime".to_owned(),
                    base_url: "https://webdav.example.test/dav".to_owned(),
                    username: None,
                    password_env: None,
                    timeout_ms: 15_000,
                    max_attempts: 4,
                }),
            },
            LocalLibraryConfig {
                id: added_id,
                name: "Added Home Videos".to_owned(),
                root: temp.path().join("home-videos"),
                preset: taru_core::LibraryPreset::HomeVideo,
                webdav: None,
            },
        ],
    );
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let updated = store.get_library(updated_id).await.unwrap().unwrap();
    let added = store.get_library(added_id).await.unwrap().unwrap();
    let retained = store.get_library(retained_id).await.unwrap().unwrap();

    let reconciliation = &app.startup_report().library_reconciliation;
    assert_eq!(app.startup_report().configured_libraries, 3);
    assert_eq!(reconciliation.configured_libraries, 3);
    assert_eq!(reconciliation.added_libraries, 1);
    assert_eq!(reconciliation.updated_libraries, 1);
    assert_eq!(reconciliation.unchanged_libraries, 1);
    assert_eq!(reconciliation.retained_unconfigured_libraries, 1);
    assert_eq!(updated.name, "Updated Anime");
    assert_eq!(updated.roots, vec!["webdav:///Anime".to_owned()]);
    assert_eq!(added.name, "Added Home Videos");
    assert_eq!(added.roots, vec!["local:///".to_owned()]);
    assert_eq!(retained.name, "Retained Historical Library");
    assert_eq!(retained.roots, vec!["local:///Retained".to_owned()]);
}

#[tokio::test]
async fn scan_library_uses_reconciled_library_row_after_startup() {
    let temp = tempfile::tempdir().unwrap();
    let configured_root = temp.path().join("configured");
    let persisted_root = temp.path().join("persisted");
    fs::create_dir_all(&configured_root).unwrap();
    fs::create_dir_all(&persisted_root).unwrap();
    fs::write(configured_root.join("configured.mkv"), b"media").unwrap();
    fs::write(persisted_root.join("persisted.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: configured_root,
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Persisted Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions {
                scan: taru_core::LibraryScanOptions {
                    max_depth: Some(0),
                    ..taru_core::LibraryScanOptions::default()
                },
                ..LibraryOptions::from_preset(taru_core::LibraryPreset::Movies)
            },
        })
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let loaded = store.get_library(library_id).await.unwrap().unwrap();

    assert_eq!(loaded.name, "Persisted Movies");
    assert_eq!(output.index.discovered_files, 0);
    assert_eq!(output.index.inserted_sources, 0);
}

#[tokio::test]
async fn app_startup_rejects_duplicate_configured_library_ids() {
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
async fn app_startup_rejects_duplicate_metadata_provider_configs() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let mut metadata = MetadataConfig::default();
    metadata.providers = vec![
        MetadataProviderConfig {
            provider: ExternalProvider::Tmdb,
            enabled: false,
            token_env: None,
            api_key_env: None,
            api_base_url: None,
            image_base_url: None,
            language: None,
            include_adult: false,
            headers: Vec::new(),
            runtime: None,
        },
        MetadataProviderConfig {
            provider: ExternalProvider::Tmdb,
            enabled: false,
            token_env: None,
            api_key_env: None,
            api_base_url: None,
            image_base_url: None,
            language: None,
            include_adult: false,
            headers: Vec::new(),
            runtime: None,
        },
    ];
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();

    let err = TaruApp::new_with_store(config, store).await.unwrap_err();

    let TaruError::InvalidInput { message } = err else {
        panic!("expected duplicate provider validation error");
    };
    assert_eq!(message, "duplicate metadata provider config: tmdb");
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
    let restarted = TaruApp::new_with_store(config, store.clone())
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
    assert_eq!(restarted.startup_report().configured_libraries, 1);
    assert_eq!(restarted.startup_report().recovered_transcode_sessions, 1);
    assert_eq!(
        restarted.startup_report().metadata_lifecycle_tasks_started,
        0
    );
}

#[tokio::test]
async fn app_startup_marks_unfinished_jobs_failed() {
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
    let config = app.config().clone();

    let queued_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: queued_id,
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.refresh".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();

    let running_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: running_id,
            kind: JobKind::LibraryScan,
            resource_class: "library.scan".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(running_id).await.unwrap();

    let succeeded_id = JobId::new();
    store
        .enqueue_job(NewJob {
            id: succeeded_id,
            kind: JobKind::NfoImport,
            resource_class: "nfo.import".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();
    store.start_job(succeeded_id).await.unwrap();
    store
        .succeed_job(succeeded_id, Some(r#"{"imported":1}"#.to_owned()))
        .await
        .unwrap();

    drop(app);
    let restarted = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let queued = store.get_job(queued_id).await.unwrap().unwrap();
    let running = store.get_job(running_id).await.unwrap().unwrap();
    let succeeded = store.get_job(succeeded_id).await.unwrap().unwrap();

    assert_eq!(queued.status, JobStatus::Failed);
    assert_eq!(
        queued.error,
        Some("job was unfinished during server startup".to_owned())
    );
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(
        running.error,
        Some("job was unfinished during server startup".to_owned())
    );
    assert_eq!(succeeded.status, JobStatus::Succeeded);
    assert_eq!(restarted.startup_report().recovered_jobs, 2);
}

#[tokio::test]
async fn startup_report_tracks_disabled_staging_cleanup() {
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
        staging: StagingConfig {
            cleanup_on_startup: false,
            ..StagingConfig::default()
        },
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
    let app = TaruApp::new_with_store(config, store).await.unwrap();

    assert_eq!(app.startup_report().configured_libraries, 1);
    assert_eq!(app.startup_report().staging_cleanup, None);
}
