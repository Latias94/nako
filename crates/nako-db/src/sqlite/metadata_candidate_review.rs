use std::collections::HashSet;

use nako_core::*;
use sqlx::sqlite::SqliteRow;

use super::{SqliteStore, codec::*, jobs::insert_job_tx};

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

    async fn get_metadata_candidate_review_batch(
        &self,
        batch_id: MetadataCandidateReviewBatchId,
    ) -> Result<Option<MetadataCandidateReviewBatchRecord>> {
        self.load_metadata_candidate_review_batch(batch_id).await
    }

    async fn find_metadata_candidate_review_batch(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<MetadataCandidateReviewBatchRecord>> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM metadata_candidate_review_batches
            WHERE idempotency_key = ?1
            "#,
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        self.load_metadata_candidate_review_batch(parse_id(row_get::<String>(&row, "id")?)?)
            .await
    }

    async fn commit_metadata_candidate_review_batch(
        &self,
        commit: &MetadataCandidateReviewBatchCommit,
    ) -> Result<MetadataCandidateReviewBatchRecord> {
        validate_metadata_candidate_review_batch_commit(commit)?;
        if let Some(existing) = self
            .find_metadata_candidate_review_batch(&commit.idempotency_key)
            .await?
        {
            return Ok(existing);
        }

        let selection_json = serde_json::to_string(&commit.selection).map_err(database_error)?;
        let summary_json = serde_json::to_string(&commit.summary).map_err(database_error)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        insert_job_tx(&mut transaction, commit.job.clone()).await?;

        sqlx::query(
            r#"
            INSERT INTO metadata_candidate_review_batches (
                id,
                job_id,
                idempotency_key,
                status,
                selection_json,
                summary_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(commit.id.to_string())
        .bind(commit.job.id.to_string())
        .bind(&commit.idempotency_key)
        .bind(commit.status.as_str())
        .bind(selection_json)
        .bind(summary_json)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        for item in &commit.items {
            let plan_json = serde_json::to_string(&item.plan).map_err(database_error)?;
            sqlx::query(
                r#"
                INSERT INTO metadata_candidate_review_batch_items (
                    batch_id,
                    review_id,
                    item_id,
                    position,
                    status,
                    idempotency_key,
                    expected_updated_at_ms,
                    provider_subject_id,
                    provider_mapping_id,
                    error_code,
                    error_message,
                    plan_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, NULL, NULL, ?8)
                "#,
            )
            .bind(commit.id.to_string())
            .bind(item.review_id.to_string())
            .bind(item.item_id.to_string())
            .bind(u32_to_i64(item.position))
            .bind(item.status.as_str())
            .bind(&item.idempotency_key)
            .bind(item.expected_updated_at_ms)
            .bind(plan_json)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        self.load_metadata_candidate_review_batch(commit.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "metadata candidate review batch {} was not found after commit",
                    commit.id
                ),
            })
    }

    async fn commit_metadata_candidate_review_batch_item_outcome(
        &self,
        commit: &MetadataCandidateReviewBatchItemOutcomeCommit,
    ) -> Result<MetadataCandidateReviewBatchRecord> {
        validate_metadata_candidate_review_batch_item_outcome_commit(commit)?;
        let result = sqlx::query(
            r#"
            UPDATE metadata_candidate_review_batch_items
            SET status = ?1,
                provider_subject_id = ?2,
                provider_mapping_id = ?3,
                error_code = ?4,
                error_message = ?5,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE batch_id = ?6 AND review_id = ?7
            "#,
        )
        .bind(commit.status.as_str())
        .bind(commit.provider_subject_id.map(|id| id.to_string()))
        .bind(commit.provider_mapping_id.map(|id| id.to_string()))
        .bind(&commit.error_code)
        .bind(&commit.error_message)
        .bind(commit.batch_id.to_string())
        .bind(commit.review_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(NakoError::NotFound {
                entity: "metadata_candidate_review_batch_item",
                id: format!("{}:{}", commit.batch_id, commit.review_id),
            });
        }

        self.load_metadata_candidate_review_batch(commit.batch_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "metadata candidate review batch {} was not found after item outcome commit",
                    commit.batch_id
                ),
            })
    }

    async fn update_metadata_candidate_review_batch_status(
        &self,
        batch_id: MetadataCandidateReviewBatchId,
        expected: MetadataCandidateReviewBatchStatus,
        status: MetadataCandidateReviewBatchStatus,
    ) -> Result<MetadataCandidateReviewBatchRecord> {
        let result = sqlx::query(
            r#"
            UPDATE metadata_candidate_review_batches
            SET status = ?1,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?2 AND status = ?3
            "#,
        )
        .bind(status.as_str())
        .bind(batch_id.to_string())
        .bind(expected.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            if self
                .load_metadata_candidate_review_batch(batch_id)
                .await?
                .is_none()
            {
                return Err(NakoError::NotFound {
                    entity: "metadata_candidate_review_batch",
                    id: batch_id.to_string(),
                });
            }
            return Err(NakoError::InvalidInput {
                message: format!(
                    "cannot transition metadata candidate review batch {batch_id} from {:?} to {:?}",
                    expected, status
                ),
            });
        }

        self.load_metadata_candidate_review_batch(batch_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "metadata candidate review batch {batch_id} was not found after status update"
                ),
            })
    }
}

impl SqliteStore {
    async fn load_metadata_candidate_review_batch(
        &self,
        batch_id: MetadataCandidateReviewBatchId,
    ) -> Result<Option<MetadataCandidateReviewBatchRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                idempotency_key,
                status,
                selection_json,
                summary_json,
                created_at,
                updated_at
            FROM metadata_candidate_review_batches
            WHERE id = ?1
            "#,
        )
        .bind(batch_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let item_rows = sqlx::query(
            r#"
            SELECT
                batch_id,
                review_id,
                item_id,
                position,
                status,
                idempotency_key,
                expected_updated_at_ms,
                provider_subject_id,
                provider_mapping_id,
                error_code,
                error_message,
                plan_json,
                created_at,
                updated_at
            FROM metadata_candidate_review_batch_items
            WHERE batch_id = ?1
            ORDER BY position ASC
            "#,
        )
        .bind(batch_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        let items = item_rows
            .into_iter()
            .map(row_to_metadata_candidate_review_batch_item)
            .collect::<Result<Vec<_>>>()?;

        row_to_metadata_candidate_review_batch(row, items).map(Some)
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

fn validate_metadata_candidate_review_batch_commit(
    commit: &MetadataCandidateReviewBatchCommit,
) -> Result<()> {
    if commit.job.kind != JobKind::MetadataCandidateReviewBatchApply {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch job must use JobKind::MetadataCandidateReviewBatchApply"
                .to_owned(),
        });
    }
    if commit.job.resource_class != METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch job resource_class is invalid".to_owned(),
        });
    }
    if commit.idempotency_key.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch idempotency_key cannot be empty".to_owned(),
        });
    }
    if commit.status != MetadataCandidateReviewBatchStatus::Queued {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch commits must start queued".to_owned(),
        });
    }
    if commit.items.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch requires at least one item".to_owned(),
        });
    }

    let mut review_ids = HashSet::new();
    let mut item_idempotency_keys = HashSet::new();
    for item in &commit.items {
        if item.idempotency_key.trim().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "metadata candidate review batch item idempotency_key cannot be empty"
                    .to_owned(),
            });
        }
        if !review_ids.insert(item.review_id) {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "metadata candidate review batch contains duplicate review {}",
                    item.review_id
                ),
            });
        }
        if !item_idempotency_keys.insert(item.idempotency_key.as_str()) {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "metadata candidate review batch contains duplicate item idempotency_key {}",
                    item.idempotency_key
                ),
            });
        }
        if item.review_id != item.plan.review_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "metadata candidate review batch item review_id {} does not match plan review_id {}",
                    item.review_id, item.plan.review_id
                ),
            });
        }
        if item.item_id != item.plan.item_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "metadata candidate review batch item item_id {} does not match plan item_id {}",
                    item.item_id, item.plan.item_id
                ),
            });
        }
        if !matches!(
            item.status,
            MetadataCandidateReviewBatchItemStatus::Pending
                | MetadataCandidateReviewBatchItemStatus::Skipped
                | MetadataCandidateReviewBatchItemStatus::Blocked
        ) {
            return Err(NakoError::InvalidInput {
                message: "metadata candidate review batch item commit status must be pre-execution"
                    .to_owned(),
            });
        }
        if item
            .expected_updated_at_ms
            .is_some_and(|updated_at_ms| updated_at_ms < 0)
        {
            return Err(NakoError::InvalidInput {
                message:
                    "metadata candidate review batch item expected_updated_at_ms cannot be negative"
                        .to_owned(),
            });
        }
    }

    Ok(())
}

fn validate_metadata_candidate_review_batch_item_outcome_commit(
    commit: &MetadataCandidateReviewBatchItemOutcomeCommit,
) -> Result<()> {
    if matches!(
        commit.status,
        MetadataCandidateReviewBatchItemStatus::Pending
            | MetadataCandidateReviewBatchItemStatus::Skipped
            | MetadataCandidateReviewBatchItemStatus::Blocked
    ) {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch item outcome must be terminal".to_owned(),
        });
    }
    if commit.error_code.as_deref().is_some_and(str::is_empty)
        || commit.error_message.as_deref().is_some_and(str::is_empty)
    {
        return Err(NakoError::InvalidInput {
            message: "metadata candidate review batch item outcome error fields cannot be empty"
                .to_owned(),
        });
    }

    Ok(())
}

fn row_to_metadata_candidate_review_batch(
    row: SqliteRow,
    items: Vec<MetadataCandidateReviewBatchItemRecord>,
) -> Result<MetadataCandidateReviewBatchRecord> {
    let selection_json: String = row_get(&row, "selection_json")?;
    let summary_json: String = row_get(&row, "summary_json")?;

    Ok(MetadataCandidateReviewBatchRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        status: MetadataCandidateReviewBatchStatus::parse(&row_get::<String>(&row, "status")?)?,
        selection: serde_json::from_str(&selection_json).map_err(database_error)?,
        summary: serde_json::from_str(&summary_json).map_err(database_error)?,
        execution_summary: MetadataCandidateReviewBatchExecutionSummary::from_items(&items),
        items,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_metadata_candidate_review_batch_item(
    row: SqliteRow,
) -> Result<MetadataCandidateReviewBatchItemRecord> {
    let plan_json: String = row_get(&row, "plan_json")?;

    Ok(MetadataCandidateReviewBatchItemRecord {
        batch_id: parse_id(row_get::<String>(&row, "batch_id")?)?,
        review_id: parse_id(row_get::<String>(&row, "review_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        position: i64_to_u32(row_get::<i64>(&row, "position")?)?,
        status: MetadataCandidateReviewBatchItemStatus::parse(&row_get::<String>(&row, "status")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        expected_updated_at_ms: row_get(&row, "expected_updated_at_ms")?,
        provider_subject_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "provider_subject_id",
        )?)?,
        provider_mapping_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "provider_mapping_id",
        )?)?,
        error_code: row_get(&row, "error_code")?,
        error_message: row_get(&row, "error_message")?,
        plan: serde_json::from_str(&plan_json).map_err(database_error)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}
