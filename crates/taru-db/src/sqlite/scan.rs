use super::{SqliteStore, codec::*};
use sqlx::Sqlite;
use taru_core::*;

#[async_trait::async_trait]
impl ScanRepository for SqliteStore {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            INSERT INTO scan_snapshots (id, library_id, root, status)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(id.to_string())
        .bind(library_id.to_string())
        .bind(root)
        .bind(ScanStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            UPDATE scan_snapshots
            SET
                status = ?2,
                error = ?3,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT id, library_id, root, started_at, completed_at, status, error
            FROM scan_snapshots
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_scan_snapshot).transpose()
    }

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO directory_snapshots (
                scan_id, uri, etag, modified_at, child_count
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(scan_id, uri) DO UPDATE SET
                etag = excluded.etag,
                modified_at = excluded.modified_at,
                child_count = excluded.child_count
            "#,
        )
        .bind(snapshot.scan_id.to_string())
        .bind(&snapshot.uri)
        .bind(&snapshot.etag)
        .bind(&snapshot.modified_at)
        .bind(u64_to_i64(snapshot.child_count)?)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT scan_id, uri, etag, modified_at, child_count
            FROM directory_snapshots
            WHERE scan_id = ?1
            ORDER BY uri ASC
            "#,
        )
        .bind(scan_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_directory_snapshot).collect()
    }

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_source_state_in_transaction(&mut transaction, state).await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn commit_library_scan_source(
        &self,
        commit: &LibraryScanSourcePersistenceCommit,
    ) -> Result<LibraryScanSourcePersistenceSummary> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        for item in &commit.items {
            crate::sqlite::media::upsert_media_item_in_transaction(&mut transaction, item).await?;
        }
        crate::sqlite::media::upsert_media_source_in_transaction(&mut transaction, &commit.source)
            .await?;
        upsert_source_state_in_transaction(&mut transaction, &commit.source_state).await?;
        for state in &commit.library_item_states {
            crate::sqlite::library_item::upsert_library_item_state_tx(&mut transaction, state)
                .await?;
        }
        for evidence in &commit.local_inference_evidence {
            crate::sqlite::local_inference::upsert_local_inference_evidence_tx(
                &mut transaction,
                evidence,
            )
            .await?;
        }
        for projection in &commit.search_projections {
            crate::sqlite::catalog::upsert_search_projection_tx(&mut transaction, projection)
                .await?;
        }
        let mut resolved_ingestion_failures = 0;
        for resolution in &commit.resolved_ingestion_failures {
            resolved_ingestion_failures += crate::sqlite::ingestion::resolve_ingestion_failure_tx(
                &mut transaction,
                resolution.library_id,
                resolution.phase,
                &resolution.target_uri,
                resolution.resolved_at_ms,
            )
            .await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(LibraryScanSourcePersistenceSummary {
            item_ids: commit.items.iter().map(|item| item.id).collect(),
            source_id: commit.source.id,
            library_item_states: commit.library_item_states.len() as u64,
            local_inference_evidence: commit.local_inference_evidence.len() as u64,
            search_projections: commit.search_projections.len() as u64,
            resolved_ingestion_failures,
        })
    }

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            FROM source_states
            WHERE library_id = ?1 AND uri = ?2
            "#,
        )
        .bind(library_id.to_string())
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_source_state).transpose()
    }

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            FROM source_states
            WHERE library_id = ?1
            ORDER BY uri ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_source_state).collect()
    }
}

async fn upsert_source_state_in_transaction(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    state: &SourceState,
) -> Result<()> {
    sqlx::query(
        r#"
            INSERT INTO source_states (
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(library_id, uri) DO UPDATE SET
                source_id = excluded.source_id,
                size_bytes = excluded.size_bytes,
                modified_at = excluded.modified_at,
                etag = excluded.etag,
                fingerprint = excluded.fingerprint,
                last_seen_scan_id = excluded.last_seen_scan_id,
                tombstoned = excluded.tombstoned,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
    )
    .bind(state.library_id.to_string())
    .bind(state.source_id.map(|id| id.to_string()))
    .bind(&state.uri)
    .bind(optional_u64_to_i64(state.size_bytes)?)
    .bind(&state.modified_at)
    .bind(&state.etag)
    .bind(&state.fingerprint)
    .bind(state.last_seen_scan_id.to_string())
    .bind(bool_to_i64(state.tombstoned))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use taru_core::{
        CanonicalMetadata, CatalogSearchProjection, DatabaseLifecycle, IngestionFailureClass,
        IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRepository,
        IngestionFailureResolution, IngestionFailureStatus, Library, LibraryId,
        LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryRepository,
        LibraryScanSourcePersistenceCommit, LocalInferenceEvidence, LocalInferenceEvidenceId,
        LocalInferenceEvidenceSource, LocalInferenceRepository, MediaItem, MediaItemId, MediaKind,
        MediaRepository, MediaSource, MediaSourceId, NewIngestionFailure, PageRequest,
        ScanRepository, ScanSnapshotId, SourceState,
    };
    use taru_search::{SearchIndex, SearchQuery};

    use crate::sqlite::SqliteStore;

    #[tokio::test]
    async fn commit_library_scan_source_writes_full_source_unit_and_resolves_failure() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let scan_id = ScanSnapshotId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let locator = "local:///Movies/M19.mkv";
        let item = media_item(item_id, "M19");
        let source = media_source(library_id, item_id, source_id, locator);
        let state = source_state(library_id, source_id, scan_id, locator);

        store
            .upsert_library(&Library {
                id: library_id,
                name: "Movies".to_owned(),
                roots: vec!["local:///Movies".to_owned()],
                options: LibraryOptions::default(),
            })
            .await
            .unwrap();
        store
            .begin_scan_snapshot(scan_id, library_id, "local:///Movies")
            .await
            .unwrap();
        store
            .record_ingestion_failure(NewIngestionFailure {
                library_id,
                job_id: None,
                scan_id: Some(scan_id),
                source_id: None,
                phase: IngestionFailurePhase::Scan,
                target_uri: locator.to_owned(),
                target_kind: "source".to_owned(),
                failure_class: IngestionFailureClass::Storage,
                message: "source was previously unreadable".to_owned(),
                retryable: true,
                failed_at_ms: 10,
            })
            .await
            .unwrap();

        store
            .commit_library_scan_source(&LibraryScanSourcePersistenceCommit {
                items: vec![item.clone()],
                source: source.clone(),
                source_state: state.clone(),
                library_item_states: vec![LibraryItemState {
                    library_id,
                    item_id,
                    provisional: true,
                }],
                local_inference_evidence: vec![local_inference_evidence(source_id)],
                search_projections: vec![
                    CatalogSearchProjection::try_from_facet_labels(
                        item_id,
                        "M19",
                        "M19 M19.mkv",
                        vec!["kind:movie".to_owned(), "source:M19.mkv".to_owned()],
                    )
                    .unwrap(),
                ],
                resolved_ingestion_failures: vec![IngestionFailureResolution {
                    library_id,
                    phase: IngestionFailurePhase::Scan,
                    target_uri: locator.to_owned(),
                    resolved_at_ms: 20,
                }],
            })
            .await
            .unwrap();

        assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(item));
        assert_eq!(
            store.get_media_source(source_id).await.unwrap(),
            Some(source)
        );
        assert_eq!(
            store.get_source_state(library_id, locator).await.unwrap(),
            Some(state)
        );
        assert_eq!(
            store
                .get_library_item_state(library_id, item_id)
                .await
                .unwrap(),
            Some(LibraryItemState {
                library_id,
                item_id,
                provisional: true,
            })
        );
        assert_eq!(
            store
                .list_local_inference_evidence_for_source(source_id, PageRequest::first_page())
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            store
                .search(SearchQuery::from_facet_labels(
                    "m19",
                    vec!["source:M19.mkv".to_owned()],
                    10,
                    0,
                )
                .unwrap())
                .await
                .unwrap()[0]
                .item_id,
            item_id
        );
        assert_eq!(
            store
                .list_ingestion_failures(
                    IngestionFailureFilter {
                        library_id: Some(library_id),
                        phase: Some(IngestionFailurePhase::Scan),
                        status: Some(IngestionFailureStatus::Resolved),
                    },
                    PageRequest::first_page(),
                )
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn commit_library_scan_source_rolls_back_when_search_projection_fails() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let scan_id = ScanSnapshotId::new();
        let item_id = MediaItemId::new();
        let missing_item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let locator = "local:///Movies/BrokenSearch.mkv";
        let item = media_item(item_id, "Broken Search");
        let source = media_source(library_id, item_id, source_id, locator);
        let state = source_state(library_id, source_id, scan_id, locator);

        store
            .upsert_library(&Library {
                id: library_id,
                name: "Movies".to_owned(),
                roots: vec!["local:///Movies".to_owned()],
                options: LibraryOptions::default(),
            })
            .await
            .unwrap();
        store
            .begin_scan_snapshot(scan_id, library_id, "local:///Movies")
            .await
            .unwrap();
        store
            .record_ingestion_failure(NewIngestionFailure {
                library_id,
                job_id: None,
                scan_id: Some(scan_id),
                source_id: None,
                phase: IngestionFailurePhase::Scan,
                target_uri: locator.to_owned(),
                target_kind: "source".to_owned(),
                failure_class: IngestionFailureClass::Storage,
                message: "source was previously unreadable".to_owned(),
                retryable: true,
                failed_at_ms: 10,
            })
            .await
            .unwrap();

        let err = store
            .commit_library_scan_source(&LibraryScanSourcePersistenceCommit {
                items: vec![item],
                source: source.clone(),
                source_state: state,
                library_item_states: vec![LibraryItemState {
                    library_id,
                    item_id,
                    provisional: true,
                }],
                local_inference_evidence: vec![local_inference_evidence(source_id)],
                search_projections: vec![CatalogSearchProjection::new(
                    missing_item_id,
                    "Broken",
                    String::new(),
                )],
                resolved_ingestion_failures: vec![IngestionFailureResolution {
                    library_id,
                    phase: IngestionFailurePhase::Scan,
                    target_uri: locator.to_owned(),
                    resolved_at_ms: 20,
                }],
            })
            .await
            .unwrap_err();
        assert!(!err.to_string().is_empty());
        assert_eq!(store.get_media_item(item_id).await.unwrap(), None);
        assert_eq!(store.get_media_source(source_id).await.unwrap(), None);
        assert_eq!(
            store.get_source_state(library_id, locator).await.unwrap(),
            None
        );
        assert_eq!(
            store
                .get_library_item_state(library_id, item_id)
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .list_local_inference_evidence_for_source(source_id, PageRequest::first_page())
                .await
                .unwrap(),
            Vec::new()
        );
        assert_eq!(
            store
                .list_ingestion_failures(
                    IngestionFailureFilter {
                        library_id: Some(library_id),
                        phase: Some(IngestionFailurePhase::Scan),
                        status: Some(IngestionFailureStatus::Open),
                    },
                    PageRequest::first_page(),
                )
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn ingestion_failures_upsert_resolve_ignore_and_filter() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let scan_id = ScanSnapshotId::new();
        let source_id = MediaSourceId::new();
        let item_id = MediaItemId::new();
        let locator = "local:///Movies/Broken.mkv";

        store
            .upsert_library(&Library {
                id: library_id,
                name: "Movies".to_owned(),
                roots: vec!["local:///Movies".to_owned()],
                options: LibraryOptions::default(),
            })
            .await
            .unwrap();
        store
            .begin_scan_snapshot(scan_id, library_id, "local:///Movies")
            .await
            .unwrap();
        store
            .upsert_media_item(&media_item(item_id, "Broken"))
            .await
            .unwrap();
        store
            .upsert_media_source(&media_source(library_id, item_id, source_id, locator))
            .await
            .unwrap();

        let first = store
            .record_ingestion_failure(NewIngestionFailure {
                library_id,
                job_id: None,
                scan_id: Some(scan_id),
                source_id: Some(source_id),
                phase: IngestionFailurePhase::Probe,
                target_uri: locator.to_owned(),
                target_kind: "source".to_owned(),
                failure_class: IngestionFailureClass::Probe,
                message: "ffprobe failed".to_owned(),
                retryable: true,
                failed_at_ms: 10,
            })
            .await
            .unwrap();
        let second = store
            .record_ingestion_failure(NewIngestionFailure {
                message: "ffprobe still failed".to_owned(),
                failed_at_ms: 20,
                ..NewIngestionFailure {
                    library_id,
                    job_id: None,
                    scan_id: Some(scan_id),
                    source_id: Some(source_id),
                    phase: IngestionFailurePhase::Probe,
                    target_uri: locator.to_owned(),
                    target_kind: "source".to_owned(),
                    failure_class: IngestionFailureClass::Probe,
                    message: "unused".to_owned(),
                    retryable: true,
                    failed_at_ms: 0,
                }
            })
            .await
            .unwrap();

        assert_eq!(first.attempts, 1);
        assert_eq!(second.attempts, 2);
        assert_eq!(second.status, IngestionFailureStatus::Open);
        assert_eq!(second.message, "ffprobe still failed");
        assert_eq!(
            store
                .count_ingestion_failures(
                    library_id,
                    Some(IngestionFailurePhase::Probe),
                    IngestionFailureStatus::Open
                )
                .await
                .unwrap(),
            1
        );

        let resolved = store
            .resolve_ingestion_failure(library_id, IngestionFailurePhase::Probe, locator, 30)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved.status, IngestionFailureStatus::Resolved);
        assert_eq!(resolved.resolved_at_ms, Some(30));

        let reopened = store
            .record_ingestion_failure(NewIngestionFailure {
                library_id,
                job_id: None,
                scan_id: Some(scan_id),
                source_id: Some(source_id),
                phase: IngestionFailurePhase::Probe,
                target_uri: locator.to_owned(),
                target_kind: "source".to_owned(),
                failure_class: IngestionFailureClass::Probe,
                message: "ffprobe failed again".to_owned(),
                retryable: true,
                failed_at_ms: 40,
            })
            .await
            .unwrap();
        assert_eq!(reopened.status, IngestionFailureStatus::Open);
        assert_eq!(reopened.attempts, 3);
        assert_eq!(reopened.resolved_at_ms, None);

        let ignored = store
            .ignore_ingestion_failure(library_id, IngestionFailurePhase::Probe, locator, 50)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ignored.status, IngestionFailureStatus::Ignored);
        assert_eq!(ignored.ignored_at_ms, Some(50));

        let records = store
            .list_ingestion_failures(
                IngestionFailureFilter {
                    library_id: Some(library_id),
                    phase: Some(IngestionFailurePhase::Probe),
                    status: Some(IngestionFailureStatus::Ignored),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap();
        assert_eq!(records, vec![ignored]);
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

    fn media_source(
        library_id: LibraryId,
        item_id: MediaItemId,
        source_id: MediaSourceId,
        locator: &str,
    ) -> MediaSource {
        MediaSource {
            id: source_id,
            library_id,
            item_id,
            locator: locator.to_owned(),
            file_name: "M19.mkv".to_owned(),
            size_bytes: Some(19),
            fingerprint: Some("m19-fingerprint".to_owned()),
        }
    }

    fn source_state(
        library_id: LibraryId,
        source_id: MediaSourceId,
        scan_id: ScanSnapshotId,
        locator: &str,
    ) -> SourceState {
        SourceState {
            library_id,
            source_id: Some(source_id),
            uri: locator.to_owned(),
            size_bytes: Some(19),
            modified_at: Some("2026-05-16T00:00:00Z".to_owned()),
            etag: Some("m19-etag".to_owned()),
            fingerprint: Some("m19-fingerprint".to_owned()),
            last_seen_scan_id: scan_id,
            tombstoned: false,
        }
    }

    fn local_inference_evidence(source_id: MediaSourceId) -> LocalInferenceEvidence {
        LocalInferenceEvidence {
            id: LocalInferenceEvidenceId::new(),
            source_id,
            inferred_kind: MediaKind::Movie,
            inferred_title: Some("M19".to_owned()),
            inferred_year: None,
            inferred_season: None,
            inferred_episode: None,
            confidence_milli: Some(900),
            evidence_source: LocalInferenceEvidenceSource::FileName,
            evidence_value: "M19.mkv".to_owned(),
            inference_version: "test".to_owned(),
        }
    }
}
