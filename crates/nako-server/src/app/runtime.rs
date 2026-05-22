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
use nako_core::{Job, JobId, JobStatus, Result};
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

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

    use nako_core::{JobKind, JobStatus};
    use tokio::sync::Notify;

    use super::*;

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
            library_id: None,
            source_id: None,
            input_json: None,
            summary_json: None,
            error: None,
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        }
    }
}
