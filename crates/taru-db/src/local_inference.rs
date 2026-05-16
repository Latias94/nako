use super::*;

#[async_trait::async_trait]
impl LocalInferenceRepository for SqliteStore {
    async fn upsert_local_inference_evidence(
        &self,
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
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                source_id = excluded.source_id,
                inferred_kind = excluded.inferred_kind,
                inferred_title = excluded.inferred_title,
                inferred_year = excluded.inferred_year,
                inferred_season = excluded.inferred_season,
                inferred_episode = excluded.inferred_episode,
                confidence_milli = excluded.confidence_milli,
                evidence_source = excluded.evidence_source,
                evidence_source_key = excluded.evidence_source_key,
                evidence_value = excluded.evidence_value,
                inference_version = excluded.inference_version,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(evidence.id.to_string())
        .bind(evidence.source_id.to_string())
        .bind(media_kind_to_str(evidence.inferred_kind))
        .bind(&evidence.inferred_title)
        .bind(optional_i32_to_i64(evidence.inferred_year))
        .bind(optional_u32_to_i64(evidence.inferred_season))
        .bind(optional_u32_to_i64(evidence.inferred_episode))
        .bind(optional_u16_to_i64(evidence.confidence_milli))
        .bind(evidence_source)
        .bind(evidence_source_key)
        .bind(&evidence.evidence_value)
        .bind(&evidence.inference_version)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_local_inference_evidence(
        &self,
        id: LocalInferenceEvidenceId,
    ) -> Result<Option<LocalInferenceEvidence>> {
        let row = sqlx::query(
            r#"
            SELECT
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
            FROM local_inference_evidence
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
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
            FROM local_inference_evidence
            WHERE source_id = ?1
            ORDER BY inference_version ASC, created_at ASC, id ASC
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
            .map(row_to_local_inference_evidence)
            .collect()
    }
}
