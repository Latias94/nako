use async_trait::async_trait;

use crate::{
    Library, LibraryId, MediaItem, MediaItemId, MediaProbeResult, MediaSource, MediaSourceId,
    Result,
};

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn migrate(&self) -> Result<()>;
}

#[async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn upsert_library(&self, library: &Library) -> Result<()>;

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>>;
}

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn upsert_media_source(&self, library_id: LibraryId, source: &MediaSource) -> Result<()>;

    async fn get_media_source_by_locator(&self, locator: &str) -> Result<Option<MediaSource>>;

    async fn list_media_sources(&self, library_id: LibraryId) -> Result<Vec<MediaSource>>;
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
