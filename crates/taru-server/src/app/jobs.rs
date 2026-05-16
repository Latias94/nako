use serde::Serialize;
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, Job, JobId, JobKind, JobRepository, LibraryId,
    NewJob, NewOutboxEvent, Result, StagingPurpose, TaruError,
};
use taru_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryIndexSummary, LibraryProbeOptions,
    LibraryProbeRequest, LibraryProbeService, LibraryProbeSummary,
};
use taru_media_probe::FfprobeMediaProbe;
use tracing::{Instrument, error, info, info_span, warn};

use super::{
    ManifestRecordingStorageBackend, TaruApp, libraries_from_config, remote_probe_staging_root,
};

#[derive(Clone, Debug, Serialize)]
pub struct ScanCommandOutput {
    pub job: Job,
    pub index: LibraryIndexSummary,
    pub probe: LibraryProbeSummary,
}

impl TaruApp {
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

    pub async fn scan_library(&self, library_id: LibraryId) -> Result<ScanCommandOutput> {
        let job = self.create_library_scan_job(library_id).await?;
        self.execute_library_scan_job(job.id, library_id).await
    }

    pub async fn scan_all_configured_libraries(&self) -> Result<Vec<ScanCommandOutput>> {
        let libraries = libraries_from_config(self.config());
        if libraries.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "server config must include at least one library".to_owned(),
            });
        }

        let mut outputs = Vec::with_capacity(libraries.len());
        for library in libraries {
            outputs.push(self.scan_library(library.id).await?);
        }

        Ok(outputs)
    }

    async fn create_library_scan_job(&self, library_id: LibraryId) -> Result<Job> {
        self.configured_library_for(library_id)?;
        let input = LibraryScanJobInput {
            library_id,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
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
                self.record_library_scanned_event(job_id, library_id, &index, &probe)
                    .await;

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

    async fn record_library_scanned_event(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        index: &LibraryIndexSummary,
        probe: &LibraryProbeSummary,
    ) {
        let payload = serde_json::json!({
            "job_id": job_id,
            "library_id": library_id,
            "scan_id": index.scan_id,
            "discovered_files": index.discovered_files,
            "inserted_sources": index.inserted_sources,
            "updated_sources": index.updated_sources,
            "tombstoned_sources": index.tombstoned_sources,
            "probed_sources": probe.probed_sources,
            "failed_probe_sources": probe.failed_sources,
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("library.scanned:{job_id}"),
            payload_json: payload.to_string(),
        })
        .await;
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

        let index_backend = self.storage_backend_for_library_root(&library).await?;
        let scanner = taru_library::VfsLibraryScanner::new(index_backend);
        let index_service = LibraryIndexService::new(scanner, self.inner.store.clone());
        let index = index_service
            .index_library(LibraryIndexRequest {
                job_id,
                library: library.clone(),
                force: false,
            })
            .await?;

        let storage_backend = self.storage_backend_for_library_root(&library).await?;
        let probe_backend = ManifestRecordingStorageBackend::new(
            storage_backend.clone(),
            self.inner.store.clone(),
            StagingPurpose::ProbeInput,
            self.config().staging.max_bytes,
            self.config().staging.retention_ms,
            storage_backend.stage_permits(),
        );
        let probe = FfprobeMediaProbe::new(&self.config().ffprobe_path);
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            self.inner.store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: self.config().probe_concurrency.max(1),
                staging_root: remote_probe_staging_root(&library, self.config()),
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
}

#[derive(Clone, Debug, Serialize)]
struct ScanJobSummary {
    index: LibraryIndexSummary,
    probe: LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
struct LibraryScanJobInput {
    library_id: LibraryId,
    force: bool,
}
