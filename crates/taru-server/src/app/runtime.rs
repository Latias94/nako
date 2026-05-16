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
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

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
    resource_class: &'static str,
    abort_handle: AbortHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeSupervisorDiagnostics {
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub shutdown_requested: bool,
    pub tasks: Vec<RuntimeTaskDiagnostics>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeTaskDiagnostics {
    pub id: u64,
    pub name: &'static str,
    pub resource_class: &'static str,
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
        resource_class: &'static str,
        future: F,
    ) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.inner.next_task_id.fetch_add(1, Ordering::Relaxed);
        let inner = self.inner.clone();
        let handle = tokio::spawn(async move {
            let outcome = AssertUnwindSafe(future).catch_unwind().await;
            if outcome.is_err() {
                inner.failed_tasks.fetch_add(1, Ordering::Relaxed);
                error!(
                    task_id = id,
                    task_name = name,
                    resource_class,
                    "runtime task panicked"
                );
            }
            inner.complete_task(id);
        });
        let abort_handle = handle.abort_handle();
        drop(handle);

        self.inner
            .register_task(id, name, resource_class, abort_handle);
        id
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
                resource_class = task.resource_class,
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
            shutdown_requested: self.inner.shutdown_requested.load(Ordering::Relaxed),
            tasks: state
                .tasks
                .iter()
                .map(|(id, task)| RuntimeTaskDiagnostics {
                    id: *id,
                    name: task.name,
                    resource_class: task.resource_class,
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
        resource_class: &'static str,
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
}
