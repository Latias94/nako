use super::*;

mod artifact;
mod candidate;
mod gallery;
mod ingest;
mod lifecycle;
mod selected;

#[async_trait::async_trait]
impl ArtworkTaskRepository for SqliteStore {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO artwork_tasks (
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                image_id = excluded.image_id,
                kind = excluded.kind,
                status = excluded.status,
                resource_class = excluded.resource_class,
                attempts = excluded.attempts,
                max_attempts = excluded.max_attempts,
                error = excluded.error,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(task.id.to_string())
        .bind(task.image_id.to_string())
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
        let row = sqlx::query(
            r#"
            SELECT
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            FROM artwork_tasks
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_artwork_task).transpose()
    }

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            FROM artwork_tasks
            ORDER BY id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_task).collect()
    }
}

#[async_trait::async_trait]
impl ManagedArtworkRepository for SqliteStore {
    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        if let Some(existing) =
            ingest::get_managed_artwork_ingest_by_candidate_tx(&mut transaction, candidate_id)
                .await?
        {
            let candidate = candidate::update_artwork_candidate_status_tx(
                &mut transaction,
                candidate_id,
                ArtworkCandidateStatus::Accepted,
            )
            .await?;
            let job = ingest::get_job_tx(&mut transaction, existing.job_id).await?;
            transaction.commit().await.map_err(database_error)?;
            return Ok(ManagedArtworkAcceptanceRecord {
                candidate,
                ingest: existing,
                job,
            });
        }

        ingest::enqueue_job_tx(&mut transaction, job).await?;
        let saved_ingest =
            ingest::insert_managed_artwork_ingest_tx(&mut transaction, ingest).await?;
        let candidate = candidate::update_artwork_candidate_status_tx(
            &mut transaction,
            candidate_id,
            ArtworkCandidateStatus::Accepted,
        )
        .await?;
        let job = ingest::get_job_tx(&mut transaction, saved_ingest.job_id).await?;
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
        ingest::get_managed_artwork_ingest(&self.pool, id).await
    }

    async fn find_managed_artwork_ingest_by_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        ingest::get_managed_artwork_ingest_by_candidate(&self.pool, candidate_id).await
    }

    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let row = sqlx::query(
            r#"
            SELECT i.id
            FROM managed_artwork_ingests i
            JOIN jobs j ON j.id = i.job_id
            JOIN addon_artwork_candidates c ON c.id = i.candidate_id
            WHERE i.status = ?1
                AND j.status = ?2
                AND j.kind = ?3
                AND j.resource_class = ?4
                AND c.status = ?5
            ORDER BY i.created_at ASC, i.id ASC
            LIMIT 1
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

        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = ?2,
                failure_code = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?3
            "#,
        )
        .bind(ingest.id.to_string())
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
            SET status = ?2,
                started_at = COALESCE(started_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                completed_at = NULL,
                summary_json = NULL,
                error = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?3
            "#,
        )
        .bind(ingest.job_id.to_string())
        .bind(JobStatus::Running.as_str())
        .bind(JobStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let candidate = candidate::get_artwork_candidate_tx(&mut transaction, ingest.candidate_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: ingest.candidate_id.to_string(),
            })?;
        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let job = ingest::get_job_tx(&mut transaction, ingest.job_id).await?;
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
            return Err(TaruError::InvalidInput {
                message: "managed artwork artifact ingest_id must match committed ingest"
                    .to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let artifact_id = artifact.id;
        artifact::insert_managed_artwork_artifact_tx(&mut transaction, artifact).await?;

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = ?2,
                artifact_id = ?3,
                failure_code = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status IN (?4, ?5)
            "#,
        )
        .bind(ingest.id.to_string())
        .bind(ManagedArtworkIngestStatus::Stored.as_str())
        .bind(artifact_id.to_string())
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not claimable for artifact commit".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?2,
                summary_json = ?3,
                error = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?4
            "#,
        )
        .bind(ingest.job_id.to_string())
        .bind(JobStatus::Succeeded.as_str())
        .bind(job_summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not running for artifact commit".to_owned(),
            });
        }

        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact =
            artifact::get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id)
                .await?
                .ok_or_else(|| TaruError::Database {
                    message: "stored managed artwork ingest is missing artifact metadata"
                        .to_owned(),
                })?;
        let job = ingest::get_job_tx(&mut transaction, ingest.job_id).await?;
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
        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = ?2,
                failure_code = ?3,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status IN (?4, ?5)
            "#,
        )
        .bind(ingest.id.to_string())
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .bind(&failure_code)
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not claimable for failure commit".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?2,
                error = ?3,
                summary_json = ?4,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?5
            "#,
        )
        .bind(ingest.job_id.to_string())
        .bind(JobStatus::Failed.as_str())
        .bind(job_error)
        .bind(job_summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not running for failure commit".to_owned(),
            });
        }

        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact =
            artifact::get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id)
                .await?;
        let job = ingest::get_job_tx(&mut transaction, ingest.job_id).await?;
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
            SET status = ?1,
                failure_code = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE status IN (?3, ?4)
                AND artifact_id IS NULL
                AND EXISTS (
                    SELECT 1
                    FROM jobs j
                    WHERE j.id = managed_artwork_ingests.job_id
                        AND j.kind = ?5
                        AND j.resource_class = ?6
                        AND j.status = ?7
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
                SET status = ?1,
                    error = ?2,
                    summary_json = ?3,
                    completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                WHERE kind = ?4
                    AND resource_class = ?5
                    AND status = ?6
                    AND EXISTS (
                        SELECT 1
                        FROM managed_artwork_ingests i
                        WHERE i.job_id = jobs.id
                            AND i.status = ?7
                            AND i.failure_code = ?8
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
        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let job = ingest::get_job_tx(&mut transaction, ingest.job_id).await?;

        if job.kind != JobKind::ManagedArtworkIngest || job.resource_class != "artwork.ingest" {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not an artwork ingest job".to_owned(),
            });
        }

        if ingest.status == ManagedArtworkIngestStatus::Queued {
            if job.status != JobStatus::Queued {
                transaction.rollback().await.map_err(database_error)?;
                return Err(TaruError::Conflict {
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
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not failed or queued for requeue".to_owned(),
            });
        }

        if job.status != JobStatus::Failed {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not failed for requeue".to_owned(),
            });
        }

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = ?2,
                failure_code = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?3 AND artifact_id IS NULL
            "#,
        )
        .bind(ingest.id.to_string())
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not failed or queued for requeue".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?2,
                summary_json = NULL,
                error = NULL,
                started_at = NULL,
                completed_at = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
                AND status = ?3
                AND kind = ?4
                AND resource_class = ?5
            "#,
        )
        .bind(ingest.job_id.to_string())
        .bind(JobStatus::Queued.as_str())
        .bind(JobStatus::Failed.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not failed for requeue".to_owned(),
            });
        }

        let ingest = ingest::get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let job = ingest::get_job_tx(&mut transaction, ingest.job_id).await?;
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
        artifact::get_managed_artwork_artifact(&self.pool, id).await
    }

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        selected::publish_selected_artwork_tx(&self.pool, artifact_id, None).await
    }

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        selected::publish_selected_artwork_tx(&self.pool, artifact_id, Some((item_id, kind))).await
    }

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord> {
        selected::unpublish_selected_artwork_for_item_kind_tx(&self.pool, item_id, kind).await
    }

    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>> {
        selected::get_selected_artwork(&self.pool, id).await
    }

    async fn list_selected_artwork_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<SelectedArtworkRecord>> {
        selected::list_selected_artwork_for_item(&self.pool, item_id).await
    }

    async fn get_managed_artwork_gallery_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<ManagedArtworkGallerySnapshot> {
        gallery::managed_artwork_gallery_for_item(&self.pool, item_id, page).await
    }

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot> {
        let summary = lifecycle::managed_artwork_artifact_lifecycle_summary(&self.pool).await?;
        let rows =
            lifecycle::managed_artwork_artifact_lifecycle_rows(&self.pool, filter, page).await?;

        Ok(ManagedArtworkArtifactLifecycleSnapshot {
            summary,
            artifacts: rows,
        })
    }

    async fn cleanup_unselected_managed_artwork_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactCleanupReport> {
        lifecycle::cleanup_unselected_managed_artwork_artifacts(&self.pool, page).await
    }
}
