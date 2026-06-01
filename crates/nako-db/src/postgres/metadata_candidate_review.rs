use sqlx::postgres::PgRow;

use nako_core::*;

use super::{
    PostgresStore, database_error, metadata_candidate_source_from_parts,
    metadata_candidate_source_to_parts, parse_id, row_get, u32_to_i64, u64_to_i64,
};

const METADATA_CANDIDATE_REVIEW_SELECT: &str = r#"
            SELECT
                id::text AS id,
                item_id::text AS item_id,
                source,
                source_kind_key,
                source_key,
                status,
                plan_json::text AS plan_json,
                expires_at_ms,
                created_at_ms,
                updated_at_ms
            FROM metadata_candidate_reviews
            "#;

#[async_trait::async_trait]
impl MetadataCandidateReviewRepository for PostgresStore {
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
            VALUES ($1, $2, $3, $4, $5, 'pending', $6::jsonb, $7, $8, $9)
            ON CONFLICT(item_id, source, source_kind_key, source_key) DO UPDATE SET
                status = 'pending',
                plan_json = excluded.plan_json,
                expires_at_ms = excluded.expires_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(review.id.as_uuid())
        .bind(review.item_id.as_uuid())
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
                message: "failed to load upserted PostgreSQL metadata candidate review".to_owned(),
            })
    }

    async fn get_metadata_candidate_review(
        &self,
        id: MetadataCandidateReviewId,
    ) -> Result<Option<MetadataCandidateReviewRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {METADATA_CANDIDATE_REVIEW_SELECT}
            WHERE id = $1
            "#
        ))
        .bind(id.as_uuid())
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
            WHERE item_id = $1
              AND source = $2
              AND source_kind_key = $3
              AND source_key = $4
            LIMIT 1
            "#
        ))
        .bind(item_id.as_uuid())
        .bind(source)
        .bind(source_kind_key)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_metadata_candidate_review).transpose()
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
            WHERE item_id = $1
            ORDER BY updated_at_ms DESC, id ASC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_metadata_candidate_review)
            .collect()
    }
}

fn row_to_metadata_candidate_review(row: PgRow) -> Result<MetadataCandidateReviewRecord> {
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
