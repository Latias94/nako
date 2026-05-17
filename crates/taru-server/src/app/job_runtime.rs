use std::future::Future;

use serde::Serialize;
use taru_core::{Job, JobId, JobRepository, Result, TaruError};
use taru_db::SqliteStore;
use tracing::warn;

#[derive(Clone, Debug)]
pub(super) struct DurableJobRuntime {
    store: SqliteStore,
}

#[derive(Debug)]
pub(super) struct DurableJobRun<T> {
    pub(super) job: Job,
    pub(super) output: T,
}

impl DurableJobRuntime {
    pub(super) fn new(store: SqliteStore) -> Self {
        Self { store }
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
        self.store.start_job(job_id).await?;

        let output = match run().await {
            Ok(output) => output,
            Err(err) => {
                self.fail_job(job_id, operation, &err).await;
                return Err(err);
            }
        };

        let summary_json = match summary_json(&output) {
            Ok(summary_json) => summary_json,
            Err(err) => {
                self.fail_job(job_id, operation, &err).await;
                return Err(err);
            }
        };

        let job = self.store.succeed_job(job_id, summary_json).await?;
        Ok(DurableJobRun { job, output })
    }

    async fn fail_job(&self, job_id: JobId, operation: &'static str, err: &TaruError) {
        if let Err(update_err) = self.store.fail_job(job_id, err.to_string()).await {
            warn!(
                job_id = %job_id,
                operation,
                error = %update_err,
                "failed to persist failed durable job state"
            );
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use taru_core::{JobKind, JobStatus, NewJob, TransactionManager};
    use taru_db::SqliteStore;

    async fn migrated_store() -> SqliteStore {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
    }

    #[tokio::test]
    async fn durable_job_runtime_persists_success_summary() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::new(store.clone());
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
                || async { Ok::<_, TaruError>(1_u32) },
                |value| DurableJobRuntime::serialize_summary(value, "test summary"),
            )
            .await
            .unwrap();

        assert_eq!(run.output, 1);
        assert_eq!(run.job.status, JobStatus::Succeeded);
        assert_eq!(run.job.summary_json.as_deref(), Some("1"));
        assert!(run.job.started_at.is_some());
        assert!(run.job.completed_at.is_some());
    }

    #[tokio::test]
    async fn durable_job_runtime_persists_failure() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::new(store.clone());
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
    async fn durable_job_runtime_persists_summary_serialization_failure() {
        let store = migrated_store().await;
        let runtime = DurableJobRuntime::new(store.clone());
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
