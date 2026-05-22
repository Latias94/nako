use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::Sqlite;

#[async_trait::async_trait]
impl IngestionFailureRepository for SqliteStore {
    async fn record_ingestion_failure(
        &self,
        failure: NewIngestionFailure,
    ) -> Result<IngestionFailureRecord> {
        sqlx::query(
            r#"
            INSERT INTO ingestion_failures (
                library_id, phase, target_uri, target_kind, job_id, scan_id,
                source_id, failure_class, status, message, retryable, attempts,
                first_failed_at_ms, last_failed_at_ms, resolved_at_ms, ignored_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1, ?12, ?12, NULL, NULL)
            ON CONFLICT(library_id, phase, target_uri) DO UPDATE SET
                target_kind = excluded.target_kind,
                job_id = excluded.job_id,
                scan_id = excluded.scan_id,
                source_id = excluded.source_id,
                failure_class = excluded.failure_class,
                status = excluded.status,
                message = excluded.message,
                retryable = excluded.retryable,
                attempts = ingestion_failures.attempts + 1,
                last_failed_at_ms = excluded.last_failed_at_ms,
                resolved_at_ms = NULL,
                ignored_at_ms = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(failure.library_id.to_string())
        .bind(ingestion_failure_phase_to_str(failure.phase))
        .bind(&failure.target_uri)
        .bind(&failure.target_kind)
        .bind(failure.job_id.map(|id| id.to_string()))
        .bind(failure.scan_id.map(|id| id.to_string()))
        .bind(failure.source_id.map(|id| id.to_string()))
        .bind(ingestion_failure_class_to_str(failure.failure_class))
        .bind(ingestion_failure_status_to_str(
            IngestionFailureStatus::Open,
        ))
        .bind(&failure.message)
        .bind(bool_to_i64(failure.retryable))
        .bind(failure.failed_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_ingestion_failure(failure.library_id, failure.phase, &failure.target_uri)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "ingestion_failure",
                id: format!(
                    "{}:{}:{}",
                    failure.library_id,
                    failure.phase.as_str(),
                    failure.target_uri
                ),
            })
    }

    async fn resolve_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        resolved_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.update_ingestion_failure_status(
            library_id,
            phase,
            target_uri,
            IngestionFailureStatus::Resolved,
            Some(resolved_at_ms),
            None,
        )
        .await
    }

    async fn ignore_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        ignored_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.update_ingestion_failure_status(
            library_id,
            phase,
            target_uri,
            IngestionFailureStatus::Ignored,
            None,
            Some(ignored_at_ms),
        )
        .await
    }

    async fn list_ingestion_failures(
        &self,
        filter: IngestionFailureFilter,
        page: PageRequest,
    ) -> Result<Vec<IngestionFailureRecord>> {
        let page = page.clamped();
        let library_id = filter.library_id.map(|id| id.to_string());
        let phase = filter.phase.map(ingestion_failure_phase_to_str);
        let status = filter.status.map(ingestion_failure_status_to_str);
        let rows = sqlx::query(
            r#"
            SELECT
                library_id, phase, target_uri, target_kind, job_id, scan_id,
                source_id, failure_class, status, message, retryable, attempts,
                first_failed_at_ms, last_failed_at_ms, resolved_at_ms, ignored_at_ms
            FROM ingestion_failures
            WHERE (?1 IS NULL OR library_id = ?1)
              AND (?2 IS NULL OR phase = ?2)
              AND (?3 IS NULL OR status = ?3)
            ORDER BY last_failed_at_ms DESC, target_uri ASC
            LIMIT ?4 OFFSET ?5
            "#,
        )
        .bind(library_id)
        .bind(phase)
        .bind(status)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_ingestion_failure).collect()
    }

    async fn count_ingestion_failures(
        &self,
        library_id: LibraryId,
        phase: Option<IngestionFailurePhase>,
        status: IngestionFailureStatus,
    ) -> Result<u64> {
        let phase = phase.map(ingestion_failure_phase_to_str);
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM ingestion_failures
            WHERE library_id = ?1
              AND (?2 IS NULL OR phase = ?2)
              AND status = ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(phase)
        .bind(ingestion_failure_status_to_str(status))
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        i64_to_u64(count)
    }
}

impl SqliteStore {
    async fn get_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
    ) -> Result<Option<IngestionFailureRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id, phase, target_uri, target_kind, job_id, scan_id,
                source_id, failure_class, status, message, retryable, attempts,
                first_failed_at_ms, last_failed_at_ms, resolved_at_ms, ignored_at_ms
            FROM ingestion_failures
            WHERE library_id = ?1 AND phase = ?2 AND target_uri = ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(ingestion_failure_phase_to_str(phase))
        .bind(target_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_ingestion_failure).transpose()
    }

    async fn update_ingestion_failure_status(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        status: IngestionFailureStatus,
        resolved_at_ms: Option<i64>,
        ignored_at_ms: Option<i64>,
    ) -> Result<Option<IngestionFailureRecord>> {
        sqlx::query(
            r#"
            UPDATE ingestion_failures
            SET
                status = ?4,
                resolved_at_ms = ?5,
                ignored_at_ms = ?6,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE library_id = ?1 AND phase = ?2 AND target_uri = ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(ingestion_failure_phase_to_str(phase))
        .bind(target_uri)
        .bind(ingestion_failure_status_to_str(status))
        .bind(resolved_at_ms)
        .bind(ignored_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_ingestion_failure(library_id, phase, target_uri)
            .await
    }
}

pub(crate) async fn resolve_ingestion_failure_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    library_id: LibraryId,
    phase: IngestionFailurePhase,
    target_uri: &str,
    resolved_at_ms: i64,
) -> Result<u64> {
    let result = sqlx::query(
        r#"
            UPDATE ingestion_failures
            SET
                status = ?4,
                resolved_at_ms = ?5,
                ignored_at_ms = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE library_id = ?1 AND phase = ?2 AND target_uri = ?3
            "#,
    )
    .bind(library_id.to_string())
    .bind(ingestion_failure_phase_to_str(phase))
    .bind(target_uri)
    .bind(ingestion_failure_status_to_str(
        IngestionFailureStatus::Resolved,
    ))
    .bind(resolved_at_ms)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(result.rows_affected())
}
