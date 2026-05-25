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
                next_retry_at,
                lease_expires_at,
                forced_replay,
                replay_reason_code
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

    async fn claim_addon_event_delivery_attempt(
        &self,
        claim: ClaimAddonEventDeliveryAttempt,
    ) -> Result<Option<AddonEventDeliveryAttemptRecord>> {
        let result = sqlx::query(
            r#"
            WITH summary AS (
                SELECT
                    COALESCE(MAX(attempt_number), 0) AS max_attempt_number,
                    COUNT(*) AS attempt_count,
                    COALESCE(SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END), 0) AS succeeded_count,
                    COALESCE(SUM(CASE
                        WHEN status IN ('pending', 'running')
                            AND (lease_expires_at IS NULL OR lease_expires_at > ?6)
                        THEN 1 ELSE 0 END), 0) AS active_in_flight_count,
                    MAX(CASE
                        WHEN status = 'failed'
                            AND next_retry_at IS NOT NULL
                            AND next_retry_at <= ?6
                        THEN attempt_number ELSE NULL END) AS due_failed_attempt_number,
                    MAX(CASE
                        WHEN status IN ('pending', 'running')
                            AND lease_expires_at IS NOT NULL
                            AND lease_expires_at <= ?6
                        THEN attempt_number ELSE NULL END) AS expired_in_flight_attempt_number
                FROM addon_event_delivery_attempts
                WHERE addon_id = ?2 AND event_id = ?3 AND declaration_id = ?4
            ),
            candidate AS (
                SELECT max_attempt_number + 1 AS attempt_number
                FROM summary
                WHERE active_in_flight_count = 0
                    AND (
                        ?8 = 1
                        OR (
                            max_attempt_number + 1 <= ?5
                            AND succeeded_count = 0
                            AND (
                                attempt_count = 0
                                OR due_failed_attempt_number = max_attempt_number
                                OR expired_in_flight_attempt_number = max_attempt_number
                            )
                        )
                    )
            )
            INSERT OR IGNORE INTO addon_event_delivery_attempts (
                id,
                addon_id,
                event_id,
                declaration_id,
                attempt_number,
                status,
                lease_expires_at,
                forced_replay,
                replay_reason_code
            )
            SELECT ?1, ?2, ?3, ?4, attempt_number, 'running', ?7, ?8, ?9
            FROM candidate
            "#,
        )
        .bind(claim.id.to_string())
        .bind(claim.addon_id.to_string())
        .bind(claim.event_id.to_string())
        .bind(&claim.declaration_id)
        .bind(u32_to_i64(claim.max_attempts))
        .bind(&claim.now)
        .bind(&claim.lease_expires_at)
        .bind(if claim.forced_replay { 1_i64 } else { 0_i64 })
        .bind(&claim.replay_reason_code)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_addon_event_delivery_attempt_or_not_found(claim.id)
            .await
            .map(Some)
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
                next_retry_at = ?5,
                lease_expires_at = NULL
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

    async fn list_addon_event_scheduler_work(
        &self,
        event_id: EventId,
    ) -> Result<Vec<AddonEventSchedulerWorkRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                p.addon_id,
                p.declaration_id,
                p.event_kind,
                p.status AS routing_plan_status,
                p.target AS routing_plan_target,
                p.safe_reason_code AS routing_plan_safe_reason_code
            FROM event_outbox e
            INNER JOIN addon_routing_plans p
                ON p.event_kind = e.kind
                AND p.declaration_kind = ?2
            INNER JOIN addon_registrations a
                ON a.id = p.addon_id
                AND a.status = ?3
            WHERE e.id = ?1
            ORDER BY p.addon_id ASC, p.declaration_id ASC
            "#,
        )
        .bind(event_id.to_string())
        .bind(AddonRoutingDeclarationKind::EventSubscription.as_str())
        .bind(AddonStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut work = Vec::with_capacity(rows.len());
        for row in rows {
            let addon_id = parse_id(row_get::<String>(&row, "addon_id")?)?;
            let declaration_id = row_get::<String>(&row, "declaration_id")?;
            let event_kind = row_get::<String>(&row, "event_kind")?;
            let routing_plan_status =
                AddonRoutingPlanStatus::parse(&row_get::<String>(&row, "routing_plan_status")?)?;
            let routing_plan_target =
                AddonRoutingPlanTarget::parse(&row_get::<String>(&row, "routing_plan_target")?)?;
            let routing_plan_safe_reason_code =
                row_get::<Option<String>>(&row, "routing_plan_safe_reason_code")?;
            let attempts = self
                .list_addon_event_delivery_attempts_for_addon(addon_id, event_id, &declaration_id)
                .await?;

            work.push(addon_event_scheduler_work_record(
                addon_id,
                event_id,
                declaration_id,
                event_kind,
                routing_plan_status,
                routing_plan_target,
                routing_plan_safe_reason_code,
                attempts,
            )?);
        }

        Ok(work)
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

fn addon_event_scheduler_work_record(
    addon_id: AddonId,
    event_id: EventId,
    declaration_id: String,
    event_kind: String,
    routing_plan_status: AddonRoutingPlanStatus,
    routing_plan_target: AddonRoutingPlanTarget,
    routing_plan_safe_reason_code: Option<String>,
    attempts: Vec<AddonEventDeliveryAttemptRecord>,
) -> Result<AddonEventSchedulerWorkRecord> {
    let attempt_count = u32::try_from(attempts.len()).map_err(|_| NakoError::Database {
        message: "addon event scheduler attempt count overflowed u32".to_owned(),
    })?;
    let next_attempt_number = attempts
        .iter()
        .map(|attempt| attempt.attempt_number)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let latest = attempts.iter().max_by_key(|attempt| {
        (
            attempt.attempt_number,
            attempt.requested_at.as_str(),
            attempt.id,
        )
    });
    let has_succeeded = attempts
        .iter()
        .any(|attempt| attempt.status == AddonEventDeliveryStatus::Succeeded);
    let has_in_flight = attempts.iter().any(|attempt| {
        matches!(
            attempt.status,
            AddonEventDeliveryStatus::Pending | AddonEventDeliveryStatus::Running
        )
    });

    Ok(AddonEventSchedulerWorkRecord {
        addon_id,
        event_id,
        declaration_id,
        event_kind,
        routing_plan_status,
        routing_plan_target,
        routing_plan_safe_reason_code,
        attempt_count,
        next_attempt_number,
        latest_attempt_status: latest.map(|attempt| attempt.status),
        latest_http_status: latest.and_then(|attempt| attempt.http_status),
        latest_next_retry_at: latest.and_then(|attempt| attempt.next_retry_at.clone()),
        latest_lease_expires_at: latest.and_then(|attempt| attempt.lease_expires_at.clone()),
        has_succeeded,
        has_in_flight,
    })
}
