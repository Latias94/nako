use std::path::PathBuf;

use sqlx::postgres::PgRow;

use nako_core::*;

use super::{
    PostgresStore, database_error, optional_i64_to_u64, parse_id, parse_optional_id, row_get,
    u32_to_i64, u64_to_i64,
};

const TRANSCODE_SESSION_SELECT: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                kind,
                request_key,
                output_path,
                state,
                failure_category,
                failure_message,
                runtime_metrics_json::text AS runtime_metrics_json,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM transcode_sessions
            "#;

const TRANSCODE_SESSION_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                kind,
                request_key,
                output_path,
                state,
                failure_category,
                failure_message,
                runtime_metrics_json::text AS runtime_metrics_json,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM transcode_sessions
            WHERE id = $1
            "#;

const PLAYBACK_SESSION_SELECT: &str = r#"
            SELECT
                id::text AS id,
                principal_id,
                source_id::text AS source_id,
                item_id::text AS item_id,
                mode,
                state,
                client_capabilities_json,
                transcode_session_id::text AS transcode_session_id,
                position_ms,
                duration_ms,
                last_heartbeat_at_ms,
                started_at_ms,
                ended_at_ms,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM playback_sessions
            "#;

const PLAYBACK_SESSION_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                principal_id,
                source_id::text AS source_id,
                item_id::text AS item_id,
                mode,
                state,
                client_capabilities_json,
                transcode_session_id::text AS transcode_session_id,
                position_ms,
                duration_ms,
                last_heartbeat_at_ms,
                started_at_ms,
                ended_at_ms,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM playback_sessions
            WHERE id = $1
            "#;

#[async_trait::async_trait]
impl PlaybackSessionRepository for PostgresStore {
    async fn create_playback_session(
        &self,
        session: NewPlaybackSession,
    ) -> Result<PlaybackSessionRecord> {
        sqlx::query(
            r#"
            INSERT INTO playback_sessions (
                id, principal_id, source_id, item_id, mode, state,
                client_capabilities_json, started_at_ms, updated_at_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(session.id.as_uuid())
        .bind(session.principal_id.as_str())
        .bind(session.source_id.as_uuid())
        .bind(session.item_id.as_uuid())
        .bind(session.mode.as_str())
        .bind(session.state.as_str())
        .bind(session.client_capabilities_json)
        .bind(session.started_at_ms)
        .bind(session.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_playback_session_or_not_found(session.id).await
    }

    async fn get_playback_session(
        &self,
        id: PlaybackSessionId,
    ) -> Result<Option<PlaybackSessionRecord>> {
        let row = sqlx::query(PLAYBACK_SESSION_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_playback_session).transpose()
    }

    async fn list_playback_sessions(
        &self,
        filter: PlaybackSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackSessionRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {PLAYBACK_SESSION_SELECT}
            WHERE ($1::text IS NULL OR principal_id = $1)
              AND ($2::uuid IS NULL OR source_id = $2)
              AND ($3::text IS NULL OR state = $3)
            ORDER BY updated_at DESC, id DESC
            LIMIT $4 OFFSET $5
            "#
        ))
        .bind(filter.principal_id.map(|id| id.to_string()))
        .bind(filter.source_id.map(|id| id.as_uuid()))
        .bind(filter.state.map(PlaybackSessionState::as_str))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_playback_session).collect()
    }

    async fn find_latest_playback_session_by_transcode_session(
        &self,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<Option<PlaybackSessionRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {PLAYBACK_SESSION_SELECT}
            WHERE transcode_session_id = $1
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#
        ))
        .bind(transcode_session_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_playback_session).transpose()
    }

    async fn link_playback_session_transcode(
        &self,
        id: PlaybackSessionId,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<PlaybackSessionRecord> {
        sqlx::query(
            r#"
            UPDATE playback_sessions
            SET
                transcode_session_id = $2,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(transcode_session_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_playback_session_or_not_found(id).await
    }

    async fn record_playback_session_heartbeat(
        &self,
        heartbeat: PlaybackSessionHeartbeat,
    ) -> Result<Option<PlaybackSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE playback_sessions
            SET
                state = $2,
                position_ms = $3,
                duration_ms = $4,
                last_heartbeat_at_ms = $5,
                updated_at_ms = $5,
                ended_at_ms = CASE
                    WHEN $2 IN ('cancelled', 'ended', 'failed')
                    THEN COALESCE(ended_at_ms, $5)
                    ELSE ended_at_ms
                END,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND state NOT IN ('cancelled', 'ended', 'failed')
            "#,
        )
        .bind(heartbeat.id.as_uuid())
        .bind(heartbeat.state.as_str())
        .bind(heartbeat.position_ms.map(u64_to_i64).transpose()?)
        .bind(heartbeat.duration_ms.map(u64_to_i64).transpose()?)
        .bind(heartbeat.heartbeat_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some(
            self.get_playback_session_or_not_found(heartbeat.id).await?,
        ))
    }

    async fn set_playback_session_state(
        &self,
        id: PlaybackSessionId,
        state: PlaybackSessionState,
        ended_at_ms: Option<i64>,
    ) -> Result<Option<PlaybackSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE playback_sessions
            SET
                state = $2,
                updated_at_ms = COALESCE($3, updated_at_ms),
                ended_at_ms = CASE
                    WHEN $2 IN ('cancelled', 'ended', 'failed')
                    THEN COALESCE(ended_at_ms, $3)
                    ELSE ended_at_ms
                END,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND state NOT IN ('cancelled', 'ended', 'failed')
            "#,
        )
        .bind(id.as_uuid())
        .bind(state.as_str())
        .bind(ended_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some(self.get_playback_session_or_not_found(id).await?))
    }
}

#[async_trait::async_trait]
impl TranscodeSessionRepository for PostgresStore {
    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord> {
        sqlx::query(
            r#"
            INSERT INTO transcode_sessions (
                id, source_id, kind, request_key, output_path, state
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(session.id.as_uuid())
        .bind(session.source_id.as_uuid())
        .bind(session.kind.as_str())
        .bind(&session.request_key)
        .bind(session.output_path.display().to_string())
        .bind(session.state.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_transcode_session_or_not_found(session.id).await
    }

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let row = sqlx::query(TRANSCODE_SESSION_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
    }

    async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {TRANSCODE_SESSION_SELECT}
            WHERE ($1::uuid IS NULL OR source_id = $1)
              AND ($2::text IS NULL OR kind = $2)
              AND ($3::text IS NULL OR state = $3)
            ORDER BY updated_at DESC, id DESC
            LIMIT $4 OFFSET $5
            "#
        ))
        .bind(filter.source_id.map(|id| id.as_uuid()))
        .bind(filter.kind.map(TranscodeSessionKind::as_str))
        .bind(filter.state.map(TranscodeSessionState::as_str))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_transcode_session).collect()
    }

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {TRANSCODE_SESSION_SELECT}
            WHERE source_id = $1 AND kind = $2 AND request_key = $3
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#
        ))
        .bind(source_id.as_uuid())
        .bind(kind.as_str())
        .bind(request_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
    }

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {TRANSCODE_SESSION_SELECT}
            WHERE source_id = $1
                AND kind = $2
                AND request_key = $3
                AND state IN ('planned', 'starting', 'running', 'cancel_requested')
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#
        ))
        .bind(source_id.as_uuid())
        .bind(kind.as_str())
        .bind(request_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
    }

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord> {
        sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                state = $2,
                failure_category = $3,
                failure_message = $4,
                started_at = CASE
                    WHEN started_at IS NULL AND $2 IN ('starting', 'running')
                    THEN statement_timestamp()
                    ELSE started_at
                END,
                completed_at = CASE
                    WHEN $2 IN ('cancelled', 'failed', 'finished')
                    THEN statement_timestamp()
                    ELSE completed_at
                END,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(state.as_str())
        .bind(failure_category.map(TranscodeFailureCategory::as_str))
        .bind(failure_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_transcode_session_or_not_found(id).await
    }

    async fn update_transcode_session_runtime_metrics(
        &self,
        id: TranscodeSessionId,
        metrics: TranscodeSessionRuntimeMetrics,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                runtime_metrics_json = $2::jsonb,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(serialize_transcode_runtime_metrics_json(&metrics)?)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_transcode_session(id).await
    }

    async fn request_transcode_session_cancellation(
        &self,
        id: TranscodeSessionId,
        failure_message: String,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                state = 'cancel_requested',
                failure_category = 'cancelled',
                failure_message = $2,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND state IN ('planned', 'starting', 'running', 'cancel_requested')
            "#,
        )
        .bind(id.as_uuid())
        .bind(failure_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some(self.get_transcode_session_or_not_found(id).await?))
    }

    async fn fail_stale_transcode_sessions(
        &self,
        failure_category: TranscodeFailureCategory,
        failure_message: String,
    ) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                state = 'failed',
                failure_category = $1,
                failure_message = $2,
                completed_at = COALESCE(completed_at, statement_timestamp()),
                updated_at = statement_timestamp()
            WHERE state IN ('planned', 'starting', 'running', 'cancel_requested')
            "#,
        )
        .bind(failure_category.as_str())
        .bind(failure_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }
}

impl PostgresStore {
    async fn get_playback_session_or_not_found(
        &self,
        id: PlaybackSessionId,
    ) -> Result<PlaybackSessionRecord> {
        self.get_playback_session(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "playback_session",
                id: id.to_string(),
            })
    }

    async fn get_transcode_session_or_not_found(
        &self,
        id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        self.get_transcode_session(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "transcode_session",
                id: id.to_string(),
            })
    }
}
fn row_to_playback_session(row: PgRow) -> Result<PlaybackSessionRecord> {
    Ok(PlaybackSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        principal_id: UserPrincipalId::new(row_get::<String>(&row, "principal_id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        mode: parse_playback_session_mode(row_get(&row, "mode")?)?,
        state: parse_playback_session_state(row_get(&row, "state")?)?,
        client_capabilities_json: row_get(&row, "client_capabilities_json")?,
        transcode_session_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "transcode_session_id",
        )?)?,
        position_ms: optional_i64_to_u64(row_get(&row, "position_ms")?)?,
        duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
        last_heartbeat_at_ms: row_get(&row, "last_heartbeat_at_ms")?,
        started_at_ms: row_get(&row, "started_at_ms")?,
        ended_at_ms: row_get(&row, "ended_at_ms")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_transcode_session(row: PgRow) -> Result<TranscodeSessionRecord> {
    Ok(TranscodeSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        kind: parse_transcode_session_kind(row_get(&row, "kind")?)?,
        request_key: row_get(&row, "request_key")?,
        output_path: PathBuf::from(row_get::<String>(&row, "output_path")?),
        state: parse_transcode_session_state(row_get(&row, "state")?)?,
        failure_category: parse_transcode_failure_category(row_get(&row, "failure_category")?)?,
        failure_message: row_get(&row, "failure_message")?,
        runtime_metrics: deserialize_transcode_runtime_metrics_json(row_get(
            &row,
            "runtime_metrics_json",
        )?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
    })
}

fn serialize_transcode_runtime_metrics_json(
    metrics: &TranscodeSessionRuntimeMetrics,
) -> Result<String> {
    serde_json::to_string(metrics).map_err(database_error)
}

fn deserialize_transcode_runtime_metrics_json(
    value: Option<String>,
) -> Result<TranscodeSessionRuntimeMetrics> {
    match value {
        Some(value) if !value.trim().is_empty() => {
            serde_json::from_str(&value).map_err(database_error)
        }
        _ => Ok(TranscodeSessionRuntimeMetrics::default()),
    }
}

fn parse_transcode_session_kind(value: String) -> Result<TranscodeSessionKind> {
    TranscodeSessionKind::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown transcode session kind stored in PostgreSQL database: {value}"),
    })
}

fn parse_playback_session_mode(value: String) -> Result<PlaybackSessionMode> {
    PlaybackSessionMode::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown playback session mode stored in PostgreSQL database: {value}"),
    })
}

fn parse_playback_session_state(value: String) -> Result<PlaybackSessionState> {
    PlaybackSessionState::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown playback session state stored in PostgreSQL database: {value}"),
    })
}

fn parse_transcode_session_state(value: String) -> Result<TranscodeSessionState> {
    TranscodeSessionState::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown transcode session state stored in PostgreSQL database: {value}"),
    })
}

fn parse_transcode_failure_category(
    value: Option<String>,
) -> Result<Option<TranscodeFailureCategory>> {
    value
        .map(|value| {
            TranscodeFailureCategory::parse(&value).ok_or_else(|| NakoError::Database {
                message: format!(
                    "unknown transcode failure category stored in PostgreSQL database: {value}"
                ),
            })
        })
        .transpose()
}
