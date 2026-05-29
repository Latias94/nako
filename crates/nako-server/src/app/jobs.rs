use std::sync::Arc;

use nako_core::{
    CancelLeasedJob, CompleteLeasedJob, DomainEventKind, DomainEventSubject, EventId,
    EventOutboxRepository, FailLeasedJob, IngestionFailurePhase, IngestionFailureRecord, Job,
    JobCancellationRequestRecord, JobId, JobKind, JobLeaseClaimFilter, JobLeaseClaimRequest,
    JobLeaseHeartbeat, JobLeaseRepository, JobListFilter, JobRepository, LeasedJob, Library,
    LibraryId, LibraryRepository, MediaProbeResult, MediaSource, NakoError, NewIngestionFailure,
    NewJob, NewOutboxEvent, OutboxEventRecord, PageRequest, RequestJobCancellation, Result,
    StagingPurpose,
};
use nako_db::NakoDatabase;
use nako_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryIndexSummary, LibraryIngestionWorkflow,
    LibraryProbeOptions, LibraryProbeRequest, LibraryProbeService, LibraryProbeSummary,
    LibraryProbeWorkflow, LibraryScannerOptions,
};
use nako_media_probe::FfprobeMediaProbe;
use serde::Serialize;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use tracing::{Instrument, info, info_span, warn};

use crate::config::{NakoServerConfig, libraries_from_config};

use super::{
    addons::AddonAppService,
    job_runtime::{
        DurableJobContext, DurableJobOperationResult, DurableJobRunOutcome, DurableJobRuntime,
    },
    metadata_scan::{
        LibraryScanMetadataSummary, MetadataScanAcquisitionRequest, MetadataScanAcquisitionService,
    },
    runtime::{RuntimeSupervisor, runtime_budget_class_for_job_resource_class},
    staging::ManifestRecordingStorageBackend,
    storage::{StorageBackendRegistry, remote_probe_staging_root},
};

#[derive(Clone, Debug, Serialize)]
pub struct ScanCommandOutput {
    pub job: Job,
    pub index: LibraryIndexSummary,
    pub probe: LibraryProbeSummary,
    pub metadata: LibraryScanMetadataSummary,
}

#[derive(Clone, Debug, Serialize)]
enum LibraryScanExecution {
    Completed(ScanCommandOutput),
    Cancelled(Job),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LibraryScanScheduleOutcome {
    Scheduled(JobId),
    NoQueuedJob,
    BudgetSaturated,
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

#[async_trait::async_trait]
pub(super) trait LibraryScanWorkflowStore: std::fmt::Debug + Send + Sync {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job>;

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>>;

    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord>;
}

#[async_trait::async_trait]
impl<T> LibraryScanWorkflowStore for T
where
    T: EventOutboxRepository + JobRepository + LibraryRepository + std::fmt::Debug + Send + Sync,
{
    async fn enqueue_job(&self, job: NewJob) -> Result<Job> {
        JobRepository::enqueue_job(self, job).await
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        LibraryRepository::get_library(self, id).await
    }

    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord> {
        EventOutboxRepository::enqueue_outbox_event(self, event).await
    }
}

#[derive(Clone, Debug)]
struct LibraryScanExecutionStore {
    store: NakoDatabase,
}

impl LibraryScanExecutionStore {
    fn new(store: NakoDatabase) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl super::job_runtime::DurableJobLeaseStore for LibraryScanExecutionStore {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>> {
        super::job_runtime::DurableJobLeaseStore::claim_next_job_lease(&self.store, request).await
    }

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob> {
        super::job_runtime::DurableJobLeaseStore::heartbeat_job_lease(&self.store, heartbeat).await
    }

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job> {
        super::job_runtime::DurableJobLeaseStore::succeed_leased_job(&self.store, completion).await
    }

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job> {
        super::job_runtime::DurableJobLeaseStore::fail_leased_job(&self.store, failure).await
    }

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job> {
        super::job_runtime::DurableJobLeaseStore::cancel_leased_job(&self.store, cancellation).await
    }
}

#[async_trait::async_trait]
impl LibraryIngestionWorkflow for LibraryScanExecutionStore {
    async fn ensure_library_for_ingestion(&self, library: &Library) -> Result<()> {
        LibraryIngestionWorkflow::ensure_library_for_ingestion(&self.store, library).await
    }

    async fn begin_ingestion_scan(
        &self,
        id: nako_core::ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<nako_core::ScanSnapshot> {
        LibraryIngestionWorkflow::begin_ingestion_scan(&self.store, id, library_id, root).await
    }

    async fn complete_ingestion_scan(
        &self,
        id: nako_core::ScanSnapshotId,
        status: nako_core::ScanStatus,
        error: Option<String>,
    ) -> Result<nako_core::ScanSnapshot> {
        LibraryIngestionWorkflow::complete_ingestion_scan(&self.store, id, status, error).await
    }

    async fn record_scan_failure(
        &self,
        commit: nako_library::LibraryScanFailureCommit,
    ) -> Result<()> {
        LibraryIngestionWorkflow::record_scan_failure(&self.store, commit).await
    }

    async fn commit_directory_observation(
        &self,
        commit: nako_library::LibraryDirectoryObservationCommit,
    ) -> Result<()> {
        LibraryIngestionWorkflow::commit_directory_observation(&self.store, commit).await
    }

    async fn commit_source_observation(
        &self,
        commit: nako_library::LibrarySourceObservationCommit,
    ) -> Result<nako_library::LibrarySourceIngestionSummary> {
        LibraryIngestionWorkflow::commit_source_observation(&self.store, commit).await
    }

    async fn tombstone_sources_missing_from_scan(
        &self,
        library_id: LibraryId,
        scan_id: nako_core::ScanSnapshotId,
    ) -> Result<u64> {
        LibraryIngestionWorkflow::tombstone_sources_missing_from_scan(
            &self.store,
            library_id,
            scan_id,
        )
        .await
    }
}

#[async_trait::async_trait]
impl LibraryProbeWorkflow for LibraryScanExecutionStore {
    async fn list_media_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        LibraryProbeWorkflow::list_media_sources(&self.store, library_id, page).await
    }

    async fn get_media_probe(
        &self,
        source_id: nako_core::MediaSourceId,
    ) -> Result<Option<MediaProbeResult>> {
        LibraryProbeWorkflow::get_media_probe(&self.store, source_id).await
    }

    async fn upsert_media_probe(
        &self,
        source_id: nako_core::MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()> {
        LibraryProbeWorkflow::upsert_media_probe(&self.store, source_id, result).await
    }

    async fn record_ingestion_failure(
        &self,
        failure: NewIngestionFailure,
    ) -> Result<IngestionFailureRecord> {
        LibraryProbeWorkflow::record_ingestion_failure(&self.store, failure).await
    }

    async fn resolve_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        resolved_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        LibraryProbeWorkflow::resolve_ingestion_failure(
            &self.store,
            library_id,
            phase,
            target_uri,
            resolved_at_ms,
        )
        .await
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LibraryScanAppService {
    config: NakoServerConfig,
    workflow_store: Arc<dyn LibraryScanWorkflowStore>,
    execution_store: LibraryScanExecutionStore,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
    metadata_scan: MetadataScanAcquisitionService,
}

impl LibraryScanAppService {
    pub(super) fn new(
        config: NakoServerConfig,
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
        addons: AddonAppService,
    ) -> Self {
        let workflow_store = Arc::new(store.clone());
        let execution_store = LibraryScanExecutionStore::new(store.clone());
        let metadata_scan =
            MetadataScanAcquisitionService::new(store, storage_backends.clone(), addons);
        Self {
            config,
            workflow_store,
            execution_store,
            permits,
            storage_backends,
            runtime,
            metadata_scan,
        }
    }

    pub(crate) async fn enqueue_library_scan(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_library_scan_job(library_id).await?;
        self.schedule_queued_library_scans().await?;
        Ok(job)
    }

    pub(crate) async fn schedule_queued_library_scans(&self) -> Result<LibraryScanScheduleOutcome> {
        let Some(permit) = self.try_acquire_scan_permit()? else {
            return Ok(LibraryScanScheduleOutcome::BudgetSaturated);
        };

        let runtime = DurableJobRuntime::new(self.execution_store.clone());
        let Some(leased) = runtime
            .claim_next_job_lease(JobLeaseClaimFilter {
                kind: Some(JobKind::LibraryScan),
                resource_class: Some("disk.scan".to_owned()),
                ..JobLeaseClaimFilter::default()
            })
            .await?
        else {
            return Ok(LibraryScanScheduleOutcome::NoQueuedJob);
        };
        let job_id = leased.job.id;
        let library_id = leased
            .job
            .library_id
            .ok_or_else(|| NakoError::InvalidInput {
                message: format!("library scan job {job_id} does not include library_id"),
            })?;
        let budget_class = runtime_budget_class_for_job_resource_class(
            leased.job.kind,
            &leased.job.resource_class,
        )?;
        let service = self.clone();

        self.runtime.spawn_job(
            "library_scan_background_job",
            budget_class,
            job_id,
            move |_context| {
                async move {
                    service
                        .finish_claimed_library_scan_job(leased, library_id, permit)
                        .await
                }
                .instrument(info_span!(
                    "library_scan_background_job",
                    job_id = %job_id,
                    library_id = %library_id,
                    resource_class = "disk.scan"
                ))
            },
        );

        Ok(LibraryScanScheduleOutcome::Scheduled(job_id))
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

        self.workflow_store
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

    async fn finish_claimed_library_scan_job(
        &self,
        leased: LeasedJob,
        library_id: LibraryId,
        permit: OwnedSemaphorePermit,
    ) -> Result<Job> {
        let execution = self
            .execute_claimed_library_scan_job(leased, library_id, permit)
            .await;
        self.spawn_library_scan_scheduler_followup(library_id);

        match execution? {
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

    fn spawn_library_scan_scheduler_followup(&self, library_id: LibraryId) {
        let service = self.clone();
        self.runtime.spawn(
            "library_scan_scheduler_followup",
            "disk.scan.scheduler",
            async move {
                if let Err(err) = service.schedule_queued_library_scans().await {
                    warn!(
                        library_id = %library_id,
                        error = %err,
                        "failed to schedule next queued library scan job"
                    );
                }
            },
        );
    }

    async fn acquire_scan_permit(&self) -> Result<OwnedSemaphorePermit> {
        self.permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| NakoError::InvalidInput {
                message: format!("scan concurrency limiter is unavailable: {err}"),
            })
    }

    fn try_acquire_scan_permit(&self) -> Result<Option<OwnedSemaphorePermit>> {
        match self.permits.clone().try_acquire_owned() {
            Ok(permit) => Ok(Some(permit)),
            Err(TryAcquireError::NoPermits) => Ok(None),
            Err(TryAcquireError::Closed) => Err(NakoError::InvalidInput {
                message: "scan concurrency limiter is unavailable: semaphore closed".to_owned(),
            }),
        }
    }

    async fn execute_library_scan_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<LibraryScanExecution> {
        let permit = self.acquire_scan_permit().await?;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.execution_store.clone());
        let run = runtime
            .run_job_with_context(
                job_id,
                "library scan job",
                |context| async { self.run_library_scan(job_id, library_id, context).await },
                |(index, probe, metadata)| {
                    let summary = ScanJobSummary {
                        index: index.clone(),
                        probe: probe.clone(),
                        metadata: metadata.clone(),
                    };
                    DurableJobRuntime::serialize_summary(&summary, "library scan job summary")
                },
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => {
                let (index, probe, metadata) = run.output;
                self.record_library_scanned_event(job_id, library_id, &index, &probe)
                    .await;

                Ok(LibraryScanExecution::Completed(ScanCommandOutput {
                    job: run.job,
                    index,
                    probe,
                    metadata,
                }))
            }
            DurableJobRunOutcome::Cancelled(job) => Ok(LibraryScanExecution::Cancelled(job)),
        }
    }

    async fn execute_claimed_library_scan_job(
        &self,
        leased: LeasedJob,
        library_id: LibraryId,
        permit: OwnedSemaphorePermit,
    ) -> Result<LibraryScanExecution> {
        let job_id = leased.job.id;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.execution_store.clone());
        let run = runtime
            .run_leased_job_with_context(
                leased,
                "library scan job",
                |context| async { self.run_library_scan(job_id, library_id, context).await },
                |(index, probe, metadata)| {
                    let summary = ScanJobSummary {
                        index: index.clone(),
                        probe: probe.clone(),
                        metadata: metadata.clone(),
                    };
                    DurableJobRuntime::serialize_summary(&summary, "library scan job summary")
                },
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => {
                let (index, probe, metadata) = run.output;
                self.record_library_scanned_event(job_id, library_id, &index, &probe)
                    .await;

                Ok(LibraryScanExecution::Completed(ScanCommandOutput {
                    job: run.job,
                    index,
                    probe,
                    metadata,
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
    ) -> DurableJobOperationResult<(
        LibraryIndexSummary,
        LibraryProbeSummary,
        LibraryScanMetadataSummary,
    )> {
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
        let index_service = LibraryIndexService::new(scanner, self.execution_store.clone());
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
            Arc::new(self.execution_store.store.clone()),
            StagingPurpose::ProbeInput,
            self.config.staging.max_bytes,
            self.config.staging.retention_ms,
            storage_backend.stage_permits(),
        );
        let probe = FfprobeMediaProbe::new(&self.config.ffprobe_path);
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            self.execution_store.clone(),
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
        let metadata = self
            .metadata_scan
            .run(MetadataScanAcquisitionRequest {
                job_id,
                library: &library,
                context,
            })
            .await?;

        Ok((index, probe, metadata))
    }

    async fn record_outbox_event(&self, event: NewOutboxEvent) {
        let kind = event.kind.as_str();
        let idempotency_key = event.idempotency_key.clone();
        if let Err(err) = self.workflow_store.enqueue_outbox_event(event).await {
            warn!(
                kind,
                idempotency_key,
                error = %err,
                "failed to persist outbox event"
            );
        }
    }

    async fn library_for_scan(&self, library_id: LibraryId) -> Result<Library> {
        self.workflow_store
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
    metadata: LibraryScanMetadataSummary,
}

#[derive(Clone, Debug, Serialize)]
struct LibraryScanJobInput {
    library_id: LibraryId,
    force: bool,
}
