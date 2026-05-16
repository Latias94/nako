use super::*;

#[async_trait::async_trait]
impl MetadataRepository for SqliteStore {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()> {
        let (source, source_key) = metadata_source_to_parts(&lock.source);

        sqlx::query(
            r#"
            INSERT INTO metadata_field_locks (item_id, field, locked, source, source_key)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(item_id, field) DO UPDATE SET
                locked = excluded.locked,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(lock.item_id.to_string())
        .bind(lock.field.as_str())
        .bind(bool_to_i64(lock.locked))
        .bind(source)
        .bind(source_key)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, field, locked, source, source_key
            FROM metadata_field_locks
            WHERE item_id = ?1
            ORDER BY field ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_metadata_field_lock).collect()
    }

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_provider_raw_response_in_transaction(&mut transaction, response).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn apply_metadata_refresh(
        &self,
        item: &MediaItem,
        raw_response: &ProviderRawResponse,
    ) -> Result<()> {
        if raw_response.item_id != item.id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "metadata refresh raw response item_id {} does not match item {}",
                    raw_response.item_id, item.id
                ),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        crate::media::upsert_media_item_in_transaction(&mut transaction, item).await?;
        upsert_provider_raw_response_in_transaction(&mut transaction, raw_response).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn get_provider_raw_response(
        &self,
        item_id: MediaItemId,
        provider: &ExternalProvider,
        provider_key: &str,
    ) -> Result<Option<ProviderRawResponse>> {
        let (provider, default_provider_key) = provider_to_parts(provider);
        let provider_key = if provider_key.is_empty() {
            default_provider_key
        } else {
            provider_key.to_owned()
        };
        let row = sqlx::query(
            r#"
            SELECT item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = ?1 AND provider = ?2 AND provider_key = ?3
            LIMIT 1
            "#,
        )
        .bind(item_id.to_string())
        .bind(provider)
        .bind(provider_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_provider_raw_response).transpose()
    }

    async fn list_provider_raw_responses(
        &self,
        item_id: MediaItemId,
        filter: ProviderRawResponseFilter,
        page: PageRequest,
    ) -> Result<Vec<ProviderRawResponse>> {
        let page = page.clamped();
        let provider = filter
            .provider
            .map(|provider| provider_to_parts(&provider).0);
        let rows = sqlx::query(
            r#"
            SELECT item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = ?1
              AND (?2 IS NULL OR provider = ?2)
            ORDER BY fetched_at DESC, provider ASC, provider_key ASC
            LIMIT ?3 OFFSET ?4
            "#,
        )
        .bind(item_id.to_string())
        .bind(provider)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_raw_response).collect()
    }

    async fn cleanup_provider_raw_responses(
        &self,
        filter: ProviderRawResponseFilter,
        fetched_before: &str,
    ) -> Result<ProviderRawResponseCleanup> {
        let provider = filter
            .provider
            .map(|provider| provider_to_parts(&provider).0);
        let deleted = sqlx::query(
            r#"
            DELETE FROM provider_raw_responses
            WHERE fetched_at < ?1
              AND (?2 IS NULL OR provider = ?2)
            "#,
        )
        .bind(fetched_before)
        .bind(&provider)
        .execute(&self.pool)
        .await
        .map_err(database_error)?
        .rows_affected();

        Ok(ProviderRawResponseCleanup {
            provider: provider.map(|provider| provider_from_parts(provider, String::new())),
            fetched_before: fetched_before.to_owned(),
            deleted,
        })
    }

    async fn insert_metadata_provider_attempt(
        &self,
        attempt: NewMetadataProviderAttempt,
    ) -> Result<()> {
        let (provider, _) = provider_to_parts(&attempt.provider);

        sqlx::query(
            r#"
            INSERT INTO metadata_provider_attempts (
                id,
                job_id,
                item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(attempt.id.to_string())
        .bind(attempt.job_id.to_string())
        .bind(attempt.item_id.to_string())
        .bind(provider)
        .bind(attempt.provider_key)
        .bind(attempt.status.as_str())
        .bind(attempt.matched_by.map(MetadataMatchKind::as_str))
        .bind(attempt.started_at)
        .bind(attempt.finished_at)
        .bind(attempt.error_class.map(MetadataProviderErrorClass::as_str))
        .bind(attempt.message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_metadata_provider_attempts(
        &self,
        job_id: JobId,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            FROM metadata_provider_attempts
            WHERE job_id = ?1
            ORDER BY started_at ASC, created_at ASC
            "#,
        )
        .bind(job_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_metadata_provider_attempt)
            .collect()
    }

    async fn list_metadata_provider_attempts_for_item(
        &self,
        item_id: MediaItemId,
        filter: MetadataAttemptFilter,
        page: PageRequest,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        let page = page.clamped();
        let provider = filter
            .provider
            .map(|provider| provider_to_parts(&provider).0);
        let status = filter.status.map(MetadataProviderAttemptStatus::as_str);
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            FROM metadata_provider_attempts
            WHERE item_id = ?1
              AND (?2 IS NULL OR provider = ?2)
              AND (?3 IS NULL OR status = ?3)
            ORDER BY started_at DESC, created_at DESC
            LIMIT ?4 OFFSET ?5
            "#,
        )
        .bind(item_id.to_string())
        .bind(provider)
        .bind(status)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_metadata_provider_attempt)
            .collect()
    }
}

async fn upsert_provider_raw_response_in_transaction(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    response: &ProviderRawResponse,
) -> Result<()> {
    let (provider, provider_key) = provider_to_parts(&response.provider);
    let provider_key = if response.provider_key.is_empty() {
        provider_key
    } else {
        response.provider_key.clone()
    };

    sqlx::query(
        r#"
            INSERT INTO provider_raw_responses (
                item_id,
                provider,
                provider_key,
                body_json,
                fetched_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(item_id, provider, provider_key) DO UPDATE SET
                body_json = excluded.body_json,
                fetched_at = excluded.fetched_at,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
    )
    .bind(response.item_id.to_string())
    .bind(provider)
    .bind(provider_key)
    .bind(&response.body_json)
    .bind(&response.fetched_at)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use taru_core::{
        CanonicalMetadata, ExternalProvider, MediaItem, MediaItemId, MediaKind, MediaRepository,
        MetadataRepository, ProviderRawResponse, TransactionManager,
    };

    use crate::SqliteStore;

    #[tokio::test]
    async fn apply_metadata_refresh_updates_item_and_raw_response() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "Updated");
        let raw = raw_response(item_id, "tmdb-1");

        store.upsert_media_item(&original).await.unwrap();
        store.apply_metadata_refresh(&updated, &raw).await.unwrap();

        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(updated));
        assert_eq!(
            store
                .get_provider_raw_response(item_id, &ExternalProvider::Tmdb, "tmdb-1")
                .await
                .unwrap(),
            Some(raw)
        );
    }

    #[tokio::test]
    async fn apply_metadata_refresh_rejects_mismatched_raw_response_item() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "Updated");
        let mismatched_raw = raw_response(MediaItemId::new(), "tmdb-1");

        store.upsert_media_item(&original).await.unwrap();
        let err = store
            .apply_metadata_refresh(&updated, &mismatched_raw)
            .await
            .unwrap_err();

        assert!(err.to_string().contains("does not match"));
        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(original));
        assert_eq!(
            store
                .get_provider_raw_response(item_id, &ExternalProvider::Tmdb, "tmdb-1")
                .await
                .unwrap(),
            None
        );
    }

    fn media_item(id: MediaItemId, title: &str) -> MediaItem {
        MediaItem {
            id,
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: title.to_owned(),
                ..CanonicalMetadata::default()
            },
        }
    }

    fn raw_response(item_id: MediaItemId, provider_key: &str) -> ProviderRawResponse {
        ProviderRawResponse {
            item_id,
            provider: ExternalProvider::Tmdb,
            provider_key: provider_key.to_owned(),
            fetched_at: "2026-05-16T00:00:00Z".to_owned(),
            body_json: r#"{"title":"Updated"}"#.to_owned(),
        }
    }
}
