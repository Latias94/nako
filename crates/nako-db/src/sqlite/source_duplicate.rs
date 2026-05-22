use super::{SqliteStore, codec::*};
use nako_core::*;

#[async_trait::async_trait]
impl SourceDuplicateRepository for SqliteStore {
    async fn upsert_source_duplicate_relationship(
        &self,
        relationship: &SourceDuplicateRelationship,
    ) -> Result<()> {
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
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                source_id = excluded.source_id,
                duplicate_source_id = excluded.duplicate_source_id,
                evidence_kind = excluded.evidence_kind,
                evidence_kind_key = excluded.evidence_kind_key,
                evidence_value = excluded.evidence_value,
                status = excluded.status,
                confidence_milli = excluded.confidence_milli,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(relationship.id.to_string())
        .bind(relationship.source_id.to_string())
        .bind(relationship.duplicate_source_id.to_string())
        .bind(evidence_kind)
        .bind(evidence_kind_key)
        .bind(&relationship.evidence_value)
        .bind(relationship.status.as_str())
        .bind(optional_u16_to_i64(relationship.confidence_milli))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_source_duplicate_relationship(
        &self,
        id: SourceDuplicateRelationshipId,
    ) -> Result<Option<SourceDuplicateRelationship>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_id,
                duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
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
                id,
                source_id,
                duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE source_id = ?1 OR duplicate_source_id = ?1
            ORDER BY source_id ASC, duplicate_source_id ASC, id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(source_id.to_string())
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
