use std::sync::Arc;

use nako_api::extension::{
    AddonSideEffectResponse, AddonSideEffectSummary, AddonSideEffectTargetRequest,
    SubmitAddonSideEffectRequest,
};
use nako_core::{
    AddonPermission, AddonPrincipal, AddonRepository, AddonSideEffectId, AddonSideEffectRecord,
    AddonSideEffectRequestFingerprint, AddonSideEffectTarget, AddonSideEffectTargetKind,
    AddonSideEffectValidationStatus, LibraryId, LibraryItemRepository, LibraryRepository,
    MediaRepository, NakoError, NewAddonSideEffect, Result,
};
use nako_db::NakoDatabase;
use tokio::sync::Semaphore;

use super::{
    principal::{authorize_addon_principal_grant, resolve_addon_principal_from_store},
    side_effect_apply::AddonSideEffectApplyRouter,
};
use crate::app::storage::StorageBackendRegistry;

#[derive(Clone, Debug)]
pub(super) struct AddonSideEffectRuntime {
    store: NakoDatabase,
    apply_router: AddonSideEffectApplyRouter,
}

impl AddonSideEffectRuntime {
    pub(super) fn new(
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            apply_router: AddonSideEffectApplyRouter::new(store.clone(), permits, storage_backends),
            store,
        }
    }

    pub(super) async fn submit(
        &self,
        raw_token: &str,
        request: SubmitAddonSideEffectRequest,
    ) -> Result<AddonSideEffectResponse> {
        let principal = self.resolve_principal(raw_token).await?;
        let idempotency_key = normalize_idempotency_key(&request.idempotency_key)?;

        let existing = self
            .store
            .find_addon_side_effect_by_idempotency_key(principal.addon.id, &idempotency_key)
            .await?;
        let request = AddonSideEffectRuntimeRequest::normalize(request, idempotency_key)?;
        if let Some(existing) = existing {
            return self.replay_or_conflict(existing, &request);
        }

        let validation = self
            .validate_authority_and_target(
                &principal,
                request.permission,
                request.library_id,
                &request.target,
            )
            .await;
        let side_effect = self
            .journal_request(&principal, request, validation.as_ref().err())
            .await?;

        if let Err(error) = validation {
            self.record_validation_rejection(&side_effect).await?;
            return Err(error);
        }

        let side_effect = self.apply_router.apply(side_effect).await?;

        Ok(AddonSideEffectResponse {
            side_effect: AddonSideEffectSummary::from_record(side_effect),
            idempotent_replay: false,
        })
    }

    fn replay_or_conflict(
        &self,
        existing: AddonSideEffectRecord,
        request: &AddonSideEffectRuntimeRequest,
    ) -> Result<AddonSideEffectResponse> {
        if existing.request_fingerprint == request.request_fingerprint {
            return Ok(AddonSideEffectResponse {
                side_effect: AddonSideEffectSummary::from_record(existing),
                idempotent_replay: true,
            });
        }

        Err(NakoError::Conflict {
            message: format!(
                "addon side effect idempotency key {} was already used for a different request",
                existing.idempotency_key
            ),
        })
    }

    async fn resolve_principal(&self, raw_token: &str) -> Result<AddonPrincipal> {
        resolve_addon_principal_from_store(&self.store, raw_token).await
    }

    async fn validate_authority_and_target(
        &self,
        principal: &AddonPrincipal,
        permission: AddonPermission,
        library_id: LibraryId,
        target: &AddonSideEffectTarget,
    ) -> Result<()> {
        self.authorize_principal(principal, permission, Some(library_id))?;
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| NakoError::InvalidInput {
                message: format!("addon side effect library {library_id} is missing"),
            })?;

        match target.kind {
            AddonSideEffectTargetKind::MediaItem => {
                if matches!(permission, AddonPermission::LibraryFileWrite) {
                    return Err(NakoError::InvalidInput {
                        message:
                            "addon library_file_write side effects require a media_source target"
                                .to_owned(),
                    });
                }
                let item_id = target.id.parse().map_err(|err| NakoError::InvalidInput {
                    message: format!("invalid addon side effect media item target id: {err}"),
                })?;
                self.store
                    .get_library_item_state(library_id, item_id)
                    .await?
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: format!(
                            "addon side effect target media item {item_id} is not in library {library_id}"
                        ),
                    })?;
            }
            AddonSideEffectTargetKind::MediaSource => {
                if matches!(permission, AddonPermission::ArtworkWrite) {
                    return Err(NakoError::InvalidInput {
                        message: "addon artwork_write side effects require a media_item target"
                            .to_owned(),
                    });
                }
                let source_id = target.id.parse().map_err(|err| NakoError::InvalidInput {
                    message: format!("invalid addon side effect media source target id: {err}"),
                })?;
                let source = self
                    .store
                    .get_media_source(source_id)
                    .await?
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: format!(
                            "addon side effect target media source {source_id} is missing"
                        ),
                    })?;
                if source.library_id != library_id {
                    return Err(NakoError::InvalidInput {
                        message: format!(
                            "addon side effect target media source {source_id} is not in library {library_id}"
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    fn authorize_principal(
        &self,
        principal: &AddonPrincipal,
        permission: AddonPermission,
        library_id: Option<LibraryId>,
    ) -> Result<()> {
        authorize_addon_principal_grant(principal, permission, library_id)
    }

    async fn journal_request(
        &self,
        principal: &AddonPrincipal,
        request: AddonSideEffectRuntimeRequest,
        validation_error: Option<&NakoError>,
    ) -> Result<AddonSideEffectRecord> {
        let validation_status = if validation_error.is_some() {
            AddonSideEffectValidationStatus::Rejected
        } else {
            AddonSideEffectValidationStatus::Accepted
        };
        let safe_error_code = validation_error
            .map(side_effect_validation_error_code)
            .map(str::to_owned);

        self.store
            .create_addon_side_effect(NewAddonSideEffect {
                id: AddonSideEffectId::new(),
                addon_id: principal.addon.id,
                token_id: principal.token.id,
                permission: request.permission,
                library_id: request.library_id,
                target: request.target,
                idempotency_key: request.idempotency_key,
                provenance_json: request.provenance_json,
                payload_json: request.payload_json,
                validation_status,
                safe_error_code,
            })
            .await
    }

    async fn record_validation_rejection(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonSideEffectRecord> {
        self.apply_router
            .record_validation_rejection(side_effect)
            .await
    }
}

#[derive(Debug)]
struct AddonSideEffectRuntimeRequest {
    permission: AddonPermission,
    library_id: LibraryId,
    target: AddonSideEffectTarget,
    idempotency_key: String,
    request_fingerprint: AddonSideEffectRequestFingerprint,
    provenance_json: String,
    payload_json: String,
}

impl AddonSideEffectRuntimeRequest {
    fn normalize(request: SubmitAddonSideEffectRequest, idempotency_key: String) -> Result<Self> {
        let target = normalize_side_effect_target(request.target)?;
        let provenance_json = serialize_side_effect_json("provenance", &request.provenance)?;
        let payload_json = serialize_side_effect_json("payload", &request.payload)?;
        let request_fingerprint = AddonSideEffectRequestFingerprint::new(
            request.permission,
            request.library_id,
            &target,
            &provenance_json,
            &payload_json,
        );

        Ok(Self {
            permission: request.permission,
            library_id: request.library_id,
            target,
            idempotency_key,
            request_fingerprint,
            provenance_json,
            payload_json,
        })
    }
}

fn normalize_idempotency_key(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "addon side effect idempotency key must not be empty".to_owned(),
        });
    }
    if value.len() > 200 {
        return Err(NakoError::InvalidInput {
            message: "addon side effect idempotency key must be at most 200 bytes".to_owned(),
        });
    }

    Ok(value.to_owned())
}

fn normalize_side_effect_target(
    target: AddonSideEffectTargetRequest,
) -> Result<AddonSideEffectTarget> {
    let id = target.id.trim();
    if id.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "addon side effect target id must not be empty".to_owned(),
        });
    }
    if id.len() > 200 {
        return Err(NakoError::InvalidInput {
            message: "addon side effect target id must be at most 200 bytes".to_owned(),
        });
    }

    Ok(AddonSideEffectTarget {
        kind: target.kind,
        id: id.to_owned(),
    })
}

fn serialize_side_effect_json(field: &str, value: &serde_json::Value) -> Result<String> {
    serde_json::to_string(value).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize addon side effect {field}: {err}"),
    })
}

fn side_effect_validation_error_code(error: &NakoError) -> &'static str {
    match error {
        NakoError::Forbidden { .. } => "forbidden",
        NakoError::InvalidInput { .. } => "invalid_target",
        NakoError::NotFound { .. } => "not_found",
        NakoError::Unauthorized { .. } => "unauthorized",
        NakoError::Conflict { .. } => "conflict",
        NakoError::Unsupported(_) => "unsupported",
        NakoError::Provider { .. } => "provider_error",
        NakoError::Storage { .. } => "storage_error",
        NakoError::Database { .. } => "database_error",
    }
}
