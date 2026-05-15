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
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
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
}
