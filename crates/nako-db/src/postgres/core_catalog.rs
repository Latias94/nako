use std::fmt::Display;

use sqlx::{Postgres, QueryBuilder, postgres::PgRow};

use nako_core::*;
use nako_search::{
    SearchDocument, SearchEvaluationDocument, SearchHit, SearchIndex, SearchQuery,
    evaluate_search_documents,
};

use super::{
    PostgresStore, database_error, i64_to_u16, i64_to_u32, i64_to_u64, optional_i64_to_i32,
    optional_i64_to_u16, optional_i64_to_u32, optional_i64_to_u64, parse_id, parse_optional_id,
    provider_from_parts, provider_to_parts, row_get, u32_to_i64, u64_to_i64,
};

const MEDIA_ITEM_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                parent_id::text AS parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json::text AS metadata_json
            FROM media_items
            WHERE id = $1
            "#;

const MEDIA_SOURCE_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE id = $1
            "#;

const LOCAL_INFERENCE_EVIDENCE_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                inferred_kind,
                inferred_title,
                inferred_year,
                inferred_season,
                inferred_episode,
                confidence_milli,
                evidence_source,
                evidence_source_key,
                evidence_value,
                inference_version
            FROM local_inference_evidence
            WHERE id = $1
            "#;

const SOURCE_DUPLICATE_RELATIONSHIP_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                duplicate_source_id::text AS duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE id = $1
            "#;

#[async_trait::async_trait]
impl LibraryRepository for PostgresStore {
    async fn upsert_library(&self, library: &Library) -> Result<()> {
        let roots_json = serde_json::to_string(&library.roots).map_err(database_error)?;
        let options_json = serde_json::to_string(&library.options).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO libraries (
                id, name, roots_json, options_json, domain, preset
            )
            VALUES ($1, $2, $3::jsonb, $4::jsonb, $5, $6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                roots_json = excluded.roots_json,
                options_json = excluded.options_json,
                domain = excluded.domain,
                preset = excluded.preset,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(library.id.as_uuid())
        .bind(&library.name)
        .bind(roots_json)
        .bind(options_json)
        .bind(library.options.domain.as_str())
        .bind(library.options.preset.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                name,
                roots_json::text AS roots_json,
                options_json::text AS options_json
            FROM libraries
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_library).transpose()
    }

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                name,
                roots_json::text AS roots_json,
                options_json::text AS options_json
            FROM libraries
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_library).collect()
    }
}

#[async_trait::async_trait]
impl LibraryItemRepository for PostgresStore {
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
            SELECT
                library_id::text AS library_id,
                item_id::text AS item_id,
                provisional
            FROM library_item_states
            WHERE library_id = $1 AND item_id = $2
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(item_id.as_uuid())
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
            SELECT
                library_id::text AS library_id,
                item_id::text AS item_id,
                provisional
            FROM library_item_states
            WHERE item_id = $1
            ORDER BY library_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
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
        builder.push_bind(library_id.as_uuid());
        builder.push(
            r#"
                    UNION ALL
                    SELECT item_id, created_at AS added_at
                    FROM library_item_states
                    WHERE library_id =
            "#,
        );
        builder.push_bind(library_id.as_uuid());
        builder.push(
            r#"
                ) AS library_items
                GROUP BY item_id
            )
            SELECT
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json
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
                    builder.push_bind(kind.as_str());
                }
            }
        }

        builder.push(postgres_browse_watch_state_where(query.watch_state));
        builder.push(postgres_browse_order_by(query.sort, query.order));
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
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json
            FROM media_items
            INNER JOIN library_item_states
                ON library_item_states.item_id = media_items.id
            WHERE library_item_states.library_id = $1
              AND media_items.kind = $2
              AND (
                  ($3::uuid IS NULL AND media_items.parent_id IS NULL)
                  OR media_items.parent_id = $3
              )
              AND media_items.title = $4
            ORDER BY media_items.id ASC
            LIMIT 1
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(kind.as_str())
        .bind(parent_id.map(|id| id.as_uuid()))
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

#[async_trait::async_trait]
impl MediaRepository for PostgresStore {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_media_item_tx(&mut transaction, item).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        let row = sqlx::query(MEDIA_ITEM_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self.list_external_ids(id).await?;

        row_to_media_item(row, external_ids).map(Some)
    }

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                kind,
                parent_id::text AS parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json::text AS metadata_json
            FROM media_items
            ORDER BY title ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn list_media_items_for_library(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json
            FROM media_items
            WHERE media_items.id IN (
                SELECT item_id FROM media_sources WHERE library_id = $1
                UNION
                SELECT item_id FROM library_item_states WHERE library_id = $1
            )
            ORDER BY media_items.title ASC, media_items.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn list_library_item_added_at(
        &self,
        library_id: LibraryId,
    ) -> Result<Vec<LibraryItemAddedAt>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                to_char(
                    MIN(added_at) AT TIME ZONE 'UTC',
                    'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'
                ) AS added_at
            FROM (
                SELECT item_id, created_at AS added_at
                FROM media_sources
                WHERE library_id = $1
                UNION ALL
                SELECT item_id, created_at AS added_at
                FROM library_item_states
                WHERE library_id = $1
            ) AS library_items
            GROUP BY item_id
            ORDER BY MIN(added_at) ASC, item_id ASC
            "#,
        )
        .bind(library_id.as_uuid())
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
        upsert_media_source_tx(&mut transaction, source).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>> {
        let row = sqlx::query(MEDIA_SOURCE_SELECT_BY_ID)
            .bind(id.as_uuid())
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
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE library_id = $1 AND locator = $2
            "#,
        )
        .bind(library_id.as_uuid())
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
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE item_id = $1
            ORDER BY locator ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(item_id.as_uuid())
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
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE library_id = $1
            ORDER BY locator ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_media_source).collect()
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
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint,
                EXISTS (
                    SELECT 1
                    FROM source_states
                    WHERE source_states.library_id = media_sources.library_id
                      AND source_states.source_id = media_sources.id
                      AND source_states.tombstoned
                ) AS stale
            FROM media_sources
            WHERE library_id = $1
              AND fingerprint = $2
              AND fingerprint IS NOT NULL
              AND fingerprint != ''
              AND ($3::uuid IS NULL OR id != $3)
            ORDER BY id ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(fingerprint)
        .bind(exclude_source_id.map(MediaSourceId::as_uuid))
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
                COUNT(*)::bigint AS total_sources,
                COALESCE(SUM(
                    CASE
                        WHEN fingerprint IS NOT NULL AND fingerprint != '' THEN 1
                        ELSE 0
                    END
                ), 0)::bigint AS fingerprinted_sources,
                COALESCE(SUM(
                    CASE
                        WHEN fingerprint LIKE 'source:v1:content_hash:%' THEN 1
                        ELSE 0
                    END
                ), 0)::bigint AS content_hash_sources
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

#[async_trait::async_trait]
impl MediaProbeRepository for PostgresStore {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO media_source_probes (source_id, duration_ms, container, bit_rate)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(source_id) DO UPDATE SET
                duration_ms = excluded.duration_ms,
                container = excluded.container,
                bit_rate = excluded.bit_rate,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(result.duration_ms.map(u64_to_i64).transpose()?)
        .bind(&result.container)
        .bind(result.bit_rate.map(u64_to_i64).transpose()?)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM media_streams WHERE source_id = $1")
            .bind(source_id.as_uuid())
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb)
                "#,
            )
            .bind(source_id.as_uuid())
            .bind(u32_to_i64(stream.index))
            .bind(kind)
            .bind(kind_key)
            .bind(&stream.codec)
            .bind(&stream.language)
            .bind(stream.duration_ms.map(u64_to_i64).transpose()?)
            .bind(stream.bit_rate.map(u64_to_i64).transpose()?)
            .bind(stream.width.map(u32_to_i64))
            .bind(stream.height.map(u32_to_i64))
            .bind(stream.channels.map(u32_to_i64))
            .bind(stream.sample_rate.map(u32_to_i64))
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
            WHERE source_id = $1
            "#,
        )
        .bind(source_id.as_uuid())
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
                technical_json::text AS technical_json
            FROM media_streams
            WHERE source_id = $1
            ORDER BY stream_index ASC
            "#,
        )
        .bind(source_id.as_uuid())
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

#[async_trait::async_trait]
impl LocalInferenceRepository for PostgresStore {
    async fn upsert_local_inference_evidence(
        &self,
        evidence: &LocalInferenceEvidence,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_local_inference_evidence_tx(&mut transaction, evidence).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_local_inference_evidence(
        &self,
        id: LocalInferenceEvidenceId,
    ) -> Result<Option<LocalInferenceEvidence>> {
        let row = sqlx::query(LOCAL_INFERENCE_EVIDENCE_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_local_inference_evidence).transpose()
    }

    async fn list_local_inference_evidence_for_source(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<LocalInferenceEvidence>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                inferred_kind,
                inferred_title,
                inferred_year,
                inferred_season,
                inferred_episode,
                confidence_milli,
                evidence_source,
                evidence_source_key,
                evidence_value,
                inference_version
            FROM local_inference_evidence
            WHERE source_id = $1
            ORDER BY inference_version ASC, created_at ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_local_inference_evidence)
            .collect()
    }
}

#[async_trait::async_trait]
impl IngestionFailureRepository for PostgresStore {
    async fn record_ingestion_failure(
        &self,
        failure: NewIngestionFailure,
    ) -> Result<IngestionFailureRecord> {
        sqlx::query(
            r#"
            INSERT INTO ingestion_failures (
                library_id, phase, target_uri, target_kind, job_id, scan_id,
                source_id, failure_class, status, message, retryable, attempts,
                first_failed_at_ms, last_failed_at_ms, resolved_at_ms, ignored_at_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1, $12, $12, NULL, NULL)
            ON CONFLICT(library_id, phase, target_uri) DO UPDATE SET
                target_kind = excluded.target_kind,
                job_id = excluded.job_id,
                scan_id = excluded.scan_id,
                source_id = excluded.source_id,
                failure_class = excluded.failure_class,
                status = excluded.status,
                message = excluded.message,
                retryable = excluded.retryable,
                attempts = ingestion_failures.attempts + 1,
                last_failed_at_ms = excluded.last_failed_at_ms,
                resolved_at_ms = NULL,
                ignored_at_ms = NULL,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(failure.library_id.as_uuid())
        .bind(failure.phase.as_str())
        .bind(&failure.target_uri)
        .bind(&failure.target_kind)
        .bind(failure.job_id.map(|id| id.as_uuid()))
        .bind(failure.scan_id.map(|id| id.as_uuid()))
        .bind(failure.source_id.map(|id| id.as_uuid()))
        .bind(failure.failure_class.as_str())
        .bind(IngestionFailureStatus::Open.as_str())
        .bind(&failure.message)
        .bind(failure.retryable)
        .bind(failure.failed_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_ingestion_failure(failure.library_id, failure.phase, &failure.target_uri)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "ingestion_failure",
                id: format!(
                    "{}:{}:{}",
                    failure.library_id,
                    failure.phase.as_str(),
                    failure.target_uri
                ),
            })
    }

    async fn resolve_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        resolved_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.update_ingestion_failure_status(
            library_id,
            phase,
            target_uri,
            IngestionFailureStatus::Resolved,
            Some(resolved_at_ms),
            None,
        )
        .await
    }

    async fn ignore_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        ignored_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.update_ingestion_failure_status(
            library_id,
            phase,
            target_uri,
            IngestionFailureStatus::Ignored,
            None,
            Some(ignored_at_ms),
        )
        .await
    }

    async fn list_ingestion_failures(
        &self,
        filter: IngestionFailureFilter,
        page: PageRequest,
    ) -> Result<Vec<IngestionFailureRecord>> {
        let page = page.clamped();
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let phase = filter.phase.map(|phase| phase.as_str().to_owned());
        let status = filter.status.map(|status| status.as_str().to_owned());
        let rows = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                phase,
                target_uri,
                target_kind,
                job_id::text AS job_id,
                scan_id::text AS scan_id,
                source_id::text AS source_id,
                failure_class,
                status,
                message,
                retryable,
                attempts,
                first_failed_at_ms,
                last_failed_at_ms,
                resolved_at_ms,
                ignored_at_ms
            FROM ingestion_failures
            WHERE ($1::uuid IS NULL OR library_id = $1)
              AND ($2::text IS NULL OR phase = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY last_failed_at_ms DESC, target_uri ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(library_id)
        .bind(phase.as_deref())
        .bind(status.as_deref())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_ingestion_failure).collect()
    }

    async fn count_ingestion_failures(
        &self,
        library_id: LibraryId,
        phase: Option<IngestionFailurePhase>,
        status: IngestionFailureStatus,
    ) -> Result<u64> {
        let phase = phase.map(|phase| phase.as_str().to_owned());
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM ingestion_failures
            WHERE library_id = $1
              AND ($2::text IS NULL OR phase = $2)
              AND status = $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(phase.as_deref())
        .bind(status.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        i64_to_u64(count)
    }
}

#[async_trait::async_trait]
impl ScanRepository for PostgresStore {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            INSERT INTO scan_snapshots (id, library_id, root, status)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id.as_uuid())
        .bind(library_id.as_uuid())
        .bind(root)
        .bind(ScanStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            UPDATE scan_snapshots
            SET
                status = $2,
                error = $3,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(status.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                root,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                status,
                error
            FROM scan_snapshots
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_scan_snapshot).transpose()
    }

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO directory_snapshots (
                scan_id, uri, etag, modified_at, child_count
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(scan_id, uri) DO UPDATE SET
                etag = excluded.etag,
                modified_at = excluded.modified_at,
                child_count = excluded.child_count
            "#,
        )
        .bind(snapshot.scan_id.as_uuid())
        .bind(&snapshot.uri)
        .bind(&snapshot.etag)
        .bind(&snapshot.modified_at)
        .bind(u64_to_i64(snapshot.child_count)?)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT scan_id::text AS scan_id, uri, etag, modified_at, child_count
            FROM directory_snapshots
            WHERE scan_id = $1
            ORDER BY uri ASC
            "#,
        )
        .bind(scan_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_directory_snapshot).collect()
    }

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_source_state_tx(&mut transaction, state).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn commit_library_scan_source(
        &self,
        commit: &LibraryScanSourcePersistenceCommit,
    ) -> Result<LibraryScanSourcePersistenceSummary> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        for item in &commit.items {
            upsert_media_item_tx(&mut transaction, item).await?;
        }
        upsert_media_source_tx(&mut transaction, &commit.source).await?;
        upsert_source_state_tx(&mut transaction, &commit.source_state).await?;
        for state in &commit.library_item_states {
            upsert_library_item_state_tx(&mut transaction, state).await?;
        }
        for evidence in &commit.local_inference_evidence {
            upsert_local_inference_evidence_tx(&mut transaction, evidence).await?;
        }
        for projection in &commit.search_projections {
            upsert_search_projection_tx(&mut transaction, projection).await?;
        }
        for relationship in &commit.source_duplicate_relationships {
            upsert_source_duplicate_relationship_tx(&mut *transaction, relationship).await?;
        }
        let mut resolved_ingestion_failures = 0;
        for resolution in &commit.resolved_ingestion_failures {
            resolved_ingestion_failures += resolve_ingestion_failure_tx(
                &mut transaction,
                resolution.library_id,
                resolution.phase,
                &resolution.target_uri,
                resolution.resolved_at_ms,
            )
            .await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(LibraryScanSourcePersistenceSummary {
            item_ids: commit.items.iter().map(|item| item.id).collect(),
            source_id: commit.source.id,
            library_item_states: commit.library_item_states.len() as u64,
            local_inference_evidence: commit.local_inference_evidence.len() as u64,
            search_projections: commit.search_projections.len() as u64,
            source_duplicate_relationships: commit.source_duplicate_relationships.len() as u64,
            resolved_ingestion_failures,
        })
    }

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                source_id::text AS source_id,
                uri,
                size_bytes,
                modified_at,
                etag,
                fingerprint,
                last_seen_scan_id::text AS last_seen_scan_id,
                tombstoned
            FROM source_states
            WHERE library_id = $1 AND uri = $2
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_source_state).transpose()
    }

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                source_id::text AS source_id,
                uri,
                size_bytes,
                modified_at,
                etag,
                fingerprint,
                last_seen_scan_id::text AS last_seen_scan_id,
                tombstoned
            FROM source_states
            WHERE library_id = $1
            ORDER BY uri ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_source_state).collect()
    }
}

#[async_trait::async_trait]
impl SearchIndex for PostgresStore {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let mut projection =
            CatalogSearchProjection::new(document.item_id, document.title, document.body);
        projection.projection_version = document.projection_version;
        projection.aliases = document.aliases;
        projection.browse_facets = document.browse_facets;

        upsert_search_projection_tx(&mut transaction, &projection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn delete(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM search_documents WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                projection_version,
                title,
                body,
                aliases_json::text AS aliases_json,
                facets_json::text AS facets_json
            FROM search_documents
            ORDER BY title ASC, item_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let documents = rows
            .into_iter()
            .map(|row| {
                let aliases_json: String = row_get(&row, "aliases_json")?;
                let facets_json: String = row_get(&row, "facets_json")?;
                Ok(SearchEvaluationDocument::from_facet_labels(
                    parse_id(row_get::<String>(&row, "item_id")?)?,
                    i64_to_u16(row_get::<i64>(&row, "projection_version")?)?,
                    row_get::<String>(&row, "title")?,
                    row_get::<String>(&row, "body")?,
                    serde_json::from_str(&aliases_json).map_err(database_error)?,
                    serde_json::from_str(&facets_json).map_err(database_error)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(evaluate_search_documents(&query, documents))
    }
}

#[async_trait::async_trait]
impl SourceDuplicateRepository for PostgresStore {
    async fn upsert_source_duplicate_relationship(
        &self,
        relationship: &SourceDuplicateRelationship,
    ) -> Result<()> {
        upsert_source_duplicate_relationship_tx(&self.pool, relationship).await
    }

    async fn get_source_duplicate_relationship(
        &self,
        id: SourceDuplicateRelationshipId,
    ) -> Result<Option<SourceDuplicateRelationship>> {
        let row = sqlx::query(SOURCE_DUPLICATE_RELATIONSHIP_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_source_duplicate_relationship).transpose()
    }

    async fn get_source_duplicate_relationship_by_pair(
        &self,
        source_id: MediaSourceId,
        duplicate_source_id: MediaSourceId,
    ) -> Result<Option<SourceDuplicateRelationship>> {
        let (source_id, duplicate_source_id) =
            SourceDuplicateRelationship::canonical_pair(source_id, duplicate_source_id);
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                duplicate_source_id::text AS duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE source_id = $1 AND duplicate_source_id = $2
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(duplicate_source_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_source_duplicate_relationship).transpose()
    }

    async fn list_source_duplicate_relationships(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<SourceDuplicateRelationship>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                duplicate_source_id::text AS duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE source_id = $1 OR duplicate_source_id = $1
            ORDER BY source_id ASC, duplicate_source_id ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_source_duplicate_relationship)
            .collect()
    }
}

async fn upsert_source_duplicate_relationship_tx<'e, E>(
    executor: E,
    relationship: &SourceDuplicateRelationship,
) -> Result<()>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    let relationship = relationship.canonicalized();
    let (evidence_kind, evidence_kind_key) =
        source_duplicate_evidence_kind_to_parts(&relationship.evidence_kind);

    sqlx::query(
        r#"
            INSERT INTO source_duplicate_relationships (
                id,
                source_id,
                duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(source_id, duplicate_source_id) DO UPDATE SET
                evidence_kind = excluded.evidence_kind,
                evidence_kind_key = excluded.evidence_kind_key,
                evidence_value = excluded.evidence_value,
                status = excluded.status,
                confidence_milli = excluded.confidence_milli,
                updated_at = statement_timestamp()
            "#,
    )
    .bind(relationship.id.as_uuid())
    .bind(relationship.source_id.as_uuid())
    .bind(relationship.duplicate_source_id.as_uuid())
    .bind(evidence_kind)
    .bind(evidence_kind_key)
    .bind(&relationship.evidence_value)
    .bind(relationship.status.as_str())
    .bind(relationship.confidence_milli.map(i64::from))
    .execute(executor)
    .await
    .map_err(database_error)?;

    Ok(())
}

#[async_trait::async_trait]
impl CatalogGovernanceRepository for PostgresStore {
    async fn list_catalog_governance_items(
        &self,
        filter: CatalogGovernanceItemListFilter,
        page: PageRequest,
    ) -> Result<Vec<CatalogGovernanceItemRecord>> {
        let page = page.clamped();
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let rows = sqlx::query(
            r#"
            SELECT
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json,
                media_sources.library_id::text AS governance_library_id,
                COUNT(DISTINCT media_sources.id) AS source_count,
                (
                    SELECT representative.id::text
                    FROM media_sources AS representative
                    WHERE representative.item_id = media_items.id
                      AND representative.library_id = media_sources.library_id
                    ORDER BY representative.file_name ASC, representative.id ASC
                    LIMIT 1
                ) AS representative_source_id,
                (
                    SELECT representative.file_name
                    FROM media_sources AS representative
                    WHERE representative.item_id = media_items.id
                      AND representative.library_id = media_sources.library_id
                    ORDER BY representative.file_name ASC, representative.id ASC
                    LIMIT 1
                ) AS representative_file_name,
                (
                    SELECT COUNT(*)
                    FROM provider_mappings
                    WHERE provider_mappings.item_id = media_items.id
                ) AS provider_mapping_count,
                (
                    SELECT COUNT(*)
                    FROM provider_mappings
                    WHERE provider_mappings.item_id = media_items.id
                      AND provider_mappings.status = 'accepted'
                ) AS accepted_provider_mapping_count,
                (
                    SELECT COUNT(DISTINCT duplicate.id)
                    FROM source_duplicate_relationships AS duplicate
                    INNER JOIN media_sources AS duplicate_source
                        ON duplicate_source.id = duplicate.source_id
                        OR duplicate_source.id = duplicate.duplicate_source_id
                    WHERE duplicate_source.item_id = media_items.id
                      AND duplicate_source.library_id = media_sources.library_id
                ) AS duplicate_relationship_count,
                (
                    SELECT MAX(COALESCE(evidence.confidence_milli, 0))
                    FROM local_inference_evidence AS evidence
                    INNER JOIN media_sources AS evidence_source
                        ON evidence_source.id = evidence.source_id
                    WHERE evidence_source.item_id = media_items.id
                      AND evidence_source.library_id = media_sources.library_id
                ) AS best_confidence_milli
            FROM media_items
            INNER JOIN media_sources
                ON media_sources.item_id = media_items.id
            WHERE ($1::uuid IS NULL OR media_sources.library_id = $1)
            GROUP BY media_items.id, media_sources.library_id
            HAVING media_items.kind = $2
                OR (
                    (
                        SELECT MAX(COALESCE(evidence.confidence_milli, 0))
                        FROM local_inference_evidence AS evidence
                        INNER JOIN media_sources AS evidence_source
                            ON evidence_source.id = evidence.source_id
                        WHERE evidence_source.item_id = media_items.id
                          AND evidence_source.library_id = media_sources.library_id
                    ) IS NOT NULL
                    AND (
                        SELECT MAX(COALESCE(evidence.confidence_milli, 0))
                        FROM local_inference_evidence AS evidence
                        INNER JOIN media_sources AS evidence_source
                            ON evidence_source.id = evidence.source_id
                        WHERE evidence_source.item_id = media_items.id
                          AND evidence_source.library_id = media_sources.library_id
                    ) <= $3
                )
                OR (
                    SELECT COUNT(DISTINCT duplicate.id)
                    FROM source_duplicate_relationships AS duplicate
                    INNER JOIN media_sources AS duplicate_source
                        ON duplicate_source.id = duplicate.source_id
                        OR duplicate_source.id = duplicate.duplicate_source_id
                    WHERE duplicate_source.item_id = media_items.id
                      AND duplicate_source.library_id = media_sources.library_id
                ) > 0
            ORDER BY
                CASE WHEN media_items.kind = $2 THEN 0 ELSE 1 END ASC,
                best_confidence_milli ASC,
                media_items.title ASC,
                media_items.id ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(library_id)
        .bind(MediaKind::Unknown.as_str())
        .bind(i64::from(filter.max_confidence_milli))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(self.governance_row_to_record(row).await?);
        }

        Ok(records)
    }
}

pub(super) async fn upsert_media_item_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item: &MediaItem,
) -> Result<()> {
    let metadata_json = serde_json::to_string(&item.metadata).map_err(database_error)?;

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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb)
        ON CONFLICT(id) DO UPDATE SET
            kind = excluded.kind,
            parent_id = excluded.parent_id,
            title = excluded.title,
            original_title = excluded.original_title,
            sort_title = excluded.sort_title,
            overview = excluded.overview,
            release_date = excluded.release_date,
            metadata_json = excluded.metadata_json,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(item.id.as_uuid())
    .bind(item.kind.as_str())
    .bind(item.parent_id.map(|id| id.as_uuid()))
    .bind(&item.metadata.title)
    .bind(&item.metadata.original_title)
    .bind(&item.metadata.sort_title)
    .bind(&item.metadata.overview)
    .bind(&item.metadata.release_date)
    .bind(metadata_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM media_item_external_ids WHERE item_id = $1")
        .bind(item.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &item.metadata.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO media_item_external_ids (item_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(item.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_media_source_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
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
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT(id) DO UPDATE SET
            library_id = excluded.library_id,
            item_id = excluded.item_id,
            locator = excluded.locator,
            file_name = excluded.file_name,
            size_bytes = excluded.size_bytes,
            fingerprint = excluded.fingerprint,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(source.id.as_uuid())
    .bind(source.library_id.as_uuid())
    .bind(source.item_id.as_uuid())
    .bind(&source.locator)
    .bind(&source.file_name)
    .bind(source.size_bytes.map(u64_to_i64).transpose()?)
    .bind(&source.fingerprint)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

pub(super) async fn upsert_library_item_state_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    state: &LibraryItemState,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO library_item_states (library_id, item_id, provisional)
        VALUES ($1, $2, $3)
        ON CONFLICT(library_id, item_id) DO UPDATE SET
            provisional = excluded.provisional,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(state.library_id.as_uuid())
    .bind(state.item_id.as_uuid())
    .bind(state.provisional)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_source_state_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    state: &SourceState,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO source_states (
            library_id, source_id, uri, size_bytes, modified_at, etag,
            fingerprint, last_seen_scan_id, tombstoned
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT(library_id, uri) DO UPDATE SET
            source_id = excluded.source_id,
            size_bytes = excluded.size_bytes,
            modified_at = excluded.modified_at,
            etag = excluded.etag,
            fingerprint = excluded.fingerprint,
            last_seen_scan_id = excluded.last_seen_scan_id,
            tombstoned = excluded.tombstoned,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(state.library_id.as_uuid())
    .bind(state.source_id.map(|id| id.as_uuid()))
    .bind(&state.uri)
    .bind(state.size_bytes.map(u64_to_i64).transpose()?)
    .bind(&state.modified_at)
    .bind(&state.etag)
    .bind(&state.fingerprint)
    .bind(state.last_seen_scan_id.as_uuid())
    .bind(state.tombstoned)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_local_inference_evidence_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    evidence: &LocalInferenceEvidence,
) -> Result<()> {
    let (evidence_source, evidence_source_key) =
        local_inference_evidence_source_to_parts(&evidence.evidence_source);

    sqlx::query(
        r#"
        INSERT INTO local_inference_evidence (
            id,
            source_id,
            inferred_kind,
            inferred_title,
            inferred_year,
            inferred_season,
            inferred_episode,
            confidence_milli,
            evidence_source,
            evidence_source_key,
            evidence_value,
            inference_version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT(
            source_id,
            evidence_source,
            evidence_source_key,
            inference_version
        ) DO UPDATE SET
            inferred_kind = excluded.inferred_kind,
            inferred_title = excluded.inferred_title,
            inferred_year = excluded.inferred_year,
            inferred_season = excluded.inferred_season,
            inferred_episode = excluded.inferred_episode,
            confidence_milli = excluded.confidence_milli,
            evidence_value = excluded.evidence_value,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(evidence.id.as_uuid())
    .bind(evidence.source_id.as_uuid())
    .bind(evidence.inferred_kind.as_str())
    .bind(&evidence.inferred_title)
    .bind(evidence.inferred_year.map(i64::from))
    .bind(evidence.inferred_season.map(u32_to_i64))
    .bind(evidence.inferred_episode.map(u32_to_i64))
    .bind(evidence.confidence_milli.map(i64::from))
    .bind(evidence_source)
    .bind(evidence_source_key)
    .bind(&evidence.evidence_value)
    .bind(&evidence.inference_version)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

pub(super) async fn upsert_search_projection_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    projection: &CatalogSearchProjection,
) -> Result<()> {
    let aliases_json = serde_json::to_string(&projection.aliases).map_err(database_error)?;
    let facets = projection.facet_labels();
    let facets_json = serde_json::to_string(&facets).map_err(database_error)?;
    let facets_text = facets.join(" ");
    let sort_keys_json = serde_json::to_string(&projection.sort_keys).map_err(database_error)?;
    let provider_identifiers_json =
        serde_json::to_string(&projection.provider_identifiers).map_err(database_error)?;

    sqlx::query(
        r#"
        INSERT INTO search_documents (
            item_id, projection_version, title, body, aliases_json, facets_json,
            facets_text, sort_keys_json, provider_identifiers_json
        )
        VALUES ($1, $2, $3, $4, $5::jsonb, $6::jsonb, $7, $8::jsonb, $9::jsonb)
        ON CONFLICT(item_id) DO UPDATE SET
            projection_version = excluded.projection_version,
            title = excluded.title,
            body = excluded.body,
            aliases_json = excluded.aliases_json,
            facets_json = excluded.facets_json,
            facets_text = excluded.facets_text,
            sort_keys_json = excluded.sort_keys_json,
            provider_identifiers_json = excluded.provider_identifiers_json,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(projection.item_id.as_uuid())
    .bind(i64::from(projection.projection_version))
    .bind(&projection.title)
    .bind(projection.searchable_text())
    .bind(aliases_json)
    .bind(facets_json)
    .bind(facets_text)
    .bind(sort_keys_json)
    .bind(provider_identifiers_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn resolve_ingestion_failure_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    library_id: LibraryId,
    phase: IngestionFailurePhase,
    target_uri: &str,
    resolved_at_ms: i64,
) -> Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE ingestion_failures
        SET
            status = $4,
            resolved_at_ms = $5,
            ignored_at_ms = NULL,
            updated_at = statement_timestamp()
        WHERE library_id = $1 AND phase = $2 AND target_uri = $3
        "#,
    )
    .bind(library_id.as_uuid())
    .bind(phase.as_str())
    .bind(target_uri)
    .bind(IngestionFailureStatus::Resolved.as_str())
    .bind(resolved_at_ms)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(result.rows_affected())
}

impl PostgresStore {
    pub(super) async fn rows_to_media_items(&self, rows: Vec<PgRow>) -> Result<Vec<MediaItem>> {
        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let id = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self.list_external_ids(id).await?;
            items.push(row_to_media_item(row, external_ids)?);
        }

        Ok(items)
    }

    async fn list_external_ids(&self, item_id: MediaItemId) -> Result<Vec<ExternalId>> {
        let rows = sqlx::query(
            r#"
            SELECT provider, provider_key, value
            FROM media_item_external_ids
            WHERE item_id = $1
            ORDER BY provider ASC, provider_key ASC, value ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(ExternalId {
                    provider: provider_from_parts(
                        row_get(&row, "provider")?,
                        row_get(&row, "provider_key")?,
                    ),
                    value: row_get(&row, "value")?,
                })
            })
            .collect()
    }

    pub(super) async fn list_catalog_external_ids<T>(
        &self,
        table: &str,
        owner_column: &str,
        owner_id: T,
    ) -> Result<Vec<ExternalId>>
    where
        T: Display,
    {
        validate_catalog_external_id_lookup(table, owner_column)?;
        let query = format!(
            "SELECT provider, provider_key, value FROM {table} WHERE {owner_column} = $1::uuid ORDER BY provider ASC, provider_key ASC, value ASC"
        );
        let rows = sqlx::query(&query)
            .bind(owner_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(ExternalId {
                    provider: provider_from_parts(
                        row_get(&row, "provider")?,
                        row_get(&row, "provider_key")?,
                    ),
                    value: row_get(&row, "value")?,
                })
            })
            .collect()
    }

    async fn governance_row_to_record(&self, row: PgRow) -> Result<CatalogGovernanceItemRecord> {
        let item_id = parse_id(row_get::<String>(&row, "id")?)?;
        let library_id = parse_id(row_get::<String>(&row, "governance_library_id")?)?;
        let source_count = i64_to_u32(row_get(&row, "source_count")?)?;
        let representative_source_id =
            parse_optional_id(row_get::<Option<String>>(&row, "representative_source_id")?)?;
        let representative_file_name = row_get(&row, "representative_file_name")?;
        let provider_mapping_count = i64_to_u32(row_get(&row, "provider_mapping_count")?)?;
        let accepted_provider_mapping_count =
            i64_to_u32(row_get(&row, "accepted_provider_mapping_count")?)?;
        let duplicate_relationship_count =
            i64_to_u32(row_get(&row, "duplicate_relationship_count")?)?;
        let external_ids = self.list_external_ids(item_id).await?;
        let item = row_to_media_item(row, external_ids)?;
        let best_local_inference = self
            .best_local_inference_evidence_for_item_library(item.id, library_id)
            .await?;

        Ok(CatalogGovernanceItemRecord {
            item,
            library_id,
            source_count,
            representative_source_id,
            representative_file_name,
            best_local_inference,
            provider_mapping_count,
            accepted_provider_mapping_count,
            duplicate_relationship_count,
        })
    }

    async fn best_local_inference_evidence_for_item_library(
        &self,
        item_id: MediaItemId,
        library_id: LibraryId,
    ) -> Result<Option<LocalInferenceEvidence>> {
        let row = sqlx::query(
            r#"
            SELECT
                evidence.id::text AS id,
                evidence.source_id::text AS source_id,
                evidence.inferred_kind,
                evidence.inferred_title,
                evidence.inferred_year,
                evidence.inferred_season,
                evidence.inferred_episode,
                evidence.confidence_milli,
                evidence.evidence_source,
                evidence.evidence_source_key,
                evidence.evidence_value,
                evidence.inference_version
            FROM local_inference_evidence AS evidence
            INNER JOIN media_sources AS source
                ON source.id = evidence.source_id
            WHERE source.item_id = $1
              AND source.library_id = $2
            ORDER BY
                evidence.confidence_milli IS NULL ASC,
                evidence.confidence_milli DESC,
                evidence.updated_at DESC,
                evidence.inference_version DESC,
                evidence.id ASC
            LIMIT 1
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(library_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_local_inference_evidence).transpose()
    }

    async fn get_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
    ) -> Result<Option<IngestionFailureRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                phase,
                target_uri,
                target_kind,
                job_id::text AS job_id,
                scan_id::text AS scan_id,
                source_id::text AS source_id,
                failure_class,
                status,
                message,
                retryable,
                attempts,
                first_failed_at_ms,
                last_failed_at_ms,
                resolved_at_ms,
                ignored_at_ms
            FROM ingestion_failures
            WHERE library_id = $1 AND phase = $2 AND target_uri = $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(phase.as_str())
        .bind(target_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_ingestion_failure).transpose()
    }

    async fn update_ingestion_failure_status(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        status: IngestionFailureStatus,
        resolved_at_ms: Option<i64>,
        ignored_at_ms: Option<i64>,
    ) -> Result<Option<IngestionFailureRecord>> {
        sqlx::query(
            r#"
            UPDATE ingestion_failures
            SET
                status = $4,
                resolved_at_ms = $5,
                ignored_at_ms = $6,
                updated_at = statement_timestamp()
            WHERE library_id = $1 AND phase = $2 AND target_uri = $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(phase.as_str())
        .bind(target_uri)
        .bind(status.as_str())
        .bind(resolved_at_ms)
        .bind(ignored_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_ingestion_failure(library_id, phase, target_uri)
            .await
    }
}

fn row_to_library(row: PgRow) -> Result<Library> {
    let roots_json: String = row_get(&row, "roots_json")?;
    let options_json: String = row_get(&row, "options_json")?;

    Ok(Library {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        roots: serde_json::from_str(&roots_json).map_err(database_error)?,
        options: serde_json::from_str(&options_json).map_err(database_error)?,
    })
}

fn row_to_library_item_state(row: PgRow) -> Result<LibraryItemState> {
    Ok(LibraryItemState {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provisional: row_get(&row, "provisional")?,
    })
}

fn row_to_media_item(row: PgRow, external_ids: Vec<ExternalId>) -> Result<MediaItem> {
    let metadata_json = row_get::<Option<String>>(&row, "metadata_json")?;
    let mut metadata = match metadata_json {
        Some(value) => serde_json::from_str::<CanonicalMetadata>(&value).map_err(database_error)?,
        None => CanonicalMetadata {
            title: row_get(&row, "title")?,
            original_title: row_get(&row, "original_title")?,
            sort_title: row_get(&row, "sort_title")?,
            overview: row_get(&row, "overview")?,
            release_date: row_get(&row, "release_date")?,
            ..CanonicalMetadata::default()
        },
    };
    metadata.external_ids = external_ids;

    Ok(MediaItem {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        kind: parse_media_kind(row_get(&row, "kind")?)?,
        parent_id: parse_optional_id(row_get::<Option<String>>(&row, "parent_id")?)?,
        metadata,
    })
}

fn row_to_media_source(row: PgRow) -> Result<MediaSource> {
    Ok(MediaSource {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        locator: row_get(&row, "locator")?,
        file_name: row_get(&row, "file_name")?,
        size_bytes: row_get::<Option<i64>>(&row, "size_bytes")?
            .map(i64_to_u64)
            .transpose()?,
        fingerprint: row_get(&row, "fingerprint")?,
    })
}

fn row_to_media_source_fingerprint_match(row: PgRow) -> Result<MediaSourceFingerprintMatch> {
    let stale = row_get(&row, "stale")?;
    let source = row_to_media_source(row)?;

    Ok(MediaSourceFingerprintMatch { source, stale })
}

fn row_to_stream_info(row: PgRow) -> Result<MediaStreamInfo> {
    Ok(MediaStreamInfo {
        index: i64_to_u32(row_get(&row, "stream_index")?)?,
        kind: stream_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        codec: row_get(&row, "codec")?,
        language: row_get(&row, "language")?,
        duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
        bit_rate: optional_i64_to_u64(row_get(&row, "bit_rate")?)?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        channels: optional_i64_to_u32(row_get(&row, "channels")?)?,
        sample_rate: optional_i64_to_u32(row_get(&row, "sample_rate")?)?,
        technical: deserialize_stream_technical_json(row_get(&row, "technical_json")?)?,
    })
}

fn serialize_stream_technical_json(value: &MediaStreamTechnicalFacts) -> Result<String> {
    serde_json::to_string(value).map_err(database_error)
}

fn deserialize_stream_technical_json(value: Option<String>) -> Result<MediaStreamTechnicalFacts> {
    match value {
        Some(value) if !value.trim().is_empty() => {
            serde_json::from_str(&value).map_err(database_error)
        }
        _ => Ok(MediaStreamTechnicalFacts::default()),
    }
}

fn postgres_browse_watch_state_where(filter: LibraryItemWatchStateFilter) -> &'static str {
    match filter {
        LibraryItemWatchStateFilter::Any => "",
        LibraryItemWatchStateFilter::Watched => "\n              AND playback.watched = true",
        LibraryItemWatchStateFilter::Unwatched => {
            "\n              AND (playback.item_id IS NULL OR playback.watched = false)"
        }
        LibraryItemWatchStateFilter::InProgress => {
            r#"
              AND playback.watched = false
              AND playback.resume_position_ms IS NOT NULL
              AND playback.resume_position_ms > 0"#
        }
    }
}

fn postgres_browse_order_by(
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

fn row_to_local_inference_evidence(row: PgRow) -> Result<LocalInferenceEvidence> {
    Ok(LocalInferenceEvidence {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        inferred_kind: parse_media_kind(row_get(&row, "inferred_kind")?)?,
        inferred_title: row_get(&row, "inferred_title")?,
        inferred_year: optional_i64_to_i32(row_get(&row, "inferred_year")?)?,
        inferred_season: optional_i64_to_u32(row_get(&row, "inferred_season")?)?,
        inferred_episode: optional_i64_to_u32(row_get(&row, "inferred_episode")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
        evidence_source: local_inference_evidence_source_from_parts(
            row_get(&row, "evidence_source")?,
            row_get(&row, "evidence_source_key")?,
        ),
        evidence_value: row_get(&row, "evidence_value")?,
        inference_version: row_get(&row, "inference_version")?,
    })
}

fn row_to_scan_snapshot(row: PgRow) -> Result<ScanSnapshot> {
    Ok(ScanSnapshot {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        root: row_get(&row, "root")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
        status: ScanStatus::parse(&row_get::<String>(&row, "status")?)?,
        error: row_get(&row, "error")?,
    })
}

fn row_to_directory_snapshot(row: PgRow) -> Result<DirectorySnapshot> {
    Ok(DirectorySnapshot {
        scan_id: parse_id(row_get::<String>(&row, "scan_id")?)?,
        uri: row_get(&row, "uri")?,
        etag: row_get(&row, "etag")?,
        modified_at: row_get(&row, "modified_at")?,
        child_count: i64_to_u64(row_get(&row, "child_count")?)?,
    })
}

fn row_to_source_state(row: PgRow) -> Result<SourceState> {
    Ok(SourceState {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        uri: row_get(&row, "uri")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        modified_at: row_get(&row, "modified_at")?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        last_seen_scan_id: parse_id(row_get::<String>(&row, "last_seen_scan_id")?)?,
        tombstoned: row_get(&row, "tombstoned")?,
    })
}

fn row_to_ingestion_failure(row: PgRow) -> Result<IngestionFailureRecord> {
    Ok(IngestionFailureRecord {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        job_id: parse_optional_id(row_get::<Option<String>>(&row, "job_id")?)?,
        scan_id: parse_optional_id(row_get::<Option<String>>(&row, "scan_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        phase: IngestionFailurePhase::parse(&row_get::<String>(&row, "phase")?)?,
        target_uri: row_get(&row, "target_uri")?,
        target_kind: row_get(&row, "target_kind")?,
        failure_class: IngestionFailureClass::parse(&row_get::<String>(&row, "failure_class")?)?,
        status: IngestionFailureStatus::parse(&row_get::<String>(&row, "status")?)?,
        message: row_get(&row, "message")?,
        retryable: row_get(&row, "retryable")?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        first_failed_at_ms: row_get(&row, "first_failed_at_ms")?,
        last_failed_at_ms: row_get(&row, "last_failed_at_ms")?,
        resolved_at_ms: row_get(&row, "resolved_at_ms")?,
        ignored_at_ms: row_get(&row, "ignored_at_ms")?,
    })
}

fn row_to_source_duplicate_relationship(row: PgRow) -> Result<SourceDuplicateRelationship> {
    Ok(SourceDuplicateRelationship {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        duplicate_source_id: parse_id(row_get::<String>(&row, "duplicate_source_id")?)?,
        evidence_kind: source_duplicate_evidence_kind_from_parts(
            row_get(&row, "evidence_kind")?,
            row_get(&row, "evidence_kind_key")?,
        ),
        evidence_value: row_get(&row, "evidence_value")?,
        status: SourceDuplicateRelationshipStatus::parse(&row_get::<String>(&row, "status")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
    })
}

fn parse_media_kind(value: String) -> Result<MediaKind> {
    match value.as_str() {
        "movie" => Ok(MediaKind::Movie),
        "series" => Ok(MediaKind::Series),
        "season" => Ok(MediaKind::Season),
        "episode" => Ok(MediaKind::Episode),
        "collection" => Ok(MediaKind::Collection),
        "extra" => Ok(MediaKind::Extra),
        "unknown" => Ok(MediaKind::Unknown),
        _ => Err(NakoError::Database {
            message: format!("unknown media kind stored in PostgreSQL database: {value}"),
        }),
    }
}

fn validate_catalog_external_id_lookup(table: &str, owner_column: &str) -> Result<()> {
    let valid = matches!(
        (table, owner_column),
        ("person_external_ids", "person_id")
            | ("collection_external_ids", "collection_id")
            | ("studio_external_ids", "studio_id")
    );
    if valid {
        Ok(())
    } else {
        Err(NakoError::Database {
            message: format!(
                "invalid PostgreSQL catalog external-id lookup: {table}.{owner_column}"
            ),
        })
    }
}

fn source_duplicate_evidence_kind_to_parts(kind: &SourceDuplicateEvidenceKind) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

fn source_duplicate_evidence_kind_from_parts(
    kind: String,
    kind_key: String,
) -> SourceDuplicateEvidenceKind {
    SourceDuplicateEvidenceKind::from_parts(&kind, kind_key)
}

fn stream_kind_to_parts(kind: &MediaStreamKind) -> (String, String) {
    match kind {
        MediaStreamKind::Video => ("video".to_owned(), String::new()),
        MediaStreamKind::Audio => ("audio".to_owned(), String::new()),
        MediaStreamKind::Subtitle => ("subtitle".to_owned(), String::new()),
        MediaStreamKind::Data => ("data".to_owned(), String::new()),
        MediaStreamKind::Attachment => ("attachment".to_owned(), String::new()),
        MediaStreamKind::Other(value) => ("other".to_owned(), value.clone()),
    }
}

fn stream_kind_from_parts(kind: String, kind_key: String) -> MediaStreamKind {
    match kind.as_str() {
        "video" => MediaStreamKind::Video,
        "audio" => MediaStreamKind::Audio,
        "subtitle" => MediaStreamKind::Subtitle,
        "data" => MediaStreamKind::Data,
        "attachment" => MediaStreamKind::Attachment,
        "other" => MediaStreamKind::Other(kind_key),
        _ => MediaStreamKind::Other(kind),
    }
}

fn local_inference_evidence_source_to_parts(
    source: &LocalInferenceEvidenceSource,
) -> (String, String) {
    let (source, source_key) = source.as_parts();
    (source.to_owned(), source_key.to_owned())
}

fn local_inference_evidence_source_from_parts(
    source: String,
    source_key: String,
) -> LocalInferenceEvidenceSource {
    LocalInferenceEvidenceSource::from_parts(&source, source_key)
}
