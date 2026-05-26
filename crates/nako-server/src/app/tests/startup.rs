use super::*;
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonScope, AddonTaskRequest, AddonTaskResponse,
};
use nako_api::extension::{
    AddonGrantAssignment, IssueAddonTokenRequest, RegisterAddonRequest, ReplaceAddonGrantsRequest,
};
use nako_core::{
    AddonId, AddonPermission, AddonRepository, AddonSideEffectTarget,
    AddonSideEffectValidationStatus, AddonStatus, AddonTaskRunRepository, ArtworkCandidateId,
    ArtworkCandidateRepository, ArtworkCandidateSourceKind, IdentityAccessRepository, ImageKind,
    Library, LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryRepository,
    ManagedArtworkAcceptanceRecord, ManagedArtworkIngestId, ManagedArtworkIngestStatus,
    ManagedArtworkRepository, NewAddonRegistration, NewAddonSideEffect, NewAddonToken,
    NewArtworkCandidate, NewManagedArtworkIngest, UserPrincipalId, UserRole,
    bootstrap_admin_user_id,
};
use nako_official_addon_catalog::metadata_scraper;
use tokio::sync::Mutex;

fn startup_config(root: &Path, libraries: Vec<LocalLibraryConfig>) -> NakoServerConfig {
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
        remux_staging_root: root.join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries,
    }
}

async fn wait_for_runtime_jobs(
    app: &NakoApp,
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
        "runtime job diagnostics did not reach expected state: {:?}",
        app.runtime_diagnostics()
    );
}

#[tokio::test]
async fn app_startup_creates_deterministic_bootstrap_admin_user() {
    let temp = tempfile::tempdir().unwrap();
    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Bootstrap Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    assert!(app.startup_report().database_migrated);

    let user = store
        .get_user_by_principal(&UserPrincipalId::local_admin())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.id, bootstrap_admin_user_id());
    assert_eq!(user.username, "admin");
    assert_eq!(user.display_name, "Local administrator");
    assert_eq!(user.status, nako_core::UserStatus::Active);
    assert!(
        store
            .list_role_assignments(user.id)
            .await
            .unwrap()
            .iter()
            .any(|assignment| assignment.role == UserRole::Administrator)
    );

    let restarted = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    assert!(restarted.startup_report().database_migrated);
    assert_eq!(
        store
            .list_users(PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        1
    );
}

async fn create_startup_managed_artwork_ingest(
    store: &NakoDatabase,
    library_id: LibraryId,
    item_id: MediaItemId,
    idempotency_key: &str,
) -> ManagedArtworkAcceptanceRecord {
    let addon_id = nako_core::AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: format!("example.artwork.{idempotency_key}"),
            name: "Startup Artwork".to_owned(),
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
    let token_id = nako_core::AddonTokenId::new();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "startup artwork".to_owned(),
            token_prefix: "nako_at_startup".to_owned(),
            token_hash: format!("sha256:{idempotency_key}"),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: nako_core::AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: idempotency_key.to_owned(),
            provenance_json: r#"{"origin":"startup-test"}"#.to_owned(),
            payload_json: r#"{"intent":"propose_artwork"}"#.to_owned(),
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
            source_uri: format!("https://cdn.example.test/{idempotency_key}.png?token=secret"),
            width: Some(1),
            height: Some(1),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let job_id = JobId::new();

    store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ManagedArtworkIngestId::new(),
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
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(
                    serde_json::json!({
                        "candidate_id": candidate.id,
                        "library_id": library_id,
                        "item_id": item_id,
                        "image_kind": "poster"
                    })
                    .to_string(),
                ),
            },
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn scan_library_persists_job_success() {
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
async fn scan_library_imports_enabled_nfo_metadata_after_probe() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    write_fixture_mp4(&library_root.join("demo.mp4"));
    fs::write(
        library_root.join("demo.nfo"),
        r#"<movie>
  <title>NFO Scan Title</title>
  <plot>Imported during scan.</plot>
</movie>
"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    let item = store
        .get_media_item(sources[0].item_id)
        .await
        .unwrap()
        .unwrap();
    let nfo = output.metadata.nfo_import.expect("scan should import NFO");

    assert_eq!(output.index.discovered_files, 1);
    assert_eq!(output.probe.probed_sources, 1);
    assert_eq!(nfo.discovered_nfo, 1);
    assert_eq!(nfo.imported_items, 1);
    assert_eq!(item.metadata.title, "NFO Scan Title");
    assert_eq!(
        item.metadata.overview.as_deref(),
        Some("Imported during scan.")
    );
}

#[tokio::test]
async fn scan_library_skips_nfo_import_when_scan_metadata_is_disabled() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    write_fixture_mp4(&library_root.join("demo.mp4"));
    fs::write(
        library_root.join("demo.nfo"),
        r#"<movie><title>Should Not Import</title></movie>"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let mut config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.scan = nako_core::MetadataScanPolicy::disabled();
    config.metadata.library_profiles.insert(library_id, profile);
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    let item = store
        .get_media_item(sources[0].item_id)
        .await
        .unwrap()
        .unwrap();

    assert!(output.metadata.nfo_import.is_none());
    assert_eq!(item.metadata.title, "demo");
}

#[tokio::test]
async fn scan_library_enqueues_addon_bulk_metadata_scrape_when_enabled() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    write_fixture_mp4(&library_root.join("demo.mp4"));
    let library_id = LibraryId::new();
    let mut config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.scan.addon_scrape = true;
    config.metadata.library_profiles.insert(library_id, profile);
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let (base_url, captured) = scan_bulk_metadata_scrape_addon_server().await;
    let mut manifest = metadata_scraper::default_manifest();
    manifest.base_url = base_url;
    let registered = app
        .addons()
        .register_addon(RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        })
        .await
        .unwrap();
    let addon_id = registered.addon.summary.id;
    app.addons()
        .sync_addon_routing_plans(addon_id)
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let addon_scrape = output
        .metadata
        .addon_scrape
        .expect("scan should enqueue addon scrape");
    assert_eq!(addon_scrape.total_sources, 1);
    assert_eq!(addon_scrape.enqueued_items, 1);
    assert!(!addon_scrape.truncated);
    assert!(addon_scrape.skipped_addons.is_empty());
    assert_eq!(addon_scrape.task_runs.len(), 1);
    assert_eq!(addon_scrape.task_runs[0].addon_id, addon_id);
    assert_eq!(
        addon_scrape.task_runs[0].declaration_id,
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID
    );

    let completed = wait_for_addon_task_run_status(
        &store,
        addon_scrape.task_runs[0].job_id,
        JobStatus::Succeeded,
    )
    .await;
    assert_eq!(completed.job.library_id, Some(library_id));
    assert_eq!(
        completed.declaration_id,
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID
    );

    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    let request = &captured[0];
    assert_eq!(
        request.task_id,
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID
    );
    assert_eq!(request.library_id, Some(library_id.to_string()));
    assert!(request.source_id.is_none());
    assert_eq!(request.payload["batch_size"], 12);
    assert_eq!(request.payload["items"].as_array().unwrap().len(), 1);
    assert_eq!(request.payload["items"][0]["title"], "demo");
    assert_eq!(
        request.payload["items"][0]["library_id"],
        library_id.to_string()
    );
    assert!(request.payload["items"][0].get("writeback").is_none());
    assert!(
        request.payload["items"][0]
            .get("artwork_writeback")
            .is_none()
    );
}

#[tokio::test]
async fn scan_library_adds_addon_bulk_metadata_writeback_when_enabled() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    write_fixture_mp4(&library_root.join("demo.mp4"));
    let library_id = LibraryId::new();
    let mut config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.scan.addon_scrape = true;
    profile.scan.addon_writeback = true;
    config.metadata.library_profiles.insert(library_id, profile);
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let (base_url, captured) = scan_bulk_metadata_scrape_addon_server().await;
    let mut manifest = metadata_scraper::default_manifest();
    manifest.base_url = base_url;
    let registered = app
        .addons()
        .register_addon(RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        })
        .await
        .unwrap();
    app.addons()
        .sync_addon_routing_plans(registered.addon.summary.id)
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let addon_scrape = output
        .metadata
        .addon_scrape
        .expect("scan should enqueue addon scrape");
    wait_for_addon_task_run_status(
        &store,
        addon_scrape.task_runs[0].job_id,
        JobStatus::Succeeded,
    )
    .await;

    let captured = captured.lock().await;
    let item = &captured[0].payload["items"][0];
    let writeback = item
        .get("writeback")
        .expect("writeback should be explicit when enabled");

    assert_eq!(writeback["library_id"], library_id.to_string());
    assert_eq!(writeback["target"]["kind"], "media_source");
    assert_eq!(writeback["target"]["id"], item["source_id"]);
    assert!(
        writeback["idempotency_key"]
            .as_str()
            .unwrap()
            .starts_with(&format!(
                "library-scan:{}:addon-bulk-metadata-writeback:",
                output.job.id
            ))
    );
    assert!(item.get("artwork_writeback").is_none());
}

#[tokio::test]
async fn scan_library_continues_addon_bulk_metadata_scrape_from_next_cursor() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    for index in 0..13 {
        write_fixture_mp4(&library_root.join(format!("demo-{index:02}.mp4")));
    }
    let library_id = LibraryId::new();
    let mut config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.scan.addon_scrape = true;
    config.metadata.library_profiles.insert(library_id, profile);
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let (base_url, captured) = scan_bulk_metadata_scrape_addon_server().await;
    let mut manifest = metadata_scraper::default_manifest();
    manifest.base_url = base_url;
    let registered = app
        .addons()
        .register_addon(RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        })
        .await
        .unwrap();
    let addon_id = registered.addon.summary.id;
    app.addons()
        .sync_addon_routing_plans(addon_id)
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let addon_scrape = output
        .metadata
        .addon_scrape
        .expect("scan should enqueue addon scrape");
    assert_eq!(addon_scrape.total_sources, 13);
    assert_eq!(addon_scrape.enqueued_items, 13);
    assert!(!addon_scrape.truncated);
    let _first = wait_for_addon_task_run_status(
        &store,
        addon_scrape.task_runs[0].job_id,
        JobStatus::Succeeded,
    )
    .await;
    let second = wait_for_continuation_task_run(&store, addon_id, 12).await;

    let captured = captured.lock().await;
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[0].payload["cursor"], 0);
    assert_eq!(captured[0].payload["items"].as_array().unwrap().len(), 13);
    assert_eq!(captured[1].payload["cursor"], 12);
    assert_eq!(captured[1].payload["batch_size"], 12);
    assert_eq!(captured[1].payload["resume_state"]["marker"], "cursor-0");
    assert_eq!(second.retry_of_job_id, None);
    assert_eq!(second.job.library_id, Some(library_id));
}

#[tokio::test]
async fn scan_library_addon_bulk_metadata_writeback_merges_metadata_via_side_effect() {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    fs::create_dir_all(&library_root).unwrap();
    write_fixture_mp4(&library_root.join("demo.mp4"));
    let library_id = LibraryId::new();
    let mut config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let mut profile = nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    profile.refresh_mode = MetadataRefreshMode::MissingOnly;
    profile.scan.addon_scrape = true;
    profile.scan.addon_writeback = true;
    config.metadata.library_profiles.insert(library_id, profile);
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let nako_base_url = start_nako_http_server(app.clone()).await;
    let (sidecar_base_url, captured, raw_token_slot) =
        scan_bulk_metadata_writeback_addon_server(nako_base_url).await;
    let mut manifest = metadata_scraper::default_manifest();
    manifest.base_url = sidecar_base_url;
    let registered = app
        .addons()
        .register_addon(RegisterAddonRequest {
            id: None,
            manifest,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
                AddonScope::AutomationRun,
            ],
            status: Some(AddonStatus::Enabled),
        })
        .await
        .unwrap();
    let addon_id = registered.addon.summary.id;
    app.addons()
        .replace_addon_grants(
            addon_id,
            ReplaceAddonGrantsRequest {
                grants: vec![AddonGrantAssignment {
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library_id),
                }],
            },
        )
        .await
        .unwrap();
    let issued = app
        .addons()
        .issue_addon_token(
            addon_id,
            IssueAddonTokenRequest {
                label: Some("scan writeback sidecar".to_owned()),
            },
        )
        .await
        .unwrap();
    *raw_token_slot.lock().await = Some(issued.raw_token);
    app.addons()
        .sync_addon_routing_plans(addon_id)
        .await
        .unwrap();

    let output = app.library_scan().scan_library(library_id).await.unwrap();
    let addon_scrape = output
        .metadata
        .addon_scrape
        .expect("scan should enqueue addon scrape");
    let completed = wait_for_addon_task_run_status(
        &store,
        addon_scrape.task_runs[0].job_id,
        JobStatus::Succeeded,
    )
    .await;

    assert_eq!(completed.job.library_id, Some(library_id));
    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    assert!(captured[0].payload["items"][0].get("writeback").is_some());

    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    let item = store
        .get_media_item(sources[0].item_id)
        .await
        .unwrap()
        .expect("media item should exist");

    assert_eq!(item.metadata.title, "demo");
    assert_eq!(
        item.metadata.overview.as_deref(),
        Some("Merged through the Addon Side Effect runtime.")
    );
}

#[tokio::test]
async fn background_scan_job_uses_runtime_job_supervision() {
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
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();

    let job = app
        .library_scan()
        .enqueue_library_scan(library_id)
        .await
        .unwrap();
    let diagnostics = wait_for_runtime_jobs(&app, 1, 0, 0).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();

    assert_eq!(persisted.status, JobStatus::Succeeded);
    assert_eq!(diagnostics.completed_tasks, 1);
    assert_eq!(diagnostics.failed_tasks, 0);
}

fn write_fixture_mp4(path: &Path) {
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=16x16:d=0.1",
            "-an",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(path)
        .status()
        .expect("ffmpeg should be available for Nako app scan tests");
    assert!(status.success(), "ffmpeg failed to create fixture mp4");
}

async fn start_nako_http_server(app: NakoApp) -> String {
    let router = crate::http::build_router(app);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    tokio::task::yield_now().await;

    format!("http://{addr}")
}

async fn scan_bulk_metadata_scrape_addon_server() -> (String, Arc<Mutex<Vec<AddonTaskRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let router = Router::new().route(
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_PATH,
        axum::routing::post({
            let requests = Arc::clone(&requests);
            move |Json(request): Json<AddonTaskRequest>| {
                let requests = Arc::clone(&requests);
                async move {
                    let cursor = request
                        .payload
                        .get("cursor")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as usize;
                    let batch_size = request
                        .payload
                        .get("batch_size")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(12) as usize;
                    let item_count = request
                        .payload
                        .get("items")
                        .and_then(serde_json::Value::as_array)
                        .map_or(0, Vec::len);
                    let next_cursor = cursor.saturating_add(batch_size);
                    let next_cursor =
                        (next_cursor < item_count).then_some(serde_json::json!(next_cursor));
                    requests.lock().await.push(request.clone());

                    Json(AddonTaskResponse {
                        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                        addon_id: request.addon_id,
                        task_id: request.task_id,
                        job_id: request.job_id,
                        request_id: request.request_id,
                        output: serde_json::json!({
                            "accepted": true,
                            "item_count": item_count,
                            "next_cursor": next_cursor,
                            "resume_state": {
                                "previous_cursor": cursor,
                                "marker": format!("cursor-{cursor}")
                            }
                        }),
                    })
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    tokio::task::yield_now().await;

    (format!("http://{addr}"), requests)
}

async fn scan_bulk_metadata_writeback_addon_server(
    nako_base_url: String,
) -> (
    String,
    Arc<Mutex<Vec<AddonTaskRequest>>>,
    Arc<Mutex<Option<String>>>,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let raw_token = Arc::new(Mutex::new(None::<String>));
    let router = Router::new().route(
        metadata_scraper::BULK_METADATA_SCRAPE_TASK_PATH,
        axum::routing::post({
            let requests = Arc::clone(&requests);
            let raw_token = Arc::clone(&raw_token);
            let nako_base_url = nako_base_url.clone();
            move |Json(request): Json<AddonTaskRequest>| {
                let requests = Arc::clone(&requests);
                let raw_token = Arc::clone(&raw_token);
                let nako_base_url = nako_base_url.clone();
                async move {
                    requests.lock().await.push(request.clone());
                    let raw_token = raw_token
                        .lock()
                        .await
                        .clone()
                        .expect("addon raw token must be configured before scan");
                    let client = reqwest::Client::new();
                    let mut items = Vec::new();
                    for item in request.payload["items"].as_array().unwrap() {
                        let writeback = item
                            .get("writeback")
                            .expect("writeback should be present in scan payload");
                        let side_effect = serde_json::json!({
                            "permission": "metadata_write",
                            "library_id": writeback["library_id"].clone(),
                            "target": writeback["target"].clone(),
                            "idempotency_key": writeback["idempotency_key"].clone(),
                            "provenance": {
                                "origin": "scan-writeback-test-sidecar",
                                "request_id": request.request_id.clone(),
                                "job_id": request.job_id.clone(),
                                "source_id": item["source_id"].clone()
                            },
                            "payload": {
                                "title": "Addon Scan Writeback Title",
                                "overview": "Merged through the Addon Side Effect runtime."
                            }
                        });
                        let response = client
                            .post(format!("{nako_base_url}/addon/v1/side-effects"))
                            .bearer_auth(&raw_token)
                            .json(&side_effect)
                            .send()
                            .await
                            .expect("test sidecar should submit metadata_write side effect");
                        let status = response.status().as_u16();
                        let response_body = response
                            .json::<serde_json::Value>()
                            .await
                            .expect("side effect response should be JSON");
                        items.push(serde_json::json!({
                            "source_id": item["source_id"].clone(),
                            "writeback_http_status": status,
                            "side_effect": response_body.get("side_effect").cloned()
                        }));
                    }

                    Json(AddonTaskResponse {
                        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                        addon_id: request.addon_id,
                        task_id: request.task_id,
                        job_id: request.job_id,
                        request_id: request.request_id,
                        output: serde_json::json!({
                            "accepted": true,
                            "items": items
                        }),
                    })
                }
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    tokio::task::yield_now().await;

    (format!("http://{addr}"), requests, raw_token)
}

async fn wait_for_addon_task_run_status(
    store: &NakoDatabase,
    job_id: JobId,
    expected: JobStatus,
) -> nako_core::AddonTaskRunRecord {
    for _ in 0..100 {
        if let Some(run) = store.get_addon_task_run(job_id).await.unwrap()
            && run.job.status == expected
        {
            return run;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    panic!("addon task run {job_id} did not reach {expected:?}");
}

async fn wait_for_continuation_task_run(
    store: &NakoDatabase,
    addon_id: AddonId,
    cursor: u64,
) -> nako_core::AddonTaskRunRecord {
    for _ in 0..100 {
        let runs = store
            .list_addon_task_runs(
                nako_core::AddonTaskRunListFilter {
                    addon_id: Some(addon_id),
                    declaration_id: Some(metadata_scraper::BULK_METADATA_SCRAPE_TASK_ID.to_owned()),
                    ..nako_core::AddonTaskRunListFilter::default()
                },
                PageRequest::new(20, 0),
            )
            .await
            .unwrap();
        if let Some(run) = runs
            .into_iter()
            .find(|run| addon_task_run_payload_cursor(run) == Some(cursor))
            && run.job.status == JobStatus::Succeeded
        {
            return run;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    panic!("addon continuation task run for cursor {cursor} did not succeed");
}

fn addon_task_run_payload_cursor(run: &nako_core::AddonTaskRunRecord) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(&run.input_json)
        .ok()?
        .get("payload")?
        .get("cursor")?
        .as_u64()
}

#[tokio::test]
async fn background_scan_job_acknowledges_cancellation_before_probe_stage() {
    let server = BlockingWebDavServer::start(BlockingWebDavControl::new()).await;
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

    let job = app
        .library_scan()
        .enqueue_library_scan(library_id)
        .await
        .unwrap();
    tokio::time::timeout(
        Duration::from_secs(5),
        server.control().wait_for_first_propfind(),
    )
    .await
    .unwrap();

    let requested = app.jobs().request_job_cancellation(job.id).await.unwrap();
    assert!(requested.requested);
    assert!(!requested.terminal);
    assert_eq!(requested.job.status, JobStatus::Running);

    server.control().release_first_propfind();
    let diagnostics = wait_for_runtime_jobs(&app, 0, 1, 0).await;
    let persisted = app.jobs().get_job(job.id).await.unwrap();
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    let events = store
        .list_outbox_events(Default::default(), PageRequest::new(100, 0))
        .await
        .unwrap();

    assert_eq!(diagnostics.failed_jobs, 0);
    assert_eq!(diagnostics.cancelled_jobs, 1);
    assert_eq!(persisted.status, JobStatus::Cancelled);
    assert_eq!(persisted.summary_json, None);
    assert_eq!(persisted.error, None);
    assert_eq!(sources.len(), 1);
    assert!(
        store
            .get_media_probe(sources[0].id)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(server.control().movie_gets(), 0);
    assert!(!events.iter().any(|event| {
        event.kind == DomainEventKind::LibraryScanned
            && event.subject == DomainEventSubject::Library(library_id)
    }));
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
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: anime_id,
                name: "Remote Anime".to_owned(),
                root: temp.path().join("unused-local-root"),
                preset: nako_core::LibraryPreset::Anime,
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
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
    assert_eq!(movies.options.preset, nako_core::LibraryPreset::Movies);
    assert_eq!(anime.name, "Remote Anime");
    assert_eq!(anime.roots, vec!["webdav:///Anime".to_owned()]);
    assert_eq!(anime.options.preset, nako_core::LibraryPreset::Anime);
}

#[tokio::test]
async fn app_startup_overwrites_persisted_library_with_configured_desired_state() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Old Movies".to_owned(),
            roots: vec!["local:///OldMovies".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();

    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Remote Anime".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: nako_core::LibraryPreset::Anime,
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
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let loaded = store.get_library(library_id).await.unwrap().unwrap();

    assert_eq!(app.startup_report().configured_libraries, 1);
    assert_eq!(loaded.name, "Remote Anime");
    assert_eq!(loaded.roots, vec!["webdav:///Anime".to_owned()]);
    assert_eq!(loaded.options.preset, nako_core::LibraryPreset::Anime);
}

#[tokio::test]
async fn metadata_profile_restart_preserves_admin_update_without_toml_profile_override() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    let mut admin_profile =
        nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    admin_profile.scan = nako_core::MetadataScanPolicy::disabled();
    admin_profile.local_metadata_policy = nako_core::LocalMetadataPolicy::Disabled;

    app.library()
        .update_admin_metadata_profile(
            library_id,
            nako_api::admin::AdminUpdateLibraryMetadataProfileRequest {
                profile: admin_profile.clone(),
            },
        )
        .await
        .unwrap();
    drop(app);

    let restarted = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let loaded = store.get_library(library_id).await.unwrap().unwrap();
    let response = restarted
        .library()
        .get_admin_metadata_profile(library_id)
        .await
        .unwrap();

    assert_eq!(loaded.options.metadata_profile, admin_profile);
    assert_eq!(response.profile, admin_profile);
    assert!(!response.scan_acquisition_plan.local_nfo_import);
}

#[tokio::test]
async fn metadata_profile_restart_toml_profile_override_replaces_admin_update() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let mut config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let mut configured_profile =
        nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    configured_profile.language = Some("zh-CN".to_owned());
    configured_profile.scan.addon_scrape = true;
    config
        .metadata
        .library_profiles
        .insert(library_id, configured_profile.clone());
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();
    let mut admin_profile =
        nako_core::MetadataProfile::from_preset(nako_core::LibraryPreset::Movies);
    admin_profile.scan = nako_core::MetadataScanPolicy::disabled();
    admin_profile.local_metadata_policy = nako_core::LocalMetadataPolicy::Disabled;

    app.library()
        .update_admin_metadata_profile(
            library_id,
            nako_api::admin::AdminUpdateLibraryMetadataProfileRequest {
                profile: admin_profile,
            },
        )
        .await
        .unwrap();
    drop(app);

    NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let loaded = store.get_library(library_id).await.unwrap().unwrap();

    assert_eq!(loaded.options.metadata_profile, configured_profile);
    assert!(loaded.options.metadata_profile.scan.addon_scrape);
}

#[tokio::test]
async fn admin_metadata_raw_cache_settings_survive_restart_as_admin_override() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let app = NakoApp::new_with_store(config.clone(), store.clone())
        .await
        .unwrap();

    app.update_admin_metadata_raw_cache_settings(
        nako_api::admin::AdminUpdateMetadataRawCacheSettingsRequest {
            retention_ms: 3_600_000,
            cleanup_on_startup: false,
        },
    )
    .await
    .unwrap();
    drop(app);

    let restarted = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    assert_eq!(
        restarted.config().metadata.raw_cache_retention_ms,
        3_600_000
    );
    assert!(
        !restarted
            .config()
            .metadata
            .maintenance
            .raw_cache_cleanup_on_startup
    );
}

#[tokio::test]
async fn app_startup_retains_persisted_library_missing_from_config() {
    let temp = tempfile::tempdir().unwrap();
    let retained_id = LibraryId::new();
    let configured_id = LibraryId::new();
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

    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: configured_id,
            name: "Configured Library".to_owned(),
            root: temp.path().join("configured"),
            preset: nako_core::LibraryPreset::Anime,
            webdav: None,
        }],
    );
    let app = NakoApp::new_with_store(config, store.clone())
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
    assert_eq!(retained.options.preset, nako_core::LibraryPreset::Movies);
    assert_eq!(configured.name, "Configured Library");
    assert_eq!(configured.roots, vec!["local:///".to_owned()]);
    assert_eq!(configured.options.preset, nako_core::LibraryPreset::Anime);
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
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store.upsert_library(&unchanged).await.unwrap();
    store
        .upsert_library(&Library {
            id: updated_id,
            name: "Old Anime".to_owned(),
            roots: vec!["webdav:///OldAnime".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Anime),
        })
        .await
        .unwrap();
    store
        .upsert_library(&Library {
            id: retained_id,
            name: "Retained Historical Library".to_owned(),
            roots: vec!["local:///Retained".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::MixedVideo),
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
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: updated_id,
                name: "Updated Anime".to_owned(),
                root: temp.path().join("unused-local-root"),
                preset: nako_core::LibraryPreset::Anime,
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
                preset: nako_core::LibraryPreset::HomeVideo,
                webdav: None,
            },
        ],
    );
    let app = NakoApp::new_with_store(config, store.clone())
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
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    );
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    store
        .upsert_library(&Library {
            id: library_id,
            name: "Persisted Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions {
                scan: nako_core::LibraryScanOptions {
                    max_depth: Some(0),
                    ..nako_core::LibraryScanOptions::default()
                },
                ..LibraryOptions::from_preset(nako_core::LibraryPreset::Movies)
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
        libraries: vec![
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().join("movies"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().join("anime"),
                preset: nako_core::LibraryPreset::Anime,
                webdav: None,
            },
        ],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let err = NakoApp::new_with_store(config, store).await.unwrap_err();

    let NakoError::InvalidInput { message } = err else {
        panic!("expected duplicate library id validation error");
    };
    assert!(message.contains("duplicate configured library id"));
    assert!(message.contains(&library_id.to_string()));
}

#[tokio::test]
async fn app_startup_rejects_duplicate_configured_library_roots() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("shared-root");
    let config = startup_config(
        temp.path(),
        vec![
            LocalLibraryConfig {
                id: LibraryId::new(),
                name: "Movies".to_owned(),
                root: root.clone(),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: LibraryId::new(),
                name: "Anime".to_owned(),
                root: root.clone(),
                preset: nako_core::LibraryPreset::Anime,
                webdav: None,
            },
        ],
    );
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let err = NakoApp::new_with_store(config, store).await.unwrap_err();

    let NakoError::InvalidInput { message } = err else {
        panic!("expected duplicate library root validation error");
    };
    assert!(message.contains("duplicate configured library root"));
    assert!(message.contains(&root.display().to_string()));
}

#[tokio::test]
async fn addon_event_scheduler_runtime_task_is_supervised_and_stops_on_shutdown() {
    let temp = tempfile::tempdir().unwrap();
    let library = LocalLibraryConfig {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        root: temp.path().join("movies"),
        preset: nako_core::LibraryPreset::Movies,
        webdav: None,
    };
    fs::create_dir_all(&library.root).unwrap();
    let mut config = startup_config(temp.path(), vec![library]);
    config.addon_event_scheduler.enabled = true;
    config.addon_event_scheduler.interval_ms = 1_000;

    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();

    assert!(app.startup_report().addon_event_scheduler_started);
    let diagnostics = app.runtime_diagnostics();
    assert_eq!(diagnostics.active_tasks, 1);
    assert_eq!(diagnostics.tasks[0].name, "addon_event_scheduler");
    assert_eq!(diagnostics.tasks[0].resource_class, "addon.event.scheduler");

    app.shutdown_runtime();
    tokio::task::yield_now().await;

    let diagnostics = app.runtime_diagnostics();
    assert!(diagnostics.shutdown_requested);
    assert_eq!(diagnostics.active_tasks, 0);
}

#[tokio::test]
async fn app_startup_rejects_unsupported_configured_webdav_root_scheme() {
    let temp = tempfile::tempdir().unwrap();
    let config = startup_config(
        temp.path(),
        vec![LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: Some(WebDavLibraryConfig {
                root: "ftp:///Movies".to_owned(),
                base_url: "https://webdav.example.test/dav".to_owned(),
                username: None,
                password_env: None,
                timeout_ms: 15_000,
                max_attempts: 4,
            }),
        }],
    );
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let err = NakoApp::new_with_store(config, store).await.unwrap_err();

    let NakoError::InvalidInput { message } = err else {
        panic!("expected unsupported WebDAV root scheme validation error");
    };
    assert_eq!(
        message,
        "configured WebDAV library root must use webdav scheme: ftp:///Movies"
    );
}

#[tokio::test]
async fn app_startup_allows_same_webdav_root_on_different_endpoints() {
    let temp = tempfile::tempdir().unwrap();
    let first_id = LibraryId::new();
    let second_id = LibraryId::new();
    let config = startup_config(
        temp.path(),
        vec![
            LocalLibraryConfig {
                id: first_id,
                name: "Remote Movies A".to_owned(),
                root: temp.path().join("unused-local-root-a"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: Some(WebDavLibraryConfig {
                    root: "webdav:///Movies".to_owned(),
                    base_url: "https://a.webdav.example.test/dav".to_owned(),
                    username: None,
                    password_env: None,
                    timeout_ms: 15_000,
                    max_attempts: 4,
                }),
            },
            LocalLibraryConfig {
                id: second_id,
                name: "Remote Movies B".to_owned(),
                root: temp.path().join("unused-local-root-b"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: Some(WebDavLibraryConfig {
                    root: "webdav:///Movies".to_owned(),
                    base_url: "https://b.webdav.example.test/dav".to_owned(),
                    username: None,
                    password_env: None,
                    timeout_ms: 15_000,
                    max_attempts: 4,
                }),
            },
        ],
    );
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();

    assert_eq!(app.startup_report().configured_libraries, 2);
    assert!(store.get_library(first_id).await.unwrap().is_some());
    assert!(store.get_library(second_id).await.unwrap().is_some());
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
        metadata,
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();

    let err = NakoApp::new_with_store(config, store).await.unwrap_err();

    let NakoError::InvalidInput { message } = err else {
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
    let request_identity = local_remux_request_identity(&source, RemuxContainer::Mp4);

    store
        .create_transcode_session(NewTranscodeSession {
            id: stale_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: RemuxRequestKey {
                source_id: source.id,
                request_identity: request_identity.clone(),
            }
            .persisted_request_key(),
            output_path: staging
                .output_path(source.id, &request_identity, RemuxContainer::Mp4)
                .unwrap(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    drop(app);
    let restarted = NakoApp::new_with_store(config, store.clone())
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
async fn app_startup_recovers_unfinished_jobs_and_preserves_queued_artwork_ingests() {
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

    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Startup Artwork".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    let fetching_artwork =
        create_startup_managed_artwork_ingest(&store, library_id, item.id, "startup-fetching")
            .await;
    let fetching_claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetching_claim.ingest.id, fetching_artwork.ingest.id);
    assert_eq!(
        fetching_claim.ingest.status,
        ManagedArtworkIngestStatus::Fetching
    );
    assert_eq!(fetching_claim.job.status, JobStatus::Running);
    let queued_artwork =
        create_startup_managed_artwork_ingest(&store, library_id, item.id, "startup-queued").await;

    drop(app);
    let restarted = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let queued = store.get_job(queued_id).await.unwrap().unwrap();
    let running = store.get_job(running_id).await.unwrap().unwrap();
    let succeeded = store.get_job(succeeded_id).await.unwrap().unwrap();
    let fetching_artwork_ingest = store
        .get_managed_artwork_ingest(fetching_artwork.ingest.id)
        .await
        .unwrap()
        .unwrap();
    let fetching_artwork_job = store
        .get_job(fetching_artwork.job.id)
        .await
        .unwrap()
        .unwrap();
    let queued_artwork_ingest = store
        .get_managed_artwork_ingest(queued_artwork.ingest.id)
        .await
        .unwrap()
        .unwrap();
    let queued_artwork_job = store.get_job(queued_artwork.job.id).await.unwrap().unwrap();

    assert_eq!(queued.status, JobStatus::Queued);
    assert_eq!(queued.error, None);
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(
        running.error,
        Some("job was unfinished during server startup".to_owned())
    );
    assert_eq!(succeeded.status, JobStatus::Succeeded);
    assert_eq!(
        fetching_artwork_ingest.status,
        ManagedArtworkIngestStatus::Failed
    );
    assert_eq!(
        fetching_artwork_ingest.failure_code.as_deref(),
        Some("startup_recovery")
    );
    assert_eq!(fetching_artwork_ingest.artifact_id, None);
    assert_eq!(fetching_artwork_job.status, JobStatus::Failed);
    assert_eq!(
        fetching_artwork_job.error.as_deref(),
        Some("managed artwork ingest was unfinished during server startup")
    );
    assert!(fetching_artwork_job.summary_json.is_some());
    assert_eq!(
        queued_artwork_ingest.status,
        ManagedArtworkIngestStatus::Queued
    );
    assert_eq!(queued_artwork_ingest.failure_code, None);
    assert_eq!(queued_artwork_job.status, JobStatus::Queued);
    assert_eq!(queued_artwork_job.error, None);
    assert_eq!(restarted.startup_report().recovered_jobs, 2);

    let requeued = store
        .requeue_managed_artwork_ingest(fetching_artwork.ingest.id)
        .await
        .unwrap();
    assert!(requeued.requeued);
    assert!(requeued.had_failure);
    assert_eq!(requeued.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(requeued.job.status, JobStatus::Queued);
}

#[tokio::test]
async fn startup_report_tracks_disabled_staging_cleanup() {
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
        staging: StagingConfig {
            cleanup_on_startup: false,
            ..StagingConfig::default()
        },
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
    let app = NakoApp::new_with_store(config, store).await.unwrap();

    assert_eq!(app.startup_report().configured_libraries, 1);
    assert_eq!(app.startup_report().staging_cleanup, None);
}
