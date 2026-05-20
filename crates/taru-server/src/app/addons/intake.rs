use taru_api::extension::{
    AddonSideEffectResponse, AddonSideEffectSummary, AddonSideEffectTargetRequest,
    SubmitAddonSideEffectRequest,
};
use taru_core::{
    AddonPermission, AddonPrincipal, AddonRepository, AddonSideEffectApplyOutcome,
    AddonSideEffectApplyStatus, AddonSideEffectId, AddonSideEffectTarget,
    AddonSideEffectTargetKind, AddonSideEffectValidationStatus, LibraryId, LibraryItemRepository,
    LibraryRepository, MediaRepository, NewAddonSideEffect, Result, TaruError,
};

use super::AddonAppService;

impl AddonAppService {
    pub async fn submit_addon_side_effect(
        &self,
        raw_token: &str,
        request: SubmitAddonSideEffectRequest,
    ) -> Result<AddonSideEffectResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        let idempotency_key = normalize_idempotency_key(&request.idempotency_key)?;

        if let Some(existing) = self
            .store
            .find_addon_side_effect_by_idempotency_key(principal.addon.id, &idempotency_key)
            .await?
        {
            return Ok(AddonSideEffectResponse {
                side_effect: AddonSideEffectSummary::from_record(existing),
                idempotent_replay: true,
            });
        }

        let target = normalize_side_effect_target(request.target)?;
        let provenance_json =
            serde_json::to_string(&request.provenance).map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize addon side effect provenance: {err}"),
            })?;
        let payload_json =
            serde_json::to_string(&request.payload).map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize addon side effect payload: {err}"),
            })?;
        let validation_error = self
            .validate_side_effect_authority_and_target(
                &principal,
                request.permission,
                request.library_id,
                &target,
            )
            .await
            .err();
        let validation_status = if validation_error.is_some() {
            AddonSideEffectValidationStatus::Rejected
        } else {
            AddonSideEffectValidationStatus::Accepted
        };
        let safe_error_code = validation_error
            .as_ref()
            .map(side_effect_safe_error_code)
            .map(str::to_owned);

        let side_effect = self
            .store
            .create_addon_side_effect(NewAddonSideEffect {
                id: AddonSideEffectId::new(),
                addon_id: principal.addon.id,
                token_id: principal.token.id,
                permission: request.permission,
                library_id: request.library_id,
                target,
                idempotency_key,
                provenance_json,
                payload_json,
                validation_status,
                safe_error_code: safe_error_code.clone(),
            })
            .await?;

        if let Some(error) = validation_error {
            self.store
                .set_addon_side_effect_apply_outcome(
                    side_effect.id,
                    AddonSideEffectApplyOutcome {
                        status: AddonSideEffectApplyStatus::Skipped,
                        error_code: safe_error_code,
                        item_id: None,
                        source: None,
                        report_json: None,
                    },
                )
                .await?;
            return Err(error);
        }

        let side_effect = self.apply_addon_side_effect(side_effect).await?;

        Ok(AddonSideEffectResponse {
            side_effect: AddonSideEffectSummary::from_record(side_effect),
            idempotent_replay: false,
        })
    }

    async fn validate_side_effect_authority_and_target(
        &self,
        principal: &AddonPrincipal,
        permission: AddonPermission,
        library_id: LibraryId,
        target: &AddonSideEffectTarget,
    ) -> Result<()> {
        self.authorize_addon_principal(principal, permission, Some(library_id))?;
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| TaruError::InvalidInput {
                message: format!("addon side effect library {library_id} is missing"),
            })?;

        match target.kind {
            AddonSideEffectTargetKind::MediaItem => {
                if matches!(permission, AddonPermission::LibraryFileWrite) {
                    return Err(TaruError::InvalidInput {
                        message:
                            "addon library_file_write side effects require a media_source target"
                                .to_owned(),
                    });
                }
                let item_id = target.id.parse().map_err(|err| TaruError::InvalidInput {
                    message: format!("invalid addon side effect media item target id: {err}"),
                })?;
                self.store
                    .get_library_item_state(library_id, item_id)
                    .await?
                    .ok_or_else(|| TaruError::InvalidInput {
                        message: format!(
                            "addon side effect target media item {item_id} is not in library {library_id}"
                        ),
                    })?;
            }
            AddonSideEffectTargetKind::MediaSource => {
                if matches!(permission, AddonPermission::ArtworkWrite) {
                    return Err(TaruError::InvalidInput {
                        message: "addon artwork_write side effects require a media_item target"
                            .to_owned(),
                    });
                }
                let source_id = target.id.parse().map_err(|err| TaruError::InvalidInput {
                    message: format!("invalid addon side effect media source target id: {err}"),
                })?;
                let source = self
                    .store
                    .get_media_source(source_id)
                    .await?
                    .ok_or_else(|| TaruError::InvalidInput {
                        message: format!(
                            "addon side effect target media source {source_id} is missing"
                        ),
                    })?;
                if source.library_id != library_id {
                    return Err(TaruError::InvalidInput {
                        message: format!(
                            "addon side effect target media source {source_id} is not in library {library_id}"
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

fn normalize_idempotency_key(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaruError::InvalidInput {
            message: "addon side effect idempotency key must not be empty".to_owned(),
        });
    }
    if value.len() > 200 {
        return Err(TaruError::InvalidInput {
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
        return Err(TaruError::InvalidInput {
            message: "addon side effect target id must not be empty".to_owned(),
        });
    }
    if id.len() > 200 {
        return Err(TaruError::InvalidInput {
            message: "addon side effect target id must be at most 200 bytes".to_owned(),
        });
    }

    Ok(AddonSideEffectTarget {
        kind: target.kind,
        id: id.to_owned(),
    })
}

fn side_effect_safe_error_code(error: &TaruError) -> &'static str {
    match error {
        TaruError::Forbidden { .. } => "forbidden",
        TaruError::InvalidInput { .. } => "invalid_target",
        TaruError::NotFound { .. } => "not_found",
        TaruError::Unauthorized { .. } => "unauthorized",
        TaruError::Conflict { .. } => "conflict",
        TaruError::Unsupported(_) => "unsupported",
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
    }
}
