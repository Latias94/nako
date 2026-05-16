use super::*;

#[async_trait::async_trait]
impl LibraryItemRepository for SqliteStore {
    async fn upsert_library_item_state(&self, state: &LibraryItemState) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO library_item_states (library_id, item_id, provisional)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(library_id, item_id) DO UPDATE SET
                provisional = excluded.provisional,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(state.library_id.to_string())
        .bind(state.item_id.to_string())
        .bind(bool_to_i64(state.provisional))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>> {
        let row = sqlx::query(
            r#"
            SELECT library_id, item_id, provisional
            FROM library_item_states
            WHERE library_id = ?1 AND item_id = ?2
            "#,
        )
        .bind(library_id.to_string())
        .bind(item_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_library_item_state).transpose()
    }

    async fn find_library_item_by_kind_parent_title(
        &self,
        library_id: LibraryId,
        kind: MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
    ) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            r#"
            SELECT
                media_items.id,
                media_items.kind,
                media_items.parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json
            FROM media_items
            INNER JOIN library_item_states
                ON library_item_states.item_id = media_items.id
            WHERE library_item_states.library_id = ?1
              AND media_items.kind = ?2
              AND (
                  (?3 IS NULL AND media_items.parent_id IS NULL)
                  OR media_items.parent_id = ?3
              )
              AND media_items.title = ?4
            ORDER BY media_items.id ASC
            LIMIT 1
            "#,
        )
        .bind(library_id.to_string())
        .bind(media_kind_to_str(kind))
        .bind(parent_id.map(|id| id.to_string()))
        .bind(title)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self.list_external_ids(id).await?;
        row_to_media_item(row, external_ids).map(Some)
    }
}
