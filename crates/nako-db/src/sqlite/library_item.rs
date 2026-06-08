use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::{QueryBuilder, Sqlite};

#[async_trait::async_trait]
impl LibraryItemRepository for SqliteStore {
    async fn upsert_library_item_state(&self, state: &LibraryItemState) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_library_item_state_tx(&mut transaction, state).await?;
        transaction.commit().await.map_err(database_error)
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

    async fn list_library_item_states_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<LibraryItemState>> {
        let rows = sqlx::query(
            r#"
            SELECT library_id, item_id, provisional
            FROM library_item_states
            WHERE item_id = ?1
            ORDER BY library_id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_library_item_state).collect()
    }

    async fn list_library_items_for_browse(
        &self,
        library_id: LibraryId,
        principal_id: &UserPrincipalId,
        query: &LibraryItemBrowseQuery,
    ) -> Result<Vec<MediaItem>> {
        let page = query.page.clamped();
        let mut builder = QueryBuilder::new(
            r#"
            WITH library_item_membership AS (
                SELECT item_id, MIN(added_at) AS added_at
                FROM (
                    SELECT item_id, created_at AS added_at
                    FROM media_sources
                    WHERE library_id =
            "#,
        );
        builder.push_bind(library_id.to_string());
        builder.push(
            r#"
                    UNION ALL
                    SELECT item_id, created_at AS added_at
                    FROM library_item_states
                    WHERE library_id =
            "#,
        );
        builder.push_bind(library_id.to_string());
        builder.push(
            r#"
                ) AS library_items
                GROUP BY item_id
            )
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
            INNER JOIN library_item_membership AS membership
                ON membership.item_id = media_items.id
            LEFT JOIN user_playback_states AS playback
                ON playback.item_id = media_items.id
               AND playback.principal_id =
            "#,
        );
        builder.push_bind(principal_id.as_str());
        builder.push("\n            WHERE 1 = 1");

        for facet in &query.facets {
            match facet {
                LibraryItemBrowseFacet::Kind(kind) => {
                    builder.push("\n              AND media_items.kind = ");
                    builder.push_bind(media_kind_to_str(*kind));
                }
            }
        }

        builder.push(sqlite_browse_watch_state_where(query.watch_state));
        builder.push(sqlite_browse_order_by(query.sort, query.order));
        builder.push("\n            LIMIT ");
        builder.push_bind(u32_to_i64(page.limit));
        builder.push(" OFFSET ");
        builder.push_bind(u64_to_i64(page.offset)?);

        let rows = builder
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        self.rows_to_media_items(rows).await
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

pub(crate) async fn upsert_library_item_state_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    state: &LibraryItemState,
) -> Result<()> {
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
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

fn sqlite_browse_watch_state_where(filter: LibraryItemWatchStateFilter) -> &'static str {
    match filter {
        LibraryItemWatchStateFilter::Any => "",
        LibraryItemWatchStateFilter::Watched => "\n              AND playback.watched = 1",
        LibraryItemWatchStateFilter::Unwatched => {
            "\n              AND (playback.item_id IS NULL OR playback.watched = 0)"
        }
        LibraryItemWatchStateFilter::InProgress => {
            r#"
              AND playback.watched = 0
              AND playback.resume_position_ms IS NOT NULL
              AND playback.resume_position_ms > 0"#
        }
    }
}

fn sqlite_browse_order_by(
    sort: LibraryItemBrowseSortKey,
    order: LibraryItemBrowseSortOrder,
) -> &'static str {
    match (sort, order) {
        (LibraryItemBrowseSortKey::Title, LibraryItemBrowseSortOrder::Asc) => {
            "\n            ORDER BY COALESCE(media_items.sort_title, media_items.title) ASC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::Title, LibraryItemBrowseSortOrder::Desc) => {
            "\n            ORDER BY COALESCE(media_items.sort_title, media_items.title) DESC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::ReleaseDate, LibraryItemBrowseSortOrder::Asc) => {
            "\n            ORDER BY media_items.release_date IS NULL ASC, media_items.release_date ASC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::ReleaseDate, LibraryItemBrowseSortOrder::Desc) => {
            "\n            ORDER BY media_items.release_date IS NULL ASC, media_items.release_date DESC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::DateAdded, LibraryItemBrowseSortOrder::Asc) => {
            "\n            ORDER BY membership.added_at IS NULL ASC, membership.added_at ASC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::DateAdded, LibraryItemBrowseSortOrder::Desc) => {
            "\n            ORDER BY membership.added_at IS NULL ASC, membership.added_at DESC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::LastPlayed, LibraryItemBrowseSortOrder::Asc) => {
            "\n            ORDER BY playback.last_played_at_ms IS NULL ASC, playback.last_played_at_ms ASC, media_items.id ASC"
        }
        (LibraryItemBrowseSortKey::LastPlayed, LibraryItemBrowseSortOrder::Desc) => {
            "\n            ORDER BY playback.last_played_at_ms IS NULL ASC, playback.last_played_at_ms DESC, media_items.id ASC"
        }
    }
}
