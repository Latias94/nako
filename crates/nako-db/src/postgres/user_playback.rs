use std::collections::HashMap;

use sqlx::{Postgres, QueryBuilder, postgres::PgRow};

use nako_core::*;

use super::{
    PostgresStore, database_error, i64_to_u64, image_kind_from_parts, optional_i64_to_u32,
    optional_i64_to_u64, parse_id, parse_optional_id, row_get, u32_to_i64, u64_to_i64,
};

const USER_PLAYBACK_STATE_SELECT: &str = r#"
            SELECT
                principal_id,
                item_id::text AS item_id,
                source_id::text AS source_id,
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
impl UserPlaybackStateRepository for PostgresStore {
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(principal_id, item_id) DO UPDATE SET
                source_id = excluded.source_id,
                resume_position_ms = excluded.resume_position_ms,
                duration_ms = excluded.duration_ms,
                watched = excluded.watched,
                watched_at_ms = excluded.watched_at_ms,
                last_played_at_ms = excluded.last_played_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                version = user_playback_states.version + 1,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(state.principal_id.as_str())
        .bind(state.item_id.as_uuid())
        .bind(state.source_id.map(|id| id.as_uuid()))
        .bind(state.resume_position_ms.map(u64_to_i64).transpose()?)
        .bind(state.duration_ms.map(u64_to_i64).transpose()?)
        .bind(state.watched)
        .bind(state.watched_at_ms)
        .bind(state.last_played_at_ms)
        .bind(state.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_playback_state(&state.principal_id, state.item_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "user playback state for principal {} and item {} was not found after PostgreSQL upsert",
                    state.principal_id, state.item_id
                ),
            })
    }

    async fn get_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>> {
        let query =
            format!("{USER_PLAYBACK_STATE_SELECT} WHERE principal_id = $1 AND item_id = $2");
        let row = sqlx::query(&query)
            .bind(principal_id.as_str())
            .bind(item_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.as_ref().map(row_to_user_playback_state).transpose()
    }

    async fn list_continue_watching_states(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaybackState>> {
        let page = page.clamped();
        let query = format!(
            r#"
            {USER_PLAYBACK_STATE_SELECT}
            WHERE principal_id = $1
              AND watched = false
              AND resume_position_ms IS NOT NULL
              AND resume_position_ms > 0
            ORDER BY last_played_at_ms DESC NULLS LAST, item_id ASC
            LIMIT $2 OFFSET $3
            "#
        );
        let rows = sqlx::query(&query)
            .bind(principal_id.as_str())
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.iter().map(row_to_user_playback_state).collect()
    }

    async fn list_continue_watching_entries(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<ContinueWatchingEntry>> {
        let page = page.clamped();
        let rows = list_continue_watching_root_rows(self, principal, page).await?;
        let states = rows
            .iter()
            .map(row_to_user_playback_state)
            .collect::<Result<Vec<_>>>()?;
        let mut items = self.rows_to_media_items(rows).await?;
        let item_ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
        let mut images_by_item = list_continue_watching_images(self, &item_ids).await?;

        Ok(states
            .into_iter()
            .zip(items.drain(..))
            .map(|(state, item)| ContinueWatchingEntry {
                state,
                images: images_by_item.remove(&item.id).unwrap_or_default(),
                item,
            })
            .collect())
    }
}

async fn list_continue_watching_root_rows(
    store: &PostgresStore,
    principal: &AuthenticatedPrincipal,
    page: PageRequest,
) -> Result<Vec<PgRow>> {
    if principal.is_administrator() {
        return sqlx::query(
            r#"
            SELECT
                states.principal_id,
                states.item_id::text AS item_id,
                states.source_id::text AS source_id,
                states.resume_position_ms,
                states.duration_ms,
                states.watched,
                states.watched_at_ms,
                states.last_played_at_ms,
                states.updated_at_ms,
                states.version,
                items.id::text AS id,
                items.kind,
                items.parent_id::text AS parent_id,
                items.title,
                items.original_title,
                items.sort_title,
                items.overview,
                items.release_date,
                items.metadata_json::text AS metadata_json
            FROM user_playback_states AS states
            INNER JOIN media_items AS items
                ON items.id = states.item_id
            WHERE states.principal_id = $1
              AND states.watched = false
              AND states.resume_position_ms IS NOT NULL
              AND states.resume_position_ms > 0
            ORDER BY states.last_played_at_ms DESC NULLS LAST,
                     states.item_id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(principal.principal_id.as_str())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&store.pool)
        .await
        .map_err(database_error);
    }

    let mut query = QueryBuilder::<Postgres>::new(
        r#"
            SELECT
                states.principal_id,
                states.item_id::text AS item_id,
                states.source_id::text AS source_id,
                states.resume_position_ms,
                states.duration_ms,
                states.watched,
                states.watched_at_ms,
                states.last_played_at_ms,
                states.updated_at_ms,
                states.version,
                items.id::text AS id,
                items.kind,
                items.parent_id::text AS parent_id,
                items.title,
                items.original_title,
                items.sort_title,
                items.overview,
                items.release_date,
                items.metadata_json::text AS metadata_json
            FROM user_playback_states AS states
            INNER JOIN media_items AS items
                ON items.id = states.item_id
            WHERE states.principal_id = "#,
    );
    query.push_bind(principal.principal_id.as_str());
    query.push(
        r#"
              AND states.watched = false
              AND states.resume_position_ms IS NOT NULL
              AND states.resume_position_ms > 0
              AND EXISTS (
                  SELECT 1
                  FROM media_sources AS sources
                  WHERE sources.item_id = states.item_id
                    AND (
                        EXISTS (
                            SELECT 1
                            FROM user_library_access_policies AS user_policies
                            WHERE user_policies.user_id = "#,
    );
    query.push_bind(principal.user_id.as_uuid());
    query.push(
        r#"
                              AND user_policies.library_id = sources.library_id
                              AND user_policies.access IN ('browse', 'play', 'manage')
                        )
    "#,
    );

    if !principal.roles.is_empty() {
        query.push(
            r#"
                        OR EXISTS (
                            SELECT 1
                            FROM role_library_access_policies AS role_policies
                            WHERE role_policies.library_id = sources.library_id
                              AND role_policies.access IN ('browse', 'play', 'manage')
                              AND role_policies.role IN ("#,
        );
        let mut separated = query.separated(", ");
        for role in &principal.roles {
            separated.push_bind(role.as_str());
        }
        drop(separated);
        query.push(
            r#"
                              )
                        )
        "#,
        );
    }

    query.push(
        r#"
                    )
              )
            ORDER BY states.last_played_at_ms DESC NULLS LAST,
                     states.item_id ASC
            LIMIT "#,
    );
    query.push_bind(u32_to_i64(page.limit));
    query.push(" OFFSET ");
    query.push_bind(u64_to_i64(page.offset)?);

    query
        .build()
        .fetch_all(&store.pool)
        .await
        .map_err(database_error)
}

async fn list_continue_watching_images(
    store: &PostgresStore,
    item_ids: &[MediaItemId],
) -> Result<HashMap<MediaItemId, Vec<ContinueWatchingImageEntry>>> {
    if item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::<Postgres>::new(
        r#"
            SELECT
                selected.id::text AS selected_id,
                selected.library_id::text AS selected_library_id,
                selected.item_id::text AS selected_item_id,
                selected.kind AS selected_kind,
                selected.kind_key AS selected_kind_key,
                selected.artifact_id::text AS selected_artifact_id,
                to_char(selected.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS selected_created_at,
                to_char(selected.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS selected_updated_at,
                artifacts.id::text AS id,
                artifacts.ingest_id::text AS ingest_id,
                artifacts.library_id::text AS library_id,
                artifacts.item_id::text AS item_id,
                artifacts.kind,
                artifacts.kind_key,
                artifacts.storage_uri,
                artifacts.content_hash,
                artifacts.width,
                artifacts.height,
                artifacts.byte_len,
                artifacts.media_type,
                to_char(artifacts.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(artifacts.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM selected_artworks AS selected
            INNER JOIN managed_artwork_artifacts AS artifacts
                ON artifacts.id = selected.artifact_id
            WHERE selected.item_id IN ("#,
    );
    let mut separated = query.separated(", ");
    for item_id in item_ids {
        separated.push_bind(item_id.as_uuid());
    }
    drop(separated);
    query.push(
        r#")
            ORDER BY selected.item_id ASC,
                     selected.kind ASC,
                     selected.kind_key ASC,
                     selected.id ASC
        "#,
    );

    let rows = query
        .build()
        .fetch_all(&store.pool)
        .await
        .map_err(database_error)?;
    let mut images_by_item = HashMap::<MediaItemId, Vec<ContinueWatchingImageEntry>>::new();

    for row in rows {
        let selected = selected_artwork_from_prefixed_row(&row)?;
        let item_id = selected.item_id;
        let artifact = row_to_managed_artwork_artifact(&row)?;
        images_by_item
            .entry(item_id)
            .or_default()
            .push(ContinueWatchingImageEntry { selected, artifact });
    }

    Ok(images_by_item)
}

fn selected_artwork_from_prefixed_row(row: &PgRow) -> Result<SelectedArtworkRecord> {
    Ok(SelectedArtworkRecord {
        id: parse_id(row_get::<String>(row, "selected_id")?)?,
        library_id: parse_id(row_get::<String>(row, "selected_library_id")?)?,
        item_id: parse_id(row_get::<String>(row, "selected_item_id")?)?,
        kind: image_kind_from_parts(
            row_get(row, "selected_kind")?,
            row_get(row, "selected_kind_key")?,
        ),
        artifact_id: parse_id(row_get::<String>(row, "selected_artifact_id")?)?,
        created_at: row_get(row, "selected_created_at")?,
        updated_at: row_get(row, "selected_updated_at")?,
    })
}

fn row_to_managed_artwork_artifact(row: &PgRow) -> Result<ManagedArtworkArtifactRecord> {
    Ok(ManagedArtworkArtifactRecord {
        id: parse_id(row_get::<String>(row, "id")?)?,
        ingest_id: parse_id(row_get::<String>(row, "ingest_id")?)?,
        library_id: parse_id(row_get::<String>(row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(row, "kind")?, row_get(row, "kind_key")?),
        storage_uri: row_get(row, "storage_uri")?,
        content_hash: row_get(row, "content_hash")?,
        width: optional_i64_to_u32(row_get(row, "width")?)?,
        height: optional_i64_to_u32(row_get(row, "height")?)?,
        byte_len: optional_i64_to_u64(row_get(row, "byte_len")?)?,
        media_type: row_get(row, "media_type")?,
        created_at: row_get(row, "created_at")?,
        updated_at: row_get(row, "updated_at")?,
    })
}

fn row_to_user_playback_state(row: &PgRow) -> Result<UserPlaybackState> {
    Ok(UserPlaybackState {
        principal_id: UserPrincipalId::new(row_get::<String>(row, "principal_id")?)?,
        item_id: parse_id(row_get::<String>(row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(row, "source_id")?)?,
        resume_position_ms: optional_i64_to_u64(row_get(row, "resume_position_ms")?)?,
        duration_ms: optional_i64_to_u64(row_get(row, "duration_ms")?)?,
        watched: row_get(row, "watched")?,
        watched_at_ms: row_get(row, "watched_at_ms")?,
        last_played_at_ms: row_get(row, "last_played_at_ms")?,
        updated_at_ms: row_get(row, "updated_at_ms")?,
        version: i64_to_u64(row_get(row, "version")?)?,
    })
}
