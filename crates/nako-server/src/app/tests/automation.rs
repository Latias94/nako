use super::*;
use nako_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability,
    AutomationJobInput, AutomationProviderId, AutomationProviderStatus, AutomationRepository,
    CatalogRepository, GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS,
    GeneratedArtifactAcceptanceActionKind, GeneratedArtifactAcceptancePlanReason,
    GeneratedArtifactAcceptancePlanStatus, GeneratedArtifactMetadataApplyOutcomeStatus,
    GeneratedArtifactMetadataApplyPlanReason, GeneratedArtifactMetadataApplyPlanStatus,
    GeneratedArtifactMetadataApplyRequest, GeneratedArtifactMetadataApplyResultStatus,
    GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    GeneratedArtifactMetadataBulkApplyBatchRequest, GeneratedArtifactMetadataBulkApplyBatchStatus,
    GeneratedArtifactMetadataBulkApplyPlanItemStatus,
    GeneratedArtifactMetadataBulkApplyPlanRequest, GeneratedArtifactMetadataFieldAction,
    GeneratedArtifactMetadataFieldReason, GeneratedArtifactProviderMappingAction,
    GeneratedArtifactProviderMappingReason, GeneratedArtifactReadinessStatus,
    GeneratedArtifactReviewDecision, GeneratedArtifactTargetKind, JobRepository, JobStatus,
    NewAutomationArtifact, NewAutomationProviderConfig, ProviderMapping, ProviderMappingId,
    ProviderMappingRepository, ProviderMappingStatus, ProviderSubject, ProviderSubjectId,
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
async fn generated_artifact_metadata_apply_plan_includes_provider_mapping_without_mutation() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}],"confidence_milli":930,"explanation":"private provider reasoning"}"#,
    )
    .await;

    let plan = fixture
        .app
        .automation()
        .plan_generated_artifact_metadata_apply(fixture.artifact_id)
        .await
        .unwrap();

    assert_eq!(plan.status, GeneratedArtifactMetadataApplyPlanStatus::Ready);
    assert!(plan.executable);
    assert!(
        plan.reasons
            .contains(&GeneratedArtifactMetadataApplyPlanReason::Ready)
    );
    assert_eq!(plan.apply_field_count, 0);
    assert_eq!(plan.skipped_field_count, 0);
    assert_eq!(plan.noop_field_count, 0);
    assert_eq!(plan.apply_provider_mapping_count, 1);
    assert_eq!(plan.skipped_provider_mapping_count, 0);
    assert_eq!(plan.noop_provider_mapping_count, 0);
    assert_eq!(plan.provider_mappings.len(), 1);

    let mapping = &plan.provider_mappings[0];
    assert_eq!(
        mapping.action,
        GeneratedArtifactProviderMappingAction::Apply
    );
    assert!(
        mapping
            .reasons
            .contains(&GeneratedArtifactProviderMappingReason::Ready)
    );
    assert_eq!(
        mapping.subject.provider,
        Some(nako_core::ExternalProvider::Tmdb)
    );
    assert_eq!(mapping.subject.provider_name.as_deref(), Some("tmdb"));
    assert_eq!(
        mapping.subject.subject_kind,
        Some(nako_core::ProviderSubjectKind::Movie)
    );
    assert_eq!(mapping.subject.subject_kind_name.as_deref(), Some("movie"));
    assert_eq!(mapping.subject.subject_key.as_deref(), Some("603"));
    assert_eq!(mapping.subject.title.as_deref(), Some("The Matrix"));
    assert_eq!(mapping.subject.release_year, Some(1999));
    assert_eq!(mapping.subject.locale.as_deref(), Some("en-US"));
    assert_eq!(mapping.confidence_milli, Some(930));
    assert_eq!(mapping.existing_mapping_status, None);

    let mappings = fixture
        .store
        .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert!(mappings.is_empty());

    let body = serde_json::to_string(&plan).unwrap();
    assert!(!body.contains("private provider reasoning"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));
}

#[tokio::test]
async fn generated_artifact_metadata_apply_plan_marks_invalid_provider_mapping_proposals() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"provider_subjects":[{"provider":"tvdb","subject_kind":"movie","subject_key":"42"},{"provider":"tmdb","subject_kind":"movie","subject_key":" "}],"confidence_milli":700}"#,
    )
    .await;

    let plan = fixture
        .app
        .automation()
        .plan_generated_artifact_metadata_apply(fixture.artifact_id)
        .await
        .unwrap();

    assert_eq!(
        plan.status,
        GeneratedArtifactMetadataApplyPlanStatus::Blocked
    );
    assert!(!plan.executable);
    assert!(
        plan.reasons
            .contains(&GeneratedArtifactMetadataApplyPlanReason::NoApplicableMetadataFields)
    );
    assert_eq!(plan.provider_mappings.len(), 2);
    assert_eq!(plan.apply_provider_mapping_count, 0);
    assert_eq!(plan.skipped_provider_mapping_count, 2);
    assert_eq!(plan.noop_provider_mapping_count, 0);
    assert_eq!(
        plan.provider_mappings[0].action,
        GeneratedArtifactProviderMappingAction::Skip
    );
    assert!(
        plan.provider_mappings[0]
            .reasons
            .contains(&GeneratedArtifactProviderMappingReason::UnsupportedProvider)
    );
    assert_eq!(
        plan.provider_mappings[1].action,
        GeneratedArtifactProviderMappingAction::Skip
    );
    assert!(
        plan.provider_mappings[1]
            .reasons
            .contains(&GeneratedArtifactProviderMappingReason::MissingSubjectKey)
    );

    let mappings = fixture
        .store
        .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert!(mappings.is_empty());
}

#[tokio::test]
async fn generated_artifact_metadata_apply_commits_provider_mapping_and_replays_idempotently() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#,
    )
    .await;

    let applied = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:provider-mapping",
        ))
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
    assert_eq!(applied.plan.apply_field_count, 0);
    assert_eq!(applied.plan.apply_provider_mapping_count, 1);

    let subject = fixture
        .store
        .find_provider_subject(
            &nako_core::ExternalProvider::Tmdb,
            &nako_core::ProviderSubjectKind::Movie,
            "603",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(subject.title.as_deref(), Some("The Matrix"));
    let mappings = fixture
        .store
        .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].subject_id, subject.id);
    assert_eq!(mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(mappings[0].confidence_milli, Some(930));
    assert_eq!(mappings[0].source, MetadataSource::User);

    let replay = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:provider-mapping",
        ))
        .await
        .unwrap();
    assert_eq!(replay.outcome_id, applied.outcome_id);
    assert!(replay.idempotent_replay);
    assert_eq!(
        fixture
            .store
            .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
            .await
            .unwrap(),
        mappings
    );

    let noop = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:provider-mapping-noop",
        ))
        .await
        .unwrap();
    assert_eq!(
        noop.status,
        GeneratedArtifactMetadataApplyResultStatus::Noop
    );
    assert!(noop.plan.noop_provider_mapping_count >= 1);
}

#[tokio::test]
async fn generated_artifact_metadata_apply_accepts_candidate_and_preserves_rejected_mapping() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","confidence_milli":930}]}"#,
    )
    .await;
    let subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: nako_core::ExternalProvider::Tmdb,
        subject_kind: nako_core::ProviderSubjectKind::Movie,
        subject_key: "603".to_owned(),
        title: Some("Old Matrix".to_owned()),
        release_year: None,
        locale: None,
    };
    let candidate = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: fixture.item_id,
        subject_id: subject.id,
        status: ProviderMappingStatus::Candidate,
        confidence_milli: Some(710),
        source: MetadataSource::Provider(nako_core::ExternalProvider::Tmdb),
    };
    fixture
        .store
        .upsert_provider_subject(&subject)
        .await
        .unwrap();
    fixture
        .store
        .upsert_provider_mapping(&candidate)
        .await
        .unwrap();

    let applied = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:candidate-provider-mapping",
        ))
        .await
        .unwrap();

    assert_eq!(
        applied.status,
        GeneratedArtifactMetadataApplyResultStatus::Applied
    );
    let mappings = fixture
        .store
        .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].id, candidate.id);
    assert_eq!(mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(mappings[0].confidence_milli, Some(930));
    assert_eq!(mappings[0].source, MetadataSource::User);

    let rejected_fixture = generated_artifact_metadata_apply_fixture(
        r#"{"provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"604","title":"Rejected Matrix","confidence_milli":930}]}"#,
    )
    .await;
    let rejected_subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: nako_core::ExternalProvider::Tmdb,
        subject_kind: nako_core::ProviderSubjectKind::Movie,
        subject_key: "604".to_owned(),
        title: Some("Rejected Matrix".to_owned()),
        release_year: None,
        locale: None,
    };
    let rejected = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: rejected_fixture.item_id,
        subject_id: rejected_subject.id,
        status: ProviderMappingStatus::Rejected,
        confidence_milli: Some(100),
        source: MetadataSource::Provider(nako_core::ExternalProvider::Tmdb),
    };
    rejected_fixture
        .store
        .upsert_provider_subject(&rejected_subject)
        .await
        .unwrap();
    rejected_fixture
        .store
        .upsert_provider_mapping(&rejected)
        .await
        .unwrap();

    let noop = rejected_fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            rejected_fixture.artifact_id,
            "generated-artifact-apply:rejected-provider-mapping",
        ))
        .await
        .unwrap();
    assert_eq!(
        noop.status,
        GeneratedArtifactMetadataApplyResultStatus::Noop
    );
    let mappings = rejected_fixture
        .store
        .list_provider_mappings_for_item(rejected_fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(mappings, vec![rejected]);
}

#[tokio::test]
async fn generated_artifact_metadata_apply_plan_bulk_aggregates_redacted_read_only_selection() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning","provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#,
    )
    .await;
    let noop = fixture
        .add_accepted_metadata_artifact(
            "Noop Matrix",
            "noop-matrix.mkv",
            r#"{"provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"604","title":"Noop Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#,
        )
        .await;
    fixture
        .upsert_accepted_provider_mapping(noop.item_id, "604", "Noop Matrix", Some(930))
        .await;
    let skipped = fixture
        .add_accepted_metadata_artifact(
            "Skipped Matrix",
            "skipped-matrix.mkv",
            r#"{"provider_subjects":[{"provider":"unknown","subject_kind":"movie","subject_key":"999","title":"Skipped Matrix","confidence_milli":930},{"provider":"tmdb","subject_kind":"movie","title":"Missing Key","confidence_milli":930}]}"#,
        )
        .await;
    let missing_artifact_id = AutomationArtifactId::new();

    let plan = fixture
        .app
        .automation()
        .plan_generated_artifact_metadata_bulk_apply(
            GeneratedArtifactMetadataBulkApplyPlanRequest {
                artifact_ids: vec![
                    fixture.artifact_id,
                    fixture.artifact_id,
                    noop.artifact_id,
                    skipped.artifact_id,
                    missing_artifact_id,
                ],
            },
        )
        .await
        .unwrap();

    assert_eq!(plan.selection.requested_artifact_count, 5);
    assert_eq!(plan.selection.selected_artifact_count, 4);
    assert_eq!(plan.selection.duplicate_artifact_count, 1);
    assert_eq!(
        plan.selection.max_artifact_count,
        GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS as u32
    );
    assert_eq!(plan.summary.planned_artifact_count, 3);
    assert_eq!(plan.summary.missing_artifact_count, 1);
    assert_eq!(plan.summary.ready_artifact_count, 1);
    assert_eq!(plan.summary.blocked_artifact_count, 2);
    assert_eq!(plan.summary.executable_artifact_count, 1);
    assert_eq!(plan.summary.apply_field_count, 1);
    assert_eq!(plan.summary.skipped_field_count, 0);
    assert_eq!(plan.summary.noop_field_count, 0);
    assert_eq!(plan.summary.apply_provider_mapping_count, 1);
    assert_eq!(plan.summary.skipped_provider_mapping_count, 2);
    assert_eq!(plan.summary.noop_provider_mapping_count, 1);
    assert_eq!(plan.items.len(), 4);
    assert_eq!(
        plan.items[0].status,
        GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned
    );
    assert!(plan.items[0].executable);
    assert_eq!(
        plan.items[0].plan.as_ref().unwrap().status,
        GeneratedArtifactMetadataApplyPlanStatus::Ready
    );
    assert_eq!(
        plan.items[0]
            .plan
            .as_ref()
            .unwrap()
            .apply_provider_mapping_count,
        1
    );
    assert_eq!(
        plan.items[1].status,
        GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned
    );
    assert!(!plan.items[1].executable);
    assert_eq!(
        plan.items[1]
            .plan
            .as_ref()
            .unwrap()
            .noop_provider_mapping_count,
        1
    );
    assert_eq!(
        plan.items[2].status,
        GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned
    );
    assert!(!plan.items[2].executable);
    assert_eq!(
        plan.items[2]
            .plan
            .as_ref()
            .unwrap()
            .skipped_provider_mapping_count,
        2
    );
    assert_eq!(
        plan.items[3].status,
        GeneratedArtifactMetadataBulkApplyPlanItemStatus::Missing
    );
    assert!(!plan.items[3].executable);
    assert!(plan.items[3].plan.is_none());

    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());

    let body = serde_json::to_string(&plan).unwrap();
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
    assert!(!body.contains("sha256-private-fingerprint"));

    let too_many_artifact_ids = (0..=GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS)
        .map(|_| AutomationArtifactId::new())
        .collect();
    let err = fixture
        .app
        .automation()
        .plan_generated_artifact_metadata_bulk_apply(
            GeneratedArtifactMetadataBulkApplyPlanRequest {
                artifact_ids: too_many_artifact_ids,
            },
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("at most"));
}

#[tokio::test]
async fn generated_artifact_bulk_metadata_apply_batch_persists_confirmed_request_without_mutation()
{
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"overview":"private generated overview","confidence_milli":810,"explanation":"private reasoning","provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#,
    )
    .await;
    let missing_artifact_id = AutomationArtifactId::new();

    let batch = fixture
        .app
        .automation()
        .create_generated_artifact_metadata_bulk_apply_batch(
            GeneratedArtifactMetadataBulkApplyBatchRequest {
                artifact_ids: vec![
                    fixture.artifact_id,
                    fixture.artifact_id,
                    missing_artifact_id,
                ],
                idempotency_key: " bulk-apply:confirmed ".to_owned(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        batch.status,
        GeneratedArtifactMetadataBulkApplyBatchStatus::Queued
    );
    assert_eq!(batch.idempotency_key, "bulk-apply:confirmed");
    assert_eq!(batch.selection.requested_artifact_count, 3);
    assert_eq!(batch.selection.selected_artifact_count, 2);
    assert_eq!(batch.selection.duplicate_artifact_count, 1);
    assert_eq!(batch.summary.executable_artifact_count, 1);
    assert_eq!(batch.summary.apply_provider_mapping_count, 1);
    assert_eq!(batch.summary.skipped_provider_mapping_count, 0);
    assert_eq!(batch.summary.noop_provider_mapping_count, 0);
    assert_eq!(batch.items.len(), 2);
    assert_eq!(
        batch.items[0].status,
        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending
    );
    assert_eq!(
        batch.items[1].status,
        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped
    );
    assert_ne!(
        batch.items[0].idempotency_key,
        batch.items[1].idempotency_key
    );

    let replay = fixture
        .app
        .automation()
        .create_generated_artifact_metadata_bulk_apply_batch(
            GeneratedArtifactMetadataBulkApplyBatchRequest {
                artifact_ids: vec![missing_artifact_id],
                idempotency_key: "bulk-apply:confirmed".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(replay.id, batch.id);
    assert_eq!(replay.items, batch.items);

    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(item_after.metadata.title, "The Matrix");
    assert!(item_after.metadata.overview.is_none());
    assert!(
        fixture
            .store
            .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );

    let body = serde_json::to_string(&batch).unwrap();
    assert!(!body.contains("private generated overview"));
    assert!(!body.contains("private reasoning"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("secret"));
}

#[tokio::test]
async fn generated_artifact_bulk_metadata_apply_batch_executes_with_partial_results_and_replay() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"overview":"bulk generated overview","confidence_milli":810,"provider_subjects":[{"provider":"tmdb","subject_kind":"movie","subject_key":"603","title":"The Matrix","release_year":1999,"locale":"en-US","confidence_milli":930}]}"#,
    )
    .await;
    let noop_artifact_id = fixture
        .add_accepted_metadata_artifact_for_item(
            fixture.item_id,
            fixture.source_id,
            r#"{"overview":"bulk generated overview","confidence_milli":810}"#,
        )
        .await;
    let stale = fixture
        .add_accepted_metadata_artifact(
            "Stale Matrix",
            "stale-matrix.mkv",
            r#"{"overview":"stale generated overview","confidence_milli":810}"#,
        )
        .await;
    let failed = fixture
        .add_accepted_metadata_artifact(
            "Rejected Matrix",
            "rejected-matrix.mkv",
            r#"{"overview":"rejected generated overview","confidence_milli":810}"#,
        )
        .await;
    let missing_artifact_id = AutomationArtifactId::new();
    let batch = fixture
        .app
        .automation()
        .create_generated_artifact_metadata_bulk_apply_batch(
            GeneratedArtifactMetadataBulkApplyBatchRequest {
                artifact_ids: vec![
                    fixture.artifact_id,
                    noop_artifact_id,
                    stale.artifact_id,
                    failed.artifact_id,
                    missing_artifact_id,
                ],
                idempotency_key: "bulk-apply:execute".to_owned(),
            },
        )
        .await
        .unwrap();

    assert_eq!(
        batch.status,
        GeneratedArtifactMetadataBulkApplyBatchStatus::Queued
    );
    assert_eq!(batch.summary.apply_provider_mapping_count, 1);
    assert_eq!(batch.summary.skipped_provider_mapping_count, 0);
    assert_eq!(batch.summary.noop_provider_mapping_count, 0);
    assert_eq!(
        fixture
            .store
            .get_job(batch.job_id)
            .await
            .unwrap()
            .unwrap()
            .status,
        JobStatus::Queued
    );

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
            id: stale.source_id,
            library_id: fixture.library_id,
            item_id: moved_item.id,
            locator: "local:///Movies/private/stale-matrix.mkv".to_owned(),
            file_name: "stale-matrix.mkv".to_owned(),
            size_bytes: Some(2048),
            fingerprint: Some("sha256-private-stale-fingerprint".to_owned()),
        })
        .await
        .unwrap();
    fixture
        .store
        .set_automation_artifact_status(failed.artifact_id, AutomationArtifactStatus::Rejected)
        .await
        .unwrap();

    let executed = fixture
        .app
        .automation()
        .execute_generated_artifact_metadata_bulk_apply_batch(batch.id)
        .await
        .unwrap();

    assert_eq!(
        executed.status,
        GeneratedArtifactMetadataBulkApplyBatchStatus::Completed
    );
    assert_eq!(executed.execution_summary.applied_item_count, 1);
    assert_eq!(executed.execution_summary.noop_item_count, 1);
    assert_eq!(executed.execution_summary.stale_item_count, 1);
    assert_eq!(executed.execution_summary.failed_item_count, 1);
    assert_eq!(executed.execution_summary.skipped_item_count, 1);
    assert_eq!(executed.summary.apply_provider_mapping_count, 1);
    assert_eq!(executed.summary.skipped_provider_mapping_count, 0);
    assert_eq!(executed.summary.noop_provider_mapping_count, 0);
    assert_eq!(
        executed
            .items
            .iter()
            .map(|item| item.status)
            .collect::<Vec<_>>(),
        vec![
            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied,
            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Noop,
            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Stale,
            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Failed,
            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped,
        ]
    );
    assert!(executed.items[0].outcome_id.is_some());
    assert!(executed.items[1].outcome_id.is_some());
    assert!(executed.items[2].outcome_id.is_some());
    assert_eq!(
        executed.items[2].error_code.as_deref(),
        Some("plan_not_executable")
    );
    assert!(executed.items[3].outcome_id.is_some());
    assert_eq!(
        executed.items[3].error_code.as_deref(),
        Some("plan_not_executable")
    );
    assert!(executed.items[4].outcome_id.is_none());

    let job = fixture.store.get_job(batch.job_id).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
    assert!(job.summary_json.is_some());

    let item_after = fixture
        .store
        .get_media_item(fixture.item_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        item_after.metadata.overview.as_deref(),
        Some("bulk generated overview")
    );
    let mappings = fixture
        .store
        .list_provider_mappings_for_item(fixture.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(mappings[0].confidence_milli, Some(930));
    assert_eq!(mappings[0].source, MetadataSource::User);

    let replay = fixture
        .app
        .automation()
        .execute_generated_artifact_metadata_bulk_apply_batch(batch.id)
        .await
        .unwrap();
    assert_eq!(replay.items, executed.items);

    let body = serde_json::to_string(&executed).unwrap();
    assert!(!body.contains("bulk generated overview"));
    assert!(!body.contains("stale generated overview"));
    assert!(!body.contains("rejected generated overview"));
    assert!(!body.contains("local:///Movies/private"));
    assert!(!body.contains("sha256-private"));
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
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:commit",
        ))
        .await
        .unwrap();

    assert_eq!(
        applied.status,
        GeneratedArtifactMetadataApplyResultStatus::Applied
    );
    assert!(applied.applied);
    assert!(applied.changed);
    assert!(!applied.idempotent_replay);
    assert!(applied.outcome_id.is_some());
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
async fn generated_artifact_metadata_apply_replays_same_idempotency_key_from_durable_outcome() {
    let fixture = generated_artifact_metadata_apply_fixture(
        r#"{"overview":"private generated overview","confidence_milli":810}"#,
    )
    .await;

    let applied = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:replay",
        ))
        .await
        .unwrap();
    assert_eq!(
        applied.status,
        GeneratedArtifactMetadataApplyResultStatus::Applied
    );
    assert!(applied.outcome_id.is_some());
    assert!(!applied.idempotent_replay);

    let replay = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:replay",
        ))
        .await
        .unwrap();

    assert_eq!(
        replay.status,
        GeneratedArtifactMetadataApplyResultStatus::Applied
    );
    assert!(replay.applied);
    assert!(replay.changed);
    assert!(replay.idempotent_replay);
    assert_eq!(replay.outcome_id, applied.outcome_id);

    let noop = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:noop-after-apply",
        ))
        .await
        .unwrap();

    assert_eq!(
        noop.status,
        GeneratedArtifactMetadataApplyResultStatus::Noop
    );
    assert!(!noop.applied);
    assert!(!noop.changed);
    assert!(!noop.idempotent_replay);
    assert!(noop.outcome_id.is_some());
    assert!(
        noop.plan
            .reasons
            .contains(&GeneratedArtifactMetadataApplyPlanReason::NoApplicableMetadataFields)
    );

    let noop_replay = fixture
        .app
        .automation()
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:noop-after-apply",
        ))
        .await
        .unwrap();

    assert_eq!(
        noop_replay.status,
        GeneratedArtifactMetadataApplyResultStatus::Noop
    );
    assert!(!noop_replay.applied);
    assert!(!noop_replay.changed);
    assert!(noop_replay.idempotent_replay);
    assert_eq!(noop_replay.outcome_id, noop.outcome_id);
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
        .apply_generated_artifact_metadata(generated_artifact_metadata_apply_request(
            fixture.artifact_id,
            "generated-artifact-apply:stale",
        ))
        .await
        .unwrap_err();

    assert!(
        err.to_string()
            .contains("generated artifact metadata apply plan is not executable")
    );
    let outcome = fixture
        .store
        .find_generated_artifact_metadata_apply_outcome(
            fixture.artifact_id,
            "generated-artifact-apply:stale",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        outcome.status,
        GeneratedArtifactMetadataApplyOutcomeStatus::Failed
    );
    assert_eq!(outcome.error_code.as_deref(), Some("plan_not_executable"));
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
    provider_id: AutomationProviderId,
    item_id: MediaItemId,
    source_id: MediaSourceId,
    artifact_id: AutomationArtifactId,
}

struct GeneratedArtifactMetadataApplyFixtureArtifact {
    item_id: MediaItemId,
    source_id: MediaSourceId,
    artifact_id: AutomationArtifactId,
}

impl GeneratedArtifactMetadataApplyFixture {
    async fn add_accepted_metadata_artifact(
        &self,
        title: &str,
        file_name: &str,
        artifact_json: &str,
    ) -> GeneratedArtifactMetadataApplyFixtureArtifact {
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
            library_id: self.library_id,
            item_id: item.id,
            locator: format!("local:///Movies/private/{file_name}"),
            file_name: file_name.to_owned(),
            size_bytes: Some(2048),
            fingerprint: Some(format!("sha256-private-{file_name}")),
        };
        self.store.upsert_media_item(&item).await.unwrap();
        self.store.upsert_media_source(&source).await.unwrap();
        let artifact_id = self
            .add_accepted_metadata_artifact_for_item(item.id, source.id, artifact_json)
            .await;

        GeneratedArtifactMetadataApplyFixtureArtifact {
            item_id: item.id,
            source_id: source.id,
            artifact_id,
        }
    }

    async fn upsert_accepted_provider_mapping(
        &self,
        item_id: MediaItemId,
        subject_key: &str,
        title: &str,
        confidence_milli: Option<u16>,
    ) {
        let subject = ProviderSubject {
            id: ProviderSubjectId::new(),
            provider: nako_core::ExternalProvider::Tmdb,
            subject_kind: nako_core::ProviderSubjectKind::Movie,
            subject_key: subject_key.to_owned(),
            title: Some(title.to_owned()),
            release_year: Some(1999),
            locale: Some("en-US".to_owned()),
        };
        self.store.upsert_provider_subject(&subject).await.unwrap();
        self.store
            .upsert_provider_mapping(&ProviderMapping {
                id: ProviderMappingId::new(),
                item_id,
                subject_id: subject.id,
                status: ProviderMappingStatus::Accepted,
                confidence_milli,
                source: MetadataSource::User,
            })
            .await
            .unwrap();
    }

    async fn add_accepted_metadata_artifact_for_item(
        &self,
        item_id: MediaItemId,
        source_id: MediaSourceId,
        artifact_json: &str,
    ) -> AutomationArtifactId {
        let job = self
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::Automation,
                resource_class: "automation.external_api".to_owned(),
                library_id: Some(self.library_id),
                source_id: Some(source_id),
                input_json: Some(
                    serde_json::to_string(&AutomationJobInput {
                        provider_id: self.provider_id,
                        capability: AutomationCapability::MetadataCleanup,
                        library_id: Some(self.library_id),
                        item_id: Some(item_id),
                        source_id: Some(source_id),
                        prompt_json:
                            r#"{"path":"local:///Movies/private/extra.mkv","token":"secret"}"#
                                .to_owned(),
                        idempotency_key: format!("metadata-cleanup:{item_id}:{source_id}"),
                    })
                    .unwrap(),
                ),
            })
            .await
            .unwrap();
        let artifact = self
            .store
            .create_automation_artifact(NewAutomationArtifact {
                id: AutomationArtifactId::new(),
                job_id: job.id,
                provider_id: self.provider_id,
                capability: AutomationCapability::MetadataCleanup,
                kind: AutomationArtifactKind::MetadataSuggestion,
                library_id: Some(self.library_id),
                item_id: Some(item_id),
                source_id: Some(source_id),
                artifact_json: artifact_json.to_owned(),
            })
            .await
            .unwrap();
        self.app
            .automation()
            .review_generated_artifact(artifact.id, GeneratedArtifactReviewDecision::Accept)
            .await
            .unwrap();

        artifact.id
    }
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
        provider_id,
        item_id: item.id,
        source_id: source.id,
        artifact_id: artifact.id,
    }
}

fn generated_artifact_metadata_apply_request(
    artifact_id: AutomationArtifactId,
    idempotency_key: &str,
) -> GeneratedArtifactMetadataApplyRequest {
    GeneratedArtifactMetadataApplyRequest {
        artifact_id,
        idempotency_key: idempotency_key.to_owned(),
    }
}
