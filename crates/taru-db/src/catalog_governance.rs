use super::*;
use sqlx::QueryBuilder;

const GOVERNANCE_ITEM_SELECT: &str = r#"
            SELECT
                media_items.id,
                media_items.kind,
                media_items.parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json,
                media_sources.library_id AS governance_library_id,
                COUNT(DISTINCT media_sources.id) AS source_count,
                (
                    SELECT representative.id
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
"#;

#[async_trait::async_trait]
impl CatalogGovernanceRepository for SqliteStore {
    async fn list_catalog_governance_items(
        &self,
        filter: CatalogGovernanceItemListFilter,
        page: PageRequest,
    ) -> Result<Vec<CatalogGovernanceItemRecord>> {
        let page = page.clamped();
        let mut query = QueryBuilder::new(GOVERNANCE_ITEM_SELECT);
        query.push(" WHERE 1 = 1");

        if let Some(library_id) = filter.library_id {
            query.push(" AND media_sources.library_id = ");
            query.push_bind(library_id.to_string());
        }

        query.push(
            r#"
            GROUP BY media_items.id, media_sources.library_id
            HAVING media_items.kind = "#,
        );
        query.push_bind(media_kind_to_str(MediaKind::Unknown));
        query.push(
            r#"
                OR (
                    best_confidence_milli IS NOT NULL
                    AND best_confidence_milli <= "#,
        );
        query.push_bind(i64::from(filter.max_confidence_milli));
        query.push(
            r#"
                )
            ORDER BY
                CASE WHEN media_items.kind = "#,
        );
        query.push_bind(media_kind_to_str(MediaKind::Unknown));
        query.push(
            r#"
                    THEN 0 ELSE 1 END ASC,
                best_confidence_milli ASC,
                media_items.title ASC,
                media_items.id ASC
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

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(self.governance_row_to_record(row).await?);
        }

        Ok(records)
    }
}

impl SqliteStore {
    async fn governance_row_to_record(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<CatalogGovernanceItemRecord> {
        let item_id = parse_id(row_get::<String>(&row, "id")?)?;
        let library_id = parse_id(row_get::<String>(&row, "governance_library_id")?)?;
        let source_count = i64_to_u32(row_get(&row, "source_count")?)?;
        let representative_source_id =
            parse_optional_id(row_get(&row, "representative_source_id")?)?;
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
                evidence.id,
                evidence.source_id,
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
            WHERE source.item_id = ?1
              AND source.library_id = ?2
            ORDER BY
                evidence.confidence_milli IS NULL ASC,
                evidence.confidence_milli DESC,
                evidence.updated_at DESC,
                evidence.inference_version DESC,
                evidence.id ASC
            LIMIT 1
            "#,
        )
        .bind(item_id.to_string())
        .bind(library_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_local_inference_evidence).transpose()
    }
}
