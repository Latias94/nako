use super::*;

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

pub(super) async fn managed_artwork_gallery_for_item(
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
