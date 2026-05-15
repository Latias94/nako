use std::{io::ErrorKind, path::Path, sync::Arc};

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
use tokio::sync::{Mutex, Semaphore};

use super::current_time_ms;

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
            store.delete_staging_manifest_record(record.id).await?;
            summary.deleted_records += 1;
        }
    }

    Ok(summary)
}

pub(super) struct ManifestRecordingStorageBackend {
    inner: Box<dyn StorageBackend>,
    store: SqliteStore,
    purpose: StagingPurpose,
    max_bytes: u64,
    retention_ms: u64,
    stage_permits: Arc<Semaphore>,
    budget_lock: Arc<Mutex<()>>,
}

#[derive(Clone, Debug)]
struct StagingReservation {
    record: StagingManifestRecord,
    previous: Option<StagingManifestRecord>,
}

impl ManifestRecordingStorageBackend {
    pub(super) fn new(
        inner: Box<dyn StorageBackend>,
        store: SqliteStore,
        purpose: StagingPurpose,
        max_bytes: u64,
        retention_ms: u64,
        stage_permits: Arc<Semaphore>,
        budget_lock: Arc<Mutex<()>>,
    ) -> Self {
        Self {
            inner,
            store,
            purpose,
            max_bytes,
            retention_ms,
            stage_permits,
            budget_lock,
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
        let existing = self
            .store
            .find_staging_manifest_record_by_path(&candidate_path.display().to_string())
            .await?;
        let now_ms = current_time_ms()?;
        if existing.as_ref().is_some_and(|record| {
            record.state == StagingState::Staging && !record_expired(record, now_ms)
        }) {
            return Err(TaruError::Storage {
                uri: request.uri.to_string(),
                message: format!(
                    "staging input is already reserved: {}",
                    candidate_path.display()
                ),
            });
        }

        let existing_bytes = existing
            .as_ref()
            .and_then(|record| record.size_bytes)
            .unwrap_or(0);
        let additional_bytes = incoming_bytes.saturating_sub(existing_bytes);
        let used_bytes = self.store.sum_staging_manifest_bytes().await?;
        let projected_bytes = used_bytes.saturating_add(additional_bytes);

        if additional_bytes > 0 && projected_bytes > self.max_bytes {
            return Err(TaruError::Storage {
                uri: request.uri.to_string(),
                message: format!(
                    "staging disk budget exhausted: used={used_bytes}, additional={additional_bytes}, max={}",
                    self.max_bytes
                ),
            });
        }

        let expires_at_ms = staging_expires_at_ms(now_ms, self.retention_ms)?;
        let id = existing
            .as_ref()
            .map(|record| record.id)
            .unwrap_or_else(StagingManifestId::new);
        let created_at_ms = existing
            .as_ref()
            .map(|record| record.created_at_ms)
            .unwrap_or(now_ms);
        let record = self
            .store
            .upsert_staging_manifest_record(NewStagingManifestRecord {
                id,
                source_uri: request.uri.to_string(),
                source_scheme: request.uri.scheme().to_owned(),
                purpose: self.purpose,
                local_path: candidate_path.display().to_string(),
                size_bytes: Some(incoming_bytes),
                etag: metadata.etag.clone(),
                fingerprint: metadata.fingerprint.clone(),
                state: StagingState::Staging,
                created_at_ms,
                updated_at_ms: now_ms,
                last_accessed_at_ms: now_ms,
                expires_at_ms: Some(expires_at_ms),
                active_leases: 0,
                validation_error: None,
            })
            .await?;

        Ok(StagingReservation {
            record,
            previous: existing,
        })
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

    async fn rollback_reservation(&self, reservation: StagingReservation) -> Result<()> {
        let _budget_guard = self.budget_lock.lock().await;
        match reservation.previous {
            Some(previous) => {
                self.store
                    .upsert_staging_manifest_record(new_staging_record_from_existing(previous))
                    .await?;
            }
            None => {
                self.store
                    .delete_staging_manifest_record(reservation.record.id)
                    .await?;
            }
        }

        Ok(())
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
        let reservation = {
            let _budget_guard = self.budget_lock.lock().await;
            self.reserve_budget(&request, &metadata).await?
        };

        let staged = match self.inner.stage(request).await {
            Ok(staged) => staged,
            Err(err) => {
                self.rollback_reservation(reservation).await?;
                return Err(err);
            }
        };

        if let Err(err) = self.complete_reservation(&reservation, &staged).await {
            self.rollback_reservation(reservation).await?;
            return Err(err);
        }

        Ok(staged)
    }
}

fn new_staging_record_from_existing(record: StagingManifestRecord) -> NewStagingManifestRecord {
    NewStagingManifestRecord {
        id: record.id,
        source_uri: record.source_uri,
        source_scheme: record.source_scheme,
        purpose: record.purpose,
        local_path: record.local_path,
        size_bytes: record.size_bytes,
        etag: record.etag,
        fingerprint: record.fingerprint,
        state: record.state,
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
        last_accessed_at_ms: record.last_accessed_at_ms,
        expires_at_ms: record.expires_at_ms,
        active_leases: record.active_leases,
        validation_error: record.validation_error,
    }
}

fn record_expired(record: &StagingManifestRecord, now_ms: i64) -> bool {
    record
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_ms)
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
