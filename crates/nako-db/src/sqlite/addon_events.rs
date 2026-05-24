use super::{SqliteStore, codec::*};
use nako_core::*;

const ADDON_EVENT_DELIVERY_ATTEMPT_SELECT: &str = r#"
            SELECT
                id,
                addon_id,
                event_id,
                declaration_id,
                attempt_number,
                status,
                http_status,
                error,
                requested_at,
                completed_at,
                next_retry_at
            FROM addon_event_delivery_attempts
            "#;

#[async_trait::async_trait]
impl AddonEventDeliveryRepository for SqliteStore {
    async fn create_addon_event_delivery_attempt(
        &self,
        attempt: NewAddonEventDeliveryAttempt,
    ) -> Result<AddonEventDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            INSERT INTO addon_event_delivery_attempts (
                id,
                addon_id,
                event_id,
                declaration_id,
                attempt_number,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(attempt.id.to_string())
        .bind(attempt.addon_id.to_string())
        .bind(attempt.event_id.to_string())
        .bind(&attempt.declaration_id)
        .bind(u32_to_i64(attempt.attempt_number))
        .bind(AddonEventDeliveryStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_event_delivery_attempt_or_not_found(attempt.id)
            .await
    }

    async fn set_addon_event_delivery_attempt_result(
        &self,
        id: AddonEventDeliveryAttemptId,
        status: AddonEventDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<AddonEventDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            UPDATE addon_event_delivery_attempts
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

        self.get_addon_event_delivery_attempt_or_not_found(id).await
    }

    async fn list_addon_event_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<Vec<AddonEventDeliveryAttemptRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {ADDON_EVENT_DELIVERY_ATTEMPT_SELECT}
            WHERE event_id = ?1
            ORDER BY addon_id ASC, declaration_id ASC, attempt_number ASC, requested_at ASC, id ASC
            "#
        ))
        .bind(event_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_addon_event_delivery_attempt)
            .collect()
    }

    async fn list_addon_event_delivery_attempts_for_addon(
        &self,
        addon_id: AddonId,
        event_id: EventId,
        declaration_id: &str,
    ) -> Result<Vec<AddonEventDeliveryAttemptRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {ADDON_EVENT_DELIVERY_ATTEMPT_SELECT}
            WHERE addon_id = ?1 AND event_id = ?2 AND declaration_id = ?3
            ORDER BY attempt_number ASC, requested_at ASC, id ASC
            "#
        ))
        .bind(addon_id.to_string())
        .bind(event_id.to_string())
        .bind(declaration_id)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_addon_event_delivery_attempt)
            .collect()
    }
}

impl SqliteStore {
    pub(crate) async fn get_addon_event_delivery_attempt_or_not_found(
        &self,
        id: AddonEventDeliveryAttemptId,
    ) -> Result<AddonEventDeliveryAttemptRecord> {
        let row = sqlx::query(&format!(
            "{ADDON_EVENT_DELIVERY_ATTEMPT_SELECT} WHERE id = ?1"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_event_delivery_attempt)
            .transpose()?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_event_delivery_attempt",
                id: id.to_string(),
            })
    }
}
