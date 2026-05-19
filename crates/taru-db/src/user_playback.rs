use super::*;

const USER_PLAYBACK_STATE_SELECT: &str = r#"
            SELECT
                principal_id,
                item_id,
                source_id,
                resume_position_ms,
                duration_ms,
                watched,
                watched_at_ms,
                last_played_at_ms,
                updated_at_ms,
                version
            FROM user_playback_states
            "#;

#[async_trait::async_trait]
impl UserPlaybackStateRepository for SqliteStore {
    async fn upsert_user_playback_state(
        &self,
        state: UserPlaybackStateWrite,
    ) -> Result<UserPlaybackState> {
        sqlx::query(
            r#"
            INSERT INTO user_playback_states (
                principal_id,
                item_id,
                source_id,
                resume_position_ms,
                duration_ms,
                watched,
                watched_at_ms,
                last_played_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(principal_id, item_id) DO UPDATE SET
                source_id = excluded.source_id,
                resume_position_ms = excluded.resume_position_ms,
                duration_ms = excluded.duration_ms,
                watched = excluded.watched,
                watched_at_ms = excluded.watched_at_ms,
                last_played_at_ms = excluded.last_played_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                version = user_playback_states.version + 1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(state.principal_id.as_str())
        .bind(state.item_id.to_string())
        .bind(state.source_id.map(|id| id.to_string()))
        .bind(optional_u64_to_i64(state.resume_position_ms)?)
        .bind(optional_u64_to_i64(state.duration_ms)?)
        .bind(bool_to_i64(state.watched))
        .bind(state.watched_at_ms)
        .bind(state.last_played_at_ms)
        .bind(state.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_playback_state(&state.principal_id, state.item_id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "user playback state for principal {} and item {} was not found after upsert",
                    state.principal_id, state.item_id
                ),
            })
    }

    async fn get_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>> {
        let row = sqlx::query(&format!(
            "{USER_PLAYBACK_STATE_SELECT} WHERE principal_id = ?1 AND item_id = ?2"
        ))
        .bind(principal_id.as_str())
        .bind(item_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_user_playback_state).transpose()
    }

    async fn list_continue_watching_states(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaybackState>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {USER_PLAYBACK_STATE_SELECT}
            WHERE principal_id = ?1
              AND watched = 0
              AND resume_position_ms IS NOT NULL
              AND resume_position_ms > 0
            ORDER BY last_played_at_ms DESC, item_id ASC
            LIMIT ?2 OFFSET ?3
            "#
        ))
        .bind(principal_id.as_str())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_user_playback_state).collect()
    }
}

fn row_to_user_playback_state(row: sqlx::sqlite::SqliteRow) -> Result<UserPlaybackState> {
    Ok(UserPlaybackState {
        principal_id: UserPrincipalId::new(row_get::<String>(&row, "principal_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        resume_position_ms: optional_i64_to_u64(row_get(&row, "resume_position_ms")?)?,
        duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
        watched: i64_to_bool(row_get(&row, "watched")?)?,
        watched_at_ms: row_get(&row, "watched_at_ms")?,
        last_played_at_ms: row_get(&row, "last_played_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        version: i64_to_u64(row_get(&row, "version")?)?,
    })
}
