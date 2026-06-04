use std::{fmt, future::Future, path::PathBuf, sync::Arc};

use nako_core::{
    MediaSource, NakoError, Result, StagingAttribution, StagingManifestRepository, StagingPurpose,
};
use nako_vfs::{StageRequest, StorageBackend, StorageUri};
use tracing::warn;

use crate::config::NakoServerConfig;

use super::super::{
    runtime::RuntimeSupervisor,
    staging::{ManifestRecordingStorageBackend, StagingLease},
    storage::LibraryStorageBackend,
};

struct FfmpegSourceInput {
    path: PathBuf,
    lease: Option<StagingLease>,
}

pub(super) struct FfmpegSourceInputScope {
    input: FfmpegSourceInput,
}

#[derive(Clone)]
pub(super) struct FfmpegInputService {
    config: NakoServerConfig,
    store: Arc<dyn StagingManifestRepository>,
    runtime: RuntimeSupervisor,
}

impl fmt::Debug for FfmpegInputService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FfmpegInputService")
            .field("config", &self.config)
            .field("runtime", &self.runtime)
            .finish_non_exhaustive()
    }
}

impl FfmpegInputService {
    pub(super) fn new(
        config: NakoServerConfig,
        store: Arc<dyn StagingManifestRepository>,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            config,
            store,
            runtime,
        }
    }

    pub(super) async fn with_source_input<T, Operation, OperationFuture>(
        &self,
        source: &MediaSource,
        uri: &StorageUri,
        backend: &LibraryStorageBackend,
        operation: Operation,
    ) -> Result<T>
    where
        Operation: FnOnce(PathBuf) -> OperationFuture,
        OperationFuture: Future<Output = Result<T>>,
    {
        let input = self.source_input_scope(source, uri, backend).await?;
        self.with_prepared_source_input(input, operation).await
    }

    pub(super) async fn source_input_scope(
        &self,
        source: &MediaSource,
        uri: &StorageUri,
        backend: &LibraryStorageBackend,
    ) -> Result<FfmpegSourceInputScope> {
        self.source_input_for_ffmpeg(source, uri, backend)
            .await
            .map(|input| FfmpegSourceInputScope { input })
    }

    pub(super) async fn with_prepared_source_input<T, Operation, OperationFuture>(
        &self,
        scope: FfmpegSourceInputScope,
        operation: Operation,
    ) -> Result<T>
    where
        Operation: FnOnce(PathBuf) -> OperationFuture,
        OperationFuture: Future<Output = Result<T>>,
    {
        let input = scope.input;
        let path = input.path.clone();
        let result = operation(path).await;
        self.finish_source_input_scope(input, result).await
    }

    async fn source_input_for_ffmpeg(
        &self,
        source: &MediaSource,
        uri: &StorageUri,
        backend: &LibraryStorageBackend,
    ) -> Result<FfmpegSourceInput> {
        let backend = ManifestRecordingStorageBackend::new(
            backend.clone_backend(),
            self.store.clone(),
            StagingAttribution::attributed(source.library_id),
            StagingPurpose::FfmpegInput,
            self.config.staging.max_bytes,
            self.config.staging.retention_ms,
            backend.stage_permits(),
        );
        match local_source_path_and_len(source, uri, &backend).await {
            Ok((path, _len)) => Ok(FfmpegSourceInput { path, lease: None }),
            Err(NakoError::Unsupported(_)) => {
                let staged = backend
                    .stage(StageRequest::new(
                        uri.clone(),
                        self.config.remux_staging_root.join("inputs"),
                    ))
                    .await?;
                let record = self
                    .store
                    .find_staging_manifest_record_by_path(&staged.path.display().to_string())
                    .await?
                    .ok_or_else(|| NakoError::NotFound {
                        entity: "staging_manifest_record",
                        id: staged.path.display().to_string(),
                    })?;
                let lease =
                    StagingLease::acquire(self.store.clone(), record.id, self.runtime.clone())
                        .await?;
                Ok(FfmpegSourceInput {
                    path: staged.path,
                    lease: Some(lease),
                })
            }
            Err(err) => Err(err),
        }
    }

    #[cfg(test)]
    pub(super) async fn source_path_for_ffmpeg(
        &self,
        source: &MediaSource,
        uri: &StorageUri,
        backend: &LibraryStorageBackend,
    ) -> Result<PathBuf> {
        self.with_source_input(source, uri, backend, |path| async move { Ok(path) })
            .await
    }

    async fn finish_source_input_scope<T>(
        &self,
        input: FfmpegSourceInput,
        result: Result<T>,
    ) -> Result<T> {
        match result {
            Ok(output) => {
                self.release_source_input(input).await?;
                Ok(output)
            }
            Err(err) => {
                if let Err(release_err) = self.release_source_input(input).await {
                    warn!(
                        error = %release_err,
                        "failed to release ffmpeg input staging lease after operation error"
                    );
                }
                Err(err)
            }
        }
    }

    async fn release_source_input(&self, input: FfmpegSourceInput) -> Result<()> {
        if let Some(lease) = input.lease {
            lease.release().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) async fn source_path_for_ffmpeg_with_backend(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
    staging_root: PathBuf,
) -> Result<PathBuf> {
    match local_source_path_and_len(source, uri, backend).await {
        Ok((path, _len)) => Ok(path),
        Err(NakoError::Unsupported(_)) => {
            let staged = backend
                .stage(StageRequest::new(uri.clone(), staging_root))
                .await?;
            Ok(staged.path)
        }
        Err(err) => Err(err),
    }
}

pub(super) async fn local_source_path_and_len(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
) -> Result<(PathBuf, u64)> {
    let metadata = backend.stat(uri).await?;
    let virtual_file = backend.open_range(uri, None).await?;
    let local_path = virtual_file.local_path_hint.ok_or_else(|| {
        NakoError::Unsupported("local playback operations currently require a local path hint")
    })?;
    let total_len = match metadata.len {
        Some(len) => len,
        None => tokio::fs::metadata(&local_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    source.locator.clone(),
                    format!("failed to read playback source length: {err}"),
                )
            })?
            .len(),
    };

    Ok((local_path, total_len))
}
