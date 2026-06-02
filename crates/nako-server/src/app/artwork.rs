use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use nako_api::admin::{
    AcceptManagedArtworkCandidateResponse, AdminManagedArtworkArtifactCleanupResponse,
    AdminManagedArtworkArtifactFileCleanupSummary, AdminManagedArtworkArtifactLifecycleResponse,
    AdminManagedArtworkArtifactRemediationMissingArtifact,
    AdminManagedArtworkArtifactRemediationPlanResponse,
    AdminManagedArtworkArtifactRemediationStrayFile, AdminManagedArtworkArtifactRemediationSummary,
    AdminManagedArtworkArtifactStorageDriftArtifact,
    AdminManagedArtworkArtifactStorageDriftArtifactIssue,
    AdminManagedArtworkArtifactStorageDriftFile, AdminManagedArtworkArtifactStorageDriftFileReason,
    AdminManagedArtworkArtifactStorageDriftResponse,
    AdminManagedArtworkArtifactStorageDriftSummary,
    AdminManagedArtworkArtifactStrayFileCleanupItem,
    AdminManagedArtworkArtifactStrayFileCleanupResponse,
    AdminManagedArtworkArtifactStrayFileCleanupStatus,
    AdminManagedArtworkArtifactStrayFileCleanupSummary,
    AdminManagedArtworkArtifactStrayFileRemediationAction, AdminManagedArtworkGalleryResponse,
    ProcessManagedArtworkIngestResponse, PublishSelectedArtworkResponse,
    RequeueManagedArtworkIngestResponse, UnpublishSelectedArtworkResponse,
};
use nako_api::public_client::page_info_from_request;
use nako_core::{
    ArtworkCandidateId, ArtworkCandidateRecord, ArtworkCandidateRepository, ArtworkCandidateStatus,
    JobId, JobKind, JobPriority, LibraryItemRepository, LibraryItemState,
    ManagedArtworkAcceptanceRecord, ManagedArtworkArtifactCleanupReport, ManagedArtworkArtifactId,
    ManagedArtworkArtifactLifecycleFilter, ManagedArtworkArtifactLifecycleSnapshot,
    ManagedArtworkArtifactRecord, ManagedArtworkGallerySnapshot, ManagedArtworkIngestClaimRecord,
    ManagedArtworkIngestId, ManagedArtworkIngestProcessingRecord,
    ManagedArtworkIngestRequeueRecord, ManagedArtworkIngestStatus, ManagedArtworkRepository,
    MediaItem, MediaItemId, MediaRepository, NakoError, NewJob, NewManagedArtworkArtifact,
    NewManagedArtworkIngest, PageRequest, Result, SelectedArtworkId,
    SelectedArtworkPublicationRecord, SelectedArtworkRecord, SelectedArtworkUnpublicationRecord,
};
use nako_db::NakoDatabase;
use serde::Serialize;
use tracing::{info, warn};

use crate::config::ArtworkConfig;

use super::runtime::RuntimeSupervisor;
use artifact_store::{
    ArtifactFileDeleteOutcome, ArtifactFileStatus, ArtifactStoreFileIssue,
    ClassifiedArtifactStoreFile, DiscoveredArtifactFile, DiscoveredArtifactFileLayout,
    LocalManagedArtworkArtifactStore,
};
use ingest_pipeline::{ManagedArtworkFailure, ManagedArtworkIngestPipeline};
use variant::ImageVariantPolicy;

mod artifact_store;
mod ingest_pipeline;
mod variant;

pub(crate) use variant::{ImageVariantRequest, ManagedArtworkImageBytes};

#[async_trait]
trait ArtworkAcceptanceWorkflowStore: std::fmt::Debug + Send + Sync {
    async fn get_artwork_candidate(
        &self,
        id: ArtworkCandidateId,
    ) -> Result<Option<ArtworkCandidateRecord>>;

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn get_library_item_state(
        &self,
        library_id: nako_core::LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>>;

    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord>;
}

#[async_trait]
trait ArtworkSelectionWorkflowStore: std::fmt::Debug + Send + Sync {
    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>>;

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord>;

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: nako_core::ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord>;

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: nako_core::ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord>;
}

#[async_trait]
trait ArtworkIngestWorkflowStore: std::fmt::Debug + Send + Sync {
    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>>;

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord>;

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
}

#[async_trait]
trait ArtworkLifecycleWorkflowStore: std::fmt::Debug + Send + Sync {
    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>>;

    async fn get_managed_artwork_artifact(
        &self,
        id: ManagedArtworkArtifactId,
    ) -> Result<Option<ManagedArtworkArtifactRecord>>;

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
impl<T> ArtworkAcceptanceWorkflowStore for T
where
    T: ArtworkCandidateRepository
        + LibraryItemRepository
        + ManagedArtworkRepository
        + MediaRepository
        + std::fmt::Debug
        + Send
        + Sync,
{
    async fn get_artwork_candidate(
        &self,
        id: ArtworkCandidateId,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        ArtworkCandidateRepository::get_artwork_candidate(self, id).await
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        MediaRepository::get_media_item(self, id).await
    }

    async fn get_library_item_state(
        &self,
        library_id: nako_core::LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>> {
        LibraryItemRepository::get_library_item_state(self, library_id, item_id).await
    }

    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord> {
        ManagedArtworkRepository::accept_managed_artwork_candidate_ingest(
            self,
            candidate_id,
            ingest,
            job,
        )
        .await
    }
}

#[async_trait]
impl<T> ArtworkSelectionWorkflowStore for T
where
    T: ManagedArtworkRepository + MediaRepository + std::fmt::Debug + Send + Sync,
{
    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        MediaRepository::get_media_item(self, id).await
    }

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        ManagedArtworkRepository::publish_selected_artwork(self, artifact_id).await
    }

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: nako_core::ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        ManagedArtworkRepository::publish_selected_artwork_for_item_kind(
            self,
            item_id,
            kind,
            artifact_id,
        )
        .await
    }

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: nako_core::ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord> {
        ManagedArtworkRepository::unpublish_selected_artwork_for_item_kind(self, item_id, kind)
            .await
    }
}

#[async_trait]
impl<T> ArtworkIngestWorkflowStore for T
where
    T: ManagedArtworkRepository + std::fmt::Debug + Send + Sync,
{
    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>> {
        ManagedArtworkRepository::claim_next_queued_managed_artwork_ingest(self).await
    }

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord> {
        ManagedArtworkRepository::requeue_managed_artwork_ingest(self, ingest_id).await
    }

    async fn commit_managed_artwork_artifact(
        &self,
        ingest_id: ManagedArtworkIngestId,
        artifact: NewManagedArtworkArtifact,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        ManagedArtworkRepository::commit_managed_artwork_artifact(
            self,
            ingest_id,
            artifact,
            job_summary_json,
        )
        .await
    }

    async fn fail_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        ManagedArtworkRepository::fail_managed_artwork_ingest(
            self,
            ingest_id,
            failure_code,
            job_error,
            job_summary_json,
        )
        .await
    }
}

#[async_trait]
impl<T> ArtworkLifecycleWorkflowStore for T
where
    T: ManagedArtworkRepository + std::fmt::Debug + Send + Sync,
{
    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>> {
        ManagedArtworkRepository::get_selected_artwork(self, id).await
    }

    async fn get_managed_artwork_artifact(
        &self,
        id: ManagedArtworkArtifactId,
    ) -> Result<Option<ManagedArtworkArtifactRecord>> {
        ManagedArtworkRepository::get_managed_artwork_artifact(self, id).await
    }

    async fn get_managed_artwork_gallery_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<ManagedArtworkGallerySnapshot> {
        ManagedArtworkRepository::get_managed_artwork_gallery_for_item(self, item_id, page).await
    }

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot> {
        ManagedArtworkRepository::list_managed_artwork_artifact_lifecycle(self, filter, page).await
    }

    async fn cleanup_unselected_managed_artwork_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactCleanupReport> {
        ManagedArtworkRepository::cleanup_unselected_managed_artwork_artifacts(self, page).await
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedArtworkAppService {
    acceptance_store: Arc<dyn ArtworkAcceptanceWorkflowStore>,
    selection_store: Arc<dyn ArtworkSelectionWorkflowStore>,
    ingest_store: Arc<dyn ArtworkIngestWorkflowStore>,
    lifecycle_store: Arc<dyn ArtworkLifecycleWorkflowStore>,
    ingest_pipeline: ManagedArtworkIngestPipeline,
    artifact_store: LocalManagedArtworkArtifactStore,
    variant_policy: ImageVariantPolicy,
    ingest_worker_idle: Duration,
}

impl ManagedArtworkAppService {
    pub(crate) fn new(config: ArtworkConfig, store: NakoDatabase) -> Result<Self> {
        let acceptance_store = Arc::new(store.clone());
        let selection_store = Arc::new(store.clone());
        let ingest_store = Arc::new(store.clone());
        let lifecycle_store = Arc::new(store);
        Ok(Self {
            acceptance_store,
            selection_store,
            ingest_store,
            lifecycle_store,
            ingest_pipeline: ManagedArtworkIngestPipeline::new(config.clone())?,
            variant_policy: ImageVariantPolicy::new(config.max_width, config.max_height),
            artifact_store: LocalManagedArtworkArtifactStore::new(config.artifact_root),
            ingest_worker_idle: Duration::from_millis(config.ingest_worker_idle_ms.max(1)),
        })
    }

    pub(super) fn start_ingest_worker(&self, runtime: &RuntimeSupervisor) -> bool {
        let app = self.clone();
        let token = runtime.shutdown_token();
        runtime.spawn(
            "managed_artwork_ingest_worker",
            "artwork.ingest",
            async move {
                loop {
                    tokio::select! {
                        () = token.cancelled() => break,
                        result = app.process_next_unit() => {
                            match result {
                                Ok(Some(processing)) => {
                                    info!(
                                        ingest_id = %processing.ingest.id,
                                        job_id = %processing.job.id,
                                        status = ?processing.ingest.status,
                                        "managed artwork ingest worker processed job"
                                    );
                                }
                                Ok(None) => {
                                    tokio::select! {
                                        () = token.cancelled() => break,
                                        () = tokio::time::sleep(app.ingest_worker_idle) => {}
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        error = %err,
                                        "managed artwork ingest worker iteration failed"
                                    );
                                    tokio::select! {
                                        () = token.cancelled() => break,
                                        () = tokio::time::sleep(app.ingest_worker_idle) => {}
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );
        true
    }

    pub(crate) async fn accept_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<AcceptManagedArtworkCandidateResponse> {
        let candidate = self
            .acceptance_store
            .get_artwork_candidate(candidate_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "artwork_candidate",
                id: candidate_id.to_string(),
            })?;

        if candidate.status == ArtworkCandidateStatus::Rejected {
            return Err(NakoError::InvalidInput {
                message: "rejected artwork candidates cannot be accepted".to_owned(),
            });
        }

        self.acceptance_store
            .get_media_item(candidate.item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: candidate.item_id.to_string(),
            })?;
        self.acceptance_store
            .get_library_item_state(candidate.library_id, candidate.item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library_item_state",
                id: format!("{}:{}", candidate.library_id, candidate.item_id),
            })?;

        let job_id = JobId::new();
        let input = ManagedArtworkIngestJobInput {
            candidate_id,
            library_id: candidate.library_id,
            item_id: candidate.item_id,
            image_kind: image_kind_label(&candidate.kind).to_owned(),
        };
        let input_json = serde_json::to_string(&input).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to serialize managed artwork ingest job input: {err}"),
        })?;
        let acceptance = self
            .acceptance_store
            .accept_managed_artwork_candidate_ingest(
                candidate_id,
                NewManagedArtworkIngest {
                    id: ManagedArtworkIngestId::new(),
                    candidate_id,
                    job_id,
                    library_id: candidate.library_id,
                    item_id: candidate.item_id,
                    kind: candidate.kind,
                    status: ManagedArtworkIngestStatus::Queued,
                    artifact_id: None,
                    failure_code: None,
                },
                NewJob {
                    id: job_id,
                    kind: JobKind::ManagedArtworkIngest,
                    resource_class: "artwork.ingest".to_owned(),
                    priority: JobPriority::Normal,
                    library_id: Some(candidate.library_id),
                    source_id: None,
                    input_json: Some(input_json),
                },
            )
            .await?;

        Ok(AcceptManagedArtworkCandidateResponse::from_acceptance(
            acceptance,
        ))
    }

    pub(crate) async fn process_next(&self) -> Result<ProcessManagedArtworkIngestResponse> {
        Ok(self
            .process_next_unit()
            .await?
            .map(ProcessManagedArtworkIngestResponse::from_processing)
            .unwrap_or_else(ProcessManagedArtworkIngestResponse::empty))
    }

    pub(crate) async fn requeue_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<RequeueManagedArtworkIngestResponse> {
        let requeue = self
            .ingest_store
            .requeue_managed_artwork_ingest(ingest_id)
            .await?;
        Ok(RequeueManagedArtworkIngestResponse::from_requeue(requeue))
    }

    pub(crate) async fn publish_artifact(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<PublishSelectedArtworkResponse> {
        let publication = self
            .selection_store
            .publish_selected_artwork(artifact_id)
            .await?;
        Ok(PublishSelectedArtworkResponse::from_publication(
            publication,
        ))
    }

    pub(crate) async fn select_item_artwork(
        &self,
        item_id: MediaItemId,
        kind: nako_core::ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<PublishSelectedArtworkResponse> {
        self.selection_store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let publication = self
            .selection_store
            .publish_selected_artwork_for_item_kind(item_id, kind, artifact_id)
            .await?;
        Ok(PublishSelectedArtworkResponse::from_publication(
            publication,
        ))
    }

    pub(crate) async fn unpublish_item_artwork(
        &self,
        item_id: MediaItemId,
        kind: nako_core::ImageKind,
    ) -> Result<UnpublishSelectedArtworkResponse> {
        self.selection_store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let unpublication = self
            .selection_store
            .unpublish_selected_artwork_for_item_kind(item_id, kind)
            .await?;
        Ok(UnpublishSelectedArtworkResponse::from_unpublication(
            unpublication,
        ))
    }

    pub(crate) async fn item_gallery(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<AdminManagedArtworkGalleryResponse> {
        self.selection_store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let snapshot = self
            .lifecycle_store
            .get_managed_artwork_gallery_for_item(item_id, page)
            .await?;
        let returned = snapshot.candidates.len().max(snapshot.artifacts.len());

        Ok(AdminManagedArtworkGalleryResponse::from_snapshot(
            snapshot,
            page_info_from_request(page, returned),
        ))
    }

    pub(crate) async fn artifact_lifecycle_diagnostics(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<AdminManagedArtworkArtifactLifecycleResponse> {
        let snapshot = self
            .lifecycle_store
            .list_managed_artwork_artifact_lifecycle(filter, page)
            .await?;
        let returned = snapshot.artifacts.len();

        Ok(AdminManagedArtworkArtifactLifecycleResponse::from_snapshot(
            snapshot,
            page_info_from_request(page, returned),
        ))
    }

    pub(crate) async fn cleanup_unselected_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<AdminManagedArtworkArtifactCleanupResponse> {
        let report = self
            .lifecycle_store
            .cleanup_unselected_managed_artwork_artifacts(page)
            .await?;
        let mut file_cleanup = AdminManagedArtworkArtifactFileCleanupSummary::default();
        for artifact in &report.cleaned_artifacts {
            match self
                .artifact_store
                .delete_artifact_best_effort(artifact)
                .await
            {
                ArtifactFileDeleteOutcome::Deleted => {
                    file_cleanup.file_deleted_artifacts =
                        file_cleanup.file_deleted_artifacts.saturating_add(1);
                }
                ArtifactFileDeleteOutcome::Missing => {
                    file_cleanup.file_missing_artifacts =
                        file_cleanup.file_missing_artifacts.saturating_add(1);
                }
                ArtifactFileDeleteOutcome::Failed => {
                    file_cleanup.file_delete_failed_artifacts =
                        file_cleanup.file_delete_failed_artifacts.saturating_add(1);
                }
            }
        }

        Ok(AdminManagedArtworkArtifactCleanupResponse::from_report(
            report,
            file_cleanup,
        ))
    }

    pub(crate) async fn artifact_storage_drift_diagnostics(
        &self,
        page: PageRequest,
        file_scan_limit: u32,
    ) -> Result<AdminManagedArtworkArtifactStorageDriftResponse> {
        let page = page.clamped();
        let snapshot = self
            .lifecycle_store
            .list_managed_artwork_artifact_lifecycle(
                ManagedArtworkArtifactLifecycleFilter::All,
                page,
            )
            .await?;
        let returned = snapshot.artifacts.len();
        let mut summary = AdminManagedArtworkArtifactStorageDriftSummary {
            scanned_db_artifacts: u32::try_from(returned).unwrap_or(u32::MAX),
            file_scan_limit,
            ..AdminManagedArtworkArtifactStorageDriftSummary::default()
        };
        let mut missing_artifacts = Vec::new();

        for record in snapshot.artifacts {
            match self.artifact_store.file_status(&record.artifact).await {
                ArtifactFileStatus::Present => {
                    summary.db_backed_present_artifacts =
                        summary.db_backed_present_artifacts.saturating_add(1);
                }
                ArtifactFileStatus::Missing => {
                    summary.db_backed_missing_artifacts =
                        summary.db_backed_missing_artifacts.saturating_add(1);
                    missing_artifacts.push(
                        AdminManagedArtworkArtifactStorageDriftArtifact::from_lifecycle_record(
                            record,
                            AdminManagedArtworkArtifactStorageDriftArtifactIssue::MissingFile,
                        ),
                    );
                }
                ArtifactFileStatus::UnresolvableExpectedPath => {
                    summary.db_backed_unresolvable_artifacts =
                        summary.db_backed_unresolvable_artifacts.saturating_add(1);
                    missing_artifacts.push(
                        AdminManagedArtworkArtifactStorageDriftArtifact::from_lifecycle_record(
                            record,
                            AdminManagedArtworkArtifactStorageDriftArtifactIssue::UnresolvableExpectedPath,
                        ),
                    );
                }
                ArtifactFileStatus::MetadataReadFailed => {
                    summary.db_backed_metadata_read_failed_artifacts = summary
                        .db_backed_metadata_read_failed_artifacts
                        .saturating_add(1);
                    missing_artifacts.push(
                        AdminManagedArtworkArtifactStorageDriftArtifact::from_lifecycle_record(
                            record,
                            AdminManagedArtworkArtifactStorageDriftArtifactIssue::MetadataReadFailed,
                        ),
                    );
                }
            }
        }

        let inventory = self.artifact_store.discover_files(file_scan_limit).await?;
        summary.scanned_files = inventory.scanned_files;
        summary.file_scan_truncated = inventory.truncated;

        let mut stray_files = Vec::new();
        for file in inventory.files {
            let Some(stray_file) = self.classify_stray_file(file).await? else {
                continue;
            };
            match stray_file.reason {
                AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile => {
                    summary.untracked_artifact_files =
                        summary.untracked_artifact_files.saturating_add(1);
                }
                AdminManagedArtworkArtifactStorageDriftFileReason::UnexpectedActiveArtifactPath => {
                    summary.unexpected_active_artifact_files =
                        summary.unexpected_active_artifact_files.saturating_add(1);
                }
                AdminManagedArtworkArtifactStorageDriftFileReason::UnsupportedExtension => {
                    summary.unsupported_extension_files =
                        summary.unsupported_extension_files.saturating_add(1);
                }
                AdminManagedArtworkArtifactStorageDriftFileReason::UnrecognizedLayout => {
                    summary.unrecognized_layout_files =
                        summary.unrecognized_layout_files.saturating_add(1);
                }
            }
            stray_files.push(stray_file);
        }
        summary.stray_files = u32::try_from(stray_files.len()).unwrap_or(u32::MAX);

        Ok(AdminManagedArtworkArtifactStorageDriftResponse::new(
            summary,
            missing_artifacts,
            stray_files,
            page_info_from_request(page, returned),
        ))
    }

    pub(crate) async fn artifact_remediation_plan(
        &self,
        page: PageRequest,
        file_scan_limit: u32,
    ) -> Result<AdminManagedArtworkArtifactRemediationPlanResponse> {
        let drift = self
            .artifact_storage_drift_diagnostics(page, file_scan_limit)
            .await?;
        let mut summary = AdminManagedArtworkArtifactRemediationSummary {
            scanned_db_artifacts: drift.summary.scanned_db_artifacts,
            missing_db_backed_artifacts: u32::try_from(drift.missing_artifacts.len())
                .unwrap_or(u32::MAX),
            file_scan_limit: drift.summary.file_scan_limit,
            scanned_files: drift.summary.scanned_files,
            file_scan_truncated: drift.summary.file_scan_truncated,
            ..AdminManagedArtworkArtifactRemediationSummary::default()
        };
        let mut missing_artifacts = Vec::new();
        for artifact in drift.missing_artifacts {
            if artifact.selected_artwork_count > 0 {
                summary.selected_missing_artifacts =
                    summary.selected_missing_artifacts.saturating_add(1);
            }
            if artifact.cleanup_candidate {
                summary.cleanup_candidate_missing_artifacts = summary
                    .cleanup_candidate_missing_artifacts
                    .saturating_add(1);
            }
            missing_artifacts.push(
                AdminManagedArtworkArtifactRemediationMissingArtifact::from_storage_drift(artifact),
            );
        }

        let mut stray_files = Vec::new();
        for file in drift.stray_files {
            let stray = AdminManagedArtworkArtifactRemediationStrayFile::from_storage_drift(file);
            match stray.action {
                AdminManagedArtworkArtifactStrayFileRemediationAction::DeleteStrayFile => {
                    summary.cleanable_stray_files = summary.cleanable_stray_files.saturating_add(1);
                }
                AdminManagedArtworkArtifactStrayFileRemediationAction::InspectManually => {
                    summary.blocked_stray_files = summary.blocked_stray_files.saturating_add(1);
                }
            }
            stray_files.push(stray);
        }

        Ok(AdminManagedArtworkArtifactRemediationPlanResponse::new(
            summary,
            missing_artifacts,
            stray_files,
            drift.page,
        ))
    }

    pub(crate) async fn cleanup_untracked_artifact_files(
        &self,
        file_scan_limit: u32,
    ) -> Result<AdminManagedArtworkArtifactStrayFileCleanupResponse> {
        let inventory = self.artifact_store.discover_files(file_scan_limit).await?;
        let mut summary = AdminManagedArtworkArtifactStrayFileCleanupSummary {
            file_scan_limit,
            scanned_files: inventory.scanned_files,
            file_scan_truncated: inventory.truncated,
            ..AdminManagedArtworkArtifactStrayFileCleanupSummary::default()
        };
        let mut cleaned_files = Vec::new();
        let mut blocked_files = Vec::new();

        for file in inventory.files {
            let Some(classified) = self.classify_stray_file_record(file).await? else {
                continue;
            };
            let remediation_file =
                AdminManagedArtworkArtifactRemediationStrayFile::from_storage_drift(
                    classified_artifact_store_file_to_drift_file_ref(&classified),
                );
            if remediation_file.action
                != AdminManagedArtworkArtifactStrayFileRemediationAction::DeleteStrayFile
            {
                summary.blocked_stray_files = summary.blocked_stray_files.saturating_add(1);
                blocked_files.push(remediation_file);
                continue;
            }

            let Some(artifact_id) = classified.recognized_artifact_id else {
                summary.blocked_stray_files = summary.blocked_stray_files.saturating_add(1);
                blocked_files.push(remediation_file);
                continue;
            };
            if self
                .lifecycle_store
                .get_managed_artwork_artifact(artifact_id)
                .await?
                .is_some()
            {
                summary.blocked_stray_files = summary.blocked_stray_files.saturating_add(1);
                blocked_files.push(AdminManagedArtworkArtifactRemediationStrayFile {
                    action: AdminManagedArtworkArtifactStrayFileRemediationAction::InspectManually,
                    ..remediation_file
                });
                continue;
            }

            summary.cleanable_stray_files = summary.cleanable_stray_files.saturating_add(1);
            let status = match self
                .artifact_store
                .delete_discovered_file_best_effort(&classified.path)
                .await
            {
                ArtifactFileDeleteOutcome::Deleted => {
                    summary.deleted_files = summary.deleted_files.saturating_add(1);
                    AdminManagedArtworkArtifactStrayFileCleanupStatus::Deleted
                }
                ArtifactFileDeleteOutcome::Missing => {
                    summary.missing_files = summary.missing_files.saturating_add(1);
                    AdminManagedArtworkArtifactStrayFileCleanupStatus::Missing
                }
                ArtifactFileDeleteOutcome::Failed => {
                    summary.failed_files = summary.failed_files.saturating_add(1);
                    AdminManagedArtworkArtifactStrayFileCleanupStatus::Failed
                }
            };
            cleaned_files.push(AdminManagedArtworkArtifactStrayFileCleanupItem {
                recognized_artifact_id: artifact_id,
                extension: classified.extension,
                byte_len: classified.byte_len,
                status,
            });
        }

        Ok(AdminManagedArtworkArtifactStrayFileCleanupResponse::new(
            summary,
            cleaned_files,
            blocked_files,
        ))
    }

    async fn classify_stray_file(
        &self,
        file: DiscoveredArtifactFile,
    ) -> Result<Option<AdminManagedArtworkArtifactStorageDriftFile>> {
        Ok(self
            .classify_stray_file_record(file)
            .await?
            .map(classified_artifact_store_file_to_drift_file))
    }

    async fn classify_stray_file_record(
        &self,
        file: DiscoveredArtifactFile,
    ) -> Result<Option<ClassifiedArtifactStoreFile>> {
        let issue = match &file.layout {
            DiscoveredArtifactFileLayout::Recognized {
                artifact_id,
                supported_extension: true,
                shard_matches: true,
                ..
            } => {
                let Some(artifact) = self
                    .lifecycle_store
                    .get_managed_artwork_artifact(*artifact_id)
                    .await?
                else {
                    return Ok(Some(
                        file.into_classified(ArtifactStoreFileIssue::UntrackedArtifactFile),
                    ));
                };
                match self.artifact_store.path_for_artifact(&artifact) {
                    Ok(path) if path == file.path => return Ok(None),
                    _ => ArtifactStoreFileIssue::UnexpectedActiveArtifactPath,
                }
            }
            DiscoveredArtifactFileLayout::Recognized {
                supported_extension: false,
                ..
            } => ArtifactStoreFileIssue::UnsupportedExtension,
            DiscoveredArtifactFileLayout::Recognized {
                shard_matches: false,
                ..
            } => ArtifactStoreFileIssue::UnrecognizedLayout,
            DiscoveredArtifactFileLayout::Unrecognized => {
                ArtifactStoreFileIssue::UnrecognizedLayout
            }
        };

        Ok(Some(file.into_classified(issue)))
    }

    pub(crate) async fn read_selected_image(
        &self,
        selected_id: SelectedArtworkId,
        variant: ImageVariantRequest,
    ) -> Result<ManagedArtworkImageBytes> {
        let variant = self.variant_policy.validate(variant)?;

        let selected = self
            .lifecycle_store
            .get_selected_artwork(selected_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "selected_artwork",
                id: selected_id.to_string(),
            })?;
        let artifact = self
            .lifecycle_store
            .get_managed_artwork_artifact(selected.artifact_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: "selected artwork references missing managed artwork artifact".to_owned(),
            })?;

        if selected.library_id != artifact.library_id
            || selected.item_id != artifact.item_id
            || selected.kind != artifact.kind
        {
            return Err(NakoError::Database {
                message: "selected artwork and managed artwork artifact are inconsistent"
                    .to_owned(),
            });
        }

        let variant = variant.for_artifact(&artifact)?;
        let bytes = self.artifact_store.read(selected_id, &artifact).await?;
        variant.derive(selected.id, &artifact, bytes)
    }

    async fn process_claim(
        &self,
        claim: ManagedArtworkIngestClaimRecord,
    ) -> std::result::Result<nako_core::ManagedArtworkIngestProcessingRecord, ManagedArtworkFailure>
    {
        let prepared = self
            .ingest_pipeline
            .prepare_artifact(&claim, &self.artifact_store)
            .await?;
        let result = self
            .ingest_store
            .commit_managed_artwork_artifact(
                claim.ingest.id,
                prepared.artifact,
                Some(prepared.summary_json),
            )
            .await;

        match result {
            Ok(processing) => Ok(processing),
            Err(_) => {
                self.artifact_store
                    .delete_best_effort(&prepared.stored)
                    .await;
                Err(ManagedArtworkFailure::storage_failed())
            }
        }
    }

    async fn process_next_unit(
        &self,
    ) -> Result<Option<nako_core::ManagedArtworkIngestProcessingRecord>> {
        let Some(claim) = self
            .ingest_store
            .claim_next_queued_managed_artwork_ingest()
            .await?
        else {
            return Ok(None);
        };

        self.process_claim_with_failure_commit(claim)
            .await
            .map(Some)
    }

    async fn process_claim_with_failure_commit(
        &self,
        claim: ManagedArtworkIngestClaimRecord,
    ) -> Result<nako_core::ManagedArtworkIngestProcessingRecord> {
        match self.process_claim(claim.clone()).await {
            Ok(processing) => Ok(processing),
            Err(failure) => {
                let (failure_code, summary_json) =
                    ManagedArtworkIngestPipeline::failure_summary_json(failure, &claim);
                self.ingest_store
                    .fail_managed_artwork_ingest(
                        claim.ingest.id,
                        failure_code.clone(),
                        failure_code,
                        summary_json,
                    )
                    .await
            }
        }
    }
}

#[derive(Serialize)]
struct ManagedArtworkIngestJobInput {
    candidate_id: ArtworkCandidateId,
    library_id: nako_core::LibraryId,
    item_id: nako_core::MediaItemId,
    image_kind: String,
}

fn classified_artifact_store_file_to_drift_file(
    file: ClassifiedArtifactStoreFile,
) -> AdminManagedArtworkArtifactStorageDriftFile {
    AdminManagedArtworkArtifactStorageDriftFile {
        reason: artifact_store_file_issue_to_drift_reason(file.issue),
        recognized_artifact_id: file.recognized_artifact_id,
        extension: file.extension,
        byte_len: file.byte_len,
    }
}

fn classified_artifact_store_file_to_drift_file_ref(
    file: &ClassifiedArtifactStoreFile,
) -> AdminManagedArtworkArtifactStorageDriftFile {
    AdminManagedArtworkArtifactStorageDriftFile {
        reason: artifact_store_file_issue_to_drift_reason(file.issue),
        recognized_artifact_id: file.recognized_artifact_id,
        extension: file.extension.clone(),
        byte_len: file.byte_len,
    }
}

const fn artifact_store_file_issue_to_drift_reason(
    issue: ArtifactStoreFileIssue,
) -> AdminManagedArtworkArtifactStorageDriftFileReason {
    match issue {
        ArtifactStoreFileIssue::UntrackedArtifactFile => {
            AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile
        }
        ArtifactStoreFileIssue::UnexpectedActiveArtifactPath => {
            AdminManagedArtworkArtifactStorageDriftFileReason::UnexpectedActiveArtifactPath
        }
        ArtifactStoreFileIssue::UnsupportedExtension => {
            AdminManagedArtworkArtifactStorageDriftFileReason::UnsupportedExtension
        }
        ArtifactStoreFileIssue::UnrecognizedLayout => {
            AdminManagedArtworkArtifactStorageDriftFileReason::UnrecognizedLayout
        }
    }
}

fn image_format_for_media_type(media_type: &str) -> Option<(image::ImageFormat, &'static str)> {
    match media_type {
        "image/jpeg" => Some((image::ImageFormat::Jpeg, "jpg")),
        "image/png" => Some((image::ImageFormat::Png, "png")),
        "image/webp" => Some((image::ImageFormat::WebP, "webp")),
        _ => None,
    }
}

fn image_kind_label(kind: &nako_core::ImageKind) -> &'static str {
    match kind {
        nako_core::ImageKind::Poster => "poster",
        nako_core::ImageKind::Backdrop => "backdrop",
        nako_core::ImageKind::Logo => "logo",
        nako_core::ImageKind::Thumbnail => "thumbnail",
        nako_core::ImageKind::Banner => "banner",
        nako_core::ImageKind::Other(_) => "other",
    }
}
