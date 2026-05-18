use super::*;

#[async_trait::async_trait]
impl MetadataRepository for SqliteStore {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_field_lock_tx(&mut transaction, lock).await?;

        transaction.commit().await.map_err(database_error)
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

    async fn commit_metadata_refresh(
        &self,
        commit: &MetadataRefreshPersistenceCommit,
    ) -> Result<MetadataRefreshPersistenceSummary> {
        if commit.raw_response.item_id != commit.item.id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "metadata refresh raw response item_id {} does not match item {}",
                    commit.raw_response.item_id, commit.item.id
                ),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        crate::media::upsert_media_item_in_transaction(&mut transaction, &commit.item).await?;
        upsert_provider_raw_response_in_transaction(&mut transaction, &commit.raw_response).await?;
        crate::provider_mapping::upsert_provider_subject_tx(
            &mut transaction,
            &commit.provider_mapping.subject,
        )
        .await?;

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
        crate::provider_mapping::upsert_provider_mapping_tx(&mut transaction, &mapping).await?;

        let confirmed_libraries = library_ids_for_item_tx(&mut transaction, commit.item.id).await?;
        for library_id in &confirmed_libraries {
            crate::library_item::upsert_library_item_state_tx(
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
            crate::media::upsert_media_item_in_transaction(&mut transaction, item).await?;
        }
        for lock in &commit.field_locks {
            upsert_field_lock_tx(&mut transaction, lock).await?;
        }
        for state in &commit.library_item_states {
            crate::library_item::upsert_library_item_state_tx(&mut transaction, state).await?;
        }
        for projection in &commit.catalog_projections {
            crate::catalog::replace_item_catalog_graph_tx(
                &mut transaction,
                projection.search.item_id,
                &projection.graph,
            )
            .await?;
            crate::catalog::upsert_search_projection_tx(&mut transaction, &projection.search)
                .await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(NfoImportPersistenceSummary {
            item_ids: commit.items.iter().map(|item| item.id).collect(),
            locked_fields: commit.field_locks.len() as u64,
            confirmed_items: commit.library_item_states.len() as u64,
            projected_items: commit.catalog_projections.len() as u64,
        })
    }

    async fn commit_metadata_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        crate::media::upsert_media_item_in_transaction(&mut transaction, item).await?;
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

async fn upsert_field_lock_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    lock: &MetadataFieldLock,
) -> Result<()> {
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
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
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

async fn library_ids_for_item_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    item_id: MediaItemId,
) -> Result<Vec<LibraryId>> {
    let rows = sqlx::query(
        r#"
            SELECT library_id
            FROM library_item_states
            WHERE item_id = ?1
            ORDER BY library_id ASC
            "#,
    )
    .bind(item_id.to_string())
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(|row| parse_id(row_get::<String>(&row, "library_id")?))
        .collect()
}

#[cfg(test)]
mod tests {
    use taru_core::{
        CanonicalMetadata, CatalogItemGraphReplacement, CatalogItemProjectionCommit,
        CatalogRepository, CatalogSearchProjection, ExternalProvider, Genre, GenreId, ItemGenre,
        Library, LibraryId, LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryPreset,
        LibraryRepository, MediaItem, MediaItemId, MediaKind, MediaRepository, MetadataField,
        MetadataFieldLock, MetadataRefreshPersistenceCommit, MetadataRefreshProviderMappingCommit,
        MetadataRepository, MetadataSource, NfoImportPersistenceCommit, ProviderMappingRepository,
        ProviderRawResponse, ProviderSubject, ProviderSubjectId, ProviderSubjectKind,
        TransactionManager,
    };

    use crate::SqliteStore;

    #[tokio::test]
    async fn commit_metadata_refresh_rejects_mismatched_raw_response_item() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "Updated");
        let mismatched_raw = raw_response(MediaItemId::new(), "tmdb-1");
        let subject = provider_subject(ProviderSubjectId::new(), "tmdb-1");

        store.upsert_media_item(&original).await.unwrap();
        let err = store
            .commit_metadata_refresh(&MetadataRefreshPersistenceCommit {
                item: updated,
                raw_response: mismatched_raw,
                provider_mapping: MetadataRefreshProviderMappingCommit {
                    id: None,
                    subject,
                    confidence_milli: Some(1_000),
                    source: MetadataSource::Provider(ExternalProvider::Tmdb),
                },
            })
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

    #[tokio::test]
    async fn commit_metadata_refresh_updates_metadata_provider_mapping_and_library_state() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let library = library(LibraryId::new());
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "Updated");
        let raw = raw_response(item_id, "tmdb-1");
        let subject = provider_subject(ProviderSubjectId::new(), "tmdb-1");

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&original).await.unwrap();
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: true,
            })
            .await
            .unwrap();

        let summary = store
            .commit_metadata_refresh(&MetadataRefreshPersistenceCommit {
                item: updated.clone(),
                raw_response: raw.clone(),
                provider_mapping: MetadataRefreshProviderMappingCommit {
                    id: None,
                    subject: subject.clone(),
                    confidence_milli: Some(1_000),
                    source: MetadataSource::Provider(ExternalProvider::Tmdb),
                },
            })
            .await
            .unwrap();

        assert_eq!(summary.item_id, item_id);
        assert_eq!(summary.provider_subject_id, subject.id);
        assert_eq!(summary.confirmed_libraries, vec![library.id]);
        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(updated));
        assert_eq!(
            store
                .get_provider_raw_response(item_id, &ExternalProvider::Tmdb, "tmdb-1")
                .await
                .unwrap(),
            Some(raw)
        );
        assert_eq!(
            store
                .find_provider_subject(
                    &ExternalProvider::Tmdb,
                    &ProviderSubjectKind::Movie,
                    "tmdb-1"
                )
                .await
                .unwrap(),
            Some(subject.clone())
        );
        let mappings = store
            .list_provider_mappings_for_item(item_id, taru_core::PageRequest::first_page())
            .await
            .unwrap();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].id, summary.provider_mapping_id);
        assert_eq!(mappings[0].item_id, item_id);
        assert_eq!(mappings[0].subject_id, subject.id);
        assert_eq!(
            store
                .get_library_item_state(library.id, item_id)
                .await
                .unwrap(),
            Some(LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: false,
            })
        );
    }

    #[tokio::test]
    async fn commit_metadata_refresh_rolls_back_when_provider_subject_write_fails() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let library = library(LibraryId::new());
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "Updated");
        let raw = raw_response(item_id, "tmdb-1");
        let existing_subject = provider_subject(ProviderSubjectId::new(), "tmdb-1");
        let conflicting_subject = provider_subject(ProviderSubjectId::new(), "tmdb-1");

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&original).await.unwrap();
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: true,
            })
            .await
            .unwrap();
        store
            .upsert_provider_subject(&existing_subject)
            .await
            .unwrap();

        let err = store
            .commit_metadata_refresh(&MetadataRefreshPersistenceCommit {
                item: updated,
                raw_response: raw.clone(),
                provider_mapping: MetadataRefreshProviderMappingCommit {
                    id: None,
                    subject: conflicting_subject,
                    confidence_milli: Some(1_000),
                    source: MetadataSource::Provider(ExternalProvider::Tmdb),
                },
            })
            .await
            .unwrap_err();

        assert!(!err.to_string().is_empty());
        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(original));
        assert_eq!(
            store
                .get_provider_raw_response(item_id, &ExternalProvider::Tmdb, "tmdb-1")
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .find_provider_subject(
                    &ExternalProvider::Tmdb,
                    &ProviderSubjectKind::Movie,
                    "tmdb-1"
                )
                .await
                .unwrap(),
            Some(existing_subject)
        );
        assert_eq!(
            store
                .list_provider_mappings_for_item(item_id, taru_core::PageRequest::first_page())
                .await
                .unwrap(),
            Vec::new()
        );
        assert_eq!(
            store
                .get_library_item_state(library.id, item_id)
                .await
                .unwrap(),
            Some(LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: true,
            })
        );
    }

    #[tokio::test]
    async fn commit_nfo_import_persists_item_locks_library_state_and_projection() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let library = library(LibraryId::new());
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "NFO Title");
        let genre = Genre {
            id: GenreId::new(),
            name: "Action".to_owned(),
            source: MetadataSource::Nfo,
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&original).await.unwrap();

        let summary = store
            .commit_nfo_import(&NfoImportPersistenceCommit {
                items: vec![updated.clone()],
                field_locks: vec![MetadataFieldLock {
                    item_id,
                    field: MetadataField::Title,
                    locked: true,
                    source: MetadataSource::Nfo,
                }],
                library_item_states: vec![LibraryItemState {
                    library_id: library.id,
                    item_id,
                    provisional: false,
                }],
                catalog_projections: vec![CatalogItemProjectionCommit {
                    graph: CatalogItemGraphReplacement {
                        genres: vec![genre.clone()],
                        item_genres: vec![ItemGenre {
                            item_id,
                            genre_id: genre.id,
                        }],
                        ..CatalogItemGraphReplacement::default()
                    },
                    search: CatalogSearchProjection {
                        item_id,
                        title: "NFO Title".to_owned(),
                        body: "NFO Title Action".to_owned(),
                        facets: vec!["genre:Action".to_owned(), "kind:movie".to_owned()],
                    },
                }],
            })
            .await
            .unwrap();

        assert_eq!(summary.item_ids, vec![item_id]);
        assert_eq!(summary.locked_fields, 1);
        assert_eq!(summary.confirmed_items, 1);
        assert_eq!(summary.projected_items, 1);
        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(updated));
        assert_eq!(
            store.list_field_locks(item_id).await.unwrap(),
            vec![MetadataFieldLock {
                item_id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::Nfo,
            }]
        );
        assert_eq!(
            store
                .get_library_item_state(library.id, item_id)
                .await
                .unwrap(),
            Some(LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: false,
            })
        );
        assert_eq!(
            store
                .list_genres(taru_core::PageRequest::first_page())
                .await
                .unwrap(),
            vec![genre]
        );
    }

    #[tokio::test]
    async fn commit_nfo_import_rolls_back_item_and_locks_when_projection_fails() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item_id = MediaItemId::new();
        let missing_item_id = MediaItemId::new();
        let library = library(LibraryId::new());
        let original = media_item(item_id, "Original");
        let updated = media_item(item_id, "NFO Title");

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&original).await.unwrap();

        let err = store
            .commit_nfo_import(&NfoImportPersistenceCommit {
                items: vec![updated],
                field_locks: vec![MetadataFieldLock {
                    item_id,
                    field: MetadataField::Title,
                    locked: true,
                    source: MetadataSource::Nfo,
                }],
                library_item_states: vec![LibraryItemState {
                    library_id: library.id,
                    item_id,
                    provisional: false,
                }],
                catalog_projections: vec![CatalogItemProjectionCommit {
                    graph: CatalogItemGraphReplacement {
                        genres: vec![Genre {
                            id: GenreId::new(),
                            name: "Broken".to_owned(),
                            source: MetadataSource::Nfo,
                        }],
                        item_genres: vec![ItemGenre {
                            item_id: missing_item_id,
                            genre_id: GenreId::new(),
                        }],
                        ..CatalogItemGraphReplacement::default()
                    },
                    search: CatalogSearchProjection {
                        item_id: missing_item_id,
                        title: "Broken".to_owned(),
                        body: String::new(),
                        facets: Vec::new(),
                    },
                }],
            })
            .await
            .unwrap_err();

        assert!(!err.to_string().is_empty());
        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(original));
        assert_eq!(store.list_field_locks(item_id).await.unwrap(), Vec::new());
        assert_eq!(
            store
                .get_library_item_state(library.id, item_id)
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .list_genres(taru_core::PageRequest::first_page())
                .await
                .unwrap(),
            Vec::new()
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

    fn library(id: LibraryId) -> Library {
        Library {
            id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        }
    }

    fn provider_subject(id: ProviderSubjectId, subject_key: &str) -> ProviderSubject {
        ProviderSubject {
            id,
            provider: ExternalProvider::Tmdb,
            subject_kind: ProviderSubjectKind::Movie,
            subject_key: subject_key.to_owned(),
            title: Some("Updated".to_owned()),
            release_year: Some(2026),
            locale: Some("en-US".to_owned()),
        }
    }
}
