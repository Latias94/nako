use async_trait::async_trait;
use nako_core::{
    BrowseFacet, BrowseFacetKind, CanonicalMetadata, CatalogRepository, CatalogSearchProjection,
    DirectorySnapshot, IngestionFailurePhase, IngestionFailureRepository,
    IngestionFailureResolution, JobId, Library, LibraryId, LibraryItemRepository, LibraryItemState,
    LibraryRepository, LibraryScanSourcePersistenceCommit, LibraryScanSourcePersistenceSummary,
    MediaItem, MediaItemId, MediaRepository, MediaSource, MediaSourceId, NewIngestionFailure,
    PageRequest, Result, ScanRepository, ScanSnapshot, ScanSnapshotId, ScanStatus, SortKey,
    SortKeyKind,
};

use super::{
    failure::ingestion_failure_time_ms,
    local_inference::{
        LocalInferenceEngine, LocalInferencePlan, LocalInferenceRequest, MediaItemResolution,
        ProvisionalAncestorPlan, ProvisionalItemPlan, resolve_local_inference_plan,
    },
    scan::{DiscoveredMediaSource, ScannedDirectory},
    summary::LibraryScanFailure,
};

#[async_trait]
pub trait LibraryIngestionWorkflow: Send + Sync {
    async fn ensure_library_for_ingestion(&self, library: &Library) -> Result<()>;

    async fn begin_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot>;

    async fn complete_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot>;

    async fn record_scan_failure(&self, commit: LibraryScanFailureCommit) -> Result<()>;

    async fn commit_directory_observation(
        &self,
        commit: LibraryDirectoryObservationCommit,
    ) -> Result<()>;

    async fn commit_source_observation(
        &self,
        commit: LibrarySourceObservationCommit,
    ) -> Result<LibrarySourceIngestionSummary>;

    async fn tombstone_sources_missing_from_scan(
        &self,
        library_id: LibraryId,
        scan_id: ScanSnapshotId,
    ) -> Result<u64>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryScanFailureCommit {
    pub library_id: LibraryId,
    pub job_id: JobId,
    pub scan_id: ScanSnapshotId,
    pub failure: LibraryScanFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryDirectoryObservationCommit {
    pub library_id: LibraryId,
    pub scan_id: ScanSnapshotId,
    pub directory: ScannedDirectory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrarySourceObservationCommit {
    pub library_id: LibraryId,
    pub scan_id: ScanSnapshotId,
    pub discovered: DiscoveredMediaSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrarySourceIngestionSummary {
    pub disposition: LibrarySourceIngestionDisposition,
    pub persistence: LibraryScanSourcePersistenceSummary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LibrarySourceIngestionDisposition {
    Inserted,
    Updated,
}

impl LibrarySourceIngestionDisposition {
    #[must_use]
    pub const fn is_update(self) -> bool {
        matches!(self, Self::Updated)
    }
}

#[async_trait]
impl<T> LibraryIngestionWorkflow for T
where
    T: CatalogRepository
        + IngestionFailureRepository
        + LibraryItemRepository
        + LibraryRepository
        + MediaRepository
        + ScanRepository,
{
    async fn ensure_library_for_ingestion(&self, library: &Library) -> Result<()> {
        LibraryRepository::upsert_library(self, library).await
    }

    async fn begin_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        ScanRepository::begin_scan_snapshot(self, id, library_id, root).await
    }

    async fn complete_ingestion_scan(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        ScanRepository::complete_scan_snapshot(self, id, status, error).await
    }

    async fn record_scan_failure(&self, commit: LibraryScanFailureCommit) -> Result<()> {
        IngestionFailureRepository::record_ingestion_failure(
            self,
            NewIngestionFailure {
                library_id: commit.library_id,
                job_id: Some(commit.job_id),
                scan_id: Some(commit.scan_id),
                source_id: None,
                phase: IngestionFailurePhase::Scan,
                target_uri: commit.failure.uri.as_str().to_owned(),
                target_kind: commit.failure.target_kind,
                failure_class: commit.failure.failure_class,
                message: commit.failure.message,
                retryable: commit.failure.retryable,
                failed_at_ms: ingestion_failure_time_ms(),
            },
        )
        .await?;

        Ok(())
    }

    async fn commit_directory_observation(
        &self,
        commit: LibraryDirectoryObservationCommit,
    ) -> Result<()> {
        ScanRepository::upsert_directory_snapshot(
            self,
            &DirectorySnapshot {
                scan_id: commit.scan_id,
                uri: commit.directory.uri.as_str().to_owned(),
                etag: commit.directory.etag,
                modified_at: commit.directory.modified_at,
                child_count: commit.directory.child_count,
            },
        )
        .await?;
        IngestionFailureRepository::resolve_ingestion_failure(
            self,
            commit.library_id,
            IngestionFailurePhase::Scan,
            commit.directory.uri.as_str(),
            ingestion_failure_time_ms(),
        )
        .await?;

        Ok(())
    }

    async fn commit_source_observation(
        &self,
        commit: LibrarySourceObservationCommit,
    ) -> Result<LibrarySourceIngestionSummary> {
        let locator = commit.discovered.uri.as_str().to_owned();
        let existing =
            MediaRepository::get_media_source_by_locator(self, commit.library_id, &locator).await?;
        let disposition = if existing.is_some() {
            LibrarySourceIngestionDisposition::Updated
        } else {
            LibrarySourceIngestionDisposition::Inserted
        };
        let item_id = existing
            .as_ref()
            .map(|source| source.item_id)
            .unwrap_or_else(MediaItemId::new);
        let source_id = existing
            .as_ref()
            .map(|source| source.id)
            .unwrap_or_else(MediaSourceId::new);

        let local_inference =
            LocalInferenceEngine::with_default_parser().plan_source(LocalInferenceRequest {
                library_id: commit.library_id,
                source_id,
                item_id,
                scan_id: commit.scan_id,
                discovered: &commit.discovered,
            });
        let item_resolution = media_item_for_local_inference(
            self,
            commit.library_id,
            item_id,
            local_inference.clone(),
        )
        .await?;
        let mut source = local_inference.media_source;
        source.item_id = item_resolution.item.id;
        let search_projection =
            search_projection_for_source(self, &item_resolution.item, &source).await?;
        let mut items = item_resolution.supporting_items;
        items.push(item_resolution.item.clone());
        let mut library_item_states = item_resolution.supporting_library_item_states;
        library_item_states.push(LibraryItemState {
            library_id: commit.library_id,
            item_id: item_resolution.item.id,
            provisional: item_resolution.provisional,
        });

        let persistence = ScanRepository::commit_library_scan_source(
            self,
            &LibraryScanSourcePersistenceCommit {
                items,
                source,
                source_state: local_inference.source_state,
                library_item_states,
                local_inference_evidence: vec![local_inference.evidence],
                search_projections: vec![search_projection],
                resolved_ingestion_failures: vec![IngestionFailureResolution {
                    library_id: commit.library_id,
                    phase: IngestionFailurePhase::Scan,
                    target_uri: locator,
                    resolved_at_ms: ingestion_failure_time_ms(),
                }],
            },
        )
        .await?;

        Ok(LibrarySourceIngestionSummary {
            disposition,
            persistence,
        })
    }

    async fn tombstone_sources_missing_from_scan(
        &self,
        library_id: LibraryId,
        scan_id: ScanSnapshotId,
    ) -> Result<u64> {
        let mut offset = 0;
        let mut tombstoned = 0;

        loop {
            let states = ScanRepository::list_source_states(
                self,
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
                    ScanRepository::upsert_source_state(self, &state).await?;
                    tombstoned += 1;
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(tombstoned)
    }
}

async fn media_item_for_local_inference<R>(
    repository: &R,
    library_id: LibraryId,
    item_id: MediaItemId,
    plan: LocalInferencePlan,
) -> Result<MediaItemResolution>
where
    R: LibraryItemRepository + MediaRepository + ?Sized,
{
    if let Some(state) =
        LibraryItemRepository::get_library_item_state(repository, library_id, item_id).await?
    {
        if !state.provisional {
            if let Some(item) = MediaRepository::get_media_item(repository, item_id).await? {
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
        let supporting =
            plan_or_reuse_provisional_ancestor(repository, library_id, parent_id, ancestor).await?;
        parent_id = Some(supporting.item.id);
        supporting_items.push(supporting);
    }

    Ok(resolve_local_inference_plan(plan, supporting_items))
}

async fn plan_or_reuse_provisional_ancestor<R>(
    repository: &R,
    library_id: LibraryId,
    parent_id: Option<MediaItemId>,
    plan: &ProvisionalAncestorPlan,
) -> Result<ProvisionalItemPlan>
where
    R: LibraryItemRepository + ?Sized,
{
    if let Some(item) = LibraryItemRepository::find_library_item_by_kind_parent_title(
        repository,
        library_id,
        plan.kind,
        parent_id,
        &plan.title,
    )
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

async fn search_projection_for_source<R>(
    repository: &R,
    item: &MediaItem,
    source: &MediaSource,
) -> Result<CatalogSearchProjection>
where
    R: CatalogRepository + ?Sized,
{
    let item_credits = CatalogRepository::list_item_credits(repository, item.id).await?;
    let item_genres = CatalogRepository::list_item_genres(repository, item.id).await?;
    let item_tags = CatalogRepository::list_item_tags(repository, item.id).await?;
    let item_studios = CatalogRepository::list_item_studios(repository, item.id).await?;
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
        if let Some(genre) = CatalogRepository::get_genre(repository, genre.genre_id).await? {
            body_parts.push(genre.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Genre, genre.name),
            );
        }
    }

    for tag in item_tags {
        if let Some(tag) = CatalogRepository::get_tag(repository, tag.tag_id).await? {
            body_parts.push(tag.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Tag, tag.name),
            );
        }
    }

    for studio in item_studios {
        if let Some(studio) = CatalogRepository::get_studio(repository, studio.studio_id).await? {
            body_parts.push(studio.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Studio, studio.name),
            );
        }
    }

    for credit in item_credits {
        if let Some(person) = CatalogRepository::get_person(repository, credit.person_id).await? {
            body_parts.push(person.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Credit, person.name),
            );
        }
    }

    browse_facets.sort_by_key(BrowseFacet::label);
    let mut projection =
        CatalogSearchProjection::new(item.id, item.metadata.title.clone(), body_parts.join(" "));
    projection.aliases = aliases;
    projection.browse_facets = browse_facets;
    projection.sort_keys = sort_keys;
    projection.provider_identifiers = item.metadata.external_ids.clone();
    Ok(projection)
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
