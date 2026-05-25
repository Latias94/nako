use super::*;
use crate::app::acquisition_intake::{
    AcceptAcquisitionIntakeCandidateRequest, DiscoverWatchFolderCandidatesRequest,
    RecordAcquisitionIntakeCandidateRequest,
};
use crate::app::managed_import::{CreateManagedImportArtifactRequest, ManagedImportAppService};
use nako_core::{
    AcquisitionIntakeCandidateListFilter, AcquisitionIntakeCandidateState,
    AcquisitionIntakeSourceKind, LibraryPreset, ManagedImportArtifactListFilter,
    ManagedImportArtifactState, ManagedImportRepository, ManagedImportSourceKind,
};

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
    assert_eq!(first.ready_candidates, 2);
    assert_eq!(first.blocked_candidates, 2);
    assert_eq!(first.incomplete_candidates, 1);
    assert_eq!(first.unsupported_candidates, 1);
    assert_eq!(first.recorded_candidates, 4);
    assert_eq!(first.failures.len(), 0);
    assert!(!first.writes_library);
    assert!(!first.managed_import_artifacts_created);
    assert!(!first.promotion_apply);
    assert_eq!(replayed.recorded_candidates, 4);

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
