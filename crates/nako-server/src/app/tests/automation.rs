use super::*;
use nako_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability,
    AutomationJobInput, AutomationProviderId, AutomationProviderStatus, AutomationRepository,
    CatalogRepository, GeneratedArtifactAcceptanceActionKind,
    GeneratedArtifactAcceptancePlanReason, GeneratedArtifactAcceptancePlanStatus,
    GeneratedArtifactMetadataApplyPlanReason, GeneratedArtifactMetadataApplyPlanStatus,
    GeneratedArtifactMetadataApplyResultStatus, GeneratedArtifactMetadataFieldAction,
    GeneratedArtifactMetadataFieldReason, GeneratedArtifactReadinessStatus,
    GeneratedArtifactReviewDecision, GeneratedArtifactTargetKind, NewAutomationArtifact,
    NewAutomationProviderConfig,
};
use nako_search::{SearchIndex, SearchQuery};

#[tokio::test]
async fn automation_app_lists_generated_artifact_proposals_without_raw_payloads_or_mutation() {
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
            root: temp.path().join("Movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning"}"#.to_owned(),
        })
        .await
        .unwrap();

    let proposals = app
        .automation()
        .list_generated_artifact_proposals(PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(proposals.len(), 1);
    assert_eq!(
        proposals[0].readiness.status,
        GeneratedArtifactReadinessStatus::Ready
    );
    assert_eq!(
        proposals[0].target.kind,
        GeneratedArtifactTargetKind::MediaSource
    );
    assert_eq!(proposals[0].payload.confidence_milli, Some(810));
    let item_after = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());

    let body = serde_json::to_string(&proposals[0]).unwrap();
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn automation_app_reviews_metadata_cleanup_proposal_without_canonical_mutation() {
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
            root: temp.path().join("Movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning"}"#.to_owned(),
        })
        .await
        .unwrap();

    let plan = app
        .automation()
        .plan_generated_artifact_review(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();
    assert_eq!(plan.status, GeneratedArtifactAcceptancePlanStatus::Ready);
    assert_eq!(
        plan.action,
        GeneratedArtifactAcceptanceActionKind::StageMetadataAuthorityReview
    );
    assert!(plan.boundary.requires_metadata_authority_apply);
    assert!(!plan.boundary.accepted_into_canonical_metadata);
    assert!(!plan.boundary.writes_sidecar);
    assert!(!plan.boundary.writes_library_files);
    assert!(!plan.boundary.applies_immediately);
    assert!(
        plan.reasons
            .contains(&GeneratedArtifactAcceptancePlanReason::MetadataAuthorityApplyRequired)
    );

    let accepted = app
        .automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();
    assert_eq!(accepted.artifact_status, AutomationArtifactStatus::Accepted);
    assert!(accepted.accepted_at.is_some());
    assert!(!accepted.idempotent_replay);
    assert!(!accepted.plan.boundary.accepted_into_canonical_metadata);

    let replay = app
        .automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();
    assert!(replay.idempotent_replay);
    assert_eq!(replay.artifact_status, AutomationArtifactStatus::Accepted);
    let reverse_err = app
        .automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Reject)
        .await
        .unwrap_err();
    assert!(
        reverse_err
            .to_string()
            .contains("cannot change reviewed generated artifact")
    );

    let item_after = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    let source_after = store.get_media_source(source.id).await.unwrap().unwrap();
    assert_eq!(source_after.locator, source.locator);

    let body = serde_json::to_string(&accepted).unwrap();
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn generated_artifact_metadata_apply_plan_is_field_level_redacted_and_read_only() {
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
            root: temp.path().join("Movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_field_lock(&nako_core::MetadataFieldLock {
            item_id: item.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"title":"Private AI Title","overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning"}"#.to_owned(),
        })
        .await
        .unwrap();
    app.automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();

    let plan = app
        .automation()
        .plan_generated_artifact_metadata_apply(artifact.id)
        .await
        .unwrap();

    assert_eq!(plan.status, GeneratedArtifactMetadataApplyPlanStatus::Ready);
    assert!(plan.executable);
    assert!(
        plan.reasons
            .contains(&GeneratedArtifactMetadataApplyPlanReason::Ready)
    );
    assert_eq!(plan.apply_field_count, 1);
    assert_eq!(plan.skipped_field_count, 1);
    assert_eq!(plan.noop_field_count, 0);
    assert_eq!(plan.payload.confidence_milli, Some(810));
    let title = plan
        .fields
        .iter()
        .find(|field| field.field == MetadataField::Title)
        .unwrap();
    assert_eq!(title.action, GeneratedArtifactMetadataFieldAction::Skip);
    assert!(
        title
            .reasons
            .contains(&GeneratedArtifactMetadataFieldReason::FieldLocked)
    );
    let overview = plan
        .fields
        .iter()
        .find(|field| field.field == MetadataField::Overview)
        .unwrap();
    assert_eq!(overview.action, GeneratedArtifactMetadataFieldAction::Apply);
    assert!(
        overview
            .reasons
            .contains(&GeneratedArtifactMetadataFieldReason::Ready)
    );

    let item_after = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    let body = serde_json::to_string(&plan).unwrap();
    assert!(!body.contains("Private AI Title"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn generated_artifact_metadata_apply_commits_unlocked_fields_and_catalog_projection() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"title":"Private AI Title","overview":"private generated overview","genres":["Cyberpunk"],"confidence_milli":810,"explanation":"private reasoning"}"#,
    )
    .await;
    fixture
        .store
        .upsert_field_lock(&nako_core::MetadataFieldLock {
            item_id: fixture.item_id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();

    let applied = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(fixture.artifact_id)
        .await
        .unwrap();

    assert_eq!(
        applied.status,
        GeneratedArtifactMetadataApplyResultStatus::Applied
    );
    assert!(applied.applied);
    assert!(applied.changed);
    assert!(!applied.idempotent_replay);
    assert_eq!(applied.applied_source.as_deref(), Some("user"));
    assert_eq!(applied.plan.apply_field_count, 2);
    assert_eq!(applied.plan.skipped_field_count, 1);
    assert_eq!(applied.plan.noop_field_count, 0);
    let title = applied
        .plan
        .fields
        .iter()
        .find(|field| field.field == MetadataField::Title)
        .unwrap();
    assert_eq!(title.action, GeneratedArtifactMetadataFieldAction::Skip);
    let overview = applied
        .plan
        .fields
        .iter()
        .find(|field| field.field == MetadataField::Overview)
        .unwrap();
    assert_eq!(overview.action, GeneratedArtifactMetadataFieldAction::Apply);

    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert_eq!(
        item_after.metadata.overview.as_deref(),
        Some("private generated overview")
    );
    assert_eq!(item_after.metadata.genres, vec!["Cyberpunk"]);

    let genres = fixture
        .store
        .list_genres(PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(
        genres
            .iter()
            .map(|genre| genre.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Cyberpunk"]
    );
    let item_genres = fixture
        .store
        .list_item_genres(fixture.item_id)
        .await
        .unwrap();
    assert_eq!(item_genres.len(), 1);

    let hits = fixture
        .store
        .search(
            SearchQuery::from_facet_labels("private generated overview", Vec::new(), 10, 0)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        hits.iter().map(|hit| hit.item_id).collect::<Vec<_>>(),
        vec![fixture.item_id]
    );
    let facet_hits = fixture
        .store
        .search(
            SearchQuery::from_facet_labels("", vec!["genre:Cyberpunk".to_owned()], 10, 0).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        facet_hits.iter().map(|hit| hit.item_id).collect::<Vec<_>>(),
        vec![fixture.item_id]
    );

    let body = serde_json::to_string(&applied).unwrap();
    assert!(!body.contains("Private AI Title"));
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn generated_artifact_metadata_apply_replay_is_idempotent_noop() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"overview":"private generated overview","confidence_milli":810}"#,
    )
    .await;

    let applied = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(fixture.artifact_id)
        .await
        .unwrap();
    assert_eq!(
        applied.status,
        GeneratedArtifactMetadataApplyResultStatus::Applied
    );

    let replay = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(fixture.artifact_id)
        .await
        .unwrap();

    assert_eq!(
        replay.status,
        GeneratedArtifactMetadataApplyResultStatus::Noop
    );
    assert!(!replay.applied);
    assert!(!replay.changed);
    assert!(replay.idempotent_replay);
    assert!(
        replay
            .plan
            .reasons
            .contains(&GeneratedArtifactMetadataApplyPlanReason::NoApplicableMetadataFields)
    );
}

#[tokio::test]
async fn generated_artifact_metadata_apply_rejects_stale_target_before_mutation() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"overview":"private generated overview","confidence_milli":810}"#,
    )
    .await;
    let moved_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Moved Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    fixture.store.upsert_media_item(&moved_item).await.unwrap();
    fixture
        .store
        .upsert_media_source(&MediaSource {
            id: fixture.source_id,
            library_id: fixture.library_id,
            item_id: moved_item.id,
            locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
            file_name: "The Matrix.mkv".to_owned(),
            size_bytes: Some(1024),
            fingerprint: Some("sha256-private-fingerprint".to_owned()),
        })
        .await
        .unwrap();

    let err = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(fixture.artifact_id)
        .await
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("generated artifact metadata apply plan is not executable")
    );
    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert!(item_after.metadata.overview.is_none());
}

#[tokio::test]
async fn automation_app_blocks_stale_generated_artifact_acceptance_and_allows_reject() {
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
            root: temp.path().join("Movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let original_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let moved_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Moved Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: original_item.id,
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: None,
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&original_item).await.unwrap();
    store.upsert_media_item(&moved_item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: None,
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(original_item.id),
                    source_id: Some(source.id),
                    prompt_json: "{}".to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", source.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(moved_item.id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"stale generated overview"}"#.to_owned(),
        })
        .await
        .unwrap();

    let blocked = app
        .automation()
        .plan_generated_artifact_review(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();
    assert_eq!(blocked.status, GeneratedArtifactAcceptancePlanStatus::Stale);
    assert_eq!(blocked.action, GeneratedArtifactAcceptanceActionKind::Noop);
    assert!(
        blocked
            .reasons
            .contains(&GeneratedArtifactAcceptancePlanReason::ProposalNotReady)
    );
    let err = app
        .automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("generated artifact review plan is not executable")
    );

    let rejected = app
        .automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Reject)
        .await
        .unwrap();
    assert_eq!(rejected.artifact_status, AutomationArtifactStatus::Rejected);
    assert_eq!(
        rejected.plan.action,
        GeneratedArtifactAcceptanceActionKind::RejectProposal
    );
    assert!(!rejected.plan.boundary.applies_immediately);
    let reverse_err = app
        .automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap_err();
    assert!(
        reverse_err
            .to_string()
            .contains("cannot change reviewed generated artifact")
    );
}

struct GeneratedArtifactMetadataApplyFixture {
    _temp: tempfile::TempDir,
    app: NakoApp,
    store: NakoDatabase,
    library_id: LibraryId,
    item_id: MediaItemId,
    source_id: MediaSourceId,
    artifact_id: AutomationArtifactId,
}

async fn generated_artifact_metadata_apply_fixture(
    artifact_json: &str,
) -> GeneratedArtifactMetadataApplyFixture {
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
            root: temp.path().join("Movies"),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///Movies/private/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1024),
        fingerprint: Some("sha256-private-fingerprint".to_owned()),
    };
    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    let provider_id = AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Gateway".to_owned(),
            base_url: "https://example.test/automation".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_SECRET".to_owned()),
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 10_000,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::Automation,
            resource_class: "automation.external_api".to_owned(),
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::to_string(&AutomationJobInput {
                    provider_id,
                    capability: AutomationCapability::MetadataCleanup,
                    library_id: Some(library_id),
                    item_id: Some(item.id),
                    source_id: Some(source.id),
                    prompt_json:
                        r#"{"path":"local:///Movies/private/The Matrix.mkv","token":"secret"}"#
                            .to_owned(),
                    idempotency_key: format!("metadata-cleanup:{}", item.id),
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library_id),
            item_id: Some(item.id),
            source_id: Some(source.id),
            artifact_json: artifact_json.to_owned(),
        })
        .await
        .unwrap();
    app.automation()
        .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
        .await
        .unwrap();

    GeneratedArtifactMetadataApplyFixture {
        _temp: temp,
        app,
        store,
        library_id,
        item_id: item.id,
        source_id: source.id,
        artifact_id: artifact.id,
    }
}
