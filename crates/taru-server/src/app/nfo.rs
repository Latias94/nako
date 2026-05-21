use std::sync::Arc;

use serde::Serialize;
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository, Job, JobId, JobKind,
    JobRepository, Library, LibraryId, LibraryRepository, MediaItemId, MediaRepository,
    MediaSourceId, NewJob, NewNfoSidecarApply, NewOutboxEvent, NfoSidecarApplyId,
    NfoSidecarApplyOperationKind, NfoSidecarApplyRecord, NfoSidecarApplyRepository,
    NfoSidecarApplyState, Result, TaruError, UserPrincipalId,
};
use taru_db::TaruDatabase;
use taru_nfo::{
    MovieNfoCodec, NfoAuthorityPreviewAction, NfoAuthorityPreviewDecision,
    NfoAuthorityPreviewOperation, NfoAuthorityPreviewReason, NfoAuthorityPreviewRequest,
    NfoAuthorityPreviewSummary, NfoBackupReport, NfoCancellationCheck, NfoCancellationDecision,
    NfoExportRequest, NfoExportSourceRequest, NfoExportSourceSummary, NfoExportSummary,
    NfoImportRequest, NfoImportSourceRequest, NfoImportSourceSummary, NfoImportSummary,
    NfoJobInput, NfoLibraryRunOutcome, NfoService, NfoSidecarCheckpoint,
};
use taru_vfs::{StorageBackend, StorageCapabilities, StorageUri};
use tokio::sync::Semaphore;
use tracing::{Instrument, info, info_span, warn};

use super::{
    job_runtime::{
        DurableJobContext, DurableJobOperationError, DurableJobOperationResult,
        DurableJobRunOutcome, DurableJobRuntime,
    },
    runtime::RuntimeSupervisor,
    storage::StorageBackendRegistry,
};

#[derive(Clone, Debug, Serialize)]
pub struct NfoImportCommandOutput {
    pub job: Job,
    pub import: NfoImportSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct NfoExportCommandOutput {
    pub job: Job,
    pub export: NfoExportSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcceptNfoSidecarApplyRequest {
    pub(crate) target_library_id: LibraryId,
    pub(crate) media_item_id: MediaItemId,
    pub(crate) media_source_id: Option<MediaSourceId>,
    pub(crate) requested_by: UserPrincipalId,
    pub(crate) idempotency_key: String,
    pub(crate) operation_kind: NfoSidecarApplyOperationKind,
    pub(crate) sidecar_locator: String,
    pub(crate) accepted_preview: NfoAuthorityPreviewSummary,
    pub(crate) accepted_warning_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyNfoSidecarApplyRequest {
    pub(crate) apply_id: NfoSidecarApplyId,
    pub(crate) requested_by: UserPrincipalId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NfoSidecarApplyAcceptanceDiagnostic {
    pub(crate) id: NfoSidecarApplyId,
    pub(crate) target_library_id: LibraryId,
    pub(crate) media_item_id: MediaItemId,
    pub(crate) media_source_id: Option<MediaSourceId>,
    pub(crate) requested_by: UserPrincipalId,
    pub(crate) operation_kind: NfoSidecarApplyOperationKind,
    pub(crate) sidecar_scheme: Option<String>,
    pub(crate) sidecar_locator: Option<String>,
    pub(crate) state: NfoSidecarApplyState,
    pub(crate) replayed: bool,
    pub(crate) accepted_preview_snapshot: bool,
    pub(crate) accepted_warnings_snapshot: bool,
    pub(crate) policy_version: String,
    pub(crate) has_outcome: bool,
    pub(crate) safe_error_code: Option<String>,
    pub(crate) safe_message: Option<String>,
    pub(crate) has_raw_storage_path: bool,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

impl NfoSidecarApplyAcceptanceDiagnostic {
    #[must_use]
    pub(crate) fn from_record(record: NfoSidecarApplyRecord, replayed: bool) -> Self {
        Self {
            id: record.id,
            target_library_id: record.target_library_id,
            media_item_id: record.media_item_id,
            media_source_id: record.media_source_id,
            requested_by: record.requested_by,
            operation_kind: record.operation_kind,
            sidecar_scheme: uri_scheme(&record.sidecar_locator).map(str::to_owned),
            sidecar_locator: Some(record.sidecar_locator),
            state: record.state,
            replayed,
            accepted_preview_snapshot: !record.accepted_preview_json.trim().is_empty(),
            accepted_warnings_snapshot: record.accepted_warnings_json.is_some(),
            policy_version: record.policy_version,
            has_outcome: record.outcome_json.is_some(),
            safe_error_code: record.safe_error_code,
            safe_message: record.safe_message,
            has_raw_storage_path: false,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
enum NfoImportExecution {
    Completed(NfoImportCommandOutput),
    Cancelled(Job),
}

#[derive(Clone, Debug, Serialize)]
enum NfoExportExecution {
    Completed(NfoExportCommandOutput),
    Cancelled(Job),
}

#[derive(Clone, Debug)]
pub(crate) struct NfoAppService {
    store: TaruDatabase,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
    #[cfg(test)]
    audit_failure: Option<NfoSidecarApplyAuditFailurePoint>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NfoSidecarApplyAuditFailurePoint {
    BeforeCommittedState,
    BeforeMetadataCommit,
}

impl NfoAppService {
    pub(super) fn new(
        store: TaruDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            store,
            permits,
            storage_backends,
            runtime,
            #[cfg(test)]
            audit_failure: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_audit_failure_for_test(
        mut self,
        point: NfoSidecarApplyAuditFailurePoint,
    ) -> Self {
        self.audit_failure = Some(point);
        self
    }

    pub(crate) async fn enqueue_nfo_import(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_import_job(library_id).await?;
        let job_id = job.id;
        let service = self.clone();

        self.runtime.spawn_job(
            "nfo_import_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move { service.finish_nfo_import_job(job_id, library_id).await }.instrument(
                    info_span!(
                        "nfo_import_background_job",
                        job_id = %job_id,
                        library_id = %library_id,
                        resource_class = "metadata.nfo.import"
                    ),
                )
            },
        );

        Ok(job)
    }

    pub(crate) async fn enqueue_nfo_export(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_export_job(library_id).await?;
        let job_id = job.id;
        let service = self.clone();

        self.runtime.spawn_job(
            "nfo_export_background_job",
            job.resource_class.clone(),
            job_id,
            move |_context| {
                async move { service.finish_nfo_export_job(job_id, library_id).await }.instrument(
                    info_span!(
                        "nfo_export_background_job",
                        job_id = %job_id,
                        library_id = %library_id,
                        resource_class = "metadata.nfo.export"
                    ),
                )
            },
        );

        Ok(job)
    }

    pub(crate) async fn import_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let job = self.create_nfo_import_job(library_id).await?;
        self.execute_nfo_import_command(job.id, library_id).await
    }

    pub(crate) async fn export_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let job = self.create_nfo_export_job(library_id).await?;
        self.execute_nfo_export_command(job.id, library_id).await
    }

    pub(crate) async fn preview_library_nfo_authority(
        &self,
        library_id: LibraryId,
        operation: NfoAuthorityPreviewOperation,
        force: bool,
    ) -> Result<NfoAuthorityPreviewSummary> {
        let library = self.library_for_nfo(library_id).await?;
        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        let service = NfoService::new(backend, self.store.clone(), MovieNfoCodec);

        service
            .preview_authority(NfoAuthorityPreviewRequest {
                library_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                operation,
                force,
            })
            .await
    }

    pub(crate) async fn accept_sidecar_apply(
        &self,
        request: AcceptNfoSidecarApplyRequest,
    ) -> Result<NfoSidecarApplyAcceptanceDiagnostic> {
        let idempotency_key = require_non_empty(
            "NFO sidecar apply idempotency_key",
            request.idempotency_key.clone(),
        )?;
        let sidecar_locator = require_non_empty(
            "NFO sidecar apply sidecar_locator",
            request.sidecar_locator.clone(),
        )?;
        let preview_operation = preview_operation_for_apply(request.operation_kind)?;
        let accepted_preview_json = accepted_nfo_preview_json(&request.accepted_preview)?;
        let accepted_warning_codes =
            validate_nfo_accepted_warning_codes(request.accepted_warning_codes.clone())?;

        if let Some(existing) = self
            .store
            .find_nfo_sidecar_apply_by_idempotency_key(request.target_library_id, &idempotency_key)
            .await?
        {
            validate_idempotent_nfo_sidecar_apply_replay(
                &existing,
                &request,
                &idempotency_key,
                &sidecar_locator,
                &accepted_preview_json,
                &accepted_warning_codes,
            )?;
            return Ok(NfoSidecarApplyAcceptanceDiagnostic::from_record(
                existing, true,
            ));
        }

        let accepted_decision = validate_accepted_preview_target(
            &request.accepted_preview,
            request.target_library_id,
            request.media_item_id,
            request.media_source_id,
            request.operation_kind,
            preview_operation,
            &sidecar_locator,
        )?;
        let accepted_source_id = accepted_decision.source_id;
        let accepted_warnings_json = accepted_nfo_warnings_json(
            accepted_decision,
            request.accepted_preview.force,
            &accepted_warning_codes,
        )?;

        let source = self
            .store
            .get_media_source(accepted_source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: accepted_source_id.to_string(),
            })?;
        if source.library_id != request.target_library_id
            || source.item_id != request.media_item_id
            || request
                .media_source_id
                .is_some_and(|source_id| source_id != source.id)
        {
            return Err(TaruError::Conflict {
                message: "NFO sidecar apply target no longer matches cataloged media source"
                    .to_owned(),
            });
        }
        self.store
            .get_media_item(request.media_item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: request.media_item_id.to_string(),
            })?;

        let current_preview = self
            .preview_library_nfo_authority(
                request.target_library_id,
                preview_operation,
                request.accepted_preview.force,
            )
            .await?;
        if current_preview != request.accepted_preview {
            return Err(TaruError::Conflict {
                message: "NFO sidecar apply preview is stale; refresh preview before accepting"
                    .to_owned(),
            });
        }

        let now_ms = super::current_time_ms()?;
        let outcome_json = serde_json::json!({
            "accepted": true,
            "preview_revalidated": true,
            "writes_library": false,
            "storage_mutation": false,
            "metadata_mutation": false
        })
        .to_string();
        let record = self
            .store
            .upsert_nfo_sidecar_apply(NewNfoSidecarApply {
                id: NfoSidecarApplyId::new(),
                target_library_id: request.target_library_id,
                media_item_id: request.media_item_id,
                media_source_id: request.media_source_id,
                requested_by: request.requested_by,
                idempotency_key,
                operation_kind: request.operation_kind,
                sidecar_locator,
                accepted_preview_json,
                accepted_warnings_json,
                policy_version: NFO_SIDECAR_APPLY_POLICY_VERSION.to_owned(),
                state: NfoSidecarApplyState::Accepted,
                outcome_json: Some(outcome_json),
                safe_error_code: None,
                safe_message: Some("NFO sidecar apply accepted for future execution".to_owned()),
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            })
            .await?;

        Ok(NfoSidecarApplyAcceptanceDiagnostic::from_record(
            record, false,
        ))
    }

    pub(crate) async fn apply_sidecar_apply(
        &self,
        request: ApplyNfoSidecarApplyRequest,
    ) -> Result<NfoSidecarApplyAcceptanceDiagnostic> {
        let accepted = self
            .store
            .get_nfo_sidecar_apply(request.apply_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: request.apply_id.to_string(),
            })?;

        validate_nfo_sidecar_apply_request(&accepted, request.requested_by)?;
        if accepted.state == NfoSidecarApplyState::Committed {
            return Ok(NfoSidecarApplyAcceptanceDiagnostic::from_record(
                accepted, true,
            ));
        }
        if matches!(
            accepted.state,
            NfoSidecarApplyState::RepairPending | NfoSidecarApplyState::RollbackComplete
        ) {
            return Ok(NfoSidecarApplyAcceptanceDiagnostic::from_record(
                accepted, true,
            ));
        }
        match accepted.operation_kind {
            NfoSidecarApplyOperationKind::ExportSidecar => {
                self.apply_export_sidecar_apply(accepted).await
            }
            NfoSidecarApplyOperationKind::ImportSidecar => {
                self.apply_import_sidecar_apply(accepted).await
            }
            NfoSidecarApplyOperationKind::RoundTripUpdate => Err(TaruError::Unsupported(
                "NFO round-trip sidecar apply requires a dedicated round-trip planner",
            )),
        }
    }

    async fn apply_export_sidecar_apply(
        &self,
        accepted: NfoSidecarApplyRecord,
    ) -> Result<NfoSidecarApplyAcceptanceDiagnostic> {
        let source_id = accepted
            .media_source_id
            .ok_or_else(|| TaruError::Conflict {
                message: "accepted NFO export sidecar apply does not target a media source"
                    .to_owned(),
            })?;
        let force = accepted_preview_force_from_record(&accepted)?;
        let library = self.library_for_nfo(accepted.target_library_id).await?;
        let current_preview = self
            .preview_library_nfo_authority(
                accepted.target_library_id,
                NfoAuthorityPreviewOperation::Export,
                force,
            )
            .await?;
        if accepted_nfo_preview_json(&current_preview)? != accepted.accepted_preview_json {
            self.record_nfo_sidecar_pre_mutation_failure(
                &accepted,
                "nfo_sidecar_apply_preview_stale",
                "accepted NFO sidecar apply preview is stale",
            )
            .await?;
            return Err(TaruError::Conflict {
                message:
                    "accepted NFO sidecar apply preview is stale; refresh preview before apply"
                        .to_owned(),
            });
        }
        validate_accepted_preview_target(
            &current_preview,
            accepted.target_library_id,
            accepted.media_item_id,
            accepted.media_source_id,
            accepted.operation_kind,
            NfoAuthorityPreviewOperation::Export,
            &accepted.sidecar_locator,
        )?;

        let writing = self
            .store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::WritingSidecar,
                super::current_time_ms()?,
                Some(nfo_sidecar_apply_started_outcome_json(&accepted)?),
                None,
                Some("NFO sidecar export apply started".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })?;

        let backend = match self
            .storage_backends
            .backend_for_library_root(&library)
            .await
        {
            Ok(backend) => backend,
            Err(err) => {
                self.record_nfo_sidecar_pre_mutation_failure(
                    &writing,
                    "nfo_sidecar_apply_storage_backend_unavailable",
                    "NFO sidecar export storage backend is unavailable",
                )
                .await?;
                return Err(err);
            }
        };
        let service = NfoService::new(backend.clone(), self.store.clone(), MovieNfoCodec);
        let summary = service
            .export_media_source(NfoExportSourceRequest {
                library_id: accepted.target_library_id,
                source_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force,
            })
            .await?;

        if summary.exported_items == 1 && summary.failed_items == 0 {
            return self
                .commit_export_sidecar_apply(&accepted, &summary, backend.as_ref())
                .await;
        }

        let failed = self
            .store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::FailedBeforeMutation,
                super::current_time_ms()?,
                Some(nfo_sidecar_export_not_committed_outcome_json(
                    &accepted, &summary,
                )?),
                Some("nfo_sidecar_export_not_committed".to_owned()),
                Some("NFO sidecar export apply did not commit".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })?;
        Err(TaruError::Conflict {
            message: format!(
                "NFO sidecar export apply did not commit: {}",
                failed
                    .safe_error_code
                    .as_deref()
                    .unwrap_or("nfo_sidecar_export_not_committed")
            ),
        })
    }

    async fn apply_import_sidecar_apply(
        &self,
        accepted: NfoSidecarApplyRecord,
    ) -> Result<NfoSidecarApplyAcceptanceDiagnostic> {
        let source_id = accepted
            .media_source_id
            .ok_or_else(|| TaruError::Conflict {
                message: "accepted NFO import sidecar apply does not target a media source"
                    .to_owned(),
            })?;
        let force = accepted_preview_force_from_record(&accepted)?;
        let library = self.library_for_nfo(accepted.target_library_id).await?;
        let current_preview = self
            .preview_library_nfo_authority(
                accepted.target_library_id,
                NfoAuthorityPreviewOperation::Import,
                force,
            )
            .await?;
        if accepted_nfo_preview_json(&current_preview)? != accepted.accepted_preview_json {
            self.record_nfo_sidecar_pre_mutation_failure(
                &accepted,
                "nfo_sidecar_apply_preview_stale",
                "accepted NFO sidecar apply preview is stale",
            )
            .await?;
            return Err(TaruError::Conflict {
                message:
                    "accepted NFO sidecar apply preview is stale; refresh preview before apply"
                        .to_owned(),
            });
        }
        validate_accepted_preview_target(
            &current_preview,
            accepted.target_library_id,
            accepted.media_item_id,
            accepted.media_source_id,
            accepted.operation_kind,
            NfoAuthorityPreviewOperation::Import,
            &accepted.sidecar_locator,
        )?;

        let importing = self
            .store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::ImportingMetadata,
                super::current_time_ms()?,
                Some(nfo_sidecar_import_started_outcome_json(&accepted)?),
                None,
                Some("NFO sidecar import apply started".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })?;

        let backend = match self
            .storage_backends
            .backend_for_library_root(&library)
            .await
        {
            Ok(backend) => backend,
            Err(err) => {
                self.record_nfo_sidecar_pre_mutation_failure(
                    &importing,
                    "nfo_sidecar_apply_storage_backend_unavailable",
                    "NFO sidecar import storage backend is unavailable",
                )
                .await?;
                return Err(err);
            }
        };
        let service = NfoService::new(backend, self.store.clone(), MovieNfoCodec);
        if let Err(err) = self.fail_nfo_sidecar_metadata_commit_for_test() {
            self.record_nfo_sidecar_pre_mutation_failure(
                &importing,
                "nfo_sidecar_import_metadata_commit_failed",
                "NFO sidecar import metadata commit failed before mutation",
            )
            .await?;
            return Err(err);
        }
        let summary = service
            .import_media_source(NfoImportSourceRequest {
                library_id: accepted.target_library_id,
                source_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force,
            })
            .await?;

        if summary.imported_items == 1 && summary.failed_items == 0 {
            return self.commit_import_sidecar_apply(&accepted, &summary).await;
        }

        let failed = self
            .store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::FailedBeforeMutation,
                super::current_time_ms()?,
                Some(nfo_sidecar_import_not_committed_outcome_json(
                    &accepted, &summary,
                )?),
                Some("nfo_sidecar_import_not_committed".to_owned()),
                Some("NFO sidecar import apply did not commit".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })?;
        Err(TaruError::Conflict {
            message: format!(
                "NFO sidecar import apply did not commit: {}",
                failed
                    .safe_error_code
                    .as_deref()
                    .unwrap_or("nfo_sidecar_import_not_committed")
            ),
        })
    }

    async fn record_nfo_sidecar_pre_mutation_failure(
        &self,
        accepted: &NfoSidecarApplyRecord,
        safe_error_code: &'static str,
        safe_message: &'static str,
    ) -> Result<NfoSidecarApplyRecord> {
        self.store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::FailedBeforeMutation,
                super::current_time_ms()?,
                Some(nfo_sidecar_pre_mutation_failure_outcome_json(
                    accepted,
                    safe_error_code,
                )?),
                Some(safe_error_code.to_owned()),
                Some(safe_message.to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn commit_export_sidecar_apply(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoExportSourceSummary,
        backend: &dyn StorageBackend,
    ) -> Result<NfoSidecarApplyAcceptanceDiagnostic> {
        match self.write_export_committed_audit(accepted, summary).await {
            Ok(committed) => Ok(NfoSidecarApplyAcceptanceDiagnostic::from_record(
                committed, false,
            )),
            Err(err) => {
                self.record_export_terminal_after_audit_commit_failure(accepted, summary, backend)
                    .await
                    .map_err(|_| nfo_sidecar_audit_and_repair_audit_failed_error())?;
                Err(nfo_sidecar_audit_commit_failed_error(err))
            }
        }
    }

    async fn write_export_committed_audit(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoExportSourceSummary,
    ) -> Result<NfoSidecarApplyRecord> {
        self.fail_nfo_sidecar_audit_commit_for_test()?;
        self.store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::Committed,
                super::current_time_ms()?,
                Some(nfo_sidecar_export_committed_outcome_json(
                    accepted, summary,
                )?),
                None,
                Some("NFO sidecar export apply committed".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn record_export_terminal_after_audit_commit_failure(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoExportSourceSummary,
        backend: &dyn StorageBackend,
    ) -> Result<NfoSidecarApplyRecord> {
        let rollback = attempt_export_rollback_from_backup(summary, backend).await;
        if rollback.restored {
            return self
                .record_export_rollback_complete_after_audit_commit_failure(
                    accepted, summary, &rollback,
                )
                .await;
        }

        self.record_export_repair_pending_after_audit_commit_failure(accepted, summary, &rollback)
            .await
    }

    async fn record_export_repair_pending_after_audit_commit_failure(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoExportSourceSummary,
        rollback: &NfoSidecarRollbackReport,
    ) -> Result<NfoSidecarApplyRecord> {
        self.store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::RepairPending,
                super::current_time_ms()?,
                Some(nfo_sidecar_export_repair_pending_outcome_json(
                    accepted, summary, rollback,
                )?),
                Some("nfo_sidecar_apply_audit_commit_failed".to_owned()),
                Some(
                    "NFO sidecar export apply wrote the sidecar but failed to commit final audit state"
                        .to_owned(),
                ),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn record_export_rollback_complete_after_audit_commit_failure(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoExportSourceSummary,
        rollback: &NfoSidecarRollbackReport,
    ) -> Result<NfoSidecarApplyRecord> {
        self.store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::RollbackComplete,
                super::current_time_ms()?,
                Some(nfo_sidecar_export_rollback_complete_outcome_json(
                    accepted, summary, rollback,
                )?),
                Some("nfo_sidecar_apply_audit_commit_failed_rollback_complete".to_owned()),
                Some(
                    "NFO sidecar export apply wrote the sidecar, failed final audit, and restored the previous sidecar from backup"
                        .to_owned(),
                ),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn commit_import_sidecar_apply(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoImportSourceSummary,
    ) -> Result<NfoSidecarApplyAcceptanceDiagnostic> {
        match self.write_import_committed_audit(accepted, summary).await {
            Ok(committed) => Ok(NfoSidecarApplyAcceptanceDiagnostic::from_record(
                committed, false,
            )),
            Err(err) => {
                self.record_import_repair_pending_after_audit_commit_failure(accepted, summary)
                    .await
                    .map_err(|_| nfo_sidecar_audit_and_repair_audit_failed_error())?;
                Err(nfo_sidecar_audit_commit_failed_error(err))
            }
        }
    }

    async fn write_import_committed_audit(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoImportSourceSummary,
    ) -> Result<NfoSidecarApplyRecord> {
        self.fail_nfo_sidecar_audit_commit_for_test()?;
        self.store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::Committed,
                super::current_time_ms()?,
                Some(nfo_sidecar_import_committed_outcome_json(
                    accepted, summary,
                )?),
                None,
                Some("NFO sidecar import apply committed".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn record_import_repair_pending_after_audit_commit_failure(
        &self,
        accepted: &NfoSidecarApplyRecord,
        summary: &NfoImportSourceSummary,
    ) -> Result<NfoSidecarApplyRecord> {
        self.store
            .set_nfo_sidecar_apply_state(
                accepted.id,
                NfoSidecarApplyState::RepairPending,
                super::current_time_ms()?,
                Some(nfo_sidecar_import_repair_pending_outcome_json(
                    accepted, summary,
                )?),
                Some("nfo_sidecar_apply_audit_commit_failed".to_owned()),
                Some(
                    "NFO sidecar import apply mutated metadata but failed to commit final audit state"
                        .to_owned(),
                ),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "nfo_sidecar_apply",
                id: accepted.id.to_string(),
            })
    }

    fn fail_nfo_sidecar_audit_commit_for_test(&self) -> Result<()> {
        #[cfg(test)]
        {
            if self.audit_failure == Some(NfoSidecarApplyAuditFailurePoint::BeforeCommittedState) {
                return Err(TaruError::Database {
                    message: "injected NFO sidecar apply audit commit failure".to_owned(),
                });
            }
        }
        Ok(())
    }

    fn fail_nfo_sidecar_metadata_commit_for_test(&self) -> Result<()> {
        #[cfg(test)]
        {
            if self.audit_failure == Some(NfoSidecarApplyAuditFailurePoint::BeforeMetadataCommit) {
                return Err(TaruError::Database {
                    message: "injected NFO sidecar metadata commit failure".to_owned(),
                });
            }
        }
        Ok(())
    }

    async fn create_nfo_import_job(&self, library_id: LibraryId) -> Result<Job> {
        let library = self.library_for_nfo(library_id).await?;
        let input = NfoJobInput {
            library_id,
            policy: library.options.metadata_profile.local_metadata_policy,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize NFO import job input: {err}"),
        })?;

        self.store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoImport,
                resource_class: "metadata.nfo.import".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_nfo_export_job(&self, library_id: LibraryId) -> Result<Job> {
        let library = self.library_for_nfo(library_id).await?;
        let input = NfoJobInput {
            library_id,
            policy: library.options.metadata_profile.local_metadata_policy,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize NFO export job input: {err}"),
        })?;

        self.store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoExport,
                resource_class: "metadata.nfo.export".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn finish_nfo_import_job(&self, job_id: JobId, library_id: LibraryId) -> Result<Job> {
        match self.execute_nfo_import_job(job_id, library_id).await? {
            NfoImportExecution::Completed(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    imported_items = output.import.imported_items,
                    status = ?output.job.status,
                    "NFO import job completed"
                );
                Ok(output.job)
            }
            NfoImportExecution::Cancelled(job) => {
                info!(
                    job_id = %job.id,
                    library_id = %library_id,
                    status = ?job.status,
                    "NFO import job cancelled"
                );
                Ok(job)
            }
        }
    }

    async fn finish_nfo_export_job(&self, job_id: JobId, library_id: LibraryId) -> Result<Job> {
        match self.execute_nfo_export_job(job_id, library_id).await? {
            NfoExportExecution::Completed(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    exported_items = output.export.exported_items,
                    status = ?output.job.status,
                    "NFO export job completed"
                );
                Ok(output.job)
            }
            NfoExportExecution::Cancelled(job) => {
                info!(
                    job_id = %job.id,
                    library_id = %library_id,
                    status = ?job.status,
                    "NFO export job cancelled"
                );
                Ok(job)
            }
        }
    }

    async fn execute_nfo_import_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportExecution> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("NFO concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job_with_context(
                job_id,
                "NFO import job",
                |context| async { self.run_nfo_import(job_id, library_id, context).await },
                |import| DurableJobRuntime::serialize_summary(import, "NFO import job summary"),
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => {
                let import = run.output;
                self.record_nfo_imported_event(job_id, library_id, &import)
                    .await;

                Ok(NfoImportExecution::Completed(NfoImportCommandOutput {
                    job: run.job,
                    import,
                }))
            }
            DurableJobRunOutcome::Cancelled(job) => Ok(NfoImportExecution::Cancelled(job)),
        }
    }

    async fn execute_nfo_export_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportExecution> {
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("NFO concurrency limiter is unavailable: {err}"),
                })?;
        let _permit = permit;

        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job_with_context(
                job_id,
                "NFO export job",
                |context| async { self.run_nfo_export(job_id, library_id, context).await },
                |export| DurableJobRuntime::serialize_summary(export, "NFO export job summary"),
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => {
                let export = run.output;
                self.record_nfo_exported_event(job_id, library_id, &export)
                    .await;

                Ok(NfoExportExecution::Completed(NfoExportCommandOutput {
                    job: run.job,
                    export,
                }))
            }
            DurableJobRunOutcome::Cancelled(job) => Ok(NfoExportExecution::Cancelled(job)),
        }
    }

    async fn execute_nfo_import_command(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        match self.execute_nfo_import_job(job_id, library_id).await? {
            NfoImportExecution::Completed(output) => Ok(output),
            NfoImportExecution::Cancelled(job) => Err(TaruError::Conflict {
                message: format!("NFO import job {} was cancelled", job.id),
            }),
        }
    }

    async fn execute_nfo_export_command(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        match self.execute_nfo_export_job(job_id, library_id).await? {
            NfoExportExecution::Completed(output) => Ok(output),
            NfoExportExecution::Cancelled(job) => Err(TaruError::Conflict {
                message: format!("NFO export job {} was cancelled", job.id),
            }),
        }
    }

    async fn record_nfo_imported_event(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        import: &NfoImportSummary,
    ) {
        let payload = serde_json::json!({
            "job_id": job_id,
            "library_id": library_id,
            "scanned_sources": import.scanned_sources,
            "discovered_nfo": import.discovered_nfo,
            "imported_items": import.imported_items,
            "skipped_items": import.skipped_items,
            "failed_items": import.failed_items,
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::NfoImported,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("nfo.imported:{job_id}:{library_id}"),
            payload_json: payload.to_string(),
        })
        .await;
    }

    async fn record_nfo_exported_event(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        export: &NfoExportSummary,
    ) {
        let payload = serde_json::json!({
            "job_id": job_id,
            "library_id": library_id,
            "scanned_sources": export.scanned_sources,
            "exported_items": export.exported_items,
            "skipped_items": export.skipped_items,
            "failed_items": export.failed_items,
            "backed_up_items": export.backed_up_items,
            "pruned_backup_items": export.pruned_backup_items,
            "pruned_backups": export.pruned_backups,
            "prune_failures": export.prune_failures.len(),
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::NfoExported,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: format!("nfo.exported:{job_id}:{library_id}"),
            payload_json: payload.to_string(),
        })
        .await;
    }

    async fn run_nfo_import(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        context: DurableJobContext,
    ) -> DurableJobOperationResult<NfoImportSummary> {
        let library = self.library_for_nfo(library_id).await?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            policy = ?library.options.metadata_profile.local_metadata_policy,
            "starting NFO import job"
        );

        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        let service = NfoService::new(backend, self.store.clone(), MovieNfoCodec);
        let cancellation = DurableNfoCancellationCheck {
            context: context.clone(),
        };

        context.check_cancelled().await?;
        let outcome = service
            .import_library_with_cancellation(
                NfoImportRequest {
                    job_id,
                    library_id,
                    policy: library.options.metadata_profile.local_metadata_policy,
                    force: false,
                },
                &cancellation,
            )
            .await?;
        let summary = match outcome {
            NfoLibraryRunOutcome::Completed(summary) => summary,
            NfoLibraryRunOutcome::Cancelled(_summary) => {
                return Err(DurableJobOperationError::Cancelled);
            }
        };
        context.check_cancelled().await?;

        Ok(summary)
    }

    async fn run_nfo_export(
        &self,
        job_id: JobId,
        library_id: LibraryId,
        context: DurableJobContext,
    ) -> DurableJobOperationResult<NfoExportSummary> {
        let library = self.library_for_nfo(library_id).await?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            policy = ?library.options.metadata_profile.local_metadata_policy,
            "starting NFO export job"
        );

        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        ensure_nfo_export_writable(backend.as_ref(), &library).await?;
        let service = NfoService::new(backend, self.store.clone(), MovieNfoCodec);
        let cancellation = DurableNfoCancellationCheck {
            context: context.clone(),
        };

        context.check_cancelled().await?;
        let outcome = service
            .export_library_with_cancellation(
                NfoExportRequest {
                    job_id,
                    library_id,
                    policy: library.options.metadata_profile.local_metadata_policy,
                    force: false,
                },
                &cancellation,
            )
            .await?;
        let summary = match outcome {
            NfoLibraryRunOutcome::Completed(summary) => summary,
            NfoLibraryRunOutcome::Cancelled(_summary) => {
                return Err(DurableJobOperationError::Cancelled);
            }
        };
        context.check_cancelled().await?;

        Ok(summary)
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

    async fn library_for_nfo(&self, library_id: LibraryId) -> Result<Library> {
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
    }
}

const NFO_SIDECAR_APPLY_POLICY_VERSION: &str = "nfo-sidecar-acceptance-v1";

pub(crate) async fn ensure_nfo_export_writable(
    backend: &dyn StorageBackend,
    library: &taru_core::Library,
) -> Result<()> {
    let root = library
        .roots
        .first()
        .map(String::as_str)
        .unwrap_or("local:///");
    let uri = StorageUri::parse(root)?;
    let metadata = backend.stat(&uri).await?;

    if metadata
        .capabilities
        .contains(StorageCapabilities::WRITABLE)
    {
        Ok(())
    } else {
        Err(TaruError::Unsupported(
            "NFO export requires a writable storage backend",
        ))
    }
}

fn preview_operation_for_apply(
    operation_kind: NfoSidecarApplyOperationKind,
) -> Result<NfoAuthorityPreviewOperation> {
    match operation_kind {
        NfoSidecarApplyOperationKind::ExportSidecar => Ok(NfoAuthorityPreviewOperation::Export),
        NfoSidecarApplyOperationKind::ImportSidecar => Ok(NfoAuthorityPreviewOperation::Import),
        NfoSidecarApplyOperationKind::RoundTripUpdate => Err(TaruError::Unsupported(
            "NFO round-trip sidecar apply requires a dedicated round-trip planner",
        )),
    }
}

fn validate_accepted_preview_target<'a>(
    preview: &'a NfoAuthorityPreviewSummary,
    target_library_id: LibraryId,
    media_item_id: MediaItemId,
    media_source_id: Option<MediaSourceId>,
    operation_kind: NfoSidecarApplyOperationKind,
    preview_operation: NfoAuthorityPreviewOperation,
    sidecar_locator: &str,
) -> Result<&'a NfoAuthorityPreviewDecision> {
    if preview.library_id != target_library_id || preview.operation != preview_operation {
        return Err(TaruError::Conflict {
            message: "NFO sidecar apply preview does not match the requested library or operation"
                .to_owned(),
        });
    }

    let decision = preview
        .decisions
        .iter()
        .find(|decision| {
            decision.item_id == media_item_id
                && media_source_id.is_none_or(|source_id| decision.source_id == source_id)
        })
        .ok_or_else(|| TaruError::Conflict {
            message: "NFO sidecar apply target is not present in the accepted preview".to_owned(),
        })?;

    let decision_sidecar = decision.nfo_uri.as_ref().map(ToString::to_string);
    if decision_sidecar.as_deref() != Some(sidecar_locator) {
        return Err(TaruError::Conflict {
            message: "NFO sidecar apply sidecar locator does not match the accepted preview"
                .to_owned(),
        });
    }

    if !decision_is_accept_ready(decision, operation_kind) {
        return Err(TaruError::Conflict {
            message: format!(
                "NFO sidecar apply target is not ready for acceptance: {:?}",
                decision.reason
            ),
        });
    }

    Ok(decision)
}

fn decision_is_accept_ready(
    decision: &NfoAuthorityPreviewDecision,
    operation_kind: NfoSidecarApplyOperationKind,
) -> bool {
    match operation_kind {
        NfoSidecarApplyOperationKind::ExportSidecar => matches!(
            (decision.action, decision.reason),
            (
                NfoAuthorityPreviewAction::Create,
                NfoAuthorityPreviewReason::ExportWouldCreateSidecar
            ) | (
                NfoAuthorityPreviewAction::Update,
                NfoAuthorityPreviewReason::ExportWouldUpdateExistingSidecar
            )
        ),
        NfoSidecarApplyOperationKind::ImportSidecar => matches!(
            (decision.action, decision.reason),
            (
                NfoAuthorityPreviewAction::Update,
                NfoAuthorityPreviewReason::ImportWouldReadSidecar
            )
        ),
        NfoSidecarApplyOperationKind::RoundTripUpdate => false,
    }
}

fn validate_idempotent_nfo_sidecar_apply_replay(
    existing: &NfoSidecarApplyRecord,
    request: &AcceptNfoSidecarApplyRequest,
    idempotency_key: &str,
    sidecar_locator: &str,
    accepted_preview_json: &str,
    accepted_warning_codes: &[String],
) -> Result<()> {
    if existing.target_library_id != request.target_library_id
        || existing.media_item_id != request.media_item_id
        || existing.media_source_id != request.media_source_id
        || existing.requested_by != request.requested_by
        || existing.idempotency_key != idempotency_key
        || existing.operation_kind != request.operation_kind
        || existing.sidecar_locator != sidecar_locator
        || existing.accepted_preview_json != accepted_preview_json
        || existing.policy_version != NFO_SIDECAR_APPLY_POLICY_VERSION
        || !accepted_warnings_match_replay(
            existing.accepted_warnings_json.as_deref(),
            accepted_warning_codes,
        )
    {
        return Err(TaruError::Conflict {
            message: "NFO sidecar apply idempotency key was already used for a different acceptance request".to_owned(),
        });
    }

    Ok(())
}

fn validate_nfo_sidecar_apply_request(
    record: &NfoSidecarApplyRecord,
    requested_by: UserPrincipalId,
) -> Result<()> {
    if record.requested_by != requested_by {
        return Err(TaruError::Forbidden {
            message: "NFO sidecar apply requester does not match the accepted operator".to_owned(),
        });
    }
    if !matches!(
        record.state,
        NfoSidecarApplyState::Accepted
            | NfoSidecarApplyState::Committed
            | NfoSidecarApplyState::RepairPending
            | NfoSidecarApplyState::RollbackComplete
    ) {
        return Err(TaruError::Conflict {
            message: format!(
                "NFO sidecar apply is not in an applyable state: {}",
                record.state.as_str()
            ),
        });
    }

    Ok(())
}

fn accepted_preview_force_from_record(record: &NfoSidecarApplyRecord) -> Result<bool> {
    serde_json::from_str::<serde_json::Value>(&record.accepted_preview_json)
        .ok()
        .and_then(|value| value.get("force").and_then(serde_json::Value::as_bool))
        .ok_or_else(|| TaruError::Database {
            message: "accepted NFO sidecar preview snapshot is missing force".to_owned(),
        })
}

fn validate_nfo_accepted_warning_codes(warnings: Vec<String>) -> Result<Vec<String>> {
    let mut normalized = Vec::with_capacity(warnings.len());
    for warning in warnings {
        let warning = require_non_empty("NFO sidecar apply accepted warning code", warning)?;
        if !warning
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
        {
            return Err(TaruError::InvalidInput {
                message: "NFO sidecar apply accepted warning codes must be safe lowercase tokens"
                    .to_owned(),
            });
        }
        if !normalized.contains(&warning) {
            normalized.push(warning);
        }
    }
    normalized.sort();

    Ok(normalized)
}

fn accepted_nfo_warnings_json(
    decision: &NfoAuthorityPreviewDecision,
    force: bool,
    accepted_warning_codes: &[String],
) -> Result<Option<String>> {
    if !force && !decision.backup_required && accepted_warning_codes.is_empty() {
        return Ok(None);
    }

    serde_json::to_string(&serde_json::json!({
        "accepted_warning_codes": accepted_warning_codes,
        "force": force,
        "backup_required": decision.backup_required,
        "preview_action": preview_action_name(decision.action),
        "preview_reason": preview_reason_name(decision.reason)
    }))
    .map(Some)
    .map_err(database_error)
}

fn accepted_warnings_match_replay(
    existing_warnings_json: Option<&str>,
    accepted_warning_codes: &[String],
) -> bool {
    if existing_warnings_json.is_none() {
        return accepted_warning_codes.is_empty();
    }

    existing_warnings_json
        .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
        .and_then(|value| value.get("accepted_warning_codes").cloned())
        .and_then(|codes| serde_json::from_value::<Vec<String>>(codes).ok())
        .is_some_and(|mut codes| {
            codes.sort();
            codes == accepted_warning_codes
        })
}

fn nfo_sidecar_apply_started_outcome_json(record: &NfoSidecarApplyRecord) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": false,
        "storage_mutation": false,
        "metadata_mutation": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::WritingSidecar,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator)
    }))
    .map_err(database_error)
}

fn nfo_sidecar_import_started_outcome_json(record: &NfoSidecarApplyRecord) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": true,
        "storage_mutation": false,
        "metadata_mutation": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::ImportingMetadata,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator)
    }))
    .map_err(database_error)
}

fn nfo_sidecar_export_committed_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoExportSourceSummary,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": true,
        "writes_library": true,
        "storage_mutation": true,
        "metadata_mutation": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::Committed,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "sidecar_locator": record.sidecar_locator,
        "source_id": summary.source_id,
        "exported_items": summary.exported_items,
        "backed_up_items": summary.backed_up_items,
        "pruned_backup_items": summary.pruned_backup_items,
        "pruned_backups": summary.pruned_backups,
        "backup_count": summary.backups.len(),
        "prune_failure_count": summary.prune_failures.len()
    }))
    .map_err(database_error)
}

fn nfo_sidecar_import_committed_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoImportSourceSummary,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": true,
        "writes_library": true,
        "storage_mutation": false,
        "metadata_mutation": true,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::Committed,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "source_id": summary.source_id,
        "discovered_nfo": summary.discovered_nfo,
        "imported_items": summary.imported_items,
        "skipped_items": summary.skipped_items,
        "failed_items": summary.failed_items,
        "failure_count": summary.failures.len()
    }))
    .map_err(database_error)
}

fn nfo_sidecar_export_not_committed_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoExportSourceSummary,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": false,
        "storage_mutation": false,
        "metadata_mutation": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::FailedBeforeMutation,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "source_id": summary.source_id,
        "exported_items": summary.exported_items,
        "skipped_items": summary.skipped_items,
        "failed_items": summary.failed_items,
        "failure_count": summary.failures.len()
    }))
    .map_err(database_error)
}

fn nfo_sidecar_export_repair_pending_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoExportSourceSummary,
    rollback: &NfoSidecarRollbackReport,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": true,
        "storage_mutation": true,
        "metadata_mutation": false,
        "repair_required": true,
        "audit_commit_completed": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::RepairPending,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "source_id": summary.source_id,
        "exported_items": summary.exported_items,
        "backed_up_items": summary.backed_up_items,
        "backup_count": summary.backups.len(),
        "prune_failure_count": summary.prune_failures.len(),
        "rollback_attempted": rollback.attempted,
        "rollback_complete": rollback.restored,
        "rollback_status": rollback.status,
        "rollback_backup_count": rollback.backup_count
    }))
    .map_err(database_error)
}

fn nfo_sidecar_export_rollback_complete_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoExportSourceSummary,
    rollback: &NfoSidecarRollbackReport,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": false,
        "storage_mutation": true,
        "metadata_mutation": false,
        "repair_required": false,
        "audit_commit_completed": false,
        "rollback_attempted": rollback.attempted,
        "rollback_complete": rollback.restored,
        "rollback_status": rollback.status,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::RollbackComplete,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "source_id": summary.source_id,
        "exported_items": summary.exported_items,
        "backed_up_items": summary.backed_up_items,
        "backup_count": summary.backups.len(),
        "rollback_backup_count": rollback.backup_count,
        "prune_failure_count": summary.prune_failures.len()
    }))
    .map_err(database_error)
}

fn nfo_sidecar_import_not_committed_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoImportSourceSummary,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": false,
        "storage_mutation": false,
        "metadata_mutation": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::FailedBeforeMutation,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "source_id": summary.source_id,
        "discovered_nfo": summary.discovered_nfo,
        "imported_items": summary.imported_items,
        "skipped_items": summary.skipped_items,
        "failed_items": summary.failed_items,
        "failure_count": summary.failures.len()
    }))
    .map_err(database_error)
}

fn nfo_sidecar_import_repair_pending_outcome_json(
    record: &NfoSidecarApplyRecord,
    summary: &NfoImportSourceSummary,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": true,
        "storage_mutation": false,
        "metadata_mutation": true,
        "repair_required": true,
        "audit_commit_completed": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::RepairPending,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "source_id": summary.source_id,
        "discovered_nfo": summary.discovered_nfo,
        "imported_items": summary.imported_items,
        "failure_count": summary.failures.len()
    }))
    .map_err(database_error)
}

fn nfo_sidecar_pre_mutation_failure_outcome_json(
    record: &NfoSidecarApplyRecord,
    safe_error_code: &str,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "committed": false,
        "writes_library": false,
        "storage_mutation": false,
        "metadata_mutation": false,
        "operation_kind": record.operation_kind,
        "state": NfoSidecarApplyState::FailedBeforeMutation,
        "sidecar_scheme": uri_scheme(&record.sidecar_locator),
        "safe_error_code": safe_error_code
    }))
    .map_err(database_error)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NfoSidecarRollbackReport {
    attempted: bool,
    restored: bool,
    status: &'static str,
    backup_count: usize,
}

impl NfoSidecarRollbackReport {
    const fn not_attempted(backup_count: usize) -> Self {
        Self {
            attempted: false,
            restored: false,
            status: "not_attempted",
            backup_count,
        }
    }
}

async fn attempt_export_rollback_from_backup(
    summary: &NfoExportSourceSummary,
    backend: &dyn StorageBackend,
) -> NfoSidecarRollbackReport {
    let Some(backup) = newest_export_backup(summary) else {
        return NfoSidecarRollbackReport::not_attempted(summary.backups.len());
    };

    match backend
        .restore(taru_vfs::StorageRestoreRequest::new(
            backup.backup_uri.clone(),
            backup.original_uri.clone(),
        ))
        .await
    {
        Ok(report) if report.restored => NfoSidecarRollbackReport {
            attempted: true,
            restored: true,
            status: "restored",
            backup_count: summary.backups.len(),
        },
        Ok(report) => NfoSidecarRollbackReport {
            attempted: true,
            restored: false,
            status: storage_restore_status_code(report.status),
            backup_count: summary.backups.len(),
        },
        Err(_) => NfoSidecarRollbackReport {
            attempted: true,
            restored: false,
            status: "restore_error",
            backup_count: summary.backups.len(),
        },
    }
}

fn newest_export_backup(summary: &NfoExportSourceSummary) -> Option<&NfoBackupReport> {
    summary
        .backups
        .iter()
        .max_by(|left, right| left.backup_uri.as_str().cmp(right.backup_uri.as_str()))
}

fn storage_restore_status_code(status: taru_vfs::StorageRestoreStatus) -> &'static str {
    match status {
        taru_vfs::StorageRestoreStatus::Restored => "restored",
        taru_vfs::StorageRestoreStatus::Unsupported => "unsupported",
        taru_vfs::StorageRestoreStatus::BackupMissing => "backup_missing",
        taru_vfs::StorageRestoreStatus::BackupNotFile => "backup_not_file",
        taru_vfs::StorageRestoreStatus::TargetParentMissing => "target_parent_missing",
        taru_vfs::StorageRestoreStatus::TargetParentNotDirectory => "target_parent_not_directory",
        taru_vfs::StorageRestoreStatus::SecurityViolation => "security_violation",
        taru_vfs::StorageRestoreStatus::RestoreFailed => "restore_failed",
    }
}

fn nfo_sidecar_audit_commit_failed_error(error: TaruError) -> TaruError {
    TaruError::Conflict {
        message: format!("nfo_sidecar_apply_audit_commit_failed: {error}"),
    }
}

fn nfo_sidecar_audit_and_repair_audit_failed_error() -> TaruError {
    TaruError::Database {
        message:
            "NFO sidecar apply mutation completed, committed-state audit failed, and repair-pending audit also failed"
                .to_owned(),
    }
}

fn accepted_nfo_preview_json(preview: &NfoAuthorityPreviewSummary) -> Result<String> {
    serde_json::to_string(&NfoAuthorityPreviewSnapshot::from(preview)).map_err(database_error)
}

#[derive(Serialize)]
struct NfoAuthorityPreviewSnapshot<'a> {
    library_id: LibraryId,
    operation: &'static str,
    policy: taru_core::LocalMetadataPolicy,
    force: bool,
    scanned_sources: u64,
    create_items: u64,
    skip_items: u64,
    update_items: u64,
    backup_required_items: u64,
    policy_rejected_items: u64,
    failure_items: u64,
    decisions: Vec<NfoAuthorityPreviewDecisionSnapshot<'a>>,
}

impl<'a> From<&'a NfoAuthorityPreviewSummary> for NfoAuthorityPreviewSnapshot<'a> {
    fn from(preview: &'a NfoAuthorityPreviewSummary) -> Self {
        Self {
            library_id: preview.library_id,
            operation: preview_operation_name(preview.operation),
            policy: preview.policy,
            force: preview.force,
            scanned_sources: preview.scanned_sources,
            create_items: preview.create_items,
            skip_items: preview.skip_items,
            update_items: preview.update_items,
            backup_required_items: preview.backup_required_items,
            policy_rejected_items: preview.policy_rejected_items,
            failure_items: preview.failure_items,
            decisions: preview
                .decisions
                .iter()
                .map(NfoAuthorityPreviewDecisionSnapshot::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct NfoAuthorityPreviewDecisionSnapshot<'a> {
    source_id: MediaSourceId,
    item_id: MediaItemId,
    locator: &'a str,
    nfo_uri: Option<String>,
    content_fingerprint: Option<&'a str>,
    action: &'static str,
    reason: &'static str,
    backup_required: bool,
    message: &'a str,
}

impl<'a> From<&'a NfoAuthorityPreviewDecision> for NfoAuthorityPreviewDecisionSnapshot<'a> {
    fn from(decision: &'a NfoAuthorityPreviewDecision) -> Self {
        Self {
            source_id: decision.source_id,
            item_id: decision.item_id,
            locator: &decision.locator,
            nfo_uri: decision.nfo_uri.as_ref().map(ToString::to_string),
            content_fingerprint: decision.content_fingerprint.as_deref(),
            action: preview_action_name(decision.action),
            reason: preview_reason_name(decision.reason),
            backup_required: decision.backup_required,
            message: &decision.message,
        }
    }
}

fn preview_operation_name(operation: NfoAuthorityPreviewOperation) -> &'static str {
    match operation {
        NfoAuthorityPreviewOperation::Import => "import",
        NfoAuthorityPreviewOperation::Export => "export",
    }
}

fn preview_action_name(action: NfoAuthorityPreviewAction) -> &'static str {
    match action {
        NfoAuthorityPreviewAction::Create => "create",
        NfoAuthorityPreviewAction::Skip => "skip",
        NfoAuthorityPreviewAction::Update => "update",
        NfoAuthorityPreviewAction::PolicyRejected => "policy_rejected",
        NfoAuthorityPreviewAction::Fail => "fail",
    }
}

fn preview_reason_name(reason: NfoAuthorityPreviewReason) -> &'static str {
    match reason {
        NfoAuthorityPreviewReason::ExportWouldCreateSidecar => "export_would_create_sidecar",
        NfoAuthorityPreviewReason::ExportWouldSkipExistingSidecar => {
            "export_would_skip_existing_sidecar"
        }
        NfoAuthorityPreviewReason::ExportWouldUpdateExistingSidecar => {
            "export_would_update_existing_sidecar"
        }
        NfoAuthorityPreviewReason::ImportWouldReadSidecar => "import_would_read_sidecar",
        NfoAuthorityPreviewReason::ImportSidecarMissing => "import_sidecar_missing",
        NfoAuthorityPreviewReason::PolicyDoesNotAllowOperation => "policy_does_not_allow_operation",
        NfoAuthorityPreviewReason::UnsupportedMediaKind => "unsupported_media_kind",
        NfoAuthorityPreviewReason::MissingMediaItem => "missing_media_item",
        NfoAuthorityPreviewReason::InvalidSidecarPath => "invalid_sidecar_path",
        NfoAuthorityPreviewReason::StorageReadFailed => "storage_read_failed",
        NfoAuthorityPreviewReason::StorageUnsupported => "storage_unsupported",
        NfoAuthorityPreviewReason::NfoParseFailed => "nfo_parse_failed",
        NfoAuthorityPreviewReason::NfoRenderFailed => "nfo_render_failed",
        NfoAuthorityPreviewReason::NfoPreservationFailed => "nfo_preservation_failed",
    }
}

fn require_non_empty(label: &str, value: String) -> Result<String> {
    optional_non_empty(Some(value)).ok_or_else(|| TaruError::InvalidInput {
        message: format!("{label} cannot be empty"),
    })
}

fn optional_non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn database_error<E: std::fmt::Display>(err: E) -> TaruError {
    TaruError::Database {
        message: err.to_string(),
    }
}

fn uri_scheme(value: &str) -> Option<&str> {
    value
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|scheme| !scheme.is_empty())
}

#[derive(Clone, Debug)]
struct DurableNfoCancellationCheck {
    context: DurableJobContext,
}

#[async_trait::async_trait]
impl NfoCancellationCheck for DurableNfoCancellationCheck {
    async fn check(
        &self,
        _checkpoint: NfoSidecarCheckpoint,
    ) -> taru_core::Result<NfoCancellationDecision> {
        match self.context.check_cancelled().await {
            Ok(()) => Ok(NfoCancellationDecision::Continue),
            Err(DurableJobOperationError::Cancelled) => Ok(NfoCancellationDecision::Cancel),
            Err(DurableJobOperationError::Failed(err)) => Err(err),
        }
    }
}
