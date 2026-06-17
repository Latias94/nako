use std::collections::HashMap;

use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::{QueryBuilder, Sqlite, sqlite::SqliteRow};

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

const USER_PLAYBACK_PROFILE_PREFERENCE_SELECT: &str = r#"
            SELECT
                principal_id,
                capabilities_json,
                updated_at_ms,
                version
            FROM user_playback_profile_preferences
            "#;

const USER_PLAYBACK_PROFILE_SELECT: &str = r#"
            SELECT
                profile_id,
                principal_id,
                name,
                capabilities_json,
                is_default,
                updated_at_ms,
                version
            FROM user_playback_profiles
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
            .ok_or_else(|| NakoError::Database {
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

        row.as_ref().map(row_to_user_playback_state).transpose()
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

#[async_trait::async_trait]
impl UserPlaybackProfilePreferenceRepository for SqliteStore {
    async fn upsert_user_playback_profile_preference(
        &self,
        preference: UserPlaybackProfilePreferenceWrite,
    ) -> Result<UserPlaybackProfilePreference> {
        sqlx::query(
            r#"
            INSERT INTO user_playback_profile_preferences (
                principal_id,
                capabilities_json,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3)
            ON CONFLICT(principal_id) DO UPDATE SET
                capabilities_json = excluded.capabilities_json,
                updated_at_ms = excluded.updated_at_ms,
                version = user_playback_profile_preferences.version + 1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(preference.principal_id.as_str())
        .bind(&preference.capabilities_json)
        .bind(preference.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_playback_profile_preference(&preference.principal_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "user playback profile preference for principal {} was not found after upsert",
                    preference.principal_id
                ),
            })
    }

    async fn get_user_playback_profile_preference(
        &self,
        principal_id: &UserPrincipalId,
    ) -> Result<Option<UserPlaybackProfilePreference>> {
        let row = sqlx::query(&format!(
            "{USER_PLAYBACK_PROFILE_PREFERENCE_SELECT} WHERE principal_id = ?1"
        ))
        .bind(principal_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.as_ref()
            .map(row_to_user_playback_profile_preference)
            .transpose()
    }

    async fn delete_user_playback_profile_preference(
        &self,
        principal_id: &UserPrincipalId,
    ) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM user_playback_profile_preferences WHERE principal_id = ?1")
                .bind(principal_id.as_str())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;

        Ok(result.rows_affected() > 0)
    }
}

#[async_trait::async_trait]
impl UserPlaybackProfileRepository for SqliteStore {
    async fn create_user_playback_profile(
        &self,
        profile: NewUserPlaybackProfile,
    ) -> Result<UserPlaybackProfile> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let make_default = profile.is_default
            || !user_playback_profile_exists(&mut transaction, &profile.principal_id).await?;

        if make_default {
            clear_user_playback_profile_defaults(
                &mut transaction,
                &profile.principal_id,
                None,
                profile.updated_at_ms,
            )
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO user_playback_profiles (
                profile_id,
                principal_id,
                name,
                capabilities_json,
                is_default,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(profile.profile_id.to_string())
        .bind(profile.principal_id.as_str())
        .bind(&profile.name)
        .bind(&profile.capabilities_json)
        .bind(bool_to_i64(make_default))
        .bind(profile.updated_at_ms)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        self.get_user_playback_profile(&profile.principal_id, profile.profile_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "user playback profile {} for principal {} was not found after insert",
                    profile.profile_id, profile.principal_id
                ),
            })
    }

    async fn get_user_playback_profile(
        &self,
        principal_id: &UserPrincipalId,
        profile_id: UserPlaybackProfileId,
    ) -> Result<Option<UserPlaybackProfile>> {
        let row = sqlx::query(&format!(
            "{USER_PLAYBACK_PROFILE_SELECT} WHERE principal_id = ?1 AND profile_id = ?2"
        ))
        .bind(principal_id.as_str())
        .bind(profile_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.as_ref().map(row_to_user_playback_profile).transpose()
    }

    async fn get_default_user_playback_profile(
        &self,
        principal_id: &UserPrincipalId,
    ) -> Result<Option<UserPlaybackProfile>> {
        let row = sqlx::query(&format!(
            r#"
            {USER_PLAYBACK_PROFILE_SELECT}
            WHERE principal_id = ?1 AND is_default = 1
            "#
        ))
        .bind(principal_id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.as_ref().map(row_to_user_playback_profile).transpose()
    }

    async fn list_user_playback_profiles(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaybackProfile>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {USER_PLAYBACK_PROFILE_SELECT}
            WHERE principal_id = ?1
            ORDER BY is_default DESC, updated_at_ms DESC, profile_id ASC
            LIMIT ?2 OFFSET ?3
            "#
        ))
        .bind(principal_id.as_str())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.iter().map(row_to_user_playback_profile).collect()
    }

    async fn update_user_playback_profile(
        &self,
        profile: UserPlaybackProfileUpdate,
    ) -> Result<Option<UserPlaybackProfile>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM user_playback_profiles WHERE principal_id = ?1 AND profile_id = ?2",
        )
        .bind(profile.principal_id.as_str())
        .bind(profile.profile_id.to_string())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        if exists.is_none() {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        }

        if profile.is_default {
            clear_user_playback_profile_defaults(
                &mut transaction,
                &profile.principal_id,
                Some(profile.profile_id),
                profile.updated_at_ms,
            )
            .await?;
        }

        sqlx::query(
            r#"
            UPDATE user_playback_profiles
            SET
                name = ?3,
                capabilities_json = ?4,
                is_default = ?5,
                updated_at_ms = ?6,
                version = version + 1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE principal_id = ?1
              AND profile_id = ?2
            "#,
        )
        .bind(profile.principal_id.as_str())
        .bind(profile.profile_id.to_string())
        .bind(&profile.name)
        .bind(&profile.capabilities_json)
        .bind(bool_to_i64(profile.is_default))
        .bind(profile.updated_at_ms)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        self.get_user_playback_profile(&profile.principal_id, profile.profile_id)
            .await
    }

    async fn delete_user_playback_profile(
        &self,
        principal_id: &UserPrincipalId,
        profile_id: UserPlaybackProfileId,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM user_playback_profiles WHERE principal_id = ?1 AND profile_id = ?2",
        )
        .bind(principal_id.as_str())
        .bind(profile_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected() > 0)
    }
}

async fn list_continue_watching_root_rows(
    store: &SqliteStore,
    principal: &AuthenticatedPrincipal,
    page: PageRequest,
) -> Result<Vec<SqliteRow>> {
    if principal.is_administrator() {
        return sqlx::query(
            r#"
            SELECT
                states.principal_id,
                states.item_id,
                states.source_id,
                states.resume_position_ms,
                states.duration_ms,
                states.watched,
                states.watched_at_ms,
                states.last_played_at_ms,
                states.updated_at_ms,
                states.version,
                items.id,
                items.kind,
                items.parent_id,
                items.title,
                items.original_title,
                items.sort_title,
                items.overview,
                items.release_date,
                items.metadata_json
            FROM user_playback_states AS states
            INNER JOIN media_items AS items
                ON items.id = states.item_id
            WHERE states.principal_id = ?1
              AND states.watched = 0
              AND states.resume_position_ms IS NOT NULL
              AND states.resume_position_ms > 0
            ORDER BY states.last_played_at_ms IS NULL ASC,
                     states.last_played_at_ms DESC,
                     states.item_id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(principal.principal_id.as_str())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&store.pool)
        .await
        .map_err(database_error);
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
            SELECT
                states.principal_id,
                states.item_id,
                states.source_id,
                states.resume_position_ms,
                states.duration_ms,
                states.watched,
                states.watched_at_ms,
                states.last_played_at_ms,
                states.updated_at_ms,
                states.version,
                items.id,
                items.kind,
                items.parent_id,
                items.title,
                items.original_title,
                items.sort_title,
                items.overview,
                items.release_date,
                items.metadata_json
            FROM user_playback_states AS states
            INNER JOIN media_items AS items
                ON items.id = states.item_id
            WHERE states.principal_id = "#,
    );
    query.push_bind(principal.principal_id.as_str());
    query.push(
        r#"
              AND states.watched = 0
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
    query.push_bind(principal.user_id.to_string());
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
            ORDER BY states.last_played_at_ms IS NULL ASC,
                     states.last_played_at_ms DESC,
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
    store: &SqliteStore,
    item_ids: &[MediaItemId],
) -> Result<HashMap<MediaItemId, Vec<ContinueWatchingImageEntry>>> {
    if item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
            SELECT
                selected.id AS selected_id,
                selected.library_id AS selected_library_id,
                selected.item_id AS selected_item_id,
                selected.kind AS selected_kind,
                selected.kind_key AS selected_kind_key,
                selected.artifact_id AS selected_artifact_id,
                selected.created_at AS selected_created_at,
                selected.updated_at AS selected_updated_at,
                artifacts.id,
                artifacts.ingest_id,
                artifacts.library_id,
                artifacts.item_id,
                artifacts.kind,
                artifacts.kind_key,
                artifacts.storage_uri,
                artifacts.content_hash,
                artifacts.width,
                artifacts.height,
                artifacts.byte_len,
                artifacts.media_type,
                artifacts.created_at,
                artifacts.updated_at
            FROM selected_artworks AS selected
            INNER JOIN managed_artwork_artifacts AS artifacts
                ON artifacts.id = selected.artifact_id
            WHERE selected.item_id IN ("#,
    );
    let mut separated = query.separated(", ");
    for item_id in item_ids {
        separated.push_bind(item_id.to_string());
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
        let artifact = row_to_managed_artwork_artifact(row)?;
        images_by_item
            .entry(item_id)
            .or_default()
            .push(ContinueWatchingImageEntry { selected, artifact });
    }

    Ok(images_by_item)
}

fn selected_artwork_from_prefixed_row(row: &SqliteRow) -> Result<SelectedArtworkRecord> {
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

fn row_to_user_playback_state(row: &SqliteRow) -> Result<UserPlaybackState> {
    Ok(UserPlaybackState {
        principal_id: UserPrincipalId::new(row_get::<String>(row, "principal_id")?)?,
        item_id: parse_id(row_get::<String>(row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(row, "source_id")?)?,
        resume_position_ms: optional_i64_to_u64(row_get(row, "resume_position_ms")?)?,
        duration_ms: optional_i64_to_u64(row_get(row, "duration_ms")?)?,
        watched: i64_to_bool(row_get(row, "watched")?)?,
        watched_at_ms: row_get(row, "watched_at_ms")?,
        last_played_at_ms: row_get(row, "last_played_at_ms")?,
        updated_at_ms: row_get(row, "updated_at_ms")?,
        version: i64_to_u64(row_get(row, "version")?)?,
    })
}

fn row_to_user_playback_profile_preference(
    row: &SqliteRow,
) -> Result<UserPlaybackProfilePreference> {
    Ok(UserPlaybackProfilePreference {
        principal_id: UserPrincipalId::new(row_get::<String>(row, "principal_id")?)?,
        capabilities_json: row_get(row, "capabilities_json")?,
        updated_at_ms: row_get(row, "updated_at_ms")?,
        version: i64_to_u64(row_get(row, "version")?)?,
    })
}

fn row_to_user_playback_profile(row: &SqliteRow) -> Result<UserPlaybackProfile> {
    Ok(UserPlaybackProfile {
        profile_id: parse_id(row_get::<String>(row, "profile_id")?)?,
        principal_id: UserPrincipalId::new(row_get::<String>(row, "principal_id")?)?,
        name: row_get(row, "name")?,
        capabilities_json: row_get(row, "capabilities_json")?,
        is_default: i64_to_bool(row_get(row, "is_default")?)?,
        updated_at_ms: row_get(row, "updated_at_ms")?,
        version: i64_to_u64(row_get(row, "version")?)?,
    })
}

async fn user_playback_profile_exists(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    principal_id: &UserPrincipalId,
) -> Result<bool> {
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM user_playback_profiles WHERE principal_id = ?1 LIMIT 1")
            .bind(principal_id.as_str())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(database_error)?;

    Ok(exists.is_some())
}

async fn clear_user_playback_profile_defaults(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    principal_id: &UserPrincipalId,
    except_profile_id: Option<UserPlaybackProfileId>,
    updated_at_ms: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE user_playback_profiles
        SET
            is_default = 0,
            updated_at_ms = ?3,
            version = version + 1,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE principal_id = ?1
          AND is_default = 1
          AND (?2 IS NULL OR profile_id <> ?2)
        "#,
    )
    .bind(principal_id.as_str())
    .bind(except_profile_id.map(|id| id.to_string()))
    .bind(updated_at_ms)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}
