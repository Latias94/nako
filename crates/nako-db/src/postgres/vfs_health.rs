use sqlx::postgres::PgRow;

use super::{
    PostgresStore, database_error, i64_to_u32, parse_optional_id, row_get, u32_to_i64, u64_to_i64,
};
use nako_core::*;

const STORAGE_BACKEND_HEALTH_SELECT: &str = r#"
            SELECT
                backend_key,
                library_id::text AS library_id,
                scheme,
                status,
                circuit_breaker_state,
                consecutive_failures,
                last_success_at_ms,
                last_failure_at_ms,
                last_failure_class,
                last_failure_safe_message,
                circuit_opened_at_ms,
                backoff_until_ms,
                updated_at_ms
            FROM storage_backend_health
            "#;

#[async_trait::async_trait]
impl StorageBackendHealthRepository for PostgresStore {
    async fn upsert_storage_backend_health(
        &self,
        record: StorageBackendHealthRecord,
    ) -> Result<StorageBackendHealthRecord> {
        sqlx::query(storage_backend_health_upsert_sql())
            .bind(&record.backend_key)
            .bind(record.library_id.map(|id| id.as_uuid()))
            .bind(&record.scheme)
            .bind(record.status.as_str())
            .bind(record.circuit_breaker_state.as_str())
            .bind(u32_to_i64(record.consecutive_failures))
            .bind(record.last_success_at_ms)
            .bind(record.last_failure_at_ms)
            .bind(record.last_failure_class.map(StorageFailureClass::as_str))
            .bind(&record.last_failure_safe_message)
            .bind(record.circuit_opened_at_ms)
            .bind(record.backoff_until_ms)
            .bind(record.updated_at_ms)
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        self.get_storage_backend_health(&record.backend_key)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "storage_backend_health",
                id: record.backend_key,
            })
    }

    async fn get_storage_backend_health(
        &self,
        backend_key: &str,
    ) -> Result<Option<StorageBackendHealthRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {STORAGE_BACKEND_HEALTH_SELECT}
            WHERE backend_key = $1
            "#
        ))
        .bind(backend_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_storage_backend_health_record).transpose()
    }

    async fn list_storage_backend_health(
        &self,
        filter: StorageBackendHealthListFilter,
        page: PageRequest,
    ) -> Result<Vec<StorageBackendHealthRecord>> {
        let page = page.clamped();
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let status = filter.status.map(|status| status.as_str().to_owned());
        let circuit_breaker_state = filter
            .circuit_breaker_state
            .map(|state| state.as_str().to_owned());

        let rows = sqlx::query(&format!(
            r#"
            {STORAGE_BACKEND_HEALTH_SELECT}
            WHERE ($1::uuid IS NULL OR library_id = $1)
              AND ($2::text IS NULL OR scheme = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR circuit_breaker_state = $4)
            ORDER BY updated_at_ms DESC, backend_key ASC
            LIMIT $5 OFFSET $6
            "#
        ))
        .bind(library_id)
        .bind(filter.scheme.as_deref())
        .bind(status.as_deref())
        .bind(circuit_breaker_state.as_deref())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_storage_backend_health_record)
            .collect()
    }

    async fn clear_storage_backend_health(
        &self,
        backend_key: &str,
        cleared_at_ms: i64,
    ) -> Result<Option<StorageBackendHealthRecord>> {
        sqlx::query(
            r#"
            UPDATE storage_backend_health
            SET status = $2,
                circuit_breaker_state = $3,
                consecutive_failures = 0,
                last_failure_at_ms = NULL,
                last_failure_class = NULL,
                last_failure_safe_message = NULL,
                circuit_opened_at_ms = NULL,
                backoff_until_ms = NULL,
                updated_at_ms = $4,
                updated_at = statement_timestamp()
            WHERE backend_key = $1
            "#,
        )
        .bind(backend_key)
        .bind(StorageBackendHealthStatus::Healthy.as_str())
        .bind(StorageCircuitBreakerState::Closed.as_str())
        .bind(cleared_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_storage_backend_health(backend_key).await
    }
}

fn storage_backend_health_upsert_sql() -> &'static str {
    r#"
    INSERT INTO storage_backend_health (
        backend_key,
        library_id,
        scheme,
        status,
        circuit_breaker_state,
        consecutive_failures,
        last_success_at_ms,
        last_failure_at_ms,
        last_failure_class,
        last_failure_safe_message,
        circuit_opened_at_ms,
        backoff_until_ms,
        updated_at_ms
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    ON CONFLICT(backend_key) DO UPDATE SET
        library_id = excluded.library_id,
        scheme = excluded.scheme,
        status = excluded.status,
        circuit_breaker_state = excluded.circuit_breaker_state,
        consecutive_failures = excluded.consecutive_failures,
        last_success_at_ms = excluded.last_success_at_ms,
        last_failure_at_ms = excluded.last_failure_at_ms,
        last_failure_class = excluded.last_failure_class,
        last_failure_safe_message = excluded.last_failure_safe_message,
        circuit_opened_at_ms = excluded.circuit_opened_at_ms,
        backoff_until_ms = excluded.backoff_until_ms,
        updated_at_ms = excluded.updated_at_ms,
        updated_at = statement_timestamp()
    "#
}

fn row_to_storage_backend_health_record(row: PgRow) -> Result<StorageBackendHealthRecord> {
    Ok(StorageBackendHealthRecord {
        backend_key: row_get(&row, "backend_key")?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        scheme: row_get(&row, "scheme")?,
        status: StorageBackendHealthStatus::parse(&row_get::<String>(&row, "status")?)?,
        circuit_breaker_state: StorageCircuitBreakerState::parse(&row_get::<String>(
            &row,
            "circuit_breaker_state",
        )?)?,
        consecutive_failures: i64_to_u32(row_get(&row, "consecutive_failures")?)?,
        last_success_at_ms: row_get(&row, "last_success_at_ms")?,
        last_failure_at_ms: row_get(&row, "last_failure_at_ms")?,
        last_failure_class: row_get::<Option<String>>(&row, "last_failure_class")?
            .map(|value| StorageFailureClass::parse(&value))
            .transpose()?,
        last_failure_safe_message: row_get(&row, "last_failure_safe_message")?,
        circuit_opened_at_ms: row_get(&row, "circuit_opened_at_ms")?,
        backoff_until_ms: row_get(&row, "backoff_until_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}
