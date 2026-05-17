use sqlx::{Sqlite, Transaction, sqlite::SqliteRow};
use taru_core::{
    NewStagingManifestRecord, PageRequest, Result, StagingManifestId, StagingManifestRecord,
    StagingManifestRepository, StagingPurpose, StagingState, TaruError,
};

use crate::{
    SqliteStore, database_error, i64_to_u32, i64_to_u64, optional_i64_to_u64, optional_u64_to_i64,
    parse_id, row_get, u32_to_i64, u64_to_i64,
};

#[async_trait::async_trait]
impl StagingManifestRepository for SqliteStore {
    async fn upsert_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        sqlx::query(
            r#"
            INSERT INTO staging_manifest_records (
                id, source_uri, source_scheme, purpose, local_path, size_bytes,
                etag, fingerprint, state, created_at_ms, updated_at_ms,
                last_accessed_at_ms, expires_at_ms, active_leases, validation_error
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
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
            "#,
        )
        .bind(record.id.to_string())
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
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(record.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "staging_manifest_record",
                id: record.id.to_string(),
            })
    }

    async fn reserve_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
        max_total_bytes: u64,
        now_ms: i64,
    ) -> Result<StagingManifestRecord> {
        if record.state != StagingState::Reserved {
            return Err(TaruError::InvalidInput {
                message: "staging reservation must use reserved state".to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let existing_row = sqlx::query(
            r#"
            SELECT *
            FROM staging_manifest_records
            WHERE local_path = ?1
            "#,
        )
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
            return Err(TaruError::storage_resource_budget_closed(
                record.source_uri,
                format!("staging input is already reserved: {}", record.local_path),
            ));
        }
        if existing.as_ref().is_some_and(|existing| {
            existing.state == StagingState::Leased || existing.active_leases > 0
        }) {
            return Err(TaruError::Conflict {
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
        let used_bytes = sum_staging_manifest_bytes_in_transaction(&mut transaction).await?;
        let projected_bytes = used_bytes.saturating_add(additional_bytes);

        if additional_bytes > 0 && projected_bytes > max_total_bytes {
            return Err(TaruError::storage_staging_budget_exhausted(
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
        let saved = upsert_staging_manifest_record_in_transaction(&mut transaction, record).await?;
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
            SET state = ?2,
                updated_at_ms = ?3,
                last_accessed_at_ms = ?3
            WHERE id = ?1
              AND state = ?4
              AND active_leases = 0
            "#,
        )
        .bind(id.to_string())
        .bind(StagingState::Staging.as_str())
        .bind(started_at_ms)
        .bind(StagingState::Reserved.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(TaruError::Conflict {
                message: format!("staging manifest {id} is not reserved"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn complete_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        if record.state != StagingState::Ready {
            return Err(TaruError::InvalidInput {
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
            SET state = ?2,
                updated_at_ms = ?3,
                last_accessed_at_ms = ?3,
                active_leases = 0,
                validation_error = ?4
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
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
            SET state = ?2,
                updated_at_ms = ?3,
                last_accessed_at_ms = ?3,
                active_leases = 0
            WHERE id = ?1
              AND active_leases = 0
              AND state IN ('reserved', 'staging', 'ready', 'failed', 'expired')
            "#,
        )
        .bind(id.to_string())
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
            SET state = ?2,
                updated_at_ms = ?3,
                last_accessed_at_ms = ?3,
                active_leases = 0,
                expires_at_ms = NULL
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
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
                state = ?2,
                updated_at_ms = ?3,
                last_accessed_at_ms = ?3
            WHERE id = ?1
              AND state IN ('ready', 'leased')
              AND active_leases >= 0
            "#,
        )
        .bind(id.to_string())
        .bind(StagingState::Leased.as_str())
        .bind(leased_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(TaruError::Conflict {
                message: format!("staging manifest {id} is not ready to lease"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
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
                    WHEN active_leases - 1 = 0 THEN ?2
                    ELSE ?3
                END,
                updated_at_ms = ?4,
                last_accessed_at_ms = ?4
            WHERE id = ?1
              AND state = ?3
              AND active_leases > 0
            "#,
        )
        .bind(id.to_string())
        .bind(StagingState::Ready.as_str())
        .bind(StagingState::Leased.as_str())
        .bind(released_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(TaruError::Conflict {
                message: format!("staging manifest {id} has no active lease to release"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn get_staging_manifest_record(
        &self,
        id: StagingManifestId,
    ) -> Result<Option<StagingManifestRecord>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM staging_manifest_records
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_staging_manifest_record).transpose()
    }

    async fn find_staging_manifest_record_by_path(
        &self,
        local_path: &str,
    ) -> Result<Option<StagingManifestRecord>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM staging_manifest_records
            WHERE local_path = ?1
            "#,
        )
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
        let purpose = purpose.map(StagingPurpose::as_str);
        let state = state.map(StagingState::as_str);
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM staging_manifest_records
            WHERE (?1 IS NULL OR purpose = ?1)
              AND (?2 IS NULL OR state = ?2)
            ORDER BY last_accessed_at_ms ASC, id ASC
            LIMIT ?3 OFFSET ?4
            "#,
        )
        .bind(purpose)
        .bind(state)
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
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM staging_manifest_records
            WHERE state IN ('reserved', 'staging', 'ready', 'failed', 'expired')
              AND active_leases = 0
              AND expires_at_ms IS NOT NULL
              AND expires_at_ms <= ?1
            ORDER BY last_accessed_at_ms ASC, id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
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
            SET last_accessed_at_ms = ?2,
                updated_at_ms = ?2
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(accessed_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn delete_staging_manifest_record(&self, id: StagingManifestId) -> Result<()> {
        sqlx::query("DELETE FROM staging_manifest_records WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn sum_staging_manifest_bytes(&self) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(size_bytes), 0) AS total_bytes
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

fn row_to_staging_manifest_record(row: SqliteRow) -> Result<StagingManifestRecord> {
    Ok(StagingManifestRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
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

async fn upsert_staging_manifest_record_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    record: NewStagingManifestRecord,
) -> Result<StagingManifestRecord> {
    sqlx::query(
        r#"
        INSERT INTO staging_manifest_records (
            id, source_uri, source_scheme, purpose, local_path, size_bytes,
            etag, fingerprint, state, created_at_ms, updated_at_ms,
            last_accessed_at_ms, expires_at_ms, active_leases, validation_error
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(id) DO UPDATE SET
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
        "#,
    )
    .bind(record.id.to_string())
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

    get_staging_manifest_record_in_transaction(transaction, record.id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "staging_manifest_record",
            id: record.id.to_string(),
        })
}

async fn get_staging_manifest_record_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    id: StagingManifestId,
) -> Result<Option<StagingManifestRecord>> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM staging_manifest_records
        WHERE id = ?1
        "#,
    )
    .bind(id.to_string())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_staging_manifest_record).transpose()
}

async fn sum_staging_manifest_bytes_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<u64> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(size_bytes), 0) AS total_bytes
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

#[cfg(test)]
mod tests {
    use taru_core::{
        NewStagingManifestRecord, PageRequest, StagingManifestId, StagingManifestRepository,
        StagingPurpose, StagingState, TransactionManager,
    };

    use crate::SqliteStore;

    #[tokio::test]
    async fn sqlite_store_round_trips_staging_manifest_records() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let id = StagingManifestId::new();
        let record = NewStagingManifestRecord {
            id,
            source_uri: "webdav:///Movies/Demo.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::FfmpegInput,
            local_path: "F:/Taru/cache/remux/inputs/demo.mkv".to_owned(),
            size_bytes: Some(12),
            etag: Some("etag-demo".to_owned()),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: StagingState::Ready,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(1_500),
            active_leases: 0,
            validation_error: None,
        };

        let saved = store
            .upsert_staging_manifest_record(record.clone())
            .await
            .unwrap();
        assert_eq!(saved.id, id);
        assert_eq!(saved.size_bytes, Some(12));
        assert!(saved.is_cleanup_candidate_at(2_000));

        assert_eq!(
            store.get_staging_manifest_record(id).await.unwrap(),
            Some(saved.clone())
        );
        assert_eq!(
            store
                .find_staging_manifest_record_by_path(&record.local_path)
                .await
                .unwrap(),
            Some(saved.clone())
        );
        assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 12);

        let listed = store
            .list_staging_manifest_records(
                Some(StagingPurpose::FfmpegInput),
                Some(StagingState::Ready),
                PageRequest::first_page(),
            )
            .await
            .unwrap();
        assert_eq!(listed, vec![saved.clone()]);

        let cleanup = store
            .list_staging_cleanup_candidates(2_000, PageRequest::first_page())
            .await
            .unwrap();
        assert_eq!(cleanup, vec![saved.clone()]);

        let staging_id = StagingManifestId::new();
        let staging_record = NewStagingManifestRecord {
            id: staging_id,
            state: StagingState::Reserved,
            local_path: "F:/Taru/cache/remux/inputs/pending.mkv".to_owned(),
            last_accessed_at_ms: 1_300,
            ..record.clone()
        };
        let saved_reserved = store
            .upsert_staging_manifest_record(staging_record)
            .await
            .unwrap();

        let cleanup = store
            .list_staging_cleanup_candidates(2_000, PageRequest::first_page())
            .await
            .unwrap();
        assert_eq!(cleanup, vec![saved.clone(), saved_reserved.clone()]);

        let started = store
            .start_staging_manifest_record(staging_id, 2_100)
            .await
            .unwrap();
        assert_eq!(started.state, StagingState::Staging);
        assert_eq!(started.active_leases, 0);

        let touched = store
            .touch_staging_manifest_record(id, 2_500)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(touched.last_accessed_at_ms, 2_500);
        assert_eq!(touched.updated_at_ms, 2_500);

        store.delete_staging_manifest_record(id).await.unwrap();
        store
            .delete_staging_manifest_record(staging_id)
            .await
            .unwrap();
        assert!(
            store
                .get_staging_manifest_record(id)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn sqlite_store_does_not_expire_active_staging_lease() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let id = StagingManifestId::new();
        let record = NewStagingManifestRecord {
            id,
            source_uri: "webdav:///Movies/Demo.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::FfmpegInput,
            local_path: "F:/Taru/cache/remux/inputs/demo.mkv".to_owned(),
            size_bytes: Some(12),
            etag: Some("etag-demo".to_owned()),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: StagingState::Ready,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(1_500),
            active_leases: 0,
            validation_error: None,
        };

        store.upsert_staging_manifest_record(record).await.unwrap();
        let leased = store
            .acquire_staging_manifest_lease(id, 1_300)
            .await
            .unwrap();
        assert_eq!(leased.state, StagingState::Leased);
        assert_eq!(leased.active_leases, 1);

        let after_expire_attempt = store
            .expire_staging_manifest_record(id, 2_000)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(after_expire_attempt.state, StagingState::Leased);
        assert_eq!(after_expire_attempt.active_leases, 1);
        store
            .release_staging_manifest_lease(id, 2_100)
            .await
            .unwrap();
    }
}
