use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::QueryBuilder;

const TRANSCODE_SESSION_SELECT: &str = r#"
            SELECT
                id, source_id, kind, request_key, output_path, state,
                failure_category, failure_message, runtime_metrics_json,
                created_at, updated_at, started_at, completed_at
            FROM transcode_sessions
            "#;

const TRANSCODE_SESSION_SELECT_BY_ID: &str = r#"
            SELECT
                id, source_id, kind, request_key, output_path, state,
                failure_category, failure_message, runtime_metrics_json,
                created_at, updated_at, started_at, completed_at
            FROM transcode_sessions
            WHERE id = ?1
            "#;

const PLAYBACK_SESSION_SELECT: &str = r#"
            SELECT
                id, principal_id, source_id, item_id, mode, state,
                client_capabilities_json, transcode_session_id,
                position_ms, duration_ms, last_heartbeat_at_ms,
                started_at_ms, ended_at_ms, created_at, updated_at
            FROM playback_sessions
            "#;

const PLAYBACK_SESSION_SELECT_BY_ID: &str = r#"
            SELECT
                id, principal_id, source_id, item_id, mode, state,
                client_capabilities_json, transcode_session_id,
                position_ms, duration_ms, last_heartbeat_at_ms,
                started_at_ms, ended_at_ms, created_at, updated_at
            FROM playback_sessions
            WHERE id = ?1
            "#;

#[async_trait::async_trait]
impl PlaybackSessionRepository for SqliteStore {
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
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.principal_id.as_str())
        .bind(session.source_id.to_string())
        .bind(session.item_id.to_string())
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
            .bind(id.to_string())
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
        let mut query = QueryBuilder::new(PLAYBACK_SESSION_SELECT);
        query.push(" WHERE 1 = 1");

        if let Some(principal_id) = filter.principal_id {
            query.push(" AND principal_id = ");
            query.push_bind(principal_id.as_str().to_owned());
        }
        if let Some(source_id) = filter.source_id {
            query.push(" AND source_id = ");
            query.push_bind(source_id.to_string());
        }
        if let Some(state) = filter.state {
            query.push(" AND state = ");
            query.push_bind(state.as_str());
        }

        query.push(" ORDER BY updated_at DESC, id DESC LIMIT ");
        query.push_bind(u32_to_i64(page.limit));
        query.push(" OFFSET ");
        query.push_bind(u64_to_i64(page.offset)?);

        let rows = query
            .build()
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
            WHERE transcode_session_id = ?1
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#
        ))
        .bind(transcode_session_id.to_string())
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
                transcode_session_id = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(transcode_session_id.to_string())
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
                state = ?2,
                position_ms = ?3,
                duration_ms = ?4,
                last_heartbeat_at_ms = ?5,
                updated_at_ms = ?5,
                ended_at_ms = CASE
                    WHEN ?2 IN ('cancelled', 'ended', 'failed')
                    THEN COALESCE(ended_at_ms, ?5)
                    ELSE ended_at_ms
                END,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND state NOT IN ('cancelled', 'ended', 'failed')
            "#,
        )
        .bind(heartbeat.id.to_string())
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
                state = ?2,
                updated_at_ms = COALESCE(?3, updated_at_ms),
                ended_at_ms = CASE
                    WHEN ?2 IN ('cancelled', 'ended', 'failed')
                    THEN COALESCE(ended_at_ms, ?3)
                    ELSE ended_at_ms
                END,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND state NOT IN ('cancelled', 'ended', 'failed')
            "#,
        )
        .bind(id.to_string())
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
impl TranscodeSessionRepository for SqliteStore {
    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord> {
        sqlx::query(
            r#"
            INSERT INTO transcode_sessions (
                id, source_id, kind, request_key, output_path, state
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.source_id.to_string())
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
            .bind(id.to_string())
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
        let mut query = QueryBuilder::new(TRANSCODE_SESSION_SELECT);
        query.push(" WHERE 1 = 1");

        if let Some(source_id) = filter.source_id {
            query.push(" AND source_id = ");
            query.push_bind(source_id.to_string());
        }
        if let Some(kind) = filter.kind {
            query.push(" AND kind = ");
            query.push_bind(kind.as_str());
        }
        if let Some(state) = filter.state {
            query.push(" AND state = ");
            query.push_bind(state.as_str());
        }

        query.push(" ORDER BY updated_at DESC, id DESC LIMIT ");
        query.push_bind(u32_to_i64(page.limit));
        query.push(" OFFSET ");
        query.push_bind(u64_to_i64(page.offset)?);

        let rows = query
            .build()
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
        let row = sqlx::query(
            r#"
            SELECT
                id, source_id, kind, request_key, output_path, state,
                failure_category, failure_message, runtime_metrics_json,
                created_at, updated_at, started_at, completed_at
            FROM transcode_sessions
            WHERE source_id = ?1 AND kind = ?2 AND request_key = ?3
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(source_id.to_string())
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
        let row = sqlx::query(
            r#"
            SELECT
                id, source_id, kind, request_key, output_path, state,
                failure_category, failure_message, runtime_metrics_json,
                created_at, updated_at, started_at, completed_at
            FROM transcode_sessions
            WHERE source_id = ?1
                AND kind = ?2
                AND request_key = ?3
                AND state IN ('planned', 'starting', 'running', 'cancel_requested')
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(source_id.to_string())
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
                state = ?2,
                failure_category = ?3,
                failure_message = ?4,
                started_at = CASE
                    WHEN started_at IS NULL AND ?2 IN ('starting', 'running')
                    THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    ELSE started_at
                END,
                completed_at = CASE
                    WHEN ?2 IN ('cancelled', 'failed', 'finished')
                    THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    ELSE completed_at
                END,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
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
                runtime_metrics_json = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
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
                failure_message = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND state IN ('planned', 'starting', 'running', 'cancel_requested')
            "#,
        )
        .bind(id.to_string())
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
                failure_category = ?1,
                failure_message = ?2,
                completed_at = COALESCE(completed_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
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

impl SqliteStore {
    pub(crate) async fn get_playback_session_or_not_found(
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

    pub(crate) async fn get_transcode_session_or_not_found(
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

fn serialize_transcode_runtime_metrics_json(
    metrics: &TranscodeSessionRuntimeMetrics,
) -> Result<String> {
    serde_json::to_string(metrics).map_err(database_error)
}
