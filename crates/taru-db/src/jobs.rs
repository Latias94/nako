use super::*;

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

    async fn get_job(&self, id: JobId) -> Result<Option<Job>> {
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
                completed_at
            FROM jobs
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_job).transpose()
    }
}
