use std::{
    fmt,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use taru_core::{
    NewStagingManifestRecord, PageRequest, Result, StagingManifestId, StagingManifestRecord,
    StagingManifestRepository, StagingPurpose, StagingState, TaruError,
};
use taru_db::SqliteStore;
use taru_vfs::{
    ByteRange, ObjectListing, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile,
    StorageBackend, StorageUri, VirtualFile, deterministic_stage_path,
};
use tokio::sync::Semaphore;
use tracing::warn;

use super::{current_time_ms, runtime::RuntimeSupervisor};

pub(super) async fn record_staged_input(
    store: &SqliteStore,
    purpose: StagingPurpose,
    uri: &StorageUri,
    staged: &StagedFile,
    retention_ms: u64,
) -> Result<()> {
    let now_ms = current_time_ms()?;
    let expires_at_ms = staging_expires_at_ms(now_ms, retention_ms)?;
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
        .complete_staging_manifest_record(NewStagingManifestRecord {
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
            expires_at_ms: Some(expires_at_ms),
            active_leases: 0,
            validation_error: None,
        })
        .await?;

    Ok(())
}

pub(super) struct StagingCleanupSummary {
    pub(super) deleted_records: usize,
    pub(super) deleted_files: usize,
}

pub(super) async fn cleanup_expired_staging_inputs(
    store: &SqliteStore,
    now_ms: i64,
) -> Result<StagingCleanupSummary> {
    let mut summary = StagingCleanupSummary {
        deleted_records: 0,
        deleted_files: 0,
    };

    loop {
        let candidates = store
            .list_staging_cleanup_candidates(now_ms, PageRequest::new(100, 0))
            .await?;
        if candidates.is_empty() {
            break;
        }

        for record in candidates {
            match store
                .expire_staging_manifest_record(record.id, now_ms)
                .await
            {
                Ok(Some(expired))
                    if expired.state == StagingState::Expired && expired.active_leases == 0 => {}
                Ok(Some(current)) => {
                    warn!(
                        record_id = %record.id,
                        state = ?current.state,
                        active_leases = current.active_leases,
                        "skipped staging cleanup because record is no longer an unleased cleanup candidate"
                    );
                    continue;
                }
                Ok(None) => {
                    warn!(
                        record_id = %record.id,
                        "skipped staging cleanup because record disappeared before expiration"
                    );
                    continue;
                }
                Err(err) => {
                    warn!(
                        record_id = %record.id,
                        error = %err,
                        "failed to mark staging record expired before deletion"
                    );
                    continue;
                }
            }
            match tokio::fs::remove_file(Path::new(&record.local_path)).await {
                Ok(()) => {
                    summary.deleted_files += 1;
                }
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(TaruError::Storage {
                        uri: record.local_path,
                        message: format!("failed to delete expired staged input: {err}"),
                    });
                }
            }
            if let Err(err) = store
                .mark_deleted_staging_manifest_record(record.id, now_ms)
                .await
            {
                warn!(
                    record_id = %record.id,
                    error = %err,
                    "failed to mark staging record deleted after deletion"
                );
            }
            summary.deleted_records += 1;
        }
    }

    Ok(summary)
}

pub(super) struct ManifestRecordingStorageBackend {
    inner: Arc<dyn StorageBackend>,
    store: SqliteStore,
    purpose: StagingPurpose,
    max_bytes: u64,
    retention_ms: u64,
    stage_permits: Arc<Semaphore>,
}

#[derive(Clone, Debug)]
struct StagingReservation {
    record: StagingManifestRecord,
}

impl ManifestRecordingStorageBackend {
    pub(super) fn new(
        inner: Arc<dyn StorageBackend>,
        store: SqliteStore,
        purpose: StagingPurpose,
        max_bytes: u64,
        retention_ms: u64,
        stage_permits: Arc<Semaphore>,
    ) -> Self {
        Self {
            inner,
            store,
            purpose,
            max_bytes,
            retention_ms,
            stage_permits,
        }
    }

    async fn reserve_budget(
        &self,
        request: &StageRequest,
        metadata: &ObjectMetadata,
    ) -> Result<StagingReservation> {
        let incoming_bytes = metadata.len.ok_or_else(|| TaruError::Storage {
            uri: request.uri.to_string(),
            message: "staging disk budget requires a known source size".to_owned(),
        })?;
        let candidate_path = deterministic_stage_path(
            &request.root,
            &request.uri,
            metadata.fingerprint.as_deref().or(metadata.etag.as_deref()),
        )?;
        let now_ms = current_time_ms()?;

        let expires_at_ms = staging_expires_at_ms(now_ms, self.retention_ms)?;
        let record = self
            .store
            .reserve_staging_manifest_record(
                NewStagingManifestRecord {
                    id: StagingManifestId::new(),
                    source_uri: request.uri.to_string(),
                    source_scheme: request.uri.scheme().to_owned(),
                    purpose: self.purpose,
                    local_path: candidate_path.display().to_string(),
                    size_bytes: Some(incoming_bytes),
                    etag: metadata.etag.clone(),
                    fingerprint: metadata.fingerprint.clone(),
                    state: StagingState::Reserved,
                    created_at_ms: now_ms,
                    updated_at_ms: now_ms,
                    last_accessed_at_ms: now_ms,
                    expires_at_ms: Some(expires_at_ms),
                    active_leases: 0,
                    validation_error: None,
                },
                self.max_bytes,
                now_ms,
            )
            .await?;

        Ok(StagingReservation { record })
    }

    async fn complete_reservation(
        &self,
        reservation: &StagingReservation,
        staged: &StagedFile,
    ) -> Result<()> {
        let staged_path = staged.path.display().to_string();
        if staged_path != reservation.record.local_path {
            return Err(TaruError::Storage {
                uri: staged.uri.to_string(),
                message: format!(
                    "staging backend returned a different path than the reserved manifest path: reserved={}, staged={staged_path}",
                    reservation.record.local_path
                ),
            });
        }

        record_staged_input(
            &self.store,
            self.purpose,
            &staged.uri,
            staged,
            self.retention_ms,
        )
        .await
    }

    async fn fail_reservation(
        &self,
        reservation: &StagingReservation,
        reason: String,
    ) -> Result<()> {
        self.store
            .fail_staging_manifest_record(reservation.record.id, current_time_ms()?, reason)
            .await?;

        Ok(())
    }

    async fn cleanup_failed_reservation(&self, reservation: &StagingReservation) {
        if let Err(err) = tokio::fs::remove_file(Path::new(&reservation.record.local_path)).await {
            if err.kind() != ErrorKind::NotFound {
                warn!(
                    record_id = %reservation.record.id,
                    path = %reservation.record.local_path,
                    error = %err,
                    "failed to remove failed staged input"
                );
            }
        }
    }

    async fn record_reservation_failure(&self, reservation: &StagingReservation, err: &TaruError) {
        if let Err(rollback_err) = self.fail_reservation(reservation, err.to_string()).await {
            warn!(
                record_id = %reservation.record.id,
                original_error = %err,
                rollback_error = %rollback_err,
                "failed to mark staging reservation failed"
            );
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
        let _permit = self
            .stage_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::Storage {
                uri: "playback.remote.stage".to_owned(),
                message: format!("remote staging resource budget was closed: {err}"),
            })?;

        let metadata = self.inner.stat(&request.uri).await?;
        let reservation = self.reserve_budget(&request, &metadata).await?;

        if let Err(err) = self
            .store
            .start_staging_manifest_record(reservation.record.id, current_time_ms()?)
            .await
        {
            self.record_reservation_failure(&reservation, &err).await;
            return Err(err);
        }

        let staged = match self.inner.stage(request).await {
            Ok(staged) => staged,
            Err(err) => {
                self.record_reservation_failure(&reservation, &err).await;
                self.cleanup_failed_reservation(&reservation).await;
                return Err(err);
            }
        };

        if let Err(err) = self.complete_reservation(&reservation, &staged).await {
            self.record_reservation_failure(&reservation, &err).await;
            self.cleanup_failed_reservation(&reservation).await;
            return Err(err);
        }

        Ok(staged)
    }
}

pub(super) struct StagingLease {
    store: SqliteStore,
    record_id: StagingManifestId,
    local_path: PathBuf,
    runtime: RuntimeSupervisor,
    released: bool,
}

impl StagingLease {
    pub(super) async fn acquire(
        store: SqliteStore,
        record_id: StagingManifestId,
        runtime: RuntimeSupervisor,
    ) -> Result<Self> {
        let record = store
            .acquire_staging_manifest_lease(record_id, current_time_ms()?)
            .await?;
        Ok(Self {
            store,
            record_id,
            local_path: PathBuf::from(record.local_path),
            runtime,
            released: false,
        })
    }

    pub(super) async fn release(mut self) -> Result<StagingManifestRecord> {
        let store = self.store.clone();
        let record_id = self.record_id;
        let record = store
            .release_staging_manifest_lease(record_id, current_time_ms()?)
            .await?;
        self.released = true;

        Ok(record)
    }
}

impl Drop for StagingLease {
    fn drop(&mut self) {
        if self.released {
            return;
        }

        let store = self.store.clone();
        let record_id = self.record_id;
        let local_path = self.local_path.clone();
        let runtime = self.runtime.clone();

        runtime.spawn(
            "staging_lease_drop_release",
            "storage.staging.lease",
            async move {
                let released_at_ms = match current_time_ms() {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            record_id = %record_id,
                            path = %local_path.display(),
                            error = %err,
                            "failed to compute staging lease drop release timestamp"
                        );
                        return;
                    }
                };

                if let Err(err) = store
                    .release_staging_manifest_lease(record_id, released_at_ms)
                    .await
                {
                    warn!(
                        record_id = %record_id,
                        path = %local_path.display(),
                        error = %err,
                        "failed to release dropped staging lease"
                    );
                }
            },
        );
    }
}

impl fmt::Debug for StagingLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StagingLease")
            .field("record_id", &self.record_id)
            .field("local_path", &self.local_path)
            .finish()
    }
}

fn staging_expires_at_ms(now_ms: i64, retention_ms: u64) -> Result<i64> {
    let retention_ms = i64::try_from(retention_ms).map_err(|err| TaruError::InvalidInput {
        message: format!("staging retention does not fit i64 milliseconds: {err}"),
    })?;

    now_ms
        .checked_add(retention_ms)
        .ok_or_else(|| TaruError::InvalidInput {
            message: "staging expiration timestamp overflowed".to_owned(),
        })
}
