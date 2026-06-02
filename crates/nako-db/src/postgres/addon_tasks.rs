use sqlx::{Postgres, postgres::PgRow};

use super::{
    PostgresStore, database_error, i64_to_u32, optional_i64_to_u32, parse_id, parse_optional_id,
    row_get, u32_to_i64, u64_to_i64,
};
use nako_core::*;

const ADDON_TASK_RUN_SELECT: &str = r#"
            SELECT
                jobs.id::text AS id,
                jobs.kind,
                jobs.status,
                jobs.resource_class,
                jobs.priority,
                jobs.library_id::text AS library_id,
                jobs.source_id::text AS source_id,
                jobs.input_json AS job_input_json,
                jobs.summary_json,
                jobs.error,
                jobs.attempt AS job_attempt,
                jobs.max_attempts AS job_max_attempts,
                jobs.retry_of_job_id::text AS job_retry_of_job_id,
                to_char(jobs.next_attempt_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS next_attempt_at,
                to_char(jobs.queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(jobs.started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(jobs.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                to_char(jobs.cancel_requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS cancel_requested_at,
                addon_task_runs.addon_id::text AS addon_id,
                addon_task_runs.manifest_id,
                addon_task_runs.manifest_version,
                addon_task_runs.manifest_fingerprint,
                addon_task_runs.declaration_id,
                addon_task_runs.declaration_name,
                addon_task_runs.declaration_path,
                addon_task_runs.idempotency_key,
                addon_task_runs.request_fingerprint,
                addon_task_runs.attempt,
                addon_task_runs.max_attempts,
                addon_task_runs.retry_of_job_id::text AS retry_of_job_id,
                addon_task_runs.input_json AS run_input_json,
                addon_task_runs.progress_json,
                addon_task_runs.result_json,
                addon_task_runs.safe_error_code,
                to_char(addon_task_runs.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(addon_task_runs.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_task_runs
            INNER JOIN jobs ON jobs.id = addon_task_runs.job_id
            "#;

const LEASED_ADDON_TASK_RUN_SELECT: &str = r#"
            SELECT
                jobs.id::text AS id,
                jobs.kind,
                jobs.status,
                jobs.resource_class,
                jobs.priority,
                jobs.library_id::text AS library_id,
                jobs.source_id::text AS source_id,
                jobs.input_json AS job_input_json,
                jobs.summary_json,
                jobs.error,
                jobs.attempt AS job_attempt,
                jobs.max_attempts AS job_max_attempts,
                jobs.retry_of_job_id::text AS job_retry_of_job_id,
                to_char(jobs.next_attempt_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS next_attempt_at,
                to_char(jobs.queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(jobs.started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(jobs.completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                jobs.worker_id::text AS worker_id,
                jobs.run_token::text AS run_token,
                to_char(jobs.heartbeat_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS heartbeat_at,
                to_char(jobs.lease_expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS lease_expires_at,
                to_char(jobs.cancel_requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS cancel_requested_at,
                jobs.cancel_reason,
                addon_task_runs.addon_id::text AS addon_id,
                addon_task_runs.manifest_id,
                addon_task_runs.manifest_version,
                addon_task_runs.manifest_fingerprint,
                addon_task_runs.declaration_id,
                addon_task_runs.declaration_name,
                addon_task_runs.declaration_path,
                addon_task_runs.idempotency_key,
                addon_task_runs.request_fingerprint,
                addon_task_runs.attempt,
                addon_task_runs.max_attempts,
                addon_task_runs.retry_of_job_id::text AS retry_of_job_id,
                addon_task_runs.input_json AS run_input_json,
                addon_task_runs.progress_json,
                addon_task_runs.result_json,
                addon_task_runs.safe_error_code,
                to_char(addon_task_runs.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(addon_task_runs.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_task_runs
            INNER JOIN jobs ON jobs.id = addon_task_runs.job_id
            "#;

#[async_trait::async_trait]
impl AddonTaskRunRepository for PostgresStore {
    async fn create_addon_task_run(
        &self,
        job: NewJob,
        run: NewAddonTaskRun,
    ) -> Result<CreatedAddonTaskRun> {
        validate_new_task_run(&job, &run)?;
        let job_id = job.id;
        let addon_id = run.addon_id;
        let idempotency_key = run.idempotency_key.clone();
        let request_fingerprint = run.request_fingerprint.clone();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        if let Some(existing) =
            find_addon_task_run_by_idempotency_key_tx(&mut transaction, addon_id, &idempotency_key)
                .await?
        {
            transaction.commit().await.map_err(database_error)?;
            if existing.request_fingerprint != request_fingerprint {
                return Err(NakoError::Conflict {
                    message: format!(
                        "addon task run idempotency key {idempotency_key} was already used for a different request"
                    ),
                });
            }
            return Ok(CreatedAddonTaskRun {
                run: existing,
                idempotent_replay: true,
            });
        }

        insert_job_tx(&mut transaction, job).await?;
        insert_addon_task_run_tx(&mut transaction, run).await?;
        let run = get_addon_task_run_tx(&mut transaction, job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(CreatedAddonTaskRun {
            run,
            idempotent_replay: false,
        })
    }

    async fn get_addon_task_run(&self, job_id: JobId) -> Result<Option<AddonTaskRunRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {ADDON_TASK_RUN_SELECT}
            WHERE addon_task_runs.job_id = $1
            "#
        ))
        .bind(job_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_task_run).transpose()
    }

    async fn list_addon_task_runs(
        &self,
        filter: AddonTaskRunListFilter,
        page: PageRequest,
    ) -> Result<Vec<AddonTaskRunRecord>> {
        let page = page.clamped();
        let addon_id = filter.addon_id.map(|id| id.as_uuid());
        let declaration_id = filter.declaration_id;
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let source_id = filter.source_id.map(|id| id.as_uuid());
        let rows = sqlx::query(&format!(
            r#"
            {ADDON_TASK_RUN_SELECT}
            WHERE ($1::uuid IS NULL OR addon_task_runs.addon_id = $1)
                AND ($2::text IS NULL OR addon_task_runs.declaration_id = $2)
                AND ($3::uuid IS NULL OR jobs.library_id = $3)
                AND ($4::uuid IS NULL OR jobs.source_id = $4)
            ORDER BY jobs.queued_at DESC, jobs.id DESC
            LIMIT $5 OFFSET $6
            "#
        ))
        .bind(addon_id)
        .bind(declaration_id.as_deref())
        .bind(library_id)
        .bind(source_id)
        .bind(i64::from(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_task_run).collect()
    }

    async fn claim_next_addon_task_run(
        &self,
        request: AddonTaskRunClaimRequest,
    ) -> Result<Option<LeasedAddonTaskRun>> {
        validate_lease_duration(request.lease_duration_ms)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let declaration_id = request.declaration_id;
        let job_id_filter = request.job_id.map(|id| id.as_uuid());

        let job_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT jobs.id::text
            FROM jobs
            INNER JOIN addon_task_runs ON addon_task_runs.job_id = jobs.id
            WHERE jobs.status = $1
                AND jobs.kind = $2
                AND addon_task_runs.addon_id = $3
                AND ($4::text IS NULL OR addon_task_runs.declaration_id = $4)
                AND ($5::uuid IS NULL OR jobs.id = $5)
            ORDER BY jobs.queued_at ASC, jobs.id ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
            "#,
        )
        .bind(JobStatus::Queued.as_str())
        .bind(JobKind::AddonTask.as_str())
        .bind(request.addon_id.as_uuid())
        .bind(declaration_id.as_deref())
        .bind(job_id_filter)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some(job_id) = job_id else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        let job_id: JobId = parse_id(job_id)?;
        let run_token = JobRunToken::new();
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $2,
                worker_id = $3,
                run_token = $4,
                heartbeat_at = statement_timestamp(),
                lease_expires_at = statement_timestamp() + ($5::double precision * INTERVAL '1 millisecond'),
                started_at = COALESCE(started_at, statement_timestamp()),
                completed_at = NULL,
                error = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND status = $6
            "#,
        )
        .bind(job_id.as_uuid())
        .bind(JobStatus::Running.as_str())
        .bind(request.worker_id.as_uuid())
        .bind(run_token.as_uuid())
        .bind(u64_to_i64(request.lease_duration_ms)?)
        .bind(JobStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let run = get_leased_addon_task_run_tx(&mut transaction, job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(Some(run))
    }

    async fn report_addon_task_run_progress(
        &self,
        progress: ReportAddonTaskRunProgress,
    ) -> Result<LeasedAddonTaskRun> {
        validate_lease_duration(progress.lease_duration_ms)?;
        let guard: JobLeaseGuard = progress.guard.into();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                heartbeat_at = statement_timestamp(),
                lease_expires_at = statement_timestamp() + ($3::double precision * INTERVAL '1 millisecond'),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $4
                AND kind = $5
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(guard.run_token.as_uuid())
        .bind(u64_to_i64(progress.lease_duration_ms)?)
        .bind(JobStatus::Running.as_str())
        .bind(JobKind::AddonTask.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        let updated = sqlx::query(
            r#"
            UPDATE addon_task_runs
            SET
                progress_json = $2,
                updated_at = statement_timestamp()
            WHERE job_id = $1
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(progress.progress_json)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(NakoError::NotFound {
                entity: "addon_task_run",
                id: guard.job_id.to_string(),
            });
        }

        let run = get_leased_addon_task_run_tx(&mut transaction, guard.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(run)
    }

    async fn complete_addon_task_run(
        &self,
        completion: CompleteAddonTaskRun,
    ) -> Result<AddonTaskRunRecord> {
        let guard: JobLeaseGuard = completion.guard.into();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $3,
                summary_json = $4,
                error = NULL,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                cancel_requested_at = NULL,
                cancel_reason = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $5
                AND kind = $6
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(guard.run_token.as_uuid())
        .bind(JobStatus::Succeeded.as_str())
        .bind(&completion.result_json)
        .bind(JobStatus::Running.as_str())
        .bind(JobKind::AddonTask.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        let updated = sqlx::query(
            r#"
            UPDATE addon_task_runs
            SET
                result_json = $2,
                safe_error_code = NULL,
                updated_at = statement_timestamp()
            WHERE job_id = $1
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(&completion.result_json)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(NakoError::NotFound {
                entity: "addon_task_run",
                id: guard.job_id.to_string(),
            });
        }

        let run = get_addon_task_run_tx(&mut transaction, guard.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(run)
    }

    async fn fail_addon_task_run(&self, failure: FailAddonTaskRun) -> Result<AddonTaskRunRecord> {
        let guard: JobLeaseGuard = failure.guard.into();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $3,
                summary_json = $4,
                error = $5,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                cancel_requested_at = NULL,
                cancel_reason = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $6
                AND kind = $7
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(guard.run_token.as_uuid())
        .bind(JobStatus::Failed.as_str())
        .bind(&failure.result_json)
        .bind(&failure.safe_error_code)
        .bind(JobStatus::Running.as_str())
        .bind(JobKind::AddonTask.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        let updated = sqlx::query(
            r#"
            UPDATE addon_task_runs
            SET
                result_json = $2,
                safe_error_code = $3,
                updated_at = statement_timestamp()
            WHERE job_id = $1
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(&failure.result_json)
        .bind(&failure.safe_error_code)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(NakoError::NotFound {
                entity: "addon_task_run",
                id: guard.job_id.to_string(),
            });
        }

        let run = get_addon_task_run_tx(&mut transaction, guard.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(run)
    }

    async fn cancel_addon_task_run(
        &self,
        cancellation: CancelAddonTaskRun,
    ) -> Result<AddonTaskRunRecord> {
        let guard: JobLeaseGuard = cancellation.guard.into();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $3,
                summary_json = $4,
                error = NULL,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $5
                AND kind = $6
                AND cancel_requested_at IS NOT NULL
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(guard.run_token.as_uuid())
        .bind(JobStatus::Cancelled.as_str())
        .bind(&cancellation.result_json)
        .bind(JobStatus::Running.as_str())
        .bind(JobKind::AddonTask.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        let updated = sqlx::query(
            r#"
            UPDATE addon_task_runs
            SET
                result_json = $2,
                updated_at = statement_timestamp()
            WHERE job_id = $1
            "#,
        )
        .bind(guard.job_id.as_uuid())
        .bind(&cancellation.result_json)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if updated.rows_affected() != 1 {
            return Err(NakoError::NotFound {
                entity: "addon_task_run",
                id: guard.job_id.to_string(),
            });
        }

        let run = get_addon_task_run_tx(&mut transaction, guard.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(run)
    }

    async fn find_addon_task_run_by_idempotency_key(
        &self,
        addon_id: AddonId,
        idempotency_key: &str,
    ) -> Result<Option<AddonTaskRunRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {ADDON_TASK_RUN_SELECT}
            WHERE addon_task_runs.addon_id = $1
                AND addon_task_runs.idempotency_key = $2
            ORDER BY addon_task_runs.created_at ASC, addon_task_runs.job_id ASC
            LIMIT 1
            "#
        ))
        .bind(addon_id.as_uuid())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_task_run).transpose()
    }
}

fn validate_new_task_run(job: &NewJob, run: &NewAddonTaskRun) -> Result<()> {
    if job.id != run.job_id {
        return Err(NakoError::InvalidInput {
            message: "addon task run job_id does not match job id".to_owned(),
        });
    }
    if job.kind != JobKind::AddonTask {
        return Err(NakoError::InvalidInput {
            message: "addon task runs must use JobKind::AddonTask".to_owned(),
        });
    }
    if job.input_json.as_deref() != Some(run.input_json.as_str()) {
        return Err(NakoError::InvalidInput {
            message: "addon task run input_json must match job input_json".to_owned(),
        });
    }
    if run.idempotency_key.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "addon task run idempotency_key must not be empty".to_owned(),
        });
    }
    if run.attempt == 0 {
        return Err(NakoError::InvalidInput {
            message: "addon task run attempt must be greater than zero".to_owned(),
        });
    }
    if run.max_attempts.is_some_and(|max| run.attempt > max) {
        return Err(NakoError::InvalidInput {
            message: "addon task run attempt cannot exceed max_attempts".to_owned(),
        });
    }

    Ok(())
}

async fn insert_job_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    job: NewJob,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO jobs (
            id,
            kind,
            status,
            resource_class,
            priority,
            library_id,
            source_id,
            input_json
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(job.id.as_uuid())
    .bind(job.kind.as_str())
    .bind(JobStatus::Queued.as_str())
    .bind(job.resource_class)
    .bind(job.priority.score())
    .bind(job.library_id.map(|id| id.as_uuid()))
    .bind(job.source_id.map(|id| id.as_uuid()))
    .bind(job.input_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn insert_addon_task_run_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    run: NewAddonTaskRun,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO addon_task_runs (
            job_id,
            addon_id,
            manifest_id,
            manifest_version,
            manifest_fingerprint,
            declaration_id,
            declaration_name,
            declaration_path,
            idempotency_key,
            request_fingerprint,
            attempt,
            max_attempts,
            retry_of_job_id,
            input_json
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
    )
    .bind(run.job_id.as_uuid())
    .bind(run.addon_id.as_uuid())
    .bind(run.manifest_id)
    .bind(run.manifest_version)
    .bind(run.manifest_fingerprint.as_str())
    .bind(run.declaration_id)
    .bind(run.declaration_name)
    .bind(run.declaration_path)
    .bind(run.idempotency_key)
    .bind(run.request_fingerprint.as_str())
    .bind(u32_to_i64(run.attempt))
    .bind(run.max_attempts.map(u32_to_i64))
    .bind(run.retry_of_job_id.map(|id| id.as_uuid()))
    .bind(run.input_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn get_addon_task_run_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    job_id: JobId,
) -> Result<AddonTaskRunRecord> {
    let row = sqlx::query(&format!(
        r#"
        {ADDON_TASK_RUN_SELECT}
        WHERE addon_task_runs.job_id = $1
        "#
    ))
    .bind(job_id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_addon_task_run)
        .transpose()?
        .ok_or_else(|| NakoError::NotFound {
            entity: "addon_task_run",
            id: job_id.to_string(),
        })
}

async fn find_addon_task_run_by_idempotency_key_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    addon_id: AddonId,
    idempotency_key: &str,
) -> Result<Option<AddonTaskRunRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {ADDON_TASK_RUN_SELECT}
        WHERE addon_task_runs.addon_id = $1
            AND addon_task_runs.idempotency_key = $2
        ORDER BY addon_task_runs.created_at ASC, addon_task_runs.job_id ASC
        LIMIT 1
        "#
    ))
    .bind(addon_id.as_uuid())
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_addon_task_run).transpose()
}

async fn get_leased_addon_task_run_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    job_id: JobId,
) -> Result<LeasedAddonTaskRun> {
    let row = sqlx::query(&format!(
        r#"
        {LEASED_ADDON_TASK_RUN_SELECT}
        WHERE addon_task_runs.job_id = $1
        "#
    ))
    .bind(job_id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_leased_addon_task_run)
        .transpose()?
        .ok_or_else(|| NakoError::NotFound {
            entity: "addon_task_run",
            id: job_id.to_string(),
        })
}

fn row_to_addon_task_run(row: PgRow) -> Result<AddonTaskRunRecord> {
    let job_id: JobId = parse_id(row_get::<String>(&row, "id")?)?;
    let job = Job {
        id: job_id,
        kind: JobKind::parse(&row_get::<String>(&row, "kind")?)?,
        status: JobStatus::parse(&row_get::<String>(&row, "status")?)?,
        resource_class: row_get(&row, "resource_class")?,
        priority: JobPriority::from_score(row_get::<i64>(&row, "priority")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        input_json: row_get(&row, "job_input_json")?,
        summary_json: row_get(&row, "summary_json")?,
        error: row_get(&row, "error")?,
        attempt: i64_to_u32(row_get(&row, "job_attempt")?)?,
        max_attempts: i64_to_u32(row_get(&row, "job_max_attempts")?)?,
        retry_of_job_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "job_retry_of_job_id",
        )?)?,
        next_attempt_at: row_get(&row, "next_attempt_at")?,
        queued_at: row_get(&row, "queued_at")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
    };

    Ok(AddonTaskRunRecord {
        job,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        manifest_id: row_get(&row, "manifest_id")?,
        manifest_version: row_get(&row, "manifest_version")?,
        manifest_fingerprint: AddonManifestFingerprint::parse(row_get::<String>(
            &row,
            "manifest_fingerprint",
        )?)?,
        declaration_id: row_get(&row, "declaration_id")?,
        declaration_name: row_get(&row, "declaration_name")?,
        declaration_path: row_get(&row, "declaration_path")?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        request_fingerprint: AddonTaskRunRequestFingerprint::parse(row_get::<String>(
            &row,
            "request_fingerprint",
        )?)?,
        attempt: i64_to_u32(row_get(&row, "attempt")?)?,
        max_attempts: optional_i64_to_u32(row_get(&row, "max_attempts")?)?,
        retry_of_job_id: parse_optional_id(row_get::<Option<String>>(&row, "retry_of_job_id")?)?,
        input_json: row_get(&row, "run_input_json")?,
        progress_json: row_get(&row, "progress_json")?,
        result_json: row_get(&row, "result_json")?,
        safe_error_code: row_get(&row, "safe_error_code")?,
        cancel_requested_at: row_get(&row, "cancel_requested_at")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_leased_addon_task_run(row: PgRow) -> Result<LeasedAddonTaskRun> {
    let job_id: JobId = parse_id(row_get::<String>(&row, "id")?)?;
    let lease = JobLeaseRecord {
        job_id,
        worker_id: parse_id(row_get::<String>(&row, "worker_id")?)?,
        run_token: parse_id(row_get::<String>(&row, "run_token")?)?,
        heartbeat_at: row_get(&row, "heartbeat_at")?,
        lease_expires_at: row_get(&row, "lease_expires_at")?,
        cancel_requested_at: row_get(&row, "cancel_requested_at")?,
        cancel_reason: row_get(&row, "cancel_reason")?,
    };
    let run = row_to_addon_task_run(row)?;

    Ok(LeasedAddonTaskRun { run, lease })
}

fn validate_lease_duration(lease_duration_ms: u64) -> Result<()> {
    if lease_duration_ms == 0 {
        return Err(NakoError::InvalidInput {
            message: "lease_duration_ms must be greater than zero".to_owned(),
        });
    }

    Ok(())
}

fn stale_job_lease_error() -> NakoError {
    NakoError::Conflict {
        message: "job lease is no longer owned by this run token".to_owned(),
    }
}
