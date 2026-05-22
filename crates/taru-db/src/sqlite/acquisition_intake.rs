use sqlx::sqlite::SqliteRow;

use super::{SqliteStore, codec::*};
use taru_core::*;

#[async_trait::async_trait]
impl AcquisitionIntakeRepository for SqliteStore {
    async fn upsert_acquisition_intake_candidate(
        &self,
        candidate: NewAcquisitionIntakeCandidate,
    ) -> Result<AcquisitionIntakeCandidateRecord> {
        let (source_kind, source_kind_key) =
            acquisition_intake_source_kind_to_parts(&candidate.source_kind);
        sqlx::query(
            r#"
            INSERT INTO acquisition_intake_candidates (
                id, target_library_id, source_kind, source_kind_key, source_key,
                source_uri, display_name, intended_locator, size_bytes, fingerprint,
                managed_import_artifact_id, state, diagnostics_json, first_seen_at_ms,
                last_seen_at_ms, created_at_ms, updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(target_library_id, source_kind, source_kind_key, source_key) DO UPDATE SET
                source_uri = excluded.source_uri,
                display_name = excluded.display_name,
                intended_locator = excluded.intended_locator,
                size_bytes = excluded.size_bytes,
                fingerprint = excluded.fingerprint,
                managed_import_artifact_id = excluded.managed_import_artifact_id,
                state = excluded.state,
                diagnostics_json = excluded.diagnostics_json,
                first_seen_at_ms = min(acquisition_intake_candidates.first_seen_at_ms, excluded.first_seen_at_ms),
                last_seen_at_ms = excluded.last_seen_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(candidate.id.to_string())
        .bind(candidate.target_library_id.to_string())
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(&candidate.source_key)
        .bind(&candidate.source_uri)
        .bind(&candidate.display_name)
        .bind(&candidate.intended_locator)
        .bind(optional_u64_to_i64(candidate.size_bytes)?)
        .bind(&candidate.fingerprint)
        .bind(candidate.managed_import_artifact_id.map(|id| id.to_string()))
        .bind(candidate.state.as_str())
        .bind(&candidate.diagnostics_json)
        .bind(candidate.first_seen_at_ms)
        .bind(candidate.last_seen_at_ms)
        .bind(candidate.created_at_ms)
        .bind(candidate.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_acquisition_intake_candidate_by_source_key(
            candidate.target_library_id,
            &candidate.source_kind,
            &candidate.source_key,
        )
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "acquisition_intake_candidate",
            id: candidate.id.to_string(),
        })
    }

    async fn get_acquisition_intake_candidate(
        &self,
        id: AcquisitionIntakeCandidateId,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM acquisition_intake_candidates
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_acquisition_intake_candidate).transpose()
    }

    async fn find_acquisition_intake_candidate_by_source_key(
        &self,
        target_library_id: LibraryId,
        source_kind: &AcquisitionIntakeSourceKind,
        source_key: &str,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        let (source_kind, source_kind_key) = acquisition_intake_source_kind_to_parts(source_kind);
        let row = sqlx::query(
            r#"
            SELECT *
            FROM acquisition_intake_candidates
            WHERE target_library_id = ?1
              AND source_kind = ?2
              AND source_kind_key = ?3
              AND source_key = ?4
            "#,
        )
        .bind(target_library_id.to_string())
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_acquisition_intake_candidate).transpose()
    }

    async fn list_acquisition_intake_candidates(
        &self,
        filter: AcquisitionIntakeCandidateListFilter,
        page: PageRequest,
    ) -> Result<Vec<AcquisitionIntakeCandidateRecord>> {
        let page = page.clamped();
        let target_library_id = filter.target_library_id.map(|id| id.to_string());
        let state = filter.state.map(AcquisitionIntakeCandidateState::as_str);
        let (source_kind, source_kind_key) = filter
            .source_kind
            .as_ref()
            .map(acquisition_intake_source_kind_to_parts)
            .map_or((None, None), |(kind, kind_key)| {
                (Some(kind), Some(kind_key))
            });
        let managed_import_artifact_id = filter.managed_import_artifact_id.map(|id| id.to_string());

        let rows = sqlx::query(
            r#"
            SELECT *
            FROM acquisition_intake_candidates
            WHERE (?1 IS NULL OR target_library_id = ?1)
              AND (?2 IS NULL OR state = ?2)
              AND (?3 IS NULL OR (source_kind = ?3 AND source_kind_key = ?4))
              AND (?5 IS NULL OR managed_import_artifact_id = ?5)
            ORDER BY updated_at_ms DESC, id ASC
            LIMIT ?6 OFFSET ?7
            "#,
        )
        .bind(target_library_id)
        .bind(state)
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(managed_import_artifact_id)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_acquisition_intake_candidate)
            .collect()
    }

    async fn set_acquisition_intake_candidate_state(
        &self,
        id: AcquisitionIntakeCandidateId,
        state: AcquisitionIntakeCandidateState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        sqlx::query(
            r#"
            UPDATE acquisition_intake_candidates
            SET state = ?2,
                diagnostics_json = ?3,
                updated_at_ms = ?4,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(state.as_str())
        .bind(diagnostics_json)
        .bind(updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_acquisition_intake_candidate(id).await
    }

    async fn link_acquisition_intake_candidate_managed_import_artifact(
        &self,
        id: AcquisitionIntakeCandidateId,
        managed_import_artifact_id: ManagedImportArtifactId,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>> {
        sqlx::query(
            r#"
            UPDATE acquisition_intake_candidates
            SET managed_import_artifact_id = ?2,
                state = ?3,
                diagnostics_json = ?4,
                updated_at_ms = ?5,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(managed_import_artifact_id.to_string())
        .bind(AcquisitionIntakeCandidateState::Accepted.as_str())
        .bind(diagnostics_json)
        .bind(updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_acquisition_intake_candidate(id).await
    }
}

fn row_to_acquisition_intake_candidate(row: SqliteRow) -> Result<AcquisitionIntakeCandidateRecord> {
    Ok(AcquisitionIntakeCandidateRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        target_library_id: parse_id(row_get::<String>(&row, "target_library_id")?)?,
        source_kind: acquisition_intake_source_kind_from_parts(
            row_get(&row, "source_kind")?,
            row_get(&row, "source_kind_key")?,
        ),
        source_key: row_get(&row, "source_key")?,
        source_uri: row_get(&row, "source_uri")?,
        display_name: row_get(&row, "display_name")?,
        intended_locator: row_get(&row, "intended_locator")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        fingerprint: row_get(&row, "fingerprint")?,
        managed_import_artifact_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "managed_import_artifact_id",
        )?)?,
        state: AcquisitionIntakeCandidateState::parse(&row_get::<String>(&row, "state")?)?,
        diagnostics_json: row_get(&row, "diagnostics_json")?,
        first_seen_at_ms: row_get(&row, "first_seen_at_ms")?,
        last_seen_at_ms: row_get(&row, "last_seen_at_ms")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}
