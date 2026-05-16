use taru_core::{
    CanonicalMetadata, CatalogRepository, DirectorySnapshot, IngestionFailurePhase,
    IngestionFailureRepository, LibraryId, LibraryItemRepository, LibraryItemState,
    LibraryRepository, LocalInferenceRepository, MediaItem, MediaItemId, MediaRepository,
    MediaSource, MediaSourceId, NewIngestionFailure, PageRequest, Result, ScanRepository,
    ScanSnapshotId, ScanStatus,
};
use taru_search::{SearchDocument, SearchIndex};
use taru_vfs::StorageUri;

use super::{
    failure::ingestion_failure_time_ms,
    local_inference::{
        MediaItemResolution, local_inference_evidence_from_discovered, media_item_from_discovered,
        media_source_from_discovered, source_state_from_discovered,
    },
    scan::{DiscoveredMediaSource, LibraryScanner},
    summary::{LibraryIndexRequest, LibraryIndexSummary, LibraryScanFailure, LibraryScanRequest},
};

#[derive(Debug)]
pub struct LibraryIndexService<S, R> {
    scanner: S,
    repository: R,
}

impl<S, R> LibraryIndexService<S, R> {
    pub fn new(scanner: S, repository: R) -> Self {
        Self {
            scanner,
            repository,
        }
    }

    #[must_use]
    pub fn scanner(&self) -> &S {
        &self.scanner
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<S, R> LibraryIndexService<S, R>
where
    S: LibraryScanner,
    R: CatalogRepository
        + IngestionFailureRepository
        + LibraryItemRepository
        + LibraryRepository
        + LocalInferenceRepository
        + MediaRepository
        + ScanRepository
        + SearchIndex,
{
    pub async fn index_library(&self, request: LibraryIndexRequest) -> Result<LibraryIndexSummary> {
        self.repository.upsert_library(&request.library).await?;
        let scan_id = ScanSnapshotId::new();

        let mut summary = LibraryIndexSummary {
            job_id: request.job_id,
            library_id: request.library.id,
            scan_id,
            scanned_roots: 0,
            discovered_files: 0,
            inserted_sources: 0,
            updated_sources: 0,
            tombstoned_sources: 0,
            failed_entries: 0,
        };

        let first_root = request
            .library
            .roots
            .first()
            .map(String::as_str)
            .unwrap_or("local:///");
        self.repository
            .begin_scan_snapshot(scan_id, request.library.id, first_root)
            .await?;

        let result = self.index_roots(&request, scan_id, &mut summary).await;

        match result {
            Ok(scan) => {
                if scan.complete {
                    self.mark_missing_sources_tombstoned(request.library.id, scan_id, &mut summary)
                        .await?;
                }
                self.repository
                    .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
                    .await?;
                Ok(summary)
            }
            Err(err) => {
                self.repository
                    .complete_scan_snapshot(scan_id, ScanStatus::Failed, Some(err.to_string()))
                    .await?;
                Err(err)
            }
        }
    }

    async fn index_roots(
        &self,
        request: &LibraryIndexRequest,
        scan_id: ScanSnapshotId,
        summary: &mut LibraryIndexSummary,
    ) -> Result<IndexRootsOutcome> {
        let mut complete = true;

        for root in &request.library.roots {
            let root = StorageUri::parse(root)?;
            let scan = self
                .scanner
                .scan(LibraryScanRequest {
                    job_id: request.job_id,
                    library_id: request.library.id,
                    root,
                    force: request.force,
                })
                .await?;

            summary.scanned_roots += 1;
            summary.discovered_files += scan.discovered_files;
            summary.failed_entries += scan.failures.len() as u64;
            if scan.used_stale_cache || !scan.failures.is_empty() {
                complete = false;
            }

            for failure in &scan.failures {
                self.persist_scan_failure(request, scan_id, failure).await?;
            }

            for directory in scan.directories {
                self.repository
                    .upsert_directory_snapshot(&DirectorySnapshot {
                        scan_id,
                        uri: directory.uri.as_str().to_owned(),
                        etag: directory.etag,
                        modified_at: directory.modified_at,
                        child_count: directory.child_count,
                    })
                    .await?;
                self.repository
                    .resolve_ingestion_failure(
                        request.library.id,
                        IngestionFailurePhase::Scan,
                        directory.uri.as_str(),
                        ingestion_failure_time_ms(),
                    )
                    .await?;
            }

            for discovered in scan.media_sources {
                let locator = discovered.uri.as_str().to_owned();
                let existing = self
                    .repository
                    .get_media_source_by_locator(request.library.id, &locator)
                    .await?;

                let item_id = existing
                    .as_ref()
                    .map(|source| source.item_id)
                    .unwrap_or_else(MediaItemId::new);
                let source_id = existing
                    .as_ref()
                    .map(|source| source.id)
                    .unwrap_or_else(MediaSourceId::new);

                let item_resolution = self
                    .media_item_for_discovered(request.library.id, item_id, &discovered)
                    .await?;
                let evidence = local_inference_evidence_from_discovered(source_id, &discovered);
                let state = source_state_from_discovered(
                    request.library.id,
                    source_id,
                    scan_id,
                    &discovered,
                );
                let source = media_source_from_discovered(
                    source_id,
                    request.library.id,
                    item_resolution.item.id,
                    discovered,
                );

                self.repository
                    .record_scanned_media_source(&item_resolution.item, &source, &state)
                    .await?;
                self.record_library_item_state(
                    request.library.id,
                    item_resolution.item.id,
                    item_resolution.provisional,
                )
                .await?;
                self.repository
                    .upsert_local_inference_evidence(&evidence)
                    .await?;
                self.rebuild_search_document(item_resolution.item, source)
                    .await?;
                self.repository
                    .resolve_ingestion_failure(
                        request.library.id,
                        IngestionFailurePhase::Scan,
                        &locator,
                        ingestion_failure_time_ms(),
                    )
                    .await?;

                if existing.is_some() {
                    summary.updated_sources += 1;
                } else {
                    summary.inserted_sources += 1;
                }
            }
        }

        Ok(IndexRootsOutcome { complete })
    }

    async fn media_item_for_discovered(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
        discovered: &DiscoveredMediaSource,
    ) -> Result<MediaItemResolution> {
        if let Some(state) = self
            .repository
            .get_library_item_state(library_id, item_id)
            .await?
        {
            if !state.provisional {
                if let Some(item) = self.repository.get_media_item(item_id).await? {
                    return Ok(MediaItemResolution {
                        item,
                        provisional: false,
                    });
                }
            }
        }

        if discovered.parsed_name.kind_hint != taru_core::MediaKind::Episode {
            return Ok(MediaItemResolution {
                item: media_item_from_discovered(item_id, discovered),
                provisional: true,
            });
        }

        let Some(season_number) = discovered.parsed_name.season_number else {
            return Ok(MediaItemResolution {
                item: media_item_from_discovered(item_id, discovered),
                provisional: true,
            });
        };
        let series = self
            .find_or_create_provisional_item(
                library_id,
                taru_core::MediaKind::Series,
                None,
                &discovered.parsed_name.title,
                None,
            )
            .await?;
        let season = self
            .find_or_create_provisional_item(
                library_id,
                taru_core::MediaKind::Season,
                Some(series.id),
                &format!("Season {season_number}"),
                None,
            )
            .await?;
        let episode_title = discovered
            .parsed_name
            .episode_number
            .map(|episode| format!("Episode {episode}"))
            .unwrap_or_else(|| discovered.parsed_name.title.clone());

        Ok(MediaItemResolution {
            item: MediaItem {
                id: item_id,
                kind: taru_core::MediaKind::Episode,
                parent_id: Some(season.id),
                metadata: CanonicalMetadata {
                    title: episode_title,
                    original_title: None,
                    sort_title: None,
                    overview: None,
                    release_date: discovered.parsed_name.year.map(|year| year.to_string()),
                    external_ids: Vec::new(),
                    ..CanonicalMetadata::default()
                },
            },
            provisional: true,
        })
    }

    async fn find_or_create_provisional_item(
        &self,
        library_id: LibraryId,
        kind: taru_core::MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
        release_year: Option<u16>,
    ) -> Result<MediaItem> {
        if let Some(item) = self
            .repository
            .find_library_item_by_kind_parent_title(library_id, kind, parent_id, title)
            .await?
        {
            return Ok(item);
        }

        let item = MediaItem {
            id: MediaItemId::new(),
            kind,
            parent_id,
            metadata: CanonicalMetadata {
                title: title.to_owned(),
                original_title: None,
                sort_title: None,
                overview: None,
                release_date: release_year.map(|year| year.to_string()),
                external_ids: Vec::new(),
                ..CanonicalMetadata::default()
            },
        };

        self.repository.upsert_media_item(&item).await?;
        self.record_library_item_state(library_id, item.id, true)
            .await?;

        Ok(item)
    }

    async fn record_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
        provisional: bool,
    ) -> Result<()> {
        self.repository
            .upsert_library_item_state(&LibraryItemState {
                library_id,
                item_id,
                provisional,
            })
            .await
    }

    async fn persist_scan_failure(
        &self,
        request: &LibraryIndexRequest,
        scan_id: ScanSnapshotId,
        failure: &LibraryScanFailure,
    ) -> Result<()> {
        self.repository
            .record_ingestion_failure(NewIngestionFailure {
                library_id: request.library.id,
                job_id: Some(request.job_id),
                scan_id: Some(scan_id),
                source_id: None,
                phase: IngestionFailurePhase::Scan,
                target_uri: failure.uri.as_str().to_owned(),
                target_kind: failure.target_kind.clone(),
                failure_class: failure.failure_class,
                message: failure.message.clone(),
                retryable: failure.retryable,
                failed_at_ms: ingestion_failure_time_ms(),
            })
            .await?;

        Ok(())
    }

    async fn rebuild_search_document(&self, item: MediaItem, source: MediaSource) -> Result<()> {
        let item_credits = self.repository.list_item_credits(item.id).await?;
        let item_genres = self.repository.list_item_genres(item.id).await?;
        let item_tags = self.repository.list_item_tags(item.id).await?;
        let item_studios = self.repository.list_item_studios(item.id).await?;
        let mut body_parts = Vec::new();
        let mut facets = vec![
            format!("kind:{}", item.kind.as_str()),
            format!("source:{}", source.file_name),
        ];

        if let Some(value) = &item.metadata.original_title {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.overview {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.tagline {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.release_date {
            facets.push(format!("release_date:{value}"));
        }

        for genre in item_genres {
            if let Some(genre) = self.repository.get_genre(genre.genre_id).await? {
                body_parts.push(genre.name.clone());
                facets.push(format!("genre:{}", genre.name));
            }
        }

        for tag in item_tags {
            if let Some(tag) = self.repository.get_tag(tag.tag_id).await? {
                body_parts.push(tag.name.clone());
                facets.push(format!("tag:{}", tag.name));
            }
        }

        for studio in item_studios {
            if let Some(studio) = self.repository.get_studio(studio.studio_id).await? {
                body_parts.push(studio.name.clone());
                facets.push(format!("studio:{}", studio.name));
            }
        }

        for credit in item_credits {
            if let Some(person) = self.repository.get_person(credit.person_id).await? {
                body_parts.push(person.name.clone());
                facets.push(format!("credit:{}", person.name));
            }
        }

        self.repository
            .upsert(SearchDocument {
                item_id: item.id,
                title: item.metadata.title,
                body: body_parts.join(" "),
                facets,
            })
            .await
    }

    async fn mark_missing_sources_tombstoned(
        &self,
        library_id: LibraryId,
        scan_id: ScanSnapshotId,
        summary: &mut LibraryIndexSummary,
    ) -> Result<()> {
        let mut offset = 0;

        loop {
            let states = self
                .repository
                .list_source_states(
                    library_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = states.len();

            for mut state in states {
                if state.last_seen_scan_id != scan_id && !state.tombstoned {
                    state.tombstoned = true;
                    self.repository.upsert_source_state(&state).await?;
                    summary.tombstoned_sources += 1;
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IndexRootsOutcome {
    complete: bool,
}
