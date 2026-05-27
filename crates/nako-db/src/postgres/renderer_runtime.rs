use sqlx::postgres::PgRow;

use nako_core::*;

use super::{
    PostgresStore, database_error, optional_i64_to_u64, parse_id, parse_optional_id, row_get,
    u32_to_i64, u64_to_i64,
};

const RENDERER_SESSION_SELECT: &str = r#"
            SELECT
                id::text AS id,
                owner_principal_id,
                target_kind,
                display_name,
                network_scope,
                transport_auth,
                media_capabilities_json::text AS media_capabilities_json,
                control_capabilities_json::text AS control_capabilities_json,
                state,
                active_playback_session_id::text AS active_playback_session_id,
                last_seen_at_ms,
                expires_at_ms,
                created_at_ms,
                updated_at_ms,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM renderer_sessions
            "#;

const RENDERER_SESSION_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                owner_principal_id,
                target_kind,
                display_name,
                network_scope,
                transport_auth,
                media_capabilities_json::text AS media_capabilities_json,
                control_capabilities_json::text AS control_capabilities_json,
                state,
                active_playback_session_id::text AS active_playback_session_id,
                last_seen_at_ms,
                expires_at_ms,
                created_at_ms,
                updated_at_ms,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM renderer_sessions
            WHERE id = $1
            "#;

const RENDERER_COMMAND_SELECT: &str = r#"
            SELECT
                id::text AS id,
                renderer_session_id::text AS renderer_session_id,
                controlling_principal_id,
                command,
                state,
                item_id::text AS item_id,
                source_id::text AS source_id,
                playback_session_id::text AS playback_session_id,
                position_ms,
                volume_percent,
                payload_json::text AS payload_json,
                created_at_ms,
                updated_at_ms,
                delivered_at_ms,
                completed_at_ms,
                failure_message,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM renderer_commands
            "#;

const RENDERER_COMMAND_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                renderer_session_id::text AS renderer_session_id,
                controlling_principal_id,
                command,
                state,
                item_id::text AS item_id,
                source_id::text AS source_id,
                playback_session_id::text AS playback_session_id,
                position_ms,
                volume_percent,
                payload_json::text AS payload_json,
                created_at_ms,
                updated_at_ms,
                delivered_at_ms,
                completed_at_ms,
                failure_message,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM renderer_commands
            WHERE id = $1
            "#;

const RENDERER_COMMAND_RETURNING: &str = r#"
            RETURNING
                id::text AS id,
                renderer_session_id::text AS renderer_session_id,
                controlling_principal_id,
                command,
                state,
                item_id::text AS item_id,
                source_id::text AS source_id,
                playback_session_id::text AS playback_session_id,
                position_ms,
                volume_percent,
                payload_json::text AS payload_json,
                created_at_ms,
                updated_at_ms,
                delivered_at_ms,
                completed_at_ms,
                failure_message,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            "#;

#[async_trait::async_trait]
impl RendererSessionRepository for PostgresStore {
    async fn upsert_renderer_session(
        &self,
        session: NewRendererSession,
    ) -> Result<RendererSessionRecord> {
        let control_capabilities_json = control_capabilities_json(&session.control_capabilities)?;
        sqlx::query(
            r#"
            INSERT INTO renderer_sessions (
                id, owner_principal_id, target_kind, display_name,
                network_scope, transport_auth, media_capabilities_json,
                control_capabilities_json, state, last_seen_at_ms,
                expires_at_ms, created_at_ms, updated_at_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8::jsonb, $9, $10, $11, $12, $13)
            ON CONFLICT(id) DO UPDATE SET
                owner_principal_id = excluded.owner_principal_id,
                target_kind = excluded.target_kind,
                display_name = excluded.display_name,
                network_scope = excluded.network_scope,
                transport_auth = excluded.transport_auth,
                media_capabilities_json = excluded.media_capabilities_json,
                control_capabilities_json = excluded.control_capabilities_json,
                state = excluded.state,
                last_seen_at_ms = excluded.last_seen_at_ms,
                expires_at_ms = excluded.expires_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(session.id.as_uuid())
        .bind(session.owner_principal_id.as_str())
        .bind(session.target_kind.as_str())
        .bind(session.display_name)
        .bind(session.network_scope.as_str())
        .bind(session.transport_auth.as_str())
        .bind(session.media_capabilities_json)
        .bind(control_capabilities_json)
        .bind(session.state.as_str())
        .bind(session.last_seen_at_ms)
        .bind(session.expires_at_ms)
        .bind(session.created_at_ms)
        .bind(session.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_renderer_session_or_not_found(session.id).await
    }

    async fn get_renderer_session(
        &self,
        id: RendererSessionId,
    ) -> Result<Option<RendererSessionRecord>> {
        let row = sqlx::query(RENDERER_SESSION_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_renderer_session).transpose()
    }

    async fn list_renderer_sessions(
        &self,
        filter: RendererSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<RendererSessionRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {RENDERER_SESSION_SELECT}
            WHERE ($1::text IS NULL OR owner_principal_id = $1)
              AND ($2::text IS NULL OR state = $2)
            ORDER BY updated_at_ms DESC, id DESC
            LIMIT $3 OFFSET $4
            "#
        ))
        .bind(filter.owner_principal_id.map(|id| id.to_string()))
        .bind(filter.state.map(RendererSessionState::as_str))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_renderer_session).collect()
    }

    async fn record_renderer_session_heartbeat(
        &self,
        heartbeat: RendererSessionHeartbeat,
    ) -> Result<Option<RendererSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE renderer_sessions
            SET
                state = $2,
                last_seen_at_ms = $3,
                expires_at_ms = $4,
                updated_at_ms = $3,
                updated_at = statement_timestamp()
            WHERE id = $1 AND state != 'revoked'
            "#,
        )
        .bind(heartbeat.id.as_uuid())
        .bind(heartbeat.state.as_str())
        .bind(heartbeat.last_seen_at_ms)
        .bind(heartbeat.expires_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_renderer_session(heartbeat.id).await
    }

    async fn attach_renderer_playback_session(
        &self,
        id: RendererSessionId,
        playback_session_id: Option<PlaybackSessionId>,
        updated_at_ms: i64,
    ) -> Result<Option<RendererSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE renderer_sessions
            SET
                active_playback_session_id = $2,
                updated_at_ms = $3,
                updated_at = statement_timestamp()
            WHERE id = $1 AND state != 'revoked'
            "#,
        )
        .bind(id.as_uuid())
        .bind(playback_session_id.map(|id| id.as_uuid()))
        .bind(updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_renderer_session(id).await
    }

    async fn create_renderer_command(
        &self,
        command: NewRendererCommand,
    ) -> Result<RendererCommandRecord> {
        sqlx::query(
            r#"
            INSERT INTO renderer_commands (
                id, renderer_session_id, controlling_principal_id, command, state,
                item_id, source_id, playback_session_id, position_ms, volume_percent,
                payload_json, created_at_ms, updated_at_ms
            )
            VALUES ($1, $2, $3, $4, 'queued', $5, $6, $7, $8, $9, $10::jsonb, $11, $12)
            "#,
        )
        .bind(command.id.as_uuid())
        .bind(command.renderer_session_id.as_uuid())
        .bind(command.controlling_principal_id.as_str())
        .bind(command.command.as_str())
        .bind(command.item_id.map(|id| id.as_uuid()))
        .bind(command.source_id.map(|id| id.as_uuid()))
        .bind(command.playback_session_id.map(|id| id.as_uuid()))
        .bind(command.position_ms.map(u64_to_i64).transpose()?)
        .bind(command.volume_percent.map(i64::from))
        .bind(command.payload_json)
        .bind(command.created_at_ms)
        .bind(command.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_renderer_command_or_not_found(command.id).await
    }

    async fn get_renderer_command(
        &self,
        id: RendererCommandId,
    ) -> Result<Option<RendererCommandRecord>> {
        let row = sqlx::query(RENDERER_COMMAND_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_renderer_command).transpose()
    }

    async fn list_renderer_commands(
        &self,
        filter: RendererCommandListFilter,
        page: PageRequest,
    ) -> Result<Vec<RendererCommandRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {RENDERER_COMMAND_SELECT}
            WHERE ($1::uuid IS NULL OR renderer_session_id = $1)
              AND ($2::text IS NULL OR state = $2)
            ORDER BY created_at_ms ASC, id ASC
            LIMIT $3 OFFSET $4
            "#
        ))
        .bind(filter.renderer_session_id.map(|id| id.as_uuid()))
        .bind(filter.state.map(RendererCommandState::as_str))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_renderer_command).collect()
    }

    async fn claim_next_renderer_command(
        &self,
        renderer_session_id: RendererSessionId,
        delivered_at_ms: i64,
    ) -> Result<Option<RendererCommandRecord>> {
        let row = sqlx::query(&format!(
            r#"
            UPDATE renderer_commands
            SET
                state = 'delivered',
                delivered_at_ms = $2,
                updated_at_ms = $2,
                updated_at = statement_timestamp()
            WHERE id = (
                SELECT id
                FROM renderer_commands
                WHERE renderer_session_id = $1 AND state = 'queued'
                ORDER BY created_at_ms ASC, id ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            {RENDERER_COMMAND_RETURNING}
            "#
        ))
        .bind(renderer_session_id.as_uuid())
        .bind(delivered_at_ms)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_renderer_command).transpose()
    }

    async fn complete_renderer_command(
        &self,
        completion: RendererCommandCompletion,
    ) -> Result<Option<RendererCommandRecord>> {
        if !completion.state.is_terminal() {
            return Err(NakoError::InvalidInput {
                message: "renderer command completion state must be terminal".to_owned(),
            });
        }

        let row = sqlx::query(&format!(
            r#"
            UPDATE renderer_commands
            SET
                state = $2,
                completed_at_ms = $3,
                failure_message = $4,
                updated_at_ms = $3,
                updated_at = statement_timestamp()
            WHERE id = $1 AND state IN ('queued', 'delivered')
            {RENDERER_COMMAND_RETURNING}
            "#
        ))
        .bind(completion.id.as_uuid())
        .bind(completion.state.as_str())
        .bind(completion.completed_at_ms)
        .bind(completion.failure_message)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_renderer_command).transpose()
    }
}

impl PostgresStore {
    async fn get_renderer_session_or_not_found(
        &self,
        id: RendererSessionId,
    ) -> Result<RendererSessionRecord> {
        self.get_renderer_session(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_session",
                id: id.to_string(),
            })
    }

    async fn get_renderer_command_or_not_found(
        &self,
        id: RendererCommandId,
    ) -> Result<RendererCommandRecord> {
        self.get_renderer_command(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_command",
                id: id.to_string(),
            })
    }
}

fn row_to_renderer_session(row: PgRow) -> Result<RendererSessionRecord> {
    Ok(RendererSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        owner_principal_id: UserPrincipalId::new(row_get::<String>(&row, "owner_principal_id")?)?,
        target_kind: parse_playback_target_kind(row_get(&row, "target_kind")?)?,
        display_name: row_get(&row, "display_name")?,
        network_scope: parse_playback_target_network_scope(row_get(&row, "network_scope")?)?,
        transport_auth: parse_playback_target_transport_auth(row_get(&row, "transport_auth")?)?,
        media_capabilities_json: row_get(&row, "media_capabilities_json")?,
        control_capabilities: parse_control_capabilities(row_get(
            &row,
            "control_capabilities_json",
        )?)?,
        state: parse_renderer_session_state(row_get(&row, "state")?)?,
        active_playback_session_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "active_playback_session_id",
        )?)?,
        last_seen_at_ms: row_get(&row, "last_seen_at_ms")?,
        expires_at_ms: row_get(&row, "expires_at_ms")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_renderer_command(row: PgRow) -> Result<RendererCommandRecord> {
    Ok(RendererCommandRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        renderer_session_id: parse_id(row_get::<String>(&row, "renderer_session_id")?)?,
        controlling_principal_id: UserPrincipalId::new(row_get::<String>(
            &row,
            "controlling_principal_id",
        )?)?,
        command: parse_renderer_control_command(row_get(&row, "command")?)?,
        state: parse_renderer_command_state(row_get(&row, "state")?)?,
        item_id: parse_optional_id(row_get::<Option<String>>(&row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        playback_session_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "playback_session_id",
        )?)?,
        position_ms: optional_i64_to_u64(row_get(&row, "position_ms")?)?,
        volume_percent: optional_i64_to_u8(row_get(&row, "volume_percent")?)?,
        payload_json: row_get(&row, "payload_json")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        delivered_at_ms: row_get(&row, "delivered_at_ms")?,
        completed_at_ms: row_get(&row, "completed_at_ms")?,
        failure_message: row_get(&row, "failure_message")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn control_capabilities_json(value: &RendererControlCapabilities) -> Result<String> {
    serde_json::to_string(value).map_err(database_error)
}

fn parse_control_capabilities(value: String) -> Result<RendererControlCapabilities> {
    serde_json::from_str(&value).map_err(database_error)
}

fn optional_i64_to_u8(value: Option<i64>) -> Result<Option<u8>> {
    value
        .map(|value| {
            u8::try_from(value).map_err(|err| NakoError::Database {
                message: format!("PostgreSQL bigint cannot be converted to u8: {err}"),
            })
        })
        .transpose()
}

fn parse_playback_target_kind(value: String) -> Result<PlaybackTargetKind> {
    PlaybackTargetKind::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown playback target kind stored in PostgreSQL database: {value}"),
    })
}

fn parse_playback_target_network_scope(value: String) -> Result<PlaybackTargetNetworkScope> {
    PlaybackTargetNetworkScope::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!(
            "unknown playback target network scope stored in PostgreSQL database: {value}"
        ),
    })
}

fn parse_playback_target_transport_auth(value: String) -> Result<PlaybackTargetTransportAuth> {
    PlaybackTargetTransportAuth::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!(
            "unknown playback target transport auth stored in PostgreSQL database: {value}"
        ),
    })
}

fn parse_renderer_control_command(value: String) -> Result<RendererControlCommand> {
    RendererControlCommand::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown renderer control command stored in PostgreSQL database: {value}"),
    })
}

fn parse_renderer_session_state(value: String) -> Result<RendererSessionState> {
    RendererSessionState::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown renderer session state stored in PostgreSQL database: {value}"),
    })
}

fn parse_renderer_command_state(value: String) -> Result<RendererCommandState> {
    RendererCommandState::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown renderer command state stored in PostgreSQL database: {value}"),
    })
}
