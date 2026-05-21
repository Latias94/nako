use std::path::{Component, Path};

use serde::Serialize;
use taru_core::{
    CanonicalMetadata, Library, LibraryId, LibraryItemRepository, LibraryItemState, LibraryPreset,
    LibraryRepository, LocalMetadataPolicy, ManagedImportArtifactId,
    ManagedImportArtifactListFilter, ManagedImportArtifactRecord, ManagedImportArtifactState,
    ManagedImportPromotionApplyId, ManagedImportPromotionApplyRecord,
    ManagedImportPromotionApplyState, ManagedImportPromotionBlockedReason,
    ManagedImportPromotionDuplicateHint, ManagedImportPromotionFileOperation,
    ManagedImportPromotionNfoAuthorityHint, ManagedImportPromotionOperationKind,
    ManagedImportPromotionOperationStatus, ManagedImportPromotionPlan,
    ManagedImportPromotionProviderIdentityHint, ManagedImportRepository, ManagedImportSourceKind,
    MediaItem, MediaItemId, MediaKind, MediaRepository, MediaSource, MediaSourceId,
    NewManagedImportArtifact, NewManagedImportPromotionApply, PageRequest, Result,
    SourceDuplicateEvidenceKind, SourceDuplicateRelationship, SourceDuplicateRelationshipId,
    SourceDuplicateRelationshipStatus, SourceDuplicateRepository, StagingManifestId,
    StagingManifestRepository, TaruError, UserPrincipalId,
};
use taru_db::TaruDatabase;
use taru_vfs::{
    StorageApplyKind, StorageApplyReport, StorageApplyRequest, StorageApplyStatus, StorageBackend,
    StorageLinkKind, StorageLinkPlanRequest, StorageUri,
};

use super::storage::StorageBackendRegistry;

#[derive(Clone, Debug)]
pub(crate) struct ManagedImportAppService {
    store: TaruDatabase,
    storage_backends: Option<StorageBackendRegistry>,
}

impl ManagedImportAppService {
    pub(crate) fn new(store: TaruDatabase) -> Self {
        Self {
            store,
            storage_backends: None,
        }
    }

    pub(super) fn new_with_storage(
        store: TaruDatabase,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            store,
            storage_backends: Some(storage_backends),
        }
    }

    pub(crate) async fn create_artifact(
        &self,
        request: CreateManagedImportArtifactRequest,
    ) -> Result<ManagedImportArtifactDiagnostic> {
        self.store
            .get_library(request.target_library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: request.target_library_id.to_string(),
            })?;

        let source_uri = require_non_empty("managed import source_uri", request.source_uri)?;
        let state = request.state.unwrap_or(match request.staging_manifest_id {
            Some(_) => ManagedImportArtifactState::Staged,
            None => ManagedImportArtifactState::Proposed,
        });
        validate_create_state(state)?;

        let staging_manifest = match request.staging_manifest_id {
            Some(id) => Some(
                self.store
                    .get_staging_manifest_record(id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "staging_manifest_record",
                        id: id.to_string(),
                    })?,
            ),
            None => None,
        };
        let now_ms = super::current_time_ms()?;
        let artifact = NewManagedImportArtifact {
            id: request.id.unwrap_or_else(ManagedImportArtifactId::new),
            target_library_id: request.target_library_id,
            source_kind: request.source_kind,
            source_uri,
            staging_manifest_id: request.staging_manifest_id,
            artifact_uri: optional_non_empty(request.artifact_uri),
            original_file_name: optional_non_empty(request.original_file_name),
            intended_locator: optional_non_empty(request.intended_locator),
            size_bytes: request.size_bytes.or_else(|| {
                staging_manifest
                    .as_ref()
                    .and_then(|record| record.size_bytes)
            }),
            fingerprint: request
                .fingerprint
                .or_else(|| staging_manifest.and_then(|record| record.fingerprint)),
            state,
            diagnostics_json: optional_non_empty(request.diagnostics_json),
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        };

        let record = self.store.upsert_managed_import_artifact(artifact).await?;

        Ok(ManagedImportArtifactDiagnostic::from_record(record))
    }

    pub(crate) async fn list_artifacts(
        &self,
        filter: ManagedImportArtifactListFilter,
        page: PageRequest,
    ) -> Result<ManagedImportArtifactDiagnostics> {
        let page = page.clamped();
        if let Some(library_id) = filter.target_library_id {
            self.store
                .get_library(library_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "library",
                    id: library_id.to_string(),
                })?;
        }

        let records = self
            .store
            .list_managed_import_artifacts(filter, page)
            .await?;
        let returned = records.len();
        let artifacts = records
            .into_iter()
            .map(ManagedImportArtifactDiagnostic::from_record)
            .collect::<Vec<_>>();

        Ok(ManagedImportArtifactDiagnostics {
            limit: page.limit,
            offset: page.offset,
            returned,
            artifacts,
        })
    }

    pub(crate) async fn preview_promotion_plan(
        &self,
        artifact_id: ManagedImportArtifactId,
    ) -> Result<ManagedImportPromotionPlan> {
        let artifact = self
            .store
            .get_managed_import_artifact(artifact_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_artifact",
                id: artifact_id.to_string(),
            })?;
        let library = self
            .store
            .get_library(artifact.target_library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: artifact.target_library_id.to_string(),
            })?;
        let destination_locator =
            destination_locator(&library, artifact.intended_locator.as_deref());
        let mut blocked_reasons =
            promotion_blocked_reasons(&artifact, &library, destination_locator.as_ref());
        let file_operations = self
            .preview_file_operations(
                &artifact,
                &library,
                destination_locator.as_ref(),
                &mut blocked_reasons,
            )
            .await;
        let duplicate_hints = self.duplicate_hints(&artifact).await?;
        let nfo_authority = self
            .nfo_authority_hint(&library, destination_locator.as_ref())
            .await;
        let provider_identity = provider_identity_hint(&library, &artifact);

        if provider_identity.needs_identity_review
            && !blocked_reasons
                .contains(&ManagedImportPromotionBlockedReason::ProviderIdentityMissing)
        {
            blocked_reasons.push(ManagedImportPromotionBlockedReason::ProviderIdentityMissing);
        }

        Ok(ManagedImportPromotionPlan {
            artifact_id: artifact.id,
            artifact_state: artifact.state,
            target_library_id: library.id,
            target_library_name: library.name,
            destination_locator,
            file_operations,
            duplicate_hints,
            nfo_authority,
            provider_identity,
            blocked_reasons,
        })
    }

    pub(crate) async fn accept_promotion(
        &self,
        request: AcceptManagedImportPromotionRequest,
    ) -> Result<ManagedImportPromotionAcceptanceDiagnostic> {
        if request.operation_kind == ManagedImportPromotionOperationKind::Move {
            return Err(TaruError::Unsupported(
                "managed import move promotion apply is deferred until source-retention semantics are proven",
            ));
        }

        let idempotency_key = require_non_empty(
            "managed import promotion idempotency_key",
            request.idempotency_key.clone(),
        )?;
        let plan = self.preview_promotion_plan(request.artifact_id).await?;

        if let Some(existing) = self
            .store
            .find_managed_import_promotion_apply_by_idempotency_key(
                plan.target_library_id,
                &idempotency_key,
            )
            .await?
        {
            validate_idempotent_promotion_replay(&existing, &request, &plan)?;
            return Ok(ManagedImportPromotionAcceptanceDiagnostic::from_record(
                existing, true,
            ));
        }

        let blocking_reasons = promotion_acceptance_blocking_reasons(&plan, request.operation_kind);
        if !blocking_reasons.is_empty() {
            return Err(TaruError::Conflict {
                message: format!(
                    "promotion plan is blocked: {}",
                    promotion_blocked_reason_summary(&blocking_reasons)
                ),
            });
        }

        let operation = plan
            .file_operations
            .iter()
            .find(|operation| operation.kind == request.operation_kind)
            .ok_or_else(|| TaruError::InvalidInput {
                message: format!(
                    "promotion operation is not available in the current plan: {}",
                    request.operation_kind.as_str()
                ),
            })?;
        if !operation.can_apply || operation.status != ManagedImportPromotionOperationStatus::Ready
        {
            return Err(TaruError::Conflict {
                message: format!(
                    "promotion operation is not ready: {}",
                    request.operation_kind.as_str()
                ),
            });
        }

        let destination_locator =
            plan.destination_locator
                .clone()
                .ok_or_else(|| TaruError::Conflict {
                    message: "promotion plan does not have a destination locator".to_owned(),
                })?;
        let artifact = self
            .store
            .get_managed_import_artifact(request.artifact_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_artifact",
                id: request.artifact_id.to_string(),
            })?;
        let now_ms = super::current_time_ms()?;
        let accepted_plan_json = accepted_promotion_plan_json(&plan, request.operation_kind)?;
        let accepted_warnings_json = accepted_blocked_reasons_json(
            &request.accepted_blocked_reasons,
            !plan.duplicate_hints.is_empty(),
            plan.nfo_authority.backup_required,
            plan.provider_identity.needs_identity_review,
        )?;
        let outcome_json = serde_json::json!({
            "accepted": true,
            "writes_library": false,
            "storage_mutation": false,
            "media_source_mutation": false
        })
        .to_string();
        let record = self
            .store
            .upsert_managed_import_promotion_apply(NewManagedImportPromotionApply {
                id: ManagedImportPromotionApplyId::new(),
                artifact_id: artifact.id,
                target_library_id: plan.target_library_id,
                requested_by: request.requested_by,
                idempotency_key,
                operation_kind: request.operation_kind,
                source_artifact_uri: artifact.artifact_uri,
                destination_locator,
                accepted_plan_json,
                accepted_warnings_json,
                state: ManagedImportPromotionApplyState::Accepted,
                outcome_json: Some(outcome_json),
                safe_error_code: None,
                safe_message: Some("promotion accepted for future storage apply".to_owned()),
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            })
            .await?;

        Ok(ManagedImportPromotionAcceptanceDiagnostic::from_record(
            record, false,
        ))
    }

    pub(crate) async fn apply_promotion(
        &self,
        request: ApplyManagedImportPromotionRequest,
    ) -> Result<ManagedImportPromotionAcceptanceDiagnostic> {
        let accepted = self
            .store
            .get_managed_import_promotion_apply(request.apply_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_promotion_apply",
                id: request.apply_id.to_string(),
            })?;

        validate_promotion_apply_request(&accepted, request.requested_by)?;
        if accepted.state == ManagedImportPromotionApplyState::Promoted {
            return Ok(ManagedImportPromotionAcceptanceDiagnostic::from_record(
                accepted, true,
            ));
        }

        let plan = self.preview_promotion_plan(accepted.artifact_id).await?;
        let artifact = self
            .store
            .get_managed_import_artifact(accepted.artifact_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_artifact",
                id: accepted.artifact_id.to_string(),
            })?;
        let library = self
            .store
            .get_library(accepted.target_library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: accepted.target_library_id.to_string(),
            })?;

        if let Err(err) = revalidate_promotion_apply_facts(&accepted, &artifact, &plan) {
            self.record_pre_mutation_apply_failure(
                &accepted,
                "promotion_apply_revalidation_failed",
                "accepted promotion facts are stale or blocked",
            )
            .await?;
            return Err(err);
        }

        let source_uri =
            match StorageUri::parse(accepted.source_artifact_uri.as_deref().ok_or_else(|| {
                TaruError::Conflict {
                    message: "accepted promotion no longer has a source artifact URI".to_owned(),
                }
            })?) {
                Ok(uri) => uri,
                Err(err) => {
                    self.record_pre_mutation_apply_failure(
                        &accepted,
                        "promotion_apply_invalid_source_uri",
                        "accepted promotion source URI is invalid",
                    )
                    .await?;
                    return Err(err);
                }
            };
        let target_uri = match StorageUri::parse(&accepted.destination_locator) {
            Ok(uri) => uri,
            Err(err) => {
                self.record_pre_mutation_apply_failure(
                    &accepted,
                    "promotion_apply_invalid_destination_locator",
                    "accepted promotion destination locator is invalid",
                )
                .await?;
                return Err(err);
            }
        };
        if self
            .store
            .get_media_source_by_locator(accepted.target_library_id, &accepted.destination_locator)
            .await?
            .is_some()
        {
            self.record_pre_mutation_apply_failure(
                &accepted,
                "promotion_apply_destination_already_cataloged",
                "promotion destination locator is already cataloged",
            )
            .await?;
            return Err(TaruError::Conflict {
                message: "promotion destination locator is already cataloged".to_owned(),
            });
        }
        let apply_kind = storage_apply_kind(accepted.operation_kind)?;
        let Some(storage_backends) = self.storage_backends.as_ref() else {
            self.record_pre_mutation_apply_failure(
                &accepted,
                "promotion_apply_storage_registry_unavailable",
                "storage backend registry is required before promotion apply",
            )
            .await?;
            return Err(TaruError::Conflict {
                message: "storage backend registry is required to apply a promotion".to_owned(),
            });
        };
        let backend = match storage_backends.backend_for_library_root(&library).await {
            Ok(backend) => backend,
            Err(err) => {
                self.record_pre_mutation_apply_failure(
                    &accepted,
                    "promotion_apply_storage_backend_unavailable",
                    "storage backend is unavailable before promotion apply",
                )
                .await?;
                return Err(err);
            }
        };
        let now_ms = super::current_time_ms()?;
        let _applying = self
            .store
            .set_managed_import_promotion_apply_state(
                accepted.id,
                ManagedImportPromotionApplyState::ApplyingStorage,
                now_ms,
                Some(storage_applying_outcome_json(&accepted)?),
                None,
                Some("promotion storage apply started".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_promotion_apply",
                id: accepted.id.to_string(),
            })?;

        let apply_report = match backend
            .apply(StorageApplyRequest::new(source_uri, target_uri, apply_kind))
            .await
        {
            Ok(report) => report,
            Err(err) => {
                self.record_pre_mutation_apply_failure(
                    &accepted,
                    "promotion_apply_storage_backend_error",
                    "storage backend failed before reporting promotion apply outcome",
                )
                .await?;
                return Err(err);
            }
        };
        if !apply_report.applied
            || !apply_report.target_created
            || apply_report.status != StorageApplyStatus::Applied
        {
            let updated = self
                .record_storage_apply_rejection(&accepted, &apply_report)
                .await?;
            return Err(TaruError::Conflict {
                message: format!(
                    "promotion storage apply failed before catalog mutation: {}",
                    updated
                        .safe_error_code
                        .as_deref()
                        .unwrap_or("storage_apply_failed")
                ),
            });
        }

        let committing = self
            .store
            .set_managed_import_promotion_apply_state(
                accepted.id,
                ManagedImportPromotionApplyState::CommittingCatalog,
                super::current_time_ms()?,
                Some(storage_applied_outcome_json(
                    &accepted,
                    &plan,
                    &apply_report,
                )?),
                None,
                Some("promotion storage target created; committing catalog".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_promotion_apply",
                id: accepted.id.to_string(),
            })?;

        let catalog_commit = self
            .commit_promoted_media_source(&artifact, &library, &committing, &plan, &apply_report)
            .await?;
        let promoted = self
            .store
            .set_managed_import_promotion_apply_state(
                committing.id,
                ManagedImportPromotionApplyState::Promoted,
                super::current_time_ms()?,
                Some(promoted_outcome_json(
                    &committing,
                    &plan,
                    &apply_report,
                    &catalog_commit,
                )?),
                None,
                Some("promotion applied and catalog source committed".to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_promotion_apply",
                id: committing.id.to_string(),
            })?;
        self.store
            .set_managed_import_artifact_state(
                artifact.id,
                ManagedImportArtifactState::Promoted,
                super::current_time_ms()?,
                artifact.diagnostics_json,
            )
            .await?;

        Ok(ManagedImportPromotionAcceptanceDiagnostic::from_record(
            promoted, false,
        ))
    }

    async fn record_pre_mutation_apply_failure(
        &self,
        accepted: &ManagedImportPromotionApplyRecord,
        safe_error_code: &'static str,
        safe_message: &'static str,
    ) -> Result<ManagedImportPromotionApplyRecord> {
        self.store
            .set_managed_import_promotion_apply_state(
                accepted.id,
                ManagedImportPromotionApplyState::FailedBeforeMutation,
                super::current_time_ms()?,
                Some(pre_mutation_failure_outcome_json(
                    accepted,
                    safe_error_code,
                )?),
                Some(safe_error_code.to_owned()),
                Some(safe_message.to_owned()),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_promotion_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn record_storage_apply_rejection(
        &self,
        accepted: &ManagedImportPromotionApplyRecord,
        apply_report: &StorageApplyReport,
    ) -> Result<ManagedImportPromotionApplyRecord> {
        self.store
            .set_managed_import_promotion_apply_state(
                accepted.id,
                ManagedImportPromotionApplyState::FailedBeforeMutation,
                super::current_time_ms()?,
                Some(storage_apply_failure_outcome_json(accepted, apply_report)?),
                Some(storage_apply_error_code(apply_report.status).to_owned()),
                Some(storage_apply_safe_message(apply_report.status)),
            )
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_promotion_apply",
                id: accepted.id.to_string(),
            })
    }

    async fn commit_promoted_media_source(
        &self,
        artifact: &ManagedImportArtifactRecord,
        library: &Library,
        apply: &ManagedImportPromotionApplyRecord,
        plan: &ManagedImportPromotionPlan,
        apply_report: &StorageApplyReport,
    ) -> Result<PromotionCatalogCommit> {
        let media_source_id = MediaSourceId::new();
        let item_id = MediaItemId::new();
        let item = MediaItem {
            id: item_id,
            kind: media_kind_for_library(library),
            parent_id: None,
            metadata: CanonicalMetadata {
                title: promotion_item_title(artifact, &apply.destination_locator),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: media_source_id,
            library_id: apply.target_library_id,
            item_id,
            locator: apply.destination_locator.clone(),
            file_name: promotion_file_name(artifact, &apply.destination_locator),
            size_bytes: artifact
                .size_bytes
                .or_else(|| apply_report.target.as_ref().and_then(|target| target.len)),
            fingerprint: artifact.fingerprint.clone(),
        };

        self.store.upsert_media_item(&item).await?;
        self.store
            .upsert_library_item_state(&LibraryItemState {
                library_id: apply.target_library_id,
                item_id,
                provisional: false,
            })
            .await?;
        self.store.upsert_media_source(&source).await?;

        let duplicate_relationship_count = self
            .commit_duplicate_relationships(media_source_id, plan)
            .await?;

        Ok(PromotionCatalogCommit {
            item_id,
            source_id: media_source_id,
            duplicate_relationship_count,
        })
    }

    async fn commit_duplicate_relationships(
        &self,
        source_id: MediaSourceId,
        plan: &ManagedImportPromotionPlan,
    ) -> Result<usize> {
        let mut committed = 0usize;
        for hint in &plan.duplicate_hints {
            let Some(existing_source_id) = hint.existing_source_id else {
                continue;
            };
            let relationship = SourceDuplicateRelationship {
                id: SourceDuplicateRelationshipId::new(),
                source_id,
                duplicate_source_id: existing_source_id,
                evidence_kind: hint.evidence_kind.clone(),
                evidence_value: None,
                status: SourceDuplicateRelationshipStatus::Suggested,
                confidence_milli: hint.confidence_milli,
            };
            self.store
                .upsert_source_duplicate_relationship(&relationship)
                .await?;
            committed += 1;
        }

        Ok(committed)
    }

    async fn preview_file_operations(
        &self,
        artifact: &ManagedImportArtifactRecord,
        library: &Library,
        destination_locator: Option<&String>,
        blocked_reasons: &mut Vec<ManagedImportPromotionBlockedReason>,
    ) -> Vec<ManagedImportPromotionFileOperation> {
        let source_uri = artifact
            .artifact_uri
            .as_deref()
            .and_then(|uri| StorageUri::parse(uri).ok());
        let target_uri = destination_locator.and_then(|locator| StorageUri::parse(locator).ok());
        let source_scheme = source_uri.as_ref().map(|uri| uri.scheme().to_owned());

        if artifact.artifact_uri.is_some() && source_uri.is_none() {
            push_blocked_once(
                blocked_reasons,
                ManagedImportPromotionBlockedReason::InvalidArtifactUri,
            );
        }
        if destination_locator.is_some() && target_uri.is_none() {
            push_blocked_once(
                blocked_reasons,
                ManagedImportPromotionBlockedReason::InvalidDestinationLocator,
            );
        }

        let preconditions_ready = promotion_storage_preconditions_ready(blocked_reasons);
        let mut operations = vec![
            static_file_operation(
                ManagedImportPromotionOperationKind::Copy,
                source_scheme.clone(),
                destination_locator.cloned(),
                preconditions_ready && source_uri.is_some() && target_uri.is_some(),
                "copy promotion requires operator acceptance and future apply",
            ),
            static_file_operation(
                ManagedImportPromotionOperationKind::Move,
                source_scheme.clone(),
                destination_locator.cloned(),
                preconditions_ready && source_uri.is_some() && target_uri.is_some(),
                "move promotion requires operator acceptance, rollback, and cleanup audit",
            ),
        ];

        let Some(source_uri) = source_uri else {
            operations.push(blocked_file_operation(
                ManagedImportPromotionOperationKind::Hardlink,
                source_scheme.clone(),
                destination_locator.cloned(),
                "hardlink planning requires a staged artifact URI",
            ));
            operations.push(blocked_file_operation(
                ManagedImportPromotionOperationKind::Symlink,
                source_scheme,
                destination_locator.cloned(),
                "symlink planning requires a staged artifact URI",
            ));
            return operations;
        };
        let Some(target_uri) = target_uri else {
            operations.push(blocked_file_operation(
                ManagedImportPromotionOperationKind::Hardlink,
                Some(source_uri.scheme().to_owned()),
                destination_locator.cloned(),
                "hardlink planning requires a destination locator",
            ));
            operations.push(blocked_file_operation(
                ManagedImportPromotionOperationKind::Symlink,
                Some(source_uri.scheme().to_owned()),
                destination_locator.cloned(),
                "symlink planning requires a destination locator",
            ));
            return operations;
        };

        if !promotion_storage_preconditions_ready(blocked_reasons) {
            operations.push(blocked_file_operation(
                ManagedImportPromotionOperationKind::Hardlink,
                Some(source_uri.scheme().to_owned()),
                Some(target_uri.to_string()),
                "link planning requires a staged artifact and library-safe destination",
            ));
            operations.push(blocked_file_operation(
                ManagedImportPromotionOperationKind::Symlink,
                Some(source_uri.scheme().to_owned()),
                Some(target_uri.to_string()),
                "link planning requires a staged artifact and library-safe destination",
            ));
            return operations;
        }

        let Some(storage_backends) = &self.storage_backends else {
            push_blocked_once(
                blocked_reasons,
                ManagedImportPromotionBlockedReason::StoragePlanningUnavailable,
            );
            operations.push(unsupported_file_operation(
                ManagedImportPromotionOperationKind::Hardlink,
                Some(source_uri.scheme().to_owned()),
                Some(target_uri.to_string()),
                "storage backend registry is unavailable for link planning",
            ));
            operations.push(unsupported_file_operation(
                ManagedImportPromotionOperationKind::Symlink,
                Some(source_uri.scheme().to_owned()),
                Some(target_uri.to_string()),
                "storage backend registry is unavailable for link planning",
            ));
            return operations;
        };

        let backend = match storage_backends.backend_for_library_root(library).await {
            Ok(backend) => backend,
            Err(err) => {
                push_blocked_once(
                    blocked_reasons,
                    ManagedImportPromotionBlockedReason::StoragePlanningUnavailable,
                );
                let message = safe_storage_planning_message(&err);
                operations.push(unsupported_file_operation(
                    ManagedImportPromotionOperationKind::Hardlink,
                    Some(source_uri.scheme().to_owned()),
                    Some(target_uri.to_string()),
                    message.clone(),
                ));
                operations.push(unsupported_file_operation(
                    ManagedImportPromotionOperationKind::Symlink,
                    Some(source_uri.scheme().to_owned()),
                    Some(target_uri.to_string()),
                    message,
                ));
                return operations;
            }
        };

        operations.push(
            link_file_operation(
                backend.as_ref(),
                source_uri.clone(),
                target_uri.clone(),
                StorageLinkKind::Hard,
                ManagedImportPromotionOperationKind::Hardlink,
            )
            .await,
        );
        operations.push(
            link_file_operation(
                backend.as_ref(),
                source_uri,
                target_uri,
                StorageLinkKind::Soft,
                ManagedImportPromotionOperationKind::Symlink,
            )
            .await,
        );

        operations
    }

    async fn duplicate_hints(
        &self,
        artifact: &ManagedImportArtifactRecord,
    ) -> Result<Vec<ManagedImportPromotionDuplicateHint>> {
        let Some(fingerprint) = artifact.fingerprint.as_deref() else {
            return Ok(Vec::new());
        };
        let candidates = self
            .store
            .list_media_sources(
                artifact.target_library_id,
                PageRequest::new(PageRequest::MAX_LIMIT, 0),
            )
            .await?;

        Ok(candidates
            .into_iter()
            .filter_map(|source| duplicate_hint_for_source(artifact, &source, fingerprint))
            .collect())
    }

    async fn nfo_authority_hint(
        &self,
        library: &Library,
        destination_locator: Option<&String>,
    ) -> ManagedImportPromotionNfoAuthorityHint {
        let policy = library.options.metadata_profile.local_metadata_policy;
        let sidecar_locator = destination_locator.and_then(|locator| nfo_sidecar_locator(locator));
        let has_sidecar = match (self.storage_backends.as_ref(), sidecar_locator.as_deref()) {
            (Some(storage_backends), Some(sidecar_locator)) => {
                match (
                    storage_backends.backend_for_library_root(library).await,
                    StorageUri::parse(sidecar_locator),
                ) {
                    (Ok(backend), Ok(uri)) => backend.stat(&uri).await.is_ok(),
                    _ => false,
                }
            }
            _ => false,
        };
        let import_supported = matches!(
            policy,
            LocalMetadataPolicy::ReadOnly
                | LocalMetadataPolicy::LocalFirst
                | LocalMetadataPolicy::RemoteFirst
        );
        let export_supported = policy == LocalMetadataPolicy::WriteSidecar;

        ManagedImportPromotionNfoAuthorityHint {
            policy,
            sidecar_locator,
            has_sidecar,
            import_supported,
            export_supported,
            would_read_sidecar: has_sidecar && import_supported,
            would_create_sidecar: !has_sidecar && export_supported && destination_locator.is_some(),
            backup_required: has_sidecar && export_supported,
            message: nfo_authority_message(policy, has_sidecar, destination_locator.is_some()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CreateManagedImportArtifactRequest {
    pub(crate) id: Option<ManagedImportArtifactId>,
    pub(crate) target_library_id: LibraryId,
    pub(crate) source_kind: ManagedImportSourceKind,
    pub(crate) source_uri: String,
    pub(crate) staging_manifest_id: Option<StagingManifestId>,
    pub(crate) artifact_uri: Option<String>,
    pub(crate) original_file_name: Option<String>,
    pub(crate) intended_locator: Option<String>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) fingerprint: Option<String>,
    pub(crate) state: Option<ManagedImportArtifactState>,
    pub(crate) diagnostics_json: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcceptManagedImportPromotionRequest {
    pub(crate) artifact_id: ManagedImportArtifactId,
    pub(crate) requested_by: UserPrincipalId,
    pub(crate) idempotency_key: String,
    pub(crate) operation_kind: ManagedImportPromotionOperationKind,
    pub(crate) accepted_blocked_reasons: Vec<ManagedImportPromotionBlockedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyManagedImportPromotionRequest {
    pub(crate) apply_id: ManagedImportPromotionApplyId,
    pub(crate) requested_by: UserPrincipalId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PromotionCatalogCommit {
    item_id: MediaItemId,
    source_id: MediaSourceId,
    duplicate_relationship_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ManagedImportArtifactDiagnostics {
    pub(crate) limit: u32,
    pub(crate) offset: u64,
    pub(crate) returned: usize,
    pub(crate) artifacts: Vec<ManagedImportArtifactDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ManagedImportArtifactDiagnostic {
    pub(crate) id: ManagedImportArtifactId,
    pub(crate) target_library_id: LibraryId,
    pub(crate) source_kind: String,
    pub(crate) custom_source_kind: bool,
    pub(crate) source_scheme: Option<String>,
    pub(crate) source_uri_redacted: String,
    pub(crate) staging_manifest_id: Option<StagingManifestId>,
    pub(crate) has_artifact_uri: bool,
    pub(crate) has_original_file_name: bool,
    pub(crate) has_intended_locator: bool,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) has_fingerprint: bool,
    pub(crate) state: ManagedImportArtifactState,
    pub(crate) has_diagnostics: bool,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ManagedImportPromotionAcceptanceDiagnostic {
    pub(crate) id: ManagedImportPromotionApplyId,
    pub(crate) artifact_id: ManagedImportArtifactId,
    pub(crate) target_library_id: LibraryId,
    pub(crate) requested_by: UserPrincipalId,
    pub(crate) operation_kind: ManagedImportPromotionOperationKind,
    pub(crate) source_scheme: Option<String>,
    pub(crate) destination_locator: Option<String>,
    pub(crate) state: ManagedImportPromotionApplyState,
    pub(crate) replayed: bool,
    pub(crate) accepted_plan_snapshot: bool,
    pub(crate) accepted_warnings_snapshot: bool,
    pub(crate) has_outcome: bool,
    pub(crate) safe_error_code: Option<String>,
    pub(crate) safe_message: Option<String>,
    pub(crate) has_raw_source_uri: bool,
    pub(crate) has_raw_fingerprint: bool,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

impl ManagedImportPromotionAcceptanceDiagnostic {
    #[must_use]
    pub(crate) fn from_record(record: ManagedImportPromotionApplyRecord, replayed: bool) -> Self {
        Self {
            id: record.id,
            artifact_id: record.artifact_id,
            target_library_id: record.target_library_id,
            requested_by: record.requested_by,
            operation_kind: record.operation_kind,
            source_scheme: record
                .source_artifact_uri
                .as_deref()
                .and_then(uri_scheme)
                .map(str::to_owned),
            destination_locator: Some(record.destination_locator),
            state: record.state,
            replayed,
            accepted_plan_snapshot: !record.accepted_plan_json.trim().is_empty(),
            accepted_warnings_snapshot: record.accepted_warnings_json.is_some(),
            has_outcome: record.outcome_json.is_some(),
            safe_error_code: record.safe_error_code,
            safe_message: record.safe_message,
            has_raw_source_uri: false,
            has_raw_fingerprint: false,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

impl ManagedImportArtifactDiagnostic {
    #[must_use]
    pub(crate) fn from_record(record: ManagedImportArtifactRecord) -> Self {
        let (source_kind, source_kind_key) = record.source_kind.as_parts();
        Self {
            id: record.id,
            target_library_id: record.target_library_id,
            source_kind: source_kind.to_owned(),
            custom_source_kind: !source_kind_key.is_empty(),
            source_scheme: uri_scheme(&record.source_uri).map(str::to_owned),
            source_uri_redacted: redact_uri(&record.source_uri),
            staging_manifest_id: record.staging_manifest_id,
            has_artifact_uri: record.artifact_uri.is_some(),
            has_original_file_name: record.original_file_name.is_some(),
            has_intended_locator: record.intended_locator.is_some(),
            size_bytes: record.size_bytes,
            has_fingerprint: record.fingerprint.is_some(),
            state: record.state,
            has_diagnostics: record.diagnostics_json.is_some(),
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

fn validate_create_state(state: ManagedImportArtifactState) -> Result<()> {
    if matches!(
        state,
        ManagedImportArtifactState::Accepted
            | ManagedImportArtifactState::Applying
            | ManagedImportArtifactState::Promoted
            | ManagedImportArtifactState::CleanupPending
            | ManagedImportArtifactState::Cleaned
    ) {
        return Err(TaruError::InvalidInput {
            message: format!(
                "managed import creation cannot start in mutating lifecycle state: {}",
                state.as_str()
            ),
        });
    }

    Ok(())
}

fn validate_idempotent_promotion_replay(
    existing: &ManagedImportPromotionApplyRecord,
    request: &AcceptManagedImportPromotionRequest,
    plan: &ManagedImportPromotionPlan,
) -> Result<()> {
    let destination_matches = plan
        .destination_locator
        .as_deref()
        .is_some_and(|locator| locator == existing.destination_locator);
    if existing.artifact_id != request.artifact_id
        || existing.operation_kind != request.operation_kind
        || existing.requested_by != request.requested_by
        || existing.target_library_id != plan.target_library_id
        || !destination_matches
    {
        return Err(TaruError::Conflict {
            message: "managed import promotion idempotency key was already used for a different acceptance request".to_owned(),
        });
    }

    Ok(())
}

fn validate_promotion_apply_request(
    record: &ManagedImportPromotionApplyRecord,
    requested_by: UserPrincipalId,
) -> Result<()> {
    if record.requested_by != requested_by {
        return Err(TaruError::Forbidden {
            message:
                "managed import promotion apply requester does not match the accepted operator"
                    .to_owned(),
        });
    }
    if !matches!(
        record.state,
        ManagedImportPromotionApplyState::Accepted | ManagedImportPromotionApplyState::Promoted
    ) {
        return Err(TaruError::Conflict {
            message: format!(
                "managed import promotion apply is not in an applyable state: {}",
                record.state.as_str()
            ),
        });
    }

    Ok(())
}

fn revalidate_promotion_apply_facts(
    accepted: &ManagedImportPromotionApplyRecord,
    artifact: &ManagedImportArtifactRecord,
    plan: &ManagedImportPromotionPlan,
) -> Result<()> {
    if accepted.target_library_id != plan.target_library_id
        || artifact.target_library_id != accepted.target_library_id
    {
        return Err(TaruError::Conflict {
            message: "promotion target library changed after acceptance".to_owned(),
        });
    }
    if artifact.artifact_uri != accepted.source_artifact_uri {
        return Err(TaruError::Conflict {
            message: "promotion source artifact URI changed after acceptance".to_owned(),
        });
    }
    if plan.destination_locator.as_deref() != Some(accepted.destination_locator.as_str()) {
        return Err(TaruError::Conflict {
            message: "promotion destination locator changed after acceptance".to_owned(),
        });
    }

    let blocking_reasons = promotion_apply_blocking_reasons(plan);
    if !blocking_reasons.is_empty() {
        return Err(TaruError::Conflict {
            message: format!(
                "promotion plan is blocked before apply: {}",
                promotion_blocked_reason_summary(&blocking_reasons)
            ),
        });
    }

    let operation = plan
        .file_operations
        .iter()
        .find(|operation| operation.kind == accepted.operation_kind)
        .ok_or_else(|| TaruError::Conflict {
            message: format!(
                "accepted promotion operation is no longer available: {}",
                accepted.operation_kind.as_str()
            ),
        })?;
    if !operation.can_apply || operation.status != ManagedImportPromotionOperationStatus::Ready {
        return Err(TaruError::Conflict {
            message: format!(
                "accepted promotion operation is no longer ready: {}",
                accepted.operation_kind.as_str()
            ),
        });
    }

    Ok(())
}

fn accepted_promotion_plan_json(
    plan: &ManagedImportPromotionPlan,
    operation_kind: ManagedImportPromotionOperationKind,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "artifact_id": plan.artifact_id,
        "artifact_state": plan.artifact_state,
        "target_library_id": plan.target_library_id,
        "destination_locator": plan.destination_locator,
        "operation_kind": operation_kind,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "nfo_policy": plan.nfo_authority.policy,
        "provider_identity_review": plan.provider_identity.needs_identity_review,
        "blocked_reasons": plan.blocked_reasons,
        "writes_library": false
    }))
    .map_err(database_error)
}

fn accepted_blocked_reasons_json(
    accepted_blocked_reasons: &[ManagedImportPromotionBlockedReason],
    has_duplicate_hints: bool,
    nfo_backup_required: bool,
    provider_identity_review: bool,
) -> Result<Option<String>> {
    if accepted_blocked_reasons.is_empty()
        && !has_duplicate_hints
        && !nfo_backup_required
        && !provider_identity_review
    {
        return Ok(None);
    }

    serde_json::to_string(&serde_json::json!({
        "accepted_blocked_reasons": accepted_blocked_reasons,
        "has_duplicate_hints": has_duplicate_hints,
        "nfo_backup_required": nfo_backup_required,
        "provider_identity_review": provider_identity_review
    }))
    .map(Some)
    .map_err(database_error)
}

fn promotion_blocked_reason_summary(reasons: &[ManagedImportPromotionBlockedReason]) -> String {
    reasons
        .iter()
        .map(|reason| format!("{reason:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn promotion_acceptance_blocking_reasons(
    plan: &ManagedImportPromotionPlan,
    operation_kind: ManagedImportPromotionOperationKind,
) -> Vec<ManagedImportPromotionBlockedReason> {
    plan.blocked_reasons
        .iter()
        .copied()
        .filter(|reason| {
            !matches!(
                (operation_kind, reason),
                (
                    ManagedImportPromotionOperationKind::Copy,
                    ManagedImportPromotionBlockedReason::StoragePlanningUnavailable
                )
            )
        })
        .collect()
}

fn promotion_apply_blocking_reasons(
    plan: &ManagedImportPromotionPlan,
) -> Vec<ManagedImportPromotionBlockedReason> {
    plan.blocked_reasons.clone()
}

fn storage_apply_kind(
    operation_kind: ManagedImportPromotionOperationKind,
) -> Result<StorageApplyKind> {
    match operation_kind {
        ManagedImportPromotionOperationKind::Copy => Ok(StorageApplyKind::Copy),
        ManagedImportPromotionOperationKind::Hardlink => Ok(StorageApplyKind::Hardlink),
        ManagedImportPromotionOperationKind::Symlink => Ok(StorageApplyKind::Symlink),
        ManagedImportPromotionOperationKind::Move => Err(TaruError::Unsupported(
            "managed import move promotion apply is deferred until source-retention semantics are proven",
        )),
    }
}

fn media_kind_for_library(library: &Library) -> MediaKind {
    match library.options.preset {
        LibraryPreset::Movies => MediaKind::Movie,
        LibraryPreset::Tv | LibraryPreset::Anime => MediaKind::Episode,
        _ => MediaKind::Unknown,
    }
}

fn promotion_item_title(
    artifact: &ManagedImportArtifactRecord,
    destination_locator: &str,
) -> String {
    let candidate = destination_locator
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .or(artifact.original_file_name.as_deref())
        .unwrap_or("Untitled");
    let without_query = candidate
        .split_once('?')
        .map_or(candidate, |(left, _)| left);
    without_query
        .rsplit_once('.')
        .map_or(without_query, |(stem, _)| stem)
        .trim()
        .to_owned()
}

fn promotion_file_name(
    artifact: &ManagedImportArtifactRecord,
    destination_locator: &str,
) -> String {
    destination_locator
        .rsplit('/')
        .next()
        .and_then(|name| optional_non_empty(Some(name.to_owned())))
        .or_else(|| {
            artifact
                .original_file_name
                .as_ref()
                .and_then(|name| optional_non_empty(Some(name.clone())))
        })
        .unwrap_or_else(|| "media-source".to_owned())
}

fn storage_applying_outcome_json(record: &ManagedImportPromotionApplyRecord) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": false,
        "media_source_mutation": false,
        "operation_kind": record.operation_kind,
        "state": ManagedImportPromotionApplyState::ApplyingStorage
    }))
    .map_err(database_error)
}

fn storage_applied_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    plan: &ManagedImportPromotionPlan,
    apply_report: &StorageApplyReport,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": apply_report.applied,
        "media_source_mutation": false,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme()
    }))
    .map_err(database_error)
}

fn promoted_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    plan: &ManagedImportPromotionPlan,
    apply_report: &StorageApplyReport,
    catalog_commit: &PromotionCatalogCommit,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": true,
        "storage_mutation": apply_report.applied,
        "media_source_mutation": true,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme(),
        "destination_locator": record.destination_locator,
        "media_item_id": catalog_commit.item_id,
        "media_source_id": catalog_commit.source_id,
        "duplicate_hint_count": plan.duplicate_hints.len(),
        "duplicate_relationship_count": catalog_commit.duplicate_relationship_count
    }))
    .map_err(database_error)
}

fn storage_apply_failure_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    apply_report: &StorageApplyReport,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": false,
        "media_source_mutation": false,
        "target_created": apply_report.target_created,
        "operation_kind": record.operation_kind,
        "operation_status": apply_report.status,
        "source_scheme": apply_report.source_uri.scheme(),
        "target_scheme": apply_report.target_uri.scheme()
    }))
    .map_err(database_error)
}

fn pre_mutation_failure_outcome_json(
    record: &ManagedImportPromotionApplyRecord,
    safe_error_code: &str,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "accepted": true,
        "writes_library": false,
        "storage_mutation": false,
        "media_source_mutation": false,
        "target_created": false,
        "operation_kind": record.operation_kind,
        "safe_error_code": safe_error_code
    }))
    .map_err(database_error)
}

fn storage_apply_error_code(status: StorageApplyStatus) -> &'static str {
    match status {
        StorageApplyStatus::Applied => "storage_apply_applied",
        StorageApplyStatus::Unsupported => "storage_apply_unsupported",
        StorageApplyStatus::SourceMissing => "storage_apply_source_missing",
        StorageApplyStatus::SourceNotFile => "storage_apply_source_not_file",
        StorageApplyStatus::TargetParentMissing => "storage_apply_target_parent_missing",
        StorageApplyStatus::TargetParentNotDirectory => "storage_apply_target_parent_not_directory",
        StorageApplyStatus::TargetExists => "storage_apply_target_exists",
        StorageApplyStatus::SecurityViolation => "storage_apply_security_violation",
        StorageApplyStatus::ApplyFailed => "storage_apply_failed",
    }
}

fn storage_apply_safe_message(status: StorageApplyStatus) -> String {
    match status {
        StorageApplyStatus::Unsupported => {
            "storage backend does not support the accepted apply kind"
        }
        StorageApplyStatus::SourceMissing => "promotion source artifact is missing",
        StorageApplyStatus::SourceNotFile => "promotion source artifact is not a file",
        StorageApplyStatus::TargetParentMissing => "promotion target parent is missing",
        StorageApplyStatus::TargetParentNotDirectory => {
            "promotion target parent is not a directory"
        }
        StorageApplyStatus::TargetExists => "promotion target already exists",
        StorageApplyStatus::SecurityViolation => {
            "promotion storage apply violated storage safety rules"
        }
        StorageApplyStatus::ApplyFailed => "promotion storage apply failed before catalog mutation",
        StorageApplyStatus::Applied => "promotion storage apply succeeded",
    }
    .to_owned()
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

fn redact_uri(value: &str) -> String {
    uri_scheme(value)
        .map(|scheme| format!("{scheme}://<redacted>"))
        .unwrap_or_else(|| "<redacted>".to_owned())
}

fn destination_locator(library: &Library, intended_locator: Option<&str>) -> Option<String> {
    let intended_locator = optional_non_empty(intended_locator.map(str::to_owned))?;
    if intended_locator.contains("://") {
        return Some(intended_locator);
    }

    let root = library
        .roots
        .first()
        .cloned()
        .unwrap_or_else(|| "local:///".to_owned());
    let relative = intended_locator.trim_start_matches(['/', '\\']);

    Some(join_storage_locator(&root, relative))
}

fn promotion_blocked_reasons(
    artifact: &ManagedImportArtifactRecord,
    library: &Library,
    destination_locator: Option<&String>,
) -> Vec<ManagedImportPromotionBlockedReason> {
    let mut reasons = Vec::new();

    if !matches!(
        artifact.state,
        ManagedImportArtifactState::Staged
            | ManagedImportArtifactState::Inspected
            | ManagedImportArtifactState::Planned
    ) {
        reasons.push(ManagedImportPromotionBlockedReason::ArtifactNotReady);
    }
    if artifact.artifact_uri.is_none() {
        reasons.push(ManagedImportPromotionBlockedReason::MissingArtifactUri);
    }
    if destination_locator.is_none() {
        reasons.push(ManagedImportPromotionBlockedReason::MissingDestinationLocator);
    }
    if destination_locator.is_some_and(|locator| !locator_is_within_library(library, locator)) {
        reasons.push(ManagedImportPromotionBlockedReason::DestinationEscapesLibrary);
    }

    reasons
}

fn promotion_storage_preconditions_ready(
    blocked_reasons: &[ManagedImportPromotionBlockedReason],
) -> bool {
    !blocked_reasons.iter().any(|reason| {
        matches!(
            reason,
            ManagedImportPromotionBlockedReason::ArtifactNotReady
                | ManagedImportPromotionBlockedReason::MissingArtifactUri
                | ManagedImportPromotionBlockedReason::MissingDestinationLocator
                | ManagedImportPromotionBlockedReason::InvalidArtifactUri
                | ManagedImportPromotionBlockedReason::InvalidDestinationLocator
                | ManagedImportPromotionBlockedReason::DestinationEscapesLibrary
        )
    })
}

fn locator_is_within_library(library: &Library, locator: &str) -> bool {
    if locator_has_parent_components(locator) {
        return false;
    }

    library
        .roots
        .iter()
        .any(|root| locator_is_within_root(root, locator))
}

fn locator_is_within_root(root: &str, locator: &str) -> bool {
    let Ok(root_uri) = StorageUri::parse(root) else {
        return false;
    };
    let Ok(locator_uri) = StorageUri::parse(locator) else {
        return false;
    };
    if root_uri.scheme() != locator_uri.scheme() {
        return false;
    }
    if root.ends_with(":///") {
        return true;
    }

    let root = root.trim_end_matches('/');
    locator == root || locator.starts_with(&format!("{root}/"))
}

fn locator_has_parent_components(locator: &str) -> bool {
    StorageUri::parse(locator).ok().is_some_and(|uri| {
        let normalized = uri
            .path_part()
            .trim_start_matches(['/', '\\'])
            .replace('\\', "/");
        Path::new(&normalized)
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    })
}

fn push_blocked_once(
    blocked_reasons: &mut Vec<ManagedImportPromotionBlockedReason>,
    reason: ManagedImportPromotionBlockedReason,
) {
    if !blocked_reasons.contains(&reason) {
        blocked_reasons.push(reason);
    }
}

fn static_file_operation(
    kind: ManagedImportPromotionOperationKind,
    source_scheme: Option<String>,
    target_locator: Option<String>,
    ready: bool,
    message: impl Into<String>,
) -> ManagedImportPromotionFileOperation {
    ManagedImportPromotionFileOperation {
        kind,
        status: if ready {
            ManagedImportPromotionOperationStatus::Ready
        } else {
            ManagedImportPromotionOperationStatus::Blocked
        },
        can_apply: ready,
        source_scheme,
        target_locator,
        message: message.into(),
    }
}

fn blocked_file_operation(
    kind: ManagedImportPromotionOperationKind,
    source_scheme: Option<String>,
    target_locator: Option<String>,
    message: impl Into<String>,
) -> ManagedImportPromotionFileOperation {
    ManagedImportPromotionFileOperation {
        kind,
        status: ManagedImportPromotionOperationStatus::Blocked,
        can_apply: false,
        source_scheme,
        target_locator,
        message: message.into(),
    }
}

fn unsupported_file_operation(
    kind: ManagedImportPromotionOperationKind,
    source_scheme: Option<String>,
    target_locator: Option<String>,
    message: impl Into<String>,
) -> ManagedImportPromotionFileOperation {
    ManagedImportPromotionFileOperation {
        kind,
        status: ManagedImportPromotionOperationStatus::Unsupported,
        can_apply: false,
        source_scheme,
        target_locator,
        message: message.into(),
    }
}

async fn link_file_operation(
    backend: &dyn StorageBackend,
    source_uri: StorageUri,
    target_uri: StorageUri,
    link_kind: StorageLinkKind,
    operation_kind: ManagedImportPromotionOperationKind,
) -> ManagedImportPromotionFileOperation {
    let source_scheme = Some(source_uri.scheme().to_owned());
    let target_locator = Some(target_uri.to_string());

    match backend
        .plan_link(StorageLinkPlanRequest::new(
            source_uri, target_uri, link_kind,
        ))
        .await
    {
        Ok(plan) => ManagedImportPromotionFileOperation {
            kind: operation_kind,
            status: link_plan_status(plan.status),
            can_apply: plan.can_apply,
            source_scheme,
            target_locator,
            message: plan.message,
        },
        Err(err) => ManagedImportPromotionFileOperation {
            kind: operation_kind,
            status: ManagedImportPromotionOperationStatus::Failed,
            can_apply: false,
            source_scheme,
            target_locator,
            message: safe_storage_planning_message(&err),
        },
    }
}

fn link_plan_status(
    status: taru_vfs::StorageLinkPlanStatus,
) -> ManagedImportPromotionOperationStatus {
    match status {
        taru_vfs::StorageLinkPlanStatus::Ready => ManagedImportPromotionOperationStatus::Ready,
        taru_vfs::StorageLinkPlanStatus::Unsupported => {
            ManagedImportPromotionOperationStatus::Unsupported
        }
        taru_vfs::StorageLinkPlanStatus::SourceMissing => {
            ManagedImportPromotionOperationStatus::SourceMissing
        }
        taru_vfs::StorageLinkPlanStatus::SourceNotFile => {
            ManagedImportPromotionOperationStatus::SourceNotFile
        }
        taru_vfs::StorageLinkPlanStatus::TargetParentMissing => {
            ManagedImportPromotionOperationStatus::TargetParentMissing
        }
        taru_vfs::StorageLinkPlanStatus::TargetParentNotDirectory => {
            ManagedImportPromotionOperationStatus::TargetParentNotDirectory
        }
        taru_vfs::StorageLinkPlanStatus::TargetExists => {
            ManagedImportPromotionOperationStatus::TargetExists
        }
        taru_vfs::StorageLinkPlanStatus::SecurityViolation => {
            ManagedImportPromotionOperationStatus::SecurityViolation
        }
    }
}

fn duplicate_hint_for_source(
    artifact: &ManagedImportArtifactRecord,
    source: &MediaSource,
    fingerprint: &str,
) -> Option<ManagedImportPromotionDuplicateHint> {
    let fingerprint_matches = source.fingerprint.as_deref() == Some(fingerprint);
    let size_matches = artifact.size_bytes.is_some() && artifact.size_bytes == source.size_bytes;
    if !fingerprint_matches && !size_matches {
        return None;
    }

    let evidence_kind = if fingerprint_matches {
        SourceDuplicateEvidenceKind::StrongFingerprint
    } else {
        SourceDuplicateEvidenceKind::PathEvidence
    };
    let confidence_milli = if fingerprint_matches {
        Some(950)
    } else {
        Some(550)
    };

    Some(ManagedImportPromotionDuplicateHint {
        existing_source_id: Some(source.id),
        evidence_kind,
        confidence_milli,
        size_matches,
        fingerprint_matches,
        message: "staged artifact resembles an existing Media Source in the target library"
            .to_owned(),
    })
}

fn nfo_sidecar_locator(locator: &str) -> Option<String> {
    let (stem, _extension) = locator.rsplit_once('.')?;
    Some(format!("{stem}.nfo"))
}

fn nfo_authority_message(
    policy: LocalMetadataPolicy,
    has_sidecar: bool,
    has_destination: bool,
) -> String {
    if !has_destination {
        return "NFO authority preview requires a destination locator".to_owned();
    }

    match (policy, has_sidecar) {
        (LocalMetadataPolicy::Disabled, _) => {
            "local metadata policy disables NFO authority".to_owned()
        }
        (LocalMetadataPolicy::WriteSidecar, true) => {
            "NFO export would update an existing sidecar and require backup".to_owned()
        }
        (LocalMetadataPolicy::WriteSidecar, false) => {
            "NFO export would create a sidecar after promotion".to_owned()
        }
        (_, true) => "NFO import could read the destination sidecar after promotion".to_owned(),
        (_, false) => "no destination NFO sidecar exists yet".to_owned(),
    }
}

fn provider_identity_hint(
    library: &Library,
    artifact: &ManagedImportArtifactRecord,
) -> ManagedImportPromotionProviderIdentityHint {
    let configured_providers = library.options.metadata_profile.metadata_providers.clone();
    let has_import_diagnostics = artifact.diagnostics_json.is_some();
    let needs_identity_review = !configured_providers.is_empty() && !has_import_diagnostics;

    ManagedImportPromotionProviderIdentityHint {
        configured_providers,
        has_import_diagnostics,
        needs_identity_review,
        message: if needs_identity_review {
            "provider identity should be reviewed before promotion".to_owned()
        } else if has_import_diagnostics {
            "managed import carries provider/local identity diagnostics".to_owned()
        } else {
            "no provider identity review is required by this library profile".to_owned()
        },
    }
}

fn safe_storage_planning_message(err: &TaruError) -> String {
    match err {
        TaruError::NotFound { .. } => "storage planning target was not found",
        TaruError::Unsupported(_) => "storage backend does not support promotion planning",
        TaruError::InvalidInput { .. } => "storage planning input is invalid",
        TaruError::Storage { .. } => "storage backend could not plan promotion safely",
        _ => "promotion storage planning failed",
    }
    .to_owned()
}

fn join_storage_locator(root: &str, relative: &str) -> String {
    if root.ends_with(":///") {
        format!("{root}{relative}")
    } else {
        format!("{}/{}", root.trim_end_matches('/'), relative)
    }
}
