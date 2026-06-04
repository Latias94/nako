use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use nako_core::{
    NakoError, NewVfsCacheFailure, Result, StorageErrorKind, VfsCacheOperation, VfsCacheRepository,
    VfsCachedListing, VfsCachedObject, VfsCachedObjectKind,
};

use crate::{
    ByteRange, ObjectCacheState, ObjectCacheStatus, ObjectKind, ObjectListing, ObjectMetadata,
    ReadRange, ReadStream, StageRequest, StagedFile, StorageApplyReport, StorageApplyRequest,
    StorageBackend, StorageCapabilities, StorageCleanupReport, StorageCleanupRequest,
    StorageLinkPlan, StorageLinkPlanRequest, StorageUri, StorageWriteReport, StorageWriteRequest,
    VfsCacheRefreshReport, VirtualFile,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VfsCacheOptions {
    pub stat_ttl_ms: i64,
    pub list_ttl_ms: i64,
    pub serve_stale_on_error: bool,
    pub cache_local: bool,
}

impl Default for VfsCacheOptions {
    fn default() -> Self {
        Self {
            stat_ttl_ms: 300_000,
            list_ttl_ms: 120_000,
            serve_stale_on_error: true,
            cache_local: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CachedStorageBackend<B, C> {
    inner: B,
    cache: C,
    options: VfsCacheOptions,
}

impl<B, C> CachedStorageBackend<B, C> {
    pub fn new(inner: B, cache: C) -> Self {
        Self::with_options(inner, cache, VfsCacheOptions::default())
    }

    pub fn with_options(inner: B, cache: C, options: VfsCacheOptions) -> Self {
        Self {
            inner,
            cache,
            options,
        }
    }

    #[must_use]
    pub fn inner(&self) -> &B {
        &self.inner
    }

    #[must_use]
    pub fn cache(&self) -> &C {
        &self.cache
    }
}

#[async_trait]
impl<B, C> StorageBackend for CachedStorageBackend<B, C>
where
    B: StorageBackend,
    C: VfsCacheRepository,
{
    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        if !self.should_cache(uri) {
            return self.inner.stat(uri).await;
        }

        let observed_at_ms = now_ms()?;
        let cached = self.cached_object(uri).await?;
        if let Some(record) = cached
            .as_ref()
            .filter(|record| record.is_fresh_at(observed_at_ms))
        {
            return metadata_from_cached(
                record.clone(),
                ObjectCacheStatus {
                    state: ObjectCacheState::Fresh,
                    fetched_at_ms: record.fetched_at_ms,
                    fresh_until_ms: record.fresh_until_ms,
                    last_failed_at_ms: None,
                    last_error: None,
                },
            );
        }

        match self.inner.stat(uri).await {
            Ok(mut metadata) => {
                let observed_at_ms = now_ms()?;
                let record = cached_object_from_metadata(
                    &metadata,
                    observed_at_ms,
                    fresh_until_ms(observed_at_ms, self.options.stat_ttl_ms)?,
                );
                self.cache.upsert_vfs_cache_object(&record).await?;
                metadata.cache = Some(ObjectCacheStatus {
                    state: ObjectCacheState::Fresh,
                    fetched_at_ms: record.fetched_at_ms,
                    fresh_until_ms: record.fresh_until_ms,
                    last_failed_at_ms: None,
                    last_error: None,
                });
                Ok(metadata)
            }
            Err(err) => {
                let failure = self
                    .record_failure(uri, VfsCacheOperation::Stat, &err, observed_at_ms)
                    .await?;
                if self.options.serve_stale_on_error && is_transient_storage_error(&err) {
                    if let Some(record) = cached {
                        return metadata_from_cached(
                            record.clone(),
                            ObjectCacheStatus {
                                state: ObjectCacheState::StaleFallback,
                                fetched_at_ms: record.fetched_at_ms,
                                fresh_until_ms: record.fresh_until_ms,
                                last_failed_at_ms: Some(failure.failed_at_ms),
                                last_error: Some(failure.error),
                            },
                        );
                    }
                }
                Err(err)
            }
        }
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(self.list_with_status(uri).await?.entries)
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
        if !self.should_cache(uri) {
            return self.inner.list_with_status(uri).await;
        }

        let observed_at_ms = now_ms()?;
        let cached = self.cached_listing(uri).await?;
        if let Some(listing) = cached
            .as_ref()
            .filter(|listing| listing.is_fresh_at(observed_at_ms))
        {
            return listing_from_cached(
                listing.clone(),
                ObjectCacheStatus {
                    state: ObjectCacheState::Fresh,
                    fetched_at_ms: listing.fetched_at_ms,
                    fresh_until_ms: listing.fresh_until_ms,
                    last_failed_at_ms: None,
                    last_error: None,
                },
            );
        }

        match self.inner.list(uri).await {
            Ok(entries) => {
                let observed_at_ms = now_ms()?;
                let directory = self
                    .cached_or_fetched_directory(uri, observed_at_ms)
                    .await?;
                let entry_records = entries
                    .iter()
                    .map(|entry| {
                        Ok(cached_object_from_metadata(
                            entry,
                            observed_at_ms,
                            fresh_until_ms(observed_at_ms, self.options.stat_ttl_ms)?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let listing = VfsCachedListing {
                    directory,
                    entries: entry_records,
                    fetched_at_ms: observed_at_ms,
                    fresh_until_ms: fresh_until_ms(observed_at_ms, self.options.list_ttl_ms)?,
                };
                self.cache.upsert_vfs_cache_listing(&listing).await?;

                Ok(ObjectListing {
                    entries,
                    cache: Some(ObjectCacheStatus {
                        state: ObjectCacheState::Fresh,
                        fetched_at_ms: listing.fetched_at_ms,
                        fresh_until_ms: listing.fresh_until_ms,
                        last_failed_at_ms: None,
                        last_error: None,
                    }),
                })
            }
            Err(err) => {
                let failure = self
                    .record_failure(uri, VfsCacheOperation::List, &err, observed_at_ms)
                    .await?;
                if self.options.serve_stale_on_error && is_transient_storage_error(&err) {
                    if let Some(listing) = cached {
                        return listing_from_cached(
                            listing.clone(),
                            ObjectCacheStatus {
                                state: ObjectCacheState::StaleFallback,
                                fetched_at_ms: listing.fetched_at_ms,
                                fresh_until_ms: listing.fresh_until_ms,
                                last_failed_at_ms: Some(failure.failed_at_ms),
                                last_error: Some(failure.error),
                            },
                        );
                    }
                }
                Err(err)
            }
        }
    }

    async fn refresh_cache(
        &self,
        uri: &StorageUri,
        operation: VfsCacheOperation,
    ) -> Result<VfsCacheRefreshReport> {
        let cache = match operation {
            VfsCacheOperation::Stat => self.refresh_stat_cache(uri).await?,
            VfsCacheOperation::List => self.refresh_listing_cache(uri).await?,
        };

        Ok(VfsCacheRefreshReport { operation, cache })
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        self.inner.open_range(uri, range).await
    }

    async fn read_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadRange> {
        self.inner.read_range(uri, range).await
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        self.inner.stream_range(uri, range).await
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        self.inner.read_to_string(uri).await
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        self.inner.write_string(uri, content).await
    }

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        self.inner.write(request).await
    }

    async fn plan_link(&self, request: StorageLinkPlanRequest) -> Result<StorageLinkPlan> {
        self.inner.plan_link(request).await
    }

    async fn apply(&self, request: StorageApplyRequest) -> Result<StorageApplyReport> {
        self.inner.apply(request).await
    }

    async fn cleanup(&self, request: StorageCleanupRequest) -> Result<StorageCleanupReport> {
        self.inner.cleanup(request).await
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        self.inner.stage(request).await
    }
}

impl<B, C> CachedStorageBackend<B, C>
where
    C: VfsCacheRepository,
{
    fn should_cache(&self, uri: &StorageUri) -> bool {
        self.options.cache_local || uri.scheme() != "local"
    }

    async fn cached_object(&self, uri: &StorageUri) -> Result<Option<VfsCachedObject>> {
        for candidate in cache_lookup_uris(uri) {
            if let Some(record) = self.cache.get_vfs_cache_object(&candidate).await? {
                return Ok(Some(record));
            }
        }

        Ok(None)
    }

    async fn cached_listing(&self, uri: &StorageUri) -> Result<Option<VfsCachedListing>> {
        for candidate in cache_lookup_uris(uri) {
            if let Some(listing) = self.cache.get_vfs_cache_listing(&candidate).await? {
                return Ok(Some(listing));
            }
        }

        Ok(None)
    }

    async fn cached_or_fetched_directory(
        &self,
        uri: &StorageUri,
        now_ms: i64,
    ) -> Result<VfsCachedObject>
    where
        B: StorageBackend,
    {
        if let Some(record) = self.cached_object(uri).await? {
            return Ok(record);
        }

        let metadata = self.inner.stat(uri).await?;
        let record = cached_object_from_metadata(
            &metadata,
            now_ms,
            fresh_until_ms(now_ms, self.options.stat_ttl_ms)?,
        );
        self.cache.upsert_vfs_cache_object(&record).await?;
        Ok(record)
    }

    async fn record_failure(
        &self,
        uri: &StorageUri,
        operation: VfsCacheOperation,
        err: &NakoError,
        failed_at_ms: i64,
    ) -> Result<nako_core::VfsCacheFailure> {
        self.cache
            .record_vfs_cache_failure(NewVfsCacheFailure {
                uri: uri.as_str().to_owned(),
                scheme: uri.scheme().to_owned(),
                operation,
                failed_at_ms,
                error: safe_cache_failure_message(err),
            })
            .await
    }

    async fn refresh_stat_cache(&self, uri: &StorageUri) -> Result<Option<ObjectCacheStatus>>
    where
        B: StorageBackend,
    {
        if !self.should_cache(uri) {
            self.inner.stat(uri).await?;
            return Ok(None);
        }

        let observed_at_ms = now_ms()?;
        match self.inner.stat(uri).await {
            Ok(metadata) => {
                let observed_at_ms = now_ms()?;
                let record = cached_object_from_metadata(
                    &metadata,
                    observed_at_ms,
                    fresh_until_ms(observed_at_ms, self.options.stat_ttl_ms)?,
                );
                self.cache.upsert_vfs_cache_object(&record).await?;
                Ok(Some(ObjectCacheStatus {
                    state: ObjectCacheState::Fresh,
                    fetched_at_ms: record.fetched_at_ms,
                    fresh_until_ms: record.fresh_until_ms,
                    last_failed_at_ms: None,
                    last_error: None,
                }))
            }
            Err(err) => {
                self.record_failure(uri, VfsCacheOperation::Stat, &err, observed_at_ms)
                    .await?;
                Err(err)
            }
        }
    }

    async fn refresh_listing_cache(&self, uri: &StorageUri) -> Result<Option<ObjectCacheStatus>>
    where
        B: StorageBackend,
    {
        if !self.should_cache(uri) {
            self.inner.list_with_status(uri).await?;
            return Ok(None);
        }

        let observed_at_ms = now_ms()?;
        match self.inner.list(uri).await {
            Ok(entries) => {
                let observed_at_ms = now_ms()?;
                let directory = self
                    .cached_or_fetched_directory(uri, observed_at_ms)
                    .await?;
                let entry_records = entries
                    .iter()
                    .map(|entry| {
                        Ok(cached_object_from_metadata(
                            entry,
                            observed_at_ms,
                            fresh_until_ms(observed_at_ms, self.options.stat_ttl_ms)?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let listing = VfsCachedListing {
                    directory,
                    entries: entry_records,
                    fetched_at_ms: observed_at_ms,
                    fresh_until_ms: fresh_until_ms(observed_at_ms, self.options.list_ttl_ms)?,
                };
                self.cache.upsert_vfs_cache_listing(&listing).await?;

                Ok(Some(ObjectCacheStatus {
                    state: ObjectCacheState::Fresh,
                    fetched_at_ms: listing.fetched_at_ms,
                    fresh_until_ms: listing.fresh_until_ms,
                    last_failed_at_ms: None,
                    last_error: None,
                }))
            }
            Err(err) => {
                self.record_failure(uri, VfsCacheOperation::List, &err, observed_at_ms)
                    .await?;
                Err(err)
            }
        }
    }
}

fn cached_object_from_metadata(
    metadata: &ObjectMetadata,
    fetched_at_ms: i64,
    fresh_until_ms: i64,
) -> VfsCachedObject {
    VfsCachedObject {
        uri: metadata.uri.as_str().to_owned(),
        scheme: metadata.uri.scheme().to_owned(),
        kind: cached_kind(metadata.kind),
        len: metadata.len,
        modified_at: metadata.modified_at.clone(),
        etag: metadata.etag.clone(),
        fingerprint: metadata.fingerprint.clone(),
        capabilities_bits: metadata.capabilities.bits(),
        fetched_at_ms,
        fresh_until_ms,
    }
}

fn metadata_from_cached(
    record: VfsCachedObject,
    cache: ObjectCacheStatus,
) -> Result<ObjectMetadata> {
    Ok(ObjectMetadata {
        uri: StorageUri::parse(record.uri)?,
        kind: object_kind(record.kind),
        len: record.len,
        modified_at: record.modified_at,
        etag: record.etag,
        fingerprint: record.fingerprint,
        capabilities: StorageCapabilities::from_bits_truncate(record.capabilities_bits),
        cache: Some(cache),
    })
}

fn listing_from_cached(
    listing: VfsCachedListing,
    cache: ObjectCacheStatus,
) -> Result<ObjectListing> {
    let entries = listing
        .entries
        .into_iter()
        .map(|entry| metadata_from_cached(entry, cache.clone()))
        .collect::<Result<Vec<_>>>()?;

    Ok(ObjectListing {
        entries,
        cache: Some(cache),
    })
}

fn cached_kind(kind: ObjectKind) -> VfsCachedObjectKind {
    match kind {
        ObjectKind::File => VfsCachedObjectKind::File,
        ObjectKind::Directory => VfsCachedObjectKind::Directory,
        ObjectKind::Symlink => VfsCachedObjectKind::Symlink,
        ObjectKind::Other => VfsCachedObjectKind::Other,
    }
}

fn object_kind(kind: VfsCachedObjectKind) -> ObjectKind {
    match kind {
        VfsCachedObjectKind::File => ObjectKind::File,
        VfsCachedObjectKind::Directory => ObjectKind::Directory,
        VfsCachedObjectKind::Symlink => ObjectKind::Symlink,
        VfsCachedObjectKind::Other => ObjectKind::Other,
    }
}

fn cache_lookup_uris(uri: &StorageUri) -> Vec<String> {
    let mut candidates = vec![uri.as_str().to_owned()];
    if uri.path_part().trim_matches('/').is_empty() {
        return candidates;
    }

    if uri.as_str().ends_with('/') {
        candidates.push(uri.as_str().trim_end_matches('/').to_owned());
    } else {
        candidates.push(format!("{uri}/"));
    }
    candidates
}

fn fresh_until_ms(now_ms: i64, ttl_ms: i64) -> Result<i64> {
    now_ms.checked_add(ttl_ms.max(0)).ok_or_else(|| {
        NakoError::storage(
            "vfs-cache",
            StorageErrorKind::Unknown,
            "cache freshness timestamp overflowed",
        )
    })
}

fn now_ms() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            NakoError::storage(
                "vfs-cache",
                StorageErrorKind::Unknown,
                format!("system clock is before Unix epoch: {err}"),
            )
        })?;
    i64::try_from(duration.as_millis()).map_err(|err| {
        NakoError::storage(
            "vfs-cache",
            StorageErrorKind::Unknown,
            format!("system time does not fit cache timestamp: {err}"),
        )
    })
}

fn is_transient_storage_error(err: &NakoError) -> bool {
    matches!(
        err.storage_failure_class(),
        Some(
            nako_core::StorageFailureClass::Timeout
                | nako_core::StorageFailureClass::Unavailable
                | nako_core::StorageFailureClass::RateLimited
        )
    )
}

fn safe_cache_failure_message(err: &NakoError) -> String {
    err.safe_storage_message()
        .unwrap_or_else(|| "storage failure".to_owned())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use nako_core::{StorageFailureClass, VfsCacheFailure, VfsCacheSummary};

    use super::*;

    #[tokio::test]
    async fn cached_backend_reuses_fresh_stat_and_listing() {
        let inner = FakeBackend::new();
        let cache = MemoryVfsCache::default();
        let backend = CachedStorageBackend::with_options(
            inner.clone(),
            cache,
            VfsCacheOptions {
                stat_ttl_ms: 60_000,
                list_ttl_ms: 60_000,
                serve_stale_on_error: true,
                cache_local: true,
            },
        );
        let root = StorageUri::from_parts("mem", "Movies").unwrap();
        let movie = StorageUri::from_parts("mem", "Movies/Demo.mkv").unwrap();

        let first_stat = backend.stat(&movie).await.unwrap();
        let second_stat = backend.stat(&movie).await.unwrap();
        let first_listing = backend.list_with_status(&root).await.unwrap();
        let second_listing = backend.list_with_status(&root).await.unwrap();

        let first_stat_cache = first_stat.cache.unwrap();
        assert_eq!(first_stat_cache.state, ObjectCacheState::Fresh);
        let first_stat_diagnostic = first_stat_cache.repair_diagnostic();
        assert_eq!(
            first_stat_diagnostic.classification,
            crate::VfsCacheRepairClassification::Healthy
        );
        assert_eq!(
            first_stat_diagnostic.recommended_action,
            crate::VfsCacheRepairAction::None
        );
        assert_eq!(first_stat_diagnostic.failure_class, None);
        assert!(!first_stat_diagnostic.retryable);
        assert_eq!(first_stat_diagnostic.safe_message, None);

        assert_eq!(second_stat.cache.unwrap().state, ObjectCacheState::Fresh);
        assert_eq!(first_listing.cache.unwrap().state, ObjectCacheState::Fresh);
        assert_eq!(second_listing.cache.unwrap().state, ObjectCacheState::Fresh);
        assert_eq!(inner.stat_calls.load(Ordering::SeqCst), 2);
        assert_eq!(inner.list_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cached_backend_serves_stale_listing_after_transient_failure() {
        let inner = FakeBackend::new();
        let cache = MemoryVfsCache::default();
        let backend = CachedStorageBackend::with_options(
            inner.clone(),
            cache.clone(),
            VfsCacheOptions {
                stat_ttl_ms: 0,
                list_ttl_ms: 0,
                serve_stale_on_error: true,
                cache_local: true,
            },
        );
        let root = StorageUri::from_parts("mem", "Movies").unwrap();

        let fresh = backend.list_with_status(&root).await.unwrap();
        cache.expire_listing().await;
        inner.fail_list_with(StorageErrorKind::Network).await;
        let stale = backend.list_with_status(&root).await.unwrap();
        let failure = cache
            .get_vfs_cache_failure(root.as_str(), VfsCacheOperation::List)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fresh.cache.unwrap().state, ObjectCacheState::Fresh);
        let stale_cache = stale.cache.unwrap();
        assert_eq!(stale_cache.state, ObjectCacheState::StaleFallback);
        let stale_diagnostic = stale_cache.repair_diagnostic();
        assert_eq!(
            stale_diagnostic.classification,
            crate::VfsCacheRepairClassification::RepairableStaleFallback
        );
        assert_eq!(
            stale_diagnostic.recommended_action,
            crate::VfsCacheRepairAction::RefreshCache
        );
        assert_eq!(
            stale_diagnostic.failure_class,
            Some(StorageFailureClass::Unavailable)
        );
        assert!(stale_diagnostic.retryable);
        assert_eq!(
            stale_diagnostic.safe_message.as_deref(),
            Some("storage backend unavailable")
        );
        let stale_diagnostic_body = serde_json::to_string(&stale_diagnostic).unwrap();
        assert!(!stale_diagnostic_body.contains("Movies"));
        assert!(!stale_diagnostic_body.contains("Demo.mkv"));

        assert_eq!(stale.entries[0].uri.as_str(), "mem:///Movies/Demo.mkv");
        assert_eq!(failure.failure_count, 1);
        assert_eq!(failure.error, "storage backend unavailable");
    }

    #[tokio::test]
    async fn cached_backend_does_not_serve_stale_listing_after_non_transient_storage_failure() {
        let inner = FakeBackend::new();
        let cache = MemoryVfsCache::default();
        let backend = CachedStorageBackend::with_options(
            inner.clone(),
            cache.clone(),
            VfsCacheOptions {
                stat_ttl_ms: 0,
                list_ttl_ms: 0,
                serve_stale_on_error: true,
                cache_local: true,
            },
        );
        let root = StorageUri::from_parts("mem", "Movies").unwrap();

        let fresh = backend.list_with_status(&root).await.unwrap();
        cache.expire_listing().await;
        inner.fail_list_with(StorageErrorKind::Unauthorized).await;
        let err = backend.list_with_status(&root).await.unwrap_err();
        let failure = cache
            .get_vfs_cache_failure(root.as_str(), VfsCacheOperation::List)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fresh.cache.unwrap().state, ObjectCacheState::Fresh);
        assert!(matches!(
            err,
            NakoError::Storage {
                kind: StorageErrorKind::Unauthorized,
                ..
            }
        ));
        assert_eq!(failure.failure_count, 1);
        assert_eq!(failure.error, "storage permission failure");

        let failure_diagnostic = crate::VfsCacheRepairDiagnostic::from_failure(&failure);
        assert_eq!(
            failure_diagnostic.classification,
            crate::VfsCacheRepairClassification::OperatorActionRequired
        );
        assert_eq!(
            failure_diagnostic.recommended_action,
            crate::VfsCacheRepairAction::FixBackendConfiguration
        );
        assert_eq!(
            failure_diagnostic.failure_class,
            Some(StorageFailureClass::Permission)
        );
        assert!(!failure_diagnostic.retryable);
        assert_eq!(
            failure_diagnostic.safe_message.as_deref(),
            Some("storage permission failure")
        );
    }

    #[tokio::test]
    async fn cached_backend_refresh_cache_forces_fresh_stat_and_listing_updates() {
        let inner = FakeBackend::new();
        let cache = MemoryVfsCache::default();
        let backend = CachedStorageBackend::with_options(
            inner.clone(),
            cache,
            VfsCacheOptions {
                stat_ttl_ms: 60_000,
                list_ttl_ms: 60_000,
                serve_stale_on_error: true,
                cache_local: true,
            },
        );
        let root = StorageUri::from_parts("mem", "Movies").unwrap();
        let movie = StorageUri::from_parts("mem", "Movies/Demo.mkv").unwrap();

        backend.stat(&movie).await.unwrap();
        backend.stat(&movie).await.unwrap();
        let stat_refresh = backend
            .refresh_cache(&movie, VfsCacheOperation::Stat)
            .await
            .unwrap();
        backend.list_with_status(&root).await.unwrap();
        backend.list_with_status(&root).await.unwrap();
        let list_refresh = backend
            .refresh_cache(&root, VfsCacheOperation::List)
            .await
            .unwrap();

        assert_eq!(inner.stat_calls.load(Ordering::SeqCst), 3);
        assert_eq!(inner.list_calls.load(Ordering::SeqCst), 2);
        assert_eq!(stat_refresh.operation, VfsCacheOperation::Stat);
        assert_eq!(
            stat_refresh.cache.as_ref().map(|cache| cache.state),
            Some(ObjectCacheState::Fresh)
        );
        assert_eq!(list_refresh.operation, VfsCacheOperation::List);
        assert_eq!(
            list_refresh.cache.as_ref().map(|cache| cache.state),
            Some(ObjectCacheState::Fresh)
        );
    }

    #[tokio::test]
    async fn cached_backend_refresh_cache_failure_records_failure_without_stale_fallback() {
        let inner = FakeBackend::new();
        let cache = MemoryVfsCache::default();
        let backend = CachedStorageBackend::with_options(
            inner.clone(),
            cache.clone(),
            VfsCacheOptions {
                stat_ttl_ms: 0,
                list_ttl_ms: 0,
                serve_stale_on_error: true,
                cache_local: true,
            },
        );
        let root = StorageUri::from_parts("mem", "Movies").unwrap();

        backend.list_with_status(&root).await.unwrap();
        cache.expire_listing().await;
        inner.fail_list_with(StorageErrorKind::Network).await;
        let stale = backend.list_with_status(&root).await.unwrap();
        let err = backend
            .refresh_cache(&root, VfsCacheOperation::List)
            .await
            .unwrap_err();
        let failure = cache
            .get_vfs_cache_failure(root.as_str(), VfsCacheOperation::List)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(stale.cache.unwrap().state, ObjectCacheState::StaleFallback);
        assert!(matches!(
            err,
            NakoError::Storage {
                kind: StorageErrorKind::Network,
                ..
            }
        ));
        assert_eq!(failure.failure_count, 2);
        assert_eq!(failure.error, "storage backend unavailable");
        assert_eq!(inner.list_calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn vfs_cache_retryable_failure_recommends_cache_refresh() {
        let failure = VfsCacheFailure {
            uri: "webdav:///Movies/Remote/".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 1_000,
            failure_count: 2,
            error: "storage backend unavailable".to_owned(),
        };

        let diagnostic = crate::VfsCacheRepairDiagnostic::from_failure(&failure);

        assert_eq!(
            diagnostic.classification,
            crate::VfsCacheRepairClassification::RetryableRefreshFailure
        );
        assert_eq!(
            diagnostic.recommended_action,
            crate::VfsCacheRepairAction::RefreshCache
        );
        assert_eq!(
            diagnostic.failure_class,
            Some(StorageFailureClass::Unavailable)
        );
        assert!(diagnostic.retryable);
        assert_eq!(
            diagnostic.safe_message.as_deref(),
            Some("storage backend unavailable")
        );
    }

    #[test]
    fn vfs_cache_failure_diagnostic_redacts_unclassified_raw_failure_payloads() {
        let failure = VfsCacheFailure {
            uri: "webdav:///Movies/Private/Demo.mkv?token=secret".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 1_000,
            failure_count: 1,
            error: "token=secret failed at F:\\Media\\Private\\Demo.mkv".to_owned(),
        };

        let diagnostic = crate::VfsCacheRepairDiagnostic::from_failure(&failure);
        let body = serde_json::to_string(&diagnostic).unwrap();

        assert_eq!(
            diagnostic.classification,
            crate::VfsCacheRepairClassification::UnknownFailure
        );
        assert_eq!(
            diagnostic.recommended_action,
            crate::VfsCacheRepairAction::InspectFailure
        );
        assert_eq!(diagnostic.failure_class, None);
        assert_eq!(diagnostic.safe_message.as_deref(), Some("storage failure"));
        assert!(!body.contains("token"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("Demo.mkv"));
        assert!(!body.contains("F:\\Media"));
        assert!(!body.contains("webdav:///"));
    }

    #[derive(Clone)]
    struct FakeBackend {
        fail_stat_kind: Arc<tokio::sync::Mutex<Option<StorageErrorKind>>>,
        fail_list_kind: Arc<tokio::sync::Mutex<Option<StorageErrorKind>>>,
        stat_calls: Arc<AtomicUsize>,
        list_calls: Arc<AtomicUsize>,
    }

    impl FakeBackend {
        fn new() -> Self {
            Self {
                fail_stat_kind: Arc::new(tokio::sync::Mutex::new(None)),
                fail_list_kind: Arc::new(tokio::sync::Mutex::new(None)),
                stat_calls: Arc::new(AtomicUsize::new(0)),
                list_calls: Arc::new(AtomicUsize::new(0)),
            }
        }

        async fn fail_list_with(&self, kind: StorageErrorKind) {
            *self.fail_list_kind.lock().await = Some(kind);
        }

        fn metadata(&self, uri: &StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri: uri.clone(),
                kind,
                len: (kind == ObjectKind::File).then_some(4),
                modified_at: Some("100".to_owned()),
                etag: Some(uri.as_str().to_owned()),
                fingerprint: Some(format!("mem:{}", uri.as_str())),
                capabilities: StorageCapabilities::SEEKABLE
                    | StorageCapabilities::RANGE_READABLE
                    | StorageCapabilities::REMOTE_LATENCY,
                cache: None,
            }
        }
    }

    #[async_trait]
    impl StorageBackend for FakeBackend {
        fn scheme(&self) -> &'static str {
            "mem"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            self.stat_calls.fetch_add(1, Ordering::SeqCst);
            if let Some(kind) = *self.fail_stat_kind.lock().await {
                return Err(NakoError::storage(
                    uri.to_string(),
                    kind,
                    "temporary stat failure",
                ));
            }

            Ok(self.metadata(uri))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            self.list_calls.fetch_add(1, Ordering::SeqCst);
            if let Some(kind) = *self.fail_list_kind.lock().await {
                return Err(NakoError::storage(
                    uri.to_string(),
                    kind,
                    "temporary list failure",
                ));
            }

            Ok(vec![self.metadata(
                &StorageUri::from_parts("mem", "Movies/Demo.mkv").unwrap(),
            )])
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Ok(VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            Ok(ReadRange {
                uri: uri.clone(),
                range,
                bytes: b"nako".to_vec(),
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported("fake backend does not read text"))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported("fake backend does not write text"))
        }

        async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
            Ok(StagedFile {
                uri: request.uri,
                path: request.root.join("fake-stage.mkv"),
                len: Some(4),
                etag: Some("fake".to_owned()),
                fingerprint: Some("fake".to_owned()),
                reused: false,
            })
        }
    }

    #[derive(Clone, Default)]
    struct MemoryVfsCache {
        objects: Arc<tokio::sync::Mutex<HashMap<String, VfsCachedObject>>>,
        listing: Arc<tokio::sync::Mutex<Option<VfsCachedListing>>>,
        failure: Arc<tokio::sync::Mutex<Option<VfsCacheFailure>>>,
    }

    #[async_trait]
    impl VfsCacheRepository for MemoryVfsCache {
        async fn upsert_vfs_cache_object(&self, object: &VfsCachedObject) -> Result<()> {
            self.objects
                .lock()
                .await
                .insert(object.uri.clone(), object.clone());
            Ok(())
        }

        async fn upsert_vfs_cache_listing(&self, listing: &VfsCachedListing) -> Result<()> {
            let mut objects = self.objects.lock().await;
            objects.insert(listing.directory.uri.clone(), listing.directory.clone());
            for entry in &listing.entries {
                objects.insert(entry.uri.clone(), entry.clone());
            }
            *self.listing.lock().await = Some(listing.clone());
            Ok(())
        }

        async fn get_vfs_cache_object(&self, uri: &str) -> Result<Option<VfsCachedObject>> {
            Ok(self.objects.lock().await.get(uri).cloned())
        }

        async fn get_vfs_cache_listing(&self, uri: &str) -> Result<Option<VfsCachedListing>> {
            Ok(self
                .listing
                .lock()
                .await
                .as_ref()
                .filter(|listing| listing.directory.uri == uri)
                .cloned())
        }

        async fn record_vfs_cache_failure(
            &self,
            failure: NewVfsCacheFailure,
        ) -> Result<VfsCacheFailure> {
            let mut current = self.failure.lock().await;
            let failure_count = current
                .as_ref()
                .map(|failure| failure.failure_count + 1)
                .unwrap_or(1);
            let record = VfsCacheFailure {
                uri: failure.uri,
                scheme: failure.scheme,
                operation: failure.operation,
                failed_at_ms: failure.failed_at_ms,
                failure_count,
                error: failure.error,
            };
            *current = Some(record.clone());
            Ok(record)
        }

        async fn get_vfs_cache_failure(
            &self,
            uri: &str,
            operation: VfsCacheOperation,
        ) -> Result<Option<VfsCacheFailure>> {
            Ok(self
                .failure
                .lock()
                .await
                .as_ref()
                .filter(|failure| failure.uri == uri && failure.operation == operation)
                .cloned())
        }

        async fn get_latest_vfs_cache_failure(&self) -> Result<Option<VfsCacheFailure>> {
            Ok(self.failure.lock().await.clone())
        }

        async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
            let objects = self.objects.lock().await;
            let listing = self.listing.lock().await;
            let failure = self.failure.lock().await;

            Ok(VfsCacheSummary {
                object_count: objects.len() as u64,
                listing_count: u64::from(listing.is_some()),
                failure_count: u64::from(failure.is_some()),
                stale_object_count: objects
                    .values()
                    .filter(|object| !object.is_fresh_at(now_ms))
                    .count() as u64,
                stale_listing_count: listing
                    .as_ref()
                    .filter(|listing| !listing.is_fresh_at(now_ms))
                    .map_or(0, |_| 1),
                last_failure_at_ms: failure.as_ref().map(|failure| failure.failed_at_ms),
            })
        }
    }

    impl MemoryVfsCache {
        async fn expire_listing(&self) {
            if let Some(listing) = self.listing.lock().await.as_mut() {
                listing.fresh_until_ms = 0;
                listing.directory.fresh_until_ms = 0;
                for entry in &mut listing.entries {
                    entry.fresh_until_ms = 0;
                }
            }

            for object in self.objects.lock().await.values_mut() {
                object.fresh_until_ms = 0;
            }
        }
    }
}
