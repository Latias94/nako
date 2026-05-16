use std::collections::{BTreeSet, HashSet};

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

pub async fn hydrate_item_catalog<R>(
    repository: &R,
    item_id: MediaItemId,
    source: MetadataSource,
) -> Result<CatalogHydrationSummary>
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
    let mut summary = CatalogHydrationSummary {
        item_id,
        ..CatalogHydrationSummary::default()
    };

    let mut replacement = CatalogItemGraphReplacement::default();

    hydrate_credits(repository, &item, &mut summary, &mut replacement).await?;
    hydrate_genres(repository, &item, &source, &mut summary, &mut replacement).await?;
    hydrate_tags(repository, &item, &source, &mut summary, &mut replacement).await?;
    hydrate_collections(repository, &item, &source, &mut summary, &mut replacement).await?;
    hydrate_studios(repository, &item, &source, &mut summary, &mut replacement).await?;
    hydrate_images(repository, &item, &mut summary, &mut replacement).await?;
    repository
        .replace_item_catalog_graph(item.id, &replacement)
        .await?;
    rebuild_search_projection(repository, item.id).await?;
    summary.search_indexed = true;

    Ok(summary)
}

pub async fn rebuild_search_projection<R>(repository: &R, item_id: MediaItemId) -> Result<()>
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
    let credits = repository.list_item_credits(item.id).await?;
    let genres = repository.list_item_genres(item.id).await?;
    let tags = repository.list_item_tags(item.id).await?;
    let collections = repository.list_item_collections(item.id).await?;
    let studios = repository.list_item_studios(item.id).await?;
    let mut body_parts = Vec::new();
    let mut facets = BTreeSet::new();

    push_body(&mut body_parts, &item.metadata.title);
    push_optional_body(&mut body_parts, item.metadata.original_title.as_deref());
    push_optional_body(&mut body_parts, item.metadata.sort_title.as_deref());
    push_optional_body(&mut body_parts, item.metadata.overview.as_deref());
    push_optional_body(&mut body_parts, item.metadata.tagline.as_deref());
    facets.insert(format!("kind:{}", item.kind.as_str()));

    if let Some(value) = item.metadata.release_date.as_deref() {
        facets.insert(format!("release_date:{value}"));
    }

    for external_id in &item.metadata.external_ids {
        push_body(&mut body_parts, &external_id.value);
        facets.insert(format!(
            "external_id:{}:{}",
            provider_label(&external_id.provider),
            external_id.value
        ));
    }

    for source in sources {
        push_body(&mut body_parts, &source.file_name);
        facets.insert(format!("source:{}", source.file_name));
    }

    for item_genre in genres {
        if let Some(genre) = repository.get_genre(item_genre.genre_id).await? {
            push_body(&mut body_parts, &genre.name);
            facets.insert(format!("genre:{}", genre.name));
        }
    }

    for item_tag in tags {
        if let Some(tag) = repository.get_tag(item_tag.tag_id).await? {
            push_body(&mut body_parts, &tag.name);
            facets.insert(format!("tag:{}", tag.name));
        }
    }

    for membership in collections {
        if let Some(collection) = repository.get_collection(membership.collection_id).await? {
            push_body(&mut body_parts, &collection.name);
            facets.insert(format!("collection:{}", collection.name));
        }
    }

    for item_studio in studios {
        if let Some(studio) = repository.get_studio(item_studio.studio_id).await? {
            push_body(&mut body_parts, &studio.name);
            facets.insert(format!("studio:{}", studio.name));
        }
    }

    for credit in credits {
        if let Some(person) = repository.get_person(credit.person_id).await? {
            let role = credit_role_label(&credit.role);
            push_body(&mut body_parts, &person.name);
            push_optional_body(&mut body_parts, credit.character.as_deref());
            facets.insert(format!("credit:{}", person.name));
            facets.insert(format!("{role}:{}", person.name));
        }
    }

    repository
        .upsert(SearchDocument {
            item_id: item.id,
            title: item.metadata.title,
            body: body_parts.join(" "),
            facets: facets.into_iter().collect(),
        })
        .await
}

async fn hydrate_credits<R>(
    repository: &R,
    item: &MediaItem,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()>
where
    R: CatalogRepository,
{
    for credit in &item.metadata.credits {
        let Some(name) = normalized_label(&credit.name) else {
            continue;
        };
        let person = resolve_person(repository, credit, name).await?;
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

async fn hydrate_genres<R>(
    repository: &R,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()>
where
    R: CatalogRepository,
{
    let mut seen = HashSet::new();

    for name in
        item.metadata.genres.iter().filter_map(|name| {
            normalized_label(name).filter(|name| seen.insert(name.to_lowercase()))
        })
    {
        let genre = match repository.find_genre_by_name_source(&name, source).await? {
            Some(genre) => genre,
            None => Genre {
                id: GenreId::new(),
                name,
                source: source.clone(),
            },
        };
        replacement.genres.push(genre.clone());
        replacement.item_genres.push(ItemGenre {
            item_id: item.id,
            genre_id: genre.id,
        });
        summary.genres += 1;
    }

    Ok(())
}

async fn hydrate_tags<R>(
    repository: &R,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()>
where
    R: CatalogRepository,
{
    let mut seen = HashSet::new();

    for name in
        item.metadata.tags.iter().filter_map(|name| {
            normalized_label(name).filter(|name| seen.insert(name.to_lowercase()))
        })
    {
        let tag = match repository.find_tag_by_name_source(&name, source).await? {
            Some(tag) => tag,
            None => Tag {
                id: TagId::new(),
                name,
                source: source.clone(),
            },
        };
        replacement.tags.push(tag.clone());
        replacement.item_tags.push(ItemTag {
            item_id: item.id,
            tag_id: tag.id,
        });
        summary.tags += 1;
    }

    Ok(())
}

async fn hydrate_collections<R>(
    repository: &R,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()>
where
    R: CatalogRepository,
{
    for collection_ref in &item.metadata.collections {
        let Some(name) = normalized_label(&collection_ref.name) else {
            continue;
        };
        let collection = resolve_collection(repository, collection_ref, name, source).await?;
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

async fn hydrate_studios<R>(
    repository: &R,
    item: &MediaItem,
    source: &MetadataSource,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()>
where
    R: CatalogRepository,
{
    let mut seen = HashSet::new();

    for studio_ref in item.metadata.studios.iter().filter(|studio| {
        normalized_label(&studio.name).is_some_and(|name| seen.insert(name.to_lowercase()))
    }) {
        let Some(name) = normalized_label(&studio_ref.name) else {
            continue;
        };
        let studio = resolve_studio(repository, studio_ref, name, source).await?;
        replacement.studios.push(studio.clone());
        replacement.item_studios.push(ItemStudio {
            item_id: item.id,
            studio_id: studio.id,
        });
        summary.studios += 1;
    }

    Ok(())
}

async fn hydrate_images<R>(
    repository: &R,
    item: &MediaItem,
    summary: &mut CatalogHydrationSummary,
    replacement: &mut CatalogItemGraphReplacement,
) -> Result<()>
where
    R: CatalogRepository,
{
    let owner = ImageOwner::Item(item.id);
    let mut selected_kinds = HashSet::new();

    for image_ref in &item.metadata.images {
        let Some(source_uri) = normalized_label(&image_ref.uri) else {
            continue;
        };
        let existing = repository
            .find_image_asset_by_source(&owner, &image_ref.kind, &source_uri)
            .await?;
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

async fn resolve_person<R>(repository: &R, credit: &Credit, name: String) -> Result<Person>
where
    R: CatalogRepository,
{
    for external_id in non_empty_external_ids(&credit.external_ids) {
        if let Some(mut person) = repository.find_person_by_external_id(external_id).await? {
            merge_external_ids(&mut person.external_ids, &credit.external_ids);
            repository.upsert_person(&person).await?;
            return Ok(person);
        }
    }

    if let Some(mut person) = repository.find_person_by_name(&name).await? {
        merge_external_ids(&mut person.external_ids, &credit.external_ids);
        repository.upsert_person(&person).await?;
        return Ok(person);
    }

    let person = Person {
        id: PersonId::new(),
        name,
        sort_name: None,
        overview: None,
        external_ids: credit.external_ids.clone(),
    };
    repository.upsert_person(&person).await?;
    Ok(person)
}

async fn resolve_collection<R>(
    repository: &R,
    collection_ref: &CollectionRef,
    name: String,
    source: &MetadataSource,
) -> Result<Collection>
where
    R: CatalogRepository,
{
    for external_id in non_empty_external_ids(&collection_ref.external_ids) {
        if let Some(mut collection) = repository
            .find_collection_by_external_id(external_id)
            .await?
        {
            if collection.overview.is_none() {
                collection.overview = collection_ref.overview.clone();
            }
            merge_external_ids(&mut collection.external_ids, &collection_ref.external_ids);
            repository.upsert_collection(&collection).await?;
            return Ok(collection);
        }
    }

    if let Some(mut collection) = repository
        .find_collection_by_name_source(&name, source)
        .await?
    {
        if collection.overview.is_none() {
            collection.overview = collection_ref.overview.clone();
        }
        merge_external_ids(&mut collection.external_ids, &collection_ref.external_ids);
        repository.upsert_collection(&collection).await?;
        return Ok(collection);
    }

    let collection = Collection {
        id: CollectionId::new(),
        name,
        overview: collection_ref.overview.clone(),
        source: source.clone(),
        external_ids: collection_ref.external_ids.clone(),
    };
    repository.upsert_collection(&collection).await?;
    Ok(collection)
}

async fn resolve_studio<R>(
    repository: &R,
    studio_ref: &taru_core::StudioRef,
    name: String,
    source: &MetadataSource,
) -> Result<Studio>
where
    R: CatalogRepository,
{
    for external_id in non_empty_external_ids(&studio_ref.external_ids) {
        if let Some(mut studio) = repository.find_studio_by_external_id(external_id).await? {
            merge_external_ids(&mut studio.external_ids, &studio_ref.external_ids);
            repository.upsert_studio(&studio).await?;
            return Ok(studio);
        }
    }

    if let Some(mut studio) = repository.find_studio_by_name_source(&name, source).await? {
        merge_external_ids(&mut studio.external_ids, &studio_ref.external_ids);
        repository.upsert_studio(&studio).await?;
        return Ok(studio);
    }

    let studio = Studio {
        id: StudioId::new(),
        name,
        source: source.clone(),
        external_ids: studio_ref.external_ids.clone(),
    };
    repository.upsert_studio(&studio).await?;
    Ok(studio)
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
    use taru_core::{
        CanonicalMetadata, Credit, CreditRole, ExternalId, ExternalProvider, ImageKind, ImageRef,
        Library, LibraryId, LibraryOptions, LibraryPreset, MediaItem, MediaKind, MediaRepository,
        MediaSource, MediaSourceId, MetadataSource, TransactionManager,
        repository::{CatalogRepository, LibraryRepository},
    };
    use taru_db::SqliteStore;
    use taru_search::{SearchIndex, SearchQuery};

    use super::*;

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
