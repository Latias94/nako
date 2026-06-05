use nako_core::{
    Job, JobId, JobKind, JobPriority, JobRepository, LibraryId, MediaRepository, MediaSource,
    MediaSourceId, NakoError, NewJob, Result,
};
use nako_db::NakoDatabase;
use nako_library::{
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, SourceFingerprintHashExecutor,
    SourceFingerprintHashJobInput, SourceFingerprintHashJobSummary, SourceFingerprintHashMode,
    SourceFingerprintHashRequest,
};
use nako_vfs::StorageUri;

use super::{job_runtime::DurableJobRuntime, storage::StorageBackendRegistry};

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
pub(crate) struct PreparedSourceFingerprintHashExecution {
    pub(crate) job_id: JobId,
    pub(crate) library_id: LibraryId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) source_scheme: String,
    pub(crate) mode: SourceFingerprintHashMode,
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
        let source = self.source_for_hash(request.source_id).await?;
        if source.library_id != request.library_id {
            return Err(NakoError::InvalidInput {
                message: "source fingerprint hash job source does not belong to requested library"
                    .to_owned(),
            });
        }

        let input = source_fingerprint_hash_job_input(&source, request.mode)?;
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
        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job(
                job_id,
                "source fingerprint hash job",
                || async { self.run_source_fingerprint_hash_job(job_id).await },
                |summary| {
                    DurableJobRuntime::serialize_summary(
                        summary,
                        "source fingerprint hash job summary",
                    )
                },
            )
            .await?;

        Ok(SourceFingerprintHashCommandOutput {
            job: run.job,
            summary: run.output,
        })
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

        Ok((
            PreparedSourceFingerprintHashExecution {
                job_id: job.id,
                library_id: input.library_id,
                source_id: input.source_id,
                source_scheme: input.source_scheme,
                mode: input.mode,
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
    ) -> Result<SourceFingerprintHashJobSummary> {
        let job = self.job_for_hash(job_id).await?;
        let (prepared, source) = self
            .prepare_source_fingerprint_hash_execution_with_source(&job)
            .await?;
        let (_uri, backend) = self
            .storage_backends
            .backend_for_media_source(&source)
            .await?;
        let executor = SourceFingerprintHashExecutor::new(backend);
        let report = executor.execute(prepared.request).await?;

        Ok(SourceFingerprintHashJobSummary::from_report(&report))
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

    async fn job_for_hash(&self, job_id: JobId) -> Result<Job> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }
}

fn source_fingerprint_hash_job_input(
    source: &MediaSource,
    mode: SourceFingerprintHashMode,
) -> Result<SourceFingerprintHashJobInput> {
    let source_uri = source_fingerprint_hash_storage_uri(source)?;

    SourceFingerprintHashJobInput::new(source.library_id, source.id, source_uri.scheme(), mode)
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

    SourceFingerprintHashJobInput::new(
        input.library_id,
        input.source_id,
        input.source_scheme,
        input.mode,
    )
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
