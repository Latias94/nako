use super::*;
use crate::app::jobs::LibraryScanScheduleOutcome;
use nako_core::{JobPriority, ScanRepository, ScanSnapshotId, ScanStatus, SourceState};
use nako_library::{
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, SourceFingerprintHashJobInput,
    SourceFingerprintHashJobSummary, SourceFingerprintHashMode,
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
async fn source_fingerprint_hash_admin_overview_summary_aggregates_redacted_counts() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("source:v1:content_hash:sha256:private-content-hash".to_owned()),
    )
    .await;
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: source.item_id,
            locator: "webdav://private-host.example.test/private/alternate.mkv?token=secret"
                .to_owned(),
            file_name: "Alternate.mkv".to_owned(),
            size_bytes: Some(84),
            fingerprint: Some("webdav:etag=private-etag".to_owned()),
        })
        .await
        .unwrap();
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: source.item_id,
            locator: "local:///Users/Frankorz/Secret Path/No Fingerprint.mkv".to_owned(),
            file_name: "No Fingerprint.mkv".to_owned(),
            size_bytes: Some(21),
            fingerprint: None,
        })
        .await
        .unwrap();

    app.source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: Some(JobPriority::High),
        })
        .await
        .unwrap();
    let succeeded = store
        .enqueue_job(new_source_hash_job(library_id, source.id))
        .await
        .unwrap();
    store.start_job(succeeded.id).await.unwrap();
    store
        .succeed_job(succeeded.id, Some(r#"{"safe":"summary"}"#.to_owned()))
        .await
        .unwrap();
    let failed = store
        .enqueue_job(new_source_hash_job(library_id, source.id))
        .await
        .unwrap();
    store.start_job(failed.id).await.unwrap();
    store
        .fail_job(
            failed.id,
            "source fingerprint hash execution failed".to_owned(),
        )
        .await
        .unwrap();

    let summary = app.source_hash().admin_overview_summary().await.unwrap();
    let body = serde_json::to_string(&summary).unwrap();

    assert_eq!(summary.total_sources, 3);
    assert_eq!(summary.fingerprinted_sources, 2);
    assert_eq!(summary.content_hash_sources, 1);
    assert_eq!(summary.queued_jobs, 1);
    assert_eq!(summary.claimable_jobs, 1);
    assert_eq!(summary.delayed_retry_jobs, 0);
    assert_eq!(summary.succeeded_jobs, 1);
    assert_eq!(summary.failed_jobs, 1);
    assert!(summary.oldest_queued_at.is_some());
    assert_eq!(summary.next_retry_at, None);
    assert!(!body.contains("Frankorz"));
    assert!(!body.contains("Secret Path"));
    assert!(!body.contains("private-content-hash"));
    assert!(!body.contains("private-etag"));
    assert!(!body.contains("private-host"));
    assert!(!body.contains("token=secret"));
    assert!(!body.contains("source:v1:content_hash"));
}

#[tokio::test]
async fn source_fingerprint_hash_prepare_recovers_in_memory_execution_request() {
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
            priority: None,
        })
        .await
        .unwrap();
    let prepared = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&job)
        .await
        .unwrap();
    let persisted = store.get_job(job.id).await.unwrap().unwrap();

    assert_eq!(prepared.job_id, job.id);
    assert_eq!(prepared.library_id, library_id);
    assert_eq!(prepared.source_id, source.id);
    assert_eq!(prepared.source_scheme, "local");
    assert_eq!(
        prepared.mode,
        SourceFingerprintHashMode::Partial {
            prefix_bytes: 65_536,
        }
    );
    assert_eq!(
        prepared.request.mode,
        SourceFingerprintHashMode::Partial {
            prefix_bytes: 65_536,
        }
    );
    assert_eq!(prepared.request.uri.as_str(), source.locator);
    assert_eq!(persisted.status, JobStatus::Queued);
    assert!(persisted.started_at.is_none());
    assert!(persisted.completed_at.is_none());
}

#[tokio::test]
async fn source_fingerprint_hash_execute_claims_job_and_persists_safe_summary() {
    let library_id = LibraryId::new();
    let (temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Hidden Movie.mkv",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    fs::write(temp.path().join("Hidden Movie.mkv"), b"abcdef").unwrap();
    let existing_state = seed_source_hash_state(&store, library_id, &source).await;
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap();

    let output = app
        .source_hash()
        .execute_source_fingerprint_hash_job(job.id)
        .await
        .unwrap();
    let persisted = store.get_job(job.id).await.unwrap().unwrap();
    let summary_json = persisted.summary_json.as_deref().expect("summary json");
    let summary: SourceFingerprintHashJobSummary = serde_json::from_str(summary_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(job.id),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();
    let persisted_fingerprint =
        assert_source_hash_evidence_persisted(&store, &source, Some(&existing_state), "abcdef")
            .await;

    assert_eq!(output.job.id, job.id);
    assert_eq!(output.job.status, JobStatus::Succeeded);
    assert_eq!(persisted.status, JobStatus::Succeeded);
    assert_eq!(output.summary, summary);
    assert_eq!(summary.mode, SourceFingerprintHashMode::Full);
    assert_eq!(
        summary.evidence_kind,
        nako_core::SourceFingerprintEvidenceKind::ContentHash
    );
    assert_eq!(summary.confidence_milli, 1_000);
    assert!(!summary.stale);
    assert_eq!(summary.bytes_hashed, 6);
    assert!(persisted.started_at.is_some());
    assert!(persisted.completed_at.is_some());
    assert!(claim.is_none());
    assert!(!summary_json.contains("Hidden Movie"));
    assert!(!summary_json.contains("local:///"));
    assert!(!summary_json.contains("sha256"));
    assert!(!summary_json.contains("aaaaaaaa"));
    assert!(!summary_json.contains(r#""fingerprint""#));
    assert!(!summary_json.contains(&persisted_fingerprint));
}

#[tokio::test]
async fn source_fingerprint_hash_scheduler_executes_claimed_job_and_persists_safe_summary() {
    let library_id = LibraryId::new();
    let (temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Hidden Movie.mkv",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    fs::write(temp.path().join("Hidden Movie.mkv"), b"abcdef").unwrap();
    let existing_state = seed_source_hash_state(&store, library_id, &source).await;
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap();

    let outcome = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_source_hash_runtime_job(&app).await;
    let persisted = store.get_job(job.id).await.unwrap().unwrap();
    let summary_json = persisted.summary_json.as_deref().expect("summary json");
    let summary: SourceFingerprintHashJobSummary = serde_json::from_str(summary_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(job.id),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();
    let persisted_fingerprint =
        assert_source_hash_evidence_persisted(&store, &source, Some(&existing_state), "abcdef")
            .await;

    assert_eq!(outcome, LibraryScanScheduleOutcome::Scheduled(job.id));
    assert_eq!(persisted.status, JobStatus::Succeeded);
    assert_eq!(summary.mode, SourceFingerprintHashMode::Full);
    assert_eq!(
        summary.evidence_kind,
        nako_core::SourceFingerprintEvidenceKind::ContentHash
    );
    assert_eq!(summary.confidence_milli, 1_000);
    assert!(!summary.stale);
    assert_eq!(summary.bytes_hashed, 6);
    assert!(persisted.started_at.is_some());
    assert!(persisted.completed_at.is_some());
    assert!(claim.is_none());
    assert!(!summary_json.contains("Hidden Movie"));
    assert!(!summary_json.contains("local:///"));
    assert!(!summary_json.contains("sha256"));
    assert!(!summary_json.contains("aaaaaaaa"));
    assert!(!summary_json.contains("abcdef"));
    assert!(!summary_json.contains(r#""fingerprint""#));
    assert!(!summary_json.contains(&persisted_fingerprint));
}

#[tokio::test]
async fn source_fingerprint_hash_scheduler_ignores_unrelated_claimable_job_window() {
    let library_id = LibraryId::new();
    let (temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Hidden Movie.mkv",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    fs::write(temp.path().join("Hidden Movie.mkv"), b"abcdef").unwrap();

    for _ in 0..PageRequest::MAX_LIMIT {
        store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: "metadata.tmdb".to_owned(),
                priority: JobPriority::High,
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();
    }
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: Some(JobPriority::Normal),
        })
        .await
        .unwrap();

    let outcome = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_source_hash_runtime_job(&app).await;
    let persisted = store.get_job(job.id).await.unwrap().unwrap();

    assert_eq!(outcome, LibraryScanScheduleOutcome::Scheduled(job.id));
    assert_eq!(persisted.status, JobStatus::Succeeded);
    assert!(persisted.summary_json.is_some());
}

#[tokio::test]
async fn source_fingerprint_hash_scheduler_preserves_starved_disk_scan_order_across_job_kinds() {
    let library_id = LibraryId::new();
    let (temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Hidden Movie.mkv",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    fs::write(temp.path().join("Hidden Movie.mkv"), b"abcdef").unwrap();
    let hash_job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: Some(JobPriority::Low),
        })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let scan_job = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            priority: JobPriority::High,
            library_id: Some(library_id),
            source_id: None,
            input_json: None,
        })
        .await
        .unwrap();

    let outcome = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_source_hash_runtime_job(&app).await;
    let persisted_hash = store.get_job(hash_job.id).await.unwrap().unwrap();

    assert_eq!(outcome, LibraryScanScheduleOutcome::Scheduled(hash_job.id));
    assert_eq!(persisted_hash.status, JobStatus::Succeeded);
    assert!(persisted_hash.summary_json.is_some());
    assert_ne!(scan_job.id, hash_job.id);
}

#[tokio::test]
async fn source_fingerprint_hash_scheduler_persists_redacted_execution_failure() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Missing Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap();

    let outcome = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_source_hash_runtime_failure(&app).await;
    let persisted = store.get_job(job.id).await.unwrap().unwrap();
    let error = persisted.error.as_deref().expect("job error");

    assert_eq!(outcome, LibraryScanScheduleOutcome::Scheduled(job.id));
    assert_eq!(persisted.status, JobStatus::Failed);
    assert_eq!(persisted.summary_json, None);
    assert!(!error.contains("Missing Movie"));
    assert!(!error.contains("Secret Path"));
    assert!(!error.contains("Frankorz"));
    assert!(!error.contains("token"));
    assert!(!error.contains("sha256"));
    assert!(!error.contains("aaaaaaaa"));
    assert!(!error.contains("local:///"));
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

#[tokio::test]
async fn source_fingerprint_hash_prepare_rejects_wrong_kind_or_resource_class() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) =
        source_hash_app_with_source(library_id, "local:///Movies/Hidden Movie.mkv", None).await;
    let input_json = source_hash_job_input_json(
        library_id,
        source.id,
        "local",
        SourceFingerprintHashMode::Full,
    );
    let wrong_kind = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: "disk.scan".to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(input_json.clone()),
        })
        .await
        .unwrap();
    let wrong_resource = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: "disk.scan.secret.path".to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(input_json),
        })
        .await
        .unwrap();

    let wrong_kind_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&wrong_kind)
        .await
        .unwrap_err();
    let wrong_resource_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&wrong_resource)
        .await
        .unwrap_err();

    assert_eq!(
        wrong_kind_err.to_string(),
        "invalid input: job is not a source fingerprint hash job"
    );
    assert_eq!(
        wrong_resource_err.to_string(),
        "invalid input: source fingerprint hash job uses unsupported resource class"
    );
    assert!(!wrong_resource_err.to_string().contains("secret.path"));
}

#[tokio::test]
async fn source_fingerprint_hash_prepare_rejects_missing_or_unsafe_input_without_leak() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("private-fingerprint".to_owned()),
    )
    .await;
    let missing_input = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: None,
        })
        .await
        .unwrap();
    let malformed_input = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                "{\"secret\":\"local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret\""
                    .to_owned(),
            ),
        })
        .await
        .unwrap();
    let unsafe_input_json = serde_json::json!({
        "library_id": library_id,
        "source_id": source.id,
        "source_scheme": "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        "mode": "full",
    })
    .to_string();
    let unsafe_input = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(unsafe_input_json),
        })
        .await
        .unwrap();

    let missing_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&missing_input)
        .await
        .unwrap_err();
    let malformed_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&malformed_input)
        .await
        .unwrap_err();
    let unsafe_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&unsafe_input)
        .await
        .unwrap_err();
    let unsafe_message = unsafe_err.to_string();

    assert_eq!(
        missing_err.to_string(),
        "invalid input: source fingerprint hash job input is missing"
    );
    assert_eq!(
        malformed_err.to_string(),
        "invalid input: source fingerprint hash job input is invalid"
    );
    assert!(!malformed_err.to_string().contains("Hidden Movie"));
    assert!(!malformed_err.to_string().contains("Secret Path"));
    assert!(!malformed_err.to_string().contains("Frankorz"));
    assert!(!malformed_err.to_string().contains("token"));
    assert!(!malformed_err.to_string().contains("local:///"));
    assert!(unsafe_message.contains(
        "source fingerprint hash job source scheme must contain only scheme-safe ASCII characters"
    ));
    assert!(!unsafe_message.contains("Hidden Movie"));
    assert!(!unsafe_message.contains("Secret Path"));
    assert!(!unsafe_message.contains("Frankorz"));
    assert!(!unsafe_message.contains("token"));
    assert!(!unsafe_message.contains("private-fingerprint"));
    assert!(!unsafe_message.contains("local:///"));
}

#[tokio::test]
async fn source_fingerprint_hash_prepare_rejects_binding_mismatch_without_leak() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv",
        None,
    )
    .await;
    store
        .upsert_library(&Library {
            id: other_library_id,
            name: "Other".to_owned(),
            roots: vec!["local:///Other".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let mut other_source = source.clone();
    other_source.id = MediaSourceId::new();
    other_source.locator = "local:///Other/Hidden Movie.mkv".to_owned();
    store.upsert_media_source(&other_source).await.unwrap();
    let library_mismatch = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(other_library_id),
            source_id: Some(source.id),
            input_json: Some(source_hash_job_input_json(
                library_id,
                source.id,
                "local",
                SourceFingerprintHashMode::Full,
            )),
        })
        .await
        .unwrap();
    let source_mismatch = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(other_source.id),
            input_json: Some(source_hash_job_input_json(
                library_id,
                source.id,
                "local",
                SourceFingerprintHashMode::Full,
            )),
        })
        .await
        .unwrap();

    let library_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&library_mismatch)
        .await
        .unwrap_err();
    let source_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&source_mismatch)
        .await
        .unwrap_err();
    let library_message = library_err.to_string();
    let source_message = source_err.to_string();

    assert_eq!(
        library_message,
        "invalid input: source fingerprint hash job library binding does not match input"
    );
    assert_eq!(
        source_message,
        "invalid input: source fingerprint hash job source binding does not match input"
    );
    assert!(!library_message.contains("Hidden Movie"));
    assert!(!library_message.contains("Secret Path"));
    assert!(!library_message.contains("Frankorz"));
    assert!(!library_message.contains("local:///"));
    assert!(!source_message.contains("Hidden Movie"));
    assert!(!source_message.contains("Secret Path"));
    assert!(!source_message.contains("Frankorz"));
    assert!(!source_message.contains("local:///"));
}

#[tokio::test]
async fn source_fingerprint_hash_prepare_rejects_changed_locator_scheme_without_leak() {
    let library_id = LibraryId::new();
    let (_temp, app, store, mut source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("private-fingerprint".to_owned()),
    )
    .await;
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: source.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap();
    source.locator =
        "webdav:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret".to_owned();
    store.upsert_media_source(&source).await.unwrap();

    let err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&job)
        .await
        .unwrap_err();
    let message = err.to_string();

    assert_eq!(
        message,
        "conflict: source fingerprint hash job source locator scheme changed since enqueue"
    );
    assert!(!message.contains("Hidden Movie"));
    assert!(!message.contains("Secret Path"));
    assert!(!message.contains("Frankorz"));
    assert!(!message.contains("token"));
    assert!(!message.contains("private-fingerprint"));
    assert!(!message.contains("webdav:///"));
    assert!(!message.contains("local:///"));
}

async fn wait_for_source_hash_runtime_job(app: &NakoApp) {
    for _ in 0..500 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == 1
            && diagnostics.cancelled_jobs == 0
            && diagnostics.failed_jobs == 0
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "source hash scheduler job did not finish successfully: {:?}",
        app.runtime_diagnostics()
    );
}

async fn wait_for_source_hash_runtime_failure(app: &NakoApp) {
    for _ in 0..500 {
        let diagnostics = app.runtime_diagnostics();
        if diagnostics.succeeded_jobs == 0
            && diagnostics.cancelled_jobs == 0
            && diagnostics.failed_jobs == 1
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    panic!(
        "source hash scheduler job did not fail as expected: {:?}",
        app.runtime_diagnostics()
    );
}

async fn seed_source_hash_state(
    store: &NakoDatabase,
    library_id: LibraryId,
    source: &MediaSource,
) -> SourceState {
    let scan_id = ScanSnapshotId::new();
    store
        .begin_scan_snapshot(scan_id, library_id, "local:///")
        .await
        .unwrap();
    let state = SourceState {
        library_id,
        source_id: None,
        uri: source.locator.clone(),
        size_bytes: Some(123_456),
        modified_at: Some("2026-05-16T00:00:00Z".to_owned()),
        etag: Some("existing-private-etag".to_owned()),
        fingerprint: Some("existing-source-state-fingerprint".to_owned()),
        last_seen_scan_id: scan_id,
        tombstoned: true,
    };

    store.upsert_source_state(&state).await.unwrap();
    store
        .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
        .await
        .unwrap();

    state
}

async fn assert_source_hash_evidence_persisted(
    store: &NakoDatabase,
    source: &MediaSource,
    existing_state: Option<&SourceState>,
    forbidden_content: &str,
) -> String {
    let persisted_source = store.get_media_source(source.id).await.unwrap().unwrap();
    let fingerprint = persisted_source
        .fingerprint
        .clone()
        .expect("persisted source fingerprint");
    let mut expected_source = source.clone();
    expected_source.fingerprint = Some(fingerprint.clone());

    assert_eq!(persisted_source, expected_source);
    assert!(fingerprint.starts_with("source:v1:content_hash:sha256:"));
    assert!(!fingerprint.contains(&source.locator));
    assert!(!fingerprint.contains(forbidden_content));

    if let Some(existing_state) = existing_state {
        let persisted_state = store
            .get_source_state(source.library_id, &source.locator)
            .await
            .unwrap()
            .unwrap();
        let mut expected_state = existing_state.clone();
        expected_state.fingerprint = Some(fingerprint.clone());

        assert_ne!(persisted_state.fingerprint, existing_state.fingerprint);
        assert_eq!(persisted_state, expected_state);
    }

    fingerprint
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

fn new_source_hash_job(library_id: LibraryId, source_id: MediaSourceId) -> NewJob {
    NewJob {
        id: JobId::new(),
        kind: JobKind::SourceFingerprintHash,
        resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
        priority: JobPriority::default(),
        library_id: Some(library_id),
        source_id: Some(source_id),
        input_json: Some(source_hash_job_input_json(
            library_id,
            source_id,
            "local",
            SourceFingerprintHashMode::Full,
        )),
    }
}

fn source_hash_job_input_json(
    library_id: LibraryId,
    source_id: MediaSourceId,
    source_scheme: &str,
    mode: SourceFingerprintHashMode,
) -> String {
    serde_json::to_string(
        &SourceFingerprintHashJobInput::new(library_id, source_id, source_scheme, mode).unwrap(),
    )
    .unwrap()
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
