use std::collections::HashSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{BrowseFacet, BrowseFacetKind};
use taru_core::{
    CatalogItemGraphReplacement, CatalogItemProjectionCommit, CatalogRepository,
    CatalogSearchProjection, Collection, CollectionId, CollectionItem, CollectionRef, Credit,
    CreditRole, ExternalId, ExternalProvider, Genre, GenreId, ImageAsset, ImageAssetId, ImageKind,
    ImageOwner, ItemCredit, ItemGenre, ItemStudio, ItemTag, MediaItem, MediaItemId,
    MediaRepository, MetadataSource, PageRequest, Person, PersonId, ProviderMappingRepository,
    ProviderMappingStatus, ProviderSubject, Result, SortKey, SortKeyKind, Studio, StudioId, Tag,
    TagId, TaruError,
};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogHydrationSummary {
    pub item_id: MediaItemId,
    pub people: u64,
    pub credits: u64,
    pub genres: u64,
    pub tags: u64,
    pub collections: u64,
    pub studios: u64,
    pub images: u64,
    pub search_indexed: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CatalogLabelHydrationSelection {
    pub genres: bool,
    pub tags: bool,
}

impl CatalogLabelHydrationSelection {
    #[must_use]
    pub const fn any(self) -> bool {
        self.genres || self.tags
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct CatalogHydrationSnapshot {
    pub item: MediaItem,
    pub sources: Vec<taru_core::MediaSource>,
    pub credits: Vec<(ItemCredit, Person)>,
    pub genres: Vec<(ItemGenre, Genre)>,
    pub tags: Vec<(ItemTag, Tag)>,
    pub collections: Vec<(CollectionItem, Collection)>,
    pub studios: Vec<(ItemStudio, Studio)>,
    pub provider_subjects: Vec<ProviderSubject>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct CatalogHydrationLookup {
    pub person_external_id_matches: Vec<(ExternalId, Person)>,
    pub person_name_matches: Vec<(String, Person)>,
    pub genre_name_source_matches: Vec<(String, MetadataSource, Genre)>,
    pub tag_name_source_matches: Vec<(String, MetadataSource, Tag)>,
    pub collection_external_id_matches: Vec<(ExternalId, Collection)>,
    pub collection_name_source_matches: Vec<(String, MetadataSource, Collection)>,
    pub studio_external_id_matches: Vec<(ExternalId, Studio)>,
    pub studio_name_source_matches: Vec<(String, MetadataSource, Studio)>,
    pub image_source_matches: Vec<(ImageOwner, ImageKind, String, ImageAsset)>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct CatalogHydrationCommit {
    pub replacement: CatalogItemGraphReplacement,
    pub search_projection: CatalogSearchProjection,
}

#[async_trait]
pub trait CatalogHydrationPort: Send + Sync {
    async fn hydrate_catalog(
        &self,
        item_id: MediaItemId,
        source: MetadataSource,
    ) -> Result<CatalogHydrationSummary>;
}

#[async_trait]
impl<T> CatalogHydrationPort for T
where
    T: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    async fn hydrate_catalog(
        &self,
        item_id: MediaItemId,
        source: MetadataSource,
    ) -> Result<CatalogHydrationSummary> {
        hydrate_item_catalog_with_repository(self, item_id, source).await
    }
}

async fn load_hydration_snapshot<R>(
    repository: &R,
    item_id: MediaItemId,
) -> Result<CatalogHydrationSnapshot>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let item = repository
        .get_media_item(item_id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "media_item",
            id: item_id.to_string(),
        })?;
    let sources = list_all_item_sources(repository, item.id).await?;
    let mut credits = Vec::new();
    for credit in repository.list_item_credits(item.id).await? {
        let person = repository
            .get_person(credit.person_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "person",
                id: credit.person_id.to_string(),
            })?;
        credits.push((credit, person));
    }

    let mut genres = Vec::new();
    for item_genre in repository.list_item_genres(item.id).await? {
        let genre = repository
            .get_genre(item_genre.genre_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "genre",
                id: item_genre.genre_id.to_string(),
            })?;
        genres.push((item_genre, genre));
    }

    let mut tags = Vec::new();
    for item_tag in repository.list_item_tags(item.id).await? {
        let tag =
            repository
                .get_tag(item_tag.tag_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "tag",
                    id: item_tag.tag_id.to_string(),
                })?;
        tags.push((item_tag, tag));
    }

    let mut collections = Vec::new();
    for collection_item in repository.list_item_collections(item.id).await? {
        let collection = repository
            .get_collection(collection_item.collection_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "collection",
                id: collection_item.collection_id.to_string(),
            })?;
        collections.push((collection_item, collection));
    }

    let mut studios = Vec::new();
    for item_studio in repository.list_item_studios(item.id).await? {
        let studio = repository
            .get_studio(item_studio.studio_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "studio",
                id: item_studio.studio_id.to_string(),
            })?;
        studios.push((item_studio, studio));
    }
    let provider_subjects = list_accepted_provider_subjects(repository, item.id).await?;

    Ok(CatalogHydrationSnapshot {
        item,
        sources,
        credits,
        genres,
        tags,
        collections,
        studios,
        provider_subjects,
    })
}

async fn load_hydration_lookup<R>(
    repository: &R,
    item: &MediaItem,
    source: &MetadataSource,
) -> Result<CatalogHydrationLookup>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let mut lookup = CatalogHydrationLookup {
        person_external_id_matches: Vec::new(),
        person_name_matches: Vec::new(),
        genre_name_source_matches: Vec::new(),
        tag_name_source_matches: Vec::new(),
        collection_external_id_matches: Vec::new(),
        collection_name_source_matches: Vec::new(),
        studio_external_id_matches: Vec::new(),
        studio_name_source_matches: Vec::new(),
        image_source_matches: Vec::new(),
    };

    for credit in &item.metadata.credits {
        if let Some(name) = normalized_label(&credit.name) {
            if let Some(person) = repository.find_person_by_name(&name).await? {
                lookup.person_name_matches.push((name, person));
            }
        }
        for external_id in non_empty_external_ids(&credit.external_ids) {
            if let Some(person) = repository.find_person_by_external_id(external_id).await? {
                lookup
                    .person_external_id_matches
                    .push((external_id.clone(), person));
            }
        }
    }

    for name in normalized_unique_labels(&item.metadata.genres) {
        if let Some(genre) = repository.find_genre_by_name_source(&name, source).await? {
            lookup
                .genre_name_source_matches
                .push((name, source.clone(), genre));
        }
    }
    for name in normalized_unique_labels(&item.metadata.tags) {
        if let Some(tag) = repository.find_tag_by_name_source(&name, source).await? {
            lookup
                .tag_name_source_matches
                .push((name, source.clone(), tag));
        }
    }

    for collection_ref in &item.metadata.collections {
        if let Some(name) = normalized_label(&collection_ref.name) {
            if let Some(collection) = repository
                .find_collection_by_name_source(&name, source)
                .await?
            {
                lookup
                    .collection_name_source_matches
                    .push((name, source.clone(), collection));
            }
        }
        for external_id in non_empty_external_ids(&collection_ref.external_ids) {
            if let Some(collection) = repository
                .find_collection_by_external_id(external_id)
                .await?
            {
                lookup
                    .collection_external_id_matches
                    .push((external_id.clone(), collection));
            }
        }
    }

    for studio_ref in &item.metadata.studios {
        if let Some(name) = normalized_label(&studio_ref.name) {
            if let Some(studio) = repository.find_studio_by_name_source(&name, source).await? {
                lookup
                    .studio_name_source_matches
                    .push((name, source.clone(), studio));
            }
        }
        for external_id in non_empty_external_ids(&studio_ref.external_ids) {
            if let Some(studio) = repository.find_studio_by_external_id(external_id).await? {
                lookup
                    .studio_external_id_matches
                    .push((external_id.clone(), studio));
            }
        }
    }

    let owner = ImageOwner::Item(item.id);
    for image_ref in &item.metadata.images {
        let Some(source_uri) = normalized_label(&image_ref.uri) else {
            continue;
        };
        if let Some(image) = repository
            .find_image_asset_by_source(&owner, &image_ref.kind, &source_uri)
            .await?
        {
            lookup.image_source_matches.push((
                owner.clone(),
                image_ref.kind.clone(),
                source_uri,
                image,
            ));
        }
    }

    Ok(lookup)
}

async fn commit_hydration<R>(repository: &R, commit: CatalogHydrationCommit) -> Result<()>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    repository
        .commit_item_projection(&CatalogItemProjectionCommit {
            graph: commit.replacement,
            search: commit.search_projection,
        })
        .await
}

pub async fn hydrate_item_catalog<R>(
    port: &R,
    item_id: MediaItemId,
    source: MetadataSource,
) -> Result<CatalogHydrationSummary>
where
    R: CatalogHydrationPort,
{
    port.hydrate_catalog(item_id, source).await
}

pub async fn plan_item_catalog_projection<R>(
    repository: &R,
    item: MediaItem,
    source: MetadataSource,
) -> Result<CatalogItemProjectionCommit>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let mut snapshot = load_hydration_snapshot(repository, item.id).await?;
    snapshot.item = item;
    let lookup = load_hydration_lookup(repository, &snapshot.item, &source).await?;
    let mut replacement = CatalogItemGraphReplacement::default();
    let mut summary = CatalogHydrationSummary {
        item_id: snapshot.item.id,
        ..CatalogHydrationSummary::default()
    };

    hydrate_credits(&lookup, &snapshot.item, &mut summary, &mut replacement)?;
    hydrate_genres(
        &lookup,
        &snapshot.item,
        &source,
        &mut summary,
        &mut replacement,
    )?;
    hydrate_tags(
        &lookup,
        &snapshot.item,
        &source,
        &mut summary,
        &mut replacement,
    )?;
    hydrate_collections(
        &lookup,
        &snapshot.item,
        &source,
        &mut summary,
        &mut replacement,
    )?;
    hydrate_studios(
        &lookup,
        &snapshot.item,
        &source,
        &mut summary,
        &mut replacement,
    )?;
    hydrate_images(&lookup, &snapshot.item, &mut summary, &mut replacement)?;
    let search = search_projection_from_graph(&snapshot.item, &snapshot, &replacement);

    Ok(CatalogItemProjectionCommit {
        graph: replacement,
        search,
    })
}

pub async fn refresh_item_search<R>(
    repository: &R,
    item_id: MediaItemId,
) -> Result<CatalogHydrationSummary>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let snapshot = load_hydration_snapshot(repository, item_id).await?;
    let item = snapshot.item.clone();
    let replacement = replacement_from_snapshot(&snapshot);
    let search_projection = plan_item_search_projection_from_snapshot(snapshot, item);
    repository
        .upsert_search_projection(&search_projection)
        .await?;

    Ok(CatalogHydrationSummary {
        item_id,
        people: replacement.people.len() as u64,
        credits: replacement.credits.len() as u64,
        genres: replacement.genres.len() as u64,
        tags: replacement.tags.len() as u64,
        collections: replacement.collections.len() as u64,
        studios: replacement.studios.len() as u64,
        images: 0,
        search_indexed: true,
    })
}

pub async fn plan_item_search_projection<R>(
    repository: &R,
    item: MediaItem,
) -> Result<CatalogSearchProjection>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let snapshot = load_hydration_snapshot(repository, item.id).await?;
    Ok(plan_item_search_projection_from_snapshot(snapshot, item))
}

pub async fn plan_item_catalog_label_projection<R>(
    repository: &R,
    item: MediaItem,
    source: MetadataSource,
    selection: CatalogLabelHydrationSelection,
) -> Result<CatalogItemProjectionCommit>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let mut snapshot = load_hydration_snapshot(repository, item.id).await?;
    snapshot.item = item;
    let lookup = load_hydration_lookup(repository, &snapshot.item, &source).await?;
    let mut replacement = replacement_from_snapshot(&snapshot);
    let mut unused_summary = CatalogHydrationSummary {
        item_id: snapshot.item.id,
        ..CatalogHydrationSummary::default()
    };

    if selection.genres {
        replacement.genres.clear();
        replacement.item_genres.clear();
        hydrate_genres(
            &lookup,
            &snapshot.item,
            &source,
            &mut unused_summary,
            &mut replacement,
        )?;
    }

    if selection.tags {
        replacement.tags.clear();
        replacement.item_tags.clear();
        hydrate_tags(
            &lookup,
            &snapshot.item,
            &source,
            &mut unused_summary,
            &mut replacement,
        )?;
    }

    let search = search_projection_from_graph(&snapshot.item, &snapshot, &replacement);
    Ok(CatalogItemProjectionCommit {
        graph: replacement,
        search,
    })
}

pub async fn hydrate_item_catalog_labels<R>(
    repository: &R,
    item_id: MediaItemId,
    source: MetadataSource,
    selection: CatalogLabelHydrationSelection,
) -> Result<CatalogHydrationSummary>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let snapshot = load_hydration_snapshot(repository, item_id).await?;
    let commit =
        plan_item_catalog_label_projection(repository, snapshot.item.clone(), source, selection)
            .await?;
    let mut summary = summary_from_replacement(item_id, &commit.graph);
    commit_hydration(
        repository,
        CatalogHydrationCommit {
            replacement: commit.graph,
            search_projection: commit.search,
        },
    )
    .await?;

    summary.search_indexed = true;
    Ok(summary)
}

fn plan_item_search_projection_from_snapshot(
    mut snapshot: CatalogHydrationSnapshot,
    item: MediaItem,
) -> CatalogSearchProjection {
    snapshot.item = item;
    let replacement = replacement_from_snapshot(&snapshot);
    search_projection_from_graph(&snapshot.item, &snapshot, &replacement)
}

async fn hydrate_item_catalog_with_repository<R>(
    repository: &R,
    item_id: MediaItemId,
    source: MetadataSource,
) -> Result<CatalogHydrationSummary>
where
    R: CatalogRepository + MediaRepository + ProviderMappingRepository,
{
    let snapshot = load_hydration_snapshot(repository, item_id).await?;
    let lookup = load_hydration_lookup(repository, &snapshot.item, &source).await?;
    let item = &snapshot.item;
    let mut summary = CatalogHydrationSummary {
        item_id,
        ..CatalogHydrationSummary::default()
    };

    let mut replacement = CatalogItemGraphReplacement::default();

    hydrate_credits(&lookup, &item, &mut summary, &mut replacement)?;
    hydrate_genres(&lookup, &item, &source, &mut summary, &mut replacement)?;
    hydrate_tags(&lookup, &item, &source, &mut summary, &mut replacement)?;
    hydrate_collections(&lookup, &item, &source, &mut summary, &mut replacement)?;
    hydrate_studios(&lookup, &item, &source, &mut summary, &mut replacement)?;
    hydrate_images(&lookup, &item, &mut summary, &mut replacement)?;
    let search_projection = search_projection_from_graph(item, &snapshot, &replacement);
    commit_hydration(
        repository,
        CatalogHydrationCommit {
            replacement,
            search_projection,
        },
    )
    .await?;
    summary.search_indexed = true;

    Ok(summary)
}

fn replacement_from_snapshot(snapshot: &CatalogHydrationSnapshot) -> CatalogItemGraphReplacement {
    let mut replacement = CatalogItemGraphReplacement::default();
    let mut seen_people = HashSet::new();

    for (credit, person) in &snapshot.credits {
        if seen_people.insert(person.id) {
            replacement.people.push(person.clone());
        }
        replacement.credits.push(credit.clone());
    }

    for (item_genre, genre) in &snapshot.genres {
        replacement.genres.push(genre.clone());
        replacement.item_genres.push(item_genre.clone());
    }

    for (item_tag, tag) in &snapshot.tags {
        replacement.tags.push(tag.clone());
        replacement.item_tags.push(item_tag.clone());
    }

    for (collection_item, collection) in &snapshot.collections {
        replacement.collections.push(collection.clone());
        replacement.collection_items.push(collection_item.clone());
    }

    for (item_studio, studio) in &snapshot.studios {
        replacement.studios.push(studio.clone());
        replacement.item_studios.push(item_studio.clone());
    }

    replacement
}

fn summary_from_replacement(
    item_id: MediaItemId,
    replacement: &CatalogItemGraphReplacement,
) -> CatalogHydrationSummary {
    CatalogHydrationSummary {
        item_id,
        people: replacement.people.len() as u64,
        credits: replacement.credits.len() as u64,
        genres: replacement.genres.len() as u64,
        tags: replacement.tags.len() as u64,
        collections: replacement.collections.len() as u64,
        studios: replacement.studios.len() as u64,
        images: replacement.images.len() as u64,
        search_indexed: false,
    }
}

fn hydrate_credits(
    lookup: &CatalogHydrationLookup,
    item: &MediaItem,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()> {
    for credit in &item.metadata.credits {
        let Some(name) = normalized_label(&credit.name) else {
            continue;
        };
        let person = resolve_person(lookup, credit, name);
        replacement.people.push(person.clone());
        replacement.credits.push(ItemCredit {
            item_id: item.id,
            person_id: person.id,
            role: credit.role.clone(),
            character: credit.character.as_deref().and_then(normalized_label),
            sort_order: credit.order,
        });
        summary.people += 1;
        summary.credits += 1;
    }

    Ok(())
}

fn hydrate_genres(
    lookup: &CatalogHydrationLookup,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()> {
    let mut seen = HashSet::new();

    for name in
        item.metadata.genres.iter().filter_map(|name| {
            normalized_label(name).filter(|name| seen.insert(name.to_lowercase()))
        })
    {
        let genre = lookup
            .genre_name_source_matches
            .iter()
            .find(|(existing_name, existing_source, _genre)| {
                existing_name == &name && existing_source == source
            })
            .map(|(_name, _source, genre)| genre.clone())
            .unwrap_or_else(|| Genre {
                id: GenreId::new(),
                name,
                source: source.clone(),
            });
        replacement.genres.push(genre.clone());
        replacement.item_genres.push(ItemGenre {
            item_id: item.id,
            genre_id: genre.id,
        });
        summary.genres += 1;
    }

    Ok(())
}

fn hydrate_tags(
    lookup: &CatalogHydrationLookup,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()> {
    let mut seen = HashSet::new();

    for name in
        item.metadata.tags.iter().filter_map(|name| {
            normalized_label(name).filter(|name| seen.insert(name.to_lowercase()))
        })
    {
        let tag = lookup
            .tag_name_source_matches
            .iter()
            .find(|(existing_name, existing_source, _tag)| {
                existing_name == &name && existing_source == source
            })
            .map(|(_name, _source, tag)| tag.clone())
            .unwrap_or_else(|| Tag {
                id: TagId::new(),
                name,
                source: source.clone(),
            });
        replacement.tags.push(tag.clone());
        replacement.item_tags.push(ItemTag {
            item_id: item.id,
            tag_id: tag.id,
        });
        summary.tags += 1;
    }

    Ok(())
}

fn hydrate_collections(
    lookup: &CatalogHydrationLookup,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()> {
    for collection_ref in &item.metadata.collections {
        let Some(name) = normalized_label(&collection_ref.name) else {
            continue;
        };
        let collection = resolve_collection(lookup, collection_ref, name, source);
        replacement.collections.push(collection.clone());
        replacement.collection_items.push(CollectionItem {
            collection_id: collection.id,
            item_id: item.id,
            sort_order: collection_ref.sort_order,
        });
        summary.collections += 1;
    }

    Ok(())
}

fn hydrate_studios(
    lookup: &CatalogHydrationLookup,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()> {
    let mut seen = HashSet::new();

    for studio_ref in item.metadata.studios.iter().filter(|studio| {
        normalized_label(&studio.name).is_some_and(|name| seen.insert(name.to_lowercase()))
    }) {
        let Some(name) = normalized_label(&studio_ref.name) else {
            continue;
        };
        let studio = resolve_studio(lookup, studio_ref, name, source);
        replacement.studios.push(studio.clone());
        replacement.item_studios.push(ItemStudio {
            item_id: item.id,
            studio_id: studio.id,
        });
        summary.studios += 1;
    }

    Ok(())
}

fn hydrate_images(
    lookup: &CatalogHydrationLookup,
    item: &MediaItem,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()> {
    let owner = ImageOwner::Item(item.id);
    let mut selected_kinds = HashSet::new();

    for image_ref in &item.metadata.images {
        let Some(source_uri) = normalized_label(&image_ref.uri) else {
            continue;
        };
        let existing = lookup
            .image_source_matches
            .iter()
            .find(|(existing_owner, existing_kind, existing_source, _image)| {
                existing_owner == &owner
                    && existing_kind == &image_ref.kind
                    && existing_source == &source_uri
            })
            .map(|(_owner, _kind, _source_uri, image)| image.clone());
        let selected = selected_kinds.insert(image_kind_key(&image_ref.kind));
        let image = ImageAsset {
            id: existing
                .as_ref()
                .map(|image| image.id)
                .unwrap_or_else(ImageAssetId::new),
            owner: owner.clone(),
            kind: image_ref.kind.clone(),
            source_uri,
            provider: image_ref.provider.clone(),
            cache_uri: existing.as_ref().and_then(|image| image.cache_uri.clone()),
            width: image_ref.width,
            height: image_ref.height,
            language: image_ref.language.clone(),
            selected: existing.as_ref().is_some_and(|image| image.selected) || selected,
            content_hash: existing
                .as_ref()
                .and_then(|image| image.content_hash.clone()),
            etag: existing.as_ref().and_then(|image| image.etag.clone()),
        };

        replacement.images.push(image);
        summary.images += 1;
    }

    Ok(())
}

fn resolve_person(lookup: &CatalogHydrationLookup, credit: &Credit, name: String) -> Person {
    for external_id in non_empty_external_ids(&credit.external_ids) {
        if let Some(mut person) = lookup
            .person_external_id_matches
            .iter()
            .find(|(existing_external_id, _person)| existing_external_id == external_id)
            .map(|(_external_id, person)| person.clone())
        {
            merge_external_ids(&mut person.external_ids, &credit.external_ids);
            return person;
        }
    }

    if let Some(mut person) = lookup
        .person_name_matches
        .iter()
        .find(|(existing_name, _person)| existing_name == &name)
        .map(|(_name, person)| person.clone())
    {
        merge_external_ids(&mut person.external_ids, &credit.external_ids);
        return person;
    }

    Person {
        id: PersonId::new(),
        name,
        sort_name: None,
        overview: None,
        external_ids: credit.external_ids.clone(),
    }
}

fn resolve_collection(
    lookup: &CatalogHydrationLookup,
    collection_ref: &CollectionRef,
    name: String,
    source: &MetadataSource,
) -> Collection {
    for external_id in non_empty_external_ids(&collection_ref.external_ids) {
        if let Some(mut collection) = lookup
            .collection_external_id_matches
            .iter()
            .find(|(existing_external_id, _collection)| existing_external_id == external_id)
            .map(|(_external_id, collection)| collection.clone())
        {
            if collection.overview.is_none() {
                collection.overview = collection_ref.overview.clone();
            }
            merge_external_ids(&mut collection.external_ids, &collection_ref.external_ids);
            return collection;
        }
    }

    if let Some(mut collection) = lookup
        .collection_name_source_matches
        .iter()
        .find(|(existing_name, existing_source, _collection)| {
            existing_name == &name && existing_source == source
        })
        .map(|(_name, _source, collection)| collection.clone())
    {
        if collection.overview.is_none() {
            collection.overview = collection_ref.overview.clone();
        }
        merge_external_ids(&mut collection.external_ids, &collection_ref.external_ids);
        return collection;
    }

    Collection {
        id: CollectionId::new(),
        name,
        overview: collection_ref.overview.clone(),
        source: source.clone(),
        external_ids: collection_ref.external_ids.clone(),
    }
}

fn resolve_studio(
    lookup: &CatalogHydrationLookup,
    studio_ref: &taru_core::StudioRef,
    name: String,
    source: &MetadataSource,
) -> Studio {
    for external_id in non_empty_external_ids(&studio_ref.external_ids) {
        if let Some(mut studio) = lookup
            .studio_external_id_matches
            .iter()
            .find(|(existing_external_id, _studio)| existing_external_id == external_id)
            .map(|(_external_id, studio)| studio.clone())
        {
            merge_external_ids(&mut studio.external_ids, &studio_ref.external_ids);
            return studio;
        }
    }

    if let Some(mut studio) = lookup
        .studio_name_source_matches
        .iter()
        .find(|(existing_name, existing_source, _studio)| {
            existing_name == &name && existing_source == source
        })
        .map(|(_name, _source, studio)| studio.clone())
    {
        merge_external_ids(&mut studio.external_ids, &studio_ref.external_ids);
        return studio;
    }

    Studio {
        id: StudioId::new(),
        name,
        source: source.clone(),
        external_ids: studio_ref.external_ids.clone(),
    }
}

fn search_projection_from_graph(
    item: &MediaItem,
    snapshot: &CatalogHydrationSnapshot,
    replacement: &CatalogItemGraphReplacement,
) -> CatalogSearchProjection {
    let mut body_parts = Vec::new();
    let mut aliases = Vec::new();
    let mut browse_facets = Vec::new();
    let mut sort_keys = Vec::new();

    add_search_item_metadata(
        item,
        &mut body_parts,
        &mut aliases,
        &mut browse_facets,
        &mut sort_keys,
    );

    for source in &snapshot.sources {
        push_body(&mut body_parts, &source.file_name);
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(BrowseFacetKind::Source, source.file_name.clone()),
        );
    }

    for genre in &replacement.genres {
        push_body(&mut body_parts, &genre.name);
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(BrowseFacetKind::Genre, genre.name.clone()),
        );
    }

    for tag in &replacement.tags {
        push_body(&mut body_parts, &tag.name);
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(BrowseFacetKind::Tag, tag.name.clone()),
        );
    }

    for collection in &replacement.collections {
        push_body(&mut body_parts, &collection.name);
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(BrowseFacetKind::Collection, collection.name.clone()),
        );
    }

    for studio in &replacement.studios {
        push_body(&mut body_parts, &studio.name);
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(BrowseFacetKind::Studio, studio.name.clone()),
        );
    }

    for credit in &replacement.credits {
        if let Some(person) = replacement
            .people
            .iter()
            .find(|person| person.id == credit.person_id)
        {
            push_body(&mut body_parts, &person.name);
            push_optional_body(&mut body_parts, credit.character.as_deref());
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::Credit, person.name.clone()),
            );
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(credit_role_facet_kind(&credit.role), person.name.clone()),
            );
        }
    }

    for subject in &snapshot.provider_subjects {
        push_body(&mut body_parts, &subject.subject_key);
        if let Some(title) = subject.title.as_deref() {
            push_body(&mut body_parts, title);
            push_unique_string(&mut aliases, title.to_owned());
        }
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(BrowseFacetKind::Provider, provider_label(&subject.provider)),
        );
        push_unique_facet(
            &mut browse_facets,
            BrowseFacet::new(
                BrowseFacetKind::ExternalId(provider_label(&subject.provider)),
                subject.subject_key.clone(),
            ),
        );
        if let Some(year) = subject.release_year {
            push_unique_facet(
                &mut browse_facets,
                BrowseFacet::new(BrowseFacetKind::ReleaseYear, year.to_string()),
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
    projection
}

fn add_search_item_metadata(
    item: &MediaItem,
    body_parts: &mut Vec<String>,
    aliases: &mut Vec<String>,
    browse_facets: &mut Vec<BrowseFacet>,
    sort_keys: &mut Vec<SortKey>,
) {
    push_body(body_parts, &item.metadata.title);
    if let Some(original_title) = item.metadata.original_title.as_deref() {
        push_body(body_parts, original_title);
        push_unique_string(aliases, original_title.to_owned());
    }
    if let Some(sort_title) = item.metadata.sort_title.as_deref() {
        push_body(body_parts, sort_title);
        sort_keys.push(SortKey::new(SortKeyKind::SortTitle, sort_title.to_owned()));
    } else if !item.metadata.title.trim().is_empty() {
        sort_keys.push(SortKey::new(
            SortKeyKind::Title,
            item.metadata.title.clone(),
        ));
    }
    push_optional_body(body_parts, item.metadata.overview.as_deref());
    push_optional_body(body_parts, item.metadata.tagline.as_deref());
    push_unique_facet(
        browse_facets,
        BrowseFacet::new(BrowseFacetKind::Kind, item.kind.as_str()),
    );

    if let Some(value) = item.metadata.release_date.as_deref() {
        sort_keys.push(SortKey::new(SortKeyKind::ReleaseDate, value.to_owned()));
        if let Some(year) = value
            .get(0..4)
            .filter(|year| year.chars().all(|character| character.is_ascii_digit()))
        {
            push_unique_facet(
                browse_facets,
                BrowseFacet::new(BrowseFacetKind::ReleaseYear, year.to_owned()),
            );
        }
    }

    for external_id in &item.metadata.external_ids {
        push_body(body_parts, &external_id.value);
        push_unique_facet(
            browse_facets,
            BrowseFacet::new(
                BrowseFacetKind::Provider,
                provider_label(&external_id.provider),
            ),
        );
        push_unique_facet(
            browse_facets,
            BrowseFacet::new(
                BrowseFacetKind::ExternalId(provider_label(&external_id.provider)),
                external_id.value.clone(),
            ),
        );
    }
}

async fn list_all_item_sources<R>(
    repository: &R,
    item_id: MediaItemId,
) -> Result<Vec<taru_core::MediaSource>>
where
    R: MediaRepository,
{
    let mut offset = 0;
    let mut sources = Vec::new();

    loop {
        let page = repository
            .list_item_sources(
                item_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
        let returned = page.len();
        sources.extend(page);

        if returned < PageRequest::MAX_LIMIT as usize {
            break;
        }

        offset += u64::from(PageRequest::MAX_LIMIT);
    }

    Ok(sources)
}

fn normalized_label(value: &str) -> Option<String> {
    let value = value.trim();

    (!value.is_empty()).then(|| value.to_owned())
}

fn non_empty_external_ids(external_ids: &[ExternalId]) -> impl Iterator<Item = &ExternalId> {
    external_ids
        .iter()
        .filter(|external_id| !external_id.value.trim().is_empty())
}

fn normalized_unique_labels(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .iter()
        .filter_map(|value| {
            normalized_label(value).filter(|value| seen.insert(value.to_lowercase()))
        })
        .collect()
}

fn merge_external_ids(existing: &mut Vec<ExternalId>, incoming: &[ExternalId]) {
    for external_id in incoming {
        if !external_id.value.trim().is_empty() && !existing.contains(external_id) {
            existing.push(external_id.clone());
        }
    }
}

async fn list_accepted_provider_subjects<R>(
    repository: &R,
    item_id: MediaItemId,
) -> Result<Vec<ProviderSubject>>
where
    R: ProviderMappingRepository,
{
    let mut offset = 0;
    let mut subjects = Vec::new();

    loop {
        let mappings = repository
            .list_provider_mappings_for_item(
                item_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
        let count = mappings.len();
        for mapping in mappings {
            if mapping.status != ProviderMappingStatus::Accepted {
                continue;
            }
            let subject = repository
                .get_provider_subject(mapping.subject_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "provider_subject",
                    id: mapping.subject_id.to_string(),
                })?;
            subjects.push(subject);
        }
        if count < PageRequest::MAX_LIMIT as usize {
            break;
        }
        offset += u64::from(PageRequest::MAX_LIMIT);
    }

    Ok(subjects)
}

fn push_body(parts: &mut Vec<String>, value: &str) {
    if let Some(value) = normalized_label(value) {
        parts.push(value);
    }
}

fn push_optional_body(parts: &mut Vec<String>, value: Option<&str>) {
    if let Some(value) = value {
        push_body(parts, value);
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

fn credit_role_facet_kind(role: &CreditRole) -> BrowseFacetKind {
    match role {
        CreditRole::Actor => BrowseFacetKind::Actor,
        CreditRole::Director => BrowseFacetKind::Director,
        CreditRole::Writer => BrowseFacetKind::Writer,
        CreditRole::Producer => BrowseFacetKind::Producer,
        CreditRole::Creator => BrowseFacetKind::Creator,
        CreditRole::Other(value) => BrowseFacetKind::CreditRole(value.clone()),
    }
}

fn image_kind_key(kind: &ImageKind) -> String {
    match kind {
        ImageKind::Poster => "poster".to_owned(),
        ImageKind::Backdrop => "backdrop".to_owned(),
        ImageKind::Logo => "logo".to_owned(),
        ImageKind::Thumbnail => "thumbnail".to_owned(),
        ImageKind::Banner => "banner".to_owned(),
        ImageKind::Other(value) => format!("other:{value}"),
    }
}

fn provider_label(provider: &ExternalProvider) -> String {
    match provider {
        ExternalProvider::Tmdb => "tmdb".to_owned(),
        ExternalProvider::Douban => "douban".to_owned(),
        ExternalProvider::Bangumi => "bangumi".to_owned(),
        ExternalProvider::Imdb => "imdb".to_owned(),
        ExternalProvider::Local => "local".to_owned(),
        ExternalProvider::Other(value) => format!("other:{value}"),
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use taru_core::{
        CanonicalMetadata, Credit, CreditRole, DatabaseLifecycle, ExternalId, ExternalProvider,
        ImageKind, ImageRef, Library, LibraryId, LibraryOptions, LibraryPreset, MediaItem,
        MediaKind, MediaRepository, MediaSource, MediaSourceId, MetadataSource, ProviderMapping,
        ProviderMappingId, ProviderMappingStatus, ProviderSubject, ProviderSubjectId,
        ProviderSubjectKind,
        repository::{CatalogRepository, LibraryRepository, ProviderMappingRepository},
    };
    use taru_db::TaruDatabase;
    use taru_search::{SearchIndex, SearchQuery};

    use super::*;

    #[derive(Debug)]
    struct FakeCatalogHydrationPort {
        expected_item_id: MediaItemId,
        expected_source: MetadataSource,
        summary: CatalogHydrationSummary,
        requests: std::sync::Mutex<Vec<(MediaItemId, MetadataSource)>>,
    }

    #[async_trait]
    impl CatalogHydrationPort for FakeCatalogHydrationPort {
        async fn hydrate_catalog(
            &self,
            item_id: MediaItemId,
            source: MetadataSource,
        ) -> Result<CatalogHydrationSummary> {
            assert_eq!(self.expected_item_id, item_id);
            assert_eq!(self.expected_source, source);
            self.requests.lock().unwrap().push((item_id, source));
            Ok(self.summary.clone())
        }
    }

    #[tokio::test]
    async fn hydration_uses_workflow_port_without_sqlite() {
        let item_id = MediaItemId::new();
        let source = MetadataSource::Provider(ExternalProvider::Tmdb);
        let port = FakeCatalogHydrationPort {
            expected_item_id: item_id,
            expected_source: source.clone(),
            summary: CatalogHydrationSummary {
                item_id,
                people: 1,
                credits: 1,
                genres: 1,
                tags: 1,
                collections: 0,
                studios: 0,
                images: 1,
                search_indexed: true,
            },
            requests: std::sync::Mutex::new(Vec::new()),
        };

        let summary = hydrate_item_catalog(&port, item_id, source.clone())
            .await
            .unwrap();
        let requests = port.requests.lock().unwrap().clone();

        assert_eq!(summary.item_id, item_id);
        assert_eq!(summary.credits, 1);
        assert_eq!(summary.genres, 1);
        assert_eq!(summary.images, 1);
        assert_eq!(requests, vec![(item_id, source)]);
    }

    #[tokio::test]
    async fn hydration_populates_graph_and_search_projection() {
        let store = TaruDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                overview: Some("A hacker discovers reality.".to_owned()),
                genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                tags: vec!["cyberpunk".to_owned()],
                credits: vec![Credit {
                    name: "Keanu Reeves".to_owned(),
                    role: CreditRole::Actor,
                    character: Some("Neo".to_owned()),
                    order: Some(0),
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "6384".to_owned(),
                    }],
                }],
                images: vec![ImageRef {
                    kind: ImageKind::Poster,
                    uri: "https://image.example/poster.jpg".to_owned(),
                    provider: ExternalProvider::Tmdb,
                    width: Some(1000),
                    height: Some(1500),
                    language: Some("en".to_owned()),
                }],
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "603".to_owned(),
                }],
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: item.id,
            locator: "local:///Movies/The Matrix (1999).mkv".to_owned(),
            file_name: "The Matrix (1999).mkv".to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_media_source(&source).await.unwrap();

        let summary = hydrate_item_catalog(
            &store,
            item.id,
            MetadataSource::Provider(ExternalProvider::Tmdb),
        )
        .await
        .unwrap();
        let people = store.list_people(PageRequest::first_page()).await.unwrap();
        let genres = store.list_genres(PageRequest::first_page()).await.unwrap();
        let tags = store.list_tags(PageRequest::first_page()).await.unwrap();
        let images = store.list_item_images(item.id).await.unwrap();
        let hits = store
            .search(
                SearchQuery::from_facet_labels(
                    "keanu",
                    vec!["genre:Science Fiction".to_owned()],
                    10,
                    0,
                )
                .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(summary.credits, 1);
        assert_eq!(people[0].name, "Keanu Reeves");
        assert_eq!(genres.len(), 2);
        assert_eq!(tags[0].name, "cyberpunk");
        assert_eq!(images.len(), 1);
        assert_eq!(hits[0].item_id, item.id);
    }

    #[tokio::test]
    async fn hydration_keeps_same_locator_sources_isolated_by_item_and_library() {
        let store = TaruDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let first_library = Library {
            id: LibraryId::new(),
            name: "First Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let second_library = Library {
            id: LibraryId::new(),
            name: "Second Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let first_item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Shared Movie First".to_owned(),
                genres: vec!["First Genre".to_owned()],
                ..CanonicalMetadata::default()
            },
        };
        let second_item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Shared Movie Second".to_owned(),
                genres: vec!["Second Genre".to_owned()],
                ..CanonicalMetadata::default()
            },
        };
        let first_source = MediaSource {
            id: MediaSourceId::new(),
            library_id: first_library.id,
            item_id: first_item.id,
            locator: "local:///Movie.mkv".to_owned(),
            file_name: "Movie.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };
        let second_source = MediaSource {
            id: MediaSourceId::new(),
            library_id: second_library.id,
            item_id: second_item.id,
            locator: "local:///Movie.mkv".to_owned(),
            file_name: "Movie.mkv".to_owned(),
            size_bytes: Some(6),
            fingerprint: None,
        };

        store.upsert_library(&first_library).await.unwrap();
        store.upsert_library(&second_library).await.unwrap();
        store.upsert_media_item(&first_item).await.unwrap();
        store.upsert_media_item(&second_item).await.unwrap();
        store.upsert_media_source(&first_source).await.unwrap();
        store.upsert_media_source(&second_source).await.unwrap();

        hydrate_item_catalog(&store, first_item.id, MetadataSource::Local)
            .await
            .unwrap();
        hydrate_item_catalog(&store, second_item.id, MetadataSource::Local)
            .await
            .unwrap();

        let first_sources = store
            .list_media_sources(first_library.id, PageRequest::first_page())
            .await
            .unwrap();
        let second_sources = store
            .list_media_sources(second_library.id, PageRequest::first_page())
            .await
            .unwrap();
        let first_genres = store.list_item_genres(first_item.id).await.unwrap();
        let second_genres = store.list_item_genres(second_item.id).await.unwrap();
        let first_hits = store
            .search(
                SearchQuery::from_facet_labels(
                    "shared",
                    vec!["genre:First Genre".to_owned()],
                    10,
                    0,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        let second_hits = store
            .search(
                SearchQuery::from_facet_labels(
                    "shared",
                    vec!["genre:Second Genre".to_owned()],
                    10,
                    0,
                )
                .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(first_sources[0].locator, "local:///Movie.mkv");
        assert_eq!(second_sources[0].locator, "local:///Movie.mkv");
        assert_eq!(first_sources[0].item_id, first_item.id);
        assert_eq!(second_sources[0].item_id, second_item.id);
        assert_ne!(first_sources[0].id, second_sources[0].id);
        assert_ne!(first_genres[0].genre_id, second_genres[0].genre_id);
        assert_eq!(first_hits.len(), 1);
        assert_eq!(first_hits[0].item_id, first_item.id);
        assert_eq!(second_hits.len(), 1);
        assert_eq!(second_hits[0].item_id, second_item.id);
    }

    #[tokio::test]
    async fn hydration_builds_semantic_search_projection() {
        let store = TaruDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Semantic Search Fixture".to_owned(),
                original_title: Some("Original Semantic Title".to_owned()),
                sort_title: Some("Semantic Search Fixture".to_owned()),
                release_date: Some("1999-03-31".to_owned()),
                genres: vec!["Science Fiction".to_owned()],
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "603".to_owned(),
                }],
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: item.id,
            locator: "local:///Movies/Semantic.mkv".to_owned(),
            file_name: "Semantic.mkv".to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        let subject = ProviderSubject {
            id: ProviderSubjectId::new(),
            provider: ExternalProvider::Bangumi,
            subject_kind: ProviderSubjectKind::Subject,
            subject_key: "bangumi-265".to_owned(),
            title: Some("千と千尋の神隠し".to_owned()),
            release_year: Some(2001),
            locale: Some("ja-JP".to_owned()),
        };
        let mapping = ProviderMapping {
            id: ProviderMappingId::new(),
            item_id: item.id,
            subject_id: subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: Some(980),
            source: MetadataSource::Provider(ExternalProvider::Bangumi),
        };

        store.upsert_media_source(&source).await.unwrap();
        store.upsert_provider_subject(&subject).await.unwrap();
        store.upsert_provider_mapping(&mapping).await.unwrap();

        hydrate_item_catalog(&store, item.id, MetadataSource::Local)
            .await
            .unwrap();

        let by_alias = store
            .search(
                SearchQuery::from_facet_labels(
                    "original semantic",
                    vec!["release_year:1999".to_owned()],
                    10,
                    0,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        let by_external_id = store
            .search(
                SearchQuery::from_facet_labels(
                    "fixture",
                    vec!["external_id:tmdb:603".to_owned()],
                    10,
                    0,
                )
                .unwrap(),
            )
            .await
            .unwrap();
        let by_provider_title = store
            .search(
                SearchQuery::from_facet_labels("千 尋", vec!["provider:bangumi".to_owned()], 10, 0)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(by_alias[0].item_id, item.id);
        assert_eq!(by_external_id[0].item_id, item.id);
        assert_eq!(by_provider_title[0].item_id, item.id);
    }
}
