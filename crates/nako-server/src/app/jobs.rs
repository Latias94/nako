use std::sync::Arc;

use nako_core::{
    DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository, Job,
    JobCancellationRequestRecord, JobId, JobKind, JobLeaseRepository, JobListFilter, JobRepository,
    Library, LibraryId, LibraryRepository, NakoError, NewJob, NewOutboxEvent, PageRequest,
    RequestJobCancellation, Result, StagingPurpose,
};
use nako_db::NakoDatabase;
use nako_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryIndexSummary, LibraryProbeOptions,
    LibraryProbeRequest, LibraryProbeService, LibraryProbeSummary, LibraryScannerOptions,
};
use nako_media_probe::FfprobeMediaProbe;
use serde::Serialize;
use tokio::sync::Semaphore;
use tracing::{Instrument, info, info_span, warn};

use crate::config::{NakoServerConfig, libraries_from_config};

use super::{
    job_runtime::{
        DurableJobContext, DurableJobOperationResult, DurableJobRunOutcome, DurableJobRuntime,
    },
    runtime::RuntimeSupervisor,
    staging::ManifestRecordingStorageBackend,
    storage::{StorageBackendRegistry, remote_probe_staging_root},
};

#[derive(Clone, Debug, Serialize)]
pub struct ScanCommandOutput {
    pub job: Job,
    pub index: LibraryIndexSummary,
    pub probe: LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
enum LibraryScanExecution {
    Completed(ScanCommandOutput),
    Cancelled(Job),
}

#[derive(Clone, Debug)]
pub(crate) struct JobAppService {
    store: NakoDatabase,
}

impl JobAppService {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub(crate) async fn get_job(&self, job_id: JobId) -> Result<Job> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }

    pub(crate) async fn list_jobs(
        &self,
        filter: JobListFilter,
        page: PageRequest,
    ) -> Result<Vec<Job>> {
        self.store.list_jobs(filter, page).await
    }

    pub(crate) async fn request_job_cancellation(
        &self,
        job_id: JobId,
    ) -> Result<JobCancellationRequestRecord> {
        self.store
            .request_job_cancellation(RequestJobCancellation {
                job_id,
                reason: None,
            })
            .await
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LibraryScanAppService {
    config: NakoServerConfig,
    store: NakoDatabase,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
}

impl LibraryScanAppService {
    pub(super) fn new(
        config: NakoServerConfig,
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            config,
            store,
            permits,
            storage_backends,
            runtime,
        }
    }

    pub(crate) async fn enqueue_library_scan(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_library_scan_job(library_id).await?;
        let job_id = job.id;
        let service = self.clone();

        self.runtime.spawn_job(
            "library_scan_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move { service.finish_library_scan_job(job_id, library_id).await }.instrument(
                    info_span!(
                        "library_scan_background_job",
                        job_id = %job_id,
                        library_id = %library_id,
                        resource_class = "disk.scan"
                    ),
                )
            },
        );

        Ok(job)
    }

    pub(crate) async fn scan_library(&self, library_id: LibraryId) -> Result<ScanCommandOutput> {
        let job = self.create_library_scan_job(library_id).await?;
        self.execute_library_scan_command(job.id, library_id).await
    }

    pub(crate) async fn scan_all_configured_libraries(&self) -> Result<Vec<ScanCommandOutput>> {
        let libraries = libraries_from_config(&self.config);
        if libraries.is_empty() {
            return Err(NakoError::InvalidInput {
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
        self.library_for_scan(library_id).await?;
        let input = LibraryScanJobInput {
            library_id,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to serialize job input: {err}"),
        })?;

        self.store
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

    async fn finish_library_scan_job(&self, job_id: JobId, library_id: LibraryId) -> Result<Job> {
        match self.execute_library_scan_job(job_id, library_id).await? {
            LibraryScanExecution::Completed(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    status = ?output.job.status,
                    "library scan job completed"
                );
                Ok(output.job)
            }
            LibraryScanExecution::Cancelled(job) => {
                info!(
                    job_id = %job.id,
                    library_id = %library_id,
                    status = ?job.status,
                    "library scan job cancelled"
                );
                Ok(job)
            }
        }
    }

    async fn execute_library_scan_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<LibraryScanExecution> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| NakoError::InvalidInput {
                    message: format!("scan concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job_with_context(
                job_id,
                "library scan job",
                |context| async { self.run_library_scan(job_id, library_id, context).await },
                |(index, probe)| {
                    let summary = ScanJobSummary {
                        index: index.clone(),
                        probe: probe.clone(),
                    };
                    DurableJobRuntime::serialize_summary(&summary, "library scan job summary")
                },
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => {
                let (index, probe) = run.output;
                self.record_library_scanned_event(job_id, library_id, &index, &probe)
                    .await;

                Ok(LibraryScanExecution::Completed(ScanCommandOutput {
                    job: run.job,
                    index,
                    probe,
                }))
            }
            DurableJobRunOutcome::Cancelled(job) => Ok(LibraryScanExecution::Cancelled(job)),
        }
    }

    async fn execute_library_scan_command(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<ScanCommandOutput> {
        match self.execute_library_scan_job(job_id, library_id).await? {
            LibraryScanExecution::Completed(output) => Ok(output),
            LibraryScanExecution::Cancelled(job) => Err(NakoError::Conflict {
                message: format!("library scan job {} was cancelled", job.id),
            }),
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
            "failed_scan_entries": index.failed_entries,
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
        context: DurableJobContext,
    ) -> DurableJobOperationResult<(LibraryIndexSummary, LibraryProbeSummary)> {
        let library = self.library_for_scan(library_id).await?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            probe_concurrency = self.config.probe_concurrency.max(1),
            "starting library scan pipeline"
        );

        context.check_cancelled().await?;
        let index_backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        let scanner = nako_library::VfsLibraryScanner::with_options(
            index_backend,
            library_scanner_options(&library),
        );
        let index_service = LibraryIndexService::new(scanner, self.store.clone());
        let index = index_service
            .index_library(LibraryIndexRequest {
                job_id,
                library: library.clone(),
                force: false,
            })
            .await?;

        context.check_cancelled().await?;
        let storage_backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        let probe_backend = ManifestRecordingStorageBackend::new(
            storage_backend.clone(),
            self.store.clone(),
            StagingPurpose::ProbeInput,
            self.config.staging.max_bytes,
            self.config.staging.retention_ms,
            storage_backend.stage_permits(),
        );
        let probe = FfprobeMediaProbe::new(&self.config.ffprobe_path);
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            self.store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: self.config.probe_concurrency.max(1),
                staging_root: remote_probe_staging_root(&library, &self.config),
            },
        );
        let probe = probe_service
            .probe_library(LibraryProbeRequest {
                job_id,
                library_id,
                force: false,
            })
            .await?;

        context.check_cancelled().await?;
        Ok((index, probe))
    }

    async fn record_outbox_event(&self, event: NewOutboxEvent) {
        let kind = event.kind.as_str();
        let idempotency_key = event.idempotency_key.clone();
        if let Err(err) = self.store.enqueue_outbox_event(event).await {
            warn!(
                kind,
                idempotency_key,
                error = %err,
                "failed to persist outbox event"
            );
        }
    }

    async fn library_for_scan(&self, library_id: LibraryId) -> Result<Library> {
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

fn library_scanner_options(library: &Library) -> LibraryScannerOptions {
    let defaults = LibraryScannerOptions::default();

    LibraryScannerOptions {
        media_extensions: defaults.media_extensions,
        max_depth: library.options.scan.max_depth.unwrap_or(defaults.max_depth),
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
