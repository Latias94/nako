use std::{
    future::Future,
    sync::{Arc, Mutex, OnceLock},
};

use nako_core::{
    CancelLeasedJob, CompleteLeasedJob, FailLeasedJob, Job, JobId, JobLeaseClaimFilter,
    JobLeaseClaimRequest, JobLeaseGuard, JobLeaseHeartbeat, JobLeaseRepository, JobWorkerId,
    LeasedJob, NakoError, Result,
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

const DEFAULT_LEASE_DURATION_MS: u64 = 30_000;
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 10_000;
const MAX_TRACE_REQUEST_ID_LEN: usize = 96;
const MIN_TRACE_REQUEST_ID_LEN: usize = 8;

#[async_trait::async_trait]
pub(super) trait DurableJobLeaseStore: std::fmt::Debug + Send + Sync {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>>;

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob>;

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job>;

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job>;

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job>;
}

#[async_trait::async_trait]
impl<T> DurableJobLeaseStore for T
where
    T: JobLeaseRepository + std::fmt::Debug + Send + Sync,
{
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>> {
        JobLeaseRepository::claim_next_job_lease(self, request).await
    }

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob> {
        JobLeaseRepository::heartbeat_job_lease(self, heartbeat).await
    }

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job> {
        JobLeaseRepository::succeed_leased_job(self, completion).await
    }

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job> {
        JobLeaseRepository::fail_leased_job(self, failure).await
    }

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job> {
        JobLeaseRepository::cancel_leased_job(self, cancellation).await
    }
}

#[derive(Clone, Debug)]
pub(super) struct DurableJobRuntime {
    store: Arc<dyn DurableJobLeaseStore>,
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
    store: Arc<dyn DurableJobLeaseStore>,
    guard: JobLeaseGuard,
    lease_duration_ms: u64,
    cancellation: DurableJobCancellation,
    trace_context: Option<DurableJobTraceContext>,
}

#[derive(Debug)]
pub(super) enum DurableJobRunOutcome<T> {
    Completed(DurableJobRun<T>),
    Cancelled(Job),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DurableJobTraceContext {
    request_id: String,
}

#[derive(Debug)]
pub(super) enum DurableJobOperationError {
    Cancelled,
    Failed(NakoError),
}

pub(super) type DurableJobOperationResult<T> = std::result::Result<T, DurableJobOperationError>;

impl DurableJobContext {
    pub(super) fn trace_context(&self) -> Option<&DurableJobTraceContext> {
        self.trace_context.as_ref()
    }

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

impl DurableJobTraceContext {
    #[must_use]
    pub(crate) fn request_id(&self) -> &str {
        &self.request_id
    }

    pub(crate) fn from_request_id(request_id: impl AsRef<str>) -> Result<Self> {
        let request_id = normalize_trace_request_id(request_id.as_ref()).ok_or_else(|| {
            NakoError::InvalidInput {
                message: "invalid durable job trace request_id".to_owned(),
            }
        })?;
        Ok(Self { request_id })
    }
}

impl<'de> Deserialize<'de> for DurableJobTraceContext {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawTraceContext {
            request_id: String,
        }

        let raw = RawTraceContext::deserialize(deserializer)?;
        Self::from_request_id(raw.request_id).map_err(serde::de::Error::custom)
    }
}

impl From<NakoError> for DurableJobOperationError {
    fn from(value: NakoError) -> Self {
        Self::Failed(value)
    }
}

impl<T> DurableJobRunOutcome<T> {
    fn into_completed(self) -> Result<DurableJobRun<T>> {
        match self {
            Self::Completed(run) => Ok(run),
            Self::Cancelled(job) => Err(NakoError::Conflict {
                message: format!("job {} was cancelled", job.id),
            }),
        }
    }
}

impl DurableJobRuntime {
    pub(super) fn new<S>(store: S) -> Self
    where
        S: DurableJobLeaseStore + 'static,
    {
        Self {
            store: Arc::new(store),
            worker_id: default_worker_id(),
            lease_duration_ms: DEFAULT_LEASE_DURATION_MS,
            heartbeat_interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
        }
    }

    #[cfg(test)]
    pub(super) fn with_lease_timing<S>(
        store: S,
        lease_duration_ms: u64,
        heartbeat_interval_ms: u64,
    ) -> Self
    where
        S: DurableJobLeaseStore + 'static,
    {
        Self {
            store: Arc::new(store),
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
        self.run_job_with_trace_context(job_id, operation, None, run, summary_json)
            .await
    }

    pub(super) async fn run_job_with_trace_context<T, Run, RunFuture, Summary>(
        &self,
        job_id: JobId,
        operation: &'static str,
        trace_context: Option<DurableJobTraceContext>,
        run: Run,
        summary_json: Summary,
    ) -> Result<DurableJobRunOutcome<T>>
    where
        Run: FnOnce(DurableJobContext) -> RunFuture,
        RunFuture: Future<Output = DurableJobOperationResult<T>>,
        Summary: FnOnce(&T) -> Result<Option<String>>,
    {
        let leased = self
            .claim_next_job_lease(JobLeaseClaimFilter {
                job_id: Some(job_id),
                ..JobLeaseClaimFilter::default()
            })
            .await?
            .ok_or_else(|| NakoError::Conflict {
                message: format!("job {job_id} is not queued and claimable"),
            })?;

        self.run_leased_job_with_trace_context(leased, operation, trace_context, run, summary_json)
            .await
    }

    pub(super) async fn claim_next_job_lease(
        &self,
        filter: JobLeaseClaimFilter,
    ) -> Result<Option<LeasedJob>> {
        self.store
            .claim_next_job_lease(JobLeaseClaimRequest {
                worker_id: self.worker_id,
                lease_duration_ms: self.lease_duration_ms,
                filter,
            })
            .await
    }

    pub(super) async fn run_leased_job_with_trace_context<T, Run, RunFuture, Summary>(
        &self,
        leased: LeasedJob,
        operation: &'static str,
        trace_context: Option<DurableJobTraceContext>,
        run: Run,
        summary_json: Summary,
    ) -> Result<DurableJobRunOutcome<T>>
    where
        Run: FnOnce(DurableJobContext) -> RunFuture,
        RunFuture: Future<Output = DurableJobOperationResult<T>>,
        Summary: FnOnce(&T) -> Result<Option<String>>,
    {
        let guard = leased.lease.guard();
        let cancellation = DurableJobCancellation::new();
        cancellation.observe_lease(&leased.lease);
        let heartbeat = self.start_heartbeat(operation, guard, cancellation.clone());
        let context = DurableJobContext {
            store: self.store.clone(),
            guard,
            lease_duration_ms: self.lease_duration_ms,
            cancellation,
            trace_context,
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

    async fn fail_job(&self, guard: JobLeaseGuard, operation: &'static str, err: &NakoError) {
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
            .map_err(|err| NakoError::InvalidInput {
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

fn normalize_trace_request_id(value: &str) -> Option<String> {
    if value.len() < MIN_TRACE_REQUEST_ID_LEN || value.len() > MAX_TRACE_REQUEST_ID_LEN {
        return None;
    }
    if !value.bytes().all(is_safe_trace_request_id_byte) {
        return None;
    }

    Some(value.to_ascii_lowercase())
}

fn is_safe_trace_request_id_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_' | b'.')
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

    fn observe_lease(&self, lease: &nako_core::JobLeaseRecord) {
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
    use nako_core::{
        DatabaseLifecycle, JobKind, JobLeaseClaimFilter, JobPriority, JobRepository, JobStatus,
        NewJob,
    };
    use nako_db::NakoDatabase;
    use tokio::sync::Notify;

    async fn migrated_store() -> NakoDatabase {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
    }

    #[test]
    fn durable_job_trace_context_normalizes_safe_request_id() {
        let context = DurableJobTraceContext::from_request_id("REQ-SCAN_123.Trace").unwrap();

        assert_eq!(context.request_id(), "req-scan_123.trace");
        assert_eq!(
            serde_json::to_value(&context).unwrap(),
            serde_json::json!({ "request_id": "req-scan_123.trace" })
        );
    }

    #[test]
    fn durable_job_trace_context_rejects_unsafe_request_id_without_echoing_it() {
        let err =
            DurableJobTraceContext::from_request_id("https://secret.example/path?token=private")
                .unwrap_err();
        let message = err.to_string();

        assert!(message.contains("invalid durable job trace request_id"));
        assert!(!message.contains("secret.example"));
        assert!(!message.contains("token"));
        assert!(!message.contains("private"));
    }

    #[test]
    fn durable_job_trace_context_deserialization_rejects_unsafe_request_id() {
        let err = serde_json::from_value::<DurableJobTraceContext>(serde_json::json!({
            "request_id": "C:\\Users\\secret\\Movies"
        }))
        .unwrap_err();
        let message = err.to_string();

        assert!(message.contains("invalid durable job trace request_id"));
        assert!(!message.contains("Users"));
        assert!(!message.contains("secret"));
        assert!(!message.contains("Movies"));
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
                priority: JobPriority::Normal,
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
                priority: JobPriority::Normal,
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
                    Ok::<_, NakoError>(1_u32)
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
            nako_core::JobLeaseRepository::claim_next_job_lease(
                &store,
                nako_core::JobLeaseClaimRequest {
                    worker_id: nako_core::JobWorkerId::new(),
                    lease_duration_ms: 10_000,
                    filter: JobLeaseClaimFilter {
                        job_id: Some(job.id),
                        ..JobLeaseClaimFilter::default()
                    },
                },
            )
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
                priority: JobPriority::Normal,
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
                    Err::<u32, _>(NakoError::InvalidInput {
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
                priority: JobPriority::Normal,
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
            .request_job_cancellation(nako_core::RequestJobCancellation {
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
                priority: JobPriority::Normal,
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
                || async { Ok::<_, NakoError>(1_u32) },
                |_value| {
                    Err(NakoError::InvalidInput {
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
