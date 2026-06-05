use nako_core::{
    Job, JobId, JobKind, JobPriority, JobRepository, LibraryId, MediaRepository, MediaSource,
    MediaSourceId, NakoError, NewJob, Result,
};
use nako_db::NakoDatabase;
use nako_library::{
    SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS, SourceFingerprintHashJobInput,
    SourceFingerprintHashMode,
};
use nako_vfs::StorageUri;

#[derive(Clone, Debug)]
pub(crate) struct SourceFingerprintHashAppService {
    store: NakoDatabase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnqueueSourceFingerprintHashRequest {
    pub(crate) library_id: LibraryId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) mode: SourceFingerprintHashMode,
    pub(crate) priority: Option<JobPriority>,
}

impl SourceFingerprintHashAppService {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
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

    async fn source_for_hash(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }
}

fn source_fingerprint_hash_job_input(
    source: &MediaSource,
    mode: SourceFingerprintHashMode,
) -> Result<SourceFingerprintHashJobInput> {
    let source_uri =
        StorageUri::parse(&source.locator).map_err(|_err| NakoError::InvalidInput {
            message: "source fingerprint hash job source locator is not a valid storage URI"
                .to_owned(),
        })?;

    SourceFingerprintHashJobInput::new(source.library_id, source.id, source_uri.scheme(), mode)
}
