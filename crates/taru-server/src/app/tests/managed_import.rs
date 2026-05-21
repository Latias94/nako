use taru_core::{
    DatabaseLifecycle, Library, LibraryId, LibraryOptions, LibraryPreset, LibraryRepository,
    ManagedImportArtifactListFilter, ManagedImportArtifactState, ManagedImportRepository,
    ManagedImportSourceKind, MediaRepository, NewStagingManifestRecord, PageRequest,
    StagingManifestId, StagingManifestRepository, StagingPurpose, StagingState,
};
use taru_db::TaruDatabase;

use crate::app::managed_import::{CreateManagedImportArtifactRequest, ManagedImportAppService};

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
