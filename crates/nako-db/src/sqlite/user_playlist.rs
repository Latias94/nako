use std::collections::BTreeSet;

use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::Sqlite;

const USER_PLAYLIST_SELECT: &str = r#"
            SELECT
                p.id,
                p.principal_id,
                p.name,
                p.visibility,
                (
                    SELECT COUNT(*)
                    FROM user_playlist_items i
                    WHERE i.playlist_id = p.id
                ) AS item_count,
                p.created_at_ms,
                p.updated_at_ms,
                p.version
            FROM user_playlists p
            "#;

const USER_PLAYLIST_ITEM_SELECT: &str = r#"
            SELECT
                i.playlist_id,
                i.item_id,
                i.position,
                i.added_at_ms
            FROM user_playlist_items i
            JOIN user_playlists p ON p.id = i.playlist_id
            "#;

#[async_trait::async_trait]
impl UserPlaylistRepository for SqliteStore {
    async fn create_user_playlist(&self, playlist: NewUserPlaylist) -> Result<UserPlaylistRecord> {
        sqlx::query(
            r#"
            INSERT INTO user_playlists (
                id,
                principal_id,
                name,
                visibility,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, 'private', ?4, ?4)
            "#,
        )
        .bind(playlist.id.to_string())
        .bind(playlist.principal_id.as_str())
        .bind(&playlist.name)
        .bind(playlist.created_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_playlist(&playlist.principal_id, playlist.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "user playlist {} for principal {} was not found after insert",
                    playlist.id, playlist.principal_id
                ),
            })
    }

    async fn get_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistRecord>> {
        get_user_playlist(&self.pool, principal_id, playlist_id).await
    }

    async fn list_user_playlists(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {USER_PLAYLIST_SELECT}
            WHERE p.principal_id = ?1
            ORDER BY p.updated_at_ms DESC, p.id ASC
            LIMIT ?2 OFFSET ?3
            "#
        ))
        .bind(principal_id.as_str())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_user_playlist).collect()
    }

    async fn update_user_playlist_name(
        &self,
        update: UserPlaylistNameUpdate,
    ) -> Result<Option<UserPlaylistRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE user_playlists
            SET
                name = ?3,
                updated_at_ms = ?5,
                version = version + 1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
              AND principal_id = ?2
              AND (?4 IS NULL OR version = ?4)
            "#,
        )
        .bind(update.playlist_id.to_string())
        .bind(update.principal_id.as_str())
        .bind(&update.name)
        .bind(update.expected_version.map(u64_to_i64).transpose()?)
        .bind(update.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_user_playlist(&update.principal_id, update.playlist_id)
            .await
    }

    async fn delete_user_playlist(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
    ) -> Result<bool> {
        let result = sqlx::query("DELETE FROM user_playlists WHERE id = ?1 AND principal_id = ?2")
            .bind(playlist_id.to_string())
            .bind(principal_id.as_str())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn add_user_playlist_item(
        &self,
        write: UserPlaylistItemWrite,
    ) -> Result<Option<UserPlaylistRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let Some(playlist) =
            get_user_playlist_tx(&mut transaction, &write.principal_id, write.playlist_id).await?
        else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        if expected_version_mismatches(&playlist, write.expected_version) {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        }

        let mut items = list_all_user_playlist_items_tx(
            &mut transaction,
            &write.principal_id,
            write.playlist_id,
        )
        .await?;
        let existing_index = items.iter().position(|item| item.item_id == write.item_id);
        if existing_index.is_some() && write.position.is_none() {
            transaction.commit().await.map_err(database_error)?;
            return Ok(Some(playlist));
        }

        let item =
            existing_index
                .map(|index| items.remove(index))
                .unwrap_or(UserPlaylistItemRecord {
                    playlist_id: write.playlist_id,
                    item_id: write.item_id,
                    position: 0,
                    added_at_ms: write.added_at_ms,
                });
        let target_position = write
            .position
            .map(|position| position as usize)
            .unwrap_or(items.len())
            .min(items.len());
        items.insert(target_position, item);
        replace_items_tx(&mut transaction, write.playlist_id, &items).await?;
        bump_playlist_version_tx(
            &mut transaction,
            &write.principal_id,
            write.playlist_id,
            write.updated_at_ms,
        )
        .await?;
        let updated =
            get_user_playlist_tx(&mut transaction, &write.principal_id, write.playlist_id).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(updated)
    }

    async fn remove_user_playlist_item(
        &self,
        removal: UserPlaylistItemRemoval,
    ) -> Result<Option<UserPlaylistRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let Some(playlist) =
            get_user_playlist_tx(&mut transaction, &removal.principal_id, removal.playlist_id)
                .await?
        else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        if expected_version_mismatches(&playlist, removal.expected_version) {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        }

        let mut items = list_all_user_playlist_items_tx(
            &mut transaction,
            &removal.principal_id,
            removal.playlist_id,
        )
        .await?;
        let Some(index) = items
            .iter()
            .position(|item| item.item_id == removal.item_id)
        else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(Some(playlist));
        };
        items.remove(index);
        replace_items_tx(&mut transaction, removal.playlist_id, &items).await?;
        bump_playlist_version_tx(
            &mut transaction,
            &removal.principal_id,
            removal.playlist_id,
            removal.updated_at_ms,
        )
        .await?;
        let updated =
            get_user_playlist_tx(&mut transaction, &removal.principal_id, removal.playlist_id)
                .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(updated)
    }

    async fn replace_user_playlist_item_order(
        &self,
        reorder: UserPlaylistReorder,
    ) -> Result<Option<UserPlaylistRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let Some(playlist) =
            get_user_playlist_tx(&mut transaction, &reorder.principal_id, reorder.playlist_id)
                .await?
        else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        if expected_version_mismatches(&playlist, reorder.expected_version) {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        }

        let items = list_all_user_playlist_items_tx(
            &mut transaction,
            &reorder.principal_id,
            reorder.playlist_id,
        )
        .await?;
        let reordered = reorder_existing_items(reorder.playlist_id, &items, &reorder.item_ids)?;
        if reordered == items {
            transaction.commit().await.map_err(database_error)?;
            return Ok(Some(playlist));
        }

        replace_items_tx(&mut transaction, reorder.playlist_id, &reordered).await?;
        bump_playlist_version_tx(
            &mut transaction,
            &reorder.principal_id,
            reorder.playlist_id,
            reorder.updated_at_ms,
        )
        .await?;
        let updated =
            get_user_playlist_tx(&mut transaction, &reorder.principal_id, reorder.playlist_id)
                .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(updated)
    }

    async fn list_user_playlist_items(
        &self,
        principal_id: &UserPrincipalId,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistItemRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {USER_PLAYLIST_ITEM_SELECT}
            WHERE p.principal_id = ?1
              AND i.playlist_id = ?2
            ORDER BY i.position ASC, i.item_id ASC
            LIMIT ?3 OFFSET ?4
            "#
        ))
        .bind(principal_id.as_str())
        .bind(playlist_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_user_playlist_item).collect()
    }
}

async fn get_user_playlist(
    pool: &sqlx::SqlitePool,
    principal_id: &UserPrincipalId,
    playlist_id: UserPlaylistId,
) -> Result<Option<UserPlaylistRecord>> {
    let row = sqlx::query(&format!(
        "{USER_PLAYLIST_SELECT} WHERE p.principal_id = ?1 AND p.id = ?2"
    ))
    .bind(principal_id.as_str())
    .bind(playlist_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_user_playlist).transpose()
}

async fn get_user_playlist_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    principal_id: &UserPrincipalId,
    playlist_id: UserPlaylistId,
) -> Result<Option<UserPlaylistRecord>> {
    let row = sqlx::query(&format!(
        "{USER_PLAYLIST_SELECT} WHERE p.principal_id = ?1 AND p.id = ?2"
    ))
    .bind(principal_id.as_str())
    .bind(playlist_id.to_string())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_user_playlist).transpose()
}

async fn list_all_user_playlist_items_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    principal_id: &UserPrincipalId,
    playlist_id: UserPlaylistId,
) -> Result<Vec<UserPlaylistItemRecord>> {
    let rows = sqlx::query(&format!(
        r#"
        {USER_PLAYLIST_ITEM_SELECT}
        WHERE p.principal_id = ?1
          AND i.playlist_id = ?2
        ORDER BY i.position ASC, i.item_id ASC
        "#
    ))
    .bind(principal_id.as_str())
    .bind(playlist_id.to_string())
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;

    rows.into_iter().map(row_to_user_playlist_item).collect()
}

async fn replace_items_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    playlist_id: UserPlaylistId,
    items: &[UserPlaylistItemRecord],
) -> Result<()> {
    sqlx::query("DELETE FROM user_playlist_items WHERE playlist_id = ?1")
        .bind(playlist_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for (position, item) in items.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO user_playlist_items (
                playlist_id,
                item_id,
                position,
                added_at_ms
            )
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(playlist_id.to_string())
        .bind(item.item_id.to_string())
        .bind(u32_to_i64(u32::try_from(position).map_err(|err| {
            NakoError::InvalidInput {
                message: format!("playlist position does not fit into u32: {err}"),
            }
        })?))
        .bind(item.added_at_ms)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn bump_playlist_version_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    principal_id: &UserPrincipalId,
    playlist_id: UserPlaylistId,
    updated_at_ms: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE user_playlists
        SET
            updated_at_ms = ?3,
            version = version + 1,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ?1
          AND principal_id = ?2
        "#,
    )
    .bind(playlist_id.to_string())
    .bind(principal_id.as_str())
    .bind(updated_at_ms)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

fn expected_version_mismatches(
    playlist: &UserPlaylistRecord,
    expected_version: Option<u64>,
) -> bool {
    expected_version.is_some_and(|version| version != playlist.version)
}

fn reorder_existing_items(
    playlist_id: UserPlaylistId,
    current: &[UserPlaylistItemRecord],
    item_ids: &[MediaItemId],
) -> Result<Vec<UserPlaylistItemRecord>> {
    if item_ids.len() != current.len() {
        return Err(NakoError::InvalidInput {
            message: "playlist reorder must include every existing item exactly once".to_owned(),
        });
    }
    let unique = item_ids.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != item_ids.len() {
        return Err(NakoError::InvalidInput {
            message: "playlist reorder cannot contain duplicate item ids".to_owned(),
        });
    }

    let mut reordered = Vec::with_capacity(current.len());
    for (position, item_id) in item_ids.iter().copied().enumerate() {
        let Some(existing) = current.iter().find(|item| item.item_id == item_id) else {
            return Err(NakoError::InvalidInput {
                message: format!("playlist reorder contains foreign item id {item_id}"),
            });
        };
        reordered.push(UserPlaylistItemRecord {
            playlist_id,
            item_id,
            position: u32::try_from(position).map_err(|err| NakoError::InvalidInput {
                message: format!("playlist position does not fit into u32: {err}"),
            })?,
            added_at_ms: existing.added_at_ms,
        });
    }

    Ok(reordered)
}

fn row_to_user_playlist(row: sqlx::sqlite::SqliteRow) -> Result<UserPlaylistRecord> {
    let visibility = row_get::<String>(&row, "visibility")?;
    Ok(UserPlaylistRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        principal_id: UserPrincipalId::new(row_get::<String>(&row, "principal_id")?)?,
        name: row_get(&row, "name")?,
        visibility: UserPlaylistVisibility::parse(&visibility).ok_or_else(|| {
            NakoError::Database {
                message: format!("unknown user playlist visibility stored in SQLite: {visibility}"),
            }
        })?,
        item_count: i64_to_u32(row_get(&row, "item_count")?)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        version: i64_to_u64(row_get(&row, "version")?)?,
    })
}

fn row_to_user_playlist_item(row: sqlx::sqlite::SqliteRow) -> Result<UserPlaylistItemRecord> {
    Ok(UserPlaylistItemRecord {
        playlist_id: parse_id(row_get::<String>(&row, "playlist_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        position: i64_to_u32(row_get(&row, "position")?)?,
        added_at_ms: row_get(&row, "added_at_ms")?,
    })
}
