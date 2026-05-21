use std::{fs, path::PathBuf};

use taru_core::{
    CanonicalMetadata, DatabaseLifecycle, ExternalProvider, Library, LibraryId, LibraryOptions,
    LibraryPreset, LibraryRepository, ManagedImportArtifactListFilter, ManagedImportArtifactState,
    ManagedImportPromotionBlockedReason, ManagedImportPromotionOperationKind,
    ManagedImportPromotionOperationStatus, ManagedImportRepository, ManagedImportSourceKind,
    MediaItem, MediaItemId, MediaKind, MediaRepository, MediaSource, MediaSourceId,
    NewStagingManifestRecord, PageRequest, StagingManifestId, StagingManifestRepository,
    StagingPurpose, StagingState,
};
use taru_db::TaruDatabase;

use crate::app::TaruApp;
use crate::app::managed_import::{CreateManagedImportArtifactRequest, ManagedImportAppService};
use crate::config::{
    AuthConfig, LocalLibraryConfig, MetadataConfig, PlaybackConfig, StagingConfig,
    TaruServerConfig, TranscodeConfig,
};

#[tokio::test]
async fn managed_import_service_creates_redacted_artifact_diagnostics_without_library_writes() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = seed_library(&store).await;
    let staging_manifest_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: staging_manifest_id,
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            source_scheme: "file".to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: "F:/Taru/cache/import/private-demo.mkv".to_owned(),
            size_bytes: Some(42),
            etag: Some("etag-private".to_owned()),
            fingerprint: Some("sha256-private-fingerprint".to_owned()),
            state: StagingState::Ready,
            created_at_ms: 1_000,
            updated_at_ms: 1_000,
            last_accessed_at_ms: 1_000,
            expires_at_ms: Some(10_000),
            active_leases: 0,
            validation_error: None,
        })
        .await
        .unwrap();
    let service = ManagedImportAppService::new(store.clone());

    let diagnostic = service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::WatchedCandidate,
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            staging_manifest_id: Some(staging_manifest_id),
            artifact_uri: Some("staging:///managed-import/private-demo.mkv".to_owned()),
            original_file_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: None,
            fingerprint: None,
            state: None,
            diagnostics_json: Some(r#"{"raw":"private"}"#.to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(diagnostic.target_library_id, library.id);
    assert_eq!(diagnostic.source_kind, "watched_candidate");
    assert_eq!(diagnostic.source_scheme.as_deref(), Some("file"));
    assert_eq!(diagnostic.source_uri_redacted, "file://<redacted>");
    assert_eq!(diagnostic.staging_manifest_id, Some(staging_manifest_id));
    assert_eq!(diagnostic.size_bytes, Some(42));
    assert!(diagnostic.has_fingerprint);
    assert!(diagnostic.has_artifact_uri);
    assert!(diagnostic.has_original_file_name);
    assert!(diagnostic.has_intended_locator);
    assert!(diagnostic.has_diagnostics);
    assert_eq!(diagnostic.state, ManagedImportArtifactState::Staged);

    let body = serde_json::to_string(&diagnostic).unwrap();
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("private-demo.mkv"));
    assert!(!body.contains("sha256-private-fingerprint"));
    assert!(!body.contains("raw"));
    assert!(!body.contains("Movies/Demo"));
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn managed_import_service_lists_library_scoped_redacted_diagnostics() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let first_library = seed_library(&store).await;
    let second_library = Library {
        id: LibraryId::new(),
        name: "Second Movies".to_owned(),
        roots: vec!["local:///Second Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&second_library).await.unwrap();
    let service = ManagedImportAppService::new(store.clone());

    service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: first_library.id,
            source_kind: ManagedImportSourceKind::OperatorUrl,
            source_uri: "https://example.test/private/first.mkv?apikey=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: Some("first.mkv".to_owned()),
            intended_locator: None,
            size_bytes: Some(11),
            fingerprint: Some("first-private-fingerprint".to_owned()),
            state: Some(ManagedImportArtifactState::Proposed),
            diagnostics_json: None,
        })
        .await
        .unwrap();
    service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: second_library.id,
            source_kind: ManagedImportSourceKind::OperatorUrl,
            source_uri: "https://example.test/private/second.mkv?apikey=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: Some("second.mkv".to_owned()),
            intended_locator: None,
            size_bytes: Some(22),
            fingerprint: Some("second-private-fingerprint".to_owned()),
            state: Some(ManagedImportArtifactState::Proposed),
            diagnostics_json: None,
        })
        .await
        .unwrap();

    let diagnostics = service
        .list_artifacts(
            ManagedImportArtifactListFilter {
                target_library_id: Some(first_library.id),
                state: Some(ManagedImportArtifactState::Proposed),
                source_kind: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();

    assert_eq!(diagnostics.returned, 1);
    assert_eq!(diagnostics.artifacts[0].target_library_id, first_library.id);
    assert_eq!(
        diagnostics.artifacts[0].source_uri_redacted,
        "https://<redacted>"
    );
    let body = serde_json::to_string(&diagnostics).unwrap();
    assert!(!body.contains("apikey=secret"));
    assert!(!body.contains("first.mkv"));
    assert!(!body.contains("second.mkv"));
    assert!(!body.contains("private-fingerprint"));
}

#[tokio::test]
async fn managed_import_service_rejects_mutating_creation_states() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = seed_library(&store).await;
    let service = ManagedImportAppService::new(store.clone());

    let err = service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::OperatorUrl,
            source_uri: "https://example.test/private/apply.mkv".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: None,
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            state: Some(ManagedImportArtifactState::Applying),
            diagnostics_json: None,
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("mutating lifecycle state"));
    assert!(
        store
            .list_managed_import_artifacts(
                ManagedImportArtifactListFilter::all(),
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn managed_import_promotion_preview_explains_destination_hints_without_library_writes() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Demo (2026)")).unwrap();
    fs::write(temp.path().join("incoming").join("Demo.mkv"), b"media!").unwrap();
    fs::write(
        temp.path()
            .join("Movies")
            .join("Demo (2026)")
            .join("Demo.nfo"),
        r#"<movie><title>Existing NFO</title></movie>"#,
    )
    .unwrap();
    let library_id = LibraryId::new();
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let duplicate_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Existing Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let duplicate_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: duplicate_item.id,
        locator: "local:///Existing/Demo.mkv".to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: Some("fingerprint-demo".to_owned()),
    };
    store.upsert_media_item(&duplicate_item).await.unwrap();
    store.upsert_media_source(&duplicate_source).await.unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Demo.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Demo.mkv".to_owned()),
            original_file_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(5),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();

    let plan = app
        .managed_import()
        .preview_promotion_plan(artifact.id)
        .await
        .unwrap();

    assert_eq!(plan.artifact_id, artifact.id);
    assert_eq!(plan.target_library_id, library_id);
    assert_eq!(
        plan.destination_locator.as_deref(),
        Some("local:///Movies/Demo (2026)/Demo.mkv")
    );
    assert!(plan.blocked_reasons.is_empty());
    assert!(plan.file_operations.iter().any(|operation| {
        operation.kind == ManagedImportPromotionOperationKind::Hardlink
            && operation.status == ManagedImportPromotionOperationStatus::Ready
    }));
    assert!(plan.file_operations.iter().any(|operation| {
        operation.kind == ManagedImportPromotionOperationKind::Symlink
            && operation.status == ManagedImportPromotionOperationStatus::Ready
    }));
    assert!(
        plan.duplicate_hints
            .iter()
            .any(|hint| hint.existing_source_id == Some(duplicate_source.id))
    );
    assert!(plan.nfo_authority.has_sidecar);
    assert!(plan.nfo_authority.import_supported);
    assert_eq!(
        plan.provider_identity.configured_providers,
        vec![ExternalProvider::Tmdb, ExternalProvider::Douban]
    );
    assert!(plan.provider_identity.has_import_diagnostics);
    assert!(
        !temp
            .path()
            .join("Movies")
            .join("Demo (2026)")
            .join("Demo.mkv")
            .exists()
    );
    assert_eq!(
        fs::read_to_string(
            temp.path()
                .join("Movies")
                .join("Demo (2026)")
                .join("Demo.nfo")
        )
        .unwrap(),
        r#"<movie><title>Existing NFO</title></movie>"#
    );
    assert_eq!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn managed_import_promotion_preview_reports_blockers_without_media_source_writes() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = seed_library(&store).await;
    let service = ManagedImportAppService::new(store.clone());
    let artifact = service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::OperatorUrl,
            source_uri: "https://example.test/private/Demo.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: None,
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            state: Some(ManagedImportArtifactState::Proposed),
            diagnostics_json: None,
        })
        .await
        .unwrap();

    let plan = service.preview_promotion_plan(artifact.id).await.unwrap();

    assert_eq!(
        plan.blocked_reasons,
        vec![
            ManagedImportPromotionBlockedReason::ArtifactNotReady,
            ManagedImportPromotionBlockedReason::MissingArtifactUri,
            ManagedImportPromotionBlockedReason::MissingDestinationLocator,
            ManagedImportPromotionBlockedReason::ProviderIdentityMissing
        ]
    );
    assert!(
        plan.file_operations.iter().all(|operation| {
            operation.status == ManagedImportPromotionOperationStatus::Blocked
        })
    );
    assert_eq!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap(),
        Vec::<MediaSource>::new()
    );
}

#[tokio::test]
async fn managed_import_promotion_preview_blocks_destination_escape() {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = seed_library(&store).await;
    let service = ManagedImportAppService::new(store.clone());
    let artifact = service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Demo.mkv".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Demo.mkv".to_owned()),
            original_file_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("local:///../outside/Demo.mkv".to_owned()),
            size_bytes: Some(5),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();

    let plan = service.preview_promotion_plan(artifact.id).await.unwrap();

    assert!(
        plan.blocked_reasons
            .contains(&ManagedImportPromotionBlockedReason::DestinationEscapesLibrary)
    );
    assert!(
        plan.file_operations.iter().all(|operation| {
            operation.status == ManagedImportPromotionOperationStatus::Blocked
        })
    );
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
}

async fn seed_library(store: &TaruDatabase) -> Library {
    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();
    library
}

fn managed_import_test_config(root: &std::path::Path, library_id: LibraryId) -> TaruServerConfig {
    TaruServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: AuthConfig::disabled(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: root.join("cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        }],
    }
}
