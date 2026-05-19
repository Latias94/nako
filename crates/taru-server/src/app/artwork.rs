use std::{
    fmt::Write as _,
    io::{Cursor, ErrorKind},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures_util::StreamExt;
use image::{GenericImageView, imageops::FilterType};
use serde::Serialize;
use sha2::{Digest, Sha256};
use taru_api::{
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
    UnpublishSelectedArtworkResponse, page_info_from_request,
};
use taru_core::{
    ArtworkCandidateId, ArtworkCandidateRepository, ArtworkCandidateSourceKind,
    ArtworkCandidateStatus, JobId, JobKind, LibraryItemRepository, ManagedArtworkArtifactId,
    ManagedArtworkArtifactLifecycleFilter, ManagedArtworkArtifactRecord,
    ManagedArtworkIngestClaimRecord, ManagedArtworkIngestId, ManagedArtworkIngestStatus,
    ManagedArtworkRepository, MediaItemId, MediaRepository, NewJob, NewManagedArtworkArtifact,
    NewManagedArtworkIngest, PageRequest, Result, SelectedArtworkId, StorageErrorKind, TaruError,
};
use taru_db::SqliteStore;
use tokio::{fs, io::AsyncWriteExt, sync::Semaphore};

use crate::config::ArtworkConfig;

#[derive(Clone, Debug)]
pub(crate) struct ManagedArtworkAppService {
    store: SqliteStore,
    fetcher: ManagedArtworkFetcher,
    validator: ManagedArtworkImageValidator,
    artifact_store: LocalManagedArtworkArtifactStore,
    variant_policy: ImageVariantPolicy,
}

impl ManagedArtworkAppService {
    pub(crate) fn new(config: ArtworkConfig, store: SqliteStore) -> Result<Self> {
        Ok(Self {
            store,
            fetcher: ManagedArtworkFetcher::new(config.clone())?,
            validator: ManagedArtworkImageValidator::new(config.clone()),
            variant_policy: ImageVariantPolicy::new(config.max_width, config.max_height),
            artifact_store: LocalManagedArtworkArtifactStore::new(config.artifact_root),
        })
    }

    pub(crate) async fn accept_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<AcceptManagedArtworkCandidateResponse> {
        let candidate = self
            .store
            .get_artwork_candidate(candidate_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: candidate_id.to_string(),
            })?;

        if candidate.status == ArtworkCandidateStatus::Rejected {
            return Err(TaruError::InvalidInput {
                message: "rejected artwork candidates cannot be accepted".to_owned(),
            });
        }

        self.store
            .get_media_item(candidate.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: candidate.item_id.to_string(),
            })?;
        self.store
            .get_library_item_state(candidate.library_id, candidate.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
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
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize managed artwork ingest job input: {err}"),
        })?;
        let acceptance = self
            .store
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
        let Some(claim) = self
            .store
            .claim_next_queued_managed_artwork_ingest()
            .await?
        else {
            return Ok(ProcessManagedArtworkIngestResponse::empty());
        };

        match self.process_claim(claim.clone()).await {
            Ok(processing) => Ok(ProcessManagedArtworkIngestResponse::from_processing(
                processing,
            )),
            Err(failure) => {
                let summary = ManagedArtworkIngestFailureSummary {
                    ingest_id: claim.ingest.id,
                    candidate_id: claim.candidate.id,
                    status: ManagedArtworkIngestStatus::Failed.as_str(),
                    failure_code: failure.code.as_str(),
                };
                let summary_json = serde_json::to_string(&summary).ok();
                let failure_code = failure.code.as_str().to_owned();
                let processing = self
                    .store
                    .fail_managed_artwork_ingest(
                        claim.ingest.id,
                        failure_code.clone(),
                        failure_code,
                        summary_json,
                    )
                    .await?;
                Ok(ProcessManagedArtworkIngestResponse::from_processing(
                    processing,
                ))
            }
        }
    }

    pub(crate) async fn publish_artifact(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<PublishSelectedArtworkResponse> {
        let publication = self.store.publish_selected_artwork(artifact_id).await?;
        Ok(PublishSelectedArtworkResponse::from_publication(
            publication,
        ))
    }

    pub(crate) async fn select_item_artwork(
        &self,
        item_id: MediaItemId,
        kind: taru_core::ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<PublishSelectedArtworkResponse> {
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let publication = self
            .store
            .publish_selected_artwork_for_item_kind(item_id, kind, artifact_id)
            .await?;
        Ok(PublishSelectedArtworkResponse::from_publication(
            publication,
        ))
    }

    pub(crate) async fn unpublish_item_artwork(
        &self,
        item_id: MediaItemId,
        kind: taru_core::ImageKind,
    ) -> Result<UnpublishSelectedArtworkResponse> {
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let unpublication = self
            .store
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
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let snapshot = self
            .store
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
            .store
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
            .store
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
            .store
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
                    classified.to_drift_file(),
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
                .store
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
            .map(ClassifiedArtifactStoreFile::into_drift_file))
    }

    async fn classify_stray_file_record(
        &self,
        file: DiscoveredArtifactFile,
    ) -> Result<Option<ClassifiedArtifactStoreFile>> {
        let reason = match &file.layout {
            DiscoveredArtifactFileLayout::Recognized {
                artifact_id,
                supported_extension: true,
                shard_matches: true,
                ..
            } => {
                let Some(artifact) = self
                    .store
                    .get_managed_artwork_artifact(*artifact_id)
                    .await?
                else {
                    return Ok(Some(file.into_classified(
                        AdminManagedArtworkArtifactStorageDriftFileReason::UntrackedArtifactFile,
                    )));
                };
                match self.artifact_store.path_for_artifact(&artifact) {
                    Ok(path) if path == file.path => return Ok(None),
                    _ => {
                        AdminManagedArtworkArtifactStorageDriftFileReason::UnexpectedActiveArtifactPath
                    }
                }
            }
            DiscoveredArtifactFileLayout::Recognized {
                supported_extension: false,
                ..
            } => AdminManagedArtworkArtifactStorageDriftFileReason::UnsupportedExtension,
            DiscoveredArtifactFileLayout::Recognized {
                shard_matches: false,
                ..
            } => AdminManagedArtworkArtifactStorageDriftFileReason::UnrecognizedLayout,
            DiscoveredArtifactFileLayout::Unrecognized => {
                AdminManagedArtworkArtifactStorageDriftFileReason::UnrecognizedLayout
            }
        };

        Ok(Some(file.into_classified(reason)))
    }

    pub(crate) async fn read_selected_image(
        &self,
        selected_id: SelectedArtworkId,
        variant: ImageVariantRequest,
    ) -> Result<ManagedArtworkImageBytes> {
        self.variant_policy.validate(variant)?;

        let selected = self
            .store
            .get_selected_artwork(selected_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "selected_artwork",
                id: selected_id.to_string(),
            })?;
        let artifact = self
            .store
            .get_managed_artwork_artifact(selected.artifact_id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: "selected artwork references missing managed artwork artifact".to_owned(),
            })?;

        if selected.library_id != artifact.library_id
            || selected.item_id != artifact.item_id
            || selected.kind != artifact.kind
        {
            return Err(TaruError::Database {
                message: "selected artwork and managed artwork artifact are inconsistent"
                    .to_owned(),
            });
        }

        let original_media_type = artifact
            .media_type
            .clone()
            .ok_or_else(|| managed_artwork_variant_storage_error("media type is missing"))?;
        let bytes = self.artifact_store.read(selected_id, &artifact).await?;
        let image = if variant.is_original() {
            ManagedArtworkImageBytes {
                bytes,
                media_type: original_media_type,
                content_length: 0,
                etag: Some(public_selected_image_etag(
                    selected.id,
                    &artifact,
                    ImageVariantKey::Original,
                )),
            }
            .with_content_length()?
        } else {
            derive_selected_image_variant(
                selected.id,
                &artifact,
                &original_media_type,
                bytes,
                variant,
            )?
        };

        Ok(image)
    }

    async fn process_claim(
        &self,
        claim: ManagedArtworkIngestClaimRecord,
    ) -> std::result::Result<taru_core::ManagedArtworkIngestProcessingRecord, ManagedArtworkFailure>
    {
        if claim.candidate.source_kind != ArtworkCandidateSourceKind::RemoteUrl {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::UnsupportedSource,
            ));
        }

        let fetched = self.fetcher.fetch(&claim.candidate.source_uri).await?;
        let validated = self.validator.validate(&fetched)?;
        let artifact_id = ManagedArtworkArtifactId::new();
        let stored = self
            .artifact_store
            .write(artifact_id, validated.extension, &fetched.bytes)
            .await?;

        let summary = ManagedArtworkIngestJobSummary {
            ingest_id: claim.ingest.id,
            candidate_id: claim.candidate.id,
            artifact_id,
            status: ManagedArtworkIngestStatus::Stored.as_str(),
            media_type: validated.media_type.clone(),
            byte_len: validated.byte_len,
            width: validated.width,
            height: validated.height,
            content_hash: validated.content_hash.clone(),
        };
        let summary_json = serde_json::to_string(&summary)
            .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::StorageFailed))?;
        let result = self
            .store
            .commit_managed_artwork_artifact(
                claim.ingest.id,
                NewManagedArtworkArtifact {
                    id: artifact_id,
                    ingest_id: claim.ingest.id,
                    library_id: claim.ingest.library_id,
                    item_id: claim.ingest.item_id,
                    kind: claim.ingest.kind,
                    storage_uri: stored.storage_uri.clone(),
                    content_hash: Some(validated.content_hash),
                    width: Some(validated.width),
                    height: Some(validated.height),
                    byte_len: Some(validated.byte_len),
                    media_type: Some(validated.media_type),
                },
                Some(summary_json),
            )
            .await;

        match result {
            Ok(processing) => Ok(processing),
            Err(_) => {
                self.artifact_store.delete_best_effort(&stored).await;
                Err(ManagedArtworkFailure::new(
                    ManagedArtworkFailureCode::StorageFailed,
                ))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageVariantRequest {
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

impl ImageVariantRequest {
    #[must_use]
    pub(crate) const fn original() -> Self {
        Self {
            width: None,
            height: None,
        }
    }

    pub(crate) fn bounded(width: Option<u32>, height: Option<u32>) -> Result<Self> {
        validate_variant_edge("width", width)?;
        validate_variant_edge("height", height)?;
        Ok(Self { width, height })
    }

    #[must_use]
    const fn is_original(self) -> bool {
        self.width.is_none() && self.height.is_none()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImageVariantKey {
    Original,
    Bounded { width: u32, height: u32 },
}

fn validate_variant_edge(name: &str, value: Option<u32>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value == 0 {
        return Err(TaruError::InvalidInput {
            message: format!("{name} must be greater than zero"),
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ImageVariantPolicy {
    max_width: u32,
    max_height: u32,
}

impl ImageVariantPolicy {
    const fn new(max_width: u32, max_height: u32) -> Self {
        Self {
            max_width,
            max_height,
        }
    }

    fn validate(self, variant: ImageVariantRequest) -> Result<()> {
        validate_variant_edge_against_limit("width", variant.width, self.max_width)?;
        validate_variant_edge_against_limit("height", variant.height, self.max_height)?;
        Ok(())
    }
}

fn validate_variant_edge_against_limit(name: &str, value: Option<u32>, limit: u32) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value > limit {
        return Err(TaruError::InvalidInput {
            message: format!("{name} must be less than or equal to {limit}"),
        });
    }

    Ok(())
}

fn derive_selected_image_variant(
    selected_id: SelectedArtworkId,
    artifact: &ManagedArtworkArtifactRecord,
    original_media_type: &str,
    bytes: Vec<u8>,
    variant: ImageVariantRequest,
) -> Result<ManagedArtworkImageBytes> {
    let (format, _extension) = image_format_for_media_type(original_media_type)
        .ok_or_else(|| managed_artwork_variant_storage_error("media type is unsupported"))?;
    let image =
        image::load_from_memory_with_format(&bytes, format).map_err(|_err| TaruError::Storage {
            uri: "managed-artwork://artifact".to_owned(),
            kind: StorageErrorKind::Unknown,
            message: "managed artwork artifact image is invalid".to_owned(),
        })?;
    let (original_width, original_height) = image.dimensions();
    let (width, height) = variant_dimensions(original_width, original_height, variant)?;
    if width == original_width && height == original_height {
        return ManagedArtworkImageBytes {
            bytes,
            media_type: original_media_type.to_owned(),
            content_length: 0,
            etag: Some(public_selected_image_etag(
                selected_id,
                artifact,
                ImageVariantKey::Original,
            )),
        }
        .with_content_length();
    }

    let resized = image.resize(width, height, FilterType::Lanczos3);
    let mut output = Cursor::new(Vec::new());
    resized
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|_err| TaruError::Storage {
            uri: "managed-artwork://artifact".to_owned(),
            kind: StorageErrorKind::Unknown,
            message: "failed to encode managed artwork image variant".to_owned(),
        })?;

    ManagedArtworkImageBytes {
        bytes: output.into_inner(),
        media_type: "image/png".to_owned(),
        content_length: 0,
        etag: Some(public_selected_image_etag(
            selected_id,
            artifact,
            ImageVariantKey::Bounded { width, height },
        )),
    }
    .with_content_length()
}

fn variant_dimensions(
    original_width: u32,
    original_height: u32,
    variant: ImageVariantRequest,
) -> Result<(u32, u32)> {
    if original_width == 0 || original_height == 0 {
        return Err(managed_artwork_variant_storage_error(
            "image dimensions are invalid",
        ));
    }

    let target_width = variant.width.unwrap_or(original_width);
    let target_height = variant.height.unwrap_or(original_height);
    let width_ratio = target_width as f64 / original_width as f64;
    let height_ratio = target_height as f64 / original_height as f64;
    let scale = width_ratio.min(height_ratio).min(1.0);
    let width = ((original_width as f64) * scale).round().max(1.0) as u32;
    let height = ((original_height as f64) * scale).round().max(1.0) as u32;

    Ok((width, height))
}

fn public_selected_image_etag(
    selected_id: SelectedArtworkId,
    artifact: &ManagedArtworkArtifactRecord,
    variant: ImageVariantKey,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"taru-public-image-etag-v1");
    hasher.update(selected_id.to_string().as_bytes());
    hasher.update(artifact.id.to_string().as_bytes());
    hasher.update(artifact.updated_at.as_bytes());
    match variant {
        ImageVariantKey::Original => hasher.update(b"original"),
        ImageVariantKey::Bounded { width, height } => {
            hasher.update(b"bounded");
            hasher.update(width.to_be_bytes());
            hasher.update(height.to_be_bytes());
        }
    }
    let digest = hasher.finalize();
    let mut output = String::from("taru-img-v1-");
    for byte in digest.iter().take(16) {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn managed_artwork_variant_storage_error(message: &str) -> TaruError {
    TaruError::Storage {
        uri: "managed-artwork://artifact".to_owned(),
        kind: StorageErrorKind::Unknown,
        message: format!("managed artwork artifact {message}"),
    }
}

impl ManagedArtworkImageBytes {
    fn with_content_length(mut self) -> Result<Self> {
        self.content_length =
            u64::try_from(self.bytes.len()).map_err(|err| TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: format!("managed artwork image length is too large: {err}"),
            })?;

        Ok(self)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedArtworkImageBytes {
    pub(crate) bytes: Vec<u8>,
    pub(crate) media_type: String,
    pub(crate) content_length: u64,
    pub(crate) etag: Option<String>,
}

#[derive(Serialize)]
struct ManagedArtworkIngestJobInput {
    candidate_id: ArtworkCandidateId,
    library_id: taru_core::LibraryId,
    item_id: taru_core::MediaItemId,
    image_kind: String,
}

#[derive(Serialize)]
struct ManagedArtworkIngestJobSummary {
    ingest_id: ManagedArtworkIngestId,
    candidate_id: ArtworkCandidateId,
    artifact_id: ManagedArtworkArtifactId,
    status: &'static str,
    media_type: String,
    byte_len: u64,
    width: u32,
    height: u32,
    content_hash: String,
}

#[derive(Serialize)]
struct ManagedArtworkIngestFailureSummary {
    ingest_id: ManagedArtworkIngestId,
    candidate_id: ArtworkCandidateId,
    status: &'static str,
    failure_code: &'static str,
}

#[derive(Clone, Debug)]
struct ManagedArtworkFetcher {
    client: reqwest::Client,
    config: ArtworkConfig,
    permits: Arc<Semaphore>,
}

#[derive(Clone, Debug)]
struct FetchedManagedArtwork {
    bytes: Vec<u8>,
    media_type: String,
}

impl ManagedArtworkFetcher {
    fn new(config: ArtworkConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .user_agent(config.fetch_user_agent.clone())
            .timeout(Duration::from_millis(config.fetch_timeout_ms));

        if let Some(proxy) = config
            .fetch_proxy
            .as_ref()
            .filter(|proxy| !proxy.is_blank())
        {
            builder = builder.proxy(reqwest::Proxy::all(proxy.expose_secret()).map_err(|err| {
                TaruError::InvalidInput {
                    message: format!("invalid artwork fetch proxy configuration: {err}"),
                }
            })?);
        }

        let client = builder.build().map_err(|err| TaruError::InvalidInput {
            message: format!("failed to build artwork fetch HTTP client: {err}"),
        })?;
        let permits = Arc::new(Semaphore::new(config.fetch_concurrency.max(1)));

        Ok(Self {
            client,
            config,
            permits,
        })
    }

    async fn fetch(
        &self,
        source_uri: &str,
    ) -> std::result::Result<FetchedManagedArtwork, ManagedArtworkFailure> {
        let url = reqwest::Url::parse(source_uri).map_err(|_| {
            ManagedArtworkFailure::new(ManagedArtworkFailureCode::UnsupportedSource)
        })?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::UnsupportedSource,
            ));
        }

        let _permit = self.permits.acquire().await.map_err(|_| {
            ManagedArtworkFailure::new(ManagedArtworkFailureCode::ResourceBudgetClosed)
        })?;
        let mut last_failure = ManagedArtworkFailure::new(ManagedArtworkFailureCode::FetchFailed);
        let attempts = self.config.fetch_max_attempts.max(1);

        for _ in 0..attempts {
            match self.fetch_once(url.clone()).await {
                Ok(fetched) => return Ok(fetched),
                Err(failure) if failure.retryable => last_failure = failure,
                Err(failure) => return Err(failure),
            }
        }

        Err(last_failure)
    }

    async fn fetch_once(
        &self,
        url: reqwest::Url,
    ) -> std::result::Result<FetchedManagedArtwork, ManagedArtworkFailure> {
        let response = self.client.get(url).send().await.map_err(|err| {
            if err.is_timeout() {
                ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchTimeout)
            } else {
                ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchFailed)
            }
        })?;

        if !response.status().is_success() {
            return Err(ManagedArtworkFailure::retryable(
                ManagedArtworkFailureCode::FetchHttpStatus,
            ));
        }

        if response
            .content_length()
            .is_some_and(|len| len > self.config.fetch_max_bytes)
        {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::TooLarge,
            ));
        }

        let media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(normalize_media_type)
            .ok_or_else(|| {
                ManagedArtworkFailure::new(ManagedArtworkFailureCode::UnsupportedMediaType)
            })?;

        let mut bytes = Vec::new();
        let mut total_len = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|err| {
                if err.is_timeout() {
                    ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchTimeout)
                } else {
                    ManagedArtworkFailure::retryable(ManagedArtworkFailureCode::FetchFailed)
                }
            })?;
            let chunk_len = u64::try_from(chunk.len())
                .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::TooLarge))?;
            total_len = total_len
                .checked_add(chunk_len)
                .ok_or_else(|| ManagedArtworkFailure::new(ManagedArtworkFailureCode::TooLarge))?;
            if total_len > self.config.fetch_max_bytes {
                return Err(ManagedArtworkFailure::new(
                    ManagedArtworkFailureCode::TooLarge,
                ));
            }
            bytes.extend_from_slice(&chunk);
        }

        Ok(FetchedManagedArtwork { bytes, media_type })
    }
}

#[derive(Clone, Debug)]
struct ManagedArtworkImageValidator {
    max_width: u32,
    max_height: u32,
}

#[derive(Clone, Debug)]
struct ValidatedManagedArtwork {
    media_type: String,
    extension: &'static str,
    width: u32,
    height: u32,
    byte_len: u64,
    content_hash: String,
}

impl ManagedArtworkImageValidator {
    fn new(config: ArtworkConfig) -> Self {
        Self {
            max_width: config.max_width,
            max_height: config.max_height,
        }
    }

    fn validate(
        &self,
        fetched: &FetchedManagedArtwork,
    ) -> std::result::Result<ValidatedManagedArtwork, ManagedArtworkFailure> {
        let (format, extension) =
            image_format_for_media_type(&fetched.media_type).ok_or_else(|| {
                ManagedArtworkFailure::new(ManagedArtworkFailureCode::UnsupportedMediaType)
            })?;
        let image = image::load_from_memory_with_format(&fetched.bytes, format)
            .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::InvalidImage))?;
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::InvalidImage,
            ));
        }
        if width > self.max_width || height > self.max_height {
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::DimensionLimitExceeded,
            ));
        }

        Ok(ValidatedManagedArtwork {
            media_type: fetched.media_type.clone(),
            extension,
            width,
            height,
            byte_len: u64::try_from(fetched.bytes.len())
                .map_err(|_| ManagedArtworkFailure::new(ManagedArtworkFailureCode::TooLarge))?,
            content_hash: sha256_hex(&fetched.bytes),
        })
    }
}

#[derive(Clone, Debug)]
struct LocalManagedArtworkArtifactStore {
    root: PathBuf,
}

#[derive(Clone, Debug)]
struct StoredManagedArtworkArtifact {
    storage_uri: String,
    path: PathBuf,
}

impl LocalManagedArtworkArtifactStore {
    fn new(root: PathBuf) -> Self {
        Self { root }
    }

    async fn write(
        &self,
        artifact_id: ManagedArtworkArtifactId,
        extension: &str,
        bytes: &[u8],
    ) -> std::result::Result<StoredManagedArtworkArtifact, ManagedArtworkFailure> {
        let artifact_id_text = artifact_id.to_string();
        let shard = artifact_id_text
            .get(0..2)
            .ok_or_else(|| ManagedArtworkFailure::new(ManagedArtworkFailureCode::StorageFailed))?;
        let directory = self.root.join(shard);
        let final_path = directory.join(format!("{artifact_id_text}.{extension}"));
        let temp_path = directory.join(format!("{artifact_id_text}.tmp"));

        let result = async {
            fs::create_dir_all(&directory).await?;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp_path)
                .await?;
            file.write_all(bytes).await?;
            file.sync_all().await?;
            drop(file);
            fs::rename(&temp_path, &final_path).await
        }
        .await;

        if result.is_err() {
            let _ = fs::remove_file(&temp_path).await;
            return Err(ManagedArtworkFailure::new(
                ManagedArtworkFailureCode::StorageFailed,
            ));
        }

        Ok(StoredManagedArtworkArtifact {
            storage_uri: format!("managed-artwork://artifact/{artifact_id_text}"),
            path: final_path,
        })
    }

    async fn delete_best_effort(&self, stored: &StoredManagedArtworkArtifact) {
        if path_has_prefix(&stored.path, &self.root) {
            let _ = fs::remove_file(&stored.path).await;
        }
    }

    async fn delete_artifact_best_effort(
        &self,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> ArtifactFileDeleteOutcome {
        match self.path_for_artifact(artifact) {
            Ok(path) if path_has_prefix(&path, &self.root) => match fs::remove_file(&path).await {
                Ok(()) => ArtifactFileDeleteOutcome::Deleted,
                Err(err) if err.kind() == ErrorKind::NotFound => ArtifactFileDeleteOutcome::Missing,
                Err(_) => ArtifactFileDeleteOutcome::Failed,
            },
            _ => ArtifactFileDeleteOutcome::Failed,
        }
    }

    async fn delete_discovered_file_best_effort(&self, path: &Path) -> ArtifactFileDeleteOutcome {
        if !path_has_prefix(path, &self.root) {
            return ArtifactFileDeleteOutcome::Failed;
        }
        match fs::remove_file(path).await {
            Ok(()) => ArtifactFileDeleteOutcome::Deleted,
            Err(err) if err.kind() == ErrorKind::NotFound => ArtifactFileDeleteOutcome::Missing,
            Err(_) => ArtifactFileDeleteOutcome::Failed,
        }
    }

    async fn file_status(&self, artifact: &ManagedArtworkArtifactRecord) -> ArtifactFileStatus {
        let Ok(path) = self.path_for_artifact(artifact) else {
            return ArtifactFileStatus::UnresolvableExpectedPath;
        };

        match fs::metadata(&path).await {
            Ok(metadata) if metadata.is_file() => ArtifactFileStatus::Present,
            Ok(_) => ArtifactFileStatus::Missing,
            Err(err) if err.kind() == ErrorKind::NotFound => ArtifactFileStatus::Missing,
            Err(_) => ArtifactFileStatus::MetadataReadFailed,
        }
    }

    async fn discover_files(&self, max_files: u32) -> Result<ArtifactStoreFileInventory> {
        let mut inventory = ArtifactStoreFileInventory::default();
        let mut directories = vec![self.root.clone()];

        while let Some(directory) = directories.pop() {
            let mut entries = match fs::read_dir(&directory).await {
                Ok(entries) => entries,
                Err(err) if err.kind() == ErrorKind::NotFound => continue,
                Err(_) => return Err(managed_artwork_artifact_store_inventory_error()),
            };

            loop {
                let entry = entries
                    .next_entry()
                    .await
                    .map_err(|_| managed_artwork_artifact_store_inventory_error())?;
                let Some(entry) = entry else {
                    break;
                };
                let path = entry.path();
                if !path_has_prefix(&path, &self.root) {
                    continue;
                }
                let file_type = entry
                    .file_type()
                    .await
                    .map_err(|_| managed_artwork_artifact_store_inventory_error())?;
                if file_type.is_dir() {
                    directories.push(path);
                    continue;
                }

                if inventory.scanned_files >= max_files {
                    inventory.truncated = true;
                    return Ok(inventory);
                }

                let byte_len = if file_type.is_file() {
                    entry.metadata().await.ok().map(|metadata| metadata.len())
                } else {
                    None
                };
                inventory.scanned_files = inventory.scanned_files.saturating_add(1);
                inventory
                    .files
                    .push(self.describe_discovered_file(path, byte_len));
            }
        }

        Ok(inventory)
    }

    fn describe_discovered_file(
        &self,
        path: PathBuf,
        byte_len: Option<u64>,
    ) -> DiscoveredArtifactFile {
        let layout = parse_discovered_artifact_file(&self.root, &path)
            .unwrap_or(DiscoveredArtifactFileLayout::Unrecognized);
        DiscoveredArtifactFile {
            path,
            layout,
            byte_len,
        }
    }

    async fn read(
        &self,
        selected_id: SelectedArtworkId,
        artifact: &ManagedArtworkArtifactRecord,
    ) -> Result<Vec<u8>> {
        let path = self.path_for_artifact(artifact)?;

        fs::read(&path).await.map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                TaruError::NotFound {
                    entity: "selected_artwork_image",
                    id: selected_id.to_string(),
                }
            } else {
                TaruError::Storage {
                    uri: "managed-artwork://artifact".to_owned(),
                    kind: StorageErrorKind::Io,
                    message: "failed to read managed artwork artifact".to_owned(),
                }
            }
        })
    }

    fn path_for_artifact(&self, artifact: &ManagedArtworkArtifactRecord) -> Result<PathBuf> {
        let expected_storage_uri = format!("managed-artwork://artifact/{}", artifact.id);
        if artifact.storage_uri != expected_storage_uri {
            return Err(TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::SecurityViolation,
                message: "managed artwork artifact storage reference is invalid".to_owned(),
            });
        }

        let Some(media_type) = artifact.media_type.as_deref() else {
            return Err(TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: "managed artwork artifact media type is missing".to_owned(),
            });
        };
        let extension =
            image_extension_for_media_type(media_type).ok_or_else(|| TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: "managed artwork artifact media type is unsupported".to_owned(),
            })?;
        let artifact_id_text = artifact.id.to_string();
        let shard = artifact_id_text
            .get(0..2)
            .ok_or_else(|| TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::Unknown,
                message: "managed artwork artifact id is invalid".to_owned(),
            })?;
        let path = self
            .root
            .join(shard)
            .join(format!("{artifact_id_text}.{extension}"));
        if !path_has_prefix(&path, &self.root) {
            return Err(TaruError::Storage {
                uri: "managed-artwork://artifact".to_owned(),
                kind: StorageErrorKind::SecurityViolation,
                message: "managed artwork artifact path escaped artifact root".to_owned(),
            });
        }

        Ok(path)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArtifactFileDeleteOutcome {
    Deleted,
    Missing,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArtifactFileStatus {
    Present,
    Missing,
    UnresolvableExpectedPath,
    MetadataReadFailed,
}

#[derive(Clone, Debug, Default)]
struct ArtifactStoreFileInventory {
    scanned_files: u32,
    files: Vec<DiscoveredArtifactFile>,
    truncated: bool,
}

#[derive(Clone, Debug)]
struct DiscoveredArtifactFile {
    path: PathBuf,
    layout: DiscoveredArtifactFileLayout,
    byte_len: Option<u64>,
}

impl DiscoveredArtifactFile {
    fn into_classified(
        self,
        reason: AdminManagedArtworkArtifactStorageDriftFileReason,
    ) -> ClassifiedArtifactStoreFile {
        let (recognized_artifact_id, extension) = match self.layout {
            DiscoveredArtifactFileLayout::Recognized {
                artifact_id,
                extension,
                ..
            } => (Some(artifact_id), Some(extension)),
            DiscoveredArtifactFileLayout::Unrecognized => (None, None),
        };
        ClassifiedArtifactStoreFile {
            path: self.path,
            reason,
            recognized_artifact_id,
            extension,
            byte_len: self.byte_len,
        }
    }
}

#[derive(Clone, Debug)]
enum DiscoveredArtifactFileLayout {
    Recognized {
        artifact_id: ManagedArtworkArtifactId,
        extension: String,
        supported_extension: bool,
        shard_matches: bool,
    },
    Unrecognized,
}

#[derive(Clone, Debug)]
struct ClassifiedArtifactStoreFile {
    path: PathBuf,
    reason: AdminManagedArtworkArtifactStorageDriftFileReason,
    recognized_artifact_id: Option<ManagedArtworkArtifactId>,
    extension: Option<String>,
    byte_len: Option<u64>,
}

impl ClassifiedArtifactStoreFile {
    fn to_drift_file(&self) -> AdminManagedArtworkArtifactStorageDriftFile {
        AdminManagedArtworkArtifactStorageDriftFile {
            reason: self.reason,
            recognized_artifact_id: self.recognized_artifact_id,
            extension: self.extension.clone(),
            byte_len: self.byte_len,
        }
    }

    fn into_drift_file(self) -> AdminManagedArtworkArtifactStorageDriftFile {
        AdminManagedArtworkArtifactStorageDriftFile {
            reason: self.reason,
            recognized_artifact_id: self.recognized_artifact_id,
            extension: self.extension,
            byte_len: self.byte_len,
        }
    }
}

#[derive(Clone, Debug)]
struct ManagedArtworkFailure {
    code: ManagedArtworkFailureCode,
    retryable: bool,
}

impl ManagedArtworkFailure {
    const fn new(code: ManagedArtworkFailureCode) -> Self {
        Self {
            code,
            retryable: false,
        }
    }

    const fn retryable(code: ManagedArtworkFailureCode) -> Self {
        Self {
            code,
            retryable: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ManagedArtworkFailureCode {
    UnsupportedSource,
    UnsupportedMediaType,
    TooLarge,
    InvalidImage,
    DimensionLimitExceeded,
    FetchTimeout,
    FetchFailed,
    FetchHttpStatus,
    StorageFailed,
    ResourceBudgetClosed,
}

impl ManagedArtworkFailureCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSource => "unsupported_source",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::TooLarge => "too_large",
            Self::InvalidImage => "invalid_image",
            Self::DimensionLimitExceeded => "dimension_limit_exceeded",
            Self::FetchTimeout => "fetch_timeout",
            Self::FetchFailed => "fetch_failed",
            Self::FetchHttpStatus => "fetch_http_status",
            Self::StorageFailed => "storage_failed",
            Self::ResourceBudgetClosed => "resource_budget_closed",
        }
    }
}

fn normalize_media_type(value: &str) -> Option<String> {
    let media_type = value.split(';').next()?.trim().to_ascii_lowercase();
    if media_type.is_empty() {
        None
    } else {
        Some(media_type)
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

fn image_extension_for_media_type(media_type: &str) -> Option<&'static str> {
    image_format_for_media_type(media_type).map(|(_, extension)| extension)
}

fn parse_discovered_artifact_file(
    root: &Path,
    path: &Path,
) -> Option<DiscoveredArtifactFileLayout> {
    let relative = path.strip_prefix(root).ok()?;
    let mut components = relative.components();
    let shard = components.next()?.as_os_str().to_str()?;
    let file_name = components.next()?.as_os_str().to_str()?;
    if components.next().is_some() {
        return Some(DiscoveredArtifactFileLayout::Unrecognized);
    }

    let (stem, extension) = file_name.rsplit_once('.')?;
    let artifact_id = stem.parse::<ManagedArtworkArtifactId>().ok()?;
    let expected_shard = stem.get(0..2)?;
    let normalized_extension = extension.to_ascii_lowercase();

    Some(DiscoveredArtifactFileLayout::Recognized {
        artifact_id,
        extension: normalized_extension.clone(),
        supported_extension: supported_artifact_file_extension(&normalized_extension),
        shard_matches: shard == expected_shard,
    })
}

fn supported_artifact_file_extension(extension: &str) -> bool {
    matches!(extension, "jpg" | "png" | "webp")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn path_has_prefix(path: &Path, root: &Path) -> bool {
    path.starts_with(root)
}

fn managed_artwork_artifact_store_inventory_error() -> TaruError {
    TaruError::Storage {
        uri: "managed-artwork://artifact".to_owned(),
        kind: StorageErrorKind::Io,
        message: "failed to inventory managed artwork artifact store".to_owned(),
    }
}

fn image_kind_label(kind: &taru_core::ImageKind) -> &'static str {
    match kind {
        taru_core::ImageKind::Poster => "poster",
        taru_core::ImageKind::Backdrop => "backdrop",
        taru_core::ImageKind::Logo => "logo",
        taru_core::ImageKind::Thumbnail => "thumbnail",
        taru_core::ImageKind::Banner => "banner",
        taru_core::ImageKind::Other(_) => "other",
    }
}
