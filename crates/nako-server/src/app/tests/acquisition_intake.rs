use super::*;
use crate::app::acquisition_intake::{
    AcceptAcquisitionIntakeCandidateRequest, DiscoverWatchFolderCandidatesRequest,
    RecordAcquisitionIntakeCandidateRequest, RecordResourceSearchSelectionRequest,
};
use crate::app::managed_import::{CreateManagedImportArtifactRequest, ManagedImportAppService};
use nako_addon_protocol::{AddonResourceLink, AddonResourceLinkType, AddonResourceSearchResult};
use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateListFilter,
    AcquisitionIntakeCandidateState, AcquisitionIntakeRepository, AcquisitionIntakeSourceKind,
    AddonId, LibraryPreset, ManagedImportArtifactListFilter, ManagedImportArtifactState,
    ManagedImportRepository, ManagedImportSourceKind, NakoError, NewAcquisitionIntakeCandidate,
};
use nako_vfs::{LocalFsBackend, StorageBackend};

#[tokio::test]
async fn acquisition_intake_service_records_and_lists_redacted_watch_folder_candidates_without_library_writes()
 {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();

    let first = service
        .record_candidate(RecordAcquisitionIntakeCandidateRequest {
            id: None,
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::WatchFolder,
            source_key: "F:/incoming/private/Demo.mkv?token=secret".to_owned(),
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            display_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(42),
            fingerprint: Some("sha256-private-fingerprint".to_owned()),
            managed_import_artifact_id: None,
            state: None,
            diagnostics_json: Some(r#"{"raw":"private"}"#.to_owned()),
        })
        .await
        .unwrap();
    let replayed = service
        .record_candidate(RecordAcquisitionIntakeCandidateRequest {
            id: None,
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::WatchFolder,
            source_key: "F:/incoming/private/Demo.mkv?token=secret".to_owned(),
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            display_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(42),
            fingerprint: Some("sha256-private-fingerprint".to_owned()),
            managed_import_artifact_id: None,
            state: None,
            diagnostics_json: Some(r#"{"raw":"private"}"#.to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(first.id, replayed.id);
    assert_eq!(first.target_library_id, library.id);
    assert_eq!(first.source_kind, "watch_folder");
    assert_eq!(first.source_scheme.as_deref(), Some("file"));
    assert_eq!(first.source_uri_redacted, "file://<redacted>");
    assert!(first.source_key_fingerprint.starts_with("sha256:"));
    assert_ne!(
        first.source_key_fingerprint,
        "F:/incoming/private/Demo.mkv?token=secret"
    );
    assert!(first.has_display_name);
    assert!(first.has_intended_locator);
    assert_eq!(first.size_bytes, Some(42));
    assert!(first.has_fingerprint);
    assert!(first.has_diagnostics);
    assert_eq!(first.state, AcquisitionIntakeCandidateState::Discovered);

    let listed = service
        .list_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: Some(AcquisitionIntakeCandidateState::Discovered),
                source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();

    assert_eq!(listed.returned, 1);
    assert_eq!(listed.candidates[0].id, first.id);
    let body = serde_json::to_string(&listed).unwrap();
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("incoming/private"));
    assert!(!body.contains("Demo.mkv"));
    assert!(!body.contains("sha256-private-fingerprint"));
    assert!(!body.contains("Movies/Demo"));
    assert!(!body.contains(r#""raw":"#));
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
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
async fn acquisition_intake_service_accepts_candidate_into_managed_import_without_promotion_or_media_source()
 {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();

    let candidate = service
        .record_candidate(RecordAcquisitionIntakeCandidateRequest {
            id: None,
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::WatchFolder,
            source_key: "watch-folder://demo-ready".to_owned(),
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            display_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(42),
            fingerprint: Some("sha256-private-fingerprint".to_owned()),
            managed_import_artifact_id: None,
            state: Some(AcquisitionIntakeCandidateState::Ready),
            diagnostics_json: Some(r#"{"raw":"private"}"#.to_owned()),
        })
        .await
        .unwrap();

    let accepted = service
        .accept_candidate(AcceptAcquisitionIntakeCandidateRequest {
            candidate_id: candidate.id,
            managed_import_artifact_id: None,
        })
        .await
        .unwrap();
    let replayed = service
        .accept_candidate(AcceptAcquisitionIntakeCandidateRequest {
            candidate_id: candidate.id,
            managed_import_artifact_id: None,
        })
        .await
        .unwrap();
    let same_source_candidate = service
        .record_candidate(RecordAcquisitionIntakeCandidateRequest {
            id: None,
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::WatchFolder,
            source_key: "watch-folder://demo-ready-alias".to_owned(),
            source_uri: "file:///incoming/private/Demo.mkv?token=secret".to_owned(),
            display_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(42),
            fingerprint: Some("sha256-private-fingerprint".to_owned()),
            managed_import_artifact_id: None,
            state: Some(AcquisitionIntakeCandidateState::Ready),
            diagnostics_json: Some(r#"{"raw":"private"}"#.to_owned()),
        })
        .await
        .unwrap();
    let reused = service
        .accept_candidate(AcceptAcquisitionIntakeCandidateRequest {
            candidate_id: same_source_candidate.id,
            managed_import_artifact_id: None,
        })
        .await
        .unwrap();

    assert_eq!(accepted.candidate.id, candidate.id);
    assert_eq!(
        accepted.candidate.state,
        AcquisitionIntakeCandidateState::Accepted
    );
    assert_eq!(
        accepted.candidate.managed_import_artifact_id,
        Some(accepted.artifact_id)
    );
    assert_eq!(
        accepted.artifact_state,
        ManagedImportArtifactState::Proposed
    );
    assert!(!accepted.replayed);
    assert_eq!(replayed.artifact_id, accepted.artifact_id);
    assert!(replayed.replayed);
    assert_eq!(reused.artifact_id, accepted.artifact_id);
    assert_eq!(
        reused.candidate.managed_import_artifact_id,
        Some(accepted.artifact_id)
    );
    assert!(!accepted.writes_library);
    assert!(!accepted.promotion_apply);
    assert!(!accepted.media_source_created);
    assert_eq!(
        store
            .list_managed_import_artifacts(
                ManagedImportArtifactListFilter::all(),
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .len(),
        1
    );

    let artifact = store
        .get_managed_import_artifact(accepted.artifact_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(artifact.target_library_id, library.id);
    assert_eq!(
        artifact.source_kind,
        ManagedImportSourceKind::WatchedCandidate
    );
    assert_eq!(artifact.state, ManagedImportArtifactState::Proposed);
    assert_eq!(artifact.original_file_name.as_deref(), Some("Demo.mkv"));
    assert_eq!(
        artifact.intended_locator.as_deref(),
        Some("Movies/Demo (2026)/Demo.mkv")
    );
    assert_eq!(artifact.size_bytes, Some(42));
    assert_eq!(
        artifact.fingerprint.as_deref(),
        Some("sha256-private-fingerprint")
    );

    assert!(
        store
            .list_managed_import_promotion_applies_for_artifact(
                accepted.artifact_id,
                PageRequest::first_page(),
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

    let accepted_body = serde_json::to_string(&accepted).unwrap();
    assert!(!accepted_body.contains("token=secret"));
    assert!(!accepted_body.contains("incoming/private"));
    assert!(!accepted_body.contains("Demo.mkv"));
    assert!(!accepted_body.contains("sha256-private-fingerprint"));
    assert!(!accepted_body.contains("Movies/Demo"));
    assert!(!accepted_body.contains(r#""raw":"#));
}

#[tokio::test]
async fn acquisition_intake_watch_folder_discovery_preserves_accepted_candidate_link() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    fs::create_dir_all(&watch).unwrap();
    fs::write(watch.join("Accepted Movie.mkv"), b"accepted").unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let root_uri = StorageUri::from_parts("local", "watch").unwrap();

    let first = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri.clone()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let ready = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri.clone()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let candidate = service
        .list_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: Some(AcquisitionIntakeCandidateState::Ready),
                source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap()
        .candidates
        .into_iter()
        .next()
        .unwrap();
    let accepted = service
        .accept_candidate(AcceptAcquisitionIntakeCandidateRequest {
            candidate_id: candidate.id,
            managed_import_artifact_id: None,
        })
        .await
        .unwrap();
    let rediscovered = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let persisted = store
        .get_acquisition_intake_candidate(candidate.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(first.inspecting_candidates, 1);
    assert_eq!(ready.ready_candidates, 1);
    assert_eq!(ready.newly_ready_candidates, 1);
    assert_eq!(
        accepted.candidate.state,
        AcquisitionIntakeCandidateState::Accepted
    );
    assert_eq!(persisted.state, AcquisitionIntakeCandidateState::Accepted);
    assert_eq!(
        persisted.managed_import_artifact_id,
        Some(accepted.artifact_id)
    );
    assert_eq!(rediscovered.newly_ready_candidates, 0);
    assert_eq!(rediscovered.ready_candidates, 1);
    assert_eq!(rediscovered.inspecting_candidates, 0);
    assert_eq!(
        service
            .list_candidates(
                AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library.id),
                    state: Some(AcquisitionIntakeCandidateState::Accepted),
                    source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                    managed_import_artifact_id: Some(accepted.artifact_id),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .returned,
        1
    );
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );

    let body = serde_json::to_string(&rediscovered).unwrap();
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Accepted Movie"));
    assert!(!body.contains("local:///"));
}

#[tokio::test]
async fn acquisition_intake_service_links_explicit_existing_managed_import_artifact() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let existing = ManagedImportAppService::new(store.clone())
        .create_artifact(CreateManagedImportArtifactRequest {
            id: None,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::WatchedCandidate,
            source_uri: "file:///staging/private/Existing.mkv?token=secret".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: Some("Existing.mkv".to_owned()),
            intended_locator: Some("Movies/Existing (2026)/Existing.mkv".to_owned()),
            size_bytes: Some(100),
            fingerprint: Some("sha256-existing-private-fingerprint".to_owned()),
            state: Some(ManagedImportArtifactState::Proposed),
            diagnostics_json: Some(r#"{"raw":"existing-private"}"#.to_owned()),
        })
        .await
        .unwrap();
    let candidate = service
        .record_candidate(RecordAcquisitionIntakeCandidateRequest {
            id: None,
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::WatchFolder,
            source_key: "watch-folder://explicit-existing".to_owned(),
            source_uri: "file:///incoming/private/Explicit.mkv?token=secret".to_owned(),
            display_name: Some("Explicit.mkv".to_owned()),
            intended_locator: Some("Movies/Explicit (2026)/Explicit.mkv".to_owned()),
            size_bytes: Some(100),
            fingerprint: Some("sha256-explicit-private-fingerprint".to_owned()),
            managed_import_artifact_id: None,
            state: Some(AcquisitionIntakeCandidateState::Ready),
            diagnostics_json: Some(r#"{"raw":"candidate-private"}"#.to_owned()),
        })
        .await
        .unwrap();

    let accepted = service
        .accept_candidate(AcceptAcquisitionIntakeCandidateRequest {
            candidate_id: candidate.id,
            managed_import_artifact_id: Some(existing.id),
        })
        .await
        .unwrap();

    assert_eq!(accepted.artifact_id, existing.id);
    assert!(accepted.artifact_reused);
    assert_eq!(
        accepted.candidate.managed_import_artifact_id,
        Some(existing.id)
    );
    assert_eq!(
        accepted.candidate.state,
        AcquisitionIntakeCandidateState::Accepted
    );
    assert_eq!(
        store
            .list_managed_import_artifacts(
                ManagedImportArtifactListFilter::all(),
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );

    let accepted_body = serde_json::to_string(&accepted).unwrap();
    assert!(!accepted_body.contains("token=secret"));
    assert!(!accepted_body.contains("incoming/private"));
    assert!(!accepted_body.contains("staging/private"));
    assert!(!accepted_body.contains("Explicit.mkv"));
    assert!(!accepted_body.contains("Existing.mkv"));
    assert!(!accepted_body.contains("private-fingerprint"));
    assert!(!accepted_body.contains(r#""raw":"#));
}

#[tokio::test]
async fn acquisition_intake_records_resource_search_selection_as_host_owned_candidate() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let selected_link = AddonResourceLink {
        url: "https://pan.quark.cn/s/private-token?pwd=hidden".to_owned(),
        normalized_url: "https://pan.quark.cn/s/private-token?pwd=hidden".to_owned(),
        link_type: AddonResourceLinkType::Quark,
        source: "pansou-quark".to_owned(),
        password: Some("hidden-code".to_owned()),
        note: Some("private note should stay internal".to_owned()),
    };
    let result = AddonResourceSearchResult {
        id: "pansou-result://private/movie-1?token=secret".to_owned(),
        title: "Private Movie 1".to_owned(),
        source: "pansou".to_owned(),
        content: Some("private content should not echo".to_owned()),
        links: vec![selected_link.clone()],
        tags: vec!["private-tag".to_owned()],
        images: vec!["https://images.example/private.jpg?token=secret".to_owned()],
        score: 930,
    };
    let request = RecordResourceSearchSelectionRequest {
        target_library_id: library.id,
        addon_id: AddonId::new(),
        manifest_id: "nako.official.resource-search".to_owned(),
        query: "Private Query Token".to_owned(),
        result,
        selected_link,
    };

    let first = service
        .record_resource_search_selection(request.clone())
        .await
        .unwrap();
    let replayed = service
        .record_resource_search_selection(request.clone())
        .await
        .unwrap();

    assert!(!first.idempotent_replay);
    assert!(replayed.idempotent_replay);
    let first = first.candidate;
    assert_eq!(first.id, replayed.candidate.id);
    assert_eq!(first.target_library_id, library.id);
    assert_eq!(first.source_kind, "resource_search_selection");
    assert_eq!(first.source_scheme.as_deref(), Some("https"));
    assert_eq!(first.source_uri_redacted, "https://<redacted>");
    assert!(first.source_key_fingerprint.starts_with("sha256:"));
    assert!(first.has_display_name);
    assert!(!first.has_intended_locator);
    assert_eq!(first.size_bytes, None);
    assert!(!first.has_fingerprint);
    assert!(first.has_diagnostics);
    assert_eq!(first.state, AcquisitionIntakeCandidateState::Ready);

    let listed = service
        .list_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: Some(AcquisitionIntakeCandidateState::Ready),
                source_kind: Some(AcquisitionIntakeSourceKind::ResourceSearchSelection),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(listed.returned, 1);
    assert_eq!(listed.candidates[0].id, first.id);

    let accepted = service
        .accept_candidate(AcceptAcquisitionIntakeCandidateRequest {
            candidate_id: first.id,
            managed_import_artifact_id: None,
        })
        .await
        .unwrap();
    assert_eq!(accepted.candidate.id, first.id);
    assert_eq!(
        accepted.candidate.state,
        AcquisitionIntakeCandidateState::Accepted
    );
    assert_eq!(
        accepted.candidate.managed_import_artifact_id,
        Some(accepted.artifact_id)
    );
    assert_eq!(
        accepted.artifact_state,
        ManagedImportArtifactState::Proposed
    );
    assert!(!accepted.writes_library);
    assert!(!accepted.promotion_apply);
    assert!(!accepted.media_source_created);
    let accepted_replay = service
        .record_resource_search_selection(request)
        .await
        .unwrap();
    assert!(accepted_replay.idempotent_replay);
    assert_eq!(accepted_replay.candidate.id, first.id);
    assert_eq!(
        accepted_replay.candidate.state,
        AcquisitionIntakeCandidateState::Accepted
    );
    assert_eq!(
        accepted_replay.candidate.managed_import_artifact_id,
        Some(accepted.artifact_id)
    );

    let artifact = store
        .get_managed_import_artifact(accepted.artifact_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        artifact.source_kind,
        ManagedImportSourceKind::ResourceSearchSelection
    );
    assert_eq!(artifact.target_library_id, library.id);
    assert_eq!(
        artifact.original_file_name.as_deref(),
        Some("Private Movie 1")
    );
    assert_eq!(artifact.state, ManagedImportArtifactState::Proposed);
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_managed_import_promotion_applies_for_artifact(
                accepted.artifact_id,
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );

    for body in [
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&listed).unwrap(),
        serde_json::to_string(&accepted).unwrap(),
        serde_json::to_string(&accepted_replay.candidate).unwrap(),
    ] {
        assert!(!body.contains("private-token"));
        assert!(!body.contains("pwd=hidden"));
        assert!(!body.contains("hidden-code"));
        assert!(!body.contains("private note"));
        assert!(!body.contains("Private Movie 1"));
        assert!(!body.contains("Private Query Token"));
        assert!(!body.contains("private content"));
        assert!(!body.contains("private-tag"));
        assert!(!body.contains("token=secret"));
    }
}

#[tokio::test]
async fn acquisition_intake_rejects_resource_search_selection_without_link_uri() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let selected_link = AddonResourceLink {
        url: "   ".to_owned(),
        normalized_url: "\t".to_owned(),
        link_type: AddonResourceLinkType::Web,
        source: "pansou".to_owned(),
        password: None,
        note: None,
    };
    let result = AddonResourceSearchResult {
        id: "result-1".to_owned(),
        title: "No Link".to_owned(),
        source: "pansou".to_owned(),
        content: None,
        links: vec![selected_link.clone()],
        tags: Vec::new(),
        images: Vec::new(),
        score: 10,
    };

    let err = service
        .record_resource_search_selection(RecordResourceSearchSelectionRequest {
            target_library_id: library.id,
            addon_id: AddonId::new(),
            manifest_id: "nako.official.resource-search".to_owned(),
            query: "No Link".to_owned(),
            result,
            selected_link,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, NakoError::InvalidInput { .. }));
    assert!(
        store
            .list_acquisition_intake_candidates(
                AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library.id),
                    state: None,
                    source_kind: Some(AcquisitionIntakeSourceKind::ResourceSearchSelection),
                    managed_import_artifact_id: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn acquisition_intake_watch_folder_discovery_records_classified_candidates_without_mutation()
{
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    fs::create_dir_all(watch.join("Season 01")).unwrap();
    fs::write(watch.join("Ready Movie.mkv"), b"ready").unwrap();
    fs::write(watch.join("Season 01").join("Episode 01.mp4"), b"episode").unwrap();
    fs::write(watch.join("Downloading.part"), b"partial").unwrap();
    fs::write(watch.join("Notes.txt"), b"notes").unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let root_uri = StorageUri::from_parts("local", "watch").unwrap();

    let first = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri.clone()),
            max_depth: Some(4),
        })
        .await
        .unwrap();
    let replayed = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri),
            max_depth: Some(4),
        })
        .await
        .unwrap();

    assert_eq!(first.target_library_id, library.id);
    assert_eq!(first.root_uri_redacted, "local://<redacted>");
    assert_eq!(first.ready_candidates, 0);
    assert_eq!(first.inspecting_candidates, 2);
    assert_eq!(first.blocked_candidates, 2);
    assert_eq!(first.incomplete_candidates, 1);
    assert_eq!(first.unsupported_candidates, 1);
    assert_eq!(first.recorded_candidates, 4);
    assert_eq!(first.newly_ready_candidates, 0);
    assert_eq!(first.failures.len(), 0);
    assert!(!first.writes_library);
    assert!(!first.managed_import_artifacts_created);
    assert!(!first.promotion_apply);
    assert_eq!(replayed.recorded_candidates, 4);
    assert_eq!(replayed.ready_candidates, 2);
    assert_eq!(replayed.inspecting_candidates, 0);
    assert_eq!(replayed.newly_ready_candidates, 2);

    assert_eq!(
        service
            .list_candidates(
                AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library.id),
                    state: None,
                    source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                    managed_import_artifact_id: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .returned,
        4
    );
    assert_eq!(
        service
            .list_candidates(
                AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library.id),
                    state: Some(AcquisitionIntakeCandidateState::Ready),
                    source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                    managed_import_artifact_id: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .returned,
        2
    );
    assert_eq!(
        service
            .list_candidates(
                AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library.id),
                    state: Some(AcquisitionIntakeCandidateState::Blocked),
                    source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                    managed_import_artifact_id: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .returned,
        2
    );
    assert!(
        store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
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

    let body = serde_json::to_string(&first).unwrap();
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Ready Movie"));
    assert!(!body.contains("Episode 01"));
    assert!(!body.contains("Downloading"));
    assert!(!body.contains("Notes.txt"));
}

#[tokio::test]
async fn acquisition_intake_watch_folder_discovery_updates_legacy_source_key_without_duplicate() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    fs::create_dir_all(&watch).unwrap();
    fs::write(watch.join("Legacy Movie.mkv"), b"legacy").unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let media_uri = StorageUri::from_parts("local", "watch/Legacy Movie.mkv").unwrap();
    let metadata = LocalFsBackend::new(temp.path())
        .unwrap()
        .stat(&media_uri)
        .await
        .unwrap();
    let legacy_source_key = match (&metadata.fingerprint, metadata.len) {
        (Some(fingerprint), Some(size_bytes)) => {
            format!("{media_uri}|size={size_bytes}|fingerprint={fingerprint}")
        }
        (Some(fingerprint), None) => format!("{media_uri}|fingerprint={fingerprint}"),
        (None, Some(size_bytes)) => format!("{media_uri}|size={size_bytes}"),
        (None, None) => media_uri.to_string(),
    };
    let now_ms = crate::app::current_time_ms().unwrap();
    store
        .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
            id: AcquisitionIntakeCandidateId::new(),
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::WatchFolder,
            source_key: legacy_source_key,
            source_uri: media_uri.to_string(),
            display_name: Some("Legacy Movie.mkv".to_owned()),
            intended_locator: None,
            size_bytes: metadata.len,
            fingerprint: metadata.fingerprint.clone(),
            managed_import_artifact_id: None,
            state: AcquisitionIntakeCandidateState::Inspecting,
            diagnostics_json: None,
            first_seen_at_ms: now_ms,
            last_seen_at_ms: now_ms,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
        .await
        .unwrap();

    let first = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(StorageUri::from_parts("local", "watch").unwrap()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let second = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(StorageUri::from_parts("local", "watch").unwrap()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let listed = service
        .list_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();

    assert_eq!(first.recorded_candidates, 1);
    assert_eq!(first.inspecting_candidates, 1);
    assert_eq!(first.newly_ready_candidates, 0);
    assert_eq!(second.recorded_candidates, 1);
    assert_eq!(second.ready_candidates, 1);
    assert_eq!(second.newly_ready_candidates, 1);
    assert_eq!(listed.returned, 1);
    assert_eq!(
        listed.candidates[0].state,
        AcquisitionIntakeCandidateState::Ready
    );
    assert!(
        listed.candidates[0]
            .source_key_fingerprint
            .starts_with("sha256:")
    );

    let body = serde_json::to_string(&second).unwrap();
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Legacy Movie"));
    assert!(!body.contains("local:///"));
    assert!(!body.contains("fingerprint="));
}

#[tokio::test]
async fn acquisition_intake_watch_folder_discovery_resets_stability_when_observation_changes() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    let media_path = watch.join("Growing Movie.mkv");
    fs::create_dir_all(&watch).unwrap();
    fs::write(&media_path, b"partial").unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let root_uri = StorageUri::from_parts("local", "watch").unwrap();

    let first = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri.clone()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    fs::write(&media_path, b"partial-but-still-growing").unwrap();
    let changed = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri.clone()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let stable = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri),
            max_depth: Some(1),
        })
        .await
        .unwrap();

    assert_eq!(first.inspecting_candidates, 1);
    assert_eq!(first.newly_ready_candidates, 0);
    assert_eq!(changed.inspecting_candidates, 1);
    assert_eq!(changed.ready_candidates, 0);
    assert_eq!(changed.newly_ready_candidates, 0);
    assert_eq!(stable.ready_candidates, 1);
    assert_eq!(stable.newly_ready_candidates, 1);

    let records = store
        .list_acquisition_intake_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].state, AcquisitionIntakeCandidateState::Ready);
    let diagnostics: serde_json::Value =
        serde_json::from_str(records[0].diagnostics_json.as_deref().unwrap()).unwrap();
    assert_eq!(diagnostics["classification"], "ready");
    assert_eq!(
        diagnostics["stability_reason"],
        "stability_threshold_reached"
    );
    assert_eq!(
        diagnostics["stable_candidate"]["consecutive_stable_observations"],
        2
    );

    let body = serde_json::to_string(&changed).unwrap();
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Growing Movie"));
    assert!(!body.contains("partial-but-still-growing"));
    assert!(!body.contains("local:///"));
}

#[tokio::test]
async fn acquisition_intake_watch_folder_discovery_rejects_out_of_scope_root_without_recording() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&watch).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("Outside Movie.mkv"), b"outside").unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, &watch).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();

    let err = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(StorageUri::from_parts("local", "../outside").unwrap()),
            max_depth: Some(1),
        })
        .await
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "invalid input: watch-folder discovery root_uri is outside the library root"
    );
    assert!(!err.to_string().contains(&temp.path().display().to_string()));
    assert!(!err.to_string().contains("Outside Movie"));
    assert!(
        store
            .list_acquisition_intake_candidates(
                AcquisitionIntakeCandidateListFilter {
                    target_library_id: Some(library.id),
                    state: None,
                    source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                    managed_import_artifact_id: None,
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn acquisition_intake_watch_folder_discovery_suppresses_planned_host_writes_without_raw_scope()
 {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let temp = tempfile::tempdir().unwrap();
    let watch = temp.path().join("watch");
    fs::create_dir_all(&watch).unwrap();
    fs::write(watch.join("Suppressed Movie.mkv"), b"generated").unwrap();
    fs::write(watch.join("Visible Movie.mkv"), b"visible").unwrap();
    let library_id = LibraryId::new();
    let app = acquisition_app_with_store(store.clone(), library_id, temp.path()).await;
    let library = store.get_library(library_id).await.unwrap().unwrap();
    let service = app.acquisition_intake();
    let root_uri = StorageUri::from_parts("local", "watch").unwrap();
    app.watch_folder_suppression()
        .begin_planned_write_suppression(BeginPlannedWatchFolderWriteSuppressionRequest {
            target_library_id: library.id,
            scope_uri: StorageUri::from_parts("local", "watch/Suppressed Movie.mkv").unwrap(),
            owner: "nfo".to_owned(),
            reason: "sidecar_write".to_owned(),
            ttl_ms: Some(60_000),
            completion: PlannedWatchFolderWriteCompletion::SuppressOnly,
        })
        .await
        .unwrap();

    let first = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri.clone()),
            max_depth: Some(1),
        })
        .await
        .unwrap();
    let replayed = service
        .discover_watch_folder_candidates(DiscoverWatchFolderCandidatesRequest {
            target_library_id: library.id,
            root_uri: Some(root_uri),
            max_depth: Some(1),
        })
        .await
        .unwrap();

    assert_eq!(first.suppressed_candidates, 1);
    assert_eq!(first.recorded_candidates, 1);
    assert_eq!(first.inspecting_candidates, 1);
    assert_eq!(first.newly_ready_candidates, 0);
    assert_eq!(first.active_suppressions.len(), 1);
    assert_eq!(first.active_suppressions[0].scope_scheme, "local");
    assert_eq!(
        first.active_suppressions[0].scope_ref_redacted,
        "local://<redacted>"
    );
    assert_eq!(first.active_suppressions[0].owner, "nfo");
    assert_eq!(first.active_suppressions[0].reason, "sidecar_write");
    assert_eq!(replayed.suppressed_candidates, 1);
    assert_eq!(replayed.recorded_candidates, 1);
    assert_eq!(replayed.ready_candidates, 1);
    assert_eq!(replayed.newly_ready_candidates, 1);

    let listed = service
        .list_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(AcquisitionIntakeSourceKind::WatchFolder),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(listed.returned, 1);

    let body = serde_json::to_string(&first).unwrap();
    assert!(!body.contains(&temp.path().display().to_string()));
    assert!(!body.contains("Suppressed Movie"));
    assert!(!body.contains("Visible Movie"));
    assert!(!body.contains("scope_uri"));
    assert!(!body.contains("token"));
    assert!(!body.contains("local:///"));
}

async fn acquisition_app_with_store(
    store: NakoDatabase,
    library_id: LibraryId,
    root: &Path,
) -> NakoApp {
    NakoApp::new_with_store(
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
            vfs_cache_repair_automation:
                crate::config::VfsCacheRepairAutomationRuntimeConfig::default(),
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
        },
        store,
    )
    .await
    .unwrap()
}
