use std::sync::Arc;

use serde::Serialize;
use taru_api::{ItemsResponse, LibraryListResponse, LibrarySourceResponse, LibrarySourcesResponse};
use taru_core::{
    Job, JobId, JobKind, JobRepository, Library, LibraryId, LibraryRepository,
    MediaProbeRepository, MediaRepository, MediaSourceId, NewJob, Result, TaruError,
    TransactionManager,
};
use taru_db::SqliteStore;
use taru_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryIndexSummary, LibraryProbeOptions,
    LibraryProbeRequest, LibraryProbeService, LibraryProbeSummary,
};
use taru_media_probe::FfprobeMediaProbe;
use taru_vfs::LocalFsBackend;
use tokio::sync::Semaphore;
use tracing::{Instrument, error, info, info_span, warn};

use crate::config::{TaruServerConfig, library_from_config};

#[derive(Clone, Debug)]
pub struct TaruApp {
    inner: Arc<TaruAppInner>,
}

#[derive(Debug)]
struct TaruAppInner {
    config: TaruServerConfig,
    store: SqliteStore,
    scan_permits: Arc<Semaphore>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanCommandOutput {
    pub job: Job,
    pub index: LibraryIndexSummary,
    pub probe: LibraryProbeSummary,
}

impl TaruApp {
    pub async fn new(config: TaruServerConfig) -> Result<Self> {
        let store = SqliteStore::connect(&config.database_url).await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: TaruServerConfig, store: SqliteStore) -> Result<Self> {
        store.migrate().await?;

        let app = Self {
            inner: Arc::new(TaruAppInner {
                scan_permits: Arc::new(Semaphore::new(config.scan_concurrency.max(1))),
                config,
                store,
            }),
        };

        app.ensure_configured_library().await?;
        Ok(app)
    }

    #[must_use]
    pub fn config(&self) -> &TaruServerConfig {
        &self.inner.config
    }

    pub async fn list_libraries(&self) -> Result<LibraryListResponse> {
        Ok(LibraryListResponse {
            libraries: self.inner.store.list_libraries().await?,
        })
    }

    pub async fn list_library_sources(
        &self,
        library_id: LibraryId,
    ) -> Result<LibrarySourcesResponse> {
        let library = self.get_library_or_not_found(library_id).await?;
        let sources = self.inner.store.list_media_sources(library.id).await?;
        let mut output_sources = Vec::with_capacity(sources.len());

        for source in sources {
            let item = self.inner.store.get_media_item(source.item_id).await?;
            let probe = self.inner.store.get_media_probe(source.id).await?;
            output_sources.push(LibrarySourceResponse {
                source,
                item,
                probe,
            });
        }

        Ok(LibrarySourcesResponse {
            library,
            sources: output_sources,
        })
    }

    pub async fn list_items(&self) -> Result<ItemsResponse> {
        Ok(ItemsResponse {
            items: self.inner.store.list_media_items().await?,
        })
    }

    pub async fn get_source_probe(
        &self,
        source_id: MediaSourceId,
    ) -> Result<taru_api::SourceProbeResponse> {
        let probe = self
            .inner
            .store
            .get_media_probe(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source_probe",
                id: source_id.to_string(),
            })?;

        Ok(taru_api::SourceProbeResponse { source_id, probe })
    }

    pub async fn get_job(&self, job_id: JobId) -> Result<Job> {
        self.inner
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }

    pub async fn enqueue_library_scan(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_library_scan_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_library_scan_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "library_scan_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "disk.scan"
            )),
        );

        Ok(job)
    }

    pub async fn scan_configured_library(&self) -> Result<ScanCommandOutput> {
        let library_id = self.config().library.id;
        let job = self.create_library_scan_job(library_id).await?;
        self.execute_library_scan_job(job.id, library_id).await
    }

    async fn ensure_configured_library(&self) -> Result<()> {
        let library = library_from_config(self.config());
        self.inner.store.upsert_library(&library).await?;
        Ok(())
    }

    async fn create_library_scan_job(&self, library_id: LibraryId) -> Result<Job> {
        self.configured_library_for(library_id)?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: Some(library_id),
                source_id: None,
            })
            .await
    }

    async fn finish_library_scan_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_library_scan_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    status = ?output.job.status,
                    "library scan job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "library scan job failed"
                );
            }
        }
    }

    async fn execute_library_scan_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<ScanCommandOutput> {
        let permit = self
            .inner
            .scan_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("scan concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_library_scan(job_id, library_id).await {
            Ok((index, probe)) => {
                let output = ScanJobSummary {
                    index: index.clone(),
                    probe: probe.clone(),
                };
                let summary_json =
                    serde_json::to_string(&output).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(ScanCommandOutput { job, index, probe })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn run_library_scan(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<(LibraryIndexSummary, LibraryProbeSummary)> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            probe_concurrency = self.config().probe_concurrency.max(1),
            "starting library scan pipeline"
        );

        let index_backend = LocalFsBackend::new(&self.config().library.root)?;
        let scanner = taru_library::VfsLibraryScanner::new(index_backend);
        let index_service = LibraryIndexService::new(scanner, self.inner.store.clone());
        let index = index_service
            .index_library(LibraryIndexRequest {
                job_id,
                library,
                force: false,
            })
            .await?;

        let probe_backend = LocalFsBackend::new(&self.config().library.root)?;
        let probe = FfprobeMediaProbe::new(&self.config().ffprobe_path);
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            self.inner.store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: self.config().probe_concurrency.max(1),
            },
        );
        let probe = probe_service
            .probe_library(LibraryProbeRequest {
                job_id,
                library_id,
                force: false,
            })
            .await?;

        Ok((index, probe))
    }

    fn configured_library_for(&self, library_id: LibraryId) -> Result<Library> {
        let library = library_from_config(self.config());

        if library.id == library_id {
            Ok(library)
        } else {
            Err(TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
        }
    }

    async fn get_library_or_not_found(&self, library_id: LibraryId) -> Result<Library> {
        self.inner
            .store
            .get_library(library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

#[derive(Clone, Debug, Serialize)]
struct ScanJobSummary {
    index: LibraryIndexSummary,
    probe: LibraryProbeSummary,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use taru_core::{JobStatus, LibraryId};

    use super::*;
    use crate::config::LocalLibraryConfig;

    #[tokio::test]
    async fn scan_configured_library_persists_job_success() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store).await.unwrap();

        let output = app.scan_configured_library().await.unwrap();
        let job = app.get_job(output.job.id).await.unwrap();

        assert_eq!(output.job.status, JobStatus::Succeeded);
        assert_eq!(job.status, JobStatus::Succeeded);
        assert_eq!(output.index.discovered_files, 0);
        assert_eq!(output.probe.total_sources, 0);
    }
}
