use sqlx::{QueryBuilder, Sqlite, sqlite::SqliteRow};

use super::{SqliteStore, codec::*};
use nako_core::*;

const JOB_SELECT: &str = r#"
            SELECT
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json,
                summary_json,
                error,
                queued_at,
                started_at,
                completed_at
            FROM jobs
            "#;

const JOB_SELECT_BY_ID: &str = r#"
            SELECT
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json,
                summary_json,
                error,
                queued_at,
                started_at,
                completed_at
            FROM jobs
            WHERE id = ?1
            "#;

const JOB_LEASE_SELECT_BY_ID: &str = r#"
            SELECT
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json,
                summary_json,
                error,
                queued_at,
                started_at,
                completed_at,
                worker_id,
                run_token,
                heartbeat_at,
                lease_expires_at,
                cancel_requested_at,
                cancel_reason
            FROM jobs
            WHERE id = ?1
            "#;

#[async_trait::async_trait]
impl JobRepository for SqliteStore {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(job.id.to_string())
        .bind(job.kind.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(job.resource_class)
        .bind(job.library_id.map(|id| id.to_string()))
        .bind(job.source_id.map(|id| id.to_string()))
        .bind(job.input_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(job.id).await
    }

    async fn start_job(&self, id: JobId) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                started_at = COALESCE(started_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                completed_at = NULL,
                error = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                summary_json = ?3,
                error = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Succeeded.as_str())
        .bind(summary_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                error = ?3,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn fail_unfinished_jobs(&self, error: String) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?1,
                error = ?2,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE status = ?3
                AND kind <> ?4
            "#,
        )
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .bind(JobStatus::Running.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }

    async fn get_job(&self, id: JobId) -> Result<Option<Job>> {
        let row = sqlx::query(JOB_SELECT_BY_ID)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_job).transpose()
    }

    async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>> {
        let page = page.clamped();
        let mut query = QueryBuilder::new(JOB_SELECT);
        query.push(" WHERE 1 = 1");

        if let Some(status) = filter.status {
            query.push(" AND status = ");
            query.push_bind(status.as_str());
        }
        if let Some(kind) = filter.kind {
            query.push(" AND kind = ");
            query.push_bind(kind.as_str());
        }
        if let Some(resource_class) = filter.resource_class {
            query.push(" AND resource_class = ");
            query.push_bind(resource_class);
        }
        if let Some(library_id) = filter.library_id {
            query.push(" AND library_id = ");
            query.push_bind(library_id.to_string());
        }
        if let Some(source_id) = filter.source_id {
            query.push(" AND source_id = ");
            query.push_bind(source_id.to_string());
        }

        query.push(" ORDER BY queued_at DESC, id DESC LIMIT ");
        query.push_bind(u32_to_i64(page.limit));
        query.push(" OFFSET ");
        query.push_bind(u64_to_i64(page.offset)?);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(row_to_job).collect()
    }
}

#[async_trait::async_trait]
impl JobLeaseRepository for SqliteStore {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>> {
        validate_lease_duration(request.lease_duration_ms)?;

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let kind = request.filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = request.filter.resource_class;
        let requested_job_id = request.filter.job_id.map(|id| id.to_string());
        let library_id = request.filter.library_id.map(|id| id.to_string());
        let source_id = request.filter.source_id.map(|id| id.to_string());
        let job_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id
            FROM jobs
            WHERE status = ?1
                AND (?2 IS NULL OR kind = ?2)
                AND (?3 IS NULL OR resource_class = ?3)
                AND (?4 IS NULL OR id = ?4)
                AND (?5 IS NULL OR library_id = ?5)
                AND (?6 IS NULL OR source_id = ?6)
            ORDER BY queued_at ASC, id ASC
            LIMIT 1
            "#,
        )
        .bind(JobStatus::Queued.as_str())
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(requested_job_id.as_deref())
        .bind(library_id.as_deref())
        .bind(source_id.as_deref())
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
                status = ?2,
                worker_id = ?3,
                run_token = ?4,
                heartbeat_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                lease_expires_at = strftime(
                    '%Y-%m-%dT%H:%M:%fZ',
                    'now',
                    '+' || (CAST(?5 AS REAL) / 1000.0) || ' seconds'
                ),
                started_at = COALESCE(started_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                completed_at = NULL,
                error = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND status = ?6
            "#,
        )
        .bind(job_id.to_string())
        .bind(JobStatus::Running.as_str())
        .bind(request.worker_id.to_string())
        .bind(run_token.to_string())
        .bind(u64_to_i64(request.lease_duration_ms)?)
        .bind(JobStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let leased = get_leased_job_tx(&mut transaction, job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(Some(leased))
    }

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob> {
        validate_lease_duration(heartbeat.lease_duration_ms)?;

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                heartbeat_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                lease_expires_at = strftime(
                    '%Y-%m-%dT%H:%M:%fZ',
                    'now',
                    '+' || (CAST(?3 AS REAL) / 1000.0) || ' seconds'
                ),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND run_token = ?2
                AND status = ?4
            "#,
        )
        .bind(heartbeat.guard.job_id.to_string())
        .bind(heartbeat.guard.run_token.to_string())
        .bind(u64_to_i64(heartbeat.lease_duration_ms)?)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_leased_job(heartbeat.guard.job_id).await
    }

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?3,
                summary_json = ?4,
                error = NULL,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                cancel_requested_at = NULL,
                cancel_reason = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND run_token = ?2
                AND status = ?5
            "#,
        )
        .bind(completion.guard.job_id.to_string())
        .bind(completion.guard.run_token.to_string())
        .bind(JobStatus::Succeeded.as_str())
        .bind(completion.summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_job_or_not_found(completion.guard.job_id).await
    }

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?3,
                error = ?4,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                cancel_requested_at = NULL,
                cancel_reason = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND run_token = ?2
                AND status = ?5
            "#,
        )
        .bind(failure.guard.job_id.to_string())
        .bind(failure.guard.run_token.to_string())
        .bind(JobStatus::Failed.as_str())
        .bind(failure.error)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_job_or_not_found(failure.guard.job_id).await
    }

    async fn request_job_cancellation(
        &self,
        request: RequestJobCancellation,
    ) -> Result<JobCancellationRequestRecord> {
        let current = self.get_job_or_not_found(request.job_id).await?;
        match current.status {
            JobStatus::Queued => {
                let updated = sqlx::query(
                    r#"
                    UPDATE jobs
                    SET
                        status = ?2,
                        cancel_requested_at = COALESCE(cancel_requested_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                        cancel_reason = COALESCE(cancel_reason, ?3),
                        summary_json = NULL,
                        error = NULL,
                        worker_id = NULL,
                        run_token = NULL,
                        heartbeat_at = NULL,
                        lease_expires_at = NULL,
                        completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    WHERE id = ?1
                        AND status = ?4
                    "#,
                )
                .bind(request.job_id.to_string())
                .bind(JobStatus::Cancelled.as_str())
                .bind(request.reason)
                .bind(JobStatus::Queued.as_str())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;

                if updated.rows_affected() != 1 {
                    return Err(stale_job_lease_error());
                }

                self.get_cancellation_request_record(request.job_id, true)
                    .await
            }
            JobStatus::Running => {
                let updated = sqlx::query(
                    r#"
                    UPDATE jobs
                    SET
                        cancel_requested_at = COALESCE(cancel_requested_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                        cancel_reason = COALESCE(cancel_reason, ?2),
                        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    WHERE id = ?1
                        AND status = ?3
                    "#,
                )
                .bind(request.job_id.to_string())
                .bind(request.reason)
                .bind(JobStatus::Running.as_str())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;

                if updated.rows_affected() != 1 {
                    return Err(stale_job_lease_error());
                }

                self.get_cancellation_request_record(request.job_id, false)
                    .await
            }
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled => {
                Err(NakoError::Conflict {
                    message: "terminal jobs cannot be cancelled".to_owned(),
                })
            }
        }
    }

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?3,
                summary_json = ?4,
                error = NULL,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND run_token = ?2
                AND status = ?5
                AND cancel_requested_at IS NOT NULL
            "#,
        )
        .bind(cancellation.guard.job_id.to_string())
        .bind(cancellation.guard.run_token.to_string())
        .bind(JobStatus::Cancelled.as_str())
        .bind(cancellation.summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_job_or_not_found(cancellation.guard.job_id).await
    }

    async fn recover_expired_job_leases(&self, recovery: RecoverExpiredJobLeases) -> Result<u64> {
        let kind = recovery.filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = recovery.filter.resource_class;
        let job_id = recovery.filter.job_id.map(|id| id.to_string());
        let library_id = recovery.filter.library_id.map(|id| id.to_string());
        let source_id = recovery.filter.source_id.map(|id| id.to_string());
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?8,
                error = ?9,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE status = ?1
                AND lease_expires_at IS NOT NULL
                AND lease_expires_at < ?2
                AND (?3 IS NULL OR kind = ?3)
                AND (?4 IS NULL OR resource_class = ?4)
                AND (?5 IS NULL OR id = ?5)
                AND (?6 IS NULL OR library_id = ?6)
                AND (?7 IS NULL OR source_id = ?7)
            "#,
        )
        .bind(JobStatus::Running.as_str())
        .bind(recovery.expired_before)
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(job_id.as_deref())
        .bind(library_id.as_deref())
        .bind(source_id.as_deref())
        .bind(JobStatus::Failed.as_str())
        .bind(recovery.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }
}

impl SqliteStore {
    pub(crate) async fn get_job_or_not_found(&self, id: JobId) -> Result<Job> {
        self.get_job(id).await?.ok_or_else(|| NakoError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
    }

    async fn get_leased_job(&self, id: JobId) -> Result<LeasedJob> {
        let row = sqlx::query(JOB_LEASE_SELECT_BY_ID)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_leased_job)
            .transpose()?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: id.to_string(),
            })
    }

    async fn get_cancellation_request_record(
        &self,
        id: JobId,
        terminal: bool,
    ) -> Result<JobCancellationRequestRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json,
                summary_json,
                error,
                queued_at,
                started_at,
                completed_at,
                cancel_requested_at
            FROM jobs
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Err(NakoError::NotFound {
                entity: "job",
                id: id.to_string(),
            });
        };
        let cancel_requested_at = row_get(&row, "cancel_requested_at")?;
        let job = row_to_job(row)?;

        Ok(JobCancellationRequestRecord {
            job,
            requested: true,
            terminal,
            cancel_requested_at,
        })
    }
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

async fn get_leased_job_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: JobId,
) -> Result<LeasedJob> {
    let row = sqlx::query(JOB_LEASE_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?;

    row_to_leased_job(row)
}

fn row_to_leased_job(row: SqliteRow) -> Result<LeasedJob> {
    let job_id: JobId = parse_id(row_get::<String>(&row, "id")?)?;
    let job = Job {
        id: job_id,
        kind: JobKind::parse(&row_get::<String>(&row, "kind")?)?,
        status: JobStatus::parse(&row_get::<String>(&row, "status")?)?,
        resource_class: row_get(&row, "resource_class")?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        input_json: row_get(&row, "input_json")?,
        summary_json: row_get(&row, "summary_json")?,
        error: row_get(&row, "error")?,
        queued_at: row_get(&row, "queued_at")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
    };
    let lease = JobLeaseRecord {
        job_id,
        worker_id: parse_id(row_get::<String>(&row, "worker_id")?)?,
        run_token: parse_id(row_get::<String>(&row, "run_token")?)?,
        heartbeat_at: row_get(&row, "heartbeat_at")?,
        lease_expires_at: row_get(&row, "lease_expires_at")?,
        cancel_requested_at: row_get(&row, "cancel_requested_at")?,
        cancel_reason: row_get(&row, "cancel_reason")?,
    };

    Ok(LeasedJob { job, lease })
}
