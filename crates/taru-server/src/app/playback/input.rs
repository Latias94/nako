use std::path::PathBuf;

use taru_core::{MediaSource, Result, StagingManifestRepository, StagingPurpose, TaruError};
use taru_db::SqliteStore;
use taru_vfs::{StageRequest, StorageBackend, StorageUri};

use crate::config::TaruServerConfig;

use super::super::{
    runtime::RuntimeSupervisor,
    staging::{ManifestRecordingStorageBackend, StagingLease},
    storage::LibraryStorageBackend,
};

pub(super) struct FfmpegSourceInput {
    pub(super) path: PathBuf,
    lease: Option<StagingLease>,
}

#[derive(Clone, Debug)]
pub(super) struct FfmpegInputService {
    config: TaruServerConfig,
    store: SqliteStore,
    runtime: RuntimeSupervisor,
}

impl FfmpegInputService {
    pub(super) fn new(
        config: TaruServerConfig,
        store: SqliteStore,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            config,
            store,
            runtime,
        }
    }

    pub(super) async fn source_input_for_ffmpeg(
        &self,
        source: &MediaSource,
        uri: &StorageUri,
        backend: &LibraryStorageBackend,
    ) -> Result<FfmpegSourceInput> {
        let backend = ManifestRecordingStorageBackend::new(
            backend.clone_backend(),
            self.store.clone(),
            StagingPurpose::FfmpegInput,
            self.config.staging.max_bytes,
            self.config.staging.retention_ms,
            backend.stage_permits(),
        );
        match local_source_path_and_len(source, uri, &backend).await {
            Ok((path, _len)) => Ok(FfmpegSourceInput { path, lease: None }),
            Err(TaruError::Unsupported(_)) => {
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
                    .ok_or_else(|| TaruError::NotFound {
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
        let input = self.source_input_for_ffmpeg(source, uri, backend).await?;
        let path = input.path.clone();
        self.release_source_input(input).await?;
        Ok(path)
    }

    pub(super) async fn release_source_input(&self, input: FfmpegSourceInput) -> Result<()> {
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
        Err(TaruError::Unsupported(_)) => {
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
        TaruError::Unsupported("local playback operations currently require a local path hint")
    })?;
    let total_len = match metadata.len {
        Some(len) => len,
        None => tokio::fs::metadata(&local_path)
            .await
            .map_err(|err| TaruError::Storage {
                uri: source.locator.clone(),
                message: format!("failed to read playback source length: {err}"),
            })?
            .len(),
    };

    Ok((local_path, total_len))
}
