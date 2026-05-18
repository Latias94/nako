use async_trait::async_trait;

use super::PageRequest;
use crate::{
    ArtworkTask, ArtworkTaskId, ExternalProvider, JobId, LocalInferenceEvidence,
    LocalInferenceEvidenceId, MediaItem, MediaItemId, MediaSourceId, MetadataAttemptFilter,
    MetadataFieldLock, MetadataProviderAttemptRecord, MetadataRefreshPersistenceCommit,
    MetadataRefreshPersistenceSummary, NewMetadataProviderAttempt, ProviderMapping,
    ProviderRawResponse, ProviderRawResponseCleanup, ProviderRawResponseFilter, ProviderSubject,
    ProviderSubjectId, ProviderSubjectKind, Result, SourceDuplicateRelationship,
    SourceDuplicateRelationshipId,
};

#[async_trait]
pub trait ArtworkTaskRepository: Send + Sync {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()>;

    async fn get_artwork_task(&self, id: ArtworkTaskId) -> Result<Option<ArtworkTask>>;

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>>;
}

#[async_trait]
pub trait MetadataRepository: Send + Sync {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()>;

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>>;

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()>;

    async fn commit_metadata_refresh(
        &self,
        commit: &MetadataRefreshPersistenceCommit,
    ) -> Result<MetadataRefreshPersistenceSummary>;

    async fn commit_metadata_item(&self, item: &MediaItem) -> Result<()>;

    async fn get_provider_raw_response(
        &self,
        item_id: MediaItemId,
        provider: &ExternalProvider,
        provider_key: &str,
    ) -> Result<Option<ProviderRawResponse>>;

    async fn list_provider_raw_responses(
        &self,
        item_id: MediaItemId,
        filter: ProviderRawResponseFilter,
        page: PageRequest,
    ) -> Result<Vec<ProviderRawResponse>>;

    async fn cleanup_provider_raw_responses(
        &self,
        filter: ProviderRawResponseFilter,
        fetched_before: &str,
    ) -> Result<ProviderRawResponseCleanup>;

    async fn insert_metadata_provider_attempt(
        &self,
        attempt: NewMetadataProviderAttempt,
    ) -> Result<()>;

    async fn list_metadata_provider_attempts(
        &self,
        job_id: JobId,
    ) -> Result<Vec<MetadataProviderAttemptRecord>>;

    async fn list_metadata_provider_attempts_for_item(
        &self,
        item_id: MediaItemId,
        filter: MetadataAttemptFilter,
        page: PageRequest,
    ) -> Result<Vec<MetadataProviderAttemptRecord>>;
}

#[async_trait]
pub trait ProviderMappingRepository: Send + Sync {
    async fn upsert_provider_subject(&self, subject: &ProviderSubject) -> Result<()>;

    async fn get_provider_subject(&self, id: ProviderSubjectId) -> Result<Option<ProviderSubject>>;

    async fn find_provider_subject(
        &self,
        provider: &ExternalProvider,
        subject_kind: &ProviderSubjectKind,
        subject_key: &str,
    ) -> Result<Option<ProviderSubject>>;

    async fn list_provider_subjects_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderSubject>>;

    async fn upsert_provider_mapping(&self, mapping: &ProviderMapping) -> Result<()>;

    async fn list_provider_mappings_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderMapping>>;
}

#[async_trait]
pub trait SourceDuplicateRepository: Send + Sync {
    async fn upsert_source_duplicate_relationship(
        &self,
        relationship: &SourceDuplicateRelationship,
    ) -> Result<()>;

    async fn get_source_duplicate_relationship(
        &self,
        id: SourceDuplicateRelationshipId,
    ) -> Result<Option<SourceDuplicateRelationship>>;

    async fn list_source_duplicate_relationships(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<SourceDuplicateRelationship>>;
}

#[async_trait]
pub trait LocalInferenceRepository: Send + Sync {
    async fn upsert_local_inference_evidence(
        &self,
        evidence: &LocalInferenceEvidence,
    ) -> Result<()>;

    async fn get_local_inference_evidence(
        &self,
        id: LocalInferenceEvidenceId,
    ) -> Result<Option<LocalInferenceEvidence>>;

    async fn list_local_inference_evidence_for_source(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<LocalInferenceEvidence>>;
}
