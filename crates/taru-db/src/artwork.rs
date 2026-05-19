use super::*;

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
impl ArtworkCandidateRepository for SqliteStore {
    async fn create_artwork_candidate(
        &self,
        candidate: NewArtworkCandidate,
    ) -> Result<ArtworkCandidateRecord> {
        let (kind, kind_key) = image_kind_to_parts(&candidate.kind);
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO addon_artwork_candidates (
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(candidate.id.to_string())
        .bind(candidate.addon_id.to_string())
        .bind(candidate.side_effect_id.to_string())
        .bind(candidate.library_id.to_string())
        .bind(candidate.item_id.to_string())
        .bind(kind)
        .bind(kind_key)
        .bind(candidate.source_kind.as_str())
        .bind(&candidate.source_uri)
        .bind(optional_u32_to_i64(candidate.width))
        .bind(optional_u32_to_i64(candidate.height))
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
        .ok_or_else(|| TaruError::Database {
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
        sqlx::query(
            r#"
            UPDATE addon_artwork_candidates
            SET status = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        get_artwork_candidate(&self.pool, id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: id.to_string(),
            })
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
        let row = sqlx::query(
            r#"
            SELECT
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status,
                created_at, updated_at
            FROM addon_artwork_candidates
            WHERE addon_id = ?1 AND library_id = ?2 AND item_id = ?3
                AND kind = ?4 AND kind_key = ?5 AND source_kind = ?6
                AND source_uri = ?7
            LIMIT 1
            "#,
        )
        .bind(addon_id.to_string())
        .bind(library_id.to_string())
        .bind(item_id.to_string())
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
        let rows = sqlx::query(
            r#"
            SELECT
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status,
                created_at, updated_at
            FROM addon_artwork_candidates
            WHERE item_id = ?1
            ORDER BY created_at DESC, id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_candidate).collect()
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

        enqueue_job_tx(&mut transaction, job).await?;
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
        let row = sqlx::query(MANAGED_ARTWORK_INGEST_SELECT_BY_CANDIDATE)
            .bind(candidate_id.to_string())
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

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
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

        let candidate = get_artwork_candidate_tx(&mut transaction, ingest.candidate_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: ingest.candidate_id.to_string(),
            })?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
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
            return Err(TaruError::InvalidInput {
                message: "managed artwork artifact ingest_id must match committed ingest"
                    .to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let artifact_id = artifact.id;
        insert_managed_artwork_artifact_tx(&mut transaction, artifact).await?;

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

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact = get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::Database {
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
        .bind(failure_code)
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

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
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

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;

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

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
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
        unpublish_selected_artwork_for_item_kind_tx(&self.pool, item_id, kind).await
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
        let rows = sqlx::query(SELECTED_ARTWORK_SELECT_BY_ITEM)
            .bind(item_id.to_string())
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
        managed_artwork_gallery_for_item(&self.pool, item_id, page).await
    }

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot> {
        let summary = managed_artwork_artifact_lifecycle_summary(&self.pool).await?;
        let rows = managed_artwork_artifact_lifecycle_rows(&self.pool, filter, page).await?;

        Ok(ManagedArtworkArtifactLifecycleSnapshot {
            summary,
            artifacts: rows,
        })
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
                SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                WHERE id = ?1
                    AND deleted_at IS NULL
                    AND NOT EXISTS (
                        SELECT 1
                        FROM selected_artworks s
                        WHERE s.artifact_id = managed_artwork_artifacts.id
                    )
                "#,
            )
            .bind(artifact.id.to_string())
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

const ARTWORK_CANDIDATE_SELECT_BY_ID: &str = r#"
            SELECT
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status,
                created_at, updated_at
            FROM addon_artwork_candidates
            WHERE id = ?1
            "#;

const MANAGED_ARTWORK_INGEST_SELECT_BY_ID: &str = r#"
            SELECT
                id, candidate_id, job_id, library_id, item_id, kind, kind_key,
                status, artifact_id, failure_code, created_at, updated_at
            FROM managed_artwork_ingests
            WHERE id = ?1
            "#;

const MANAGED_ARTWORK_INGEST_SELECT_BY_CANDIDATE: &str = r#"
            SELECT
                id, candidate_id, job_id, library_id, item_id, kind, kind_key,
                status, artifact_id, failure_code, created_at, updated_at
            FROM managed_artwork_ingests
            WHERE candidate_id = ?1
            "#;

const MANAGED_ARTWORK_ARTIFACT_SELECT_BY_ID: &str = r#"
            SELECT
                id, ingest_id, library_id, item_id, kind, kind_key, storage_uri,
                content_hash, width, height, byte_len, media_type,
                created_at, updated_at
            FROM managed_artwork_artifacts
            WHERE id = ?1 AND deleted_at IS NULL
            "#;

const MANAGED_ARTWORK_ARTIFACT_SELECT_BY_INGEST: &str = r#"
            SELECT
                id, ingest_id, library_id, item_id, kind, kind_key, storage_uri,
                content_hash, width, height, byte_len, media_type,
                created_at, updated_at
            FROM managed_artwork_artifacts
            WHERE ingest_id = ?1 AND deleted_at IS NULL
            "#;

const SELECTED_ARTWORK_SELECT_BY_ID: &str = r#"
            SELECT
                id, library_id, item_id, kind, kind_key, artifact_id,
                created_at, updated_at
            FROM selected_artworks
            WHERE id = ?1
            "#;

const SELECTED_ARTWORK_SELECT_BY_SLOT: &str = r#"
            SELECT
                id, library_id, item_id, kind, kind_key, artifact_id,
                created_at, updated_at
            FROM selected_artworks
            WHERE item_id = ?1 AND kind = ?2 AND kind_key = ?3
            "#;

const SELECTED_ARTWORK_SELECT_BY_ITEM: &str = r#"
            SELECT
                id, library_id, item_id, kind, kind_key, artifact_id,
                created_at, updated_at
            FROM selected_artworks
            WHERE item_id = ?1
            ORDER BY kind ASC, id ASC
            "#;

const MANAGED_ARTWORK_GALLERY_CANDIDATE_SELECT_BY_ITEM: &str = r#"
            SELECT
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
                i.id AS ingest_id,
                i.job_id AS ingest_job_id,
                i.status AS ingest_status,
                i.artifact_id AS ingest_artifact_id,
                i.failure_code AS ingest_failure_code,
                i.created_at AS ingest_created_at,
                i.updated_at AS ingest_updated_at,
                COUNT(s.id) AS selected_artwork_count
            FROM addon_artwork_candidates c
            LEFT JOIN managed_artwork_ingests i ON i.candidate_id = c.id
            LEFT JOIN managed_artwork_artifacts a ON a.ingest_id = i.id
                AND a.deleted_at IS NULL
            LEFT JOIN selected_artworks s ON s.artifact_id = a.id
            WHERE c.item_id = ?1
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
            LIMIT ?2 OFFSET ?3
            "#;

const MANAGED_ARTWORK_GALLERY_ARTIFACT_SELECT_BY_ITEM: &str = r#"
            SELECT
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
                a.content_hash IS NOT NULL AS has_content_hash,
                COUNT(s.id) AS selected_artwork_count,
                a.created_at,
                a.updated_at
            FROM managed_artwork_artifacts a
            INNER JOIN managed_artwork_ingests i ON i.id = a.ingest_id
            LEFT JOIN selected_artworks s ON s.artifact_id = a.id
            WHERE a.item_id = ?1 AND a.deleted_at IS NULL
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
            LIMIT ?2 OFFSET ?3
            "#;

const MANAGED_ARTWORK_GALLERY_SELECTED_SELECT_BY_ITEM: &str = r#"
            SELECT
                s.id AS selected_id,
                s.library_id AS selected_library_id,
                s.item_id AS selected_item_id,
                s.kind AS selected_kind,
                s.kind_key AS selected_kind_key,
                s.artifact_id AS selected_artifact_id,
                s.created_at AS selected_created_at,
                s.updated_at AS selected_updated_at,
                a.id AS artifact_id,
                a.ingest_id AS artifact_ingest_id,
                i.candidate_id AS artifact_candidate_id,
                a.library_id AS artifact_library_id,
                a.item_id AS artifact_item_id,
                a.kind AS artifact_kind,
                a.kind_key AS artifact_kind_key,
                a.width AS artifact_width,
                a.height AS artifact_height,
                a.byte_len AS artifact_byte_len,
                a.media_type AS artifact_media_type,
                a.content_hash IS NOT NULL AS artifact_has_content_hash,
                COUNT(linked_s.id) AS artifact_selected_artwork_count,
                a.created_at AS artifact_created_at,
                a.updated_at AS artifact_updated_at
            FROM selected_artworks s
            INNER JOIN managed_artwork_artifacts a ON a.id = s.artifact_id
                AND a.deleted_at IS NULL
            INNER JOIN managed_artwork_ingests i ON i.id = a.ingest_id
            LEFT JOIN selected_artworks linked_s ON linked_s.artifact_id = a.id
            WHERE s.item_id = ?1
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
            "#;

const MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT: &str = r#"
            SELECT
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
                a.updated_at,
                COUNT(s.id) AS selected_artwork_count
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
            LIMIT ?1 OFFSET ?2
            "#;

const MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT: &str = r#"
            SELECT
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
                a.updated_at,
                COUNT(s.id) AS selected_artwork_count
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
            HAVING selected_artwork_count = 0
            ORDER BY a.created_at ASC, a.id ASC
            LIMIT ?1 OFFSET ?2
            "#;

async fn get_artwork_candidate(
    pool: &sqlx::SqlitePool,
    id: ArtworkCandidateId,
) -> Result<Option<ArtworkCandidateRecord>> {
    let row = sqlx::query(ARTWORK_CANDIDATE_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(database_error)?;

    row.map(row_to_artwork_candidate).transpose()
}

async fn get_artwork_candidate_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: ArtworkCandidateId,
) -> Result<Option<ArtworkCandidateRecord>> {
    let row = sqlx::query(ARTWORK_CANDIDATE_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_artwork_candidate).transpose()
}

async fn update_artwork_candidate_status_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: ArtworkCandidateId,
    status: ArtworkCandidateStatus,
) -> Result<ArtworkCandidateRecord> {
    sqlx::query(
        r#"
        UPDATE addon_artwork_candidates
        SET status = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ?1
        "#,
    )
    .bind(id.to_string())
    .bind(status.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    get_artwork_candidate_tx(transaction, id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "artwork_candidate",
            id: id.to_string(),
        })
}

async fn enqueue_job_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    job: NewJob,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO jobs (
            id,
            kind,
            status,
            resource_class,
            library_id,
            source_id,
            input_json
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(job.id.to_string())
    .bind(job.kind.as_str())
    .bind(JobStatus::Queued.as_str())
    .bind(job.resource_class)
    .bind(job.library_id.map(|id| id.to_string()))
    .bind(job.source_id.map(|id| id.to_string()))
    .bind(job.input_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn get_job_tx(transaction: &mut sqlx::Transaction<'_, Sqlite>, id: JobId) -> Result<Job> {
    let row = sqlx::query(
        r#"
        SELECT
            id,
            kind,
            status,
            resource_class,
            library_id,
            source_id,
            input_json,
            summary_json,
            error,
            queued_at,
            started_at,
            completed_at
        FROM jobs
        WHERE id = ?1
        "#,
    )
    .bind(id.to_string())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_job)
        .transpose()?
        .ok_or_else(|| TaruError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
}

async fn insert_managed_artwork_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    ingest: NewManagedArtworkIngest,
) -> Result<ManagedArtworkIngestRecord> {
    let (kind, kind_key) = image_kind_to_parts(&ingest.kind);
    sqlx::query(
        r#"
        INSERT INTO managed_artwork_ingests (
            id, candidate_id, job_id, library_id, item_id, kind, kind_key,
            status, artifact_id, failure_code
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(ingest.id.to_string())
    .bind(ingest.candidate_id.to_string())
    .bind(ingest.job_id.to_string())
    .bind(ingest.library_id.to_string())
    .bind(ingest.item_id.to_string())
    .bind(kind)
    .bind(kind_key)
    .bind(ingest.status.as_str())
    .bind(ingest.artifact_id.map(|id| id.to_string()))
    .bind(ingest.failure_code)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    get_managed_artwork_ingest_tx(transaction, ingest.id)
        .await?
        .ok_or_else(|| TaruError::Database {
            message: "failed to load created managed artwork ingest".to_owned(),
        })
}

async fn get_managed_artwork_ingest(
    pool: &sqlx::SqlitePool,
    id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_INGEST_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn get_managed_artwork_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_INGEST_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn get_managed_artwork_ingest_by_candidate_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    candidate_id: ArtworkCandidateId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_INGEST_SELECT_BY_CANDIDATE)
        .bind(candidate_id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn insert_managed_artwork_artifact_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    artifact: NewManagedArtworkArtifact,
) -> Result<()> {
    let (kind, kind_key) = image_kind_to_parts(&artifact.kind);
    sqlx::query(
        r#"
        INSERT INTO managed_artwork_artifacts (
            id, ingest_id, library_id, item_id, kind, kind_key, storage_uri,
            content_hash, width, height, byte_len, media_type
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
    )
    .bind(artifact.id.to_string())
    .bind(artifact.ingest_id.to_string())
    .bind(artifact.library_id.to_string())
    .bind(artifact.item_id.to_string())
    .bind(kind)
    .bind(kind_key)
    .bind(artifact.storage_uri)
    .bind(artifact.content_hash)
    .bind(optional_u32_to_i64(artifact.width))
    .bind(optional_u32_to_i64(artifact.height))
    .bind(optional_u64_to_i64(artifact.byte_len)?)
    .bind(artifact.media_type)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn get_managed_artwork_artifact(
    pool: &sqlx::SqlitePool,
    id: ManagedArtworkArtifactId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_ARTIFACT_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn get_managed_artwork_artifact_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: ManagedArtworkArtifactId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_ARTIFACT_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn get_managed_artwork_artifact_by_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    ingest_id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_ARTIFACT_SELECT_BY_INGEST)
        .bind(ingest_id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn publish_selected_artwork_tx(
    pool: &sqlx::SqlitePool,
    artifact_id: ManagedArtworkArtifactId,
    expected_slot: Option<(MediaItemId, ImageKind)>,
) -> Result<SelectedArtworkPublicationRecord> {
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let artifact = get_managed_artwork_artifact_tx(&mut transaction, artifact_id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "managed_artwork_artifact",
            id: artifact_id.to_string(),
        })?;

    if let Some((expected_item_id, expected_kind)) = expected_slot.as_ref() {
        if artifact.item_id != *expected_item_id || artifact.kind != *expected_kind {
            return Err(TaruError::Conflict {
                message: "managed artwork artifact does not match the requested item artwork slot"
                    .to_owned(),
            });
        }
    }

    get_managed_artwork_ingest_tx(&mut transaction, artifact.ingest_id)
        .await?
        .filter(|ingest| ingest.artifact_id == Some(artifact.id))
        .filter(|ingest| ingest.status == ManagedArtworkIngestStatus::Stored)
        .ok_or_else(|| TaruError::Conflict {
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
                SET library_id = ?2,
                    artifact_id = ?3,
                    updated_at = CASE
                        WHEN artifact_id = ?3 THEN updated_at
                        ELSE strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    END
                WHERE id = ?1
                "#,
        )
        .bind(existing.id.to_string())
        .bind(artifact.library_id.to_string())
        .bind(artifact.id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    } else {
        sqlx::query(
            r#"
                INSERT INTO selected_artworks (
                    id, library_id, item_id, kind, kind_key, artifact_id
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
        )
        .bind(selected_id.to_string())
        .bind(artifact.library_id.to_string())
        .bind(artifact.item_id.to_string())
        .bind(kind)
        .bind(kind_key)
        .bind(artifact.id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }

    let selected_artwork = get_selected_artwork_tx(&mut transaction, selected_id)
        .await?
        .ok_or_else(|| TaruError::Database {
            message: "failed to load selected artwork publication".to_owned(),
        })?;
    transaction.commit().await.map_err(database_error)?;

    Ok(SelectedArtworkPublicationRecord {
        selected_artwork,
        artifact,
        changed,
    })
}

async fn unpublish_selected_artwork_for_item_kind_tx(
    pool: &sqlx::SqlitePool,
    item_id: MediaItemId,
    kind: ImageKind,
) -> Result<SelectedArtworkUnpublicationRecord> {
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let (kind_part, kind_key) = image_kind_to_parts(&kind);
    let unpublished =
        get_selected_artwork_by_slot_tx(&mut transaction, item_id, &kind_part, &kind_key).await?;
    let artifact = if let Some(selected) = unpublished.as_ref() {
        Some(
            get_managed_artwork_artifact_tx(&mut transaction, selected.artifact_id)
                .await?
                .ok_or_else(|| TaruError::Database {
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
            WHERE id = ?1
            "#,
        )
        .bind(selected.id.to_string())
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

async fn managed_artwork_gallery_for_item(
    pool: &sqlx::SqlitePool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<ManagedArtworkGallerySnapshot> {
    let page = page.clamped();
    let candidates = managed_artwork_gallery_candidates(pool, item_id, page).await?;
    let artifacts = managed_artwork_gallery_artifacts(pool, item_id, page).await?;
    let selected = managed_artwork_gallery_selected(pool, item_id).await?;
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

async fn managed_artwork_gallery_candidates(
    pool: &sqlx::SqlitePool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkGalleryCandidateRecord>> {
    let rows = sqlx::query(MANAGED_ARTWORK_GALLERY_CANDIDATE_SELECT_BY_ITEM)
        .bind(item_id.to_string())
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
    pool: &sqlx::SqlitePool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkGalleryArtifactRecord>> {
    let rows = sqlx::query(MANAGED_ARTWORK_GALLERY_ARTIFACT_SELECT_BY_ITEM)
        .bind(item_id.to_string())
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
    pool: &sqlx::SqlitePool,
    item_id: MediaItemId,
) -> Result<Vec<ManagedArtworkGallerySelectedRecord>> {
    let rows = sqlx::query(MANAGED_ARTWORK_GALLERY_SELECTED_SELECT_BY_ITEM)
        .bind(item_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_selected)
        .collect()
}

fn row_to_managed_artwork_gallery_candidate(
    row: sqlx::sqlite::SqliteRow,
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
    row: sqlx::sqlite::SqliteRow,
) -> Result<ManagedArtworkGalleryArtifactRecord> {
    managed_artwork_gallery_artifact_from_row(&row, "")
}

fn row_to_managed_artwork_gallery_selected(
    row: sqlx::sqlite::SqliteRow,
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
    row: &sqlx::sqlite::SqliteRow,
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
        has_content_hash: row_get::<i64>(row, &format!("{prefix}has_content_hash"))? != 0,
        selected_artwork_count: i64_to_u32(row_get(
            row,
            &format!("{prefix}selected_artwork_count"),
        )?)?,
        created_at: row_get(row, &format!("{prefix}created_at"))?,
        updated_at: row_get(row, &format!("{prefix}updated_at"))?,
    })
}

async fn managed_artwork_artifact_lifecycle_summary(
    pool: &sqlx::SqlitePool,
) -> Result<ManagedArtworkArtifactLifecycleSummary> {
    let rows = sqlx::query(
        r#"
        SELECT
            a.byte_len,
            COUNT(s.id) AS selected_artwork_count
        FROM managed_artwork_artifacts a
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE a.deleted_at IS NULL
        GROUP BY a.id, a.byte_len
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

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

async fn managed_artwork_artifact_lifecycle_rows(
    pool: &sqlx::SqlitePool,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let sql = match filter {
        ManagedArtworkArtifactLifecycleFilter::All => MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT,
        ManagedArtworkArtifactLifecycleFilter::CleanupCandidates => {
            MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT
        }
    };
    let rows = sqlx::query(sql)
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
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let sql = match filter {
        ManagedArtworkArtifactLifecycleFilter::All => MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT,
        ManagedArtworkArtifactLifecycleFilter::CleanupCandidates => {
            MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT
        }
    };
    let rows = sqlx::query(sql)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&mut **transaction)
        .await
        .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_artifact_lifecycle)
        .collect()
}

async fn get_selected_artwork(
    pool: &sqlx::SqlitePool,
    id: SelectedArtworkId,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(SELECTED_ARTWORK_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn get_selected_artwork_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: SelectedArtworkId,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(SELECTED_ARTWORK_SELECT_BY_ID)
        .bind(id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn get_selected_artwork_by_slot_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item_id: MediaItemId,
    kind: &str,
    kind_key: &str,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(SELECTED_ARTWORK_SELECT_BY_SLOT)
        .bind(item_id.to_string())
        .bind(kind)
        .bind(kind_key)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}
