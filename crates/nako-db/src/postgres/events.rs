use sqlx::postgres::PgRow;

use super::{
    PostgresStore, database_error, i64_to_u32, i64_to_u64, optional_i64_to_u16, parse_id,
    parse_optional_id, row_get, u32_to_i64, u64_to_i64,
};
use nako_core::*;

const OUTBOX_EVENT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                kind,
                subject_kind,
                subject_id::text AS subject_id,
                library_id::text AS library_id,
                source_id::text AS source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS occurred_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                next_attempt_at
            FROM event_outbox
            "#;

const OUTBOX_EVENT_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                subject_kind,
                subject_id::text AS subject_id,
                library_id::text AS library_id,
                source_id::text AS source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS occurred_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                next_attempt_at
            FROM event_outbox
            WHERE id = $1
            "#;

const WEBHOOK_ENDPOINT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json::text AS subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM webhook_endpoints
            "#;

const WEBHOOK_DELIVERY_ATTEMPT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                endpoint_id::text AS endpoint_id,
                event_id::text AS event_id,
                attempt_number,
                status,
                http_status,
                error,
                to_char(requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS requested_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                next_retry_at
            FROM webhook_delivery_attempts
            "#;

const ADDON_EVENT_DELIVERY_ATTEMPT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                event_id::text AS event_id,
                declaration_id,
                attempt_number,
                status,
                http_status,
                error,
                to_char(requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS requested_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                next_retry_at,
                lease_expires_at
            FROM addon_event_delivery_attempts
            "#;

#[async_trait::async_trait]
impl EventOutboxRepository for PostgresStore {
    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord> {
        let subject_kind = event.subject.kind();
        let subject_id = event.subject.id();

        sqlx::query(
            r#"
            INSERT INTO event_outbox (
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status
            )
            VALUES ($1, $2, $3, $4::uuid, $5, $6, $7, $8, $9)
            ON CONFLICT(kind, idempotency_key) DO NOTHING
            "#,
        )
        .bind(event.id.as_uuid())
        .bind(event.kind.as_str())
        .bind(subject_kind)
        .bind(subject_id)
        .bind(event.library_id.map(|id| id.as_uuid()))
        .bind(event.source_id.map(|id| id.as_uuid()))
        .bind(&event.idempotency_key)
        .bind(&event.payload_json)
        .bind(OutboxEventStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_outbox_event_by_idempotency_key(event.kind, &event.idempotency_key)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "outbox event was not found after enqueue for key {}",
                    event.idempotency_key
                ),
            })
    }

    async fn get_outbox_event(&self, id: EventId) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(OUTBOX_EVENT_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_outbox_event).transpose()
    }

    async fn find_outbox_event_by_idempotency_key(
        &self,
        kind: DomainEventKind,
        idempotency_key: &str,
    ) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(&format!(
            "{OUTBOX_EVENT_SELECT} WHERE kind = $1 AND idempotency_key = $2"
        ))
        .bind(kind.as_str())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_outbox_event).transpose()
    }

    async fn list_outbox_events(
        &self,
        filter: OutboxEventListFilter,
        page: PageRequest,
    ) -> Result<Vec<OutboxEventRecord>> {
        let page = page.clamped();
        let kind = filter.kind.map(|kind| kind.as_str().to_owned());
        let status = filter.status.map(|status| status.as_str().to_owned());
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let source_id = filter.source_id.map(|id| id.as_uuid());
        let rows = sqlx::query(&format!(
            r#"
            {OUTBOX_EVENT_SELECT}
            WHERE ($1::text IS NULL OR kind = $1)
                AND ($2::text IS NULL OR status = $2)
                AND ($3::uuid IS NULL OR library_id = $3)
                AND ($4::uuid IS NULL OR source_id = $4)
            ORDER BY occurred_at DESC, id DESC
            LIMIT $5 OFFSET $6
            "#
        ))
        .bind(kind.as_deref())
        .bind(status.as_deref())
        .bind(library_id)
        .bind(source_id)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_outbox_event).collect()
    }
}

#[async_trait::async_trait]
impl WebhookRepository for PostgresStore {
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
            VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                secret_env = excluded.secret_env,
                subscribed_event_kinds_json = excluded.subscribed_event_kinds_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(endpoint.id.as_uuid())
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
            .ok_or_else(|| NakoError::Database {
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
        let row = sqlx::query(&format!("{WEBHOOK_ENDPOINT_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_webhook_endpoint).transpose()
    }

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {WEBHOOK_ENDPOINT_SELECT}
            WHERE status = $1
            ORDER BY created_at ASC, id ASC
            "#
        ))
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
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(attempt.id.as_uuid())
        .bind(attempt.endpoint_id.as_uuid())
        .bind(attempt.event_id.as_uuid())
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
                status = $2,
                http_status = $3,
                error = $4,
                completed_at = statement_timestamp(),
                next_retry_at = $5
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
        let rows = sqlx::query(&format!(
            r#"
            {WEBHOOK_DELIVERY_ATTEMPT_SELECT}
            WHERE event_id = $1
            ORDER BY attempt_number ASC, requested_at ASC, id ASC
            "#
        ))
        .bind(event_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_webhook_delivery_attempt)
            .collect()
    }
}

impl PostgresStore {
    async fn get_webhook_delivery_attempt_or_not_found(
        &self,
        id: WebhookDeliveryAttemptId,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        let row = sqlx::query(&format!("{WEBHOOK_DELIVERY_ATTEMPT_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_webhook_delivery_attempt)
            .transpose()?
            .ok_or_else(|| NakoError::NotFound {
                entity: "webhook_delivery_attempt",
                id: id.to_string(),
            })
    }
}

#[async_trait::async_trait]
impl AddonEventDeliveryRepository for PostgresStore {
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
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(attempt.id.as_uuid())
        .bind(attempt.addon_id.as_uuid())
        .bind(attempt.event_id.as_uuid())
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
        let row = sqlx::query(
            r#"
            WITH summary AS (
                SELECT
                    COALESCE(MAX(attempt_number), 0) AS max_attempt_number,
                    COUNT(*) AS attempt_count,
                    COALESCE(SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END), 0) AS succeeded_count,
                    COALESCE(SUM(CASE
                        WHEN status IN ('pending', 'running')
                            AND (lease_expires_at IS NULL OR lease_expires_at > $6)
                        THEN 1 ELSE 0 END), 0) AS active_in_flight_count,
                    MAX(CASE
                        WHEN status = 'failed'
                            AND next_retry_at IS NOT NULL
                            AND next_retry_at <= $6
                        THEN attempt_number ELSE NULL END) AS due_failed_attempt_number,
                    MAX(CASE
                        WHEN status IN ('pending', 'running')
                            AND lease_expires_at IS NOT NULL
                            AND lease_expires_at <= $6
                        THEN attempt_number ELSE NULL END) AS expired_in_flight_attempt_number
                FROM addon_event_delivery_attempts
                WHERE addon_id = $2 AND event_id = $3 AND declaration_id = $4
            ),
            candidate AS (
                SELECT max_attempt_number + 1 AS attempt_number
                FROM summary
                WHERE max_attempt_number + 1 <= $5
                    AND succeeded_count = 0
                    AND active_in_flight_count = 0
                    AND (
                        attempt_count = 0
                        OR due_failed_attempt_number = max_attempt_number
                        OR expired_in_flight_attempt_number = max_attempt_number
                    )
            ),
            inserted AS (
                INSERT INTO addon_event_delivery_attempts (
                    id,
                    addon_id,
                    event_id,
                    declaration_id,
                    attempt_number,
                    status,
                    lease_expires_at
                )
                SELECT $1, $2, $3, $4, attempt_number, 'running', $7
                FROM candidate
                ON CONFLICT(addon_id, event_id, declaration_id, attempt_number) DO NOTHING
                RETURNING id::text AS id
            )
            SELECT id FROM inserted
            "#,
        )
        .bind(claim.id.as_uuid())
        .bind(claim.addon_id.as_uuid())
        .bind(claim.event_id.as_uuid())
        .bind(&claim.declaration_id)
        .bind(u32_to_i64(claim.max_attempts))
        .bind(&claim.now)
        .bind(&claim.lease_expires_at)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        if row.is_none() {
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
                status = $2,
                http_status = $3,
                error = $4,
                completed_at = statement_timestamp(),
                next_retry_at = $5,
                lease_expires_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
            WHERE event_id = $1
            ORDER BY addon_id ASC, declaration_id ASC, attempt_number ASC, requested_at ASC, id ASC
            "#
        ))
        .bind(event_id.as_uuid())
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
            WHERE addon_id = $1 AND event_id = $2 AND declaration_id = $3
            ORDER BY attempt_number ASC, requested_at ASC, id ASC
            "#
        ))
        .bind(addon_id.as_uuid())
        .bind(event_id.as_uuid())
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
                p.addon_id::text AS addon_id,
                p.declaration_id,
                p.event_kind,
                p.status AS routing_plan_status,
                p.target AS routing_plan_target,
                p.safe_reason_code AS routing_plan_safe_reason_code
            FROM event_outbox e
            INNER JOIN addon_routing_plans p
                ON p.event_kind = e.kind
                AND p.declaration_kind = $2
            INNER JOIN addon_registrations a
                ON a.id = p.addon_id
                AND a.status = $3
            WHERE e.id = $1
            ORDER BY p.addon_id ASC, p.declaration_id ASC
            "#,
        )
        .bind(event_id.as_uuid())
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

impl PostgresStore {
    async fn get_addon_event_delivery_attempt_or_not_found(
        &self,
        id: AddonEventDeliveryAttemptId,
    ) -> Result<AddonEventDeliveryAttemptRecord> {
        let row = sqlx::query(&format!(
            "{ADDON_EVENT_DELIVERY_ATTEMPT_SELECT} WHERE id = $1"
        ))
        .bind(id.as_uuid())
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

fn row_to_outbox_event(row: PgRow) -> Result<OutboxEventRecord> {
    Ok(OutboxEventRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        kind: DomainEventKind::parse(&row_get::<String>(&row, "kind")?)?,
        subject: event_subject_from_parts(
            row_get::<String>(&row, "subject_kind")?,
            row_get::<String>(&row, "subject_id")?,
        )?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        payload_json: row_get(&row, "payload_json")?,
        status: OutboxEventStatus::parse(&row_get::<String>(&row, "status")?)?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        last_error: row_get(&row, "last_error")?,
        occurred_at: row_get(&row, "occurred_at")?,
        updated_at: row_get(&row, "updated_at")?,
        next_attempt_at: row_get(&row, "next_attempt_at")?,
    })
}

fn row_to_webhook_endpoint(row: PgRow) -> Result<WebhookEndpointRecord> {
    let subscribed_event_kinds =
        serde_json::from_str(&row_get::<String>(&row, "subscribed_event_kinds_json")?)
            .map_err(database_error)?;

    Ok(WebhookEndpointRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        url: row_get(&row, "url")?,
        secret_env: row_get(&row, "secret_env")?,
        subscribed_event_kinds,
        timeout_ms: i64_to_u64(row_get(&row, "timeout_ms")?)?,
        max_attempts: i64_to_u32(row_get(&row, "max_attempts")?)?,
        status: WebhookEndpointStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_webhook_delivery_attempt(row: PgRow) -> Result<WebhookDeliveryAttemptRecord> {
    Ok(WebhookDeliveryAttemptRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        endpoint_id: parse_id(row_get::<String>(&row, "endpoint_id")?)?,
        event_id: parse_id(row_get::<String>(&row, "event_id")?)?,
        attempt_number: i64_to_u32(row_get(&row, "attempt_number")?)?,
        status: WebhookDeliveryStatus::parse(&row_get::<String>(&row, "status")?)?,
        http_status: optional_i64_to_u16(row_get(&row, "http_status")?)?,
        error: row_get(&row, "error")?,
        requested_at: row_get(&row, "requested_at")?,
        completed_at: row_get(&row, "completed_at")?,
        next_retry_at: row_get(&row, "next_retry_at")?,
    })
}

fn row_to_addon_event_delivery_attempt(row: PgRow) -> Result<AddonEventDeliveryAttemptRecord> {
    Ok(AddonEventDeliveryAttemptRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        event_id: parse_id(row_get::<String>(&row, "event_id")?)?,
        declaration_id: row_get(&row, "declaration_id")?,
        attempt_number: i64_to_u32(row_get(&row, "attempt_number")?)?,
        status: AddonEventDeliveryStatus::parse(&row_get::<String>(&row, "status")?)?,
        http_status: optional_i64_to_u16(row_get(&row, "http_status")?)?,
        error: row_get(&row, "error")?,
        requested_at: row_get(&row, "requested_at")?,
        completed_at: row_get(&row, "completed_at")?,
        next_retry_at: row_get(&row, "next_retry_at")?,
        lease_expires_at: row_get(&row, "lease_expires_at")?,
    })
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

fn event_subject_from_parts(kind: String, id: String) -> Result<DomainEventSubject> {
    match kind.as_str() {
        "library" => Ok(DomainEventSubject::Library(parse_id(id)?)),
        "item" => Ok(DomainEventSubject::Item(parse_id(id)?)),
        "source" => Ok(DomainEventSubject::Source(parse_id(id)?)),
        "job" => Ok(DomainEventSubject::Job(parse_id(id)?)),
        "playback_session" => Ok(DomainEventSubject::PlaybackSession(parse_id(id)?)),
        _ => Err(NakoError::Database {
            message: format!("unknown event subject kind stored in PostgreSQL database: {kind}"),
        }),
    }
}
