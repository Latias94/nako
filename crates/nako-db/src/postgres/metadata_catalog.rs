use sqlx::{Postgres, postgres::PgRow};

use nako_core::*;

use super::addons_automation::set_addon_side_effect_apply_outcome_tx;
use super::core_catalog::{
    upsert_library_item_state_tx, upsert_media_item_tx, upsert_search_projection_tx,
};
use super::{
    PostgresStore, credit_role_from_parts, credit_role_to_parts, database_error,
    image_kind_from_parts, image_kind_to_parts, image_owner_from_parts, image_owner_to_parts,
    metadata_field_from_str, metadata_source_from_parts, metadata_source_to_parts,
    optional_i64_to_i32, optional_i64_to_u16, optional_i64_to_u32, parse_id, provider_from_parts,
    provider_subject_kind_from_parts, provider_subject_kind_to_parts, provider_to_parts, row_get,
    u32_to_i64, u64_to_i64,
};

const PROVIDER_SUBJECT_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            FROM provider_subjects
            WHERE id = $1
            "#;

#[async_trait::async_trait]
impl ProviderMappingRepository for PostgresStore {
    async fn upsert_provider_subject(&self, subject: &ProviderSubject) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_provider_subject_tx(&mut transaction, subject).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_provider_subject(&self, id: ProviderSubjectId) -> Result<Option<ProviderSubject>> {
        let row = sqlx::query(PROVIDER_SUBJECT_SELECT_BY_ID)
            .bind(id.as_uuid())
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
                id::text AS id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            FROM provider_subjects
            WHERE provider = $1
              AND provider_key = $2
              AND subject_kind = $3
              AND subject_kind_key = $4
              AND subject_key = $5
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
                provider_subjects.id::text AS id,
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
            WHERE provider_mappings.item_id = $1
            ORDER BY
                provider_subjects.provider ASC,
                provider_subjects.provider_key ASC,
                provider_subjects.subject_kind ASC,
                provider_subjects.subject_kind_key ASC,
                provider_subjects.subject_key ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_subject).collect()
    }

    async fn upsert_provider_mapping(&self, mapping: &ProviderMapping) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_provider_mapping_tx(&mut transaction, mapping).await?;
        transaction.commit().await.map_err(database_error)
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
                provider_mappings.id::text AS id,
                provider_mappings.item_id::text AS item_id,
                provider_mappings.subject_id::text AS subject_id,
                provider_mappings.status,
                provider_mappings.confidence_milli,
                provider_mappings.source,
                provider_mappings.source_key
            FROM provider_mappings
            INNER JOIN provider_subjects
                ON provider_subjects.id = provider_mappings.subject_id
            WHERE provider_mappings.item_id = $1
            ORDER BY
                provider_subjects.provider ASC,
                provider_subjects.provider_key ASC,
                provider_subjects.subject_kind ASC,
                provider_subjects.subject_kind_key ASC,
                provider_subjects.subject_key ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_mapping).collect()
    }
}

#[async_trait::async_trait]
impl MetadataRepository for PostgresStore {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_field_lock_tx(&mut transaction, lock).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, field, locked, source, source_key
            FROM metadata_field_locks
            WHERE item_id = $1
            ORDER BY field ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_metadata_field_lock).collect()
    }

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_provider_raw_response_tx(&mut transaction, response).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn commit_metadata_refresh(
        &self,
        commit: &MetadataRefreshPersistenceCommit,
    ) -> Result<MetadataRefreshPersistenceSummary> {
        if commit.raw_response.item_id != commit.item.id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "metadata refresh raw response item_id {} does not match item {}",
                    commit.raw_response.item_id, commit.item.id
                ),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_media_item_tx(&mut transaction, &commit.item).await?;
        upsert_provider_raw_response_tx(&mut transaction, &commit.raw_response).await?;
        upsert_provider_subject_tx(&mut transaction, &commit.provider_mapping.subject).await?;

        let mapping_id = commit
            .provider_mapping
            .id
            .unwrap_or_else(ProviderMappingId::new);
        let mapping = ProviderMapping {
            id: mapping_id,
            item_id: commit.item.id,
            subject_id: commit.provider_mapping.subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: commit.provider_mapping.confidence_milli,
            source: commit.provider_mapping.source.clone(),
        };
        upsert_provider_mapping_tx(&mut transaction, &mapping).await?;

        let confirmed_libraries = library_ids_for_item_tx(&mut transaction, commit.item.id).await?;
        for library_id in &confirmed_libraries {
            upsert_library_item_state_tx(
                &mut transaction,
                &LibraryItemState {
                    library_id: *library_id,
                    item_id: commit.item.id,
                    provisional: false,
                },
            )
            .await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(MetadataRefreshPersistenceSummary {
            item_id: commit.item.id,
            provider_subject_id: commit.provider_mapping.subject.id,
            provider_mapping_id: mapping_id,
            confirmed_libraries,
        })
    }

    async fn commit_nfo_import(
        &self,
        commit: &NfoImportPersistenceCommit,
    ) -> Result<NfoImportPersistenceSummary> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        for item in &commit.items {
            upsert_media_item_tx(&mut transaction, item).await?;
        }
        for lock in &commit.field_locks {
            upsert_field_lock_tx(&mut transaction, lock).await?;
        }
        for state in &commit.library_item_states {
            upsert_library_item_state_tx(&mut transaction, state).await?;
        }
        for projection in &commit.catalog_projections {
            replace_item_catalog_graph_tx(
                &mut transaction,
                projection.search.item_id,
                &projection.graph,
            )
            .await?;
            upsert_search_projection_tx(&mut transaction, &projection.search).await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(NfoImportPersistenceSummary {
            item_ids: commit.items.iter().map(|item| item.id).collect(),
            locked_fields: commit.field_locks.len() as u64,
            confirmed_items: commit.library_item_states.len() as u64,
            projected_items: commit.catalog_projections.len() as u64,
        })
    }

    async fn commit_addon_metadata_write(
        &self,
        commit: &AddonMetadataWritePersistenceCommit,
    ) -> Result<AddonMetadataWritePersistenceSummary> {
        if commit.catalog.search.item_id != commit.item.id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "addon metadata write search projection item_id {} does not match item {}",
                    commit.catalog.search.item_id, commit.item.id
                ),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_media_item_tx(&mut transaction, &commit.item).await?;
        if let Some(graph) = &commit.catalog.graph {
            replace_item_catalog_graph_tx(&mut transaction, commit.item.id, graph).await?;
        }
        upsert_search_projection_tx(&mut transaction, &commit.catalog.search).await?;
        let side_effect = set_addon_side_effect_apply_outcome_tx(
            &mut transaction,
            commit.side_effect_id,
            &AddonSideEffectApplyOutcome {
                status: AddonSideEffectApplyStatus::Applied,
                error_code: None,
                item_id: Some(commit.item.id),
                source: Some(commit.applied_source.clone()),
                report_json: commit.apply_report_json.clone(),
            },
        )
        .await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(AddonMetadataWritePersistenceSummary {
            item_id: commit.item.id,
            projected_items: 1,
            side_effect,
        })
    }

    async fn commit_metadata_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_media_item_tx(&mut transaction, item).await?;
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
            SELECT item_id::text AS item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = $1 AND provider = $2 AND provider_key = $3
            LIMIT 1
            "#,
        )
        .bind(item_id.as_uuid())
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
            SELECT item_id::text AS item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = $1
              AND ($2::text IS NULL OR provider = $2)
            ORDER BY fetched_at DESC, provider ASC, provider_key ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(provider.as_deref())
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
            WHERE fetched_at < $1
              AND ($2::text IS NULL OR provider = $2)
            "#,
        )
        .bind(fetched_before)
        .bind(provider.as_deref())
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(attempt.id.as_uuid())
        .bind(attempt.job_id.as_uuid())
        .bind(attempt.item_id.as_uuid())
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
                id::text AS id,
                job_id::text AS job_id,
                item_id::text AS item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            FROM metadata_provider_attempts
            WHERE job_id = $1
            ORDER BY started_at ASC, created_at ASC
            "#,
        )
        .bind(job_id.as_uuid())
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
                id::text AS id,
                job_id::text AS job_id,
                item_id::text AS item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            FROM metadata_provider_attempts
            WHERE item_id = $1
              AND ($2::text IS NULL OR provider = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY started_at DESC, created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(provider.as_deref())
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

#[async_trait::async_trait]
impl CatalogRepository for PostgresStore {
    async fn replace_item_catalog_graph(
        &self,
        item_id: MediaItemId,
        replacement: &CatalogItemGraphReplacement,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        replace_item_catalog_graph_tx(&mut transaction, item_id, replacement).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn commit_item_projection(&self, commit: &CatalogItemProjectionCommit) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        replace_item_catalog_graph_tx(&mut transaction, commit.search.item_id, &commit.graph)
            .await?;
        upsert_search_projection_tx(&mut transaction, &commit.search).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn upsert_search_projection(&self, projection: &CatalogSearchProjection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_search_projection_tx(&mut transaction, projection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn upsert_person(&self, person: &Person) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_person_tx(&mut transaction, person).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>> {
        let row = sqlx::query(
            "SELECT id::text AS id, name, sort_name, overview FROM people WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let external_ids = self
            .list_catalog_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Person>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT p.id::text AS id, p.name, p.sort_name, p.overview
            FROM people p
            JOIN person_external_ids e ON e.person_id = p.id
            WHERE e.provider = $1 AND e.provider_key = $2 AND e.value = $3
            ORDER BY p.name ASC, p.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_catalog_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>> {
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, sort_name, overview
            FROM people
            WHERE name = $1
            ORDER BY name ASC, id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_catalog_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, sort_name, overview
            FROM people
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut people = Vec::with_capacity(rows.len());
        for row in rows {
            let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_catalog_external_ids("person_external_ids", "person_id", id)
                .await?;
            people.push(row_to_person(row, external_ids)?);
        }

        Ok(people)
    }

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_credit_tx(&mut transaction, credit).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_credits WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                person_id::text AS person_id,
                role,
                role_key,
                character,
                sort_order
            FROM item_credits
            WHERE item_id = $1
            ORDER BY COALESCE(sort_order, 2147483647), role ASC, person_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                person_id::text AS person_id,
                role,
                role_key,
                character,
                sort_order
            FROM item_credits
            WHERE person_id = $1
            ORDER BY role ASC, COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(person_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                mi.id::text AS id,
                mi.kind,
                mi.parent_id::text AS parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json::text AS metadata_json
            FROM media_items mi
            WHERE mi.id IN (
                SELECT item_id FROM item_credits WHERE person_id = $1
            )
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(person_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_genre(&self, genre: &Genre) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_genre_tx(&mut transaction, genre).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>> {
        let row = sqlx::query(
            "SELECT id::text AS id, name, source, source_key FROM genres WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_genre).transpose()
    }

    async fn find_genre_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Genre>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM genres
            WHERE name = $1 AND source = $2 AND source_key = $3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_genre).transpose()
    }

    async fn list_genres(&self, page: PageRequest) -> Result<Vec<Genre>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM genres
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_genre).collect()
    }

    async fn upsert_item_genre(&self, item_genre: &ItemGenre) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_genre_tx(&mut transaction, item_genre).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_genres WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
        let rows = sqlx::query(
            "SELECT item_id::text AS item_id, genre_id::text AS genre_id FROM item_genres WHERE item_id = $1 ORDER BY genre_id ASC",
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_genre).collect()
    }

    async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                mi.id::text AS id,
                mi.kind,
                mi.parent_id::text AS parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json::text AS metadata_json
            FROM media_items mi
            WHERE mi.id IN (
                SELECT item_id FROM item_genres WHERE genre_id = $1
            )
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(genre_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_tag(&self, tag: &Tag) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_tag_tx(&mut transaction, tag).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>> {
        let row =
            sqlx::query("SELECT id::text AS id, name, source, source_key FROM tags WHERE id = $1")
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(database_error)?;

        row.map(row_to_tag).transpose()
    }

    async fn find_tag_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Tag>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM tags
            WHERE name = $1 AND source = $2 AND source_key = $3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_tag).transpose()
    }

    async fn list_tags(&self, page: PageRequest) -> Result<Vec<Tag>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM tags
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_tag).collect()
    }

    async fn upsert_item_tag(&self, item_tag: &ItemTag) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_tag_tx(&mut transaction, item_tag).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_tags WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, tag_id::text AS tag_id
            FROM item_tags
            WHERE item_id = $1
            ORDER BY tag_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_tag).collect()
    }

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                mi.id::text AS id,
                mi.kind,
                mi.parent_id::text AS parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json::text AS metadata_json
            FROM media_items mi
            WHERE mi.id IN (
                SELECT item_id FROM item_tags WHERE tag_id = $1
            )
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tag_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_collection(&self, collection: &Collection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_collection_tx(&mut transaction, collection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, overview, source, source_key
            FROM collections
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_catalog_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn find_collection_by_external_id(
        &self,
        external_id: &ExternalId,
    ) -> Result<Option<Collection>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT c.id::text AS id, c.name, c.overview, c.source, c.source_key
            FROM collections c
            JOIN collection_external_ids e ON e.collection_id = c.id
            WHERE e.provider = $1 AND e.provider_key = $2 AND e.value = $3
            ORDER BY c.name ASC, c.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_catalog_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn find_collection_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Collection>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, overview, source, source_key
            FROM collections
            WHERE name = $1 AND source = $2 AND source_key = $3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_catalog_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, overview, source, source_key
            FROM collections
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut collections = Vec::with_capacity(rows.len());
        for row in rows {
            let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_catalog_external_ids("collection_external_ids", "collection_id", id)
                .await?;
            collections.push(row_to_collection(row, external_ids)?);
        }

        Ok(collections)
    }

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_collection_item_tx(&mut transaction, item).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM collection_items WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id::text AS collection_id, item_id::text AS item_id, sort_order
            FROM collection_items
            WHERE item_id = $1
            ORDER BY COALESCE(sort_order, 2147483647), collection_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn list_collection_items(
        &self,
        collection_id: CollectionId,
    ) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id::text AS collection_id, item_id::text AS item_id, sort_order
            FROM collection_items
            WHERE collection_id = $1
            ORDER BY COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(collection_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn upsert_studio(&self, studio: &Studio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_studio_tx(&mut transaction, studio).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>> {
        let row = sqlx::query(
            "SELECT id::text AS id, name, source, source_key FROM studios WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_catalog_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Studio>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT s.id::text AS id, s.name, s.source, s.source_key
            FROM studios s
            JOIN studio_external_ids e ON e.studio_id = s.id
            WHERE e.provider = $1 AND e.provider_key = $2 AND e.value = $3
            ORDER BY s.name ASC, s.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_catalog_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Studio>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM studios
            WHERE name = $1 AND source = $2 AND source_key = $3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_catalog_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM studios
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut studios = Vec::with_capacity(rows.len());
        for row in rows {
            let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_catalog_external_ids("studio_external_ids", "studio_id", id)
                .await?;
            studios.push(row_to_studio(row, external_ids)?);
        }

        Ok(studios)
    }

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_studio_tx(&mut transaction, item_studio).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_studios WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, studio_id::text AS studio_id
            FROM item_studios
            WHERE item_id = $1
            ORDER BY studio_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_studio).collect()
    }

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_image_asset_tx(&mut transaction, image).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                owner_kind,
                owner_id::text AS owner_id,
                kind,
                kind_key,
                source_uri,
                provider,
                provider_key,
                cache_uri,
                width,
                height,
                language,
                selected,
                content_hash,
                etag
            FROM image_assets
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_image_asset).transpose()
    }

    async fn find_image_asset_by_source(
        &self,
        owner: &ImageOwner,
        kind: &ImageKind,
        source_uri: &str,
    ) -> Result<Option<ImageAsset>> {
        let (owner_kind, owner_id) = image_owner_to_parts(owner);
        let (kind, kind_key) = image_kind_to_parts(kind);
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                owner_kind,
                owner_id::text AS owner_id,
                kind,
                kind_key,
                source_uri,
                provider,
                provider_key,
                cache_uri,
                width,
                height,
                language,
                selected,
                content_hash,
                etag
            FROM image_assets
            WHERE owner_kind = $1 AND owner_id = $2::uuid AND kind = $3
                AND kind_key = $4 AND source_uri = $5
            LIMIT 1
            "#,
        )
        .bind(owner_kind)
        .bind(owner_id)
        .bind(kind)
        .bind(kind_key)
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_image_asset).transpose()
    }

    async fn list_item_images(&self, item_id: MediaItemId) -> Result<Vec<ImageAsset>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                owner_kind,
                owner_id::text AS owner_id,
                kind,
                kind_key,
                source_uri,
                provider,
                provider_key,
                cache_uri,
                width,
                height,
                language,
                selected,
                content_hash,
                etag
            FROM image_assets
            WHERE owner_kind = 'item' AND owner_id = $1
            ORDER BY selected DESC, kind ASC, id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_image_asset).collect()
    }
}

async fn upsert_field_lock_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    lock: &MetadataFieldLock,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&lock.source);

    sqlx::query(
        r#"
        INSERT INTO metadata_field_locks (item_id, field, locked, source, source_key)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(item_id, field) DO UPDATE SET
            locked = excluded.locked,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(lock.item_id.as_uuid())
    .bind(lock.field.as_str())
    .bind(lock.locked)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_provider_raw_response_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
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
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(item_id, provider, provider_key) DO UPDATE SET
            body_json = excluded.body_json,
            fetched_at = excluded.fetched_at,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(response.item_id.as_uuid())
    .bind(provider)
    .bind(provider_key)
    .bind(&response.body_json)
    .bind(&response.fetched_at)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_provider_subject_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    subject: &ProviderSubject,
) -> Result<()> {
    let (provider, provider_key) = provider_to_parts(&subject.provider);
    let (subject_kind, subject_kind_key) = provider_subject_kind_to_parts(&subject.subject_kind);

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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT(id) DO UPDATE SET
            provider = excluded.provider,
            provider_key = excluded.provider_key,
            subject_kind = excluded.subject_kind,
            subject_kind_key = excluded.subject_kind_key,
            subject_key = excluded.subject_key,
            title = excluded.title,
            release_year = excluded.release_year,
            locale = excluded.locale,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(subject.id.as_uuid())
    .bind(provider)
    .bind(provider_key)
    .bind(subject_kind)
    .bind(subject_kind_key)
    .bind(&subject.subject_key)
    .bind(&subject.title)
    .bind(subject.release_year.map(i64::from))
    .bind(&subject.locale)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_provider_mapping_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    mapping: &ProviderMapping,
) -> Result<()> {
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
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT(id) DO UPDATE SET
            item_id = excluded.item_id,
            subject_id = excluded.subject_id,
            status = excluded.status,
            confidence_milli = excluded.confidence_milli,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(mapping.id.as_uuid())
    .bind(mapping.item_id.as_uuid())
    .bind(mapping.subject_id.as_uuid())
    .bind(mapping.status.as_str())
    .bind(mapping.confidence_milli.map(i64::from))
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn library_ids_for_item_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_id: MediaItemId,
) -> Result<Vec<LibraryId>> {
    let rows = sqlx::query(
        r#"
        SELECT library_id::text AS library_id
        FROM library_item_states
        WHERE item_id = $1
        ORDER BY library_id ASC
        "#,
    )
    .bind(item_id.as_uuid())
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(|row| parse_id(row_get::<String>(&row, "library_id")?))
        .collect()
}

async fn replace_item_catalog_graph_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_id: MediaItemId,
    replacement: &CatalogItemGraphReplacement,
) -> Result<()> {
    sqlx::query("DELETE FROM item_credits WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_genres WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_tags WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM collection_items WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_studios WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for person in &replacement.people {
        upsert_person_tx(transaction, person).await?;
    }
    for credit in &replacement.credits {
        upsert_item_credit_tx(transaction, credit).await?;
    }
    for genre in &replacement.genres {
        upsert_genre_tx(transaction, genre).await?;
    }
    for item_genre in &replacement.item_genres {
        upsert_item_genre_tx(transaction, item_genre).await?;
    }
    for tag in &replacement.tags {
        upsert_tag_tx(transaction, tag).await?;
    }
    for item_tag in &replacement.item_tags {
        upsert_item_tag_tx(transaction, item_tag).await?;
    }
    for collection in &replacement.collections {
        upsert_collection_tx(transaction, collection).await?;
    }
    for collection_item in &replacement.collection_items {
        upsert_collection_item_tx(transaction, collection_item).await?;
    }
    for studio in &replacement.studios {
        upsert_studio_tx(transaction, studio).await?;
    }
    for item_studio in &replacement.item_studios {
        upsert_item_studio_tx(transaction, item_studio).await?;
    }
    for image in &replacement.images {
        upsert_image_asset_tx(transaction, image).await?;
    }

    Ok(())
}

async fn upsert_person_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    person: &Person,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO people (id, name, sort_name, overview)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            sort_name = excluded.sort_name,
            overview = excluded.overview,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(person.id.as_uuid())
    .bind(&person.name)
    .bind(&person.sort_name)
    .bind(&person.overview)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM person_external_ids WHERE person_id = $1")
        .bind(person.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &person.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO person_external_ids (person_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(person_id, provider, provider_key, value) DO NOTHING
            "#,
        )
        .bind(person.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_item_credit_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    credit: &ItemCredit,
) -> Result<()> {
    let (role, role_key) = credit_role_to_parts(&credit.role);
    sqlx::query(
        r#"
        INSERT INTO item_credits (
            item_id, person_id, role, role_key, character, sort_order
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT(item_id, person_id, role, role_key, character) DO UPDATE SET
            sort_order = excluded.sort_order,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(credit.item_id.as_uuid())
    .bind(credit.person_id.as_uuid())
    .bind(role)
    .bind(role_key)
    .bind(credit.character.clone().unwrap_or_default())
    .bind(credit.sort_order.map(u32_to_i64))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_genre_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    genre: &Genre,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&genre.source);
    sqlx::query(
        r#"
        INSERT INTO genres (id, name, source, source_key)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(genre.id.as_uuid())
    .bind(&genre.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_item_genre_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_genre: &ItemGenre,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO item_genres (item_id, genre_id)
        VALUES ($1, $2)
        ON CONFLICT(item_id, genre_id) DO NOTHING
        "#,
    )
    .bind(item_genre.item_id.as_uuid())
    .bind(item_genre.genre_id.as_uuid())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_tag_tx(transaction: &mut sqlx::Transaction<'_, Postgres>, tag: &Tag) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&tag.source);
    sqlx::query(
        r#"
        INSERT INTO tags (id, name, source, source_key)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(tag.id.as_uuid())
    .bind(&tag.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_item_tag_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_tag: &ItemTag,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO item_tags (item_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT(item_id, tag_id) DO NOTHING
        "#,
    )
    .bind(item_tag.item_id.as_uuid())
    .bind(item_tag.tag_id.as_uuid())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_collection_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    collection: &Collection,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&collection.source);

    sqlx::query(
        r#"
        INSERT INTO collections (id, name, overview, source, source_key)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            overview = excluded.overview,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(collection.id.as_uuid())
    .bind(&collection.name)
    .bind(&collection.overview)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM collection_external_ids WHERE collection_id = $1")
        .bind(collection.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &collection.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO collection_external_ids (collection_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(collection_id, provider, provider_key, value) DO NOTHING
            "#,
        )
        .bind(collection.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_collection_item_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item: &CollectionItem,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO collection_items (collection_id, item_id, sort_order)
        VALUES ($1, $2, $3)
        ON CONFLICT(collection_id, item_id) DO UPDATE SET
            sort_order = excluded.sort_order
        "#,
    )
    .bind(item.collection_id.as_uuid())
    .bind(item.item_id.as_uuid())
    .bind(item.sort_order.map(u32_to_i64))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_studio_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    studio: &Studio,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&studio.source);

    sqlx::query(
        r#"
        INSERT INTO studios (id, name, source, source_key)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(studio.id.as_uuid())
    .bind(&studio.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM studio_external_ids WHERE studio_id = $1")
        .bind(studio.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &studio.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO studio_external_ids (studio_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(studio_id, provider, provider_key, value) DO NOTHING
            "#,
        )
        .bind(studio.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_item_studio_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_studio: &ItemStudio,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO item_studios (item_id, studio_id)
        VALUES ($1, $2)
        ON CONFLICT(item_id, studio_id) DO NOTHING
        "#,
    )
    .bind(item_studio.item_id.as_uuid())
    .bind(item_studio.studio_id.as_uuid())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_image_asset_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    image: &ImageAsset,
) -> Result<()> {
    let (owner_kind, owner_id) = image_owner_to_parts(&image.owner);
    let (kind, kind_key) = image_kind_to_parts(&image.kind);
    let (provider, provider_key) = provider_to_parts(&image.provider);

    sqlx::query(
        r#"
        INSERT INTO image_assets (
            id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
            provider_key, cache_uri, width, height, language, selected,
            content_hash, etag
        )
        VALUES ($1, $2, $3::uuid, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        ON CONFLICT(id) DO UPDATE SET
            owner_kind = excluded.owner_kind,
            owner_id = excluded.owner_id,
            kind = excluded.kind,
            kind_key = excluded.kind_key,
            source_uri = excluded.source_uri,
            provider = excluded.provider,
            provider_key = excluded.provider_key,
            cache_uri = excluded.cache_uri,
            width = excluded.width,
            height = excluded.height,
            language = excluded.language,
            selected = excluded.selected,
            content_hash = excluded.content_hash,
            etag = excluded.etag,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(image.id.as_uuid())
    .bind(owner_kind)
    .bind(owner_id)
    .bind(kind)
    .bind(kind_key)
    .bind(&image.source_uri)
    .bind(provider)
    .bind(provider_key)
    .bind(&image.cache_uri)
    .bind(image.width.map(u32_to_i64))
    .bind(image.height.map(u32_to_i64))
    .bind(&image.language)
    .bind(image.selected)
    .bind(&image.content_hash)
    .bind(&image.etag)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

fn row_to_metadata_field_lock(row: PgRow) -> Result<MetadataFieldLock> {
    Ok(MetadataFieldLock {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        field: metadata_field_from_str(&row_get::<String>(&row, "field")?)?,
        locked: row_get(&row, "locked")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_provider_raw_response(row: PgRow) -> Result<ProviderRawResponse> {
    Ok(ProviderRawResponse {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        provider_key: row_get(&row, "provider_key")?,
        body_json: row_get(&row, "body_json")?,
        fetched_at: row_get(&row, "fetched_at")?,
    })
}

fn row_to_metadata_provider_attempt(row: PgRow) -> Result<MetadataProviderAttemptRecord> {
    Ok(MetadataProviderAttemptRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, String::new()),
        provider_key: row_get(&row, "provider_key")?,
        status: MetadataProviderAttemptStatus::parse(&row_get::<String>(&row, "status")?)?,
        matched_by: row_get::<Option<String>>(&row, "matched_by")?
            .map(|value| MetadataMatchKind::parse(&value))
            .transpose()?,
        started_at: row_get(&row, "started_at")?,
        finished_at: row_get(&row, "finished_at")?,
        error_class: row_get::<Option<String>>(&row, "error_class")?
            .map(|value| MetadataProviderErrorClass::parse(&value))
            .transpose()?,
        message: row_get(&row, "message")?,
    })
}

fn row_to_provider_subject(row: PgRow) -> Result<ProviderSubject> {
    Ok(ProviderSubject {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        subject_kind: provider_subject_kind_from_parts(
            row_get(&row, "subject_kind")?,
            row_get(&row, "subject_kind_key")?,
        ),
        subject_key: row_get(&row, "subject_key")?,
        title: row_get(&row, "title")?,
        release_year: optional_i64_to_i32(row_get(&row, "release_year")?)?,
        locale: row_get(&row, "locale")?,
    })
}

fn row_to_provider_mapping(row: PgRow) -> Result<ProviderMapping> {
    Ok(ProviderMapping {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        subject_id: parse_id(row_get::<String>(&row, "subject_id")?)?,
        status: ProviderMappingStatus::parse(&row_get::<String>(&row, "status")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_person(row: PgRow, external_ids: Vec<ExternalId>) -> Result<Person> {
    Ok(Person {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        sort_name: row_get(&row, "sort_name")?,
        overview: row_get(&row, "overview")?,
        external_ids,
    })
}

fn row_to_item_credit(row: PgRow) -> Result<ItemCredit> {
    let character = row_get::<String>(&row, "character")?;
    Ok(ItemCredit {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        person_id: parse_id(row_get::<String>(&row, "person_id")?)?,
        role: credit_role_from_parts(row_get(&row, "role")?, row_get(&row, "role_key")?),
        character: (!character.is_empty()).then_some(character),
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

fn row_to_genre(row: PgRow) -> Result<Genre> {
    Ok(Genre {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_item_genre(row: PgRow) -> Result<ItemGenre> {
    Ok(ItemGenre {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        genre_id: parse_id(row_get::<String>(&row, "genre_id")?)?,
    })
}

fn row_to_tag(row: PgRow) -> Result<Tag> {
    Ok(Tag {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_item_tag(row: PgRow) -> Result<ItemTag> {
    Ok(ItemTag {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        tag_id: parse_id(row_get::<String>(&row, "tag_id")?)?,
    })
}

fn row_to_collection(row: PgRow, external_ids: Vec<ExternalId>) -> Result<Collection> {
    Ok(Collection {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        overview: row_get(&row, "overview")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

fn row_to_collection_item(row: PgRow) -> Result<CollectionItem> {
    Ok(CollectionItem {
        collection_id: parse_id(row_get::<String>(&row, "collection_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

fn row_to_studio(row: PgRow, external_ids: Vec<ExternalId>) -> Result<Studio> {
    Ok(Studio {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

fn row_to_item_studio(row: PgRow) -> Result<ItemStudio> {
    Ok(ItemStudio {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        studio_id: parse_id(row_get::<String>(&row, "studio_id")?)?,
    })
}

fn row_to_image_asset(row: PgRow) -> Result<ImageAsset> {
    Ok(ImageAsset {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        owner: image_owner_from_parts(row_get(&row, "owner_kind")?, row_get(&row, "owner_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        source_uri: row_get(&row, "source_uri")?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        cache_uri: row_get(&row, "cache_uri")?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        language: row_get(&row, "language")?,
        selected: row_get(&row, "selected")?,
        content_hash: row_get(&row, "content_hash")?,
        etag: row_get(&row, "etag")?,
    })
}
