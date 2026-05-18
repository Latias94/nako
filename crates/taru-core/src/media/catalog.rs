use serde::{Deserialize, Serialize};

use crate::{CollectionId, GenreId, ImageAssetId, MediaItemId, PersonId, StudioId, TagId};

use super::{
    item::{CreditRole, ImageKind},
    metadata::MetadataSource,
    provider::{ExternalId, ExternalProvider},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    pub sort_name: Option<String>,
    pub overview: Option<String>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemCredit {
    pub item_id: MediaItemId,
    pub person_id: PersonId,
    pub role: CreditRole,
    pub character: Option<String>,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Genre {
    pub id: GenreId,
    pub name: String,
    pub source: MetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemGenre {
    pub item_id: MediaItemId,
    pub genre_id: GenreId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub source: MetadataSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemTag {
    pub item_id: MediaItemId,
    pub tag_id: TagId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub overview: Option<String>,
    pub source: MetadataSource,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionItem {
    pub collection_id: CollectionId,
    pub item_id: MediaItemId,
    pub sort_order: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Studio {
    pub id: StudioId,
    pub name: String,
    pub source: MetadataSource,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemStudio {
    pub item_id: MediaItemId,
    pub studio_id: StudioId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImageAsset {
    pub id: ImageAssetId,
    pub owner: ImageOwner,
    pub kind: ImageKind,
    pub source_uri: String,
    pub provider: ExternalProvider,
    pub cache_uri: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub language: Option<String>,
    pub selected: bool,
    pub content_hash: Option<String>,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogItemGraphReplacement {
    pub people: Vec<Person>,
    pub credits: Vec<ItemCredit>,
    pub genres: Vec<Genre>,
    pub item_genres: Vec<ItemGenre>,
    pub tags: Vec<Tag>,
    pub item_tags: Vec<ItemTag>,
    pub collections: Vec<Collection>,
    pub collection_items: Vec<CollectionItem>,
    pub studios: Vec<Studio>,
    pub item_studios: Vec<ItemStudio>,
    pub images: Vec<ImageAsset>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogSearchProjection {
    pub item_id: MediaItemId,
    pub title: String,
    pub body: String,
    pub facets: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogItemProjectionCommit {
    pub graph: CatalogItemGraphReplacement,
    pub search: CatalogSearchProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageOwner {
    Item(MediaItemId),
    Person(PersonId),
    Collection(CollectionId),
    Studio(StudioId),
}
