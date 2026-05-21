use sqlx::sqlite::SqliteRow;

use super::{SqliteStore, codec::*};
use taru_core::*;

#[async_trait::async_trait]
impl ManagedImportRepository for SqliteStore {
    async fn upsert_managed_import_artifact(
        &self,
        artifact: NewManagedImportArtifact,
    ) -> Result<ManagedImportArtifactRecord> {
        let (source_kind, source_kind_key) =
            managed_import_source_kind_to_parts(&artifact.source_kind);
        sqlx::query(
            r#"
            INSERT INTO managed_import_artifacts (
                id, target_library_id, source_kind, source_kind_key, source_uri,
                staging_manifest_id, artifact_uri, original_file_name, intended_locator,
                size_bytes, fingerprint, state, diagnostics_json, created_at_ms, updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                target_library_id = excluded.target_library_id,
                source_kind = excluded.source_kind,
                source_kind_key = excluded.source_kind_key,
                source_uri = excluded.source_uri,
                staging_manifest_id = excluded.staging_manifest_id,
                artifact_uri = excluded.artifact_uri,
                original_file_name = excluded.original_file_name,
                intended_locator = excluded.intended_locator,
                size_bytes = excluded.size_bytes,
                fingerprint = excluded.fingerprint,
                state = excluded.state,
                diagnostics_json = excluded.diagnostics_json,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(artifact.id.to_string())
        .bind(artifact.target_library_id.to_string())
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(&artifact.source_uri)
        .bind(artifact.staging_manifest_id.map(|id| id.to_string()))
        .bind(&artifact.artifact_uri)
        .bind(&artifact.original_file_name)
        .bind(&artifact.intended_locator)
        .bind(optional_u64_to_i64(artifact.size_bytes)?)
        .bind(&artifact.fingerprint)
        .bind(artifact.state.as_str())
        .bind(&artifact.diagnostics_json)
        .bind(artifact.created_at_ms)
        .bind(artifact.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_managed_import_artifact(artifact.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_artifact",
                id: artifact.id.to_string(),
            })
    }

    async fn get_managed_import_artifact(
        &self,
        id: ManagedImportArtifactId,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM managed_import_artifacts
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_managed_import_artifact).transpose()
    }

    async fn find_managed_import_artifact_by_source(
        &self,
        target_library_id: LibraryId,
        source_kind: &ManagedImportSourceKind,
        source_uri: &str,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        let (source_kind, source_kind_key) = managed_import_source_kind_to_parts(source_kind);
        let row = sqlx::query(
            r#"
            SELECT *
            FROM managed_import_artifacts
            WHERE target_library_id = ?1
              AND source_kind = ?2
              AND source_kind_key = ?3
              AND source_uri = ?4
            "#,
        )
        .bind(target_library_id.to_string())
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_managed_import_artifact).transpose()
    }

    async fn list_managed_import_artifacts(
        &self,
        filter: ManagedImportArtifactListFilter,
        page: PageRequest,
    ) -> Result<Vec<ManagedImportArtifactRecord>> {
        let page = page.clamped();
        let target_library_id = filter.target_library_id.map(|id| id.to_string());
        let state = filter.state.map(ManagedImportArtifactState::as_str);
        let (source_kind, source_kind_key) = filter
            .source_kind
            .as_ref()
            .map(managed_import_source_kind_to_parts)
            .map_or((None, None), |(kind, kind_key)| {
                (Some(kind), Some(kind_key))
            });

        let rows = sqlx::query(
            r#"
            SELECT *
            FROM managed_import_artifacts
            WHERE (?1 IS NULL OR target_library_id = ?1)
              AND (?2 IS NULL OR state = ?2)
              AND (?3 IS NULL OR (source_kind = ?3 AND source_kind_key = ?4))
            ORDER BY updated_at_ms DESC, id ASC
            LIMIT ?5 OFFSET ?6
            "#,
        )
        .bind(target_library_id)
        .bind(state)
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_managed_import_artifact)
            .collect()
    }

    async fn set_managed_import_artifact_state(
        &self,
        id: ManagedImportArtifactId,
        state: ManagedImportArtifactState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        sqlx::query(
            r#"
            UPDATE managed_import_artifacts
            SET state = ?2,
                updated_at_ms = ?3,
                diagnostics_json = ?4,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(state.as_str())
        .bind(updated_at_ms)
        .bind(diagnostics_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_managed_import_artifact(id).await
    }
}

fn row_to_managed_import_artifact(row: SqliteRow) -> Result<ManagedImportArtifactRecord> {
    Ok(ManagedImportArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        target_library_id: parse_id(row_get::<String>(&row, "target_library_id")?)?,
        source_kind: managed_import_source_kind_from_parts(
            row_get(&row, "source_kind")?,
            row_get(&row, "source_kind_key")?,
        ),
        source_uri: row_get(&row, "source_uri")?,
        staging_manifest_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "staging_manifest_id",
        )?)?,
        artifact_uri: row_get(&row, "artifact_uri")?,
        original_file_name: row_get(&row, "original_file_name")?,
        intended_locator: row_get(&row, "intended_locator")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        fingerprint: row_get(&row, "fingerprint")?,
        state: ManagedImportArtifactState::parse(&row_get::<String>(&row, "state")?)?,
        diagnostics_json: row_get(&row, "diagnostics_json")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}
