use super::*;

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

pub(super) async fn insert_managed_artwork_artifact_tx(
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

pub(super) async fn get_managed_artwork_artifact(
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

pub(super) async fn get_managed_artwork_artifact_tx(
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

pub(super) async fn get_managed_artwork_artifact_by_ingest_tx(
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
