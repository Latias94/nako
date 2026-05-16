use super::*;

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
        let row = sqlx::query(
            r#"
            SELECT
                id, source_id, kind, request_key, output_path, state,
                failure_category, failure_message, created_at, updated_at,
                started_at, completed_at
            FROM transcode_sessions
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
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
                failure_category, failure_message, created_at, updated_at,
                started_at, completed_at
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
                failure_category, failure_message, created_at, updated_at,
                started_at, completed_at
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
    pub(crate) async fn get_transcode_session_or_not_found(
        &self,
        id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        self.get_transcode_session(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: id.to_string(),
            })
    }
}
