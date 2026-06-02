use super::*;

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

pub(super) async fn enqueue_job_tx(
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
            priority,
            library_id,
            source_id,
            input_json
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(job.id.to_string())
    .bind(job.kind.as_str())
    .bind(JobStatus::Queued.as_str())
    .bind(job.resource_class)
    .bind(job.priority.score())
    .bind(job.library_id.map(|id| id.to_string()))
    .bind(job.source_id.map(|id| id.to_string()))
    .bind(job.input_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

pub(super) async fn get_job_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    id: JobId,
) -> Result<Job> {
    let row = sqlx::query(
        r#"
        SELECT
            id,
            kind,
            status,
            resource_class,
            priority,
            library_id,
            source_id,
            input_json,
            summary_json,
            error,
            attempt,
            max_attempts,
            retry_of_job_id,
            next_attempt_at,
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
        .ok_or_else(|| NakoError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
}

pub(super) async fn insert_managed_artwork_ingest_tx(
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
        .ok_or_else(|| NakoError::Database {
            message: "failed to load created managed artwork ingest".to_owned(),
        })
}

pub(super) async fn get_managed_artwork_ingest(
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

pub(super) async fn get_managed_artwork_ingest_by_candidate(
    pool: &sqlx::SqlitePool,
    candidate_id: ArtworkCandidateId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(MANAGED_ARTWORK_INGEST_SELECT_BY_CANDIDATE)
        .bind(candidate_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

pub(super) async fn get_managed_artwork_ingest_tx(
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

pub(super) async fn get_managed_artwork_ingest_by_candidate_tx(
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
