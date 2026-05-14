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

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>>;
}

#[async_trait]
pub trait MediaRepository: Send + Sync {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>>;

    async fn upsert_media_source(&self, library_id: LibraryId, source: &MediaSource) -> Result<()>;

    async fn get_media_source_by_locator(&self, locator: &str) -> Result<Option<MediaSource>>;

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

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job>;

    async fn start_job(&self, id: JobId) -> Result<Job>;

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job>;

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job>;

    async fn get_job(&self, id: JobId) -> Result<Option<Job>>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub limit: u32,
    pub offset: u64,
}

impl PageRequest {
    pub const DEFAULT_LIMIT: u32 = 50;
    pub const MAX_LIMIT: u32 = 500;

    #[must_use]
    pub const fn new(limit: u32, offset: u64) -> Self {
        Self { limit, offset }
    }

    #[must_use]
    pub const fn first_page() -> Self {
        Self {
            limit: Self::DEFAULT_LIMIT,
            offset: 0,
        }
    }

    #[must_use]
    pub fn clamped(self) -> Self {
        let limit = if self.limit == 0 {
            Self::DEFAULT_LIMIT
        } else {
            self.limit.min(Self::MAX_LIMIT)
        };

        Self {
            limit,
            offset: self.offset,
        }
    }
}
