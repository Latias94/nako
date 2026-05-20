use std::{borrow::Cow, fmt::Display, str::FromStr};

use sqlx::{
    Decode, Executor, PgPool, Postgres, Row, Type,
    migrate::{Migration, MigrationType, Migrator},
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
};
use taru_core::*;

const POSTGRES_MAX_CONNECTIONS: u32 = 5;

const MIGRATIONS: &[(i64, &str, &str)] = &[(
    1,
    "contract jobs",
    include_str!("../migrations/postgres/0001_contract_jobs.sql"),
)];

const JOB_SELECT: &str = r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM jobs
            "#;

const JOB_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM jobs
            WHERE id = $1
            "#;

const JOB_LEASE_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                worker_id::text AS worker_id,
                run_token::text AS run_token,
                to_char(heartbeat_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS heartbeat_at,
                to_char(lease_expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS lease_expires_at,
                to_char(cancel_requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS cancel_requested_at,
                cancel_reason
            FROM jobs
            WHERE id = $1
            "#;

#[derive(Clone, Debug)]
pub(crate) struct PostgresStore {
    pool: PgPool,
    schema_name: String,
}

impl PostgresStore {
    pub async fn connect_with_schema(database_url: &str, schema_name: &str) -> Result<Self> {
        validate_schema_name(schema_name)?;
        let setup_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(database_error)?;
        let create_schema = format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema_name}""#);
        setup_pool
            .execute(create_schema.as_str())
            .await
            .map_err(database_error)?;
        setup_pool.close().await;

        let options = PgConnectOptions::from_str(database_url)
            .map_err(database_error)?
            .options([("search_path", schema_name)]);
        let pool = PgPoolOptions::new()
            .max_connections(POSTGRES_MAX_CONNECTIONS)
            .connect_with(options)
            .await
            .map_err(database_error)?;

        Ok(Self {
            pool,
            schema_name: schema_name.to_owned(),
        })
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn drop_schema(&self) -> Result<()> {
        validate_schema_name(&self.schema_name)?;
        let drop_schema = format!(r#"DROP SCHEMA IF EXISTS "{}" CASCADE"#, self.schema_name);
        self.pool
            .execute(drop_schema.as_str())
            .await
            .map_err(database_error)?;
        self.pool.close().await;

        Ok(())
    }

    async fn get_job_or_not_found(&self, id: JobId) -> Result<Job> {
        self.get_job(id).await?.ok_or_else(|| TaruError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
    }

    async fn get_leased_job(&self, id: JobId) -> Result<LeasedJob> {
        let row = sqlx::query(JOB_LEASE_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_leased_job)
            .transpose()?
            .ok_or_else(|| TaruError::NotFound {
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
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                to_char(cancel_requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS cancel_requested_at
            FROM jobs
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Err(TaruError::NotFound {
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

#[async_trait::async_trait]
impl DatabaseLifecycle for PostgresStore {
    async fn migrate(&self) -> Result<()> {
        migrator().run(self.pool()).await.map_err(database_error)
    }
}

#[async_trait::async_trait]
impl LibraryRepository for PostgresStore {
    async fn upsert_library(&self, library: &Library) -> Result<()> {
        let roots_json = serde_json::to_string(&library.roots).map_err(database_error)?;
        let options_json = serde_json::to_string(&library.options).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO libraries (
                id, name, roots_json, options_json, domain, preset
            )
            VALUES ($1, $2, $3::jsonb, $4::jsonb, $5, $6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                roots_json = excluded.roots_json,
                options_json = excluded.options_json,
                domain = excluded.domain,
                preset = excluded.preset,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(library.id.as_uuid())
        .bind(&library.name)
        .bind(roots_json)
        .bind(options_json)
        .bind(library.options.domain.as_str())
        .bind(library.options.preset.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                name,
                roots_json::text AS roots_json,
                options_json::text AS options_json
            FROM libraries
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_library).transpose()
    }

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                name,
                roots_json::text AS roots_json,
                options_json::text AS options_json
            FROM libraries
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_library).collect()
    }
}

#[async_trait::async_trait]
impl JobRepository for PostgresStore {
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
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(job.id.as_uuid())
        .bind(job.kind.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(job.resource_class)
        .bind(job.library_id.map(|id| id.as_uuid()))
        .bind(job.source_id.map(|id| id.as_uuid()))
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
                status = $2,
                started_at = COALESCE(started_at, statement_timestamp()),
                completed_at = NULL,
                error = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
                status = $2,
                summary_json = $3,
                error = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
                status = $2,
                error = $3,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
                status = $1,
                error = $2,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE status = $3
                AND kind <> $4
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
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_job).transpose()
    }

    async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>> {
        let page = page.clamped();
        let status = filter.status.map(|status| status.as_str().to_owned());
        let kind = filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = filter.resource_class;
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let source_id = filter.source_id.map(|id| id.as_uuid());
        let rows = sqlx::query(&format!(
            r#"
            {JOB_SELECT}
            WHERE ($1::text IS NULL OR status = $1)
                AND ($2::text IS NULL OR kind = $2)
                AND ($3::text IS NULL OR resource_class = $3)
                AND ($4::uuid IS NULL OR library_id = $4)
                AND ($5::uuid IS NULL OR source_id = $5)
            ORDER BY queued_at DESC, id DESC
            LIMIT $6 OFFSET $7
            "#
        ))
        .bind(status.as_deref())
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(library_id)
        .bind(source_id)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_job).collect()
    }
}

#[async_trait::async_trait]
impl JobLeaseRepository for PostgresStore {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>> {
        validate_lease_duration(request.lease_duration_ms)?;

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let kind = request.filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = request.filter.resource_class;
        let requested_job_id = request.filter.job_id.map(|id| id.as_uuid());
        let library_id = request.filter.library_id.map(|id| id.as_uuid());
        let source_id = request.filter.source_id.map(|id| id.as_uuid());
        let job_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id::text
            FROM jobs
            WHERE status = $1
                AND ($2::text IS NULL OR kind = $2)
                AND ($3::text IS NULL OR resource_class = $3)
                AND ($4::uuid IS NULL OR id = $4)
                AND ($5::uuid IS NULL OR library_id = $5)
                AND ($6::uuid IS NULL OR source_id = $6)
            ORDER BY queued_at ASC, id ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
            "#,
        )
        .bind(JobStatus::Queued.as_str())
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(requested_job_id)
        .bind(library_id)
        .bind(source_id)
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
                heartbeat_at = statement_timestamp(),
                lease_expires_at = statement_timestamp() + ($3::double precision * INTERVAL '1 millisecond'),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $4
            "#,
        )
        .bind(heartbeat.guard.job_id.as_uuid())
        .bind(heartbeat.guard.run_token.as_uuid())
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
            "#,
        )
        .bind(completion.guard.job_id.as_uuid())
        .bind(completion.guard.run_token.as_uuid())
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
                status = $3,
                error = $4,
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
            "#,
        )
        .bind(failure.guard.job_id.as_uuid())
        .bind(failure.guard.run_token.as_uuid())
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
                        status = $2,
                        cancel_requested_at = COALESCE(cancel_requested_at, statement_timestamp()),
                        cancel_reason = COALESCE(cancel_reason, $3),
                        summary_json = NULL,
                        error = NULL,
                        worker_id = NULL,
                        run_token = NULL,
                        heartbeat_at = NULL,
                        lease_expires_at = NULL,
                        completed_at = statement_timestamp(),
                        updated_at = statement_timestamp()
                    WHERE id = $1
                        AND status = $4
                    "#,
                )
                .bind(request.job_id.as_uuid())
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
                        cancel_requested_at = COALESCE(cancel_requested_at, statement_timestamp()),
                        cancel_reason = COALESCE(cancel_reason, $2),
                        updated_at = statement_timestamp()
                    WHERE id = $1
                        AND status = $3
                    "#,
                )
                .bind(request.job_id.as_uuid())
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
                Err(TaruError::Conflict {
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
                AND cancel_requested_at IS NOT NULL
            "#,
        )
        .bind(cancellation.guard.job_id.as_uuid())
        .bind(cancellation.guard.run_token.as_uuid())
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
        let job_id = recovery.filter.job_id.map(|id| id.as_uuid());
        let library_id = recovery.filter.library_id.map(|id| id.as_uuid());
        let source_id = recovery.filter.source_id.map(|id| id.as_uuid());
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $8,
                error = $9,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE status = $1
                AND lease_expires_at IS NOT NULL
                AND lease_expires_at < $2::timestamptz
                AND ($3::text IS NULL OR kind = $3)
                AND ($4::text IS NULL OR resource_class = $4)
                AND ($5::uuid IS NULL OR id = $5)
                AND ($6::uuid IS NULL OR library_id = $6)
                AND ($7::uuid IS NULL OR source_id = $7)
            "#,
        )
        .bind(JobStatus::Running.as_str())
        .bind(recovery.expired_before)
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(job_id)
        .bind(library_id)
        .bind(source_id)
        .bind(JobStatus::Failed.as_str())
        .bind(recovery.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }
}

fn migrator() -> Migrator {
    Migrator {
        migrations: Cow::Owned(
            MIGRATIONS
                .iter()
                .map(|(version, description, sql)| {
                    Migration::new(
                        *version,
                        Cow::Borrowed(*description),
                        MigrationType::Simple,
                        Cow::Borrowed(*sql),
                        false,
                    )
                })
                .collect(),
        ),
        ..Migrator::DEFAULT
    }
}

async fn get_leased_job_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: JobId,
) -> Result<LeasedJob> {
    let row = sqlx::query(JOB_LEASE_SELECT_BY_ID)
        .bind(id.as_uuid())
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?;

    row_to_leased_job(row)
}

fn row_to_library(row: PgRow) -> Result<Library> {
    let roots_json: String = row_get(&row, "roots_json")?;
    let options_json: String = row_get(&row, "options_json")?;

    Ok(Library {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        roots: serde_json::from_str(&roots_json).map_err(database_error)?,
        options: serde_json::from_str(&options_json).map_err(database_error)?,
    })
}

fn row_to_job(row: PgRow) -> Result<Job> {
    Ok(Job {
        id: parse_id(row_get::<String>(&row, "id")?)?,
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
    })
}

fn row_to_leased_job(row: PgRow) -> Result<LeasedJob> {
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

fn row_get<'r, T>(row: &'r PgRow, column: &str) -> Result<T>
where
    T: Decode<'r, Postgres> + Type<Postgres>,
{
    row.try_get(column).map_err(database_error)
}

fn parse_id<T>(value: String) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    value.parse().map_err(database_error)
}

fn parse_optional_id<T>(value: Option<String>) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: Display,
{
    value.map(parse_id).transpose()
}

fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

fn u64_to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|err| TaruError::Database {
        message: format!("value does not fit into PostgreSQL bigint: {err}"),
    })
}

fn validate_lease_duration(lease_duration_ms: u64) -> Result<()> {
    if lease_duration_ms == 0 {
        return Err(TaruError::InvalidInput {
            message: "lease_duration_ms must be greater than zero".to_owned(),
        });
    }

    Ok(())
}

fn stale_job_lease_error() -> TaruError {
    TaruError::Conflict {
        message: "job lease is no longer owned by this run token".to_owned(),
    }
}

fn validate_schema_name(schema_name: &str) -> Result<()> {
    let valid = !schema_name.is_empty()
        && schema_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if !valid {
        return Err(TaruError::InvalidInput {
            message: "PostgreSQL contract schema name must contain only lowercase ASCII letters, digits, and underscores".to_owned(),
        });
    }

    Ok(())
}

fn database_error<E: Display>(err: E) -> TaruError {
    TaruError::Database {
        message: err.to_string(),
    }
}
