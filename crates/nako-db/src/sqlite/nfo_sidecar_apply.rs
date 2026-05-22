use sqlx::sqlite::SqliteRow;

use super::{SqliteStore, codec::*};
use nako_core::*;

#[async_trait::async_trait]
impl NfoSidecarApplyRepository for SqliteStore {
    async fn upsert_nfo_sidecar_apply(
        &self,
        apply: NewNfoSidecarApply,
    ) -> Result<NfoSidecarApplyRecord> {
        sqlx::query(
            r#"
            INSERT INTO nfo_sidecar_applies (
                id, target_library_id, media_item_id, media_source_id, requested_by,
                idempotency_key, operation_kind, sidecar_locator,
                accepted_preview_json, accepted_warnings_json, policy_version, state,
                outcome_json, safe_error_code, safe_message, created_at_ms, updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(id) DO UPDATE SET
                target_library_id = excluded.target_library_id,
                media_item_id = excluded.media_item_id,
                media_source_id = excluded.media_source_id,
                requested_by = excluded.requested_by,
                idempotency_key = excluded.idempotency_key,
                operation_kind = excluded.operation_kind,
                sidecar_locator = excluded.sidecar_locator,
                accepted_preview_json = excluded.accepted_preview_json,
                accepted_warnings_json = excluded.accepted_warnings_json,
                policy_version = excluded.policy_version,
                state = excluded.state,
                outcome_json = excluded.outcome_json,
                safe_error_code = excluded.safe_error_code,
                safe_message = excluded.safe_message,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(apply.id.to_string())
        .bind(apply.target_library_id.to_string())
        .bind(apply.media_item_id.to_string())
        .bind(apply.media_source_id.map(|id| id.to_string()))
        .bind(apply.requested_by.to_string())
        .bind(&apply.idempotency_key)
        .bind(apply.operation_kind.as_str())
        .bind(&apply.sidecar_locator)
        .bind(&apply.accepted_preview_json)
        .bind(&apply.accepted_warnings_json)
        .bind(&apply.policy_version)
        .bind(apply.state.as_str())
        .bind(&apply.outcome_json)
        .bind(&apply.safe_error_code)
        .bind(&apply.safe_message)
        .bind(apply.created_at_ms)
        .bind(apply.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_nfo_sidecar_apply(apply.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "nfo_sidecar_apply",
                id: apply.id.to_string(),
            })
    }

    async fn get_nfo_sidecar_apply(
        &self,
        id: NfoSidecarApplyId,
    ) -> Result<Option<NfoSidecarApplyRecord>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM nfo_sidecar_applies
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_nfo_sidecar_apply).transpose()
    }

    async fn find_nfo_sidecar_apply_by_idempotency_key(
        &self,
        target_library_id: LibraryId,
        idempotency_key: &str,
    ) -> Result<Option<NfoSidecarApplyRecord>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM nfo_sidecar_applies
            WHERE target_library_id = ?1
              AND idempotency_key = ?2
            "#,
        )
        .bind(target_library_id.to_string())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_nfo_sidecar_apply).transpose()
    }

    async fn list_nfo_sidecar_applies_for_item(
        &self,
        media_item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<NfoSidecarApplyRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM nfo_sidecar_applies
            WHERE media_item_id = ?1
            ORDER BY updated_at_ms DESC, id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(media_item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_nfo_sidecar_apply).collect()
    }

    async fn set_nfo_sidecar_apply_state(
        &self,
        id: NfoSidecarApplyId,
        state: NfoSidecarApplyState,
        updated_at_ms: i64,
        outcome_json: Option<String>,
        safe_error_code: Option<String>,
        safe_message: Option<String>,
    ) -> Result<Option<NfoSidecarApplyRecord>> {
        sqlx::query(
            r#"
            UPDATE nfo_sidecar_applies
            SET state = ?2,
                updated_at_ms = ?3,
                outcome_json = ?4,
                safe_error_code = ?5,
                safe_message = ?6,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(state.as_str())
        .bind(updated_at_ms)
        .bind(outcome_json)
        .bind(safe_error_code)
        .bind(safe_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_nfo_sidecar_apply(id).await
    }
}

fn row_to_nfo_sidecar_apply(row: SqliteRow) -> Result<NfoSidecarApplyRecord> {
    Ok(NfoSidecarApplyRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        target_library_id: parse_id(row_get::<String>(&row, "target_library_id")?)?,
        media_item_id: parse_id(row_get::<String>(&row, "media_item_id")?)?,
        media_source_id: parse_optional_id(row_get::<Option<String>>(&row, "media_source_id")?)?,
        requested_by: UserPrincipalId::new(row_get::<String>(&row, "requested_by")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        operation_kind: NfoSidecarApplyOperationKind::parse(&row_get::<String>(
            &row,
            "operation_kind",
        )?)?,
        sidecar_locator: row_get(&row, "sidecar_locator")?,
        accepted_preview_json: row_get(&row, "accepted_preview_json")?,
        accepted_warnings_json: row_get(&row, "accepted_warnings_json")?,
        policy_version: row_get(&row, "policy_version")?,
        state: NfoSidecarApplyState::parse(&row_get::<String>(&row, "state")?)?,
        outcome_json: row_get(&row, "outcome_json")?,
        safe_error_code: row_get(&row, "safe_error_code")?,
        safe_message: row_get(&row, "safe_message")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}
