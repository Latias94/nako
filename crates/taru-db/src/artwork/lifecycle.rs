use super::*;

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

pub(super) async fn managed_artwork_artifact_lifecycle_summary(
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

pub(super) async fn managed_artwork_artifact_lifecycle_rows(
    pool: &sqlx::SqlitePool,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let sql = lifecycle_select_sql(filter);
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

pub(super) async fn managed_artwork_artifact_lifecycle_rows_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let sql = lifecycle_select_sql(filter);
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

const fn lifecycle_select_sql(filter: ManagedArtworkArtifactLifecycleFilter) -> &'static str {
    match filter {
        ManagedArtworkArtifactLifecycleFilter::All => MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT,
        ManagedArtworkArtifactLifecycleFilter::CleanupCandidates => {
            MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT
        }
    }
}
