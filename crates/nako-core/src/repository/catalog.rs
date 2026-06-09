use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AuthenticatedPrincipal, CatalogItemGraphReplacement, CatalogItemProjectionCommit,
    CatalogSearchProjection, Collection, CollectionId, CollectionItem, ExternalId, Genre, GenreId,
    ImageAsset, ImageAssetId, ImageKind, ImageOwner, ItemCredit, ItemGenre, ItemStudio, ItemTag,
    MediaItem, MediaItemId, MetadataSource, Person, PersonId, Result, Studio, StudioId, Tag, TagId,
};

#[async_trait]
pub trait CatalogRepository: Send + Sync {
    async fn replace_item_catalog_graph(
        &self,
        item_id: MediaItemId,
        replacement: &CatalogItemGraphReplacement,
    ) -> Result<()>;

    async fn commit_item_projection(&self, commit: &CatalogItemProjectionCommit) -> Result<()>;

    async fn upsert_search_projection(&self, projection: &CatalogSearchProjection) -> Result<()>;

    async fn upsert_person(&self, person: &Person) -> Result<()>;

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>>;

    async fn find_person_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Person>>;

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>>;

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>>;

    async fn list_accessible_people(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<Person>>;

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()>;

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>>;

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>>;

    async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn list_accessible_person_items(
        &self,
        principal: &AuthenticatedPrincipal,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn upsert_genre(&self, genre: &Genre) -> Result<()>;

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>>;

    async fn find_genre_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Genre>>;

    async fn list_genres(&self, page: PageRequest) -> Result<Vec<Genre>>;

    async fn list_accessible_genres(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<Genre>>;

    async fn upsert_item_genre(&self, item_genre: &ItemGenre) -> Result<()>;

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>>;

    async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn list_accessible_genre_items(
        &self,
        principal: &AuthenticatedPrincipal,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn upsert_tag(&self, tag: &Tag) -> Result<()>;

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>>;

    async fn find_tag_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Tag>>;

    async fn list_tags(&self, page: PageRequest) -> Result<Vec<Tag>>;

    async fn list_accessible_tags(
        &self,
        principal: &AuthenticatedPrincipal,
        page: PageRequest,
    ) -> Result<Vec<Tag>>;

    async fn upsert_item_tag(&self, item_tag: &ItemTag) -> Result<()>;

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>>;

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>>;

    async fn list_accessible_tag_items(
        &self,
        principal: &AuthenticatedPrincipal,
        tag_id: TagId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn upsert_collection(&self, collection: &Collection) -> Result<()>;

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>>;

    async fn find_collection_by_external_id(
        &self,
        external_id: &ExternalId,
    ) -> Result<Option<Collection>>;

    async fn find_collection_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Collection>>;

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>>;

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()>;

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>>;

    async fn list_collection_items(
        &self,
        collection_id: CollectionId,
    ) -> Result<Vec<CollectionItem>>;

    async fn upsert_studio(&self, studio: &Studio) -> Result<()>;

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>>;

    async fn find_studio_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Studio>>;

    async fn find_studio_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Studio>>;

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>>;

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()>;

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()>;

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>>;

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()>;

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>>;

    async fn find_image_asset_by_source(
        &self,
        owner: &ImageOwner,
        kind: &ImageKind,
        source_uri: &str,
    ) -> Result<Option<ImageAsset>>;

    async fn list_item_images(&self, item_id: MediaItemId) -> Result<Vec<ImageAsset>>;
}
