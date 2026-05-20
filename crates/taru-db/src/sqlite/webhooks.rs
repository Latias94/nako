use super::{SqliteStore, codec::*};
use taru_core::*;

#[async_trait::async_trait]
impl WebhookRepository for SqliteStore {
    async fn upsert_webhook_endpoint(
        &self,
        endpoint: NewWebhookEndpoint,
    ) -> Result<WebhookEndpointRecord> {
        let event_kinds_json =
            serde_json::to_string(&endpoint.subscribed_event_kinds).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO webhook_endpoints (
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                secret_env = excluded.secret_env,
                subscribed_event_kinds_json = excluded.subscribed_event_kinds_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(endpoint.id.to_string())
        .bind(&endpoint.name)
        .bind(&endpoint.url)
        .bind(&endpoint.secret_env)
        .bind(event_kinds_json)
        .bind(u64_to_i64(endpoint.timeout_ms)?)
        .bind(u32_to_i64(endpoint.max_attempts))
        .bind(endpoint.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_endpoint(endpoint.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "webhook endpoint {} was not found after upsert",
                    endpoint.id
                ),
            })
    }

    async fn get_webhook_endpoint(
        &self,
        id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpointRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM webhook_endpoints
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_webhook_endpoint).transpose()
    }

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM webhook_endpoints
            WHERE status = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(WebhookEndpointStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_webhook_endpoint).collect()
    }

    async fn create_webhook_delivery_attempt(
        &self,
        attempt: NewWebhookDeliveryAttempt,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            INSERT INTO webhook_delivery_attempts (
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(attempt.id.to_string())
        .bind(attempt.endpoint_id.to_string())
        .bind(attempt.event_id.to_string())
        .bind(u32_to_i64(attempt.attempt_number))
        .bind(WebhookDeliveryStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_delivery_attempt_or_not_found(attempt.id)
            .await
    }

    async fn set_webhook_delivery_attempt_result(
        &self,
        id: WebhookDeliveryAttemptId,
        status: WebhookDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            UPDATE webhook_delivery_attempts
            SET
                status = ?2,
                http_status = ?3,
                error = ?4,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                next_retry_at = ?5
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(http_status.map(i64::from))
        .bind(error)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_delivery_attempt_or_not_found(id).await
    }

    async fn list_webhook_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<WebhookDeliveryAttemptRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status,
                http_status,
                error,
                requested_at,
                completed_at,
                next_retry_at
            FROM webhook_delivery_attempts
            WHERE event_id = ?1
            ORDER BY attempt_number ASC, requested_at ASC, id ASC
            "#,
        )
        .bind(event_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_webhook_delivery_attempt)
            .collect()
    }
}

impl SqliteStore {
    pub(crate) async fn get_webhook_delivery_attempt_or_not_found(
        &self,
        id: WebhookDeliveryAttemptId,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status,
                http_status,
                error,
                requested_at,
                completed_at,
                next_retry_at
            FROM webhook_delivery_attempts
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_webhook_delivery_attempt)
            .transpose()?
            .ok_or_else(|| TaruError::NotFound {
                entity: "webhook_delivery_attempt",
                id: id.to_string(),
            })
    }
}
