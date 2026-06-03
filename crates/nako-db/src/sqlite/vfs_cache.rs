use super::{SqliteStore, codec::*};
use nako_core::*;

#[async_trait::async_trait]
impl VfsCacheRepository for SqliteStore {
    async fn upsert_vfs_cache_object(&self, object: &VfsCachedObject) -> Result<()> {
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
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
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
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(uri) DO UPDATE SET
                scheme = excluded.scheme,
                fetched_at_ms = excluded.fetched_at_ms,
                fresh_until_ms = excluded.fresh_until_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(&listing.directory.uri)
        .bind(&listing.directory.scheme)
        .bind(listing.fetched_at_ms)
        .bind(listing.fresh_until_ms)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM vfs_cache_listing_entries WHERE listing_uri = ?1")
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
                VALUES (?1, ?2, ?3)
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
        let row = sqlx::query(
            r#"
            SELECT
                uri, scheme, kind, len, modified_at, etag, fingerprint,
                capabilities_bits, fetched_at_ms, fresh_until_ms
            FROM vfs_cache_objects
            WHERE uri = ?1
            "#,
        )
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
            WHERE uri = ?1
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

        let entry_rows = sqlx::query(
            r#"
            SELECT
                object.uri, object.scheme, object.kind, object.len,
                object.modified_at, object.etag, object.fingerprint,
                object.capabilities_bits, object.fetched_at_ms, object.fresh_until_ms
            FROM vfs_cache_listing_entries entry
            JOIN vfs_cache_objects object ON object.uri = entry.entry_uri
            WHERE entry.listing_uri = ?1
            ORDER BY entry.sort_order ASC, entry.entry_uri ASC
            "#,
        )
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
            VALUES (?1, ?2, ?3, ?4, 1, ?5)
            ON CONFLICT(uri, operation) DO UPDATE SET
                scheme = excluded.scheme,
                failed_at_ms = excluded.failed_at_ms,
                failure_count = vfs_cache_failures.failure_count + 1,
                error = excluded.error,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
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
            WHERE uri = ?1 AND operation = ?2
            "#,
        )
        .bind(uri)
        .bind(operation.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_vfs_cache_failure).transpose()
    }

    async fn get_latest_vfs_cache_failure(&self) -> Result<Option<VfsCacheFailure>> {
        let row = sqlx::query(
            r#"
            SELECT uri, scheme, operation, failed_at_ms, failure_count, error
            FROM vfs_cache_failures
            ORDER BY failed_at_ms DESC, uri ASC, operation ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_vfs_cache_failure).transpose()
    }

    async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM vfs_cache_objects) AS object_count,
                (SELECT COUNT(*) FROM vfs_cache_listings) AS listing_count,
                (SELECT COUNT(*) FROM vfs_cache_failures) AS failure_count,
                (SELECT COUNT(*) FROM vfs_cache_objects WHERE fresh_until_ms < ?1) AS stale_object_count,
                (SELECT COUNT(*) FROM vfs_cache_listings WHERE fresh_until_ms < ?1) AS stale_listing_count,
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
