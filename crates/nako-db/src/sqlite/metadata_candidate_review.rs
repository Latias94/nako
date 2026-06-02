use nako_core::*;
use sqlx::sqlite::SqliteRow;

use super::{SqliteStore, codec::*};

const METADATA_CANDIDATE_REVIEW_SELECT: &str = r#"
            SELECT
                id,
                item_id,
                source,
                source_kind_key,
                source_key,
                status,
                plan_json,
                expires_at_ms,
                created_at_ms,
                updated_at_ms
            FROM metadata_candidate_reviews
            "#;

#[async_trait::async_trait]
impl MetadataCandidateReviewRepository for SqliteStore {
    async fn upsert_metadata_candidate_review(
        &self,
        review: NewMetadataCandidateReview,
    ) -> Result<MetadataCandidateReviewRecord> {
        let (source, source_kind_key) = metadata_candidate_source_to_parts(&review.source);
        let plan_json = serde_json::to_string(&review.plan).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO metadata_candidate_reviews (
                id,
                item_id,
                source,
                source_kind_key,
                source_key,
                status,
                plan_json,
                expires_at_ms,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7, ?8, ?9)
            ON CONFLICT(item_id, source, source_kind_key, source_key) DO UPDATE SET
                status = 'pending',
                plan_json = excluded.plan_json,
                expires_at_ms = excluded.expires_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(review.id.to_string())
        .bind(review.item_id.to_string())
        .bind(&source)
        .bind(&source_kind_key)
        .bind(&review.source_key)
        .bind(plan_json)
        .bind(review.expires_at_ms)
        .bind(review.created_at_ms)
        .bind(review.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_metadata_candidate_review(review.item_id, &review.source, &review.source_key)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: "failed to load upserted metadata candidate review".to_owned(),
            })
    }

    async fn get_metadata_candidate_review(
        &self,
        id: MetadataCandidateReviewId,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {METADATA_CANDIDATE_REVIEW_SELECT}
            WHERE id = ?1
            "#
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_metadata_candidate_review).transpose()
    }

    async fn find_metadata_candidate_review(
        &self,
        item_id: MediaItemId,
        source: &MetadataCandidateSource,
        source_key: &str,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        let (source, source_kind_key) = metadata_candidate_source_to_parts(source);
        let row = sqlx::query(&format!(
            r#"
            {METADATA_CANDIDATE_REVIEW_SELECT}
            WHERE item_id = ?1
              AND source = ?2
              AND source_kind_key = ?3
              AND source_key = ?4
            LIMIT 1
            "#
        ))
        .bind(item_id.to_string())
        .bind(source)
        .bind(source_kind_key)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_metadata_candidate_review).transpose()
    }

    async fn set_metadata_candidate_review_status(
        &self,
        id: MetadataCandidateReviewId,
        status: MetadataCandidateReviewStatus,
        updated_at_ms: i64,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE metadata_candidate_reviews
            SET
                status = ?2,
                updated_at_ms = ?3,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_metadata_candidate_review(id).await
    }

    async fn list_metadata_candidate_reviews_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MetadataCandidateReviewRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {METADATA_CANDIDATE_REVIEW_SELECT}
            WHERE item_id = ?1
            ORDER BY updated_at_ms DESC, id ASC
            LIMIT ?2 OFFSET ?3
            "#
        ))
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_metadata_candidate_review)
            .collect()
    }

    async fn list_metadata_candidate_reviews(
        &self,
        filter: MetadataCandidateReviewQueueFilter,
        page: PageRequest,
    ) -> Result<Vec<MetadataCandidateReviewRecord>> {
        let page = page.clamped();
        let provider_parts = filter.provider.map(|provider| {
            metadata_candidate_source_to_parts(&MetadataCandidateSource::Provider(provider))
        });

        let rows = match (filter.status, provider_parts) {
            (Some(status), Some((source, source_kind_key))) => sqlx::query(&format!(
                r#"
                    {METADATA_CANDIDATE_REVIEW_SELECT}
                    WHERE status = ?1
                      AND source = ?2
                      AND source_kind_key = ?3
                    ORDER BY updated_at_ms DESC, id ASC
                    LIMIT ?4 OFFSET ?5
                    "#
            ))
            .bind(status.as_str())
            .bind(source)
            .bind(source_kind_key)
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?,
            (Some(status), None) => sqlx::query(&format!(
                r#"
                    {METADATA_CANDIDATE_REVIEW_SELECT}
                    WHERE status = ?1
                    ORDER BY updated_at_ms DESC, id ASC
                    LIMIT ?2 OFFSET ?3
                    "#
            ))
            .bind(status.as_str())
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?,
            (None, Some((source, source_kind_key))) => sqlx::query(&format!(
                r#"
                    {METADATA_CANDIDATE_REVIEW_SELECT}
                    WHERE source = ?1
                      AND source_kind_key = ?2
                    ORDER BY updated_at_ms DESC, id ASC
                    LIMIT ?3 OFFSET ?4
                    "#
            ))
            .bind(source)
            .bind(source_kind_key)
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?,
            (None, None) => sqlx::query(&format!(
                r#"
                    {METADATA_CANDIDATE_REVIEW_SELECT}
                    ORDER BY updated_at_ms DESC, id ASC
                    LIMIT ?1 OFFSET ?2
                    "#
            ))
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?,
        };

        rows.into_iter()
            .map(row_to_metadata_candidate_review)
            .collect()
    }
}

fn row_to_metadata_candidate_review(row: SqliteRow) -> Result<MetadataCandidateReviewRecord> {
    let plan_json: String = row_get(&row, "plan_json")?;
    let plan = serde_json::from_str(&plan_json).map_err(database_error)?;

    Ok(MetadataCandidateReviewRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        source: metadata_candidate_source_from_parts(
            row_get(&row, "source")?,
            row_get(&row, "source_kind_key")?,
        ),
        source_key: row_get(&row, "source_key")?,
        status: MetadataCandidateReviewStatus::parse(&row_get::<String>(&row, "status")?)?,
        plan,
        expires_at_ms: row_get(&row, "expires_at_ms")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}
