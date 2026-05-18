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
