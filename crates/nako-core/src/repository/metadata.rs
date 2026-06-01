use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AddonId, AddonMetadataWritePersistenceCommit, AddonMetadataWritePersistenceSummary,
    ArtworkCandidateRecord, ArtworkCandidateSourceKind, ArtworkCandidateStatus, ArtworkTask,
    ArtworkTaskId, ExternalProvider, ImageKind, JobId, LibraryId, LocalInferenceEvidence,
    LocalInferenceEvidenceId, ManagedArtworkAcceptanceRecord, ManagedArtworkArtifactCleanupReport,
    ManagedArtworkArtifactId, ManagedArtworkArtifactLifecycleFilter,
    ManagedArtworkArtifactLifecycleSnapshot, ManagedArtworkArtifactRecord,
    ManagedArtworkGallerySnapshot, ManagedArtworkIngestClaimRecord, ManagedArtworkIngestId,
    ManagedArtworkIngestProcessingRecord, ManagedArtworkIngestRecord,
    ManagedArtworkIngestRequeueRecord, MediaItem, MediaItemId, MediaSourceId,
    MetadataApplicationPersistenceCommit, MetadataApplicationPersistenceSummary,
    MetadataAttemptFilter, MetadataCandidateReviewId, MetadataCandidateReviewRecord,
    MetadataCandidateSource, MetadataFieldLock, MetadataProviderAttemptRecord,
    MetadataRefreshPersistenceCommit, MetadataRefreshPersistenceSummary, NewArtworkCandidate,
    NewJob, NewManagedArtworkArtifact, NewManagedArtworkIngest, NewMetadataCandidateReview,
    NewMetadataProviderAttempt, NfoImportPersistenceCommit, NfoImportPersistenceSummary,
    ProviderMapping, ProviderRawResponse, ProviderRawResponseCleanup, ProviderRawResponseFilter,
    ProviderSubject, ProviderSubjectId, ProviderSubjectKind, Result, SelectedArtworkId,
    SelectedArtworkPublicationRecord, SelectedArtworkRecord, SelectedArtworkUnpublicationRecord,
    SourceDuplicateRelationship, SourceDuplicateRelationshipId,
};

#[async_trait]
pub trait ArtworkTaskRepository: Send + Sync {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()>;

    async fn get_artwork_task(&self, id: ArtworkTaskId) -> Result<Option<ArtworkTask>>;

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>>;
}

#[async_trait]
pub trait ArtworkCandidateRepository: Send + Sync {
    async fn create_artwork_candidate(
        &self,
        candidate: NewArtworkCandidate,
    ) -> Result<ArtworkCandidateRecord>;

    async fn get_artwork_candidate(
        &self,
        id: crate::ArtworkCandidateId,
    ) -> Result<Option<ArtworkCandidateRecord>>;

    async fn set_artwork_candidate_status(
        &self,
        id: crate::ArtworkCandidateId,
        status: ArtworkCandidateStatus,
    ) -> Result<ArtworkCandidateRecord>;

    async fn find_artwork_candidate_by_source(
        &self,
        addon_id: AddonId,
        library_id: LibraryId,
        item_id: MediaItemId,
        kind: &ImageKind,
        source_kind: ArtworkCandidateSourceKind,
        source_uri: &str,
    ) -> Result<Option<ArtworkCandidateRecord>>;

    async fn list_artwork_candidates_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ArtworkCandidateRecord>>;
}

#[async_trait]
pub trait ManagedArtworkRepository: Send + Sync {
    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: crate::ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord>;

    async fn get_managed_artwork_ingest(
        &self,
        id: ManagedArtworkIngestId,
    ) -> Result<Option<ManagedArtworkIngestRecord>>;

    async fn find_managed_artwork_ingest_by_candidate(
        &self,
        candidate_id: crate::ArtworkCandidateId,
    ) -> Result<Option<ManagedArtworkIngestRecord>>;

    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>>;

    async fn commit_managed_artwork_artifact(
        &self,
        ingest_id: ManagedArtworkIngestId,
        artifact: NewManagedArtworkArtifact,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord>;

    async fn fail_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord>;

    async fn fail_unfinished_managed_artwork_ingests(
        &self,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<u64>;

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord>;

    async fn get_managed_artwork_artifact(
        &self,
        id: ManagedArtworkArtifactId,
    ) -> Result<Option<ManagedArtworkArtifactRecord>>;

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord>;

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord>;

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord>;

    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>>;

    async fn list_selected_artwork_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<SelectedArtworkRecord>>;

    async fn get_managed_artwork_gallery_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<ManagedArtworkGallerySnapshot>;

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot>;

    async fn cleanup_unselected_managed_artwork_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactCleanupReport>;
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

    async fn commit_nfo_import(
        &self,
        commit: &NfoImportPersistenceCommit,
    ) -> Result<NfoImportPersistenceSummary>;

    async fn commit_addon_metadata_write(
        &self,
        commit: &AddonMetadataWritePersistenceCommit,
    ) -> Result<AddonMetadataWritePersistenceSummary>;

    async fn commit_metadata_application(
        &self,
        commit: &MetadataApplicationPersistenceCommit,
    ) -> Result<MetadataApplicationPersistenceSummary>;

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
pub trait MetadataCandidateReviewRepository: Send + Sync {
    async fn upsert_metadata_candidate_review(
        &self,
        review: NewMetadataCandidateReview,
    ) -> Result<MetadataCandidateReviewRecord>;

    async fn get_metadata_candidate_review(
        &self,
        id: MetadataCandidateReviewId,
    ) -> Result<Option<MetadataCandidateReviewRecord>>;

    async fn find_metadata_candidate_review(
        &self,
        item_id: MediaItemId,
        source: &MetadataCandidateSource,
        source_key: &str,
    ) -> Result<Option<MetadataCandidateReviewRecord>>;

    async fn list_metadata_candidate_reviews_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MetadataCandidateReviewRecord>>;
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
