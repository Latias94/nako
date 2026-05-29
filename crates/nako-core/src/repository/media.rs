use async_trait::async_trait;

use super::PageRequest;
use crate::{
    LibraryId, LibraryItemAddedAt, MediaItem, MediaItemId, MediaProbeResult, MediaSource,
    MediaSourceId, Result,
};

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>>;

    async fn list_media_items_for_library(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>>;

    async fn list_library_item_added_at(
        &self,
        library_id: LibraryId,
    ) -> Result<Vec<LibraryItemAddedAt>>;

    async fn upsert_media_source(&self, source: &MediaSource) -> Result<()>;

    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>>;

    async fn get_media_source_by_locator(
        &self,
        library_id: LibraryId,
        locator: &str,
    ) -> Result<Option<MediaSource>>;

    async fn list_item_sources(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>>;

    async fn list_media_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>>;
}

#[async_trait]
pub trait MediaProbeRepository: Send + Sync {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()>;

    async fn get_media_probe(&self, source_id: MediaSourceId) -> Result<Option<MediaProbeResult>>;
}
