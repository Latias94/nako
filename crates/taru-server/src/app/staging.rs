use async_trait::async_trait;
use taru_core::{
    NewStagingManifestRecord, Result, StagingManifestId, StagingManifestRepository, StagingPurpose,
    StagingState,
};
use taru_db::SqliteStore;
use taru_vfs::{
    ByteRange, ObjectListing, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile,
    StorageBackend, StorageUri, VirtualFile,
};

use super::current_time_ms;

pub(super) async fn record_staged_input(
    store: &SqliteStore,
    purpose: StagingPurpose,
    uri: &StorageUri,
    staged: &StagedFile,
) -> Result<()> {
    let now_ms = current_time_ms()?;
    let local_path = staged.path.display().to_string();
    let existing = store
        .find_staging_manifest_record_by_path(&local_path)
        .await?;
    let id = existing
        .as_ref()
        .map(|record| record.id)
        .unwrap_or_else(StagingManifestId::new);
    let created_at_ms = existing
        .as_ref()
        .map(|record| record.created_at_ms)
        .unwrap_or(now_ms);

    store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id,
            source_uri: uri.to_string(),
            source_scheme: uri.scheme().to_owned(),
            purpose,
            local_path,
            size_bytes: staged.len,
            etag: staged.etag.clone(),
            fingerprint: staged.fingerprint.clone(),
            state: StagingState::Ready,
            created_at_ms,
            updated_at_ms: now_ms,
            last_accessed_at_ms: now_ms,
            expires_at_ms: None,
            active_leases: 0,
            validation_error: None,
        })
        .await?;

    Ok(())
}

pub(super) struct ManifestRecordingStorageBackend {
    inner: Box<dyn StorageBackend>,
    store: SqliteStore,
    purpose: StagingPurpose,
}

impl ManifestRecordingStorageBackend {
    pub(super) fn new(
        inner: Box<dyn StorageBackend>,
        store: SqliteStore,
        purpose: StagingPurpose,
    ) -> Self {
        Self {
            inner,
            store,
            purpose,
        }
    }
}

#[async_trait]
impl StorageBackend for ManifestRecordingStorageBackend {
    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        self.inner.stat(uri).await
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        self.inner.list(uri).await
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
        self.inner.list_with_status(uri).await
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

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let staged = self.inner.stage(request).await?;
        record_staged_input(&self.store, self.purpose, &staged.uri, &staged).await?;
        Ok(staged)
    }
}
