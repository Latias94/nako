use async_trait::async_trait;

use super::PageRequest;
use crate::{
    NewStagingManifestRecord, NewVfsCacheFailure, Result, StagingManifestId, StagingManifestRecord,
    StagingPurpose, StagingState, VfsCacheFailure, VfsCacheOperation, VfsCacheSummary,
    VfsCachedListing, VfsCachedObject,
};

#[async_trait]
pub trait VfsCacheRepository: Send + Sync {
    async fn upsert_vfs_cache_object(&self, object: &VfsCachedObject) -> Result<()>;

    async fn upsert_vfs_cache_listing(&self, listing: &VfsCachedListing) -> Result<()>;

    async fn get_vfs_cache_object(&self, uri: &str) -> Result<Option<VfsCachedObject>>;

    async fn get_vfs_cache_listing(&self, uri: &str) -> Result<Option<VfsCachedListing>>;

    async fn record_vfs_cache_failure(
        &self,
        failure: NewVfsCacheFailure,
    ) -> Result<VfsCacheFailure>;

    async fn get_vfs_cache_failure(
        &self,
        uri: &str,
        operation: VfsCacheOperation,
    ) -> Result<Option<VfsCacheFailure>>;

    async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary>;
}

#[async_trait]
pub trait StagingManifestRepository: Send + Sync {
    async fn upsert_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord>;

    async fn reserve_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
        max_total_bytes: u64,
        now_ms: i64,
    ) -> Result<StagingManifestRecord>;

    async fn start_staging_manifest_record(
        &self,
        id: StagingManifestId,
        started_at_ms: i64,
    ) -> Result<StagingManifestRecord>;

    async fn complete_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord>;

    async fn fail_staging_manifest_record(
        &self,
        id: StagingManifestId,
        failed_at_ms: i64,
        validation_error: String,
    ) -> Result<Option<StagingManifestRecord>>;

    async fn expire_staging_manifest_record(
        &self,
        id: StagingManifestId,
        expired_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>>;

    async fn mark_deleted_staging_manifest_record(
        &self,
        id: StagingManifestId,
        deleted_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>>;

    async fn acquire_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        leased_at_ms: i64,
    ) -> Result<StagingManifestRecord>;

    async fn release_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        released_at_ms: i64,
    ) -> Result<StagingManifestRecord>;

    async fn get_staging_manifest_record(
        &self,
        id: StagingManifestId,
    ) -> Result<Option<StagingManifestRecord>>;

    async fn find_staging_manifest_record_by_path(
        &self,
        local_path: &str,
    ) -> Result<Option<StagingManifestRecord>>;

    async fn list_staging_manifest_records(
        &self,
        purpose: Option<StagingPurpose>,
        state: Option<StagingState>,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>>;

    async fn list_staging_cleanup_candidates(
        &self,
        now_ms: i64,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>>;

    async fn touch_staging_manifest_record(
        &self,
        id: StagingManifestId,
        accessed_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>>;

    async fn delete_staging_manifest_record(&self, id: StagingManifestId) -> Result<()>;

    async fn sum_staging_manifest_bytes(&self) -> Result<u64>;
}
