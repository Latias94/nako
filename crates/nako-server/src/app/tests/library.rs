use super::*;
use crate::app::LibraryScanTraceContext;
use nako_core::{IngestionFailurePhase, JobListFilter};

#[tokio::test]
async fn library_service_enforces_browse_access_for_public_reads() {
    let temp = tempfile::tempdir().unwrap();
    let allowed_root = temp.path().join("allowed");
    let blocked_root = temp.path().join("blocked");
    fs::create_dir_all(&allowed_root).unwrap();
    fs::create_dir_all(&blocked_root).unwrap();
    let allowed_library_id = LibraryId::new();
    let blocked_library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        library_access_config(
            allowed_library_id,
            allowed_root,
            blocked_library_id,
            blocked_root,
            temp.path().join("nako-cache").join("remux"),
        ),
        store.clone(),
    )
    .await
    .unwrap();
    let allowed_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Allowed Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let allowed_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: allowed_library_id,
        item_id: allowed_item.id,
        locator: "local:///Allowed Movie.mkv".to_owned(),
        file_name: "Allowed Movie.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&allowed_item).await.unwrap();
    store.upsert_media_source(&allowed_source).await.unwrap();
    let principal =
        local_library_principal_with_access(&store, allowed_library_id, LibraryAccessLevel::Browse)
            .await;

    let libraries = app
        .library()
        .list_libraries_for_browse(&principal, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(libraries.page.returned, 1);
    assert_eq!(libraries.libraries[0].id, allowed_library_id.to_string());

    let library = app
        .library()
        .get_library_for_browse(&principal, allowed_library_id)
        .await
        .unwrap();
    assert_eq!(library.library.id, allowed_library_id.to_string());

    let sources = app
        .library()
        .list_library_sources_for_browse(&principal, allowed_library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.sources.len(), 1);
    assert_eq!(sources.sources[0].source.id, allowed_source.id.to_string());

    let hidden_library = app
        .library()
        .get_library_for_browse(&principal, blocked_library_id)
        .await
        .unwrap_err();
    assert_library_browse_forbidden(hidden_library);

    let hidden_sources = app
        .library()
        .list_library_sources_for_browse(&principal, blocked_library_id, PageRequest::first_page())
        .await
        .unwrap_err();
    assert_library_browse_forbidden(hidden_sources);

    let hidden_items = app
        .library()
        .list_library_items_for_browse(
            &principal,
            blocked_library_id,
            nako_core::LibraryItemBrowseQuery::default(),
        )
        .await
        .unwrap_err();
    assert_library_not_found(hidden_items, blocked_library_id);
}

#[tokio::test]
async fn library_manage_commands_require_manage_access_in_app_services() {
    let temp = tempfile::tempdir().unwrap();
    let allowed_root = temp.path().join("allowed");
    let blocked_root = temp.path().join("blocked");
    fs::create_dir_all(&allowed_root).unwrap();
    fs::create_dir_all(&blocked_root).unwrap();
    let allowed_library_id = LibraryId::new();
    let blocked_library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        library_access_config(
            allowed_library_id,
            allowed_root,
            blocked_library_id,
            blocked_root,
            temp.path().join("nako-cache").join("remux"),
        ),
        store.clone(),
    )
    .await
    .unwrap();
    let principal =
        local_library_principal_with_access(&store, allowed_library_id, LibraryAccessLevel::Browse)
            .await;

    let trace_context = LibraryScanTraceContext::from_request_id("REQ-MANAGE_123").unwrap();
    let scan = app
        .library_scan()
        .enqueue_library_scan_with_trace_context_for_manage(
            &principal,
            allowed_library_id,
            trace_context,
        )
        .await
        .unwrap_err();
    assert_library_manage_forbidden(scan);

    let import = app
        .nfo()
        .enqueue_nfo_import_for_manage(&principal, allowed_library_id)
        .await
        .unwrap_err();
    assert_library_manage_forbidden(import);

    let export = app
        .nfo()
        .enqueue_nfo_export_for_manage(&principal, allowed_library_id)
        .await
        .unwrap_err();
    assert_library_manage_forbidden(export);

    let failures = app
        .library()
        .list_ingestion_failures_for_manage(
            &principal,
            allowed_library_id,
            None,
            None,
            PageRequest::first_page(),
        )
        .await
        .unwrap_err();
    assert_library_manage_forbidden(failures);

    let ignored = app
        .library()
        .ignore_ingestion_failure_for_manage(
            &principal,
            allowed_library_id,
            IngestionFailurePhase::Scan,
            "local:///Movies/Broken/",
        )
        .await
        .unwrap_err();
    assert_library_manage_forbidden(ignored);

    let jobs = store
        .list_jobs(JobListFilter::default(), PageRequest::first_page())
        .await
        .unwrap();
    assert!(jobs.is_empty());
}

fn library_access_config(
    allowed_library_id: LibraryId,
    allowed_root: PathBuf,
    blocked_library_id: LibraryId,
    blocked_root: PathBuf,
    remux_staging_root: PathBuf,
) -> NakoServerConfig {
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root,
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![
            LocalLibraryConfig {
                id: allowed_library_id,
                name: "Allowed".to_owned(),
                root: allowed_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: blocked_library_id,
                name: "Blocked".to_owned(),
                root: blocked_root,
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
        ],
    }
}

async fn local_library_principal_with_access(
    store: &NakoDatabase,
    library_id: LibraryId,
    access: LibraryAccessLevel,
) -> AuthenticatedPrincipal {
    let user_id = UserId::new();
    let principal_id = UserPrincipalId::new(format!("local-user:{user_id}")).unwrap();
    let user = User {
        id: user_id,
        principal_id: principal_id.clone(),
        username: format!("library-viewer-{user_id}"),
        display_name: "Library viewer".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1,
        updated_at_ms: 1,
    };

    store.upsert_user(&user).await.unwrap();
    store
        .replace_role_assignments(
            user_id,
            &[RoleAssignment {
                user_id,
                role: UserRole::Viewer,
                granted_at_ms: 1,
            }],
        )
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&LibraryAccessPolicy {
            scope: LibraryAccessPolicyScope::User(user_id),
            library_id,
            access,
            created_at_ms: 1,
            updated_at_ms: 1,
        })
        .await
        .unwrap();

    AuthenticatedPrincipal {
        user_id,
        principal_id,
        roles: vec![UserRole::Viewer],
        bootstrap: false,
    }
}

fn assert_library_browse_forbidden(error: NakoError) {
    let NakoError::Forbidden { message } = error else {
        panic!("expected library browse forbidden");
    };
    assert!(message.contains("required Library Access level 'browse'"));
}

fn assert_library_manage_forbidden(error: NakoError) {
    let NakoError::Forbidden { message } = error else {
        panic!("expected library manage forbidden");
    };
    assert!(message.contains("required Library Access level 'manage'"));
}

fn assert_library_not_found(error: NakoError, library_id: LibraryId) {
    let NakoError::NotFound { entity, id } = error else {
        panic!("expected library not found");
    };
    assert_eq!(entity, "library");
    assert_eq!(id, library_id.to_string());
}
