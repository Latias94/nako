use async_trait::async_trait;

use super::PageRequest;
use crate::{
    LibraryId, LibraryItemAddedAt, MediaItem, MediaItemId, MediaProbeResult, MediaSource,
    MediaSourceFingerprintMatch, MediaSourceId, Result,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibrarySourceInventoryEntry {
    pub source: MediaSource,
    pub item: Option<MediaItem>,
    pub probe: Option<MediaProbeResult>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MediaSourceFingerprintSummary {
    pub total_sources: u64,
    pub fingerprinted_sources: u64,
    pub content_hash_sources: u64,
}

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

    async fn list_library_source_inventory(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<LibrarySourceInventoryEntry>>;

    async fn list_media_sources_by_fingerprint(
        &self,
        library_id: LibraryId,
        fingerprint: &str,
        exclude_source_id: Option<MediaSourceId>,
        page: PageRequest,
    ) -> Result<Vec<MediaSourceFingerprintMatch>>;

    async fn summarize_media_source_fingerprints(&self) -> Result<MediaSourceFingerprintSummary>;
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
