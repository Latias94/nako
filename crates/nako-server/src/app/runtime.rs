use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    panic::AssertUnwindSafe,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use futures_util::FutureExt;
use nako_core::{
    GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS, Job, JobId, JobKind, JobStatus,
    METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS, NakoError, Result,
};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    task::AbortHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub(super) const RUNTIME_RESOURCE_CLASS_DISK_SCAN: &str = "disk.scan";
pub(super) const RUNTIME_RESOURCE_CLASS_METADATA_SHARED: &str = "metadata.shared";
pub(super) const RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK: &str = "network.webhook";
pub(super) const RUNTIME_RESOURCE_CLASS_ARTWORK_INGEST: &str = "artwork.ingest";
pub(super) const RUNTIME_RESOURCE_CLASS_ADDON_TASK: &str = "addon.task";

#[derive(Clone, Debug)]
pub(super) struct RuntimeResourceClassRegistry {
    classes: Arc<BTreeMap<String, RuntimeResourceClass>>,
}

#[derive(Debug)]
struct RuntimeResourceClass {
    semaphore: Arc<Semaphore>,
    max_permits: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeResourceClassDiagnostics {
    pub name: String,
    pub available_permits: usize,
    pub max_permits: usize,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeSupervisor {
    inner: Arc<RuntimeSupervisorInner>,
}

#[derive(Debug)]
struct RuntimeSupervisorInner {
    state: Mutex<RuntimeSupervisorState>,
    cancellation: CancellationToken,
    next_task_id: AtomicU64,
    completed_tasks: AtomicU64,
    failed_tasks: AtomicU64,
    succeeded_jobs: AtomicU64,
    cancelled_jobs: AtomicU64,
    failed_jobs: AtomicU64,
    shutdown_requested: AtomicBool,
}

#[derive(Debug, Default)]
struct RuntimeSupervisorState {
    tasks: BTreeMap<u64, RuntimeTaskRecord>,
    completed_before_registration: BTreeSet<u64>,
}

#[derive(Debug)]
struct RuntimeTaskRecord {
    name: &'static str,
    resource_class: String,
    job_id: Option<JobId>,
    abort_handle: AbortHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeSupervisorDiagnostics {
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub succeeded_jobs: u64,
    pub cancelled_jobs: u64,
    pub failed_jobs: u64,
    pub shutdown_requested: bool,
    pub tasks: Vec<RuntimeTaskDiagnostics>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeTaskDiagnostics {
    pub id: u64,
    pub name: &'static str,
    pub resource_class: String,
    pub job_id: Option<JobId>,
}

#[derive(Clone, Debug)]
pub(crate) struct RuntimeJobContext {
    pub job_id: JobId,
    shutdown_token: CancellationToken,
}

impl RuntimeJobContext {
    pub(crate) fn shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }
}

impl RuntimeResourceClassRegistry {
    pub(super) fn new<N>(entries: impl IntoIterator<Item = (N, usize)>) -> Result<Self>
    where
        N: Into<String>,
    {
        let mut classes = BTreeMap::new();

        for (name, max_permits) in entries {
            let name = name.into();
            validate_runtime_resource_class_name(&name)?;
            if max_permits == 0 {
                return Err(NakoError::InvalidInput {
                    message: format!(
                        "runtime resource class `{name}` must have at least one permit"
                    ),
                });
            }

            let class = RuntimeResourceClass {
                semaphore: Arc::new(Semaphore::new(max_permits)),
                max_permits,
            };
            if classes.insert(name.clone(), class).is_some() {
                return Err(NakoError::InvalidInput {
                    message: format!("duplicate runtime resource class `{name}`"),
                });
            }
        }

        Ok(Self {
            classes: Arc::new(classes),
        })
    }

    pub(super) fn semaphore(&self, name: &str) -> Result<Arc<Semaphore>> {
        self.classes
            .get(name)
            .map(|class| class.semaphore.clone())
            .ok_or_else(|| NakoError::InvalidInput {
                message: format!("unknown runtime resource class `{name}`"),
            })
    }

    pub(super) async fn acquire(&self, name: &str) -> Result<OwnedSemaphorePermit> {
        let semaphore = self.semaphore(name)?;
        semaphore
            .acquire_owned()
            .await
            .map_err(|err| NakoError::InvalidInput {
                message: format!("runtime resource class `{name}` is unavailable: {err}"),
            })
    }

    pub(crate) fn diagnostics(&self) -> Vec<RuntimeResourceClassDiagnostics> {
        self.classes
            .iter()
            .map(|(name, class)| RuntimeResourceClassDiagnostics {
                name: name.clone(),
                available_permits: class.semaphore.available_permits(),
                max_permits: class.max_permits,
            })
            .collect()
    }
}

fn validate_runtime_resource_class_name(name: &str) -> Result<()> {
    if name.is_empty() || name.trim() != name {
        return Err(NakoError::InvalidInput {
            message: "runtime resource class name must be non-empty and trimmed".to_owned(),
        });
    }

    Ok(())
}

pub(crate) fn runtime_budget_class_for_job_resource_class(
    kind: JobKind,
    resource_class: &str,
) -> Result<&'static str> {
    let budget_class = match kind {
        JobKind::LibraryScan | JobKind::LibraryProbe
            if resource_class == RUNTIME_RESOURCE_CLASS_DISK_SCAN =>
        {
            Some(RUNTIME_RESOURCE_CLASS_DISK_SCAN)
        }
        JobKind::MetadataRefresh | JobKind::MetadataMaintenance
            if is_metadata_job_resource_class(resource_class) =>
        {
            Some(RUNTIME_RESOURCE_CLASS_METADATA_SHARED)
        }
        JobKind::NfoImport if resource_class == "metadata.nfo.import" => {
            Some(RUNTIME_RESOURCE_CLASS_METADATA_SHARED)
        }
        JobKind::NfoExport if resource_class == "metadata.nfo.export" => {
            Some(RUNTIME_RESOURCE_CLASS_METADATA_SHARED)
        }
        JobKind::ManagedArtworkIngest
            if resource_class == RUNTIME_RESOURCE_CLASS_ARTWORK_INGEST =>
        {
            Some(RUNTIME_RESOURCE_CLASS_ARTWORK_INGEST)
        }
        JobKind::GeneratedArtifactMetadataBulkApply
            if resource_class == GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS =>
        {
            Some(RUNTIME_RESOURCE_CLASS_METADATA_SHARED)
        }
        JobKind::MetadataCandidateReviewBatchApply
            if resource_class == METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS =>
        {
            Some(RUNTIME_RESOURCE_CLASS_METADATA_SHARED)
        }
        JobKind::WebhookDelivery if resource_class == RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK => {
            Some(RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK)
        }
        JobKind::AddonTask
            if resource_class == RUNTIME_RESOURCE_CLASS_ADDON_TASK
                || resource_class == "addon.generated_artifact_handoff"
                || is_legacy_addon_task_resource_class(resource_class) =>
        {
            Some(RUNTIME_RESOURCE_CLASS_ADDON_TASK)
        }
        _ => None,
    };

    budget_class.ok_or_else(|| NakoError::InvalidInput {
        message: format!(
            "job kind `{}` with resource class `{resource_class}` has no runtime budget class mapping",
            kind.as_str()
        ),
    })
}

fn is_metadata_job_resource_class(resource_class: &str) -> bool {
    matches!(
        resource_class,
        "metadata.tmdb"
            | "metadata.douban"
            | "metadata.bangumi"
            | "metadata.imdb"
            | "metadata.local"
            | "metadata.other"
            | "metadata.maintenance"
    )
}

fn is_legacy_addon_task_resource_class(resource_class: &str) -> bool {
    resource_class
        .strip_prefix("addon.task.")
        .is_some_and(|suffix| !suffix.is_empty())
}

impl RuntimeSupervisor {
    pub(super) fn new() -> Self {
        Self {
            inner: Arc::new(RuntimeSupervisorInner {
                state: Mutex::new(RuntimeSupervisorState::default()),
                cancellation: CancellationToken::new(),
                next_task_id: AtomicU64::new(1),
                completed_tasks: AtomicU64::new(0),
                failed_tasks: AtomicU64::new(0),
                succeeded_jobs: AtomicU64::new(0),
                cancelled_jobs: AtomicU64::new(0),
                failed_jobs: AtomicU64::new(0),
                shutdown_requested: AtomicBool::new(false),
            }),
        }
    }

    pub(super) fn shutdown_token(&self) -> CancellationToken {
        self.inner.cancellation.child_token()
    }

    pub(super) fn spawn<F>(
        &self,
        name: &'static str,
        resource_class: impl Into<String>,
        future: F,
    ) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn_tracked(name, resource_class.into(), None, future)
    }

    fn spawn_tracked<F>(
        &self,
        name: &'static str,
        resource_class: String,
        job_id: Option<JobId>,
        future: F,
    ) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.inner.next_task_id.fetch_add(1, Ordering::Relaxed);
        let inner = self.inner.clone();
        let log_resource_class = resource_class.clone();
        let handle = tokio::spawn(async move {
            let outcome = AssertUnwindSafe(future).catch_unwind().await;
            if outcome.is_err() {
                inner.failed_tasks.fetch_add(1, Ordering::Relaxed);
                error!(
                    task_id = id,
                    task_name = name,
                    resource_class = %log_resource_class,
                    job_id = job_id.map(|id| id.to_string()),
                    "runtime task panicked"
                );
            }
            inner.complete_task(id);
        });
        let abort_handle = handle.abort_handle();
        drop(handle);

        self.inner
            .register_task(id, name, resource_class, job_id, abort_handle);
        id
    }

    pub(super) fn spawn_job<F, Fut>(
        &self,
        name: &'static str,
        resource_class: impl Into<String>,
        job_id: JobId,
        run: F,
    ) -> u64
    where
        F: FnOnce(RuntimeJobContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Job>> + Send + 'static,
    {
        let inner = self.inner.clone();
        let context = RuntimeJobContext {
            job_id,
            shutdown_token: self.shutdown_token(),
        };
        self.spawn_tracked(name, resource_class.into(), Some(job_id), async move {
            match run(context).await {
                Ok(job) => {
                    match job.status {
                        JobStatus::Succeeded => {
                            inner.succeeded_jobs.fetch_add(1, Ordering::Relaxed);
                        }
                        JobStatus::Cancelled => {
                            inner.cancelled_jobs.fetch_add(1, Ordering::Relaxed);
                        }
                        JobStatus::Failed => {
                            inner.failed_jobs.fetch_add(1, Ordering::Relaxed);
                        }
                        JobStatus::Queued | JobStatus::Running => {
                            inner.failed_jobs.fetch_add(1, Ordering::Relaxed);
                            warn!(
                                job_id = %job.id,
                                job_kind = %job.kind.as_str(),
                                status = %job.status.as_str(),
                                "supervised job finished with non-terminal status"
                            );
                        }
                    }
                    info!(
                        job_id = %job.id,
                        job_kind = %job.kind.as_str(),
                        job_status = ?job.status,
                        resource_class = %job.resource_class,
                        task_name = name,
                        "supervised job completed"
                    );
                }
                Err(err) => {
                    inner.failed_jobs.fetch_add(1, Ordering::Relaxed);
                    error!(
                        job_id = %job_id,
                        task_name = name,
                        error = %err,
                        "supervised job failed"
                    );
                }
            }
        })
    }

    pub(super) fn shutdown(&self) {
        self.inner.shutdown_requested.store(true, Ordering::Relaxed);
        self.inner.cancellation.cancel();

        let tasks = {
            let mut state = self
                .inner
                .state
                .lock()
                .expect("runtime supervisor poisoned");
            std::mem::take(&mut state.tasks)
        };
        for (id, task) in tasks {
            warn!(
                task_id = id,
                task_name = task.name,
                resource_class = %task.resource_class,
                job_id = task.job_id.map(|id| id.to_string()),
                "aborting runtime task during shutdown"
            );
            task.abort_handle.abort();
        }
    }

    pub(crate) fn diagnostics(&self) -> RuntimeSupervisorDiagnostics {
        let state = self
            .inner
            .state
            .lock()
            .expect("runtime supervisor poisoned");
        RuntimeSupervisorDiagnostics {
            active_tasks: state.tasks.len(),
            completed_tasks: self.inner.completed_tasks.load(Ordering::Relaxed),
            failed_tasks: self.inner.failed_tasks.load(Ordering::Relaxed),
            succeeded_jobs: self.inner.succeeded_jobs.load(Ordering::Relaxed),
            cancelled_jobs: self.inner.cancelled_jobs.load(Ordering::Relaxed),
            failed_jobs: self.inner.failed_jobs.load(Ordering::Relaxed),
            shutdown_requested: self.inner.shutdown_requested.load(Ordering::Relaxed),
            tasks: state
                .tasks
                .iter()
                .map(|(id, task)| RuntimeTaskDiagnostics {
                    id: *id,
                    name: task.name,
                    resource_class: task.resource_class.clone(),
                    job_id: task.job_id,
                })
                .collect(),
        }
    }
}

impl RuntimeSupervisorInner {
    fn register_task(
        &self,
        id: u64,
        name: &'static str,
        resource_class: String,
        job_id: Option<JobId>,
        abort_handle: AbortHandle,
    ) {
        if self.shutdown_requested.load(Ordering::Relaxed) {
            abort_handle.abort();
            return;
        }

        let mut state = self.state.lock().expect("runtime supervisor poisoned");
        if state.completed_before_registration.remove(&id) {
            return;
        }

        state.tasks.insert(
            id,
            RuntimeTaskRecord {
                name,
                resource_class,
                job_id,
                abort_handle,
            },
        );
    }

    fn complete_task(&self, id: u64) {
        self.completed_tasks.fetch_add(1, Ordering::Relaxed);
        let mut state = self.state.lock().expect("runtime supervisor poisoned");
        if state.tasks.remove(&id).is_none() {
            state.completed_before_registration.insert(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use nako_core::{JobKind, JobPriority, JobStatus};
    use tokio::sync::Notify;

    use super::*;

    #[test]
    fn runtime_resource_class_registry_rejects_duplicate_names() {
        let error =
            RuntimeResourceClassRegistry::new([("metadata.shared", 1), ("metadata.shared", 2)])
                .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("duplicate runtime resource class `metadata.shared`")
        );
    }

    #[test]
    fn runtime_resource_class_registry_rejects_unknown_names() {
        let registry = RuntimeResourceClassRegistry::new([("disk.scan", 1)]).unwrap();
        let error = registry.semaphore("metadata.shared").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("unknown runtime resource class `metadata.shared`")
        );
    }

    #[tokio::test]
    async fn runtime_resource_class_diagnostics_are_sorted_and_track_available_permits() {
        let registry =
            RuntimeResourceClassRegistry::new([("metadata.shared", 2), ("disk.scan", 1)]).unwrap();

        assert_eq!(
            registry.diagnostics(),
            vec![
                RuntimeResourceClassDiagnostics {
                    name: "disk.scan".to_owned(),
                    available_permits: 1,
                    max_permits: 1,
                },
                RuntimeResourceClassDiagnostics {
                    name: "metadata.shared".to_owned(),
                    available_permits: 2,
                    max_permits: 2,
                },
            ]
        );

        let permit = registry.acquire("metadata.shared").await.unwrap();
        assert_eq!(
            registry.diagnostics()[1],
            RuntimeResourceClassDiagnostics {
                name: "metadata.shared".to_owned(),
                available_permits: 1,
                max_permits: 2,
            }
        );

        drop(permit);
        assert_eq!(
            registry.diagnostics()[1],
            RuntimeResourceClassDiagnostics {
                name: "metadata.shared".to_owned(),
                available_permits: 2,
                max_permits: 2,
            }
        );
    }

    #[test]
    fn runtime_job_resource_class_mapping_maps_known_jobs_to_budget_classes() {
        let cases = [
            (
                JobKind::LibraryScan,
                "disk.scan",
                RUNTIME_RESOURCE_CLASS_DISK_SCAN,
            ),
            (
                JobKind::LibraryProbe,
                "disk.scan",
                RUNTIME_RESOURCE_CLASS_DISK_SCAN,
            ),
            (
                JobKind::MetadataRefresh,
                "metadata.tmdb",
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::MetadataRefresh,
                "metadata.douban",
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::MetadataMaintenance,
                "metadata.maintenance",
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::MetadataMaintenance,
                "metadata.bangumi",
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::NfoImport,
                "metadata.nfo.import",
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::NfoExport,
                "metadata.nfo.export",
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::ManagedArtworkIngest,
                "artwork.ingest",
                RUNTIME_RESOURCE_CLASS_ARTWORK_INGEST,
            ),
            (
                JobKind::GeneratedArtifactMetadataBulkApply,
                GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS,
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::MetadataCandidateReviewBatchApply,
                METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS,
                RUNTIME_RESOURCE_CLASS_METADATA_SHARED,
            ),
            (
                JobKind::WebhookDelivery,
                "network.webhook",
                RUNTIME_RESOURCE_CLASS_NETWORK_WEBHOOK,
            ),
            (
                JobKind::AddonTask,
                "addon.task",
                RUNTIME_RESOURCE_CLASS_ADDON_TASK,
            ),
            (
                JobKind::AddonTask,
                "addon.task.bulk-refresh",
                RUNTIME_RESOURCE_CLASS_ADDON_TASK,
            ),
            (
                JobKind::AddonTask,
                "addon.generated_artifact_handoff",
                RUNTIME_RESOURCE_CLASS_ADDON_TASK,
            ),
        ];

        for (kind, resource_class, expected_budget_class) in cases {
            assert_eq!(
                runtime_budget_class_for_job_resource_class(kind, resource_class).unwrap(),
                expected_budget_class
            );
        }
    }

    #[test]
    fn runtime_job_resource_class_mapping_rejects_unknown_or_wrong_kind_classes() {
        let unknown =
            runtime_budget_class_for_job_resource_class(JobKind::MetadataRefresh, "metadata.tvdb")
                .unwrap_err();
        assert!(
            unknown
                .to_string()
                .contains("metadata_refresh` with resource class `metadata.tvdb`")
        );

        let wrong_kind =
            runtime_budget_class_for_job_resource_class(JobKind::LibraryScan, "metadata.tmdb")
                .unwrap_err();
        assert!(
            wrong_kind
                .to_string()
                .contains("library_scan` with resource class `metadata.tmdb`")
        );
    }

    #[tokio::test]
    async fn supervisor_tracks_and_completes_runtime_tasks() {
        let supervisor = RuntimeSupervisor::new();

        for _ in 0..10 {
            supervisor.spawn("test_task", "test.runtime", async {});
        }

        tokio::task::yield_now().await;

        let diagnostics = supervisor.diagnostics();
        assert_eq!(diagnostics.active_tasks, 0);
        assert_eq!(diagnostics.completed_tasks, 10);
        assert_eq!(diagnostics.failed_tasks, 0);
        assert_eq!(diagnostics.succeeded_jobs, 0);
        assert_eq!(diagnostics.cancelled_jobs, 0);
        assert_eq!(diagnostics.failed_jobs, 0);
    }

    #[tokio::test]
    async fn shutdown_cancels_registered_runtime_tasks() {
        let supervisor = RuntimeSupervisor::new();
        let entered = Arc::new(Notify::new());
        let entered_task = entered.clone();

        supervisor.spawn("blocked_task", "test.runtime", async move {
            entered_task.notify_one();
            std::future::pending::<()>().await;
        });

        entered.notified().await;
        assert_eq!(supervisor.diagnostics().active_tasks, 1);

        supervisor.shutdown();
        tokio::task::yield_now().await;

        let diagnostics = supervisor.diagnostics();
        assert!(diagnostics.shutdown_requested);
        assert_eq!(diagnostics.active_tasks, 0);
    }

    #[tokio::test]
    async fn supervisor_records_panicked_runtime_tasks() {
        let supervisor = RuntimeSupervisor::new();

        supervisor.spawn("panicking_task", "test.runtime", async {
            panic!("runtime task test panic");
        });

        for _ in 0..50 {
            let diagnostics = supervisor.diagnostics();
            if diagnostics.failed_tasks == 1 {
                assert_eq!(diagnostics.active_tasks, 0);
                assert_eq!(diagnostics.completed_tasks, 1);
                return;
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        panic!(
            "panicked runtime task was not recorded: {:?}",
            supervisor.diagnostics()
        );
    }

    #[tokio::test]
    async fn supervisor_records_supervised_job_outcomes() {
        let supervisor = RuntimeSupervisor::new();
        let success_id = JobId::new();
        let cancelled_id = JobId::new();
        let persisted_failed_id = JobId::new();
        let failed_id = JobId::new();

        supervisor.spawn_job(
            "successful_job",
            "test.job".to_owned(),
            success_id,
            move |context| async move {
                assert_eq!(context.job_id, success_id);
                assert!(!context.shutdown_token().is_cancelled());
                Ok(test_job(success_id, JobStatus::Succeeded))
            },
        );
        supervisor.spawn_job(
            "cancelled_job",
            "test.job".to_owned(),
            cancelled_id,
            move |context| async move {
                assert_eq!(context.job_id, cancelled_id);
                Ok(test_job(cancelled_id, JobStatus::Cancelled))
            },
        );
        supervisor.spawn_job(
            "persisted_failed_job",
            "test.job".to_owned(),
            persisted_failed_id,
            move |context| async move {
                assert_eq!(context.job_id, persisted_failed_id);
                Ok(test_job(persisted_failed_id, JobStatus::Failed))
            },
        );
        supervisor.spawn_job(
            "failed_job",
            "test.job".to_owned(),
            failed_id,
            move |context| async move {
                assert_eq!(context.job_id, failed_id);
                Err(nako_core::NakoError::InvalidInput {
                    message: "job failed".to_owned(),
                })
            },
        );

        for _ in 0..50 {
            let diagnostics = supervisor.diagnostics();
            if diagnostics.succeeded_jobs == 1
                && diagnostics.cancelled_jobs == 1
                && diagnostics.failed_jobs == 2
            {
                assert_eq!(diagnostics.active_tasks, 0);
                assert_eq!(diagnostics.completed_tasks, 4);
                assert_eq!(diagnostics.failed_tasks, 0);
                return;
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        panic!(
            "supervised job outcomes were not recorded: {:?}",
            supervisor.diagnostics()
        );
    }

    #[tokio::test]
    async fn supervisor_exposes_active_job_diagnostics() {
        let supervisor = RuntimeSupervisor::new();
        let job_id = JobId::new();
        let entered = Arc::new(Notify::new());
        let entered_task = entered.clone();

        supervisor.spawn_job(
            "blocked_job",
            "test.blocked",
            job_id,
            move |context| async move {
                entered_task.notify_one();
                context.shutdown_token().cancelled().await;
                Ok(test_job(job_id, JobStatus::Failed))
            },
        );

        entered.notified().await;
        let diagnostics = supervisor.diagnostics();
        assert_eq!(diagnostics.active_tasks, 1);
        assert_eq!(diagnostics.tasks[0].name, "blocked_job");
        assert_eq!(diagnostics.tasks[0].resource_class, "test.blocked");
        assert_eq!(diagnostics.tasks[0].job_id, Some(job_id));

        supervisor.shutdown();
        tokio::task::yield_now().await;

        let diagnostics = supervisor.diagnostics();
        assert!(diagnostics.shutdown_requested);
        assert_eq!(diagnostics.active_tasks, 0);
    }

    fn test_job(id: JobId, status: JobStatus) -> Job {
        Job {
            id,
            kind: JobKind::LibraryScan,
            status,
            resource_class: "test.job".to_owned(),
            priority: JobPriority::Normal,
            library_id: None,
            source_id: None,
            input_json: None,
            summary_json: None,
            error: None,
            attempt: 1,
            max_attempts: 1,
            retry_of_job_id: None,
            next_attempt_at: None,
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        }
    }
}
