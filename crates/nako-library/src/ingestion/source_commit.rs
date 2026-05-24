use async_trait::async_trait;
use nako_core::{
    BrowseFacet, BrowseFacetKind, CanonicalMetadata, CatalogRepository, CatalogSearchProjection,
    Genre, IngestionFailurePhase, IngestionFailureResolution, ItemCredit, ItemGenre, ItemStudio,
    ItemTag, LibraryId, LibraryItemRepository, LibraryItemState,
    LibraryScanSourcePersistenceCommit, MediaItem, MediaItemId, MediaKind, MediaRepository,
    MediaSource, MediaSourceId, Person, Result, SortKey, SortKeyKind, Studio, Tag,
};

use crate::{
    failure::ingestion_failure_time_ms,
    ingestion::LibrarySourceObservationCommit,
    local_inference::{
        LocalInferenceEngine, LocalInferencePlan, LocalInferenceRequest, MediaItemResolution,
        ProvisionalAncestorPlan, ProvisionalItemPlan, resolve_local_inference_plan,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceObservationPersistencePlan {
    pub(crate) disposition: SourceObservationDisposition,
    pub(crate) commit: LibraryScanSourcePersistenceCommit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceObservationDisposition {
    Inserted,
    Updated,
}

impl SourceObservationDisposition {
    #[must_use]
    pub(crate) const fn is_update(self) -> bool {
        matches!(self, Self::Updated)
    }
}

#[async_trait]
pub(crate) trait SourceObservationCommitRepository: Send + Sync {
    async fn find_source_by_locator(
        &self,
        library_id: LibraryId,
        locator: &str,
    ) -> Result<Option<MediaSource>>;

    async fn get_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn find_library_item_by_kind_parent_title(
        &self,
        library_id: LibraryId,
        kind: MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
    ) -> Result<Option<MediaItem>>;

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>>;

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>>;

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>>;

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>>;

    async fn get_person(&self, id: nako_core::PersonId) -> Result<Option<Person>>;

    async fn get_genre(&self, id: nako_core::GenreId) -> Result<Option<Genre>>;

    async fn get_tag(&self, id: nako_core::TagId) -> Result<Option<Tag>>;

    async fn get_studio(&self, id: nako_core::StudioId) -> Result<Option<Studio>>;
}

#[async_trait]
impl<T> SourceObservationCommitRepository for T
where
    T: CatalogRepository + LibraryItemRepository + MediaRepository + Send + Sync + ?Sized,
{
    async fn find_source_by_locator(
        &self,
        library_id: LibraryId,
        locator: &str,
    ) -> Result<Option<MediaSource>> {
        MediaRepository::get_media_source_by_locator(self, library_id, locator).await
    }

    async fn get_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>> {
        LibraryItemRepository::get_library_item_state(self, library_id, item_id).await
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        MediaRepository::get_media_item(self, id).await
    }

    async fn find_library_item_by_kind_parent_title(
        &self,
        library_id: LibraryId,
        kind: MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
    ) -> Result<Option<MediaItem>> {
        LibraryItemRepository::find_library_item_by_kind_parent_title(
            self, library_id, kind, parent_id, title,
        )
        .await
    }

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
        CatalogRepository::list_item_credits(self, item_id).await
    }

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
        CatalogRepository::list_item_genres(self, item_id).await
    }

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>> {
        CatalogRepository::list_item_tags(self, item_id).await
    }

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
        CatalogRepository::list_item_studios(self, item_id).await
    }

    async fn get_person(&self, id: nako_core::PersonId) -> Result<Option<Person>> {
        CatalogRepository::get_person(self, id).await
    }

    async fn get_genre(&self, id: nako_core::GenreId) -> Result<Option<Genre>> {
        CatalogRepository::get_genre(self, id).await
    }

    async fn get_tag(&self, id: nako_core::TagId) -> Result<Option<Tag>> {
        CatalogRepository::get_tag(self, id).await
    }

    async fn get_studio(&self, id: nako_core::StudioId) -> Result<Option<Studio>> {
        CatalogRepository::get_studio(self, id).await
    }
}

pub(crate) async fn plan_source_observation_commit<R>(
    repository: &R,
    observation: LibrarySourceObservationCommit,
) -> Result<SourceObservationPersistencePlan>
where
    R: SourceObservationCommitRepository + ?Sized,
{
    let locator = observation.discovered.uri.as_str().to_owned();
    let existing = repository
        .find_source_by_locator(observation.library_id, &locator)
        .await?;
    let disposition = if existing.is_some() {
        SourceObservationDisposition::Updated
    } else {
        SourceObservationDisposition::Inserted
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
            library_id: observation.library_id,
            source_id,
            item_id,
            scan_id: observation.scan_id,
            discovered: &observation.discovered,
        });
    let item_resolution = media_item_for_local_inference(
        repository,
        observation.library_id,
        item_id,
        local_inference.clone(),
    )
    .await?;
    let mut source = local_inference.media_source;
    source.item_id = item_resolution.item.id;
    let search_projection =
        search_projection_for_source(repository, &item_resolution.item, &source).await?;
    let mut items = item_resolution.supporting_items;
    items.push(item_resolution.item.clone());
    let mut library_item_states = item_resolution.supporting_library_item_states;
    library_item_states.push(LibraryItemState {
        library_id: observation.library_id,
        item_id: item_resolution.item.id,
        provisional: item_resolution.provisional,
    });

    Ok(SourceObservationPersistencePlan {
        disposition,
        commit: LibraryScanSourcePersistenceCommit {
            items,
            source,
            source_state: local_inference.source_state,
            library_item_states,
            local_inference_evidence: vec![local_inference.evidence],
            search_projections: vec![search_projection],
            resolved_ingestion_failures: vec![IngestionFailureResolution {
                library_id: observation.library_id,
                phase: IngestionFailurePhase::Scan,
                target_uri: locator,
                resolved_at_ms: ingestion_failure_time_ms(),
            }],
        },
    })
}

async fn media_item_for_local_inference<R>(
    repository: &R,
    library_id: LibraryId,
    item_id: MediaItemId,
    plan: LocalInferencePlan,
) -> Result<MediaItemResolution>
where
    R: SourceObservationCommitRepository + ?Sized,
{
    if let Some(state) = repository
        .get_library_item_state(library_id, item_id)
        .await?
    {
        if !state.provisional {
            if let Some(item) = repository.get_media_item(item_id).await? {
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
    R: SourceObservationCommitRepository + ?Sized,
{
    if let Some(item) = repository
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

async fn search_projection_for_source<R>(
    repository: &R,
    item: &MediaItem,
    source: &MediaSource,
) -> Result<CatalogSearchProjection>
where
    R: SourceObservationCommitRepository + ?Sized,
{
    let item_credits = repository.list_item_credits(item.id).await?;
    let item_genres = repository.list_item_genres(item.id).await?;
    let item_tags = repository.list_item_tags(item.id).await?;
    let item_studios = repository.list_item_studios(item.id).await?;
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
        if let Some(genre) = repository.get_genre(genre.genre_id).await? {
            body_parts.push(genre.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Genre, genre.name),
            );
        }
    }

    for tag in item_tags {
        if let Some(tag) = repository.get_tag(tag.tag_id).await? {
            body_parts.push(tag.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Tag, tag.name),
            );
        }
    }

    for studio in item_studios {
        if let Some(studio) = repository.get_studio(studio.studio_id).await? {
            body_parts.push(studio.name.clone());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Studio, studio.name),
            );
        }
    }

    for credit in item_credits {
        if let Some(person) = repository.get_person(credit.person_id).await? {
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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use nako_core::{
        BrowseFacet, BrowseFacetKind, CanonicalMetadata, Genre, GenreId, ItemCredit, ItemGenre,
        ItemStudio, ItemTag, LibraryId, LibraryItemState, MediaItem, MediaItemId, MediaKind,
        MediaSource, MediaSourceId, Person, PersonId, ScanSnapshotId, Studio, StudioId, Tag, TagId,
    };
    use nako_vfs::StorageUri;

    use super::*;
    use crate::{DiscoveredMediaSource, LibrarySourceObservationCommit};

    #[tokio::test]
    async fn source_observation_plan_builds_insert_commit_from_local_inference() {
        let store = FixtureSourceCommitRepository::default();
        let library_id = LibraryId::new();
        let scan_id = ScanSnapshotId::new();

        let plan = plan_source_observation_commit(
            &store,
            LibrarySourceObservationCommit {
                library_id,
                scan_id,
                discovered: discovered("local:///TV/Firefly/S01/Firefly.S01E02.mkv"),
            },
        )
        .await
        .unwrap();

        assert_eq!(plan.disposition, SourceObservationDisposition::Inserted);
        assert_eq!(plan.commit.items.len(), 3);
        assert_eq!(
            plan.commit.source.locator,
            "local:///TV/Firefly/S01/Firefly.S01E02.mkv"
        );
        assert_eq!(plan.commit.source_state.uri, plan.commit.source.locator);
        assert_eq!(plan.commit.source_state.last_seen_scan_id, scan_id);
        assert_eq!(plan.commit.library_item_states.len(), 3);
        assert_eq!(plan.commit.local_inference_evidence.len(), 1);
        assert_eq!(plan.commit.search_projections.len(), 1);

        let item = plan
            .commit
            .items
            .iter()
            .find(|item| item.kind == MediaKind::Episode)
            .unwrap();
        let projection = &plan.commit.search_projections[0];
        assert_eq!(projection.item_id, item.id);
        assert_eq!(projection.title, "Episode 2");
        assert!(
            projection
                .browse_facets
                .contains(&BrowseFacet::new(BrowseFacetKind::Kind, "episode"))
        );
        assert!(projection.browse_facets.contains(&BrowseFacet::new(
            BrowseFacetKind::Source,
            "Firefly.S01E02.mkv"
        )));
    }

    #[tokio::test]
    async fn source_observation_plan_reuses_existing_non_provisional_item() {
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let locator = "local:///Movies/The Matrix (1999).mkv";
        let store = FixtureSourceCommitRepository {
            items: vec![MediaItem {
                id: item_id,
                kind: MediaKind::Movie,
                parent_id: None,
                metadata: CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    sort_title: Some("Matrix, The".to_owned()),
                    release_date: Some("1999".to_owned()),
                    ..CanonicalMetadata::default()
                },
            }],
            sources: vec![MediaSource {
                id: source_id,
                library_id,
                item_id,
                locator: locator.to_owned(),
                file_name: "The Matrix (1999).mkv".to_owned(),
                size_bytes: Some(10),
                fingerprint: Some("fp:matrix".to_owned()),
            }],
            states: vec![LibraryItemState {
                library_id,
                item_id,
                provisional: false,
            }],
        };

        let plan = plan_source_observation_commit(
            &store,
            LibrarySourceObservationCommit {
                library_id,
                scan_id: ScanSnapshotId::new(),
                discovered: discovered(locator),
            },
        )
        .await
        .unwrap();

        assert_eq!(plan.disposition, SourceObservationDisposition::Updated);
        assert_eq!(plan.commit.items.len(), 1);
        assert_eq!(plan.commit.items[0].id, item_id);
        assert_eq!(plan.commit.items[0].metadata.title, "The Matrix");
        assert_eq!(plan.commit.source.id, source_id);
        assert_eq!(plan.commit.source.item_id, item_id);
        assert_eq!(plan.commit.library_item_states.len(), 1);
        assert!(!plan.commit.library_item_states[0].provisional);
        assert_eq!(plan.commit.search_projections[0].title, "The Matrix");
    }

    #[derive(Default)]
    struct FixtureSourceCommitRepository {
        sources: Vec<MediaSource>,
        items: Vec<MediaItem>,
        states: Vec<LibraryItemState>,
    }

    #[async_trait]
    impl SourceObservationCommitRepository for FixtureSourceCommitRepository {
        async fn find_source_by_locator(
            &self,
            library_id: LibraryId,
            locator: &str,
        ) -> Result<Option<MediaSource>> {
            Ok(self
                .sources
                .iter()
                .find(|source| source.library_id == library_id && source.locator == locator)
                .cloned())
        }

        async fn get_library_item_state(
            &self,
            library_id: LibraryId,
            item_id: MediaItemId,
        ) -> Result<Option<LibraryItemState>> {
            Ok(self
                .states
                .iter()
                .find(|state| state.library_id == library_id && state.item_id == item_id)
                .cloned())
        }

        async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
            Ok(self.items.iter().find(|item| item.id == id).cloned())
        }

        async fn find_library_item_by_kind_parent_title(
            &self,
            library_id: LibraryId,
            kind: MediaKind,
            parent_id: Option<MediaItemId>,
            title: &str,
        ) -> Result<Option<MediaItem>> {
            Ok(self
                .items
                .iter()
                .find(|item| {
                    item.kind == kind
                        && item.parent_id == parent_id
                        && item.metadata.title == title
                        && self
                            .states
                            .iter()
                            .any(|state| state.library_id == library_id && state.item_id == item.id)
                })
                .cloned())
        }

        async fn list_item_credits(&self, _item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
            Ok(Vec::new())
        }

        async fn list_item_genres(&self, _item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
            Ok(Vec::new())
        }

        async fn list_item_tags(&self, _item_id: MediaItemId) -> Result<Vec<ItemTag>> {
            Ok(Vec::new())
        }

        async fn list_item_studios(&self, _item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
            Ok(Vec::new())
        }

        async fn get_person(&self, _id: PersonId) -> Result<Option<Person>> {
            Ok(None)
        }

        async fn get_genre(&self, _id: GenreId) -> Result<Option<Genre>> {
            Ok(None)
        }

        async fn get_tag(&self, _id: TagId) -> Result<Option<Tag>> {
            Ok(None)
        }

        async fn get_studio(&self, _id: StudioId) -> Result<Option<Studio>> {
            Ok(None)
        }
    }

    fn discovered(locator: &str) -> DiscoveredMediaSource {
        let uri = StorageUri::parse(locator).unwrap();
        let file_name = uri
            .path_part()
            .rsplit_once('/')
            .map(|(_parent, file_name)| file_name)
            .unwrap_or_else(|| uri.path_part())
            .to_owned();

        DiscoveredMediaSource {
            uri,
            file_name,
            size_bytes: Some(10),
            modified_at: None,
            etag: None,
            fingerprint: Some("fp:test".to_owned()),
            stale: false,
        }
    }
}
