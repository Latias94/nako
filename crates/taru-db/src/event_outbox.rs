use super::*;
use sqlx::QueryBuilder;

const OUTBOX_EVENT_SELECT: &str = r#"
            SELECT
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                occurred_at,
                updated_at,
                next_attempt_at
            FROM event_outbox
            "#;

const OUTBOX_EVENT_SELECT_BY_ID: &str = r#"
            SELECT
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                occurred_at,
                updated_at,
                next_attempt_at
            FROM event_outbox
            WHERE id = ?1
            "#;

#[async_trait::async_trait]
impl EventOutboxRepository for SqliteStore {
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
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(kind, idempotency_key) DO NOTHING
            "#,
        )
        .bind(event.id.to_string())
        .bind(event.kind.as_str())
        .bind(subject_kind)
        .bind(subject_id)
        .bind(event.library_id.map(|id| id.to_string()))
        .bind(event.source_id.map(|id| id.to_string()))
        .bind(&event.idempotency_key)
        .bind(&event.payload_json)
        .bind(OutboxEventStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_outbox_event_by_idempotency_key(event.kind, &event.idempotency_key)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "outbox event was not found after enqueue for key {}",
                    event.idempotency_key
                ),
            })
    }

    async fn get_outbox_event(&self, id: EventId) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(OUTBOX_EVENT_SELECT_BY_ID)
            .bind(id.to_string())
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
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                occurred_at,
                updated_at,
                next_attempt_at
            FROM event_outbox
            WHERE kind = ?1 AND idempotency_key = ?2
            "#,
        )
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
        let mut query = QueryBuilder::new(OUTBOX_EVENT_SELECT);
        query.push(" WHERE 1 = 1");

        if let Some(kind) = filter.kind {
            query.push(" AND kind = ");
            query.push_bind(kind.as_str());
        }
        if let Some(status) = filter.status {
            query.push(" AND status = ");
            query.push_bind(status.as_str());
        }
        if let Some(library_id) = filter.library_id {
            query.push(" AND library_id = ");
            query.push_bind(library_id.to_string());
        }
        if let Some(source_id) = filter.source_id {
            query.push(" AND source_id = ");
            query.push_bind(source_id.to_string());
        }

        query.push(" ORDER BY occurred_at DESC, id DESC LIMIT ");
        query.push_bind(u32_to_i64(page.limit));
        query.push(" OFFSET ");
        query.push_bind(u64_to_i64(page.offset)?);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(row_to_outbox_event).collect()
    }
}
