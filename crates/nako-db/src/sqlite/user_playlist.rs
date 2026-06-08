use std::collections::{BTreeSet, HashMap};

use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::{QueryBuilder, Sqlite, sqlite::SqliteRow};

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

    async fn get_user_playlist_summary_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
    ) -> Result<Option<UserPlaylistSummaryProjection>> {
        let mut query = user_playlist_summary_query(principal);
        query.push(" AND p.id = ");
        query.push_bind(playlist_id.to_string());

        let row = query
            .build()
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user_playlist_summary_projection).transpose()
    }

    async fn list_user_playlist_summary_projections(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<UserPlaylistSummaryProjection>> {
        let page = page.clamped();
        let mut query = user_playlist_summary_query(principal);
        query.push(
            r#"
            ORDER BY p.updated_at_ms DESC, p.id ASC
            LIMIT "#,
        );
        query.push_bind(u32_to_i64(page.limit));
        query.push(" OFFSET ");
        query.push_bind(u64_to_i64(page.offset)?);

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_user_playlist_summary_projection)
            .collect()
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

    async fn get_user_playlist_items_projection(
        &self,
        principal: &AuthenticatedPrincipal,
        playlist_id: UserPlaylistId,
        page: PageRequest,
    ) -> Result<Option<UserPlaylistItemsProjection>> {
        let page = page.clamped();
        let Some(playlist) =
            get_user_playlist(&self.pool, &principal.principal_id, playlist_id).await?
        else {
            return Ok(None);
        };
        let accessible_item_count =
            count_accessible_user_playlist_items(self, principal, playlist_id).await?;
        let rows =
            list_user_playlist_projection_root_rows(self, principal, playlist_id, page).await?;
        let playlist_items = rows
            .iter()
            .map(row_to_user_playlist_item_ref)
            .collect::<Result<Vec<_>>>()?;
        let mut items = self.rows_to_media_items(rows).await?;
        let item_ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
        let mut images_by_item = list_user_playlist_images(self, &item_ids).await?;

        let items = playlist_items
            .into_iter()
            .zip(items.drain(..))
            .map(|(playlist_item, item)| UserPlaylistItemEntry {
                playlist_item,
                images: images_by_item.remove(&item.id).unwrap_or_default(),
                item,
            })
            .collect();

        Ok(Some(UserPlaylistItemsProjection {
            playlist,
            accessible_item_count,
            items,
        }))
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

fn user_playlist_summary_query(principal: &AuthenticatedPrincipal) -> QueryBuilder<'_, Sqlite> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
            SELECT
                p.id,
                p.principal_id,
                p.name,
                p.visibility,
                (
                    SELECT COUNT(*)
                    FROM user_playlist_items AS all_items
                    WHERE all_items.playlist_id = p.id
                ) AS item_count,
                p.created_at_ms,
                p.updated_at_ms,
                p.version,
                (
                    SELECT COUNT(*)
                    FROM user_playlist_items AS i
                    INNER JOIN media_items AS items ON items.id = i.item_id
                    WHERE i.playlist_id = p.id
        "#,
    );
    push_user_playlist_access_filter(&mut query, principal);
    query.push(
        r#"
                ) AS accessible_item_count
            FROM user_playlists AS p
            WHERE p.principal_id = "#,
    );
    query.push_bind(principal.principal_id.as_str());
    query
}

async fn count_accessible_user_playlist_items(
    store: &SqliteStore,
    principal: &AuthenticatedPrincipal,
    playlist_id: UserPlaylistId,
) -> Result<u32> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
            SELECT COUNT(*) AS item_count
            FROM user_playlist_items AS i
            JOIN user_playlists AS p ON p.id = i.playlist_id
            INNER JOIN media_items AS items ON items.id = i.item_id
            WHERE p.principal_id = "#,
    );
    query.push_bind(principal.principal_id.as_str());
    query.push(" AND i.playlist_id = ");
    query.push_bind(playlist_id.to_string());
    push_user_playlist_access_filter(&mut query, principal);

    let row = query
        .build()
        .fetch_one(&store.pool)
        .await
        .map_err(database_error)?;

    i64_to_u32(row_get(&row, "item_count")?)
}

async fn list_user_playlist_projection_root_rows(
    store: &SqliteStore,
    principal: &AuthenticatedPrincipal,
    playlist_id: UserPlaylistId,
    page: PageRequest,
) -> Result<Vec<SqliteRow>> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
            SELECT
                i.playlist_id,
                i.item_id,
                i.position,
                i.added_at_ms,
                items.id,
                items.kind,
                items.parent_id,
                items.title,
                items.original_title,
                items.sort_title,
                items.overview,
                items.release_date,
                items.metadata_json
            FROM user_playlist_items AS i
            JOIN user_playlists AS p ON p.id = i.playlist_id
            INNER JOIN media_items AS items ON items.id = i.item_id
            WHERE p.principal_id = "#,
    );
    query.push_bind(principal.principal_id.as_str());
    query.push(" AND i.playlist_id = ");
    query.push_bind(playlist_id.to_string());
    push_user_playlist_access_filter(&mut query, principal);
    query.push(
        r#"
            ORDER BY i.position ASC, i.item_id ASC
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

fn push_user_playlist_access_filter(
    query: &mut QueryBuilder<'_, Sqlite>,
    principal: &AuthenticatedPrincipal,
) {
    if principal.is_administrator() {
        return;
    }

    query.push(
        r#"
              AND EXISTS (
                  SELECT 1
                  FROM media_sources AS sources
                  WHERE sources.item_id = i.item_id
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
    "#,
    );
}

async fn list_user_playlist_images(
    store: &SqliteStore,
    item_ids: &[MediaItemId],
) -> Result<HashMap<MediaItemId, Vec<UserPlaylistImageEntry>>> {
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
    let mut images_by_item = HashMap::<MediaItemId, Vec<UserPlaylistImageEntry>>::new();

    for row in rows {
        let selected = selected_artwork_from_prefixed_row(&row)?;
        let item_id = selected.item_id;
        let artifact = row_to_managed_artwork_artifact(row)?;
        images_by_item
            .entry(item_id)
            .or_default()
            .push(UserPlaylistImageEntry { selected, artifact });
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

fn row_to_user_playlist_summary_projection(
    row: SqliteRow,
) -> Result<UserPlaylistSummaryProjection> {
    let accessible_item_count = i64_to_u32(row_get(&row, "accessible_item_count")?)?;
    let playlist = row_to_user_playlist(row)?;
    Ok(UserPlaylistSummaryProjection {
        playlist,
        accessible_item_count,
    })
}

fn row_to_user_playlist_item(row: sqlx::sqlite::SqliteRow) -> Result<UserPlaylistItemRecord> {
    row_to_user_playlist_item_ref(&row)
}

fn row_to_user_playlist_item_ref(row: &SqliteRow) -> Result<UserPlaylistItemRecord> {
    Ok(UserPlaylistItemRecord {
        playlist_id: parse_id(row_get::<String>(row, "playlist_id")?)?,
        item_id: parse_id(row_get::<String>(row, "item_id")?)?,
        position: i64_to_u32(row_get(row, "position")?)?,
        added_at_ms: row_get(row, "added_at_ms")?,
    })
}
