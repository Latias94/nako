use sqlx::sqlite::SqliteRow;

use super::{
    SqliteStore,
    codec::*,
    jobs::insert_job_tx,
    provider_mapping::{upsert_provider_mapping_tx, upsert_provider_subject_tx},
};
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

    async fn find_generated_artifact_metadata_apply_outcome(
        &self,
        artifact_id: AutomationArtifactId,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataApplyOutcomeRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                artifact_id,
                idempotency_key,
                status,
                applied,
                changed,
                applied_source,
                item_id,
                plan_json,
                error_code,
                error_message,
                created_at,
                updated_at
            FROM generated_artifact_metadata_apply_outcomes
            WHERE artifact_id = ?1 AND idempotency_key = ?2
            "#,
        )
        .bind(artifact_id.to_string())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_generated_artifact_metadata_apply_outcome)
            .transpose()
    }

    async fn commit_generated_artifact_metadata_apply_outcome(
        &self,
        commit: &GeneratedArtifactMetadataApplyOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataApplyOutcomeRecord> {
        let plan_json = serde_json::to_string(&commit.plan).map_err(database_error)?;
        validate_generated_artifact_provider_mapping_apply_commits(commit)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        if let Some(application) = &commit.metadata_application {
            if application.catalog_projection.search.item_id != application.item.id {
                return Err(NakoError::InvalidInput {
                    message: format!(
                        "generated artifact metadata apply search projection item_id {} does not match item {}",
                        application.catalog_projection.search.item_id, application.item.id
                    ),
                });
            }
            crate::sqlite::media::upsert_media_item_in_transaction(
                &mut transaction,
                &application.item,
            )
            .await?;
            crate::sqlite::catalog::replace_item_catalog_graph_tx(
                &mut transaction,
                application.item.id,
                &application.catalog_projection.graph,
            )
            .await?;
            crate::sqlite::catalog::upsert_search_projection_tx(
                &mut transaction,
                &application.catalog_projection.search,
            )
            .await?;
        }
        for provider_mapping in &commit.provider_mappings {
            upsert_provider_subject_tx(&mut transaction, &provider_mapping.subject).await?;
            upsert_provider_mapping_tx(&mut transaction, &provider_mapping.mapping).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO generated_artifact_metadata_apply_outcomes (
                id,
                artifact_id,
                idempotency_key,
                status,
                applied,
                changed,
                applied_source,
                item_id,
                plan_json,
                error_code,
                error_message
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(commit.id.to_string())
        .bind(commit.artifact_id.to_string())
        .bind(&commit.idempotency_key)
        .bind(commit.status.as_str())
        .bind(bool_to_i64(commit.applied))
        .bind(bool_to_i64(commit.changed))
        .bind(&commit.applied_source)
        .bind(commit.item_id.map(|id| id.to_string()))
        .bind(plan_json)
        .bind(&commit.error_code)
        .bind(&commit.error_message)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        self.find_generated_artifact_metadata_apply_outcome(
            commit.artifact_id,
            &commit.idempotency_key,
        )
        .await?
        .ok_or_else(|| NakoError::Database {
            message: format!(
                "generated artifact metadata apply outcome {} was not found after commit",
                commit.id
            ),
        })
    }

    async fn get_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        self.load_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await
    }

    async fn find_generated_artifact_metadata_bulk_apply_batch(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM generated_artifact_metadata_bulk_apply_batches
            WHERE idempotency_key = ?1
            "#,
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        self.load_generated_artifact_metadata_bulk_apply_batch(parse_id(row_get::<String>(
            &row, "id",
        )?)?)
        .await
    }

    async fn commit_generated_artifact_metadata_bulk_apply_batch(
        &self,
        commit: &GeneratedArtifactMetadataBulkApplyBatchCommit,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        validate_generated_artifact_metadata_bulk_apply_batch_commit(commit)?;
        if let Some(existing) = self
            .find_generated_artifact_metadata_bulk_apply_batch(&commit.idempotency_key)
            .await?
        {
            return Ok(existing);
        }

        let selection_json = serde_json::to_string(&commit.selection).map_err(database_error)?;
        let summary_json = serde_json::to_string(&commit.summary).map_err(database_error)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        insert_job_tx(&mut transaction, commit.job.clone()).await?;

        sqlx::query(
            r#"
            INSERT INTO generated_artifact_metadata_bulk_apply_batches (
                id,
                job_id,
                idempotency_key,
                status,
                selection_json,
                summary_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(commit.id.to_string())
        .bind(commit.job.id.to_string())
        .bind(&commit.idempotency_key)
        .bind(commit.status.as_str())
        .bind(selection_json)
        .bind(summary_json)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        for item in &commit.items {
            let plan_item_json = serde_json::to_string(&item.plan_item).map_err(database_error)?;
            sqlx::query(
                r#"
                INSERT INTO generated_artifact_metadata_bulk_apply_batch_items (
                    batch_id,
                    position,
                    artifact_id,
                    status,
                    idempotency_key,
                    outcome_id,
                    error_code,
                    error_message,
                    plan_item_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, NULL, ?6)
                "#,
            )
            .bind(commit.id.to_string())
            .bind(u32_to_i64(item.position))
            .bind(item.artifact_id.to_string())
            .bind(item.status.as_str())
            .bind(&item.idempotency_key)
            .bind(plan_item_json)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        self.load_generated_artifact_metadata_bulk_apply_batch(commit.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "generated artifact metadata bulk apply batch {} was not found after commit",
                    commit.id
                ),
            })
    }

    async fn commit_generated_artifact_metadata_bulk_apply_batch_item_outcome(
        &self,
        commit: &GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        validate_generated_artifact_metadata_bulk_apply_batch_item_outcome_commit(commit)?;
        let result = sqlx::query(
            r#"
            UPDATE generated_artifact_metadata_bulk_apply_batch_items
            SET status = ?1,
                outcome_id = ?2,
                error_code = ?3,
                error_message = ?4,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE batch_id = ?5 AND artifact_id = ?6
            "#,
        )
        .bind(commit.status.as_str())
        .bind(commit.outcome_id.map(|id| id.to_string()))
        .bind(&commit.error_code)
        .bind(&commit.error_message)
        .bind(commit.batch_id.to_string())
        .bind(commit.artifact_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(NakoError::NotFound {
                entity: "generated_artifact_metadata_bulk_apply_batch_item",
                id: format!("{}:{}", commit.batch_id, commit.artifact_id),
            });
        }

        self.load_generated_artifact_metadata_bulk_apply_batch(commit.batch_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "generated artifact metadata bulk apply batch {} was not found after item outcome commit",
                    commit.batch_id
                ),
            })
    }

    async fn update_generated_artifact_metadata_bulk_apply_batch_status(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
        expected: GeneratedArtifactMetadataBulkApplyBatchStatus,
        status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        let result = sqlx::query(
            r#"
            UPDATE generated_artifact_metadata_bulk_apply_batches
            SET status = ?1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?2 AND status = ?3
            "#,
        )
        .bind(status.as_str())
        .bind(batch_id.to_string())
        .bind(expected.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            if self
                .load_generated_artifact_metadata_bulk_apply_batch(batch_id)
                .await?
                .is_none()
            {
                return Err(NakoError::NotFound {
                    entity: "generated_artifact_metadata_bulk_apply_batch",
                    id: batch_id.to_string(),
                });
            }
            return Err(NakoError::InvalidInput {
                message: format!(
                    "cannot transition generated artifact metadata bulk apply batch {batch_id} from {:?} to {:?}",
                    expected, status
                ),
            });
        }

        self.load_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "generated artifact metadata bulk apply batch {batch_id} was not found after status update"
                ),
            })
    }
}

impl SqliteStore {
    async fn load_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                idempotency_key,
                status,
                selection_json,
                summary_json,
                created_at,
                updated_at
            FROM generated_artifact_metadata_bulk_apply_batches
            WHERE id = ?1
            "#,
        )
        .bind(batch_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let item_rows = sqlx::query(
            r#"
            SELECT
                batch_id,
                position,
                artifact_id,
                status,
                idempotency_key,
                outcome_id,
                error_code,
                error_message,
                plan_item_json,
                created_at,
                updated_at
            FROM generated_artifact_metadata_bulk_apply_batch_items
            WHERE batch_id = ?1
            ORDER BY position ASC
            "#,
        )
        .bind(batch_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        let items = item_rows
            .into_iter()
            .map(row_to_generated_artifact_metadata_bulk_apply_batch_item)
            .collect::<Result<Vec<_>>>()?;

        row_to_generated_artifact_metadata_bulk_apply_batch(row, items).map(Some)
    }

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

fn row_to_generated_artifact_metadata_apply_outcome(
    row: SqliteRow,
) -> Result<GeneratedArtifactMetadataApplyOutcomeRecord> {
    let plan_json: String = row_get(&row, "plan_json")?;
    let plan = serde_json::from_str(&plan_json).map_err(database_error)?;

    Ok(GeneratedArtifactMetadataApplyOutcomeRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        artifact_id: parse_id(row_get::<String>(&row, "artifact_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        status: GeneratedArtifactMetadataApplyOutcomeStatus::parse(&row_get::<String>(
            &row, "status",
        )?)?,
        applied: i64_to_bool(row_get(&row, "applied")?)?,
        changed: i64_to_bool(row_get(&row, "changed")?)?,
        applied_source: row_get(&row, "applied_source")?,
        item_id: parse_optional_id(row_get::<Option<String>>(&row, "item_id")?)?,
        plan,
        error_code: row_get(&row, "error_code")?,
        error_message: row_get(&row, "error_message")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn validate_generated_artifact_metadata_bulk_apply_batch_commit(
    commit: &GeneratedArtifactMetadataBulkApplyBatchCommit,
) -> Result<()> {
    if commit.job.kind != JobKind::GeneratedArtifactMetadataBulkApply {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch job must use JobKind::GeneratedArtifactMetadataBulkApply"
                .to_owned(),
        });
    }
    if commit.job.resource_class != GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch job resource_class is invalid"
                .to_owned(),
        });
    }
    if commit.idempotency_key.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch idempotency_key cannot be empty"
                .to_owned(),
        });
    }
    if commit.items.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch requires at least one item"
                .to_owned(),
        });
    }
    for item in &commit.items {
        if item.idempotency_key.trim().is_empty() {
            return Err(NakoError::InvalidInput {
                message:
                    "generated artifact metadata bulk apply item idempotency_key cannot be empty"
                        .to_owned(),
            });
        }
        if item.artifact_id != item.plan_item.artifact_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact metadata bulk apply item artifact_id {} does not match plan item {}",
                    item.artifact_id, item.plan_item.artifact_id
                ),
            });
        }
    }

    Ok(())
}

fn validate_generated_artifact_provider_mapping_apply_commits(
    commit: &GeneratedArtifactMetadataApplyOutcomeCommit,
) -> Result<()> {
    for provider_mapping in &commit.provider_mappings {
        if provider_mapping.mapping.subject_id != provider_mapping.subject.id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact provider mapping subject_id {} does not match subject {}",
                    provider_mapping.mapping.subject_id, provider_mapping.subject.id
                ),
            });
        }
        if provider_mapping.mapping.status != ProviderMappingStatus::Accepted {
            return Err(NakoError::InvalidInput {
                message:
                    "generated artifact provider mapping apply commits must use accepted status"
                        .to_owned(),
            });
        }
        if Some(provider_mapping.mapping.item_id) != commit.item_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact provider mapping item_id {} does not match outcome item {:?}",
                    provider_mapping.mapping.item_id, commit.item_id
                ),
            });
        }
    }

    Ok(())
}

fn validate_generated_artifact_metadata_bulk_apply_batch_item_outcome_commit(
    commit: &GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
) -> Result<()> {
    if matches!(
        commit.status,
        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending
            | GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped
    ) {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply item outcome must be terminal"
                .to_owned(),
        });
    }
    if commit.error_code.as_deref().is_some_and(str::is_empty)
        || commit.error_message.as_deref().is_some_and(str::is_empty)
    {
        return Err(NakoError::InvalidInput {
            message:
                "generated artifact metadata bulk apply item outcome error fields cannot be empty"
                    .to_owned(),
        });
    }

    Ok(())
}

fn row_to_generated_artifact_metadata_bulk_apply_batch(
    row: SqliteRow,
    items: Vec<GeneratedArtifactMetadataBulkApplyBatchItemRecord>,
) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
    let selection_json: String = row_get(&row, "selection_json")?;
    let summary_json: String = row_get(&row, "summary_json")?;

    Ok(GeneratedArtifactMetadataBulkApplyBatchRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        status: GeneratedArtifactMetadataBulkApplyBatchStatus::parse(&row_get::<String>(
            &row, "status",
        )?)?,
        selection: serde_json::from_str(&selection_json).map_err(database_error)?,
        summary: serde_json::from_str(&summary_json).map_err(database_error)?,
        execution_summary: GeneratedArtifactMetadataBulkApplyBatchExecutionSummary::from_items(
            &items,
        ),
        items,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_generated_artifact_metadata_bulk_apply_batch_item(
    row: SqliteRow,
) -> Result<GeneratedArtifactMetadataBulkApplyBatchItemRecord> {
    let plan_item_json: String = row_get(&row, "plan_item_json")?;

    Ok(GeneratedArtifactMetadataBulkApplyBatchItemRecord {
        batch_id: parse_id(row_get::<String>(&row, "batch_id")?)?,
        artifact_id: parse_id(row_get::<String>(&row, "artifact_id")?)?,
        position: i64_to_u32(row_get::<i64>(&row, "position")?)?,
        status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::parse(&row_get::<String>(
            &row, "status",
        )?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        outcome_id: parse_optional_id(row_get::<Option<String>>(&row, "outcome_id")?)?,
        error_code: row_get(&row, "error_code")?,
        error_message: row_get(&row, "error_message")?,
        plan_item: serde_json::from_str(&plan_item_json).map_err(database_error)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}
