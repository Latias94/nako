use std::collections::HashSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{
    CatalogItemGraphReplacement, CatalogRepository, Collection, CollectionId, CollectionItem,
    CollectionRef, Credit, CreditRole, ExternalId, ExternalProvider, Genre, GenreId, ImageAsset,
    ImageAssetId, ImageKind, ImageOwner, ItemCredit, ItemGenre, ItemStudio, ItemTag, MediaItem,
    MediaItemId, MediaRepository, MetadataSource, PageRequest, Person, PersonId, Result, Studio,
    StudioId, Tag, TagId, TaruError,
};
use taru_search::{SearchDocument, SearchIndex};

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct CatalogHydrationSnapshot {
    pub item: MediaItem,
    pub sources: Vec<taru_core::MediaSource>,
    pub credits: Vec<(ItemCredit, Person)>,
    pub genres: Vec<(ItemGenre, Genre)>,
    pub tags: Vec<(ItemTag, Tag)>,
    pub collections: Vec<(CollectionItem, Collection)>,
    pub studios: Vec<(ItemStudio, Studio)>,
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
    pub item_id: MediaItemId,
    pub replacement: CatalogItemGraphReplacement,
    pub search_document: SearchDocument,
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
    T: CatalogRepository + MediaRepository + SearchIndex,
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
    R: CatalogRepository + MediaRepository + SearchIndex,
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

    Ok(CatalogHydrationSnapshot {
        item,
        sources,
        credits,
        genres,
        tags,
        collections,
        studios,
    })
}

async fn load_hydration_lookup<R>(
    repository: &R,
    item: &MediaItem,
    source: &MetadataSource,
) -> Result<CatalogHydrationLookup>
where
    R: CatalogRepository + MediaRepository + SearchIndex,
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
    R: CatalogRepository + MediaRepository + SearchIndex,
{
    repository
        .replace_item_catalog_graph(commit.item_id, &commit.replacement)
        .await?;
    repository.upsert(commit.search_document).await
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

async fn hydrate_item_catalog_with_repository<R>(
    repository: &R,
    item_id: MediaItemId,
    source: MetadataSource,
) -> Result<CatalogHydrationSummary>
where
    R: CatalogRepository + MediaRepository + SearchIndex,
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
    let search_document = search_document_from_graph(item, &snapshot, &replacement);
    commit_hydration(
        repository,
        CatalogHydrationCommit {
            item_id: item.id,
            replacement,
            search_document,
        },
    )
    .await?;
    summary.search_indexed = true;

    Ok(summary)
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

fn search_document_from_graph(
    item: &MediaItem,
    snapshot: &CatalogHydrationSnapshot,
    replacement: &CatalogItemGraphReplacement,
) -> SearchDocument {
    let mut body_parts = Vec::new();
    let mut facets = Vec::new();

    add_search_item_metadata(item, &mut body_parts, &mut facets);

    for source in &snapshot.sources {
        push_body(&mut body_parts, &source.file_name);
        push_unique_facet(&mut facets, format!("source:{}", source.file_name));
    }

    for genre in &replacement.genres {
        push_body(&mut body_parts, &genre.name);
        push_unique_facet(&mut facets, format!("genre:{}", genre.name));
    }

    for tag in &replacement.tags {
        push_body(&mut body_parts, &tag.name);
        push_unique_facet(&mut facets, format!("tag:{}", tag.name));
    }

    for collection in &replacement.collections {
        push_body(&mut body_parts, &collection.name);
        push_unique_facet(&mut facets, format!("collection:{}", collection.name));
    }

    for studio in &replacement.studios {
        push_body(&mut body_parts, &studio.name);
        push_unique_facet(&mut facets, format!("studio:{}", studio.name));
    }

    for credit in &replacement.credits {
        if let Some(person) = replacement
            .people
            .iter()
            .find(|person| person.id == credit.person_id)
        {
            let role = credit_role_label(&credit.role);
            push_body(&mut body_parts, &person.name);
            push_optional_body(&mut body_parts, credit.character.as_deref());
            push_unique_facet(&mut facets, format!("credit:{}", person.name));
            push_unique_facet(&mut facets, format!("{role}:{}", person.name));
        }
    }
    facets.sort();

    SearchDocument {
        item_id: item.id,
        title: item.metadata.title.clone(),
        body: body_parts.join(" "),
        facets,
    }
}

fn add_search_item_metadata(
    item: &MediaItem,
    body_parts: &mut Vec<String>,
    facets: &mut Vec<String>,
) {
    push_body(body_parts, &item.metadata.title);
    push_optional_body(body_parts, item.metadata.original_title.as_deref());
    push_optional_body(body_parts, item.metadata.sort_title.as_deref());
    push_optional_body(body_parts, item.metadata.overview.as_deref());
    push_optional_body(body_parts, item.metadata.tagline.as_deref());
    push_unique_facet(facets, format!("kind:{}", item.kind.as_str()));

    if let Some(value) = item.metadata.release_date.as_deref() {
        push_unique_facet(facets, format!("release_date:{value}"));
    }

    for external_id in &item.metadata.external_ids {
        push_body(body_parts, &external_id.value);
        push_unique_facet(
            facets,
            format!(
                "external_id:{}:{}",
                provider_label(&external_id.provider),
                external_id.value
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

fn push_unique_facet(facets: &mut Vec<String>, value: String) {
    if !facets.contains(&value) {
        facets.push(value);
    }
}

fn credit_role_label(role: &CreditRole) -> String {
    match role {
        CreditRole::Actor => "actor".to_owned(),
        CreditRole::Director => "director".to_owned(),
        CreditRole::Writer => "writer".to_owned(),
        CreditRole::Producer => "producer".to_owned(),
        CreditRole::Creator => "creator".to_owned(),
        CreditRole::Other(value) => format!("credit_role:{value}"),
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
        CanonicalMetadata, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind, ImageRef,
        Library, LibraryId, LibraryOptions, LibraryPreset, MediaItem, MediaKind, MediaRepository,
        MediaSource, MediaSourceId, MetadataSource, TransactionManager,
        repository::{CatalogRepository, LibraryRepository},
    };
    use taru_db::SqliteStore;
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
            .search(SearchQuery {
                query: "keanu".to_owned(),
                facets: vec!["genre:Science Fiction".to_owned()],
                limit: 10,
                offset: 0,
            })
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
            .search(SearchQuery {
                query: "shared".to_owned(),
                facets: vec!["genre:First Genre".to_owned()],
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();
        let second_hits = store
            .search(SearchQuery {
                query: "shared".to_owned(),
                facets: vec!["genre:Second Genre".to_owned()],
                limit: 10,
                offset: 0,
            })
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
}
