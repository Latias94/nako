use super::{SqliteStore, codec::*};
use sqlx::Sqlite;
use taru_core::*;

#[async_trait::async_trait]
impl MediaRepository for SqliteStore {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_media_item_in_transaction(&mut transaction, item).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json
            FROM media_items
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let external_ids = self.list_external_ids(id).await?;

        Ok(Some(row_to_media_item(row, external_ids)?))
    }

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json
            FROM media_items
            ORDER BY title ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let id = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self.list_external_ids(id).await?;
            items.push(row_to_media_item(row, external_ids)?);
        }

        Ok(items)
    }

    async fn list_media_items_for_library(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
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
            WHERE media_items.id IN (
                SELECT item_id FROM media_sources WHERE library_id = ?1
                UNION
                SELECT item_id FROM library_item_states WHERE library_id = ?1
            )
            ORDER BY media_items.title ASC, media_items.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let id = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self.list_external_ids(id).await?;
            items.push(row_to_media_item(row, external_ids)?);
        }

        Ok(items)
    }

    async fn upsert_media_source(&self, source: &MediaSource) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_media_source_in_transaction(&mut transaction, source).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>> {
        let row = sqlx::query(
            r#"
            SELECT id, library_id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_media_source).transpose()
    }

    async fn get_media_source_by_locator(
        &self,
        library_id: LibraryId,
        locator: &str,
    ) -> Result<Option<MediaSource>> {
        let row = sqlx::query(
            r#"
            SELECT id, library_id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE library_id = ?1 AND locator = ?2
            "#,
        )
        .bind(library_id.to_string())
        .bind(locator)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_media_source).transpose()
    }

    async fn list_item_sources(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, library_id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE item_id = ?1
            ORDER BY locator ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_media_source).collect()
    }

    async fn list_media_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, library_id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE library_id = ?1
            ORDER BY locator ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_media_source).collect()
    }
}

pub(crate) async fn upsert_media_item_in_transaction(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item: &MediaItem,
) -> Result<()> {
    sqlx::query(
        r#"
            INSERT INTO media_items (
                id,
                kind,
                parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                parent_id = excluded.parent_id,
                title = excluded.title,
                original_title = excluded.original_title,
                sort_title = excluded.sort_title,
                overview = excluded.overview,
                release_date = excluded.release_date,
                metadata_json = excluded.metadata_json,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
    )
    .bind(item.id.to_string())
    .bind(media_kind_to_str(item.kind))
    .bind(item.parent_id.map(|id| id.to_string()))
    .bind(&item.metadata.title)
    .bind(&item.metadata.original_title)
    .bind(&item.metadata.sort_title)
    .bind(&item.metadata.overview)
    .bind(&item.metadata.release_date)
    .bind(serialize_metadata_json(&item.metadata)?)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM media_item_external_ids WHERE item_id = ?1")
        .bind(item.id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &item.metadata.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
                INSERT INTO media_item_external_ids (item_id, provider, provider_key, value)
                VALUES (?1, ?2, ?3, ?4)
                "#,
        )
        .bind(item.id.to_string())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

pub(crate) async fn upsert_media_source_in_transaction(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    source: &MediaSource,
) -> Result<()> {
    sqlx::query(
        r#"
            INSERT INTO media_sources (
                id,
                library_id,
                item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                library_id = excluded.library_id,
                item_id = excluded.item_id,
                locator = excluded.locator,
                file_name = excluded.file_name,
                size_bytes = excluded.size_bytes,
                fingerprint = excluded.fingerprint,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
    )
    .bind(source.id.to_string())
    .bind(source.library_id.to_string())
    .bind(source.item_id.to_string())
    .bind(&source.locator)
    .bind(&source.file_name)
    .bind(optional_u64_to_i64(source.size_bytes)?)
    .bind(&source.fingerprint)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

#[async_trait::async_trait]
impl MediaProbeRepository for SqliteStore {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO media_source_probes (source_id, duration_ms, container, bit_rate)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(source_id) DO UPDATE SET
                duration_ms = excluded.duration_ms,
                container = excluded.container,
                bit_rate = excluded.bit_rate,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(source_id.to_string())
        .bind(optional_u64_to_i64(result.duration_ms)?)
        .bind(&result.container)
        .bind(optional_u64_to_i64(result.bit_rate)?)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM media_streams WHERE source_id = ?1")
            .bind(source_id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for stream in &result.streams {
            let (kind, kind_key) = stream_kind_to_parts(&stream.kind);

            sqlx::query(
                r#"
                INSERT INTO media_streams (
                    source_id,
                    stream_index,
                    kind,
                    kind_key,
                    codec,
                    language,
                    duration_ms,
                    bit_rate,
                    width,
                    height,
                    channels,
                    sample_rate
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
            )
            .bind(source_id.to_string())
            .bind(u32_to_i64(stream.index))
            .bind(kind)
            .bind(kind_key)
            .bind(&stream.codec)
            .bind(&stream.language)
            .bind(optional_u64_to_i64(stream.duration_ms)?)
            .bind(optional_u64_to_i64(stream.bit_rate)?)
            .bind(optional_u32_to_i64(stream.width))
            .bind(optional_u32_to_i64(stream.height))
            .bind(optional_u32_to_i64(stream.channels))
            .bind(optional_u32_to_i64(stream.sample_rate))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_probe(&self, source_id: MediaSourceId) -> Result<Option<MediaProbeResult>> {
        let row = sqlx::query(
            r#"
            SELECT duration_ms, container, bit_rate
            FROM media_source_probes
            WHERE source_id = ?1
            "#,
        )
        .bind(source_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let stream_rows = sqlx::query(
            r#"
            SELECT
                stream_index,
                kind,
                kind_key,
                codec,
                language,
                duration_ms,
                bit_rate,
                width,
                height,
                channels,
                sample_rate
            FROM media_streams
            WHERE source_id = ?1
            ORDER BY stream_index ASC
            "#,
        )
        .bind(source_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let streams = stream_rows
            .into_iter()
            .map(row_to_stream_info)
            .collect::<Result<Vec<_>>>()?;

        Ok(Some(MediaProbeResult {
            duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
            container: row_get(&row, "container")?,
            bit_rate: optional_i64_to_u64(row_get(&row, "bit_rate")?)?,
            streams,
        }))
    }
}

impl SqliteStore {
    pub(crate) async fn rows_to_media_items(
        &self,
        rows: Vec<sqlx::sqlite::SqliteRow>,
    ) -> Result<Vec<MediaItem>> {
        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let id = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self.list_external_ids(id).await?;
            items.push(row_to_media_item(row, external_ids)?);
        }

        Ok(items)
    }

    pub(crate) async fn list_external_ids(&self, item_id: MediaItemId) -> Result<Vec<ExternalId>> {
        let rows = sqlx::query(
            r#"
            SELECT provider, provider_key, value
            FROM media_item_external_ids
            WHERE item_id = ?1
            ORDER BY provider ASC, provider_key ASC, value ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                let provider = provider_from_parts(
                    row_get::<String>(&row, "provider")?,
                    row_get::<String>(&row, "provider_key")?,
                );

                Ok(ExternalId {
                    provider,
                    value: row_get(&row, "value")?,
                })
            })
            .collect()
    }
}
