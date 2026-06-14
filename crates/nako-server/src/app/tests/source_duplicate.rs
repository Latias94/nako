use super::*;
use nako_core::{
    NakoError, ScanRepository, ScanSnapshotId, ScanStatus, SourceDuplicateEvidenceKind,
    SourceDuplicateReconciliationAction, SourceDuplicateRelationship,
    SourceDuplicateRelationshipId, SourceDuplicateRelationshipStatus, SourceDuplicateRepository,
    SourceFingerprintEvidenceKind, SourceState,
};

const CONTENT_FINGERPRINT: &str = "source:v1:content_hash:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn source_duplicate_reconciliation_plans_read_only_redacted_actions() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store) = source_duplicate_app(library_id, other_library_id).await;
    let target = seed_source(
        &store,
        library_id,
        "Target",
        "local:///Users/Frankorz/Secret Target.mkv?token=secret",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let suggested = seed_source(
        &store,
        library_id,
        "Suggested",
        "local:///Users/Frankorz/Suggested.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let confirmed = seed_source(
        &store,
        library_id,
        "Confirmed",
        "local:///Users/Frankorz/Confirmed.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let rejected = seed_source(
        &store,
        library_id,
        "Rejected",
        "local:///Users/Frankorz/Rejected.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let stale = seed_source(
        &store,
        library_id,
        "Stale",
        "local:///Users/Frankorz/Stale.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let fresh = seed_source(
        &store,
        library_id,
        "Fresh",
        "local:///Users/Frankorz/Fresh.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let other_library = seed_source(
        &store,
        other_library_id,
        "Other Library",
        "local:///Users/Frankorz/Other Library.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;

    seed_stale_source_state(&store, library_id, &stale).await;
    seed_relationship(
        &store,
        target.id,
        suggested.id,
        SourceDuplicateRelationshipStatus::Suggested,
    )
    .await;
    seed_relationship(
        &store,
        target.id,
        confirmed.id,
        SourceDuplicateRelationshipStatus::Confirmed,
    )
    .await;
    seed_relationship(
        &store,
        target.id,
        rejected.id,
        SourceDuplicateRelationshipStatus::Rejected,
    )
    .await;

    let before = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();
    let plan = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id,
            source_id: target.id,
            page: PageRequest::new(20, 0),
        })
        .await
        .unwrap();
    let after = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();
    let plan_json = serde_json::to_string(&plan).unwrap();

    assert_eq!(before, after);
    assert_eq!(before.len(), 3);
    assert_eq!(plan.library_id, library_id);
    assert_eq!(plan.source_id, target.id);
    assert_eq!(
        plan.fingerprint_evidence_kind,
        SourceFingerprintEvidenceKind::ContentHash
    );
    assert_eq!(plan.confidence_milli, 1_000);
    assert!(!plan.stale);
    assert_eq!(plan.candidates.len(), 5);
    assert!(
        !plan
            .candidates
            .iter()
            .any(|candidate| candidate.duplicate_source_id == other_library.id)
    );

    assert_candidate(
        &plan,
        suggested.id,
        SourceDuplicateReconciliationAction::PreserveSuggested,
        Some(SourceDuplicateRelationshipStatus::Suggested),
        false,
        Some(1_000),
    );
    assert_candidate(
        &plan,
        confirmed.id,
        SourceDuplicateReconciliationAction::PreserveConfirmed,
        Some(SourceDuplicateRelationshipStatus::Confirmed),
        false,
        Some(1_000),
    );
    assert_candidate(
        &plan,
        rejected.id,
        SourceDuplicateReconciliationAction::PreserveRejected,
        Some(SourceDuplicateRelationshipStatus::Rejected),
        false,
        Some(1_000),
    );
    assert_candidate(
        &plan,
        stale.id,
        SourceDuplicateReconciliationAction::RefreshSourceFingerprint,
        None,
        true,
        Some(800),
    );
    assert_candidate(
        &plan,
        fresh.id,
        SourceDuplicateReconciliationAction::SuggestRelationship,
        None,
        false,
        Some(1_000),
    );

    assert!(!plan_json.contains(CONTENT_FINGERPRINT));
    assert!(!plan_json.contains("local:///"));
    assert!(!plan_json.contains("Frankorz"));
    assert!(!plan_json.contains("Secret Target"));
    assert!(!plan_json.contains("token"));
}

#[tokio::test]
async fn source_duplicate_reconciliation_apply_creates_suggested_and_replays_idempotently() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store) = source_duplicate_app(library_id, other_library_id).await;
    let target = seed_source(
        &store,
        library_id,
        "Target",
        "local:///Users/Frankorz/Secret Target.mkv?token=secret",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let duplicate = seed_source(
        &store,
        library_id,
        "Duplicate",
        "local:///Users/Frankorz/Duplicate.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;

    let applied = app
        .source_duplicate_reconciliation()
        .apply_source_duplicate_reconciliation(SourceDuplicateReconciliationApplyRequest {
            library_id,
            source_id: target.id,
            duplicate_source_id: duplicate.id,
            expected_action: SourceDuplicateReconciliationAction::SuggestRelationship,
        })
        .await
        .unwrap();
    let relationship = store
        .get_source_duplicate_relationship_by_pair(target.id, duplicate.id)
        .await
        .unwrap()
        .expect("relationship should be persisted");

    assert!(applied.created);
    assert_eq!(applied.library_id, library_id);
    assert_eq!(applied.source_id, target.id);
    assert_eq!(applied.duplicate_source_id, duplicate.id);
    assert_eq!(applied.relationship_id, relationship.id);
    assert_eq!(
        applied.relationship_status,
        SourceDuplicateRelationshipStatus::Suggested
    );
    assert_eq!(
        applied.applied_action,
        SourceDuplicateReconciliationAction::SuggestRelationship
    );
    assert_eq!(
        relationship.status,
        SourceDuplicateRelationshipStatus::Suggested
    );
    assert_eq!(
        (relationship.source_id, relationship.duplicate_source_id),
        SourceDuplicateRelationship::canonical_pair(target.id, duplicate.id)
    );
    assert_eq!(
        relationship.evidence_kind,
        SourceDuplicateEvidenceKind::StrongFingerprint
    );
    assert_eq!(relationship.evidence_value, None);
    assert_eq!(relationship.confidence_milli, Some(1_000));

    let replayed = app
        .source_duplicate_reconciliation()
        .apply_source_duplicate_reconciliation(SourceDuplicateReconciliationApplyRequest {
            library_id,
            source_id: target.id,
            duplicate_source_id: duplicate.id,
            expected_action: SourceDuplicateReconciliationAction::SuggestRelationship,
        })
        .await
        .unwrap();
    let relationships = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();
    let applied_json = serde_json::to_string(&applied).unwrap();
    let replayed_json = serde_json::to_string(&replayed).unwrap();

    assert!(!replayed.created);
    assert_eq!(replayed.relationship_id, relationship.id);
    assert_eq!(
        replayed.relationship_status,
        SourceDuplicateRelationshipStatus::Suggested
    );
    assert_eq!(
        replayed.applied_action,
        SourceDuplicateReconciliationAction::PreserveSuggested
    );
    assert_eq!(relationships.len(), 1);
    assert_source_duplicate_apply_body_redacted(&applied_json);
    assert_source_duplicate_apply_body_redacted(&replayed_json);
}

#[tokio::test]
async fn source_duplicate_reconciliation_apply_rejects_non_suggest_without_writing() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store) = source_duplicate_app(library_id, other_library_id).await;
    let target = seed_source(
        &store,
        library_id,
        "Target",
        "local:///Users/Frankorz/Secret Target.mkv?token=secret",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let confirmed = seed_source(
        &store,
        library_id,
        "Confirmed",
        "local:///Users/Frankorz/Confirmed.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let rejected = seed_source(
        &store,
        library_id,
        "Rejected",
        "local:///Users/Frankorz/Rejected.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let stale = seed_source(
        &store,
        library_id,
        "Stale",
        "local:///Users/Frankorz/Stale.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let mismatch = seed_source(
        &store,
        library_id,
        "Mismatch",
        "local:///Users/Frankorz/Mismatch.mkv",
        Some(
            "source:v1:content_hash:sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                .to_owned(),
        ),
    )
    .await;
    let missing_fingerprint = seed_source(
        &store,
        library_id,
        "Missing Fingerprint",
        "local:///Users/Frankorz/Missing.mkv?token=secret",
        None,
    )
    .await;
    let other_library = seed_source(
        &store,
        other_library_id,
        "Other Library",
        "local:///Users/Frankorz/Other Library.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;

    seed_relationship(
        &store,
        target.id,
        confirmed.id,
        SourceDuplicateRelationshipStatus::Confirmed,
    )
    .await;
    seed_relationship(
        &store,
        target.id,
        rejected.id,
        SourceDuplicateRelationshipStatus::Rejected,
    )
    .await;
    seed_stale_source_state(&store, library_id, &stale).await;
    let before = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();

    let confirmed_error = apply_duplicate(&app, library_id, target.id, confirmed.id)
        .await
        .unwrap_err();
    let rejected_error = apply_duplicate(&app, library_id, target.id, rejected.id)
        .await
        .unwrap_err();
    let stale_error = apply_duplicate(&app, library_id, target.id, stale.id)
        .await
        .unwrap_err();
    let mismatch_error = apply_duplicate(&app, library_id, target.id, mismatch.id)
        .await
        .unwrap_err();
    let missing_fingerprint_error =
        apply_duplicate(&app, library_id, target.id, missing_fingerprint.id)
            .await
            .unwrap_err();
    let cross_library_error = apply_duplicate(&app, library_id, target.id, other_library.id)
        .await
        .unwrap_err();
    let wrong_expected_action_error = app
        .source_duplicate_reconciliation()
        .apply_source_duplicate_reconciliation(SourceDuplicateReconciliationApplyRequest {
            library_id,
            source_id: target.id,
            duplicate_source_id: stale.id,
            expected_action: SourceDuplicateReconciliationAction::PreserveSuggested,
        })
        .await
        .unwrap_err();
    let self_pair_error = apply_duplicate(&app, library_id, target.id, target.id)
        .await
        .unwrap_err();
    let after = store
        .list_source_duplicate_relationships(target.id, PageRequest::new(20, 0))
        .await
        .unwrap();

    assert_eq!(before, after);
    assert_eq!(before.len(), 2);
    assert_conflict_recommendation(&confirmed_error, "preserve_confirmed");
    assert_conflict_recommendation(&rejected_error, "preserve_rejected");
    assert_conflict_recommendation(&stale_error, "refresh_source_fingerprint");
    assert_eq!(
        mismatch_error.to_string(),
        "invalid input: source duplicate reconciliation candidate fingerprint does not match source fingerprint evidence"
    );
    assert_eq!(
        missing_fingerprint_error.to_string(),
        "invalid input: source duplicate reconciliation requires source fingerprint evidence"
    );
    assert_eq!(
        cross_library_error.to_string(),
        "invalid input: source duplicate reconciliation candidate does not belong to requested library"
    );
    assert_eq!(
        wrong_expected_action_error.to_string(),
        "invalid input: source duplicate reconciliation apply supports only suggest_relationship"
    );
    assert_eq!(
        self_pair_error.to_string(),
        "invalid input: source duplicate reconciliation candidate must differ from source"
    );

    for error in [
        confirmed_error,
        rejected_error,
        stale_error,
        mismatch_error,
        missing_fingerprint_error,
        cross_library_error,
        wrong_expected_action_error,
        self_pair_error,
    ] {
        let message = error.to_string();
        assert_source_duplicate_apply_body_redacted(&message);
    }
}

#[tokio::test]
async fn source_duplicate_reconciliation_paginates_candidates_after_excluding_target() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store) = source_duplicate_app(library_id, other_library_id).await;
    let target = seed_source(
        &store,
        library_id,
        "Target",
        "local:///Users/Frankorz/Target.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let first = seed_source(
        &store,
        library_id,
        "First",
        "local:///Users/Frankorz/First.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;
    let second = seed_source(
        &store,
        library_id,
        "Second",
        "local:///Users/Frankorz/Second.mkv",
        Some(CONTENT_FINGERPRINT.to_owned()),
    )
    .await;

    let first_page = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id,
            source_id: target.id,
            page: PageRequest::new(1, 0),
        })
        .await
        .unwrap();
    let second_page = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id,
            source_id: target.id,
            page: PageRequest::new(1, 1),
        })
        .await
        .unwrap();

    assert_eq!(first_page.candidates.len(), 1);
    assert_eq!(second_page.candidates.len(), 1);
    assert_ne!(
        first_page.candidates[0].duplicate_source_id,
        second_page.candidates[0].duplicate_source_id
    );
    assert!([first.id, second.id].contains(&first_page.candidates[0].duplicate_source_id));
    assert!([first.id, second.id].contains(&second_page.candidates[0].duplicate_source_id));
}

#[tokio::test]
async fn source_duplicate_reconciliation_rejects_unsafe_inputs_without_leak() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store) = source_duplicate_app(library_id, other_library_id).await;
    let missing = seed_source(
        &store,
        library_id,
        "Missing Fingerprint",
        "local:///Users/Frankorz/Missing Fingerprint.mkv?token=secret",
        None,
    )
    .await;
    let raw = seed_source(
        &store,
        library_id,
        "Raw Fingerprint",
        "local:///Users/Frankorz/Raw Fingerprint.mkv?token=secret",
        Some("sha256:private-raw-fingerprint".to_owned()),
    )
    .await;

    let missing_error = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id,
            source_id: missing.id,
            page: PageRequest::first_page(),
        })
        .await
        .unwrap_err()
        .to_string();
    let raw_error = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id,
            source_id: raw.id,
            page: PageRequest::first_page(),
        })
        .await
        .unwrap_err()
        .to_string();
    let cross_library_error = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id: other_library_id,
            source_id: raw.id,
            page: PageRequest::first_page(),
        })
        .await
        .unwrap_err()
        .to_string();

    assert_eq!(
        missing_error,
        "invalid input: source duplicate reconciliation requires source fingerprint evidence"
    );
    assert_eq!(
        raw_error,
        "invalid input: source duplicate reconciliation requires redacted source fingerprint evidence"
    );
    assert_eq!(
        cross_library_error,
        "invalid input: source duplicate reconciliation source does not belong to requested library"
    );
    assert!(!raw_error.contains("private-raw-fingerprint"));
    assert!(!cross_library_error.contains("local:///"));
    assert!(!cross_library_error.contains("Frankorz"));
    assert!(!cross_library_error.contains("token"));
}

async fn apply_duplicate(
    app: &NakoApp,
    library_id: LibraryId,
    source_id: MediaSourceId,
    duplicate_source_id: MediaSourceId,
) -> std::result::Result<nako_core::SourceDuplicateReconciliationApplyResult, NakoError> {
    app.source_duplicate_reconciliation()
        .apply_source_duplicate_reconciliation(SourceDuplicateReconciliationApplyRequest {
            library_id,
            source_id,
            duplicate_source_id,
            expected_action: SourceDuplicateReconciliationAction::SuggestRelationship,
        })
        .await
}

fn assert_conflict_recommendation(error: &NakoError, expected_recommendation: &str) {
    let NakoError::Conflict { message } = error else {
        panic!("expected conflict error, got {error:?}");
    };

    assert_eq!(
        message,
        &format!(
            "source duplicate reconciliation apply expected suggest_relationship but current recommendation is {expected_recommendation}"
        )
    );
}

fn assert_candidate(
    plan: &nako_core::SourceDuplicateReconciliationPlan,
    duplicate_source_id: MediaSourceId,
    expected_action: SourceDuplicateReconciliationAction,
    expected_status: Option<SourceDuplicateRelationshipStatus>,
    stale: bool,
    confidence_milli: Option<u16>,
) {
    let candidate = plan
        .candidates
        .iter()
        .find(|candidate| candidate.duplicate_source_id == duplicate_source_id)
        .expect("candidate should be present");

    assert_eq!(candidate.source_id, plan.source_id);
    assert_eq!(
        candidate.evidence_kind,
        SourceDuplicateEvidenceKind::StrongFingerprint
    );
    assert_eq!(candidate.existing_status, expected_status);
    assert_eq!(candidate.recommended_action, expected_action);
    assert_eq!(candidate.stale, stale);
    assert_eq!(candidate.confidence_milli, confidence_milli);
    if expected_status.is_some() {
        assert!(candidate.relationship_id.is_some());
    } else {
        assert_eq!(candidate.relationship_id, None);
    }
}

fn assert_source_duplicate_apply_body_redacted(body: &str) {
    for forbidden in [
        CONTENT_FINGERPRINT,
        "ffffffffffffffff",
        "local:///",
        "Frankorz",
        "Secret Target",
        "Secret Path",
        "private-etag",
        "token",
        "source_uri",
        "source_locator",
        "input_json",
        "summary_json",
        "evidence_value",
        "sha256:",
        "fingerprint\":\"",
    ] {
        assert!(
            !body.contains(forbidden),
            "source duplicate apply surface leaked forbidden term: {forbidden}"
        );
    }
}

async fn source_duplicate_app(
    library_id: LibraryId,
    other_library_id: LibraryId,
) -> (tempfile::TempDir, NakoApp, NakoDatabase) {
    let temp = tempfile::tempdir().unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(
        source_duplicate_config(temp.path(), library_id, other_library_id),
        store.clone(),
    )
    .await
    .unwrap();

    (temp, app, store)
}

async fn seed_source(
    store: &NakoDatabase,
    library_id: LibraryId,
    title: &str,
    locator: &str,
    fingerprint: Option<String>,
) -> MediaSource {
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: locator.to_owned(),
        file_name: format!("{title}.mkv"),
        size_bytes: Some(42),
        fingerprint,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    source
}

async fn seed_relationship(
    store: &NakoDatabase,
    source_id: MediaSourceId,
    duplicate_source_id: MediaSourceId,
    status: SourceDuplicateRelationshipStatus,
) {
    store
        .upsert_source_duplicate_relationship(&SourceDuplicateRelationship {
            id: SourceDuplicateRelationshipId::new(),
            source_id,
            duplicate_source_id,
            evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
            evidence_value: Some("redacted-existing-evidence".to_owned()),
            status,
            confidence_milli: Some(1_000),
        })
        .await
        .unwrap();
}

async fn seed_stale_source_state(
    store: &NakoDatabase,
    library_id: LibraryId,
    source: &MediaSource,
) {
    let scan_id = ScanSnapshotId::new();
    store
        .begin_scan_snapshot(scan_id, library_id, "local:///Users/Frankorz")
        .await
        .unwrap();
    store
        .upsert_source_state(&SourceState {
            library_id,
            source_id: Some(source.id),
            uri: source.locator.clone(),
            size_bytes: source.size_bytes,
            modified_at: Some("2026-06-06T00:00:00Z".to_owned()),
            etag: Some("private-etag".to_owned()),
            fingerprint: source.fingerprint.clone(),
            last_seen_scan_id: scan_id,
            tombstoned: true,
        })
        .await
        .unwrap();
    store
        .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
        .await
        .unwrap();
}

fn source_duplicate_config(
    root: &Path,
    library_id: LibraryId,
    other_library_id: LibraryId,
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
        remux_staging_root: root.join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![
            LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: root.join("movies"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
            LocalLibraryConfig {
                id: other_library_id,
                name: "Other Movies".to_owned(),
                root: root.join("other-movies"),
                preset: nako_core::LibraryPreset::Movies,
                webdav: None,
            },
        ],
    }
}
