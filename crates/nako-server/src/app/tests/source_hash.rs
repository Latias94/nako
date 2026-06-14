use super::*;
use crate::app::job_runtime::DurableJobTraceContext;
use crate::app::jobs::LibraryScanScheduleOutcome;
use crate::app::source_hash::{
    ScanOriginatedSourceFingerprintHashOutcome, ScanOriginatedSourceFingerprintHashPolicy,
};
use nako_core::{
    JobPriority, ScanRepository, ScanSnapshotId, ScanStatus, SourceDuplicateReconciliationAction,
    SourceDuplicateRepository, SourceFingerprintEscalationAction,
    SourceFingerprintEscalationDecision, SourceFingerprintEscalationReason,
    SourceFingerprintEvidenceKind, SourceState,
};
use nako_library::{
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, ScanSourceFingerprintHashTrigger,
    SourceFingerprintHashJobInput, SourceFingerprintHashJobSummary, SourceFingerprintHashMode,
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
    let trace_context = DurableJobTraceContext::from_request_id("REQ-SOURCE_123.Trace").unwrap();

    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: source.id,
                mode: SourceFingerprintHashMode::Partial {
                    prefix_bytes: 65_536,
                },
                priority: Some(JobPriority::High),
            },
            Some(&trace_context),
        )
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
            request_id: Some("req-source_123.trace".to_owned()),
        }
    );
    assert_eq!(input.request_id.as_deref(), Some("req-source_123.trace"));
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("Secret Path"));
    assert!(!input_json.contains("Frankorz"));
    assert!(!input_json.contains("token"));
    assert!(!input_json.contains("local:///"));
    assert!(!input_json.contains("sha256"));
    assert!(!input_json.contains("etag"));
    assert!(input_json.contains(r#""request_id":"req-source_123.trace""#));
}

#[tokio::test]
async fn scan_originated_source_fingerprint_hash_enqueue_respects_policy_and_decision() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("source:v1:backend_fingerprint:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let partial_trigger = scan_source_hash_trigger(
        source.id,
        SourceFingerprintEscalationAction::PartialHash,
        Some(SourceFingerprintHashMode::Partial { prefix_bytes: 1 }),
    );
    let none_trigger =
        scan_source_hash_trigger(source.id, SourceFingerprintEscalationAction::None, None);
    let trace_context = DurableJobTraceContext::from_request_id("REQ-SCAN_456.Trace").unwrap();

    let disabled = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &partial_trigger,
            ScanOriginatedSourceFingerprintHashPolicy {
                enabled: false,
                partial_prefix_bytes: 131_072,
                priority: JobPriority::Low,
            },
        )
        .await
        .unwrap();
    let advisory = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &none_trigger,
            ScanOriginatedSourceFingerprintHashPolicy::default(),
        )
        .await
        .unwrap();
    let enqueued = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash_with_trace_context(
            library_id,
            &partial_trigger,
            ScanOriginatedSourceFingerprintHashPolicy {
                enabled: true,
                partial_prefix_bytes: 131_072,
                priority: JobPriority::Low,
            },
            Some(&trace_context),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(
        disabled,
        ScanOriginatedSourceFingerprintHashOutcome::AdvisoryOnly
    );
    assert_eq!(
        advisory,
        ScanOriginatedSourceFingerprintHashOutcome::AdvisoryOnly
    );
    let ScanOriginatedSourceFingerprintHashOutcome::Enqueued(job) = enqueued else {
        panic!("expected scan-originated source hash job");
    };
    let input_json = job.input_json.as_deref().expect("job input json");
    let input: SourceFingerprintHashJobInput = serde_json::from_str(input_json).unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.priority, JobPriority::Low);
    assert_eq!(job.library_id, Some(library_id));
    assert_eq!(job.source_id, Some(source.id));
    assert_eq!(
        input.mode,
        SourceFingerprintHashMode::Partial {
            prefix_bytes: 131_072,
        }
    );
    assert_eq!(input.request_id.as_deref(), Some("req-scan_456.trace"));
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("Secret Path"));
    assert!(!input_json.contains("Frankorz"));
    assert!(!input_json.contains("token"));
    assert!(!input_json.contains("local:///"));
    assert!(!input_json.contains("sha256"));
    assert!(input_json.contains(r#""request_id":"req-scan_456.trace""#));
}

#[tokio::test]
async fn scan_originated_full_hash_enqueue_ignores_partial_prefix_validation() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) =
        source_hash_app_with_source(library_id, "local:///Movies/Hidden Movie.mkv", None).await;
    let trigger = scan_source_hash_trigger(
        source.id,
        SourceFingerprintEscalationAction::FullHash,
        Some(SourceFingerprintHashMode::Full),
    );

    let enqueued = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &trigger,
            ScanOriginatedSourceFingerprintHashPolicy {
                enabled: true,
                partial_prefix_bytes: 0,
                priority: JobPriority::Normal,
            },
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    let ScanOriginatedSourceFingerprintHashOutcome::Enqueued(job) = enqueued else {
        panic!("expected full source hash job");
    };
    let input_json = job.input_json.as_deref().expect("job input json");
    let input: SourceFingerprintHashJobInput = serde_json::from_str(input_json).unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(input.mode, SourceFingerprintHashMode::Full);
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("local:///"));
}

#[tokio::test]
async fn scan_originated_source_fingerprint_hash_enqueue_is_idempotent_for_incomplete_same_mode() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) =
        source_hash_app_with_source(library_id, "local:///Movies/Hidden Movie.mkv", None).await;
    let trigger = scan_source_hash_trigger(
        source.id,
        SourceFingerprintEscalationAction::FullHash,
        Some(SourceFingerprintHashMode::Full),
    );

    let first = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &trigger,
            ScanOriginatedSourceFingerprintHashPolicy::default(),
        )
        .await
        .unwrap();
    let second = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &trigger,
            ScanOriginatedSourceFingerprintHashPolicy::default(),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    let ScanOriginatedSourceFingerprintHashOutcome::Enqueued(first_job) = first else {
        panic!("expected initial enqueue");
    };
    let ScanOriginatedSourceFingerprintHashOutcome::AlreadyQueued(second_job) = second else {
        panic!("expected existing queued job");
    };

    assert_eq!(first_job.id, second_job.id);
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, first_job.id);

    store.start_job(first_job.id).await.unwrap();
    let third = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &trigger,
            ScanOriginatedSourceFingerprintHashPolicy::default(),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    let ScanOriginatedSourceFingerprintHashOutcome::AlreadyQueued(third_job) = third else {
        panic!("expected running job to block duplicate enqueue");
    };
    assert_eq!(third_job.id, first_job.id);
    assert_eq!(jobs.len(), 1);
}

#[tokio::test]
async fn scan_originated_source_fingerprint_hash_enqueue_ignores_mismatched_input_binding_decoy() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("source:v1:backend_fingerprint:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let trigger = scan_source_hash_trigger(
        source.id,
        SourceFingerprintEscalationAction::FullHash,
        Some(SourceFingerprintHashMode::Full),
    );
    let decoy = store
        .enqueue_job(NewJob {
            input_json: Some(source_hash_job_input_json(
                LibraryId::new(),
                MediaSourceId::new(),
                "local",
                SourceFingerprintHashMode::Full,
            )),
            ..new_source_hash_job(library_id, source.id)
        })
        .await
        .unwrap();

    let outcome = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &trigger,
            ScanOriginatedSourceFingerprintHashPolicy::default(),
        )
        .await
        .unwrap();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    let ScanOriginatedSourceFingerprintHashOutcome::Enqueued(job) = outcome else {
        panic!("expected mismatched input binding decoy to be ignored");
    };
    let input_json = job.input_json.as_deref().expect("job input json");
    let input: SourceFingerprintHashJobInput = serde_json::from_str(input_json).unwrap();

    assert_ne!(job.id, decoy.id);
    assert_eq!(jobs.len(), 2);
    assert!(jobs.iter().any(|candidate| candidate.id == decoy.id));
    assert!(jobs.iter().any(|candidate| candidate.id == job.id));
    assert_eq!(job.kind, JobKind::SourceFingerprintHash);
    assert_eq!(job.status, JobStatus::Queued);
    assert_eq!(job.library_id, Some(library_id));
    assert_eq!(job.source_id, Some(source.id));
    assert_eq!(input.library_id, library_id);
    assert_eq!(input.source_id, source.id);
    assert_eq!(input.source_scheme, "local");
    assert_eq!(input.mode, SourceFingerprintHashMode::Full);
    assert!(!input_json.contains("Hidden Movie"));
    assert!(!input_json.contains("Secret Path"));
    assert!(!input_json.contains("Frankorz"));
    assert!(!input_json.contains("token"));
    assert!(!input_json.contains("local:///"));
    assert!(!input_json.contains("sha256"));
}

#[tokio::test]
async fn scan_originated_source_fingerprint_hash_enqueue_finds_duplicate_beyond_first_job_page() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) =
        source_hash_app_with_source(library_id, "local:///Movies/Hidden Movie.mkv", None).await;
    let trigger = scan_source_hash_trigger(
        source.id,
        SourceFingerprintEscalationAction::FullHash,
        Some(SourceFingerprintHashMode::Full),
    );
    let duplicate_job = store
        .enqueue_job(new_source_hash_job(library_id, source.id))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    for index in 0..=PageRequest::MAX_LIMIT {
        store
            .enqueue_job(NewJob {
                id: JobId::new(),
                input_json: Some(source_hash_job_input_json(
                    library_id,
                    source.id,
                    "local",
                    SourceFingerprintHashMode::Partial {
                        prefix_bytes: u64::from(index) + 1,
                    },
                )),
                ..new_source_hash_job(library_id, source.id)
            })
            .await
            .unwrap();
    }

    let outcome = app
        .source_hash()
        .enqueue_scan_originated_source_fingerprint_hash(
            library_id,
            &trigger,
            ScanOriginatedSourceFingerprintHashPolicy::default(),
        )
        .await
        .unwrap();
    let ScanOriginatedSourceFingerprintHashOutcome::AlreadyQueued(existing) = outcome else {
        panic!("expected duplicate beyond first job page to block enqueue");
    };
    let first_page = store
        .list_jobs(
            nako_core::JobListFilter {
                status: Some(JobStatus::Queued),
                kind: Some(JobKind::SourceFingerprintHash),
                resource_class: Some(SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned()),
                library_id: Some(library_id),
                source_id: Some(source.id),
            },
            PageRequest::new(PageRequest::MAX_LIMIT, 0),
        )
        .await
        .unwrap();
    let second_page = store
        .list_jobs(
            nako_core::JobListFilter {
                status: Some(JobStatus::Queued),
                kind: Some(JobKind::SourceFingerprintHash),
                resource_class: Some(SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned()),
                library_id: Some(library_id),
                source_id: Some(source.id),
            },
            PageRequest::new(PageRequest::MAX_LIMIT, u64::from(PageRequest::MAX_LIMIT)),
        )
        .await
        .unwrap();

    assert_eq!(existing.id, duplicate_job.id);
    assert!(!first_page.iter().any(|job| job.id == duplicate_job.id));
    assert!(second_page.iter().any(|job| job.id == duplicate_job.id));
    assert_eq!(first_page.len(), PageRequest::MAX_LIMIT as usize);
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
async fn source_fingerprint_hash_retry_creates_safe_delayed_retry_job() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let trace_context = DurableJobTraceContext::from_request_id("REQ-RETRY_789.Trace").unwrap();
    let source_job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: source.id,
                mode: SourceFingerprintHashMode::Partial {
                    prefix_bytes: 65_536,
                },
                priority: Some(JobPriority::High),
            },
            Some(&trace_context),
        )
        .await
        .unwrap();
    store.start_job(source_job.id).await.unwrap();
    let failed = store
        .fail_job(
            source_job.id,
            "source hash failed for local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret sha256-private-source-hash".to_owned(),
        )
        .await
        .unwrap();

    let retry = app
        .source_hash()
        .retry_source_fingerprint_hash_job(RetrySourceFingerprintHashRequest {
            job_id: failed.id,
            max_attempts: Some(3),
            next_attempt_at: Some("9999-01-01T08:00:00+08:00".to_owned()),
        })
        .await
        .unwrap();
    let retry_input_json = retry.input_json.as_deref().expect("retry input json");
    let retry_input: SourceFingerprintHashJobInput =
        serde_json::from_str(retry_input_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(retry.id),
                kind: Some(JobKind::SourceFingerprintHash),
                resource_class: Some(SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned()),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();
    let summary = app.source_hash().admin_overview_summary().await.unwrap();
    let summary_body = serde_json::to_string(&summary).unwrap();

    assert_ne!(retry.id, failed.id);
    assert_eq!(retry.kind, JobKind::SourceFingerprintHash);
    assert_eq!(retry.status, JobStatus::Queued);
    assert_eq!(
        retry.resource_class,
        SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
    );
    assert_eq!(retry.priority, JobPriority::High);
    assert_eq!(retry.library_id, Some(library_id));
    assert_eq!(retry.source_id, Some(source.id));
    assert_eq!(retry.input_json, failed.input_json);
    assert_eq!(retry.attempt, 2);
    assert_eq!(retry.max_attempts, 3);
    assert_eq!(retry.retry_of_job_id, Some(failed.id));
    assert_eq!(
        retry.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    assert_eq!(
        retry_input,
        SourceFingerprintHashJobInput {
            library_id,
            source_id: source.id,
            source_scheme: "local".to_owned(),
            mode: SourceFingerprintHashMode::Partial {
                prefix_bytes: 65_536,
            },
            request_id: Some("req-retry_789.trace".to_owned()),
        }
    );
    assert_eq!(
        retry_input.request_id.as_deref(),
        Some("req-retry_789.trace")
    );
    assert!(claim.is_none(), "future retry must not be claimable");
    assert_eq!(summary.queued_jobs, 1);
    assert_eq!(summary.claimable_jobs, 0);
    assert_eq!(summary.delayed_retry_jobs, 1);
    assert_eq!(
        summary.next_retry_at.as_deref(),
        Some("9999-01-01T00:00:00Z")
    );
    assert!(!retry_input_json.contains("Hidden Movie"));
    assert!(!retry_input_json.contains("Secret Path"));
    assert!(!retry_input_json.contains("Frankorz"));
    assert!(!retry_input_json.contains("token"));
    assert!(!retry_input_json.contains("local:///"));
    assert!(!retry_input_json.contains("sha256"));
    assert!(!summary_body.contains("Hidden Movie"));
    assert!(!summary_body.contains("Secret Path"));
    assert!(!summary_body.contains("sha256-private-source-hash"));
    assert!(!summary_body.contains("token=secret"));
    assert!(!summary_body.contains("input_json"));
}

#[tokio::test]
async fn source_fingerprint_hash_retry_scheduler_executes_due_job() {
    let library_id = LibraryId::new();
    let (temp, app, store, source) =
        source_hash_app_with_source(library_id, "local:///Hidden Movie.mkv", None).await;
    fs::write(temp.path().join("Hidden Movie.mkv"), b"abcdef").unwrap();
    let trace_context =
        DurableJobTraceContext::from_request_id("REQ-RETRY-SCHED_123.Trace").unwrap();
    let source_job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: source.id,
                mode: SourceFingerprintHashMode::Full,
                priority: Some(JobPriority::Normal),
            },
            Some(&trace_context),
        )
        .await
        .unwrap();
    store.start_job(source_job.id).await.unwrap();
    let failed = store
        .fail_job(source_job.id, "source hash failed".to_owned())
        .await
        .unwrap();

    let retry = app
        .source_hash()
        .retry_source_fingerprint_hash_job(RetrySourceFingerprintHashRequest {
            job_id: failed.id,
            max_attempts: Some(3),
            next_attempt_at: Some("0001-01-01T00:00:00Z".to_owned()),
        })
        .await
        .unwrap();

    let outcome = app
        .library_scan()
        .schedule_queued_library_scans()
        .await
        .unwrap();
    wait_for_source_hash_runtime_job(&app).await;
    let persisted = store.get_job(retry.id).await.unwrap().unwrap();
    let summary_json = persisted.summary_json.as_deref().expect("summary json");
    let summary: SourceFingerprintHashJobSummary = serde_json::from_str(summary_json).unwrap();
    let claim = nako_core::JobLeaseRepository::claim_next_job_lease(
        &store,
        nako_core::JobLeaseClaimRequest {
            worker_id: nako_core::JobWorkerId::new(),
            lease_duration_ms: 10_000,
            filter: nako_core::JobLeaseClaimFilter {
                job_id: Some(retry.id),
                ..nako_core::JobLeaseClaimFilter::default()
            },
        },
    )
    .await
    .unwrap();

    assert_eq!(outcome, LibraryScanScheduleOutcome::Scheduled(retry.id));
    assert_eq!(persisted.status, JobStatus::Succeeded);
    assert_eq!(persisted.retry_of_job_id, Some(failed.id));
    assert_eq!(persisted.next_attempt_at, None);
    assert_eq!(summary.mode, SourceFingerprintHashMode::Full);
    assert_eq!(
        summary.evidence_kind,
        nako_core::SourceFingerprintEvidenceKind::ContentHash
    );
    assert_eq!(summary.bytes_hashed, 6);
    assert!(claim.is_none());
    assert!(!summary_json.contains("Hidden Movie"));
    assert!(!summary_json.contains("local:///"));
    assert!(!summary_json.contains("abcdef"));
    assert!(!summary_json.contains(r#""fingerprint""#));
}

#[tokio::test]
async fn source_fingerprint_hash_retry_rejects_job_contract_drift_without_leak() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let input_json = source_hash_job_input_json(
        library_id,
        source.id,
        "local",
        SourceFingerprintHashMode::Full,
    );
    let wrong_kind = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(input_json.clone()),
        },
    )
    .await;
    let wrong_resource = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: "disk.scan.secret.path".to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(input_json),
        },
    )
    .await;

    let wrong_kind_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, wrong_kind.id, None).await;
    let wrong_resource_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, wrong_resource.id, None).await;

    assert_eq!(
        wrong_kind_message,
        "invalid input: job is not a source fingerprint hash job"
    );
    assert_eq!(
        wrong_resource_message,
        "invalid input: source fingerprint hash job uses unsupported resource class"
    );
    assert!(!wrong_resource_message.contains("secret.path"));
}

#[tokio::test]
async fn source_fingerprint_hash_retry_rejects_input_contract_drift_without_leak() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let missing_input = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            input_json: None,
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;
    let malformed_input = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            input_json: Some(
                "{\"input_json\":\"local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret sha256-private-source-hash\""
                    .to_owned(),
            ),
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;
    let unsafe_input = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            input_json: Some(
                serde_json::json!({
                    "library_id": library_id,
                    "source_id": source.id,
                    "source_scheme": "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
                    "mode": "full",
                })
                .to_string(),
            ),
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;

    let missing_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, missing_input.id, None).await;
    let malformed_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, malformed_input.id, None)
            .await;
    let unsafe_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, unsafe_input.id, None).await;

    assert_eq!(
        missing_message,
        "invalid input: source fingerprint hash job input is missing"
    );
    assert_eq!(
        malformed_message,
        "invalid input: source fingerprint hash job input is invalid"
    );
    assert!(unsafe_message.contains(
        "source fingerprint hash job source scheme must contain only scheme-safe ASCII characters"
    ));
}

#[tokio::test]
async fn source_fingerprint_hash_retry_rejects_binding_mismatch_without_leak() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
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
    let library_mismatch = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            library_id: Some(other_library_id),
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;
    let source_mismatch = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            source_id: Some(other_source.id),
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;
    let missing_library_binding = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            library_id: None,
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;
    let missing_source_binding = fail_source_hash_retry_source_job(
        &store,
        NewJob {
            source_id: None,
            ..new_source_hash_job(library_id, source.id)
        },
    )
    .await;

    let library_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, library_mismatch.id, None)
            .await;
    let source_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, source_mismatch.id, None)
            .await;
    let missing_library_message = retry_source_hash_job_expect_err_without_retry(
        &app,
        &store,
        missing_library_binding.id,
        None,
    )
    .await;
    let missing_source_message = retry_source_hash_job_expect_err_without_retry(
        &app,
        &store,
        missing_source_binding.id,
        None,
    )
    .await;

    assert_eq!(
        library_message,
        "invalid input: source fingerprint hash job library binding does not match input"
    );
    assert_eq!(
        source_message,
        "invalid input: source fingerprint hash job source binding does not match input"
    );
    assert_eq!(
        missing_library_message,
        "invalid input: source fingerprint hash job library binding does not match input"
    );
    assert_eq!(
        missing_source_message,
        "invalid input: source fingerprint hash job source binding does not match input"
    );
    assert!(!missing_library_message.contains("local:///"));
    assert!(!missing_source_message.contains("local:///"));
}

#[tokio::test]
async fn source_fingerprint_hash_retry_rejects_source_drift_without_leak() {
    let library_id = LibraryId::new();
    let other_library_id = LibraryId::new();
    let (_temp, app, store, mut source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    store
        .upsert_library(&Library {
            id: other_library_id,
            name: "Other".to_owned(),
            roots: vec!["webdav:///Other".to_owned()],
            options: LibraryOptions::from_preset(nako_core::LibraryPreset::Movies),
        })
        .await
        .unwrap();
    let library_drift =
        fail_source_hash_retry_source_job(&store, new_source_hash_job(library_id, source.id)).await;
    source.library_id = other_library_id;
    store.upsert_media_source(&source).await.unwrap();

    let library_drift_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, library_drift.id, None).await;

    assert_eq!(
        library_drift_message,
        "conflict: source fingerprint hash retry source no longer belongs to input library"
    );
}

#[tokio::test]
async fn source_fingerprint_hash_retry_rejects_locator_drift_without_leak() {
    let library_id = LibraryId::new();
    let (_temp, app, store, mut source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let malformed_locator =
        fail_source_hash_retry_source_job(&store, new_source_hash_job(library_id, source.id)).await;
    source.locator = "Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret".to_owned();
    store.upsert_media_source(&source).await.unwrap();
    let malformed_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, malformed_locator.id, None)
            .await;

    source.locator =
        "webdav:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret".to_owned();
    store.upsert_media_source(&source).await.unwrap();
    let scheme_drift =
        fail_source_hash_retry_source_job(&store, new_source_hash_job(library_id, source.id)).await;
    let scheme_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, scheme_drift.id, None).await;

    assert_eq!(
        malformed_message,
        "invalid input: source fingerprint hash job source locator is not a valid storage URI"
    );
    assert_eq!(
        scheme_message,
        "conflict: source fingerprint hash retry source locator scheme changed since enqueue"
    );
}

#[tokio::test]
async fn source_fingerprint_hash_retry_rejects_durable_retry_invalid_input_without_leak() {
    let library_id = LibraryId::new();
    let (_temp, app, store, source) = source_hash_app_with_source(
        library_id,
        "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
    )
    .await;
    let exhausted =
        fail_source_hash_retry_source_job(&store, new_source_hash_job(library_id, source.id)).await;
    let invalid_max_attempts =
        fail_source_hash_retry_source_job(&store, new_source_hash_job(library_id, source.id)).await;
    let invalid_next_attempt_at =
        fail_source_hash_retry_source_job(&store, new_source_hash_job(library_id, source.id)).await;

    let exhausted_message =
        retry_source_hash_job_expect_err_without_retry(&app, &store, exhausted.id, Some(1)).await;
    let invalid_max_attempts_message = retry_source_hash_job_expect_err_without_retry(
        &app,
        &store,
        invalid_max_attempts.id,
        Some(0),
    )
    .await;
    let invalid_next_attempt_at_message = retry_source_hash_job_request_expect_err_without_retry(
        &app,
        &store,
        RetrySourceFingerprintHashRequest {
            job_id: invalid_next_attempt_at.id,
            max_attempts: Some(3),
            next_attempt_at: Some(
                "local:///Users/Frankorz/Secret Path/not-a-time?token=secret".to_owned(),
            ),
        },
    )
    .await;

    assert_eq!(
        exhausted_message,
        "conflict: job retry attempts are exhausted"
    );
    assert_eq!(
        invalid_max_attempts_message,
        "invalid input: retry max_attempts must be greater than zero"
    );
    assert_eq!(
        invalid_next_attempt_at_message,
        "invalid input: source fingerprint hash retry next_attempt_at must be an RFC3339 timestamp"
    );
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
    let trace_context = DurableJobTraceContext::from_request_id("REQ-PREPARE_123.Trace").unwrap();

    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: source.id,
                mode: SourceFingerprintHashMode::Partial {
                    prefix_bytes: 65_536,
                },
                priority: None,
            },
            Some(&trace_context),
        )
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
    assert_eq!(
        prepared
            .trace_context
            .as_ref()
            .map(DurableJobTraceContext::request_id),
        Some("req-prepare_123.trace")
    );
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
    let trace_context = DurableJobTraceContext::from_request_id("REQ-EXECUTE_123.Trace").unwrap();
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: source.id,
                mode: SourceFingerprintHashMode::Full,
                priority: None,
            },
            Some(&trace_context),
        )
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
    let trace_context = DurableJobTraceContext::from_request_id("REQ-SCHEDULE_456.Trace").unwrap();
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: source.id,
                mode: SourceFingerprintHashMode::Full,
                priority: None,
            },
            Some(&trace_context),
        )
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
async fn source_fingerprint_hash_completion_feeds_duplicate_reconciliation_plan() {
    let library_id = LibraryId::new();
    let (temp, app, store, target) =
        source_hash_app_with_source(library_id, "local:///target-hash.mkv", None).await;
    let duplicate_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Duplicate Hidden Movie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let duplicate = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: duplicate_item.id,
        locator: "local:///duplicate-hash.mkv".to_owned(),
        file_name: "Duplicate Hidden Movie.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: None,
    };
    store.upsert_media_item(&duplicate_item).await.unwrap();
    store.upsert_media_source(&duplicate).await.unwrap();
    fs::write(temp.path().join("target-hash.mkv"), b"same-media-bytes").unwrap();
    fs::write(temp.path().join("duplicate-hash.mkv"), b"same-media-bytes").unwrap();

    let target_job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: target.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap();
    let duplicate_job = app
        .source_hash()
        .enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest {
            library_id,
            source_id: duplicate.id,
            mode: SourceFingerprintHashMode::Full,
            priority: None,
        })
        .await
        .unwrap();

    app.source_hash()
        .execute_source_fingerprint_hash_job(target_job.id)
        .await
        .unwrap();
    app.source_hash()
        .execute_source_fingerprint_hash_job(duplicate_job.id)
        .await
        .unwrap();

    let target = store.get_media_source(target.id).await.unwrap().unwrap();
    let duplicate = store.get_media_source(duplicate.id).await.unwrap().unwrap();
    let target_fingerprint = target.fingerprint.as_deref().expect("target fingerprint");
    let duplicate_fingerprint = duplicate
        .fingerprint
        .as_deref()
        .expect("duplicate fingerprint");
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

    assert_eq!(target_fingerprint, duplicate_fingerprint);
    assert!(target_fingerprint.starts_with("source:v1:content_hash:sha256:"));
    assert!(before.is_empty());
    assert!(after.is_empty());
    assert_eq!(plan.library_id, library_id);
    assert_eq!(plan.source_id, target.id);
    assert_eq!(
        plan.fingerprint_evidence_kind,
        SourceFingerprintEvidenceKind::ContentHash
    );
    assert_eq!(plan.confidence_milli, 1_000);
    assert!(!plan.stale);
    assert_eq!(plan.candidates.len(), 1);
    let candidate = &plan.candidates[0];
    assert_eq!(candidate.source_id, target.id);
    assert_eq!(candidate.duplicate_source_id, duplicate.id);
    assert_eq!(
        candidate.evidence_kind,
        nako_core::SourceDuplicateEvidenceKind::StrongFingerprint
    );
    assert_eq!(candidate.confidence_milli, Some(1_000));
    assert!(!candidate.stale);
    assert_eq!(candidate.relationship_id, None);
    assert_eq!(candidate.existing_status, None);
    assert_eq!(
        candidate.recommended_action,
        SourceDuplicateReconciliationAction::SuggestRelationship
    );
    assert!(!plan_json.contains(target_fingerprint));
    assert!(!plan_json.contains("same-media-bytes"));
    assert!(!plan_json.contains("target-hash"));
    assert!(!plan_json.contains("duplicate-hash"));
    assert!(!plan_json.contains("local:///"));
    assert!(!plan_json.contains("Hidden Movie"));
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
    let unsafe_trace_context_input = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source.id),
            input_json: Some(
                serde_json::json!({
                    "library_id": library_id,
                    "source_id": source.id,
                    "source_scheme": "local",
                    "mode": "full",
                    "request_id": "https://secret.example/path?token=private",
                })
                .to_string(),
            ),
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
    let unsafe_trace_context_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&unsafe_trace_context_input)
        .await
        .unwrap_err();
    let unsafe_message = unsafe_err.to_string();
    let unsafe_trace_context_message = unsafe_trace_context_err.to_string();

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
    assert_eq!(
        unsafe_trace_context_message,
        "invalid input: invalid durable job trace request_id"
    );
    assert!(!unsafe_trace_context_message.contains("secret.example"));
    assert!(!unsafe_trace_context_message.contains("token"));
    assert!(!unsafe_trace_context_message.contains("private"));
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
    let missing_library_binding = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: None,
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
    let missing_source_binding = store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::SourceFingerprintHash,
            resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: None,
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
    let missing_library_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&missing_library_binding)
        .await
        .unwrap_err();
    let missing_source_err = app
        .source_hash()
        .prepare_source_fingerprint_hash_execution(&missing_source_binding)
        .await
        .unwrap_err();
    let library_message = library_err.to_string();
    let source_message = source_err.to_string();
    let missing_library_message = missing_library_err.to_string();
    let missing_source_message = missing_source_err.to_string();

    assert_eq!(
        library_message,
        "invalid input: source fingerprint hash job library binding does not match input"
    );
    assert_eq!(
        source_message,
        "invalid input: source fingerprint hash job source binding does not match input"
    );
    assert_eq!(
        missing_library_message,
        "invalid input: source fingerprint hash job library binding does not match input"
    );
    assert_eq!(
        missing_source_message,
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
    assert!(!missing_library_message.contains("Hidden Movie"));
    assert!(!missing_library_message.contains("local:///"));
    assert!(!missing_source_message.contains("Hidden Movie"));
    assert!(!missing_source_message.contains("local:///"));
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

async fn fail_source_hash_retry_source_job(store: &NakoDatabase, job: NewJob) -> nako_core::Job {
    let job = store.enqueue_job(job).await.unwrap();
    store.start_job(job.id).await.unwrap();
    store
        .fail_job(
            job.id,
            "source hash failed for local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret sha256-private-source-hash input_json private-fingerprint".to_owned(),
        )
        .await
        .unwrap()
}

async fn retry_source_hash_job_expect_err_without_retry(
    app: &NakoApp,
    store: &NakoDatabase,
    job_id: JobId,
    max_attempts: Option<u32>,
) -> String {
    retry_source_hash_job_request_expect_err_without_retry(
        app,
        store,
        RetrySourceFingerprintHashRequest {
            job_id,
            max_attempts,
            next_attempt_at: None,
        },
    )
    .await
}

async fn retry_source_hash_job_request_expect_err_without_retry(
    app: &NakoApp,
    store: &NakoDatabase,
    request: RetrySourceFingerprintHashRequest,
) -> String {
    let job_id = request.job_id;
    let err = app
        .source_hash()
        .retry_source_fingerprint_hash_job(request)
        .await
        .unwrap_err();
    let message = err.to_string();
    let jobs = store
        .list_jobs(Default::default(), PageRequest::first_page())
        .await
        .unwrap();

    assert!(
        jobs.iter().all(|job| job.retry_of_job_id != Some(job_id)),
        "retry error must not create retry job: {message}"
    );
    assert_source_hash_retry_error_redacted(&message);

    message
}

fn assert_source_hash_retry_error_redacted(message: &str) {
    for forbidden in [
        "Hidden Movie",
        "Secret Path",
        "Frankorz",
        "token",
        "local:///",
        "webdav:///",
        "sha256:",
        "private-source-hash",
        "private-fingerprint",
        "aaaaaaaaaaaaaaaa",
        "input_json",
    ] {
        assert!(
            !message.contains(forbidden),
            "retry error leaked {forbidden:?}: {message}"
        );
    }
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

fn scan_source_hash_trigger(
    source_id: MediaSourceId,
    action: SourceFingerprintEscalationAction,
    mode: Option<SourceFingerprintHashMode>,
) -> ScanSourceFingerprintHashTrigger {
    ScanSourceFingerprintHashTrigger {
        source_id,
        decision: SourceFingerprintEscalationDecision {
            action,
            reason: match action {
                SourceFingerprintEscalationAction::None => {
                    SourceFingerprintEscalationReason::NoAmbiguousCandidate
                }
                SourceFingerprintEscalationAction::PartialHash => {
                    SourceFingerprintEscalationReason::ConfirmSingleWeakCandidate
                }
                SourceFingerprintEscalationAction::FullHash => {
                    SourceFingerprintEscalationReason::DisambiguateMultipleCandidates
                }
            },
            evidence_kind: SourceFingerprintEvidenceKind::BackendFingerprint,
            confidence_milli: 700,
            stale: false,
            candidate_count: if action == SourceFingerprintEscalationAction::None {
                0
            } else {
                1
            },
        },
        mode,
    }
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
        vfs_cache_repair_automation: crate::config::VfsCacheRepairAutomationRuntimeConfig::default(
        ),
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
