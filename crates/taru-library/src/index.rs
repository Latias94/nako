use taru_core::{
    BrowseFacet, BrowseFacetKind, CanonicalMetadata, CatalogRepository, CatalogSearchProjection,
    DirectorySnapshot, IngestionFailurePhase, IngestionFailureRepository,
    IngestionFailureResolution, LibraryId, LibraryItemRepository, LibraryItemState,
    LibraryRepository, LibraryScanSourcePersistenceCommit, MediaItem, MediaItemId, MediaRepository,
    MediaSource, MediaSourceId, NewIngestionFailure, PageRequest, Result, ScanRepository,
    ScanSnapshotId, ScanStatus, SortKey, SortKeyKind,
};
use taru_vfs::StorageUri;

use super::{
    failure::ingestion_failure_time_ms,
    local_inference::{
        LocalInferenceEngine, LocalInferencePlan, LocalInferenceRequest, MediaItemResolution,
        ProvisionalAncestorPlan, ProvisionalItemPlan, resolve_local_inference_plan,
    },
    scan::LibraryScanner,
    summary::{LibraryIndexRequest, LibraryIndexSummary, LibraryScanFailure, LibraryScanRequest},
};

pub trait LibraryIndexRepository:
    CatalogRepository
    + IngestionFailureRepository
    + LibraryItemRepository
    + LibraryRepository
    + MediaRepository
    + ScanRepository
{
}

impl<T> LibraryIndexRepository for T where
    T: CatalogRepository
        + IngestionFailureRepository
        + LibraryItemRepository
        + LibraryRepository
        + MediaRepository
        + ScanRepository
{
}

#[derive(Debug)]
pub struct LibraryIndexService<S, R> {
    scanner: S,
    repository: R,
    local_inference: LocalInferenceEngine,
}

impl<S, R> LibraryIndexService<S, R> {
    pub fn new(scanner: S, repository: R) -> Self {
        Self {
            scanner,
            repository,
            local_inference: LocalInferenceEngine::with_default_parser(),
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
    R: LibraryIndexRepository,
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

                let local_inference = self.local_inference.plan_source(LocalInferenceRequest {
                    library_id: request.library.id,
                    source_id,
                    item_id,
                    scan_id,
                    discovered: &discovered,
                });
                let item_resolution = self
                    .media_item_for_local_inference(
                        request.library.id,
                        item_id,
                        local_inference.clone(),
                    )
                    .await?;
                let mut source = local_inference.media_source;
                source.item_id = item_resolution.item.id;
                let search_projection = self
                    .search_projection_for_source(&item_resolution.item, &source)
                    .await?;
                let mut items = item_resolution.supporting_items;
                items.push(item_resolution.item.clone());
                let mut library_item_states = item_resolution.supporting_library_item_states;
                library_item_states.push(LibraryItemState {
                    library_id: request.library.id,
                    item_id: item_resolution.item.id,
                    provisional: item_resolution.provisional,
                });

                self.repository
                    .commit_library_scan_source(&LibraryScanSourcePersistenceCommit {
                        items,
                        source,
                        source_state: local_inference.source_state,
                        library_item_states,
                        local_inference_evidence: vec![local_inference.evidence],
                        search_projections: vec![search_projection],
                        resolved_ingestion_failures: vec![IngestionFailureResolution {
                            library_id: request.library.id,
                            phase: IngestionFailurePhase::Scan,
                            target_uri: locator.clone(),
                            resolved_at_ms: ingestion_failure_time_ms(),
                        }],
                    })
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

    async fn media_item_for_local_inference(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
        plan: LocalInferencePlan,
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
                        supporting_items: Vec::new(),
                        supporting_library_item_states: Vec::new(),
                    });
                }
            }
        }

        let mut parent_id = None;
        let mut supporting_items = Vec::new();
        for ancestor in &plan.hierarchy.required_ancestors {
            let supporting = self
                .plan_or_reuse_provisional_ancestor(library_id, parent_id, ancestor)
                .await?;
            parent_id = Some(supporting.item.id);
            supporting_items.push(supporting);
        }

        Ok(resolve_local_inference_plan(plan, supporting_items))
    }

    async fn plan_or_reuse_provisional_ancestor(
        &self,
        library_id: LibraryId,
        parent_id: Option<MediaItemId>,
        plan: &ProvisionalAncestorPlan,
    ) -> Result<ProvisionalItemPlan> {
        if let Some(item) = self
            .repository
            .find_library_item_by_kind_parent_title(library_id, plan.kind, parent_id, &plan.title)
            .await?
        {
            return Ok(ProvisionalItemPlan {
                item,
                created: false,
            });
        }

        let item = MediaItem {
            id: MediaItemId::new(),
            kind: plan.kind,
            parent_id,
            metadata: CanonicalMetadata {
                title: plan.title.clone(),
                original_title: None,
                sort_title: None,
                overview: None,
                release_date: plan.release_year.map(|year| year.to_string()),
                external_ids: Vec::new(),
                ..CanonicalMetadata::default()
            },
        };

        Ok(ProvisionalItemPlan {
            item,
            created: true,
        })
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

    async fn search_projection_for_source(
        &self,
        item: &MediaItem,
        source: &MediaSource,
    ) -> Result<CatalogSearchProjection> {
        let item_credits = self.repository.list_item_credits(item.id).await?;
        let item_genres = self.repository.list_item_genres(item.id).await?;
        let item_tags = self.repository.list_item_tags(item.id).await?;
        let item_studios = self.repository.list_item_studios(item.id).await?;
        let mut body_parts = Vec::new();
        let mut browse_facets = vec![
            BrowseFacet::new(BrowseFacetKind::Kind, item.kind.as_str()),
            BrowseFacet::new(BrowseFacetKind::Source, source.file_name.clone()),
        ];
        let mut aliases = Vec::new();
        let mut sort_keys = Vec::new();

        if let Some(value) = &item.metadata.original_title {
            body_parts.push(value.clone());
            push_unique_string(&mut aliases, value.clone());
        }
        if let Some(value) = &item.metadata.sort_title {
            body_parts.push(value.clone());
            sort_keys.push(SortKey::new(SortKeyKind::SortTitle, value.clone()));
        } else if !item.metadata.title.trim().is_empty() {
            sort_keys.push(SortKey::new(
                SortKeyKind::Title,
                item.metadata.title.clone(),
            ));
        }
        if let Some(value) = &item.metadata.overview {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.tagline {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.release_date {
            sort_keys.push(SortKey::new(SortKeyKind::ReleaseDate, value.clone()));
            if let Some(year) = value
                .get(0..4)
                .filter(|year| year.chars().all(|character| character.is_ascii_digit()))
            {
                push_unique_facet(
                    &mut browse_facets,
                    BrowseFacet::new(BrowseFacetKind::ReleaseYear, year.to_owned()),
                );
            }
        }

        for genre in item_genres {
            if let Some(genre) = self.repository.get_genre(genre.genre_id).await? {
                body_parts.push(genre.name.clone());
                push_unique_facet(
                    &mut browse_facets,
                    BrowseFacet::new(BrowseFacetKind::Genre, genre.name),
                );
            }
        }

        for tag in item_tags {
            if let Some(tag) = self.repository.get_tag(tag.tag_id).await? {
                body_parts.push(tag.name.clone());
                push_unique_facet(
                    &mut browse_facets,
                    BrowseFacet::new(BrowseFacetKind::Tag, tag.name),
                );
            }
        }

        for studio in item_studios {
            if let Some(studio) = self.repository.get_studio(studio.studio_id).await? {
                body_parts.push(studio.name.clone());
                push_unique_facet(
                    &mut browse_facets,
                    BrowseFacet::new(BrowseFacetKind::Studio, studio.name),
                );
            }
        }

        for credit in item_credits {
            if let Some(person) = self.repository.get_person(credit.person_id).await? {
                body_parts.push(person.name.clone());
                push_unique_facet(
                    &mut browse_facets,
                    BrowseFacet::new(BrowseFacetKind::Credit, person.name),
                );
            }
        }

        browse_facets.sort_by_key(BrowseFacet::label);
        let mut projection = CatalogSearchProjection::new(
            item.id,
            item.metadata.title.clone(),
            body_parts.join(" "),
        );
        projection.aliases = aliases;
        projection.browse_facets = browse_facets;
        projection.sort_keys = sort_keys;
        projection.provider_identifiers = item.metadata.external_ids.clone();
        Ok(projection)
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

fn push_unique_facet(facets: &mut Vec<BrowseFacet>, value: BrowseFacet) {
    if !facets.contains(&value) {
        facets.push(value);
    }
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IndexRootsOutcome {
    complete: bool,
}
