use nako_core::{
    EnqueueJobRetry, Job, JobId, JobKind, JobPriority, JobQueuePressureSummary, JobRepository,
    JobStatus, LeasedJob, LibraryId, MediaRepository, MediaSource, MediaSourceId, NakoError,
    NewJob, Result, ScanRepository,
};
use nako_db::NakoDatabase;
use nako_library::{
    DEFAULT_SCAN_SOURCE_FINGERPRINT_HASH_PARTIAL_PREFIX_BYTES,
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, ScanSourceFingerprintHashTrigger,
    SourceFingerprintHashExecutor, SourceFingerprintHashJobInput, SourceFingerprintHashJobSummary,
    SourceFingerprintHashMode, SourceFingerprintHashReport, SourceFingerprintHashRequest,
    SourceFingerprintHashSchedulingPolicy, source_fingerprint_hash_mode_for_decision,
};
use nako_vfs::StorageUri;

use super::{
    job_retry::canonical_retry_next_attempt,
    job_runtime::{
        DurableJobOperationError, DurableJobRunOutcome, DurableJobRuntime, DurableJobTraceContext,
    },
    storage::StorageBackendRegistry,
};
use tracing::Instrument;

#[derive(Clone, Debug)]
pub(crate) struct SourceFingerprintHashAppService {
    store: NakoDatabase,
    storage_backends: StorageBackendRegistry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnqueueSourceFingerprintHashRequest {
    pub(crate) library_id: LibraryId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) mode: SourceFingerprintHashMode,
    pub(crate) priority: Option<JobPriority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetrySourceFingerprintHashRequest {
    pub(crate) job_id: JobId,
    pub(crate) max_attempts: Option<u32>,
    pub(crate) next_attempt_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScanOriginatedSourceFingerprintHashPolicy {
    pub(crate) enabled: bool,
    pub(crate) partial_prefix_bytes: u64,
    pub(crate) priority: JobPriority,
}

impl Default for ScanOriginatedSourceFingerprintHashPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            partial_prefix_bytes: DEFAULT_SCAN_SOURCE_FINGERPRINT_HASH_PARTIAL_PREFIX_BYTES,
            priority: JobPriority::Normal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScanOriginatedSourceFingerprintHashOutcome {
    AdvisoryOnly,
    Enqueued(Job),
    AlreadyQueued(Job),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedSourceFingerprintHashExecution {
    pub(crate) job_id: JobId,
    pub(crate) library_id: LibraryId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) source_scheme: String,
    pub(crate) mode: SourceFingerprintHashMode,
    pub(crate) trace_context: Option<DurableJobTraceContext>,
    pub(crate) request: SourceFingerprintHashRequest,
}

#[derive(Clone, Debug)]
pub(crate) struct SourceFingerprintHashCommandOutput {
    pub(crate) job: Job,
    pub(crate) summary: SourceFingerprintHashJobSummary,
}

impl SourceFingerprintHashAppService {
    pub(super) fn new(store: NakoDatabase, storage_backends: StorageBackendRegistry) -> Self {
        Self {
            store,
            storage_backends,
        }
    }

    pub(crate) async fn enqueue_source_fingerprint_hash(
        &self,
        request: EnqueueSourceFingerprintHashRequest,
    ) -> Result<Job> {
        self.enqueue_source_fingerprint_hash_with_trace_context(request, None)
            .await
    }

    pub(crate) async fn enqueue_source_fingerprint_hash_with_trace_context(
        &self,
        request: EnqueueSourceFingerprintHashRequest,
        trace_context: Option<&DurableJobTraceContext>,
    ) -> Result<Job> {
        let source = self.source_for_hash(request.source_id).await?;
        if source.library_id != request.library_id {
            return Err(NakoError::InvalidInput {
                message: "source fingerprint hash job source does not belong to requested library"
                    .to_owned(),
            });
        }

        let input = source_fingerprint_hash_job_input(&source, request.mode, trace_context)?;
        let input_json = serde_json::to_string(&input).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to serialize source fingerprint hash job input: {err}"),
        })?;

        self.store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::SourceFingerprintHash,
                resource_class: SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned(),
                priority: request.priority.unwrap_or_default(),
                library_id: Some(request.library_id),
                source_id: Some(request.source_id),
                input_json: Some(input_json),
            })
            .await
    }

    pub(crate) async fn enqueue_scan_originated_source_fingerprint_hash(
        &self,
        library_id: LibraryId,
        trigger: &ScanSourceFingerprintHashTrigger,
        policy: ScanOriginatedSourceFingerprintHashPolicy,
    ) -> Result<ScanOriginatedSourceFingerprintHashOutcome> {
        self.enqueue_scan_originated_source_fingerprint_hash_with_trace_context(
            library_id, trigger, policy, None,
        )
        .await
    }

    pub(crate) async fn enqueue_scan_originated_source_fingerprint_hash_with_trace_context(
        &self,
        library_id: LibraryId,
        trigger: &ScanSourceFingerprintHashTrigger,
        policy: ScanOriginatedSourceFingerprintHashPolicy,
        trace_context: Option<&DurableJobTraceContext>,
    ) -> Result<ScanOriginatedSourceFingerprintHashOutcome> {
        if !policy.enabled {
            return Ok(ScanOriginatedSourceFingerprintHashOutcome::AdvisoryOnly);
        }

        let Some(mode) = source_fingerprint_hash_mode_for_decision(
            trigger.decision.action,
            SourceFingerprintHashSchedulingPolicy::Enabled {
                partial_prefix_bytes: policy.partial_prefix_bytes,
            },
        )?
        else {
            return Ok(ScanOriginatedSourceFingerprintHashOutcome::AdvisoryOnly);
        };

        if let Some(existing) = self
            .existing_incomplete_source_fingerprint_hash_job(library_id, trigger.source_id, mode)
            .await?
        {
            return Ok(ScanOriginatedSourceFingerprintHashOutcome::AlreadyQueued(
                existing,
            ));
        }

        self.enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id,
                source_id: trigger.source_id,
                mode,
                priority: Some(policy.priority),
            },
            trace_context,
        )
        .await
        .map(ScanOriginatedSourceFingerprintHashOutcome::Enqueued)
    }

    pub(crate) async fn retry_source_fingerprint_hash_job(
        &self,
        request: RetrySourceFingerprintHashRequest,
    ) -> Result<Job> {
        let next_attempt_at =
            canonical_source_fingerprint_hash_retry_next_attempt(&request.next_attempt_at)?;
        let source = self.job_for_hash(request.job_id).await?;
        validate_source_fingerprint_hash_job_contract(&source)?;
        let input = source_fingerprint_hash_job_input_from_job(&source)?;
        validate_source_fingerprint_hash_job_bindings(&source, &input)?;
        self.validate_source_fingerprint_hash_retry_source(&input)
            .await?;
        let max_attempts = request
            .max_attempts
            .unwrap_or_else(|| source.max_attempts.max(source.attempt.saturating_add(1)));

        self.store
            .enqueue_job_retry(EnqueueJobRetry {
                source_job_id: source.id,
                retry_job_id: JobId::new(),
                max_attempts,
                next_attempt_at,
            })
            .await
    }

    pub(crate) async fn prepare_source_fingerprint_hash_execution(
        &self,
        job: &Job,
    ) -> Result<PreparedSourceFingerprintHashExecution> {
        let (prepared, _source) = self
            .prepare_source_fingerprint_hash_execution_with_source(job)
            .await?;
        Ok(prepared)
    }

    pub(crate) async fn execute_source_fingerprint_hash_job(
        &self,
        job_id: JobId,
    ) -> Result<SourceFingerprintHashCommandOutput> {
        let job = self.job_for_hash(job_id).await?;
        let trace_context = source_fingerprint_hash_trace_context_from_job(&job);
        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job_with_trace_context(
                job_id,
                "source fingerprint hash job",
                trace_context,
                |context| async move {
                    self.run_source_fingerprint_hash_job(job_id, context.trace_context())
                        .await
                        .map_err(DurableJobOperationError::from)
                },
                source_fingerprint_hash_job_summary_json,
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => Ok(SourceFingerprintHashCommandOutput {
                job: run.job,
                summary: run.output,
            }),
            DurableJobRunOutcome::Cancelled(job) => Err(NakoError::Conflict {
                message: format!("job {} was cancelled", job.id),
            }),
        }
    }

    pub(crate) async fn execute_claimed_source_fingerprint_hash_job(
        &self,
        leased: LeasedJob,
    ) -> Result<SourceFingerprintHashCommandOutput> {
        let job = leased.job.clone();
        let trace_context = source_fingerprint_hash_trace_context_from_job(&job);
        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_leased_job_with_trace_context(
                leased,
                "source fingerprint hash job",
                trace_context,
                |context| async move {
                    self.run_source_fingerprint_hash_job_from_job(&job, context.trace_context())
                        .await
                        .map_err(DurableJobOperationError::from)
                },
                source_fingerprint_hash_job_summary_json,
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => Ok(SourceFingerprintHashCommandOutput {
                job: run.job,
                summary: run.output,
            }),
            DurableJobRunOutcome::Cancelled(job) => Err(NakoError::Conflict {
                message: format!("job {} was cancelled", job.id),
            }),
        }
    }

    pub(crate) async fn admin_overview_summary(
        &self,
    ) -> Result<nako_api::admin::AdminOverviewSourceFingerprintHashSummary> {
        let source_summary = self.store.summarize_media_source_fingerprints().await?;
        let queue_pressure = self.store.summarize_job_queue_pressure().await?;
        let mut summary = nako_api::admin::AdminOverviewSourceFingerprintHashSummary {
            total_sources: source_summary.total_sources,
            fingerprinted_sources: source_summary.fingerprinted_sources,
            content_hash_sources: source_summary.content_hash_sources,
            ..Default::default()
        };

        for pressure in queue_pressure
            .iter()
            .filter(|pressure| source_fingerprint_hash_queue_pressure(pressure))
        {
            match pressure.status {
                JobStatus::Queued => {
                    summary.queued_jobs += pressure.count;
                    summary.claimable_jobs += pressure.claimable_count;
                    summary.delayed_retry_jobs += pressure.delayed_retry_count;
                    update_earliest_timestamp(
                        &mut summary.oldest_queued_at,
                        &pressure.oldest_queued_at,
                    );
                    update_earliest_timestamp(
                        &mut summary.next_retry_at,
                        &pressure.next_attempt_at,
                    );
                }
                JobStatus::Running => {
                    summary.running_jobs += pressure.count;
                }
                JobStatus::Succeeded => {
                    summary.succeeded_jobs += pressure.count;
                }
                JobStatus::Failed => {
                    summary.failed_jobs += pressure.count;
                }
                JobStatus::Cancelled => {
                    summary.cancelled_jobs += pressure.count;
                }
            }
        }

        Ok(summary)
    }

    async fn prepare_source_fingerprint_hash_execution_with_source(
        &self,
        job: &Job,
    ) -> Result<(PreparedSourceFingerprintHashExecution, MediaSource)> {
        validate_source_fingerprint_hash_job_contract(job)?;
        let input = source_fingerprint_hash_job_input_from_job(job)?;
        validate_source_fingerprint_hash_job_bindings(job, &input)?;

        let source = self.source_for_hash(input.source_id).await?;
        if source.library_id != input.library_id {
            return Err(NakoError::Conflict {
                message: "source fingerprint hash job source no longer belongs to input library"
                    .to_owned(),
            });
        }

        let source_uri = source_fingerprint_hash_storage_uri(&source)?;
        if source_uri.scheme() != input.source_scheme {
            return Err(NakoError::Conflict {
                message: "source fingerprint hash job source locator scheme changed since enqueue"
                    .to_owned(),
            });
        }

        let trace_context = source_fingerprint_hash_trace_context(&input.request_id)?;

        Ok((
            PreparedSourceFingerprintHashExecution {
                job_id: job.id,
                library_id: input.library_id,
                source_id: input.source_id,
                source_scheme: input.source_scheme,
                mode: input.mode,
                trace_context,
                request: SourceFingerprintHashRequest {
                    uri: source_uri,
                    mode: input.mode,
                },
            },
            source,
        ))
    }

    async fn run_source_fingerprint_hash_job(
        &self,
        job_id: JobId,
        trace_context: Option<&DurableJobTraceContext>,
    ) -> Result<SourceFingerprintHashJobSummary> {
        let job = self.job_for_hash(job_id).await?;
        self.run_source_fingerprint_hash_job_from_job(&job, trace_context)
            .await
    }

    async fn run_source_fingerprint_hash_job_from_job(
        &self,
        job: &Job,
        trace_context: Option<&DurableJobTraceContext>,
    ) -> Result<SourceFingerprintHashJobSummary> {
        let (prepared, source) = self
            .prepare_source_fingerprint_hash_execution_with_source(job)
            .await?;
        let (_uri, backend) = self
            .storage_backends
            .backend_for_media_source(&source)
            .await?;
        let executor = SourceFingerprintHashExecutor::new(backend);
        let source_scheme = prepared.source_scheme.clone();
        let trace_request_id = trace_context
            .or(prepared.trace_context.as_ref())
            .map(DurableJobTraceContext::request_id)
            .unwrap_or("untraced");
        let report = async {
            executor
                .execute(prepared.request)
                .await
                .map_err(|err| redact_source_fingerprint_hash_execution_error(err, &source_scheme))
        }
        .instrument(tracing::info_span!(
            "source_fingerprint_hash_job",
            job_id = %prepared.job_id,
            library_id = %prepared.library_id,
            source_id = %prepared.source_id,
            request_id = %trace_request_id,
            mode = ?prepared.mode,
        ))
        .await?;
        self.persist_source_fingerprint_hash_evidence(&source, &report)
            .await?;

        Ok(SourceFingerprintHashJobSummary::from_report(&report))
    }

    async fn persist_source_fingerprint_hash_evidence(
        &self,
        source: &MediaSource,
        report: &SourceFingerprintHashReport,
    ) -> Result<()> {
        let fingerprint = report
            .evidence
            .fingerprint
            .clone()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| NakoError::Conflict {
                message: "source fingerprint hash report did not include fingerprint evidence"
                    .to_owned(),
            })?;

        let mut source = source.clone();
        source.fingerprint = Some(fingerprint.clone());

        if let Some(mut state) = self
            .store
            .get_source_state(source.library_id, &source.locator)
            .await?
        {
            state.fingerprint = Some(fingerprint);
            self.store
                .commit_library_scan_source(&nako_core::LibraryScanSourcePersistenceCommit {
                    items: Vec::new(),
                    source,
                    source_state: state,
                    library_item_states: Vec::new(),
                    local_inference_evidence: Vec::new(),
                    search_projections: Vec::new(),
                    source_duplicate_relationships: Vec::new(),
                    resolved_ingestion_failures: Vec::new(),
                })
                .await?;
        } else {
            self.store.upsert_media_source(&source).await?;
        }

        Ok(())
    }

    async fn source_for_hash(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }

    async fn validate_source_fingerprint_hash_retry_source(
        &self,
        input: &SourceFingerprintHashJobInput,
    ) -> Result<()> {
        let source = self.source_for_hash(input.source_id).await?;
        if source.library_id != input.library_id {
            return Err(NakoError::Conflict {
                message: "source fingerprint hash retry source no longer belongs to input library"
                    .to_owned(),
            });
        }

        let source_uri = source_fingerprint_hash_storage_uri(&source)?;
        if source_uri.scheme() != input.source_scheme {
            return Err(NakoError::Conflict {
                message:
                    "source fingerprint hash retry source locator scheme changed since enqueue"
                        .to_owned(),
            });
        }

        Ok(())
    }

    async fn job_for_hash(&self, job_id: JobId) -> Result<Job> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }

    async fn existing_incomplete_source_fingerprint_hash_job(
        &self,
        library_id: LibraryId,
        source_id: MediaSourceId,
        mode: SourceFingerprintHashMode,
    ) -> Result<Option<Job>> {
        for status in [JobStatus::Queued, JobStatus::Running] {
            let jobs = self
                .store
                .list_jobs(
                    nako_core::JobListFilter {
                        status: Some(status),
                        kind: Some(JobKind::SourceFingerprintHash),
                        resource_class: Some(SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS.to_owned()),
                        library_id: Some(library_id),
                        source_id: Some(source_id),
                    },
                    nako_core::PageRequest::new(nako_core::PageRequest::MAX_LIMIT, 0),
                )
                .await?;

            for job in jobs {
                if source_fingerprint_hash_job_input_from_job(&job)
                    .is_ok_and(|input| input.mode == mode)
                {
                    return Ok(Some(job));
                }
            }
        }

        Ok(None)
    }
}

fn source_fingerprint_hash_job_summary_json(
    summary: &SourceFingerprintHashJobSummary,
) -> Result<Option<String>> {
    DurableJobRuntime::serialize_summary(summary, "source fingerprint hash job summary")
}

fn source_fingerprint_hash_queue_pressure(pressure: &JobQueuePressureSummary) -> bool {
    pressure.kind == JobKind::SourceFingerprintHash
        && pressure.resource_class == SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS
}

fn update_earliest_timestamp(current: &mut Option<String>, candidate: &Option<String>) {
    if let Some(candidate) = candidate {
        match current {
            Some(existing) if existing.as_str() <= candidate.as_str() => {}
            _ => *current = Some(candidate.clone()),
        }
    }
}

fn canonical_source_fingerprint_hash_retry_next_attempt(
    next_attempt_at: &Option<String>,
) -> Result<Option<String>> {
    canonical_retry_next_attempt(
        next_attempt_at,
        "source fingerprint hash retry next_attempt_at must be an RFC3339 timestamp",
        "source fingerprint hash retry next_attempt_at could not be canonicalized",
    )
}

fn redact_source_fingerprint_hash_execution_error(
    err: NakoError,
    source_scheme: &str,
) -> NakoError {
    match err {
        NakoError::Storage { kind, .. } => NakoError::Storage {
            uri: format!("{source_scheme}://<redacted>"),
            kind,
            message: kind.failure_class().safe_message().to_owned(),
        },
        _ => NakoError::Conflict {
            message: "source fingerprint hash execution failed".to_owned(),
        },
    }
}

fn source_fingerprint_hash_job_input(
    source: &MediaSource,
    mode: SourceFingerprintHashMode,
    trace_context: Option<&DurableJobTraceContext>,
) -> Result<SourceFingerprintHashJobInput> {
    let source_uri = source_fingerprint_hash_storage_uri(source)?;
    let mut input = SourceFingerprintHashJobInput::new(
        source.library_id,
        source.id,
        source_uri.scheme(),
        mode,
    )?;
    input.request_id = trace_context.map(|trace_context| trace_context.request_id().to_owned());
    Ok(input)
}

fn source_fingerprint_hash_storage_uri(source: &MediaSource) -> Result<StorageUri> {
    StorageUri::parse(&source.locator).map_err(|_err| NakoError::InvalidInput {
        message: "source fingerprint hash job source locator is not a valid storage URI".to_owned(),
    })
}

fn validate_source_fingerprint_hash_job_contract(job: &Job) -> Result<()> {
    if job.kind != JobKind::SourceFingerprintHash {
        return Err(NakoError::InvalidInput {
            message: "job is not a source fingerprint hash job".to_owned(),
        });
    }
    if job.resource_class != SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS {
        return Err(NakoError::InvalidInput {
            message: "source fingerprint hash job uses unsupported resource class".to_owned(),
        });
    }

    Ok(())
}

fn source_fingerprint_hash_job_input_from_job(job: &Job) -> Result<SourceFingerprintHashJobInput> {
    let input_json = job
        .input_json
        .as_deref()
        .ok_or_else(|| NakoError::InvalidInput {
            message: "source fingerprint hash job input is missing".to_owned(),
        })?;
    let input: SourceFingerprintHashJobInput =
        serde_json::from_str(input_json).map_err(|_err| NakoError::InvalidInput {
            message: "source fingerprint hash job input is invalid".to_owned(),
        })?;
    let SourceFingerprintHashJobInput {
        library_id,
        source_id,
        source_scheme,
        mode,
        request_id,
    } = input;
    let validated = SourceFingerprintHashJobInput::new(library_id, source_id, source_scheme, mode)?;

    Ok(SourceFingerprintHashJobInput {
        request_id,
        ..validated
    })
}

fn source_fingerprint_hash_trace_context(
    request_id: &Option<String>,
) -> Result<Option<DurableJobTraceContext>> {
    request_id
        .as_deref()
        .map(DurableJobTraceContext::from_request_id)
        .transpose()
}

fn source_fingerprint_hash_trace_context_from_job(job: &Job) -> Option<DurableJobTraceContext> {
    let Some(input_json) = job.input_json.as_deref() else {
        return None;
    };
    let Ok(input) = serde_json::from_str::<serde_json::Value>(input_json) else {
        return None;
    };
    let Some(request_id) = input.get("request_id").and_then(serde_json::Value::as_str) else {
        return None;
    };

    DurableJobTraceContext::from_request_id(request_id).ok()
}

fn validate_source_fingerprint_hash_job_bindings(
    job: &Job,
    input: &SourceFingerprintHashJobInput,
) -> Result<()> {
    if job
        .library_id
        .is_some_and(|library_id| library_id != input.library_id)
    {
        return Err(NakoError::InvalidInput {
            message: "source fingerprint hash job library binding does not match input".to_owned(),
        });
    }
    if job
        .source_id
        .is_some_and(|source_id| source_id != input.source_id)
    {
        return Err(NakoError::InvalidInput {
            message: "source fingerprint hash job source binding does not match input".to_owned(),
        });
    }

    Ok(())
}
