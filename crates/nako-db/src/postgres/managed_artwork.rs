use sqlx::{PgPool, Postgres, postgres::PgRow};

use nako_core::*;

use super::jobs::{get_job_tx, insert_job_tx};
use super::{
    PostgresStore, database_error, i64_to_u32, image_kind_from_parts, image_kind_to_parts,
    optional_i64_to_u32, optional_i64_to_u64, optional_u64_to_i64, parse_id, parse_optional_id,
    row_get, u32_to_i64, u64_to_i64,
};

const ARTWORK_CANDIDATE_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                side_effect_id::text AS side_effect_id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                source_kind,
                source_uri,
                width,
                height,
                language,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_artwork_candidates
            "#;

const ARTWORK_TASK_SELECT: &str = r#"
            SELECT
                id::text AS id,
                image_id::text AS image_id,
                kind,
                status,
                resource_class,
                attempts,
                max_attempts,
                error
            FROM artwork_tasks
            "#;

const MANAGED_ARTWORK_INGEST_SELECT: &str = r#"
            SELECT
                id::text AS id,
                candidate_id::text AS candidate_id,
                job_id::text AS job_id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                status,
                artifact_id::text AS artifact_id,
                failure_code,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM managed_artwork_ingests
            "#;

const MANAGED_ARTWORK_ARTIFACT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                ingest_id::text AS ingest_id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                storage_uri,
                content_hash,
                width,
                height,
                byte_len,
                media_type,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM managed_artwork_artifacts
            WHERE deleted_at IS NULL
            "#;

const SELECTED_ARTWORK_SELECT: &str = r#"
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                artifact_id::text AS artifact_id,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM selected_artworks
            "#;

#[async_trait::async_trait]
impl ArtworkTaskRepository for PostgresStore {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO artwork_tasks (
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                image_id = excluded.image_id,
                kind = excluded.kind,
                status = excluded.status,
                resource_class = excluded.resource_class,
                attempts = excluded.attempts,
                max_attempts = excluded.max_attempts,
                error = excluded.error,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(task.id.as_uuid())
        .bind(task.image_id.as_uuid())
        .bind(task.kind.as_str())
        .bind(task.status.as_str())
        .bind(&task.resource_class)
        .bind(u32_to_i64(task.attempts))
        .bind(u32_to_i64(task.max_attempts))
        .bind(&task.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_artwork_task(&self, id: ArtworkTaskId) -> Result<Option<ArtworkTask>> {
        let row = sqlx::query(&format!(
            r#"
            {ARTWORK_TASK_SELECT}
            WHERE id = $1
            "#
        ))
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_artwork_task).transpose()
    }

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {ARTWORK_TASK_SELECT}
            ORDER BY id ASC
            LIMIT $1 OFFSET $2
            "#
        ))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_task).collect()
    }
}

#[async_trait::async_trait]
impl ArtworkCandidateRepository for PostgresStore {
    async fn create_artwork_candidate(
        &self,
        candidate: NewArtworkCandidate,
    ) -> Result<ArtworkCandidateRecord> {
        let (kind, kind_key) = image_kind_to_parts(&candidate.kind);
        sqlx::query(
            r#"
            INSERT INTO addon_artwork_candidates (
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(candidate.id.as_uuid())
        .bind(candidate.addon_id.as_uuid())
        .bind(candidate.side_effect_id.as_uuid())
        .bind(candidate.library_id.as_uuid())
        .bind(candidate.item_id.as_uuid())
        .bind(&kind)
        .bind(&kind_key)
        .bind(candidate.source_kind.as_str())
        .bind(&candidate.source_uri)
        .bind(candidate.width.map(u32_to_i64))
        .bind(candidate.height.map(u32_to_i64))
        .bind(&candidate.language)
        .bind(ArtworkCandidateStatus::Proposed.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_artwork_candidate_by_source(
            candidate.addon_id,
            candidate.library_id,
            candidate.item_id,
            &candidate.kind,
            candidate.source_kind,
            &candidate.source_uri,
        )
        .await?
        .ok_or_else(|| NakoError::Database {
            message: "failed to load created addon artwork candidate".to_owned(),
        })
    }

    async fn get_artwork_candidate(
        &self,
        id: ArtworkCandidateId,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        get_artwork_candidate(&self.pool, id).await
    }

    async fn set_artwork_candidate_status(
        &self,
        id: ArtworkCandidateId,
        status: ArtworkCandidateStatus,
    ) -> Result<ArtworkCandidateRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let candidate = update_artwork_candidate_status_tx(&mut transaction, id, status).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(candidate)
    }

    async fn find_artwork_candidate_by_source(
        &self,
        addon_id: AddonId,
        library_id: LibraryId,
        item_id: MediaItemId,
        kind: &ImageKind,
        source_kind: ArtworkCandidateSourceKind,
        source_uri: &str,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        let (kind, kind_key) = image_kind_to_parts(kind);
        let row = sqlx::query(&format!(
            r#"
            {ARTWORK_CANDIDATE_SELECT}
            WHERE addon_id = $1 AND library_id = $2 AND item_id = $3
                AND kind = $4 AND kind_key = $5 AND source_kind = $6
                AND source_uri = $7
            LIMIT 1
            "#
        ))
        .bind(addon_id.as_uuid())
        .bind(library_id.as_uuid())
        .bind(item_id.as_uuid())
        .bind(kind)
        .bind(kind_key)
        .bind(source_kind.as_str())
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_artwork_candidate).transpose()
    }

    async fn list_artwork_candidates_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ArtworkCandidateRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {ARTWORK_CANDIDATE_SELECT}
            WHERE item_id = $1
            ORDER BY created_at DESC, id ASC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_candidate).collect()
    }
}

#[async_trait::async_trait]
impl ManagedArtworkRepository for PostgresStore {
    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        if let Some(existing) =
            get_managed_artwork_ingest_by_candidate_tx(&mut transaction, candidate_id).await?
        {
            let candidate = update_artwork_candidate_status_tx(
                &mut transaction,
                candidate_id,
                ArtworkCandidateStatus::Accepted,
            )
            .await?;
            let job = get_job_tx(&mut transaction, existing.job_id).await?;
            transaction.commit().await.map_err(database_error)?;
            return Ok(ManagedArtworkAcceptanceRecord {
                candidate,
                ingest: existing,
                job,
            });
        }

        insert_job_tx(&mut transaction, job).await?;
        let saved_ingest = insert_managed_artwork_ingest_tx(&mut transaction, ingest).await?;
        let candidate = update_artwork_candidate_status_tx(
            &mut transaction,
            candidate_id,
            ArtworkCandidateStatus::Accepted,
        )
        .await?;
        let job = get_job_tx(&mut transaction, saved_ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkAcceptanceRecord {
            candidate,
            ingest: saved_ingest,
            job,
        })
    }

    async fn get_managed_artwork_ingest(
        &self,
        id: ManagedArtworkIngestId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        get_managed_artwork_ingest(&self.pool, id).await
    }

    async fn find_managed_artwork_ingest_by_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {MANAGED_ARTWORK_INGEST_SELECT}
            WHERE candidate_id = $1
            "#
        ))
        .bind(candidate_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_managed_artwork_ingest).transpose()
    }

    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let row = sqlx::query(
            r#"
            SELECT i.id::text AS id
            FROM managed_artwork_ingests i
            JOIN jobs j ON j.id = i.job_id
            JOIN addon_artwork_candidates c ON c.id = i.candidate_id
            WHERE i.status = $1
                AND j.status = $2
                AND j.kind = $3
                AND j.resource_class = $4
                AND c.status = $5
            ORDER BY i.created_at ASC, i.id ASC
            LIMIT 1
            FOR UPDATE OF i SKIP LOCKED
            "#,
        )
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .bind(ArtworkCandidateStatus::Accepted.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        let ingest_id: ManagedArtworkIngestId = parse_id(row_get::<String>(&row, "id")?)?;

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                failure_code = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                started_at = COALESCE(started_at, statement_timestamp()),
                completed_at = NULL,
                summary_json = NULL,
                error = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Running.as_str())
        .bind(JobStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let candidate = get_artwork_candidate_tx(&mut transaction, ingest.candidate_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "artwork_candidate",
                id: ingest.candidate_id.to_string(),
            })?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(Some(ManagedArtworkIngestClaimRecord {
            candidate,
            ingest,
            job,
        }))
    }

    async fn commit_managed_artwork_artifact(
        &self,
        ingest_id: ManagedArtworkIngestId,
        artifact: NewManagedArtworkArtifact,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        if artifact.ingest_id != ingest_id {
            return Err(NakoError::InvalidInput {
                message: "managed artwork artifact ingest_id must match committed ingest"
                    .to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let artifact_id = artifact.id;
        insert_managed_artwork_artifact_tx(&mut transaction, artifact).await?;

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                artifact_id = $3,
                failure_code = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status IN ($4, $5)
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Stored.as_str())
        .bind(artifact_id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest is not claimable for artifact commit".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                summary_json = $3,
                error = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $4
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Succeeded.as_str())
        .bind(job_summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest job is not running for artifact commit".to_owned(),
            });
        }

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact = get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: "stored managed artwork ingest is missing artifact metadata".to_owned(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkIngestProcessingRecord {
            ingest,
            artifact: Some(artifact),
            job,
        })
    }

    async fn fail_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                failure_code = $3,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status IN ($4, $5)
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .bind(&failure_code)
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest is not claimable for failure commit".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                error = $3,
                summary_json = $4,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $5
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Failed.as_str())
        .bind(job_error)
        .bind(job_summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest job is not running for failure commit".to_owned(),
            });
        }

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact =
            get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id).await?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkIngestProcessingRecord {
            ingest,
            artifact,
            job,
        })
    }

    async fn fail_unfinished_managed_artwork_ingests(
        &self,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<u64> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let result = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $1,
                failure_code = $2,
                updated_at = statement_timestamp()
            WHERE status IN ($3, $4)
                AND artifact_id IS NULL
                AND EXISTS (
                    SELECT 1
                    FROM jobs j
                    WHERE j.id = managed_artwork_ingests.job_id
                        AND j.kind = $5
                        AND j.resource_class = $6
                        AND j.status = $7
                )
            "#,
        )
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .bind(&failure_code)
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        let recovered = result.rows_affected();

        if recovered > 0 {
            sqlx::query(
                r#"
                UPDATE jobs
                SET status = $1,
                    error = $2,
                    summary_json = $3,
                    completed_at = statement_timestamp(),
                    updated_at = statement_timestamp()
                WHERE kind = $4
                    AND resource_class = $5
                    AND status = $6
                    AND EXISTS (
                        SELECT 1
                        FROM managed_artwork_ingests i
                        WHERE i.job_id = jobs.id
                            AND i.status = $7
                            AND i.failure_code = $8
                    )
                "#,
            )
            .bind(JobStatus::Failed.as_str())
            .bind(job_error)
            .bind(job_summary_json)
            .bind(JobKind::ManagedArtworkIngest.as_str())
            .bind("artwork.ingest")
            .bind(JobStatus::Running.as_str())
            .bind(ManagedArtworkIngestStatus::Failed.as_str())
            .bind(&failure_code)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(recovered)
    }

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;

        if job.kind != JobKind::ManagedArtworkIngest || job.resource_class != "artwork.ingest" {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest job is not an artwork ingest job".to_owned(),
            });
        }

        if ingest.status == ManagedArtworkIngestStatus::Queued {
            if job.status != JobStatus::Queued {
                transaction.rollback().await.map_err(database_error)?;
                return Err(NakoError::Conflict {
                    message: "queued managed artwork ingest job is not queued".to_owned(),
                });
            }
            transaction.commit().await.map_err(database_error)?;
            return Ok(ManagedArtworkIngestRequeueRecord {
                ingest,
                job,
                requeued: false,
                had_failure: false,
            });
        }

        if ingest.status != ManagedArtworkIngestStatus::Failed || ingest.artifact_id.is_some() {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest is not failed or queued for requeue".to_owned(),
            });
        }

        if job.status != JobStatus::Failed {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest job is not failed for requeue".to_owned(),
            });
        }

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                failure_code = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $3 AND artifact_id IS NULL
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest is not failed or queued for requeue".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                summary_json = NULL,
                error = NULL,
                started_at = NULL,
                completed_at = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND status = $3
                AND kind = $4
                AND resource_class = $5
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Queued.as_str())
        .bind(JobStatus::Failed.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
                message: "managed artwork ingest job is not failed for requeue".to_owned(),
            });
        }

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkIngestRequeueRecord {
            ingest,
            job,
            requeued: true,
            had_failure: true,
        })
    }

    async fn get_managed_artwork_artifact(
        &self,
        id: ManagedArtworkArtifactId,
    ) -> Result<Option<ManagedArtworkArtifactRecord>> {
        get_managed_artwork_artifact(&self.pool, id).await
    }

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        publish_selected_artwork_tx(&self.pool, artifact_id, None).await
    }

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        publish_selected_artwork_tx(&self.pool, artifact_id, Some((item_id, kind))).await
    }

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let (kind_part, kind_key) = image_kind_to_parts(&kind);
        let unpublished =
            get_selected_artwork_by_slot_tx(&mut transaction, item_id, &kind_part, &kind_key)
                .await?;
        let artifact = if let Some(selected) = unpublished.as_ref() {
            Some(
                get_managed_artwork_artifact_tx(&mut transaction, selected.artifact_id)
                    .await?
                    .ok_or_else(|| NakoError::Database {
                        message: "selected artwork is linked to a missing managed artwork artifact"
                            .to_owned(),
                    })?,
            )
        } else {
            None
        };

        if let Some(selected) = unpublished.as_ref() {
            sqlx::query(
                r#"
                DELETE FROM selected_artworks
                WHERE id = $1
                "#,
            )
            .bind(selected.id.as_uuid())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(SelectedArtworkUnpublicationRecord {
            item_id,
            kind,
            changed: unpublished.is_some(),
            unpublished,
            artifact,
        })
    }

    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>> {
        get_selected_artwork(&self.pool, id).await
    }

    async fn list_selected_artwork_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<SelectedArtworkRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {SELECTED_ARTWORK_SELECT}
            WHERE item_id = $1
            ORDER BY kind ASC, id ASC
            "#
        ))
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_selected_artwork).collect()
    }

    async fn get_managed_artwork_gallery_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<ManagedArtworkGallerySnapshot> {
        let page = page.clamped();
        let candidates = managed_artwork_gallery_candidates(&self.pool, item_id, page).await?;
        let artifacts = managed_artwork_gallery_artifacts(&self.pool, item_id, page).await?;
        let selected = managed_artwork_gallery_selected(&self.pool, item_id).await?;
        let summary = ManagedArtworkGallerySummary {
            candidates: u32::try_from(candidates.len()).unwrap_or(u32::MAX),
            artifacts: u32::try_from(artifacts.len()).unwrap_or(u32::MAX),
            selected: u32::try_from(selected.len()).unwrap_or(u32::MAX),
        };

        Ok(ManagedArtworkGallerySnapshot {
            item_id,
            summary,
            candidates,
            artifacts,
            selected,
        })
    }

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot> {
        let summary = managed_artwork_artifact_lifecycle_summary(&self.pool).await?;
        let artifacts = managed_artwork_artifact_lifecycle_rows(&self.pool, filter, page).await?;

        Ok(ManagedArtworkArtifactLifecycleSnapshot { summary, artifacts })
    }

    async fn cleanup_unselected_managed_artwork_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactCleanupReport> {
        let page = page.clamped();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let candidates = managed_artwork_artifact_lifecycle_rows_tx(
            &mut transaction,
            ManagedArtworkArtifactLifecycleFilter::CleanupCandidates,
            page,
        )
        .await?;
        let examined_artifacts = u32::try_from(candidates.len()).unwrap_or(u32::MAX);
        let mut cleaned_artifacts = Vec::new();

        for candidate in candidates {
            let artifact = candidate.artifact;
            let result = sqlx::query(
                r#"
                UPDATE managed_artwork_artifacts
                SET deleted_at = statement_timestamp(),
                    updated_at = statement_timestamp()
                WHERE id = $1
                    AND deleted_at IS NULL
                    AND NOT EXISTS (
                        SELECT 1
                        FROM selected_artworks s
                        WHERE s.artifact_id = managed_artwork_artifacts.id
                    )
                "#,
            )
            .bind(artifact.id.as_uuid())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

            if result.rows_affected() == 1 {
                cleaned_artifacts.push(artifact);
            }
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkArtifactCleanupReport {
            examined_artifacts,
            cleanup_candidate_artifacts: examined_artifacts,
            cleaned_artifacts,
        })
    }
}

async fn get_artwork_candidate(
    pool: &PgPool,
    id: ArtworkCandidateId,
) -> Result<Option<ArtworkCandidateRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {ARTWORK_CANDIDATE_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_artwork_candidate).transpose()
}

async fn get_artwork_candidate_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ArtworkCandidateId,
) -> Result<Option<ArtworkCandidateRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {ARTWORK_CANDIDATE_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_artwork_candidate).transpose()
}

async fn update_artwork_candidate_status_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ArtworkCandidateId,
    status: ArtworkCandidateStatus,
) -> Result<ArtworkCandidateRecord> {
    sqlx::query(
        r#"
        UPDATE addon_artwork_candidates
        SET status = $2,
            updated_at = statement_timestamp()
        WHERE id = $1
        "#,
    )
    .bind(id.as_uuid())
    .bind(status.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    get_artwork_candidate_tx(transaction, id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "artwork_candidate",
            id: id.to_string(),
        })
}

async fn insert_managed_artwork_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    ingest: NewManagedArtworkIngest,
) -> Result<ManagedArtworkIngestRecord> {
    let (kind, kind_key) = image_kind_to_parts(&ingest.kind);
    sqlx::query(
        r#"
        INSERT INTO managed_artwork_ingests (
            id, candidate_id, job_id, library_id, item_id, kind, kind_key,
            status, artifact_id, failure_code
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(ingest.id.as_uuid())
    .bind(ingest.candidate_id.as_uuid())
    .bind(ingest.job_id.as_uuid())
    .bind(ingest.library_id.as_uuid())
    .bind(ingest.item_id.as_uuid())
    .bind(kind)
    .bind(kind_key)
    .bind(ingest.status.as_str())
    .bind(ingest.artifact_id.map(|id| id.as_uuid()))
    .bind(ingest.failure_code)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    get_managed_artwork_ingest_tx(transaction, ingest.id)
        .await?
        .ok_or_else(|| NakoError::Database {
            message: "failed to load created managed artwork ingest".to_owned(),
        })
}

async fn get_managed_artwork_ingest(
    pool: &PgPool,
    id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_INGEST_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn get_managed_artwork_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_INGEST_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn get_managed_artwork_ingest_by_candidate_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    candidate_id: ArtworkCandidateId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_INGEST_SELECT}
        WHERE candidate_id = $1
        "#
    ))
    .bind(candidate_id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn insert_managed_artwork_artifact_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    artifact: NewManagedArtworkArtifact,
) -> Result<()> {
    let (kind, kind_key) = image_kind_to_parts(&artifact.kind);
    sqlx::query(
        r#"
        INSERT INTO managed_artwork_artifacts (
            id, ingest_id, library_id, item_id, kind, kind_key, storage_uri,
            content_hash, width, height, byte_len, media_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(artifact.id.as_uuid())
    .bind(artifact.ingest_id.as_uuid())
    .bind(artifact.library_id.as_uuid())
    .bind(artifact.item_id.as_uuid())
    .bind(kind)
    .bind(kind_key)
    .bind(artifact.storage_uri)
    .bind(artifact.content_hash)
    .bind(artifact.width.map(u32_to_i64))
    .bind(artifact.height.map(u32_to_i64))
    .bind(optional_u64_to_i64(artifact.byte_len)?)
    .bind(artifact.media_type)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn get_managed_artwork_artifact(
    pool: &PgPool,
    id: ManagedArtworkArtifactId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_ARTIFACT_SELECT}
            AND id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn get_managed_artwork_artifact_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ManagedArtworkArtifactId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_ARTIFACT_SELECT}
            AND id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn get_managed_artwork_artifact_by_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    ingest_id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_ARTIFACT_SELECT}
            AND ingest_id = $1
        "#
    ))
    .bind(ingest_id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn publish_selected_artwork_tx(
    pool: &PgPool,
    artifact_id: ManagedArtworkArtifactId,
    expected_slot: Option<(MediaItemId, ImageKind)>,
) -> Result<SelectedArtworkPublicationRecord> {
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let artifact = get_managed_artwork_artifact_tx(&mut transaction, artifact_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "managed_artwork_artifact",
            id: artifact_id.to_string(),
        })?;

    if let Some((expected_item_id, expected_kind)) = expected_slot.as_ref() {
        if artifact.item_id != *expected_item_id || artifact.kind != *expected_kind {
            return Err(NakoError::Conflict {
                message: "managed artwork artifact does not match the requested item artwork slot"
                    .to_owned(),
            });
        }
    }

    get_managed_artwork_ingest_tx(&mut transaction, artifact.ingest_id)
        .await?
        .filter(|ingest| ingest.artifact_id == Some(artifact.id))
        .filter(|ingest| ingest.status == ManagedArtworkIngestStatus::Stored)
        .ok_or_else(|| NakoError::Conflict {
            message: "managed artwork artifact is not linked to a stored ingest".to_owned(),
        })?;

    let (kind, kind_key) = image_kind_to_parts(&artifact.kind);
    let existing =
        get_selected_artwork_by_slot_tx(&mut transaction, artifact.item_id, &kind, &kind_key)
            .await?;
    let selected_id = existing
        .as_ref()
        .map_or_else(SelectedArtworkId::new, |selected| selected.id);
    let changed = existing
        .as_ref()
        .is_none_or(|selected| selected.artifact_id != artifact.id);

    if let Some(existing) = existing {
        sqlx::query(
            r#"
            UPDATE selected_artworks
            SET library_id = $2,
                artifact_id = $3,
                updated_at = CASE
                    WHEN artifact_id = $3 THEN updated_at
                    ELSE statement_timestamp()
                END
            WHERE id = $1
            "#,
        )
        .bind(existing.id.as_uuid())
        .bind(artifact.library_id.as_uuid())
        .bind(artifact.id.as_uuid())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO selected_artworks (
                id, library_id, item_id, kind, kind_key, artifact_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(selected_id.as_uuid())
        .bind(artifact.library_id.as_uuid())
        .bind(artifact.item_id.as_uuid())
        .bind(kind)
        .bind(kind_key)
        .bind(artifact.id.as_uuid())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }

    let selected_artwork = get_selected_artwork_tx(&mut transaction, selected_id)
        .await?
        .ok_or_else(|| NakoError::Database {
            message: "failed to load selected artwork publication".to_owned(),
        })?;
    transaction.commit().await.map_err(database_error)?;

    Ok(SelectedArtworkPublicationRecord {
        selected_artwork,
        artifact,
        changed,
    })
}

async fn get_selected_artwork(
    pool: &PgPool,
    id: SelectedArtworkId,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {SELECTED_ARTWORK_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn get_selected_artwork_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: SelectedArtworkId,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {SELECTED_ARTWORK_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn get_selected_artwork_by_slot_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_id: MediaItemId,
    kind: &str,
    kind_key: &str,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {SELECTED_ARTWORK_SELECT}
        WHERE item_id = $1 AND kind = $2 AND kind_key = $3
        "#
    ))
    .bind(item_id.as_uuid())
    .bind(kind)
    .bind(kind_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn managed_artwork_gallery_candidates(
    pool: &PgPool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkGalleryCandidateRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(
        r#"
        SELECT
            c.id::text AS id,
            c.addon_id::text AS addon_id,
            c.side_effect_id::text AS side_effect_id,
            c.library_id::text AS library_id,
            c.item_id::text AS item_id,
            c.kind,
            c.kind_key,
            c.source_kind,
            c.source_uri,
            c.width,
            c.height,
            c.language,
            c.status,
            to_char(c.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(c.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
            i.id::text AS ingest_id,
            i.job_id::text AS ingest_job_id,
            i.status AS ingest_status,
            i.artifact_id::text AS ingest_artifact_id,
            i.failure_code AS ingest_failure_code,
            to_char(i.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS ingest_created_at,
            to_char(i.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS ingest_updated_at,
            COUNT(s.id)::bigint AS selected_artwork_count
        FROM addon_artwork_candidates c
        LEFT JOIN managed_artwork_ingests i ON i.candidate_id = c.id
        LEFT JOIN managed_artwork_artifacts a ON a.ingest_id = i.id
            AND a.deleted_at IS NULL
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE c.item_id = $1
        GROUP BY
            c.id,
            c.addon_id,
            c.side_effect_id,
            c.library_id,
            c.item_id,
            c.kind,
            c.kind_key,
            c.source_kind,
            c.source_uri,
            c.width,
            c.height,
            c.language,
            c.status,
            c.created_at,
            c.updated_at,
            i.id,
            i.job_id,
            i.status,
            i.artifact_id,
            i.failure_code,
            i.created_at,
            i.updated_at
        ORDER BY c.created_at DESC, c.id ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(item_id.as_uuid())
    .bind(u32_to_i64(page.limit))
    .bind(u64_to_i64(page.offset)?)
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_candidate)
        .collect()
}

async fn managed_artwork_gallery_artifacts(
    pool: &PgPool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkGalleryArtifactRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(
        r#"
        SELECT
            a.id::text AS id,
            a.ingest_id::text AS ingest_id,
            i.candidate_id::text AS candidate_id,
            a.library_id::text AS library_id,
            a.item_id::text AS item_id,
            a.kind,
            a.kind_key,
            a.width,
            a.height,
            a.byte_len,
            a.media_type,
            (a.content_hash IS NOT NULL) AS has_content_hash,
            COUNT(s.id)::bigint AS selected_artwork_count,
            to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM managed_artwork_artifacts a
        INNER JOIN managed_artwork_ingests i ON i.id = a.ingest_id
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE a.item_id = $1 AND a.deleted_at IS NULL
        GROUP BY
            a.id,
            a.ingest_id,
            i.candidate_id,
            a.library_id,
            a.item_id,
            a.kind,
            a.kind_key,
            a.width,
            a.height,
            a.byte_len,
            a.media_type,
            a.content_hash,
            a.created_at,
            a.updated_at
        ORDER BY a.created_at DESC, a.id ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(item_id.as_uuid())
    .bind(u32_to_i64(page.limit))
    .bind(u64_to_i64(page.offset)?)
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_artifact)
        .collect()
}

async fn managed_artwork_gallery_selected(
    pool: &PgPool,
    item_id: MediaItemId,
) -> Result<Vec<ManagedArtworkGallerySelectedRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT
            s.id::text AS selected_id,
            s.library_id::text AS selected_library_id,
            s.item_id::text AS selected_item_id,
            s.kind AS selected_kind,
            s.kind_key AS selected_kind_key,
            s.artifact_id::text AS selected_artifact_id,
            to_char(s.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS selected_created_at,
            to_char(s.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS selected_updated_at,
            a.id::text AS artifact_id,
            a.ingest_id::text AS artifact_ingest_id,
            i.candidate_id::text AS artifact_candidate_id,
            a.library_id::text AS artifact_library_id,
            a.item_id::text AS artifact_item_id,
            a.kind AS artifact_kind,
            a.kind_key AS artifact_kind_key,
            a.width AS artifact_width,
            a.height AS artifact_height,
            a.byte_len AS artifact_byte_len,
            a.media_type AS artifact_media_type,
            (a.content_hash IS NOT NULL) AS artifact_has_content_hash,
            COUNT(linked_s.id)::bigint AS artifact_selected_artwork_count,
            to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS artifact_created_at,
            to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS artifact_updated_at
        FROM selected_artworks s
        INNER JOIN managed_artwork_artifacts a ON a.id = s.artifact_id
            AND a.deleted_at IS NULL
        INNER JOIN managed_artwork_ingests i ON i.id = a.ingest_id
        LEFT JOIN selected_artworks linked_s ON linked_s.artifact_id = a.id
        WHERE s.item_id = $1
        GROUP BY
            s.id,
            s.library_id,
            s.item_id,
            s.kind,
            s.kind_key,
            s.artifact_id,
            s.created_at,
            s.updated_at,
            a.id,
            a.ingest_id,
            i.candidate_id,
            a.library_id,
            a.item_id,
            a.kind,
            a.kind_key,
            a.width,
            a.height,
            a.byte_len,
            a.media_type,
            a.content_hash,
            a.created_at,
            a.updated_at
        ORDER BY s.kind ASC, s.id ASC
        "#,
    )
    .bind(item_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_selected)
        .collect()
}

async fn managed_artwork_artifact_lifecycle_summary(
    pool: &PgPool,
) -> Result<ManagedArtworkArtifactLifecycleSummary> {
    let rows = sqlx::query(
        r#"
        SELECT
            a.byte_len,
            COUNT(s.id)::bigint AS selected_artwork_count
        FROM managed_artwork_artifacts a
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE a.deleted_at IS NULL
        GROUP BY a.id, a.byte_len
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    managed_artwork_lifecycle_summary_from_rows(rows)
}

async fn managed_artwork_artifact_lifecycle_rows(
    pool: &PgPool,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(lifecycle_select_sql(filter))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(pool)
        .await
        .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_artifact_lifecycle)
        .collect()
}

async fn managed_artwork_artifact_lifecycle_rows_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(lifecycle_select_sql(filter))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&mut **transaction)
        .await
        .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_artifact_lifecycle)
        .collect()
}

fn managed_artwork_lifecycle_summary_from_rows(
    rows: Vec<PgRow>,
) -> Result<ManagedArtworkArtifactLifecycleSummary> {
    let mut summary = ManagedArtworkArtifactLifecycleSummary::default();
    for row in rows {
        let selected_artwork_count = i64_to_u32(row_get(&row, "selected_artwork_count")?)?;
        let byte_len = optional_i64_to_u64(row_get(&row, "byte_len")?)?;

        summary.total_artifacts = summary.total_artifacts.saturating_add(1);
        if selected_artwork_count == 0 {
            summary.cleanup_candidate_artifacts =
                summary.cleanup_candidate_artifacts.saturating_add(1);
        } else {
            summary.protected_artifacts = summary.protected_artifacts.saturating_add(1);
        }

        match byte_len {
            Some(byte_len) => {
                summary.known_total_bytes = summary.known_total_bytes.saturating_add(byte_len);
                if selected_artwork_count == 0 {
                    summary.known_cleanup_candidate_bytes = summary
                        .known_cleanup_candidate_bytes
                        .saturating_add(byte_len);
                } else {
                    summary.known_protected_bytes =
                        summary.known_protected_bytes.saturating_add(byte_len);
                }
            }
            None => {
                summary.unknown_byte_len_artifacts =
                    summary.unknown_byte_len_artifacts.saturating_add(1);
            }
        }
    }

    Ok(summary)
}

const MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT: &str = r#"
            SELECT
                a.id::text AS id,
                a.ingest_id::text AS ingest_id,
                a.library_id::text AS library_id,
                a.item_id::text AS item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                COUNT(s.id)::bigint AS selected_artwork_count
            FROM managed_artwork_artifacts a
            LEFT JOIN selected_artworks s ON s.artifact_id = a.id
            WHERE a.deleted_at IS NULL
            GROUP BY
                a.id,
                a.ingest_id,
                a.library_id,
                a.item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                a.created_at,
                a.updated_at
            ORDER BY a.created_at ASC, a.id ASC
            LIMIT $1 OFFSET $2
            "#;

const MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT: &str = r#"
            SELECT
                a.id::text AS id,
                a.ingest_id::text AS ingest_id,
                a.library_id::text AS library_id,
                a.item_id::text AS item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                COUNT(s.id)::bigint AS selected_artwork_count
            FROM managed_artwork_artifacts a
            LEFT JOIN selected_artworks s ON s.artifact_id = a.id
            WHERE a.deleted_at IS NULL
            GROUP BY
                a.id,
                a.ingest_id,
                a.library_id,
                a.item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                a.created_at,
                a.updated_at
            HAVING COUNT(s.id) = 0
            ORDER BY a.created_at ASC, a.id ASC
            LIMIT $1 OFFSET $2
            "#;

const fn lifecycle_select_sql(filter: ManagedArtworkArtifactLifecycleFilter) -> &'static str {
    match filter {
        ManagedArtworkArtifactLifecycleFilter::All => MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT,
        ManagedArtworkArtifactLifecycleFilter::CleanupCandidates => {
            MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT
        }
    }
}

fn row_to_artwork_candidate(row: PgRow) -> Result<ArtworkCandidateRecord> {
    Ok(ArtworkCandidateRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        side_effect_id: parse_id(row_get::<String>(&row, "side_effect_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        source_kind: ArtworkCandidateSourceKind::parse(&row_get::<String>(&row, "source_kind")?)?,
        source_uri: row_get(&row, "source_uri")?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        language: row_get(&row, "language")?,
        status: ArtworkCandidateStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_artwork_task(row: PgRow) -> Result<ArtworkTask> {
    Ok(ArtworkTask {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        image_id: parse_id(row_get::<String>(&row, "image_id")?)?,
        kind: ArtworkTaskKind::parse(&row_get::<String>(&row, "kind")?)?,
        status: JobStatus::parse(&row_get::<String>(&row, "status")?)?,
        resource_class: row_get(&row, "resource_class")?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        max_attempts: i64_to_u32(row_get(&row, "max_attempts")?)?,
        error: row_get(&row, "error")?,
    })
}

fn row_to_managed_artwork_ingest(row: PgRow) -> Result<ManagedArtworkIngestRecord> {
    Ok(ManagedArtworkIngestRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        candidate_id: parse_id(row_get::<String>(&row, "candidate_id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        status: ManagedArtworkIngestStatus::parse(&row_get::<String>(&row, "status")?)?,
        artifact_id: parse_optional_id(row_get::<Option<String>>(&row, "artifact_id")?)?,
        failure_code: row_get(&row, "failure_code")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_managed_artwork_artifact(row: PgRow) -> Result<ManagedArtworkArtifactRecord> {
    Ok(ManagedArtworkArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        ingest_id: parse_id(row_get::<String>(&row, "ingest_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        storage_uri: row_get(&row, "storage_uri")?,
        content_hash: row_get(&row, "content_hash")?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        byte_len: optional_i64_to_u64(row_get(&row, "byte_len")?)?,
        media_type: row_get(&row, "media_type")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_selected_artwork(row: PgRow) -> Result<SelectedArtworkRecord> {
    Ok(SelectedArtworkRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        artifact_id: parse_id(row_get::<String>(&row, "artifact_id")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_managed_artwork_artifact_lifecycle(
    row: PgRow,
) -> Result<ManagedArtworkArtifactLifecycleRecord> {
    let selected_artwork_count = i64_to_u32(row_get(&row, "selected_artwork_count")?)?;

    Ok(ManagedArtworkArtifactLifecycleRecord {
        artifact: row_to_managed_artwork_artifact(row)?,
        selected_artwork_count,
    })
}

fn row_to_managed_artwork_gallery_candidate(
    row: PgRow,
) -> Result<ManagedArtworkGalleryCandidateRecord> {
    let id = parse_id(row_get::<String>(&row, "id")?)?;
    let addon_id = parse_id(row_get::<String>(&row, "addon_id")?)?;
    let side_effect_id = parse_id(row_get::<String>(&row, "side_effect_id")?)?;
    let library_id = parse_id(row_get::<String>(&row, "library_id")?)?;
    let item_id = parse_id(row_get::<String>(&row, "item_id")?)?;
    let kind = image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?);
    let source_kind = ArtworkCandidateSourceKind::parse(&row_get::<String>(&row, "source_kind")?)?;
    let width = optional_i64_to_u32(row_get(&row, "width")?)?;
    let height = optional_i64_to_u32(row_get(&row, "height")?)?;
    let language = row_get(&row, "language")?;
    let status = ArtworkCandidateStatus::parse(&row_get::<String>(&row, "status")?)?;
    let created_at = row_get(&row, "created_at")?;
    let updated_at = row_get(&row, "updated_at")?;
    let ingest_id: Option<ManagedArtworkIngestId> =
        parse_optional_id(row_get::<Option<String>>(&row, "ingest_id")?)?;
    let ingest = if let Some(ingest_id) = ingest_id {
        Some(ManagedArtworkIngestRecord {
            id: ingest_id,
            candidate_id: id,
            job_id: parse_id(row_get::<String>(&row, "ingest_job_id")?)?,
            library_id,
            item_id,
            kind: kind.clone(),
            status: ManagedArtworkIngestStatus::parse(&row_get::<String>(&row, "ingest_status")?)?,
            artifact_id: parse_optional_id(row_get::<Option<String>>(&row, "ingest_artifact_id")?)?,
            failure_code: row_get(&row, "ingest_failure_code")?,
            created_at: row_get(&row, "ingest_created_at")?,
            updated_at: row_get(&row, "ingest_updated_at")?,
        })
    } else {
        None
    };
    let artifact_id = ingest.as_ref().and_then(|ingest| ingest.artifact_id);

    Ok(ManagedArtworkGalleryCandidateRecord {
        id,
        addon_id,
        side_effect_id,
        library_id,
        item_id,
        kind,
        source_kind,
        width,
        height,
        language,
        status,
        ingest,
        artifact_id,
        selected_artwork_count: i64_to_u32(row_get(&row, "selected_artwork_count")?)?,
        created_at,
        updated_at,
    })
}

fn row_to_managed_artwork_gallery_artifact(
    row: PgRow,
) -> Result<ManagedArtworkGalleryArtifactRecord> {
    managed_artwork_gallery_artifact_from_row(&row, "")
}

fn row_to_managed_artwork_gallery_selected(
    row: PgRow,
) -> Result<ManagedArtworkGallerySelectedRecord> {
    let selected_artwork = SelectedArtworkRecord {
        id: parse_id(row_get::<String>(&row, "selected_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "selected_library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "selected_item_id")?)?,
        kind: image_kind_from_parts(
            row_get(&row, "selected_kind")?,
            row_get(&row, "selected_kind_key")?,
        ),
        artifact_id: parse_id(row_get::<String>(&row, "selected_artifact_id")?)?,
        created_at: row_get(&row, "selected_created_at")?,
        updated_at: row_get(&row, "selected_updated_at")?,
    };
    let artifact = managed_artwork_gallery_artifact_from_row(&row, "artifact_")?;

    Ok(ManagedArtworkGallerySelectedRecord {
        selected_artwork,
        artifact,
    })
}

fn managed_artwork_gallery_artifact_from_row(
    row: &PgRow,
    prefix: &str,
) -> Result<ManagedArtworkGalleryArtifactRecord> {
    Ok(ManagedArtworkGalleryArtifactRecord {
        id: parse_id(row_get::<String>(row, &format!("{prefix}id"))?)?,
        ingest_id: parse_id(row_get::<String>(row, &format!("{prefix}ingest_id"))?)?,
        candidate_id: parse_id(row_get::<String>(row, &format!("{prefix}candidate_id"))?)?,
        library_id: parse_id(row_get::<String>(row, &format!("{prefix}library_id"))?)?,
        item_id: parse_id(row_get::<String>(row, &format!("{prefix}item_id"))?)?,
        kind: image_kind_from_parts(
            row_get(row, &format!("{prefix}kind"))?,
            row_get(row, &format!("{prefix}kind_key"))?,
        ),
        width: optional_i64_to_u32(row_get(row, &format!("{prefix}width"))?)?,
        height: optional_i64_to_u32(row_get(row, &format!("{prefix}height"))?)?,
        byte_len: optional_i64_to_u64(row_get(row, &format!("{prefix}byte_len"))?)?,
        media_type: row_get(row, &format!("{prefix}media_type"))?,
        has_content_hash: row_get(row, &format!("{prefix}has_content_hash"))?,
        selected_artwork_count: i64_to_u32(row_get(
            row,
            &format!("{prefix}selected_artwork_count"),
        )?)?,
        created_at: row_get(row, &format!("{prefix}created_at"))?,
        updated_at: row_get(row, &format!("{prefix}updated_at"))?,
    })
}
