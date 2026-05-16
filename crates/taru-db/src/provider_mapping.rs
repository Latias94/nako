use super::*;

#[async_trait::async_trait]
impl ProviderMappingRepository for SqliteStore {
    async fn upsert_provider_subject(&self, subject: &ProviderSubject) -> Result<()> {
        let (provider, provider_key) = provider_to_parts(&subject.provider);
        let (subject_kind, subject_kind_key) =
            provider_subject_kind_to_parts(&subject.subject_kind);

        sqlx::query(
            r#"
            INSERT INTO provider_subjects (
                id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                provider = excluded.provider,
                provider_key = excluded.provider_key,
                subject_kind = excluded.subject_kind,
                subject_kind_key = excluded.subject_kind_key,
                subject_key = excluded.subject_key,
                title = excluded.title,
                release_year = excluded.release_year,
                locale = excluded.locale,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(subject.id.to_string())
        .bind(provider)
        .bind(provider_key)
        .bind(subject_kind)
        .bind(subject_kind_key)
        .bind(&subject.subject_key)
        .bind(&subject.title)
        .bind(optional_i32_to_i64(subject.release_year))
        .bind(&subject.locale)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_provider_subject(&self, id: ProviderSubjectId) -> Result<Option<ProviderSubject>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            FROM provider_subjects
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_provider_subject).transpose()
    }

    async fn find_provider_subject(
        &self,
        provider: &ExternalProvider,
        subject_kind: &ProviderSubjectKind,
        subject_key: &str,
    ) -> Result<Option<ProviderSubject>> {
        let (provider, provider_key) = provider_to_parts(provider);
        let (subject_kind, subject_kind_key) = provider_subject_kind_to_parts(subject_kind);
        let row = sqlx::query(
            r#"
            SELECT
                id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            FROM provider_subjects
            WHERE provider = ?1
              AND provider_key = ?2
              AND subject_kind = ?3
              AND subject_kind_key = ?4
              AND subject_key = ?5
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(subject_kind)
        .bind(subject_kind_key)
        .bind(subject_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_provider_subject).transpose()
    }

    async fn list_provider_subjects_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderSubject>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                provider_subjects.id,
                provider_subjects.provider,
                provider_subjects.provider_key,
                provider_subjects.subject_kind,
                provider_subjects.subject_kind_key,
                provider_subjects.subject_key,
                provider_subjects.title,
                provider_subjects.release_year,
                provider_subjects.locale
            FROM provider_subjects
            INNER JOIN provider_mappings
                ON provider_mappings.subject_id = provider_subjects.id
            WHERE provider_mappings.item_id = ?1
            ORDER BY
                provider_subjects.provider ASC,
                provider_subjects.provider_key ASC,
                provider_subjects.subject_kind ASC,
                provider_subjects.subject_kind_key ASC,
                provider_subjects.subject_key ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_subject).collect()
    }

    async fn upsert_provider_mapping(&self, mapping: &ProviderMapping) -> Result<()> {
        let (source, source_key) = metadata_source_to_parts(&mapping.source);

        sqlx::query(
            r#"
            INSERT INTO provider_mappings (
                id,
                item_id,
                subject_id,
                status,
                confidence_milli,
                source,
                source_key
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                item_id = excluded.item_id,
                subject_id = excluded.subject_id,
                status = excluded.status,
                confidence_milli = excluded.confidence_milli,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(mapping.id.to_string())
        .bind(mapping.item_id.to_string())
        .bind(mapping.subject_id.to_string())
        .bind(mapping.status.as_str())
        .bind(optional_u16_to_i64(mapping.confidence_milli))
        .bind(source)
        .bind(source_key)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_provider_mappings_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderMapping>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                provider_mappings.id,
                provider_mappings.item_id,
                provider_mappings.subject_id,
                provider_mappings.status,
                provider_mappings.confidence_milli,
                provider_mappings.source,
                provider_mappings.source_key
            FROM provider_mappings
            INNER JOIN provider_subjects
                ON provider_subjects.id = provider_mappings.subject_id
            WHERE provider_mappings.item_id = ?1
            ORDER BY
                provider_subjects.provider ASC,
                provider_subjects.provider_key ASC,
                provider_subjects.subject_kind ASC,
                provider_subjects.subject_kind_key ASC,
                provider_subjects.subject_key ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_mapping).collect()
    }
}
