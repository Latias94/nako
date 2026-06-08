use std::collections::HashMap;

use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::{QueryBuilder, Sqlite, sqlite::SqliteRow};

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

    async fn list_library_item_added_at(
        &self,
        library_id: LibraryId,
    ) -> Result<Vec<LibraryItemAddedAt>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, MIN(added_at) AS added_at
            FROM (
                SELECT item_id, created_at AS added_at
                FROM media_sources
                WHERE library_id = ?1
                UNION ALL
                SELECT item_id, created_at AS added_at
                FROM library_item_states
                WHERE library_id = ?1
            ) AS library_items
            GROUP BY item_id
            ORDER BY added_at ASC, item_id ASC
            "#,
        )
        .bind(library_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(LibraryItemAddedAt {
                    item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
                    added_at: row_get(&row, "added_at")?,
                })
            })
            .collect()
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

    async fn list_library_source_inventory(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<LibrarySourceInventoryEntry>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                sources.id AS source_id,
                sources.library_id AS source_library_id,
                sources.item_id AS source_item_id,
                sources.locator AS source_locator,
                sources.file_name AS source_file_name,
                sources.size_bytes AS source_size_bytes,
                sources.fingerprint AS source_fingerprint,
                items.id AS item_id,
                items.kind AS item_kind,
                items.parent_id AS item_parent_id,
                items.title AS item_title,
                items.original_title AS item_original_title,
                items.sort_title AS item_sort_title,
                items.overview AS item_overview,
                items.release_date AS item_release_date,
                items.metadata_json AS item_metadata_json,
                probes.source_id AS probe_source_id,
                probes.duration_ms AS probe_duration_ms,
                probes.container AS probe_container,
                probes.bit_rate AS probe_bit_rate
            FROM media_sources AS sources
            LEFT JOIN media_items AS items
                ON items.id = sources.item_id
            LEFT JOIN media_source_probes AS probes
                ON probes.source_id = sources.id
            WHERE sources.library_id = ?1
            ORDER BY sources.locator ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut entries = rows
            .into_iter()
            .map(sqlite_inventory_row_to_entry)
            .collect::<Result<Vec<_>>>()?;
        let item_ids = entries
            .iter()
            .filter_map(|entry| entry.item.as_ref().map(|item| item.id))
            .collect::<Vec<_>>();
        let source_ids = entries
            .iter()
            .filter(|entry| entry.probe.is_some())
            .map(|entry| entry.source.id)
            .collect::<Vec<_>>();
        let external_ids_by_item = self.list_external_ids_for_items(&item_ids).await?;
        let streams_by_source = self.list_streams_for_sources(&source_ids).await?;

        for entry in &mut entries {
            if let Some(item) = entry.item.as_mut() {
                item.metadata.external_ids = external_ids_by_item
                    .get(&item.id)
                    .cloned()
                    .unwrap_or_default();
            }
            if let Some(probe) = entry.probe.as_mut() {
                probe.streams = streams_by_source
                    .get(&entry.source.id)
                    .cloned()
                    .unwrap_or_default();
            }
        }

        Ok(entries)
    }

    async fn list_media_sources_by_fingerprint(
        &self,
        library_id: LibraryId,
        fingerprint: &str,
        exclude_source_id: Option<MediaSourceId>,
        page: PageRequest,
    ) -> Result<Vec<MediaSourceFingerprintMatch>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                library_id,
                item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint,
                EXISTS (
                    SELECT 1
                    FROM source_states
                    WHERE source_states.library_id = media_sources.library_id
                      AND source_states.source_id = media_sources.id
                      AND source_states.tombstoned != 0
                ) AS stale
            FROM media_sources
            WHERE library_id = ?1
              AND fingerprint = ?2
              AND fingerprint IS NOT NULL
              AND fingerprint != ''
              AND (?3 IS NULL OR id != ?3)
            ORDER BY id ASC
            LIMIT ?4 OFFSET ?5
            "#,
        )
        .bind(library_id.to_string())
        .bind(fingerprint)
        .bind(exclude_source_id.map(|id| id.to_string()))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_media_source_fingerprint_match)
            .collect()
    }

    async fn summarize_media_source_fingerprints(&self) -> Result<MediaSourceFingerprintSummary> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS total_sources,
                COALESCE(SUM(
                    CASE
                        WHEN fingerprint IS NOT NULL AND fingerprint != '' THEN 1
                        ELSE 0
                    END
                ), 0) AS fingerprinted_sources,
                COALESCE(SUM(
                    CASE
                        WHEN fingerprint LIKE 'source:v1:content_hash:%' THEN 1
                        ELSE 0
                    END
                ), 0) AS content_hash_sources
            FROM media_sources
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(MediaSourceFingerprintSummary {
            total_sources: i64_to_u64(row_get(&row, "total_sources")?)?,
            fingerprinted_sources: i64_to_u64(row_get(&row, "fingerprinted_sources")?)?,
            content_hash_sources: i64_to_u64(row_get(&row, "content_hash_sources")?)?,
        })
    }
}

fn row_to_media_source_fingerprint_match(
    row: sqlx::sqlite::SqliteRow,
) -> Result<MediaSourceFingerprintMatch> {
    let stale = i64_to_bool(row_get(&row, "stale")?)?;
    let source = row_to_media_source(row)?;

    Ok(MediaSourceFingerprintMatch { source, stale })
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
                    sample_rate,
                    technical_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
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
            .bind(serialize_stream_technical_json(&stream.technical)?)
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
                sample_rate,
                technical_json
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

fn serialize_stream_technical_json(value: &MediaStreamTechnicalFacts) -> Result<String> {
    serde_json::to_string(value).map_err(database_error)
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

    async fn list_external_ids_for_items(
        &self,
        item_ids: &[MediaItemId],
    ) -> Result<HashMap<MediaItemId, Vec<ExternalId>>> {
        if item_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut query = QueryBuilder::new(
            r#"
            SELECT item_id, provider, provider_key, value
            FROM media_item_external_ids
            WHERE item_id IN (
            "#,
        );
        let mut separated = query.separated(", ");
        for item_id in item_ids {
            separated.push_bind(item_id.to_string());
        }
        drop(separated);
        query
            .push(")\n            ORDER BY item_id ASC, provider ASC, provider_key ASC, value ASC");

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;
        let mut external_ids_by_item = HashMap::<MediaItemId, Vec<ExternalId>>::new();

        for row in rows {
            let item_id = parse_id(row_get::<String>(&row, "item_id")?)?;
            let provider = provider_from_parts(
                row_get::<String>(&row, "provider")?,
                row_get::<String>(&row, "provider_key")?,
            );
            external_ids_by_item
                .entry(item_id)
                .or_default()
                .push(ExternalId {
                    provider,
                    value: row_get(&row, "value")?,
                });
        }

        Ok(external_ids_by_item)
    }

    async fn list_streams_for_sources(
        &self,
        source_ids: &[MediaSourceId],
    ) -> Result<HashMap<MediaSourceId, Vec<MediaStreamInfo>>> {
        if source_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut query = QueryBuilder::new(
            r#"
            SELECT
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
                sample_rate,
                technical_json
            FROM media_streams
            WHERE source_id IN (
            "#,
        );
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id.to_string());
        }
        drop(separated);
        query.push(")\n            ORDER BY source_id ASC, stream_index ASC");

        let rows = query
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;
        let mut streams_by_source = HashMap::<MediaSourceId, Vec<MediaStreamInfo>>::new();

        for row in rows {
            let source_id = parse_id(row_get::<String>(&row, "source_id")?)?;
            streams_by_source
                .entry(source_id)
                .or_default()
                .push(row_to_stream_info(row)?);
        }

        Ok(streams_by_source)
    }
}

fn sqlite_inventory_row_to_entry(row: SqliteRow) -> Result<LibrarySourceInventoryEntry> {
    let source = MediaSource {
        id: parse_id(row_get::<String>(&row, "source_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "source_library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "source_item_id")?)?,
        locator: row_get(&row, "source_locator")?,
        file_name: row_get(&row, "source_file_name")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "source_size_bytes")?)?,
        fingerprint: row_get(&row, "source_fingerprint")?,
    };
    let item = sqlite_inventory_row_to_item(&row)?;
    let probe = row_get::<Option<String>>(&row, "probe_source_id")?
        .map(|_| {
            Ok(MediaProbeResult {
                duration_ms: optional_i64_to_u64(row_get(&row, "probe_duration_ms")?)?,
                container: row_get(&row, "probe_container")?,
                bit_rate: optional_i64_to_u64(row_get(&row, "probe_bit_rate")?)?,
                streams: Vec::new(),
            })
        })
        .transpose()?;

    Ok(LibrarySourceInventoryEntry {
        source,
        item,
        probe,
    })
}

fn sqlite_inventory_row_to_item(row: &SqliteRow) -> Result<Option<MediaItem>> {
    let Some(id) = row_get::<Option<String>>(row, "item_id")? else {
        return Ok(None);
    };
    let metadata_json = row_get::<Option<String>>(row, "item_metadata_json")?;
    let metadata = match metadata_json {
        Some(value) => serde_json::from_str::<CanonicalMetadata>(&value).map_err(database_error)?,
        None => CanonicalMetadata {
            title: row_get(row, "item_title")?,
            original_title: row_get(row, "item_original_title")?,
            sort_title: row_get(row, "item_sort_title")?,
            overview: row_get(row, "item_overview")?,
            release_date: row_get(row, "item_release_date")?,
            ..CanonicalMetadata::default()
        },
    };

    Ok(Some(MediaItem {
        id: parse_id(id)?,
        kind: parse_media_kind(row_get(row, "item_kind")?)?,
        parent_id: parse_optional_id(row_get(row, "item_parent_id")?)?,
        metadata,
    }))
}
