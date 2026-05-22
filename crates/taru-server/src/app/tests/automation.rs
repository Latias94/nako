use super::*;
use taru_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationCapability, AutomationJobInput,
    AutomationProviderId, AutomationProviderStatus, AutomationRepository,
    GeneratedArtifactReadinessStatus, GeneratedArtifactTargetKind, NewAutomationArtifact,
    NewAutomationProviderConfig,
};

#[tokio::test]
async fn automation_app_lists_generated_artifact_proposals_without_raw_payloads_or_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
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
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("taru-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().join("Movies"),
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let library = Library {
        id: library_id,
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(taru_core::LibraryPreset::Movies),
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
            secret_env: Some("TARU_AUTOMATION_SECRET".to_owned()),
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
