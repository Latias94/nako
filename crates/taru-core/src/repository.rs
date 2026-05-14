use async_trait::async_trait;

use crate::{
    Job, JobId, Library, LibraryId, MediaItem, MediaItemId, MediaProbeResult, MediaSource,
    MediaSourceId, NewJob, Result,
};

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn migrate(&self) -> Result<()>;
}

#[async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn upsert_library(&self, library: &Library) -> Result<()>;

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>>;

    async fn list_libraries(&self) -> Result<Vec<Library>>;
}

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn list_media_items(&self) -> Result<Vec<MediaItem>>;

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

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job>;

    async fn start_job(&self, id: JobId) -> Result<Job>;

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job>;

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job>;

    async fn get_job(&self, id: JobId) -> Result<Option<Job>>;
}
