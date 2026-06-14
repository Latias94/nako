use std::{fs, path::PathBuf};

use async_trait::async_trait;
use nako_core::{
    CanonicalMetadata, DatabaseLifecycle, ExternalProvider, Library, LibraryId,
    LibraryItemRepository, LibraryOptions, LibraryPreset, LibraryRepository,
    ManagedImportArtifactListFilter, ManagedImportArtifactState, ManagedImportPromotionApplyState,
    ManagedImportPromotionBlockedReason, ManagedImportPromotionOperationKind,
    ManagedImportPromotionOperationStatus, ManagedImportRepository, ManagedImportSourceKind,
    MediaItem, MediaItemId, MediaKind, MediaRepository, MediaSource, MediaSourceId,
    NewStagingManifestRecord, PageRequest, SourceDuplicateRepository, StagingAttribution,
    StagingManifestId, StagingManifestRepository, StagingPurpose, StagingState, UserPrincipalId,
};
use nako_db::NakoDatabase;

use crate::app::NakoApp;
use crate::app::managed_import::{
    AcceptManagedImportPromotionRequest, ApplyManagedImportPromotionRequest,
    CreateManagedImportArtifactRequest, ManagedImportAppService, ManagedImportCatalogFailurePoint,
};
use crate::config::{
    AuthConfig, LocalLibraryConfig, MetadataConfig, NakoServerConfig, PlaybackConfig,
    StagingConfig, TranscodeConfig,
};
use nako_vfs::{
    ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile, StorageApplyKind,
    StorageApplyObject, StorageApplyReport, StorageApplyRequest, StorageApplyStatus,
    StorageBackend, StorageCapabilities, StorageLinkPlanRequest, StorageUri, VirtualFile,
};

#[tokio::test]
async fn managed_import_service_creates_redacted_artifact_diagnostics_without_library_writes() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = seed_library(&store).await;
    let staging_manifest_id = StagingManifestId::new();
    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: staging_manifest_id,
            attribution: StagingAttribution::unknown(),
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            source_scheme: "file".to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: "F:/Nako/cache/import/private-demo.mkv".to_owned(),
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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

#[tokio::test]
async fn managed_import_accepts_promotion_plan_with_idempotent_replay_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Demo (2026)")).unwrap();
    fs::write(temp.path().join("incoming").join("Demo.mkv"), b"media!").unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
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

    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Hardlink,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();
    let replayed = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Hardlink,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();

    assert_eq!(accepted.id, replayed.id);
    assert!(replayed.replayed);
    assert_eq!(accepted.state, ManagedImportPromotionApplyState::Accepted);
    assert_eq!(
        accepted.operation_kind,
        ManagedImportPromotionOperationKind::Hardlink
    );
    assert_eq!(accepted.target_library_id, library_id);
    assert_eq!(accepted.source_scheme.as_deref(), Some("local"));
    assert_eq!(
        accepted.destination_locator.as_deref(),
        Some("local:///Movies/Demo (2026)/Demo.mkv")
    );
    assert!(accepted.accepted_plan_snapshot);
    assert!(!accepted.has_raw_source_uri);
    assert!(!accepted.has_raw_fingerprint);
    assert!(
        !temp
            .path()
            .join("Movies")
            .join("Demo (2026)")
            .join("Demo.mkv")
            .exists()
    );
    assert!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn managed_import_applies_accepted_promotion_after_storage_target_is_durable() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Demo (2026)")).unwrap();
    fs::write(temp.path().join("incoming").join("Demo.mkv"), b"media!").unwrap();
    let target_path = temp
        .path()
        .join("Movies")
        .join("Demo (2026)")
        .join("Demo.mkv");
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
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
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-apply-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();

    assert!(!target_path.exists());

    let applied = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();

    assert_eq!(applied.id, accepted.id);
    assert_eq!(applied.state, ManagedImportPromotionApplyState::Promoted);
    assert_eq!(
        applied.safe_message.as_deref(),
        Some("promotion applied and catalog source committed")
    );
    assert_eq!(fs::read(target_path).unwrap(), b"media!");

    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.len(), 1);
    let source = &sources[0];
    assert_eq!(source.locator, "local:///Movies/Demo (2026)/Demo.mkv");
    assert_eq!(source.file_name, "Demo.mkv");
    assert_eq!(source.size_bytes, Some(6));
    assert_eq!(source.fingerprint.as_deref(), Some("fingerprint-demo"));

    let item = store.get_media_item(source.item_id).await.unwrap().unwrap();
    assert_eq!(item.kind, MediaKind::Movie);
    assert_eq!(item.metadata.title, "Demo");
    assert_eq!(item.parent_id, None);
    assert_eq!(
        store
            .get_library_item_state(library_id, item.id)
            .await
            .unwrap()
            .unwrap()
            .provisional,
        false
    );

    let promoted_artifact = store
        .get_managed_import_artifact(artifact.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        promoted_artifact.state,
        ManagedImportArtifactState::Promoted
    );

    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        apply_record.state,
        ManagedImportPromotionApplyState::Promoted
    );
    let outcome: serde_json::Value =
        serde_json::from_str(apply_record.outcome_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["writes_library"], true);
    assert_eq!(outcome["storage_mutation"], true);
    assert_eq!(outcome["media_source_mutation"], true);
    assert_eq!(outcome["target_created"], true);
    assert_eq!(outcome["duplicate_relationship_count"], 0);

    let body = serde_json::to_string(&applied).unwrap();
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("incoming/Demo.mkv"));
    assert!(!body.contains("fingerprint-demo"));
}

#[tokio::test]
async fn managed_import_acceptance_rejects_mismatched_replay_and_blocked_plan() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = seed_library(&store).await;
    let service = ManagedImportAppService::new(store.clone());
    let ready = service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Ready.mkv".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Ready.mkv".to_owned()),
            original_file_name: Some("Ready.mkv".to_owned()),
            intended_locator: Some("Movies/Ready (2026)/Ready.mkv".to_owned()),
            size_bytes: Some(5),
            fingerprint: Some("fingerprint-ready".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    service
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: ready.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-conflict".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();

    let mismatch = service
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: ready.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-conflict".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Hardlink,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap_err();
    assert!(mismatch.to_string().contains("idempotency key"));

    let blocked = service
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::OperatorUrl,
            source_uri: "https://example.test/private/Blocked.mkv?token=secret".to_owned(),
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

    let blocked_err = service
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: blocked.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-blocked".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap_err();
    assert!(
        blocked_err
            .to_string()
            .contains("promotion plan is blocked")
    );
    assert!(
        store
            .list_managed_import_promotion_applies_for_artifact(
                blocked.id,
                PageRequest::first_page()
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn managed_import_apply_records_pre_mutation_failure_without_catalog_writes() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Failure (2026)")).unwrap();
    fs::write(temp.path().join("incoming").join("Failure.mkv"), b"media!").unwrap();
    let target_path = temp
        .path()
        .join("Movies")
        .join("Failure (2026)")
        .join("Failure.mkv");
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Failure.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Failure.mkv".to_owned()),
            original_file_name: Some("Failure.mkv".to_owned()),
            intended_locator: Some("Movies/Failure (2026)/Failure.mkv".to_owned()),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-failure".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-failure-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();
    fs::remove_file(temp.path().join("incoming").join("Failure.mkv")).unwrap();

    let err = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("storage_apply_source_missing"));
    assert!(!target_path.exists());
    assert!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    let artifact_after = store
        .get_managed_import_artifact(artifact.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(artifact_after.state, ManagedImportArtifactState::Staged);
    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        apply_record.state,
        ManagedImportPromotionApplyState::FailedBeforeMutation
    );
    assert_eq!(
        apply_record.safe_error_code.as_deref(),
        Some("storage_apply_source_missing")
    );
    let outcome: serde_json::Value =
        serde_json::from_str(apply_record.outcome_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["writes_library"], false);
    assert_eq!(outcome["storage_mutation"], false);
    assert_eq!(outcome["media_source_mutation"], false);
    assert_eq!(outcome["target_created"], false);
}

#[tokio::test]
async fn managed_import_apply_rejects_stale_acceptance_before_storage_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Stale (2026)")).unwrap();
    fs::write(temp.path().join("incoming").join("Stale.mkv"), b"media!").unwrap();
    let target_path = temp
        .path()
        .join("Movies")
        .join("Stale (2026)")
        .join("Stale.mkv");
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Stale.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Stale.mkv".to_owned()),
            original_file_name: Some("Stale.mkv".to_owned()),
            intended_locator: Some("Movies/Stale (2026)/Stale.mkv".to_owned()),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-stale".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-stale-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();
    store
        .set_managed_import_artifact_state(
            artifact.id,
            ManagedImportArtifactState::Rejected,
            42,
            Some(r#"{"provider_candidates":1}"#.to_owned()),
        )
        .await
        .unwrap();

    let err = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("promotion plan is blocked"));
    assert!(!target_path.exists());
    assert!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        apply_record.state,
        ManagedImportPromotionApplyState::FailedBeforeMutation
    );
    assert_eq!(
        apply_record.safe_error_code.as_deref(),
        Some("promotion_apply_revalidation_failed")
    );
    let outcome: serde_json::Value =
        serde_json::from_str(apply_record.outcome_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["writes_library"], false);
    assert_eq!(outcome["storage_mutation"], false);
    assert_eq!(outcome["media_source_mutation"], false);
}

#[tokio::test]
async fn managed_import_apply_rejects_cataloged_destination_before_storage_mutation() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Cataloged (2026)")).unwrap();
    fs::write(
        temp.path().join("incoming").join("Cataloged.mkv"),
        b"media!",
    )
    .unwrap();
    let target_path = temp
        .path()
        .join("Movies")
        .join("Cataloged (2026)")
        .join("Cataloged.mkv");
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Cataloged.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Cataloged.mkv".to_owned()),
            original_file_name: Some("Cataloged.mkv".to_owned()),
            intended_locator: Some("Movies/Cataloged (2026)/Cataloged.mkv".to_owned()),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-cataloged".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-cataloged-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Cataloged".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: item.id,
            locator: "local:///Movies/Cataloged (2026)/Cataloged.mkv".to_owned(),
            file_name: "Cataloged.mkv".to_owned(),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-cataloged".to_owned()),
        })
        .await
        .unwrap();

    let err = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("already cataloged"));
    assert!(!target_path.exists());
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.len(), 1);
    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        apply_record.state,
        ManagedImportPromotionApplyState::FailedBeforeMutation
    );
    assert_eq!(
        apply_record.safe_error_code.as_deref(),
        Some("promotion_apply_destination_already_cataloged")
    );
}

#[tokio::test]
async fn managed_import_apply_replays_promoted_record_and_commits_duplicate_evidence() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Duplicate (2026)")).unwrap();
    fs::write(
        temp.path().join("incoming").join("Duplicate.mkv"),
        b"media!",
    )
    .unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
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
            title: "Existing Duplicate".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let duplicate_source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: duplicate_item.id,
        locator: "local:///Existing/Duplicate.mkv".to_owned(),
        file_name: "Duplicate.mkv".to_owned(),
        size_bytes: Some(6),
        fingerprint: Some("fingerprint-duplicate".to_owned()),
    };
    store.upsert_media_item(&duplicate_item).await.unwrap();
    store.upsert_media_source(&duplicate_source).await.unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Duplicate.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Duplicate.mkv".to_owned()),
            original_file_name: Some("Duplicate.mkv".to_owned()),
            intended_locator: Some("Movies/Duplicate (2026)/Duplicate.mkv".to_owned()),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-duplicate".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-duplicate-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();

    let applied = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    let replayed = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();

    assert_eq!(applied.id, replayed.id);
    assert!(replayed.replayed);
    assert_eq!(replayed.state, ManagedImportPromotionApplyState::Promoted);
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.len(), 2);
    let promoted_source = sources
        .iter()
        .find(|source| source.locator == "local:///Movies/Duplicate (2026)/Duplicate.mkv")
        .unwrap();
    let relationships = store
        .list_source_duplicate_relationships(promoted_source.id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(relationships.len(), 1);
    assert!(
        (relationships[0].source_id == promoted_source.id
            && relationships[0].duplicate_source_id == duplicate_source.id)
            || (relationships[0].source_id == duplicate_source.id
                && relationships[0].duplicate_source_id == promoted_source.id)
    );
    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    let outcome: serde_json::Value =
        serde_json::from_str(apply_record.outcome_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["duplicate_hint_count"], 1);
    assert_eq!(outcome["duplicate_relationship_count"], 1);
}

#[tokio::test]
async fn managed_import_apply_uses_storage_backend_apply_boundary() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Boundary (2026)")).unwrap();
    fs::write(temp.path().join("incoming").join("Boundary.mkv"), b"media!").unwrap();
    let target_path = temp
        .path()
        .join("Movies")
        .join("Boundary (2026)")
        .join("Boundary.mkv");
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Boundary.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Boundary.mkv".to_owned()),
            original_file_name: Some("Boundary.mkv".to_owned()),
            intended_locator: Some("Movies/Boundary (2026)/Boundary.mkv".to_owned()),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-boundary".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-boundary-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: LibraryPreset::Movies,
                webdav: None,
            },
            std::sync::Arc::new(ApplyOnlySuccessBackend),
        )
        .await;

    let applied = app
        .managed_import()
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();

    assert_eq!(applied.state, ManagedImportPromotionApplyState::Promoted);
    assert!(!target_path.exists());
    let sources = store
        .list_media_sources(library_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(
        sources[0].locator,
        "local:///Movies/Boundary (2026)/Boundary.mkv"
    );
}

#[tokio::test]
async fn managed_import_apply_cleans_storage_target_when_catalog_commit_fails() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Cleanup Complete (2026)")).unwrap();
    fs::write(
        temp.path().join("incoming").join("Cleanup Complete.mkv"),
        b"media!",
    )
    .unwrap();
    let target_path = temp
        .path()
        .join("Movies")
        .join("Cleanup Complete (2026)")
        .join("Cleanup Complete.mkv");
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Cleanup%20Complete.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Cleanup Complete.mkv".to_owned()),
            original_file_name: Some("Cleanup Complete.mkv".to_owned()),
            intended_locator: Some(
                "Movies/Cleanup Complete (2026)/Cleanup Complete.mkv".to_owned(),
            ),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-cleanup-complete".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-cleanup-complete-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();

    let service = app
        .managed_import()
        .with_catalog_failure_for_test(ManagedImportCatalogFailurePoint::BeforeMediaItem);
    let err = service
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("injected catalog commit failure"));
    assert!(!target_path.exists());
    assert!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    let artifact_after = store
        .get_managed_import_artifact(artifact.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(artifact_after.state, ManagedImportArtifactState::Staged);
    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        apply_record.state,
        ManagedImportPromotionApplyState::CleanupComplete
    );
    assert_eq!(
        apply_record.safe_error_code.as_deref(),
        Some("promotion_apply_catalog_commit_failed_cleanup_complete")
    );
    let outcome: serde_json::Value =
        serde_json::from_str(apply_record.outcome_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["writes_library"], false);
    assert_eq!(outcome["storage_mutation"], true);
    assert_eq!(outcome["media_source_mutation"], false);
    assert_eq!(outcome["target_created"], true);
    assert_eq!(outcome["catalog_commit_completed"], false);
    assert_eq!(outcome["storage_cleanup_attempted"], true);
    assert_eq!(outcome["storage_cleanup_complete"], true);
    assert_eq!(outcome["cleanup_status"], "cleaned");

    let replayed = service
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    assert!(replayed.replayed);
    assert_eq!(
        replayed.state,
        ManagedImportPromotionApplyState::CleanupComplete
    );
}

#[tokio::test]
async fn managed_import_apply_records_cleanup_pending_when_storage_cleanup_is_unsupported() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("incoming")).unwrap();
    fs::create_dir_all(temp.path().join("Movies").join("Cleanup Pending (2026)")).unwrap();
    fs::write(
        temp.path().join("incoming").join("Cleanup Pending.mkv"),
        b"media!",
    )
    .unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        managed_import_test_config(temp.path(), library_id),
        store.clone(),
    )
    .await
    .unwrap();
    let artifact = app
        .managed_import()
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library_id,
            source_kind: ManagedImportSourceKind::LocalFile,
            source_uri: "file:///operator/private/Cleanup%20Pending.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Cleanup Pending.mkv".to_owned()),
            original_file_name: Some("Cleanup Pending.mkv".to_owned()),
            intended_locator: Some("Movies/Cleanup Pending (2026)/Cleanup Pending.mkv".to_owned()),
            size_bytes: Some(6),
            fingerprint: Some("fingerprint-cleanup-pending".to_owned()),
            state: Some(ManagedImportArtifactState::Staged),
            diagnostics_json: Some(r#"{"provider_candidates":1}"#.to_owned()),
        })
        .await
        .unwrap();
    let accepted = app
        .managed_import()
        .accept_promotion(AcceptManagedImportPromotionRequest {
            artifact_id: artifact.id,
            requested_by: UserPrincipalId::local_admin(),
            idempotency_key: "accept-cleanup-pending-demo-1".to_owned(),
            operation_kind: ManagedImportPromotionOperationKind::Copy,
            accepted_blocked_reasons: Vec::new(),
        })
        .await
        .unwrap();
    app.storage()
        .replace_backend_for_test(
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: LibraryPreset::Movies,
                webdav: None,
            },
            std::sync::Arc::new(ApplyOnlySuccessBackend),
        )
        .await;

    let service = app
        .managed_import()
        .with_catalog_failure_for_test(ManagedImportCatalogFailurePoint::BeforeMediaItem);
    let err = service
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("injected catalog commit failure"));
    assert!(
        store
            .list_media_sources(library_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    let artifact_after = store
        .get_managed_import_artifact(artifact.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        artifact_after.state,
        ManagedImportArtifactState::CleanupPending
    );
    let apply_record = store
        .get_managed_import_promotion_apply(accepted.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        apply_record.state,
        ManagedImportPromotionApplyState::CleanupPending
    );
    assert_eq!(
        apply_record.safe_error_code.as_deref(),
        Some("promotion_apply_catalog_commit_failed_cleanup_pending")
    );
    let outcome: serde_json::Value =
        serde_json::from_str(apply_record.outcome_json.as_deref().unwrap()).unwrap();
    assert_eq!(outcome["writes_library"], false);
    assert_eq!(outcome["storage_mutation"], true);
    assert_eq!(outcome["media_source_mutation"], false);
    assert_eq!(outcome["target_created"], true);
    assert_eq!(outcome["catalog_commit_completed"], false);
    assert_eq!(outcome["storage_cleanup_attempted"], true);
    assert_eq!(outcome["storage_cleanup_complete"], false);
    assert_eq!(outcome["cleanup_status"], "unsupported");

    let replayed = service
        .apply_promotion(ApplyManagedImportPromotionRequest {
            apply_id: accepted.id,
            requested_by: UserPrincipalId::local_admin(),
        })
        .await
        .unwrap();
    assert!(replayed.replayed);
    assert_eq!(
        replayed.state,
        ManagedImportPromotionApplyState::CleanupPending
    );
}

async fn seed_library(store: &NakoDatabase) -> Library {
    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();
    library
}

struct ApplyOnlySuccessBackend;

#[async_trait]
impl StorageBackend for ApplyOnlySuccessBackend {
    fn scheme(&self) -> &'static str {
        "local"
    }

    async fn stat(&self, uri: &StorageUri) -> nako_core::Result<ObjectMetadata> {
        Err(nako_core::NakoError::storage_unknown(
            uri.to_string(),
            "apply-only backend does not expose stat",
        ))
    }

    async fn list(&self, uri: &StorageUri) -> nako_core::Result<Vec<ObjectMetadata>> {
        Err(nako_core::NakoError::storage_unknown(
            uri.to_string(),
            "apply-only backend does not expose list",
        ))
    }

    async fn open_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> nako_core::Result<VirtualFile> {
        Err(nako_core::NakoError::storage_unknown(
            format!("{}:{range:?}", uri),
            "apply-only backend does not expose open_range",
        ))
    }

    async fn read_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> nako_core::Result<ReadRange> {
        Err(nako_core::NakoError::storage_unknown(
            format!("{}:{range:?}", uri),
            "apply-only backend does not expose read_range",
        ))
    }

    async fn stream_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> nako_core::Result<ReadStream> {
        Err(nako_core::NakoError::storage_unknown(
            format!("{}:{range:?}", uri),
            "apply-only backend does not expose stream_range",
        ))
    }

    async fn read_to_string(&self, uri: &StorageUri) -> nako_core::Result<String> {
        Err(nako_core::NakoError::storage_unknown(
            uri.to_string(),
            "apply-only backend does not expose read_to_string",
        ))
    }

    async fn write_string(&self, uri: &StorageUri, _content: &str) -> nako_core::Result<()> {
        Err(nako_core::NakoError::storage_unknown(
            uri.to_string(),
            "apply-only backend does not expose write_string",
        ))
    }

    async fn stage(&self, request: StageRequest) -> nako_core::Result<StagedFile> {
        Err(nako_core::NakoError::storage_unknown(
            request.uri.to_string(),
            "apply-only backend does not expose stage",
        ))
    }

    async fn plan_link(
        &self,
        request: StorageLinkPlanRequest,
    ) -> nako_core::Result<nako_vfs::StorageLinkPlan> {
        Ok(nako_vfs::StorageLinkPlan {
            source_uri: request.source_uri,
            target_uri: request.target_uri,
            kind: request.kind,
            status: nako_vfs::StorageLinkPlanStatus::Unsupported,
            can_apply: false,
            source: None,
            target: None,
            message: "apply-only backend does not expose link planning".to_owned(),
        })
    }

    async fn apply(&self, request: StorageApplyRequest) -> nako_core::Result<StorageApplyReport> {
        assert_eq!(request.kind, StorageApplyKind::Copy);
        assert!(request.source_uri.as_str().starts_with("local://"));
        assert!(request.target_uri.as_str().starts_with("local:///Movies/"));
        let source = StorageApplyObject {
            uri: request.source_uri.clone(),
            kind: nako_vfs::ObjectKind::File,
            len: Some(6),
            etag: None,
            fingerprint_available: true,
            capabilities: StorageCapabilities::RANGE_READABLE,
        };
        let target = StorageApplyObject {
            uri: request.target_uri.clone(),
            kind: nako_vfs::ObjectKind::File,
            len: Some(6),
            etag: None,
            fingerprint_available: true,
            capabilities: StorageCapabilities::RANGE_READABLE,
        };

        Ok(StorageApplyReport {
            source_uri: request.source_uri,
            target_uri: request.target_uri,
            kind: request.kind,
            status: StorageApplyStatus::Applied,
            applied: true,
            target_created: true,
            source: Some(source),
            target: Some(target),
            message: "apply-only backend accepted promotion".to_owned(),
        })
    }
}

fn managed_import_test_config(root: &std::path::Path, library_id: LibraryId) -> NakoServerConfig {
    NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: AuthConfig::disabled(),
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
