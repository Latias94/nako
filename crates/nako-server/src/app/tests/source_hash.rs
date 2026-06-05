use super::*;
use nako_core::JobPriority;
use nako_library::{
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, SourceFingerprintHashJobInput,
    SourceFingerprintHashMode,
};

#[tokio::test]
async fn source_fingerprint_hash_enqueue_persists_safe_job_input() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;

    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Partial {
                prefix_bytes: 65_536,
            },
            priority: Some(JobPriority::High),
        })
        .await
        .unwrap();

    let input_json = job.input_json.as_deref().expect("job input json");
    let input: SourceFingerprintHashJobInput = serde_json::from_str(input_json).unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(
        job.resource_class,
        SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
    );
    assert_eq!(job.priority, JobPriority::High);
    assert_eq!(job.library_id, Some(library_id));
    assert_eq!(job.source_id, Some(source.id));
    assert_eq!(
        input,
        SourceFingerprintHashJobInput {
            library_id,
            source_id: source.id,
            source_scheme: "local".to_owned(),
            mode: SourceFingerprintHashMode::Partial {
                prefix_bytes: 65_536,
            },
        }
    );
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("Secret Path"));
    assert!(!input_json.contains("Frankorz"));
    assert!(!input_json.contains("token"));
    assert!(!input_json.contains("local:///"));
    assert!(!input_json.contains("sha256"));
    assert!(!input_json.contains("etag"));
}

#[tokio::test]
async fn source_fingerprint_hash_enqueue_rejects_missing_source_without_job() {
    let temp = tempfile::tempdir().unwrap();
    let library_id = LibraryId::new();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(source_hash_config(temp.path(), library_id), store.clone())
        .await
        .unwrap();
    let missing_source_id = MediaSourceId::new();

    let err = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: missing_source_id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap_err();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        err,
        NakoError::NotFound {
            entity: "media_source",
            id: missing_source_id.to_string(),
        }
    );
    assert!(jobs.is_empty());
}

#[tokio::test]
async fn source_fingerprint_hash_enqueue_rejects_cross_library_source_without_job() {
    let source_library_id = LibraryId::new();
    let requested_library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        source_library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv",
        None,
    )
    .await;

    let err = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id: requested_library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap_err();
    let message = err.to_string();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        message,
        "invalid input: source fingerprint hash job source does not belong to requested library"
    );
    assert!(jobs.is_empty());
    assert!(!message.contains("Hidden Movie"));
    assert!(!message.contains("Secret Path"));
    assert!(!message.contains("Frankorz"));
    assert!(!message.contains("local:///"));
}

#[tokio::test]
async fn source_fingerprint_hash_enqueue_rejects_invalid_locator_without_leaking_value() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("private-fingerprint".to_owned()),
    )
    .await;

    let err = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap_err();
    let message = err.to_string();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        message,
        "invalid input: source fingerprint hash job source locator is not a valid storage URI"
    );
    assert!(jobs.is_empty());
    assert!(!message.contains("Hidden Movie"));
    assert!(!message.contains("Secret Path"));
    assert!(!message.contains("Frankorz"));
    assert!(!message.contains("token"));
    assert!(!message.contains("private-fingerprint"));
}

async fn source_hash_app_with_source(
    library_id: LibraryId,
    locator: &str,
    fingerprint: Option<String>,
) -> (tempfile::TempDir, NakoApp, NakoDatabase, MediaSource) {
    let temp = tempfile::tempdir().unwrap();
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(source_hash_config(temp.path(), library_id), store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Hidden Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: locator.to_owned(),
        file_name: "Hidden Movie.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    (temp, app, store, source)
}

fn source_hash_config(root: &Path, library_id: LibraryId) -> NakoServerConfig {
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
        remux_staging_root: root.join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: root.to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    }
}
