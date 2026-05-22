use sqlx::sqlite::SqliteRow;

use super::{SqliteStore, codec::*};
use crate::automation_proposals::{GeneratedArtifactProposalFacts, generated_artifact_proposal};
use nako_core::*;

#[async_trait::async_trait]
impl AutomationRepository for SqliteStore {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord> {
        let capabilities_json =
            serde_json::to_string(&provider.capabilities).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO automation_providers (
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                base_url = excluded.base_url,
                secret_env = excluded.secret_env,
                capabilities_json = excluded.capabilities_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(provider.id.to_string())
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(&provider.secret_env)
        .bind(capabilities_json)
        .bind(u64_to_i64(provider.timeout_ms)?)
        .bind(u32_to_i64(provider.max_attempts))
        .bind(provider.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_provider(provider.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "automation provider {} was not found after upsert",
                    provider.id
                ),
            })
    }

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM automation_providers
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_automation_provider).transpose()
    }

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status,
                created_at,
                updated_at
            FROM automation_providers
            WHERE status = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(AutomationProviderStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_provider).collect()
    }

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord> {
        sqlx::query(
            r#"
            INSERT INTO automation_artifacts (
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(artifact.id.to_string())
        .bind(artifact.job_id.to_string())
        .bind(artifact.provider_id.to_string())
        .bind(artifact.capability.as_str())
        .bind(artifact.kind.as_str())
        .bind(artifact.library_id.map(|id| id.to_string()))
        .bind(artifact.item_id.map(|id| id.to_string()))
        .bind(artifact.source_id.map(|id| id.to_string()))
        .bind(artifact.artifact_json)
        .bind(AutomationArtifactStatus::Proposed.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(artifact.id).await
    }

    async fn get_automation_artifact(
        &self,
        id: AutomationArtifactId,
    ) -> Result<Option<AutomationArtifactRecord>> {
        self.get_automation_artifact(id).await
    }

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord> {
        let query = if status == AutomationArtifactStatus::Accepted {
            r#"
            UPDATE automation_artifacts
            SET
                status = ?2,
                accepted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#
        } else {
            r#"
            UPDATE automation_artifacts
            SET
                status = ?2,
                accepted_at = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#
        };

        sqlx::query(query)
            .bind(id.to_string())
            .bind(status.as_str())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(id).await
    }

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status,
                created_at,
                updated_at,
                accepted_at
            FROM automation_artifacts
            WHERE job_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(job_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status,
                created_at,
                updated_at,
                accepted_at
            FROM automation_artifacts
            WHERE item_id = ?1
            ORDER BY created_at DESC, id DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }

    async fn list_generated_artifact_proposals(
        &self,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactProposal>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                automation_artifacts.id,
                automation_artifacts.job_id,
                automation_artifacts.provider_id,
                automation_artifacts.capability,
                automation_artifacts.kind,
                automation_artifacts.library_id,
                automation_artifacts.item_id,
                automation_artifacts.source_id,
                automation_artifacts.artifact_json,
                automation_artifacts.status,
                automation_artifacts.created_at,
                automation_artifacts.updated_at,
                automation_artifacts.accepted_at,
                automation_providers.id AS provider_exists_id,
                automation_providers.name AS provider_name,
                jobs.id AS job_exists_id,
                jobs.input_json AS job_input_json,
                jobs.summary_json AS job_summary_json,
                libraries.id AS library_exists_id,
                media_items.id AS item_exists_id,
                media_sources.id AS source_exists_id,
                media_sources.library_id AS source_library_id,
                media_sources.item_id AS source_item_id
            FROM automation_artifacts
            LEFT JOIN automation_providers
                ON automation_providers.id = automation_artifacts.provider_id
            LEFT JOIN jobs
                ON jobs.id = automation_artifacts.job_id
            LEFT JOIN libraries
                ON libraries.id = automation_artifacts.library_id
            LEFT JOIN media_items
                ON media_items.id = automation_artifacts.item_id
            LEFT JOIN media_sources
                ON media_sources.id = automation_artifacts.source_id
            ORDER BY automation_artifacts.created_at DESC, automation_artifacts.id DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_generated_artifact_proposal)
            .collect()
    }
}

impl SqliteStore {
    pub(crate) async fn get_automation_artifact(
        &self,
        id: AutomationArtifactId,
    ) -> Result<Option<AutomationArtifactRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status,
                created_at,
                updated_at,
                accepted_at
            FROM automation_artifacts
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_automation_artifact).transpose()
    }

    pub(crate) async fn get_automation_artifact_or_not_found(
        &self,
        id: AutomationArtifactId,
    ) -> Result<AutomationArtifactRecord> {
        self.get_automation_artifact(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "automation_artifact",
                id: id.to_string(),
            })
    }
}

fn row_to_generated_artifact_proposal(row: SqliteRow) -> Result<GeneratedArtifactProposal> {
    let artifact = AutomationArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        provider_id: parse_id(row_get::<String>(&row, "provider_id")?)?,
        capability: AutomationCapability::parse(&row_get::<String>(&row, "capability")?)?,
        kind: AutomationArtifactKind::parse(&row_get::<String>(&row, "kind")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        item_id: parse_optional_id(row_get::<Option<String>>(&row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        artifact_json: row_get(&row, "artifact_json")?,
        status: AutomationArtifactStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
        accepted_at: row_get(&row, "accepted_at")?,
    };
    Ok(generated_artifact_proposal(
        GeneratedArtifactProposalFacts {
            artifact,
            provider_exists: row_get::<Option<String>>(&row, "provider_exists_id")?.is_some(),
            provider_name: row_get::<Option<String>>(&row, "provider_name")?,
            job_exists: row_get::<Option<String>>(&row, "job_exists_id")?.is_some(),
            job_input_json: row_get::<Option<String>>(&row, "job_input_json")?,
            job_summary_json: row_get::<Option<String>>(&row, "job_summary_json")?,
            library_exists: row_get::<Option<String>>(&row, "library_exists_id")?.is_some(),
            item_exists: row_get::<Option<String>>(&row, "item_exists_id")?.is_some(),
            source_exists: row_get::<Option<String>>(&row, "source_exists_id")?.is_some(),
            source_library_id: parse_optional_id(row_get::<Option<String>>(
                &row,
                "source_library_id",
            )?)?,
            source_item_id: parse_optional_id(row_get::<Option<String>>(&row, "source_item_id")?)?,
        },
    ))
}
