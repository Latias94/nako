use sqlx::{PgPool, Postgres, postgres::PgRow};

use super::{
    PostgresStore, database_error, i64_to_u32, i64_to_u64, optional_i64_to_u64,
    optional_u64_to_i64, parse_id, parse_optional_id, row_get, u32_to_i64, u64_to_i64,
};
use nako_core::*;

const VFS_CACHE_OBJECT_SELECT: &str = r#"
            SELECT
                uri,
                scheme,
                kind,
                len,
                modified_at,
                etag,
                fingerprint,
                capabilities_bits,
                fetched_at_ms,
                fresh_until_ms
            FROM vfs_cache_objects
            "#;

const STAGING_MANIFEST_RECORD_SELECT: &str = r#"
            SELECT
                id::text AS id,
                attribution_kind,
                attribution_library_id::text AS attribution_library_id,
                source_uri,
                source_scheme,
                purpose,
                local_path,
                size_bytes,
                etag,
                fingerprint,
                state,
                created_at_ms,
                updated_at_ms,
                last_accessed_at_ms,
                expires_at_ms,
                active_leases,
                validation_error
            FROM staging_manifest_records
            "#;

#[async_trait::async_trait]
impl VfsCacheRepository for PostgresStore {
    async fn upsert_vfs_cache_object(&self, object: &VfsCachedObject) -> Result<()> {
        upsert_vfs_cache_object(&self.pool, object).await
    }

    async fn upsert_vfs_cache_listing(&self, listing: &VfsCachedListing) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_vfs_cache_object_tx(&mut transaction, &listing.directory).await?;
        for entry in &listing.entries {
            upsert_vfs_cache_object_tx(&mut transaction, entry).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO vfs_cache_listings (
                uri, scheme, fetched_at_ms, fresh_until_ms
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(uri) DO UPDATE SET
                scheme = excluded.scheme,
                fetched_at_ms = excluded.fetched_at_ms,
                fresh_until_ms = excluded.fresh_until_ms,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(&listing.directory.uri)
        .bind(&listing.directory.scheme)
        .bind(listing.fetched_at_ms)
        .bind(listing.fresh_until_ms)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM vfs_cache_listing_entries WHERE listing_uri = $1")
            .bind(&listing.directory.uri)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for (index, entry) in listing.entries.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO vfs_cache_listing_entries (
                    listing_uri, entry_uri, sort_order
                )
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(&listing.directory.uri)
            .bind(&entry.uri)
            .bind(u64_to_i64(index as u64)?)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;
        Ok(())
    }

    async fn get_vfs_cache_object(&self, uri: &str) -> Result<Option<VfsCachedObject>> {
        let row = sqlx::query(&format!(
            r#"
            {VFS_CACHE_OBJECT_SELECT}
            WHERE uri = $1
            "#
        ))
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_vfs_cached_object).transpose()
    }

    async fn get_vfs_cache_listing(&self, uri: &str) -> Result<Option<VfsCachedListing>> {
        let listing_row = sqlx::query(
            r#"
            SELECT uri, fetched_at_ms, fresh_until_ms
            FROM vfs_cache_listings
            WHERE uri = $1
            "#,
        )
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(listing_row) = listing_row else {
            return Ok(None);
        };

        let directory =
            self.get_vfs_cache_object(uri)
                .await?
                .ok_or_else(|| NakoError::Database {
                    message: format!("VFS cache listing missing directory object: {uri}"),
                })?;

        let entry_rows = sqlx::query(&format!(
            r#"
            {VFS_CACHE_OBJECT_SELECT}
            JOIN vfs_cache_listing_entries entry ON entry.entry_uri = vfs_cache_objects.uri
            WHERE entry.listing_uri = $1
            ORDER BY entry.sort_order ASC, entry.entry_uri ASC
            "#
        ))
        .bind(uri)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(Some(VfsCachedListing {
            directory,
            entries: entry_rows
                .into_iter()
                .map(row_to_vfs_cached_object)
                .collect::<Result<Vec<_>>>()?,
            fetched_at_ms: row_get(&listing_row, "fetched_at_ms")?,
            fresh_until_ms: row_get(&listing_row, "fresh_until_ms")?,
        }))
    }

    async fn record_vfs_cache_failure(
        &self,
        failure: NewVfsCacheFailure,
    ) -> Result<VfsCacheFailure> {
        sqlx::query(
            r#"
            INSERT INTO vfs_cache_failures (
                uri, scheme, operation, failed_at_ms, failure_count, error
            )
            VALUES ($1, $2, $3, $4, 1, $5)
            ON CONFLICT(uri, operation) DO UPDATE SET
                scheme = excluded.scheme,
                failed_at_ms = excluded.failed_at_ms,
                failure_count = vfs_cache_failures.failure_count + 1,
                error = excluded.error,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(&failure.uri)
        .bind(&failure.scheme)
        .bind(failure.operation.as_str())
        .bind(failure.failed_at_ms)
        .bind(&failure.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_vfs_cache_failure(&failure.uri, failure.operation)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "vfs_cache_failure",
                id: format!("{}:{}", failure.uri, failure.operation.as_str()),
            })
    }

    async fn get_vfs_cache_failure(
        &self,
        uri: &str,
        operation: VfsCacheOperation,
    ) -> Result<Option<VfsCacheFailure>> {
        let row = sqlx::query(
            r#"
            SELECT uri, scheme, operation, failed_at_ms, failure_count, error
            FROM vfs_cache_failures
            WHERE uri = $1 AND operation = $2
            "#,
        )
        .bind(uri)
        .bind(operation.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_vfs_cache_failure).transpose()
    }

    async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM vfs_cache_objects)::bigint AS object_count,
                (SELECT COUNT(*) FROM vfs_cache_listings)::bigint AS listing_count,
                (SELECT COUNT(*) FROM vfs_cache_failures)::bigint AS failure_count,
                (SELECT COUNT(*) FROM vfs_cache_objects WHERE fresh_until_ms < $1)::bigint AS stale_object_count,
                (SELECT COUNT(*) FROM vfs_cache_listings WHERE fresh_until_ms < $1)::bigint AS stale_listing_count,
                (SELECT MAX(failed_at_ms) FROM vfs_cache_failures) AS last_failure_at_ms
            "#,
        )
        .bind(now_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(VfsCacheSummary {
            object_count: i64_to_u64(row_get::<i64>(&row, "object_count")?)?,
            listing_count: i64_to_u64(row_get::<i64>(&row, "listing_count")?)?,
            failure_count: i64_to_u64(row_get::<i64>(&row, "failure_count")?)?,
            stale_object_count: i64_to_u64(row_get::<i64>(&row, "stale_object_count")?)?,
            stale_listing_count: i64_to_u64(row_get::<i64>(&row, "stale_listing_count")?)?,
            last_failure_at_ms: row_get(&row, "last_failure_at_ms")?,
        })
    }
}

#[async_trait::async_trait]
impl StagingManifestRepository for PostgresStore {
    async fn upsert_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        upsert_staging_manifest_record(&self.pool, record).await
    }

    async fn reserve_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
        max_total_bytes: u64,
        now_ms: i64,
    ) -> Result<StagingManifestRecord> {
        if record.state != StagingState::Reserved {
            return Err(NakoError::InvalidInput {
                message: "staging reservation must use reserved state".to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let existing_row = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE local_path = $1
            FOR UPDATE
            "#
        ))
        .bind(&record.local_path)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;
        let existing = existing_row
            .map(row_to_staging_manifest_record)
            .transpose()?;

        if existing.as_ref().is_some_and(|existing| {
            matches!(
                existing.state,
                StagingState::Reserved | StagingState::Staging
            ) && !record_expired(existing, now_ms)
        }) {
            return Err(NakoError::storage_resource_budget_closed(
                record.source_uri,
                format!("staging input is already reserved: {}", record.local_path),
            ));
        }
        if existing.as_ref().is_some_and(|existing| {
            existing.state == StagingState::Leased || existing.active_leases > 0
        }) {
            return Err(NakoError::Conflict {
                message: format!("staging input is actively leased: {}", record.local_path),
            });
        }

        let incoming_bytes = record.size_bytes.unwrap_or(0);
        let existing_bytes = existing
            .as_ref()
            .filter(|existing| staging_state_counts_toward_budget(existing.state))
            .and_then(|existing| existing.size_bytes)
            .unwrap_or(0);
        let additional_bytes = incoming_bytes.saturating_sub(existing_bytes);
        let used_bytes = sum_staging_manifest_bytes_tx(&mut transaction).await?;
        let projected_bytes = used_bytes.saturating_add(additional_bytes);

        if additional_bytes > 0 && projected_bytes > max_total_bytes {
            return Err(NakoError::storage_staging_budget_exhausted(
                record.source_uri,
                format!(
                    "staging disk budget exhausted: used={used_bytes}, additional={additional_bytes}, max={max_total_bytes}",
                ),
            ));
        }

        let record = match existing {
            Some(existing) => NewStagingManifestRecord {
                id: existing.id,
                created_at_ms: existing.created_at_ms,
                ..record
            },
            None => record,
        };
        let saved = upsert_staging_manifest_record_tx(&mut transaction, record).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(saved)
    }

    async fn start_staging_manifest_record(
        &self,
        id: StagingManifestId,
        started_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        let result = sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3
            WHERE id = $1
              AND state = $4
              AND active_leases = 0
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Staging.as_str())
        .bind(started_at_ms)
        .bind(StagingState::Reserved.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(NakoError::Conflict {
                message: format!("staging manifest {id} is not reserved"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn complete_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        if record.state != StagingState::Ready {
            return Err(NakoError::InvalidInput {
                message: "completed staging manifest must use ready state".to_owned(),
            });
        }

        self.upsert_staging_manifest_record(record).await
    }

    async fn fail_staging_manifest_record(
        &self,
        id: StagingManifestId,
        failed_at_ms: i64,
        validation_error: String,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3,
                active_leases = 0,
                validation_error = $4
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Failed.as_str())
        .bind(failed_at_ms)
        .bind(validation_error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn expire_staging_manifest_record(
        &self,
        id: StagingManifestId,
        expired_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3,
                active_leases = 0
            WHERE id = $1
              AND active_leases = 0
              AND state IN ('reserved', 'staging', 'ready', 'failed', 'expired')
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Expired.as_str())
        .bind(expired_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn mark_deleted_staging_manifest_record(
        &self,
        id: StagingManifestId,
        deleted_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3,
                active_leases = 0,
                expires_at_ms = NULL
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Deleted.as_str())
        .bind(deleted_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn acquire_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        leased_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        let result = sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET active_leases = active_leases + 1,
                state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3
            WHERE id = $1
              AND state IN ('ready', 'leased')
              AND active_leases >= 0
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Leased.as_str())
        .bind(leased_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(NakoError::Conflict {
                message: format!("staging manifest {id} is not ready to lease"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn release_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        released_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        let result = sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET active_leases = active_leases - 1,
                state = CASE
                    WHEN active_leases - 1 = 0 THEN $2
                    ELSE $3
                END,
                updated_at_ms = $4,
                last_accessed_at_ms = $4
            WHERE id = $1
              AND state = $3
              AND active_leases > 0
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Ready.as_str())
        .bind(StagingState::Leased.as_str())
        .bind(released_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(NakoError::Conflict {
                message: format!("staging manifest {id} has no active lease to release"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn get_staging_manifest_record(
        &self,
        id: StagingManifestId,
    ) -> Result<Option<StagingManifestRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE id = $1
            "#
        ))
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_staging_manifest_record).transpose()
    }

    async fn find_staging_manifest_record_by_path(
        &self,
        local_path: &str,
    ) -> Result<Option<StagingManifestRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE local_path = $1
            "#
        ))
        .bind(local_path)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_staging_manifest_record).transpose()
    }

    async fn list_staging_manifest_records(
        &self,
        purpose: Option<StagingPurpose>,
        state: Option<StagingState>,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        let page = page.clamped();
        let purpose = purpose.map(|purpose| purpose.as_str().to_owned());
        let state = state.map(|state| state.as_str().to_owned());
        let rows = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE ($1::text IS NULL OR purpose = $1)
              AND ($2::text IS NULL OR state = $2)
            ORDER BY last_accessed_at_ms ASC, id ASC
            LIMIT $3 OFFSET $4
            "#
        ))
        .bind(purpose.as_deref())
        .bind(state.as_deref())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_staging_manifest_record)
            .collect()
    }

    async fn list_staging_cleanup_candidates(
        &self,
        now_ms: i64,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE state IN ('reserved', 'staging', 'ready', 'failed', 'expired')
              AND active_leases = 0
              AND expires_at_ms IS NOT NULL
              AND expires_at_ms <= $1
            ORDER BY last_accessed_at_ms ASC, id ASC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(now_ms)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_staging_manifest_record)
            .collect()
    }

    async fn touch_staging_manifest_record(
        &self,
        id: StagingManifestId,
        accessed_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET last_accessed_at_ms = $2,
                updated_at_ms = $2
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(accessed_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn delete_staging_manifest_record(&self, id: StagingManifestId) -> Result<()> {
        sqlx::query("DELETE FROM staging_manifest_records WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn sum_staging_manifest_bytes(&self) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(size_bytes), 0)::bigint AS total_bytes
            FROM staging_manifest_records
            WHERE size_bytes IS NOT NULL
              AND state IN ('reserved', 'staging', 'ready', 'leased')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        i64_to_u64(row_get::<i64>(&row, "total_bytes")?)
    }
}

async fn upsert_vfs_cache_object(pool: &PgPool, object: &VfsCachedObject) -> Result<()> {
    sqlx::query(vfs_cache_object_upsert_sql())
        .bind(&object.uri)
        .bind(&object.scheme)
        .bind(object.kind.as_str())
        .bind(optional_u64_to_i64(object.len)?)
        .bind(&object.modified_at)
        .bind(&object.etag)
        .bind(&object.fingerprint)
        .bind(u32_to_i64(object.capabilities_bits))
        .bind(object.fetched_at_ms)
        .bind(object.fresh_until_ms)
        .execute(pool)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn upsert_vfs_cache_object_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    object: &VfsCachedObject,
) -> Result<()> {
    sqlx::query(vfs_cache_object_upsert_sql())
        .bind(&object.uri)
        .bind(&object.scheme)
        .bind(object.kind.as_str())
        .bind(optional_u64_to_i64(object.len)?)
        .bind(&object.modified_at)
        .bind(&object.etag)
        .bind(&object.fingerprint)
        .bind(u32_to_i64(object.capabilities_bits))
        .bind(object.fetched_at_ms)
        .bind(object.fresh_until_ms)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

fn vfs_cache_object_upsert_sql() -> &'static str {
    r#"
    INSERT INTO vfs_cache_objects (
        uri, scheme, kind, len, modified_at, etag, fingerprint,
        capabilities_bits, fetched_at_ms, fresh_until_ms
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ON CONFLICT(uri) DO UPDATE SET
        scheme = excluded.scheme,
        kind = excluded.kind,
        len = excluded.len,
        modified_at = excluded.modified_at,
        etag = excluded.etag,
        fingerprint = excluded.fingerprint,
        capabilities_bits = excluded.capabilities_bits,
        fetched_at_ms = excluded.fetched_at_ms,
        fresh_until_ms = excluded.fresh_until_ms,
        updated_at = statement_timestamp()
    "#
}

async fn upsert_staging_manifest_record(
    pool: &PgPool,
    record: NewStagingManifestRecord,
) -> Result<StagingManifestRecord> {
    let (attribution_kind, attribution_library_id) = record.attribution.as_parts();
    sqlx::query(staging_manifest_record_upsert_sql())
        .bind(record.id.as_uuid())
        .bind(attribution_kind.as_str())
        .bind(attribution_library_id.map(|id| id.as_uuid()))
        .bind(&record.source_uri)
        .bind(&record.source_scheme)
        .bind(record.purpose.as_str())
        .bind(&record.local_path)
        .bind(optional_u64_to_i64(record.size_bytes)?)
        .bind(&record.etag)
        .bind(&record.fingerprint)
        .bind(record.state.as_str())
        .bind(record.created_at_ms)
        .bind(record.updated_at_ms)
        .bind(record.last_accessed_at_ms)
        .bind(record.expires_at_ms)
        .bind(u32_to_i64(record.active_leases))
        .bind(&record.validation_error)
        .execute(pool)
        .await
        .map_err(database_error)?;

    get_staging_manifest_record(pool, record.id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "staging_manifest_record",
            id: record.id.to_string(),
        })
}

async fn upsert_staging_manifest_record_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    record: NewStagingManifestRecord,
) -> Result<StagingManifestRecord> {
    let (attribution_kind, attribution_library_id) = record.attribution.as_parts();
    sqlx::query(staging_manifest_record_upsert_sql())
        .bind(record.id.as_uuid())
        .bind(attribution_kind.as_str())
        .bind(attribution_library_id.map(|id| id.as_uuid()))
        .bind(&record.source_uri)
        .bind(&record.source_scheme)
        .bind(record.purpose.as_str())
        .bind(&record.local_path)
        .bind(optional_u64_to_i64(record.size_bytes)?)
        .bind(&record.etag)
        .bind(&record.fingerprint)
        .bind(record.state.as_str())
        .bind(record.created_at_ms)
        .bind(record.updated_at_ms)
        .bind(record.last_accessed_at_ms)
        .bind(record.expires_at_ms)
        .bind(u32_to_i64(record.active_leases))
        .bind(&record.validation_error)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    get_staging_manifest_record_tx(transaction, record.id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "staging_manifest_record",
            id: record.id.to_string(),
        })
}

fn staging_manifest_record_upsert_sql() -> &'static str {
    r#"
    INSERT INTO staging_manifest_records (
        id, attribution_kind, attribution_library_id,
        source_uri, source_scheme, purpose, local_path, size_bytes,
        etag, fingerprint, state, created_at_ms, updated_at_ms,
        last_accessed_at_ms, expires_at_ms, active_leases, validation_error
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
    ON CONFLICT(id) DO UPDATE SET
        attribution_kind = excluded.attribution_kind,
        attribution_library_id = excluded.attribution_library_id,
        source_uri = excluded.source_uri,
        source_scheme = excluded.source_scheme,
        purpose = excluded.purpose,
        local_path = excluded.local_path,
        size_bytes = excluded.size_bytes,
        etag = excluded.etag,
        fingerprint = excluded.fingerprint,
        state = excluded.state,
        updated_at_ms = excluded.updated_at_ms,
        last_accessed_at_ms = excluded.last_accessed_at_ms,
        expires_at_ms = excluded.expires_at_ms,
        active_leases = excluded.active_leases,
        validation_error = excluded.validation_error
    "#
}

async fn get_staging_manifest_record(
    pool: &PgPool,
    id: StagingManifestId,
) -> Result<Option<StagingManifestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {STAGING_MANIFEST_RECORD_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_staging_manifest_record).transpose()
}

async fn get_staging_manifest_record_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: StagingManifestId,
) -> Result<Option<StagingManifestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {STAGING_MANIFEST_RECORD_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_staging_manifest_record).transpose()
}

async fn sum_staging_manifest_bytes_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<u64> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(size_bytes), 0)::bigint AS total_bytes
        FROM staging_manifest_records
        WHERE size_bytes IS NOT NULL
          AND state IN ('reserved', 'staging', 'ready', 'leased')
        "#,
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;

    i64_to_u64(row_get::<i64>(&row, "total_bytes")?)
}

fn record_expired(record: &StagingManifestRecord, now_ms: i64) -> bool {
    record
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_ms)
}

fn staging_state_counts_toward_budget(state: StagingState) -> bool {
    matches!(
        state,
        StagingState::Reserved | StagingState::Staging | StagingState::Ready | StagingState::Leased
    )
}

fn row_to_vfs_cached_object(row: PgRow) -> Result<VfsCachedObject> {
    Ok(VfsCachedObject {
        uri: row_get(&row, "uri")?,
        scheme: row_get(&row, "scheme")?,
        kind: VfsCachedObjectKind::parse(&row_get::<String>(&row, "kind")?)?,
        len: optional_i64_to_u64(row_get(&row, "len")?)?,
        modified_at: row_get(&row, "modified_at")?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        capabilities_bits: i64_to_u32(row_get(&row, "capabilities_bits")?)?,
        fetched_at_ms: row_get(&row, "fetched_at_ms")?,
        fresh_until_ms: row_get(&row, "fresh_until_ms")?,
    })
}

fn row_to_vfs_cache_failure(row: PgRow) -> Result<VfsCacheFailure> {
    Ok(VfsCacheFailure {
        uri: row_get(&row, "uri")?,
        scheme: row_get(&row, "scheme")?,
        operation: VfsCacheOperation::parse(&row_get::<String>(&row, "operation")?)?,
        failed_at_ms: row_get(&row, "failed_at_ms")?,
        failure_count: i64_to_u32(row_get(&row, "failure_count")?)?,
        error: row_get(&row, "error")?,
    })
}

fn row_to_staging_manifest_record(row: PgRow) -> Result<StagingManifestRecord> {
    let attribution_kind =
        StagingAttributionKind::parse(&row_get::<String>(&row, "attribution_kind")?)?;
    let attribution_library_id =
        parse_optional_id(row_get::<Option<String>>(&row, "attribution_library_id")?)?;

    Ok(StagingManifestRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        attribution: StagingAttribution::from_parts(attribution_kind, attribution_library_id)?,
        source_uri: row_get(&row, "source_uri")?,
        source_scheme: row_get(&row, "source_scheme")?,
        purpose: StagingPurpose::parse(&row_get::<String>(&row, "purpose")?)?,
        local_path: row_get(&row, "local_path")?,
        size_bytes: optional_i64_to_u64(row_get::<Option<i64>>(&row, "size_bytes")?)?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        state: StagingState::parse(&row_get::<String>(&row, "state")?)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        last_accessed_at_ms: row_get(&row, "last_accessed_at_ms")?,
        expires_at_ms: row_get(&row, "expires_at_ms")?,
        active_leases: i64_to_u32(row_get(&row, "active_leases")?)?,
        validation_error: row_get(&row, "validation_error")?,
    })
}
