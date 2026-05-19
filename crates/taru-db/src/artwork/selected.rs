use super::*;

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

pub(super) async fn publish_selected_artwork_tx(
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

pub(super) async fn unpublish_selected_artwork_for_item_kind_tx(
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

pub(super) async fn get_selected_artwork(
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

pub(super) async fn list_selected_artwork_for_item(
    pool: &sqlx::SqlitePool,
    item_id: MediaItemId,
) -> Result<Vec<SelectedArtworkRecord>> {
    let rows = sqlx::query(SELECTED_ARTWORK_SELECT_BY_ITEM)
        .bind(item_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(database_error)?;

    rows.into_iter().map(row_to_selected_artwork).collect()
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
