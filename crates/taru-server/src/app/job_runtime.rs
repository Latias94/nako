use std::{
    future::Future,
    sync::{Arc, Mutex, OnceLock},
};

use serde::Serialize;
use taru_core::{
    CancelLeasedJob, CompleteLeasedJob, FailLeasedJob, Job, JobId, JobLeaseClaimFilter,
    JobLeaseClaimRequest, JobLeaseGuard, JobLeaseHeartbeat, JobRepository, JobWorkerId, Result,
    TaruError,
};
use taru_db::SqliteStore;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

const DEFAULT_LEASE_DURATION_MS: u64 = 30_000;
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 10_000;

#[derive(Clone, Debug)]
pub(super) struct DurableJobRuntime {
    store: SqliteStore,
    worker_id: JobWorkerId,
    lease_duration_ms: u64,
    heartbeat_interval_ms: u64,
}

#[derive(Debug)]
pub(super) struct DurableJobRun<T> {
    pub(super) job: Job,
    pub(super) output: T,
}

#[derive(Clone, Debug)]
pub(super) struct DurableJobContext {
    store: SqliteStore,
    guard: JobLeaseGuard,
    lease_duration_ms: u64,
    cancellation: DurableJobCancellation,
}

#[derive(Debug)]
pub(super) enum DurableJobRunOutcome<T> {
    Completed(DurableJobRun<T>),
    Cancelled(Job),
}

#[derive(Debug)]
pub(super) enum DurableJobOperationError {
    Cancelled,
    Failed(TaruError),
}

pub(super) type DurableJobOperationResult<T> = std::result::Result<T, DurableJobOperationError>;

impl DurableJobContext {
    pub(super) fn is_cancel_requested(&self) -> bool {
        self.cancellation.is_requested()
    }

    pub(super) async fn cancelled(&self) {
        self.cancellation.cancelled().await;
    }

    pub(super) fn cancel_requested_at(&self) -> Option<String> {
        self.cancellation.requested_at()
    }

    pub(super) async fn check_cancelled(&self) -> DurableJobOperationResult<()> {
        self.refresh_cancellation().await?;
        if self.is_cancel_requested() {
            Err(DurableJobOperationError::Cancelled)
        } else {
            Ok(())
        }
    }

    async fn refresh_cancellation(&self) -> DurableJobOperationResult<()> {
        let leased = self
            .store
            .heartbeat_job_lease(JobLeaseHeartbeat {
                guard: self.guard,
                lease_duration_ms: self.lease_duration_ms,
            })
            .await
            .map_err(DurableJobOperationError::Failed)?;
        self.cancellation.observe_lease(&leased.lease);
        Ok(())
    }
}

impl From<TaruError> for DurableJobOperationError {
    fn from(value: TaruError) -> Self {
        Self::Failed(value)
    }
}

impl<T> DurableJobRunOutcome<T> {
    fn into_completed(self) -> Result<DurableJobRun<T>> {
        match self {
            Self::Completed(run) => Ok(run),
            Self::Cancelled(job) => Err(TaruError::Conflict {
                message: format!("job {} was cancelled", job.id),
            }),
        }
    }
}

impl DurableJobRuntime {
    pub(super) fn new(store: SqliteStore) -> Self {
        Self {
            store,
            worker_id: default_worker_id(),
            lease_duration_ms: DEFAULT_LEASE_DURATION_MS,
            heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
        }
    }

    #[cfg(test)]
    pub(super) fn with_lease_timing(
        store: SqliteStore,
        lease_duration_ms: u64,
        heartbeat_interval_ms: u64,
    ) -> Self {
        Self {
            store,
            worker_id: JobWorkerId::new(),
            lease_duration_ms,
            heartbeat_interval_ms,
        }
    }

    pub(super) async fn run_job<T, Run, RunFuture, Summary>(
        &self,
        job_id: JobId,
        operation: &'static str,
        run: Run,
        summary_json: Summary,
    ) -> Result<DurableJobRun<T>>
    where
        Run: FnOnce() -> RunFuture,
        RunFuture: Future<Output = Result<T>>,
        Summary: FnOnce(&T) -> Result<Option<String>>,
    {
        self.run_job_with_context(
            job_id,
            operation,
            |_context| async move { run().await.map_err(DurableJobOperationError::from) },
            summary_json,
        )
        .await?
        .into_completed()
    }

    pub(super) async fn run_job_with_context<T, Run, RunFuture, Summary>(
        &self,
        job_id: JobId,
        operation: &'static str,
        run: Run,
        summary_json: Summary,
    ) -> Result<DurableJobRunOutcome<T>>
    where
        Run: FnOnce(DurableJobContext) -> RunFuture,
        RunFuture: Future<Output = DurableJobOperationResult<T>>,
        Summary: FnOnce(&T) -> Result<Option<String>>,
    {
        let leased = self
            .store
            .claim_next_job_lease(JobLeaseClaimRequest {
                worker_id: self.worker_id,
                lease_duration_ms: self.lease_duration_ms,
                filter: JobLeaseClaimFilter {
                    job_id: Some(job_id),
                    ..JobLeaseClaimFilter::default()
                },
            })
            .await?
            .ok_or_else(|| TaruError::Conflict {
                message: format!("job {job_id} is not queued and claimable"),
            })?;
        let guard = leased.lease.guard();
        let cancellation = DurableJobCancellation::new();
        cancellation.observe_lease(&leased.lease);
        let heartbeat = self.start_heartbeat(operation, guard, cancellation.clone());
        let context = DurableJobContext {
            store: self.store.clone(),
            guard,
            lease_duration_ms: self.lease_duration_ms,
            cancellation,
        };

        let output = match run(context).await {
            Ok(output) => output,
            Err(DurableJobOperationError::Cancelled) => {
                heartbeat.stop().await;
                let job = self.cancel_job(guard, operation).await?;
                return Ok(DurableJobRunOutcome::Cancelled(job));
            }
            Err(DurableJobOperationError::Failed(err)) => {
                heartbeat.stop().await;
                self.fail_job(guard, operation, &err).await;
                return Err(err);
            }
        };

        let summary_json = match summary_json(&output) {
            Ok(summary_json) => summary_json,
            Err(err) => {
                heartbeat.stop().await;
                self.fail_job(guard, operation, &err).await;
                return Err(err);
            }
        };

        heartbeat.stop().await;
        let job = self
            .store
            .succeed_leased_job(CompleteLeasedJob {
                guard,
                summary_json,
            })
            .await?;
        Ok(DurableJobRunOutcome::Completed(DurableJobRun {
            job,
            output,
        }))
    }

    async fn fail_job(&self, guard: JobLeaseGuard, operation: &'static str, err: &TaruError) {
        if let Err(update_err) = self
            .store
            .fail_leased_job(FailLeasedJob {
                guard,
                error: err.to_string(),
            })
            .await
        {
            warn!(
                job_id = %guard.job_id,
                operation,
                error = %update_err,
                "failed to persist failed durable job state"
            );
        }
    }

    async fn cancel_job(&self, guard: JobLeaseGuard, operation: &'static str) -> Result<Job> {
        let job = self
            .store
            .cancel_leased_job(CancelLeasedJob {
                guard,
                summary_json: None,
            })
            .await?;
        debug!(
            job_id = %guard.job_id,
            operation,
            "durable job cancellation acknowledged"
        );
        Ok(job)
    }

    pub(super) fn serialize_summary<T>(summary: &T, description: &str) -> Result<Option<String>>
    where
        T: Serialize,
    {
        serde_json::to_string(summary)
            .map(Some)
            .map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize {description}: {err}"),
            })
    }

    fn start_heartbeat(
        &self,
        operation: &'static str,
        guard: JobLeaseGuard,
        cancellation: DurableJobCancellation,
    ) -> DurableJobHeartbeat {
        let (stop_tx, mut stop_rx) = oneshot::channel();
        let store = self.store.clone();
        let lease_duration_ms = self.lease_duration_ms;
        let heartbeat_interval_ms = self.heartbeat_interval_ms.max(1);
        let handle = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(heartbeat_interval_ms));
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = interval.tick() => {
                        match store.heartbeat_job_lease(JobLeaseHeartbeat {
                            guard,
                            lease_duration_ms,
                        }).await {
                            Ok(leased) => {
                                cancellation.observe_lease(&leased.lease);
                                debug!(
                                    job_id = %guard.job_id,
                                    operation,
                                    "durable job lease heartbeat persisted"
                                );
                            }
                            Err(err) => {
                                warn!(
                                    job_id = %guard.job_id,
                                    operation,
                                    error = %err,
                                    "durable job lease heartbeat failed"
                                );
                                break;
                            }
                        }
                    }
                }
            }
        });

        DurableJobHeartbeat {
            stop_tx: Some(stop_tx),
            handle,
        }
    }
}

#[derive(Clone, Debug)]
struct DurableJobCancellation {
    token: CancellationToken,
    state: Arc<Mutex<DurableJobCancellationState>>,
}

#[derive(Debug, Default)]
struct DurableJobCancellationState {
    requested_at: Option<String>,
}

impl DurableJobCancellation {
    fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            state: Arc::new(Mutex::new(DurableJobCancellationState::default())),
        }
    }

    fn observe_lease(&self, lease: &taru_core::JobLeaseRecord) {
        if let Some(requested_at) = lease.cancel_requested_at.as_ref() {
            {
                let mut state = self
                    .state
                    .lock()
                    .expect("durable job cancellation state poisoned");
                if state.requested_at.is_none() {
                    state.requested_at = Some(requested_at.clone());
                }
            }
            self.token.cancel();
        }
    }

    fn is_requested(&self) -> bool {
        self.token.is_cancelled()
    }

    async fn cancelled(&self) {
        self.token.cancelled().await;
    }

    fn requested_at(&self) -> Option<String> {
        self.state
            .lock()
            .expect("durable job cancellation state poisoned")
            .requested_at
            .clone()
    }
}

fn default_worker_id() -> JobWorkerId {
    static WORKER_ID: OnceLock<JobWorkerId> = OnceLock::new();
    *WORKER_ID.get_or_init(JobWorkerId::new)
}

struct DurableJobHeartbeat {
    stop_tx: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl DurableJobHeartbeat {
    async fn stop(mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        let _ = self.handle.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taru_core::{JobKind, JobLeaseClaimFilter, JobStatus, NewJob, TransactionManager};
    use taru_db::SqliteStore;
    use tokio::sync::Notify;

    async fn migrated_store() -> SqliteStore {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
    }

    #[tokio::test]
    async fn durable_job_runtime_persists_success_summary() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::with_lease_timing(store.clone(), 1_000, 5);
        assert_eq!(
            DurableJobRuntime::new(store.clone()).worker_id,
            DurableJobRuntime::new(store.clone()).worker_id
        );
        let other = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();
        let job = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();

        let run = runtime
            .run_job(
                job.id,
                "test success",
                || async {
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                    Ok::<_, TaruError>(1_u32)
                },
                |value| DurableJobRuntime::serialize_summary(value, "test summary"),
            )
            .await
            .unwrap();

        assert_eq!(run.output, 1);
        assert_eq!(run.job.id, job.id);
        assert_eq!(run.job.status, JobStatus::Succeeded);
        assert_eq!(run.job.summary_json.as_deref(), Some("1"));
        assert!(run.job.started_at.is_some());
        assert!(run.job.completed_at.is_some());
        assert_eq!(
            store.get_job(other.id).await.unwrap().unwrap().status,
            JobStatus::Queued
        );
        assert!(
            store
                .claim_next_job_lease(taru_core::JobLeaseClaimRequest {
                    worker_id: taru_core::JobWorkerId::new(),
                    lease_duration_ms: 10_000,
                    filter: JobLeaseClaimFilter {
                        job_id: Some(job.id),
                        ..JobLeaseClaimFilter::default()
                    },
                })
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn durable_job_runtime_persists_failure() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::with_lease_timing(store.clone(), 1_000, 5);
        let job = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: "metadata.tmdb".to_owned(),
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();

        let err = runtime
            .run_job(
                job.id,
                "test failure",
                || async {
                    Err::<u32, _>(TaruError::InvalidInput {
                        message: "planned failure".to_owned(),
                    })
                },
                |value| DurableJobRuntime::serialize_summary(value, "test summary"),
            )
            .await
            .unwrap_err();
        let loaded = store.get_job(job.id).await.unwrap().unwrap();

        assert_eq!(err.to_string(), "invalid input: planned failure");
        assert_eq!(loaded.status, JobStatus::Failed);
        assert_eq!(
            loaded.error.as_deref(),
            Some("invalid input: planned failure")
        );
        assert!(loaded.completed_at.is_some());
    }

    #[tokio::test]
    async fn durable_job_runtime_acknowledges_observed_cancellation() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::with_lease_timing(store.clone(), 1_000, 5);
        let job = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataMaintenance,
                resource_class: "metadata.maintenance".to_owned(),
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();
        let entered = Arc::new(Notify::new());
        let entered_task = entered.clone();
        let observed_at: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let observed_at_task = observed_at.clone();

        let handle = tokio::spawn(async move {
            runtime
                .run_job_with_context(
                    job.id,
                    "test cancellation",
                    move |context| async move {
                        entered_task.notify_one();
                        context.cancelled().await;
                        *observed_at_task
                            .lock()
                            .expect("test cancellation observation poisoned") =
                            context.cancel_requested_at();
                        context.check_cancelled().await?;
                        Ok(1_u32)
                    },
                    |value| DurableJobRuntime::serialize_summary(value, "test summary"),
                )
                .await
        });

        entered.notified().await;
        let requested = store
            .request_job_cancellation(taru_core::RequestJobCancellation {
                job_id: job.id,
                reason: Some("operator request".to_owned()),
            })
            .await
            .unwrap();
        assert!(requested.requested);
        assert!(!requested.terminal);

        let outcome = tokio::time::timeout(std::time::Duration::from_secs(2), handle)
            .await
            .expect("runtime should observe cancellation")
            .unwrap()
            .unwrap();
        let DurableJobRunOutcome::Cancelled(cancelled) = outcome else {
            panic!("runtime should return a cancelled outcome");
        };
        let loaded = store.get_job(job.id).await.unwrap().unwrap();

        assert_eq!(cancelled.status, JobStatus::Cancelled);
        assert_eq!(loaded.status, JobStatus::Cancelled);
        assert_eq!(loaded.summary_json, None);
        assert_eq!(loaded.error, None);
        assert!(loaded.completed_at.is_some());
        assert!(
            observed_at
                .lock()
                .expect("test cancellation observation poisoned")
                .is_some()
        );
    }

    #[tokio::test]
    async fn durable_job_runtime_persists_summary_serialization_failure() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::with_lease_timing(store.clone(), 1_000, 5);
        let job = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoExport,
                resource_class: "metadata.nfo.export".to_owned(),
                library_id: None,
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();

        let err = runtime
            .run_job(
                job.id,
                "test summary failure",
                || async { Ok::<_, TaruError>(1_u32) },
                |_value| {
                    Err(TaruError::InvalidInput {
                        message: "bad summary".to_owned(),
                    })
                },
            )
            .await
            .unwrap_err();
        let loaded = store.get_job(job.id).await.unwrap().unwrap();

        assert_eq!(err.to_string(), "invalid input: bad summary");
        assert_eq!(loaded.status, JobStatus::Failed);
        assert_eq!(loaded.error.as_deref(), Some("invalid input: bad summary"));
        assert!(loaded.completed_at.is_some());
    }
}
