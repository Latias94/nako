use std::path::{Component, Path};

use serde::Serialize;
use taru_core::{
    Library, LibraryId, LibraryRepository, LocalMetadataPolicy, ManagedImportArtifactId,
    ManagedImportArtifactListFilter, ManagedImportArtifactRecord, ManagedImportArtifactState,
    ManagedImportPromotionApplyId, ManagedImportPromotionApplyRecord,
    ManagedImportPromotionApplyState, ManagedImportPromotionBlockedReason,
    ManagedImportPromotionDuplicateHint, ManagedImportPromotionFileOperation,
    ManagedImportPromotionNfoAuthorityHint, ManagedImportPromotionOperationKind,
    ManagedImportPromotionOperationStatus, ManagedImportPromotionPlan,
    ManagedImportPromotionProviderIdentityHint, ManagedImportRepository, ManagedImportSourceKind,
    MediaRepository, MediaSource, NewManagedImportArtifact, NewManagedImportPromotionApply,
    PageRequest, Result, SourceDuplicateEvidenceKind, StagingManifestId, StagingManifestRepository,
    TaruError, UserPrincipalId,
};
use taru_db::TaruDatabase;
use taru_vfs::{StorageBackend, StorageLinkKind, StorageLinkPlanRequest, StorageUri};

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
