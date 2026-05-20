use std::{future::Future, sync::OnceLock};

use taru_core::{
    CancelLeasedJob, CompleteLeasedJob, DatabaseLifecycle, FailLeasedJob, Job, JobId, JobKind,
    JobLeaseClaimFilter, JobLeaseClaimRequest, JobLeaseGuard, JobLeaseHeartbeat,
    JobLeaseRepository, JobRepository, JobRunToken, JobStatus, JobWorkerId, Library, LibraryId,
    LibraryOptions, LibraryPreset, LibraryRepository, NewJob, RecoverExpiredJobLeases,
    RequestJobCancellation, TaruError,
};

use crate::{TaruDatabase, postgres::PostgresStore};

const TARU_TEST_POSTGRES_URL: &str = "TARU_TEST_POSTGRES_URL";

static POSTGRES_CONTRACT_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

trait JobLeaseContractBackend:
    DatabaseLifecycle + JobRepository + JobLeaseRepository + LibraryRepository + Send + Sync
{
}

impl<T> JobLeaseContractBackend for T where
    T: DatabaseLifecycle + JobRepository + JobLeaseRepository + LibraryRepository + Send + Sync
{
}

async fn migrated_sqlite_database() -> TaruDatabase {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store
}

async fn postgres_contract_database() -> Option<PostgresStore> {
    let Ok(database_url) = std::env::var(TARU_TEST_POSTGRES_URL) else {
        return None;
    };
    let schema_name = format!(
        "taru_contract_{}",
        JobRunToken::new().to_string().replace('-', "_")
    );
    let store = PostgresStore::connect_with_schema(&database_url, &schema_name)
        .await
        .unwrap();
    store.migrate().await.unwrap();
    Some(store)
}

async fn run_sqlite_job_lease_contract<F, Fut>(contract: F)
where
    F: FnOnce(TaruDatabase) -> Fut,
    Fut: Future<Output = ()>,
{
    contract(migrated_sqlite_database().await).await;
}

async fn run_postgres_job_lease_contract<F, Fut>(contract: F)
where
    F: FnOnce(PostgresStore) -> Fut,
    Fut: Future<Output = ()>,
{
    if std::env::var(TARU_TEST_POSTGRES_URL).is_err() {
        eprintln!("skipping PostgreSQL contract test because {TARU_TEST_POSTGRES_URL} is not set");
        return;
    }

    let lock = POSTGRES_CONTRACT_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = lock.lock().await;
    let store = postgres_contract_database()
        .await
        .expect("PostgreSQL URL should be present after opt-in check");

    contract(store.clone()).await;
    store.drop_schema().await.unwrap();
}

async fn seed_contract_library<S>(store: &S) -> Library
where
    S: LibraryRepository + ?Sized,
{
    let library = Library {
        id: LibraryId::new(),
        name: "Contract Movies".to_owned(),
        roots: vec!["local:///Contract Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };

    store.upsert_library(&library).await.unwrap();
    library
}

async fn enqueue_contract_job<S>(
    store: &S,
    kind: JobKind,
    resource_class: &str,
    library_id: Option<LibraryId>,
    input_json: Option<&str>,
) -> Job
where
    S: JobRepository + ?Sized,
{
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind,
            resource_class: resource_class.to_owned(),
            library_id,
            source_id: None,
            input_json: input_json.map(str::to_owned),
        })
        .await
        .unwrap()
}

async fn claim_next<S>(
    store: &S,
    worker_id: JobWorkerId,
    filter: JobLeaseClaimFilter,
) -> Option<taru_core::LeasedJob>
where
    S: JobLeaseRepository + ?Sized,
{
    store
        .claim_next_job_lease(JobLeaseClaimRequest {
            worker_id,
            lease_duration_ms: 30_000,
            filter,
        })
        .await
        .unwrap()
}

async fn claim_next_job_lease_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let library = seed_contract_library(&store).await;
    let skipped = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        Some(library.id),
        None,
    )
    .await;
    let target = enqueue_contract_job(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        Some(library.id),
        Some(r#"{"library_id":"movies"}"#),
    )
    .await;
    let decoy = enqueue_contract_job(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        Some(library.id),
        Some(r#"{"library_id":"movies","slot":"decoy"}"#),
    )
    .await;

    let worker_id = JobWorkerId::new();
    let exact_claim = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            job_id: Some(target.id),
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            library_id: Some(library.id),
            source_id: None,
        },
    )
    .await
    .expect("target library scan job should be claimable by id");

    assert_eq!(exact_claim.job.id, target.id);
    assert_eq!(exact_claim.job.input_json, target.input_json);
    assert_eq!(exact_claim.lease.job_id, target.id);
    assert_eq!(exact_claim.lease.worker_id, worker_id);
    assert_eq!(exact_claim.lease.cancel_requested_at, None);
    assert_eq!(exact_claim.lease.cancel_reason, None);

    let claimed = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            job_id: None,
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            library_id: Some(library.id),
            source_id: None,
        },
    )
    .await
    .expect("remaining library scan job should be claimable");

    assert_eq!(claimed.job.id, decoy.id);
    assert_eq!(claimed.job.status, JobStatus::Running);
    assert_eq!(claimed.job.input_json, decoy.input_json);
    assert!(claimed.job.started_at.is_some());
    assert_eq!(claimed.job.completed_at, None);
    assert_eq!(claimed.job.error, None);
    assert_eq!(claimed.lease.job_id, decoy.id);
    assert_eq!(claimed.lease.worker_id, worker_id);
    assert_eq!(claimed.lease.cancel_requested_at, None);
    assert_eq!(claimed.lease.cancel_reason, None);
    assert!(!claimed.lease.heartbeat_at.is_empty());
    assert!(!claimed.lease.lease_expires_at.is_empty());

    let loaded = store.get_job(target.id).await.unwrap().unwrap();
    let decoy_loaded = store.get_job(decoy.id).await.unwrap().unwrap();
    assert_eq!(loaded.status, JobStatus::Running);
    assert_eq!(decoy_loaded.status, JobStatus::Running);
    assert_eq!(
        store.get_job(skipped.id).await.unwrap().unwrap().status,
        JobStatus::Queued
    );
    assert!(
        claim_next(
            &store,
            worker_id,
            JobLeaseClaimFilter {
                job_id: None,
                kind: Some(JobKind::LibraryScan),
                resource_class: Some("disk.scan".to_owned()),
                library_id: Some(library.id),
                source_id: None,
            },
        )
        .await
        .is_none()
    );
}

async fn job_lease_run_token_fence_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let job = enqueue_contract_job(&store, JobKind::LibraryScan, "disk.scan", None, None).await;
    let claimed = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("queued library scan job should be claimable");
    assert_eq!(claimed.job.id, job.id);

    let stale_guard = JobLeaseGuard {
        job_id: claimed.job.id,
        run_token: JobRunToken::new(),
    };
    let stale_heartbeat = store
        .heartbeat_job_lease(JobLeaseHeartbeat {
            guard: stale_guard,
            lease_duration_ms: 10_000,
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_heartbeat, TaruError::Conflict { .. }));

    let heartbeat = store
        .heartbeat_job_lease(JobLeaseHeartbeat {
            guard: claimed.lease.guard(),
            lease_duration_ms: 20_000,
        })
        .await
        .unwrap();
    assert_eq!(heartbeat.job.id, claimed.job.id);
    assert_eq!(heartbeat.job.status, JobStatus::Running);
    assert_eq!(heartbeat.lease.run_token, claimed.lease.run_token);
    assert_ne!(
        heartbeat.lease.lease_expires_at,
        claimed.lease.lease_expires_at
    );

    let stale_success = store
        .succeed_leased_job(CompleteLeasedJob {
            guard: stale_guard,
            summary_json: Some(r#"{"ignored":true}"#.to_owned()),
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_success, TaruError::Conflict { .. }));
    assert_eq!(
        store.get_job(job.id).await.unwrap().unwrap().status,
        JobStatus::Running
    );

    let succeeded = store
        .succeed_leased_job(CompleteLeasedJob {
            guard: claimed.lease.guard(),
            summary_json: Some(r#"{"done":true}"#.to_owned()),
        })
        .await
        .unwrap();
    assert_eq!(succeeded.status, JobStatus::Succeeded);
    assert_eq!(succeeded.summary_json, Some(r#"{"done":true}"#.to_owned()));
    assert_eq!(succeeded.error, None);
    assert!(succeeded.completed_at.is_some());

    let stale_failure = store
        .fail_leased_job(FailLeasedJob {
            guard: claimed.lease.guard(),
            error: "too late".to_owned(),
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_failure, TaruError::Conflict { .. }));
}

async fn job_cancellation_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let queued = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        None,
        None,
    )
    .await;
    let queued_cancel = store
        .request_job_cancellation(RequestJobCancellation {
            job_id: queued.id,
            reason: Some("operator request".to_owned()),
        })
        .await
        .unwrap();
    assert!(queued_cancel.requested);
    assert!(queued_cancel.terminal);
    assert_eq!(queued_cancel.job.status, JobStatus::Cancelled);
    assert_eq!(queued_cancel.job.error, None);
    assert!(queued_cancel.job.completed_at.is_some());
    assert!(queued_cancel.cancel_requested_at.is_some());

    let running = enqueue_contract_job(&store, JobKind::LibraryScan, "disk.scan", None, None).await;
    let claimed = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("queued running job should be claimable");
    assert_eq!(claimed.job.id, running.id);

    let running_cancel = store
        .request_job_cancellation(RequestJobCancellation {
            job_id: running.id,
            reason: Some("operator stop".to_owned()),
        })
        .await
        .unwrap();
    assert!(running_cancel.requested);
    assert!(!running_cancel.terminal);
    assert_eq!(running_cancel.job.status, JobStatus::Running);
    assert!(running_cancel.cancel_requested_at.is_some());

    let refreshed = store
        .heartbeat_job_lease(JobLeaseHeartbeat {
            guard: claimed.lease.guard(),
            lease_duration_ms: 10_000,
        })
        .await
        .unwrap();
    assert!(refreshed.lease.cancel_requested_at.is_some());
    assert_eq!(
        refreshed.lease.cancel_reason.as_deref(),
        Some("operator stop")
    );

    let stale_cancel = store
        .cancel_leased_job(CancelLeasedJob {
            guard: JobLeaseGuard {
                job_id: running.id,
                run_token: JobRunToken::new(),
            },
            summary_json: Some(r#"{"ignored":true}"#.to_owned()),
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_cancel, TaruError::Conflict { .. }));

    let cancelled = store
        .cancel_leased_job(CancelLeasedJob {
            guard: claimed.lease.guard(),
            summary_json: Some(r#"{"cancelled":true}"#.to_owned()),
        })
        .await
        .unwrap();
    assert_eq!(cancelled.status, JobStatus::Cancelled);
    assert_eq!(
        cancelled.summary_json,
        Some(r#"{"cancelled":true}"#.to_owned())
    );
    assert_eq!(cancelled.error, None);
    assert!(cancelled.completed_at.is_some());

    let terminal_cancel = store
        .request_job_cancellation(RequestJobCancellation {
            job_id: cancelled.id,
            reason: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(terminal_cancel, TaruError::Conflict { .. }));
}

async fn recover_expired_job_leases_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let queued = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        None,
        None,
    )
    .await;
    let running = enqueue_contract_job(&store, JobKind::LibraryScan, "disk.scan", None, None).await;
    let active =
        enqueue_contract_job(&store, JobKind::LibraryProbe, "media.probe", None, None).await;

    let expired_claim = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("library scan job should be claimable");
    let active_claim = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryProbe),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("library probe job should be claimable");
    assert_eq!(expired_claim.job.id, running.id);
    assert_eq!(active_claim.job.id, active.id);

    let exact_recovery = store
        .recover_expired_job_leases(RecoverExpiredJobLeases {
            filter: JobLeaseClaimFilter {
                job_id: Some(running.id),
                ..JobLeaseClaimFilter::default()
            },
            expired_before: "9999-01-01T00:00:00.000Z".to_owned(),
            error: "lease expired during startup recovery".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(exact_recovery, 1);

    let running = store.get_job(running.id).await.unwrap().unwrap();
    let active = store.get_job(active.id).await.unwrap().unwrap();
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(active.status, JobStatus::Running);

    let recovered = store
        .recover_expired_job_leases(RecoverExpiredJobLeases {
            filter: JobLeaseClaimFilter::default(),
            expired_before: "9999-01-01T00:00:00.000Z".to_owned(),
            error: "lease expired during startup recovery".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(recovered, 1);

    let queued = store.get_job(queued.id).await.unwrap().unwrap();
    let running = store.get_job(running.id).await.unwrap().unwrap();
    let active = store.get_job(active.id).await.unwrap().unwrap();

    assert_eq!(queued.status, JobStatus::Queued);
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(
        running.error.as_deref(),
        Some("lease expired during startup recovery")
    );
    assert_eq!(active.status, JobStatus::Failed);
    assert!(running.completed_at.is_some());
    assert!(active.completed_at.is_some());
}

#[tokio::test]
async fn sqlite_job_lease_contract_claims_next_with_worker_token_and_filter() {
    run_sqlite_job_lease_contract(claim_next_job_lease_contract).await;
}

#[tokio::test]
async fn sqlite_job_lease_contract_heartbeats_and_completes_with_run_token_fence() {
    run_sqlite_job_lease_contract(job_lease_run_token_fence_contract).await;
}

#[tokio::test]
async fn sqlite_job_lease_contract_cancel_requests_are_durable_and_acknowledged_by_owner() {
    run_sqlite_job_lease_contract(job_cancellation_contract).await;
}

#[tokio::test]
async fn sqlite_job_lease_contract_recovers_only_expired_running_leases() {
    run_sqlite_job_lease_contract(recover_expired_job_leases_contract).await;
}

#[tokio::test]
#[ignore = "requires TARU_TEST_POSTGRES_URL"]
async fn postgres_job_lease_contract_claims_next_with_worker_token_and_filter() {
    run_postgres_job_lease_contract(claim_next_job_lease_contract).await;
}

#[tokio::test]
#[ignore = "requires TARU_TEST_POSTGRES_URL"]
async fn postgres_job_lease_contract_heartbeats_and_completes_with_run_token_fence() {
    run_postgres_job_lease_contract(job_lease_run_token_fence_contract).await;
}

#[tokio::test]
#[ignore = "requires TARU_TEST_POSTGRES_URL"]
async fn postgres_job_lease_contract_cancel_requests_are_durable_and_acknowledged_by_owner() {
    run_postgres_job_lease_contract(job_cancellation_contract).await;
}

#[tokio::test]
#[ignore = "requires TARU_TEST_POSTGRES_URL"]
async fn postgres_job_lease_contract_recovers_only_expired_running_leases() {
    run_postgres_job_lease_contract(recover_expired_job_leases_contract).await;
}
