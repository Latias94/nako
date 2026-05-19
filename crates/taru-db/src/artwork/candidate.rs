use super::*;

const ARTWORK_CANDIDATE_SELECT_BY_ID: &str = r#"
            SELECT
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status,
                created_at, updated_at
            FROM addon_artwork_candidates
            WHERE id = ?1
            "#;

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

pub(super) async fn get_artwork_candidate(
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

pub(super) async fn get_artwork_candidate_tx(
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

pub(super) async fn update_artwork_candidate_status_tx(
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
