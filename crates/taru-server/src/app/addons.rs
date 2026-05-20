use std::{collections::HashSet, sync::Arc};

use taru_addon_protocol::{ensure_scope_grant, validate_manifest};
use taru_api::extension::{
    AddonAccessCheckRequest, AddonAccessCheckResponse, AddonGrantAssignment, AddonGrantsResponse,
    AddonRegistrationResponse, AddonRegistrationsResponse, AddonSideEffectResponse,
    AddonSideEffectSummary, AddonSideEffectTargetRequest, AddonTokenIssuedResponse,
    AddonTokenResponse, AddonTokenRotationResponse, AddonTokenSummary, AddonTokensResponse,
    IssueAddonTokenRequest, RegisterAddonRequest, ReplaceAddonGrantsRequest,
    SubmitAddonSideEffectRequest,
};
use taru_catalog::{
    CatalogLabelHydrationSelection, hydrate_item_catalog_labels, refresh_item_search,
};
use taru_core::{
    AddonGrantId, AddonId, AddonIssuedToken, AddonPermission, AddonPrincipal,
    AddonRegistrationRecord, AddonRepository, AddonSideEffectApplyOutcome,
    AddonSideEffectApplyStatus, AddonSideEffectId, AddonSideEffectRecord, AddonSideEffectTarget,
    AddonSideEffectTargetKind, AddonSideEffectValidationStatus, AddonStatus, AddonTokenId,
    AddonTokenStatus, ArtworkCandidateId, ArtworkCandidateRepository, ArtworkCandidateSourceKind,
    CanonicalMetadata, ImageKind, LibraryId, LibraryItemRepository, LibraryRepository, MediaItem,
    MediaItemId, MediaRepository, MediaSourceId, MetadataMergePolicy, MetadataRefreshMode,
    MetadataRepository, MetadataSource, NewAddonGrant, NewAddonRegistration, NewAddonSideEffect,
    NewAddonToken, NewArtworkCandidate, Result, StorageErrorKind, TaruError, hash_addon_token,
};
use taru_db::TaruDatabase;
use taru_nfo::{MovieNfoCodec, NfoExportSourceRequest, NfoExportSourceSummary, NfoFailureKind};
use tokio::sync::Semaphore;

use super::{nfo::ensure_nfo_export_writable, storage::StorageBackendRegistry};

#[derive(Clone, Debug)]
pub(crate) struct AddonAppService {
    store: TaruDatabase,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
}

impl AddonAppService {
    pub(super) fn new(
        store: TaruDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            store,
            permits,
            storage_backends,
        }
    }

    fn normalize_addon_registration(
        &self,
        request: RegisterAddonRequest,
    ) -> Result<NewAddonRegistration> {
        validate_manifest(&request.manifest).map_err(|err| TaruError::InvalidInput {
            message: err.to_string(),
        })?;

        let mut seen = HashSet::new();
        let granted_scopes = request
            .granted_scopes
            .into_iter()
            .filter(|scope| seen.insert(*scope))
            .collect::<Vec<_>>();

        for resource in &request.manifest.resources {
            ensure_scope_grant(&request.manifest, resource.kind, &granted_scopes).map_err(
                |err| TaruError::InvalidInput {
                    message: err.to_string(),
                },
            )?;
        }

        let manifest_json =
            serde_json::to_string(&request.manifest).map_err(|err| TaruError::InvalidInput {
                message: format!("failed to serialize addon manifest: {err}"),
            })?;
        let granted_scopes = granted_scopes
            .into_iter()
            .map(|scope| scope.as_str().to_owned())
            .collect();

        Ok(NewAddonRegistration {
            id: request.id.unwrap_or_else(AddonId::new),
            manifest_id: request.manifest.id,
            name: request.manifest.name,
            version: request.manifest.version,
            protocol_version: request.manifest.protocol_version,
            base_url: request.manifest.base_url,
            manifest_json,
            granted_scopes,
            status: request.status.unwrap_or(AddonStatus::Disabled),
        })
    }

    pub async fn register_addon(
        &self,
        request: RegisterAddonRequest,
    ) -> Result<AddonRegistrationResponse> {
        let addon = self.normalize_addon_registration(request)?;
        let addon = self.store.upsert_addon_registration(addon).await?;

        Ok(AddonRegistrationResponse { addon })
    }

    pub async fn get_addon_registration(
        &self,
        addon_id: AddonId,
    ) -> Result<AddonRegistrationResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;

        Ok(AddonRegistrationResponse { addon })
    }

    pub async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<AddonRegistrationsResponse> {
        let addons = self.store.list_addon_registrations(status).await?;

        Ok(AddonRegistrationsResponse { addons })
    }

    pub async fn issue_addon_token(
        &self,
        addon_id: AddonId,
        request: IssueAddonTokenRequest,
    ) -> Result<AddonTokenIssuedResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let issued = AddonIssuedToken::generate();
        let token = self
            .store
            .create_addon_token(NewAddonToken {
                id: AddonTokenId::new(),
                addon_id,
                label: normalize_token_label(request.label.as_deref())?,
                token_prefix: issued.token_prefix.clone(),
                token_hash: issued.token_hash.clone(),
            })
            .await?;

        Ok(AddonTokenIssuedResponse {
            token: AddonTokenSummary::from_record(token),
            raw_token: issued.raw_token.expose_secret().to_owned(),
        })
    }

    pub async fn list_addon_tokens(&self, addon_id: AddonId) -> Result<AddonTokensResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let tokens = self
            .store
            .list_addon_tokens(addon_id)
            .await?
            .into_iter()
            .map(AddonTokenSummary::from_record)
            .collect();

        Ok(AddonTokensResponse { tokens })
    }

    pub async fn rotate_addon_token(
        &self,
        addon_id: AddonId,
        token_id: AddonTokenId,
        request: IssueAddonTokenRequest,
    ) -> Result<AddonTokenRotationResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let existing =
            self.store
                .get_addon_token(token_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "addon_token",
                    id: token_id.to_string(),
                })?;
        if existing.addon_id != addon_id {
            return Err(TaruError::NotFound {
                entity: "addon_token",
                id: token_id.to_string(),
            });
        }

        let issued = AddonIssuedToken::generate();
        let (rotated, token) = self
            .store
            .rotate_addon_token(
                token_id,
                NewAddonToken {
                    id: AddonTokenId::new(),
                    addon_id,
                    label: normalize_token_label(request.label.as_deref())?,
                    token_prefix: issued.token_prefix.clone(),
                    token_hash: issued.token_hash.clone(),
                },
            )
            .await?;

        Ok(AddonTokenRotationResponse {
            rotated: AddonTokenSummary::from_record(rotated),
            token: AddonTokenSummary::from_record(token),
            raw_token: issued.raw_token.expose_secret().to_owned(),
        })
    }

    pub async fn revoke_addon_token(
        &self,
        addon_id: AddonId,
        token_id: AddonTokenId,
    ) -> Result<AddonTokenResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let existing =
            self.store
                .get_addon_token(token_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "addon_token",
                    id: token_id.to_string(),
                })?;
        if existing.addon_id != addon_id {
            return Err(TaruError::NotFound {
                entity: "addon_token",
                id: token_id.to_string(),
            });
        }

        let token = self
            .store
            .revoke_addon_token(token_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "addon_token",
                id: token_id.to_string(),
            })?;

        Ok(AddonTokenResponse {
            token: AddonTokenSummary::from_record(token),
        })
    }

    pub async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        request: ReplaceAddonGrantsRequest,
    ) -> Result<AddonGrantsResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let grants = normalize_grants(addon_id, request.grants)?;
        let grants = self.store.replace_addon_grants(addon_id, grants).await?;

        Ok(AddonGrantsResponse { grants })
    }

    pub async fn list_addon_grants(&self, addon_id: AddonId) -> Result<AddonGrantsResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let grants = self.store.list_addon_grants(addon_id).await?;

        Ok(AddonGrantsResponse { grants })
    }

    pub async fn check_addon_access(
        &self,
        raw_token: &str,
        request: AddonAccessCheckRequest,
    ) -> Result<AddonAccessCheckResponse> {
        let principal = self.resolve_addon_principal(raw_token).await?;
        self.authorize_addon_principal(&principal, request.permission, request.library_id)?;

        Ok(AddonAccessCheckResponse {
            addon_id: principal.addon.id,
            token_id: principal.token.id,
            permission: request.permission,
            library_id: request.library_id,
            allowed: true,
        })
    }

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

    pub async fn resolve_addon_principal(&self, raw_token: &str) -> Result<AddonPrincipal> {
        let raw_token = raw_token.trim();
        if raw_token.is_empty() {
            return Err(TaruError::Unauthorized {
                message: "addon token is required".to_owned(),
            });
        }

        let token_hash = hash_addon_token(raw_token);
        let token = self
            .store
            .find_addon_token_by_hash(&token_hash)
            .await?
            .ok_or_else(|| TaruError::Unauthorized {
                message: "addon token is invalid".to_owned(),
            })?;

        if token.status != AddonTokenStatus::Active {
            return Err(TaruError::Unauthorized {
                message: "addon token is not active".to_owned(),
            });
        }

        let token = self
            .store
            .mark_addon_token_used(token.id)
            .await?
            .ok_or_else(|| TaruError::Unauthorized {
                message: "addon token is not active".to_owned(),
            })?;

        let addon = self
            .store
            .get_addon_registration(token.addon_id)
            .await?
            .ok_or_else(|| TaruError::Unauthorized {
                message: "addon registration is missing".to_owned(),
            })?;

        if addon.status != AddonStatus::Enabled {
            return Err(TaruError::Forbidden {
                message: "addon registration is disabled".to_owned(),
            });
        }

        let grants = self.store.list_addon_grants(addon.id).await?;

        Ok(AddonPrincipal {
            addon,
            token,
            grants,
        })
    }

    pub fn authorize_addon_principal(
        &self,
        principal: &AddonPrincipal,
        permission: AddonPermission,
        library_id: Option<LibraryId>,
    ) -> Result<()> {
        if principal.allows(permission, library_id) {
            return Ok(());
        }

        Err(TaruError::Forbidden {
            message: match library_id {
                Some(library_id) => format!(
                    "addon {} is not granted {} for library {}",
                    principal.addon.id,
                    permission.as_str(),
                    library_id
                ),
                None => format!(
                    "addon {} is not granted {}",
                    principal.addon.id,
                    permission.as_str()
                ),
            },
        })
    }

    async fn get_addon_registration_or_not_found(
        &self,
        addon_id: AddonId,
    ) -> Result<AddonRegistrationRecord> {
        self.store
            .get_addon_registration(addon_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "addon_registration",
                id: addon_id.to_string(),
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

    async fn apply_addon_side_effect(
        &self,
        side_effect: AddonSideEffectRecord,
    ) -> Result<AddonSideEffectRecord> {
        if side_effect.validation_status != AddonSideEffectValidationStatus::Accepted {
            return self
                .store
                .set_addon_side_effect_apply_outcome(
                    side_effect.id,
                    AddonSideEffectApplyOutcome {
                        status: AddonSideEffectApplyStatus::Skipped,
                        error_code: side_effect.safe_error_code.clone(),
                        item_id: None,
                        source: None,
                        report_json: None,
                    },
                )
                .await;
        }

        match side_effect.permission {
            AddonPermission::MetadataWrite => {
                match self.apply_metadata_write_side_effect(&side_effect).await {
                    Ok(item_id) => {
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Applied,
                                    error_code: None,
                                    item_id: Some(item_id),
                                    source: Some(addon_metadata_source_label(side_effect.addon_id)),
                                    report_json: None,
                                },
                            )
                            .await
                    }
                    Err(error) => {
                        let error_code = side_effect_apply_error_code(&error).to_owned();
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Failed,
                                    error_code: Some(error_code),
                                    item_id: None,
                                    source: None,
                                    report_json: None,
                                },
                            )
                            .await?;
                        Err(error)
                    }
                }
            }
            AddonPermission::LibraryFileWrite => {
                match self
                    .apply_library_file_write_side_effect(&side_effect)
                    .await
                {
                    Ok(applied) => {
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Applied,
                                    error_code: None,
                                    item_id: Some(applied.item_id),
                                    source: Some(applied.source),
                                    report_json: Some(applied.report_json),
                                },
                            )
                            .await
                    }
                    Err(error) => {
                        let error_code = side_effect_apply_error_code(&error).to_owned();
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Failed,
                                    error_code: Some(error_code),
                                    item_id: None,
                                    source: None,
                                    report_json: None,
                                },
                            )
                            .await?;
                        Err(error)
                    }
                }
            }
            AddonPermission::ArtworkWrite => {
                match self.apply_artwork_write_side_effect(&side_effect).await {
                    Ok(applied) => {
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Applied,
                                    error_code: None,
                                    item_id: Some(applied.item_id),
                                    source: Some(applied.source),
                                    report_json: Some(applied.report_json),
                                },
                            )
                            .await
                    }
                    Err(error) => {
                        let error_code = side_effect_apply_error_code(&error).to_owned();
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Failed,
                                    error_code: Some(error_code),
                                    item_id: None,
                                    source: None,
                                    report_json: None,
                                },
                            )
                            .await?;
                        Err(error)
                    }
                }
            }
            _ => {
                self.store
                    .set_addon_side_effect_apply_outcome(
                        side_effect.id,
                        AddonSideEffectApplyOutcome {
                            status: AddonSideEffectApplyStatus::Skipped,
                            error_code: Some("unsupported".to_owned()),
                            item_id: None,
                            source: None,
                            report_json: None,
                        },
                    )
                    .await
            }
        }
    }

    async fn apply_metadata_write_side_effect(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<MediaItemId> {
        let existing = self.resolve_side_effect_media_item(side_effect).await?;
        let patch = parse_addon_metadata_patch(&side_effect.payload_json)?;
        let label_selection = patch.catalog_label_selection();
        let incoming = patch.apply_to(existing.metadata.clone());
        let source = MetadataSource::Addon(side_effect.addon_id);
        let locks = self.store.list_field_locks(existing.id).await?;
        let policy = MetadataMergePolicy::for_source_refresh_mode(
            &locks,
            &source,
            MetadataRefreshMode::FullRefresh,
        );
        let merged = policy.merge(&existing.metadata, &incoming);
        let updated = MediaItem {
            metadata: merged,
            ..existing
        };

        self.store.commit_metadata_item(&updated).await?;
        if label_selection.any() {
            hydrate_item_catalog_labels(&self.store, updated.id, source, label_selection).await?;
        } else {
            refresh_item_search(&self.store, updated.id).await?;
        }

        Ok(updated.id)
    }

    async fn apply_library_file_write_side_effect(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AppliedLibraryFileWrite> {
        let payload = parse_addon_library_file_write_payload(&side_effect.payload_json)?;
        match payload.file_role {
            AddonLibraryFileRole::Nfo => {}
        }
        if side_effect.target.kind != AddonSideEffectTargetKind::MediaSource {
            return Err(TaruError::InvalidInput {
                message: "addon library_file_write NFO export requires a media_source target"
                    .to_owned(),
            });
        }
        let source_id: MediaSourceId =
            side_effect
                .target
                .id
                .parse()
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("invalid addon library_file_write media source target: {err}"),
                })?;
        let source = self
            .store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })?;
        if source.library_id != side_effect.library_id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "addon library_file_write target media source {} is not in library {}",
                    source.id, side_effect.library_id
                ),
            });
        }

        let library = self
            .store
            .get_library(side_effect.library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: side_effect.library_id.to_string(),
            })?;
        let _permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("NFO export limiter is unavailable: {err}"),
                })?;
        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        ensure_nfo_export_writable(backend.as_ref(), &library).await?;
        let service = taru_nfo::NfoService::new(backend, self.store.clone(), MovieNfoCodec);
        let summary = service
            .export_media_source(NfoExportSourceRequest {
                library_id: side_effect.library_id,
                source_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: payload.policy.force(),
            })
            .await?;

        if let Some(failure) = summary.failures.first() {
            return Err(nfo_export_failure_error(failure.kind));
        }

        Ok(AppliedLibraryFileWrite {
            item_id: source.item_id,
            source: "nfo_export".to_owned(),
            report_json: nfo_export_apply_report(payload.policy, &summary)?,
        })
    }

    async fn apply_artwork_write_side_effect(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AppliedArtworkCandidate> {
        let payload = parse_addon_artwork_write_payload(&side_effect.payload_json)?;
        if side_effect.target.kind != AddonSideEffectTargetKind::MediaItem {
            return Err(TaruError::InvalidInput {
                message: "addon artwork_write candidate proposal requires a media_item target"
                    .to_owned(),
            });
        }
        let item = self.resolve_side_effect_media_item(side_effect).await?;
        let source_uri = normalize_remote_artwork_url(&payload.source.url)?;
        let language = normalize_artwork_language(payload.language.as_deref())?;
        let kind = payload.kind.into_image_kind();

        let existing = self
            .store
            .find_artwork_candidate_by_source(
                side_effect.addon_id,
                side_effect.library_id,
                item.id,
                &kind,
                ArtworkCandidateSourceKind::RemoteUrl,
                &source_uri,
            )
            .await?;
        let (candidate_id, created) = if let Some(existing) = existing {
            (existing.id, false)
        } else {
            let candidate = self
                .store
                .create_artwork_candidate(NewArtworkCandidate {
                    id: ArtworkCandidateId::new(),
                    addon_id: side_effect.addon_id,
                    side_effect_id: side_effect.id,
                    library_id: side_effect.library_id,
                    item_id: item.id,
                    kind: kind.clone(),
                    source_kind: ArtworkCandidateSourceKind::RemoteUrl,
                    source_uri,
                    width: payload.width,
                    height: payload.height,
                    language,
                })
                .await?;
            (candidate.id, true)
        };

        Ok(AppliedArtworkCandidate {
            item_id: item.id,
            source: "artwork_candidate".to_owned(),
            report_json: artwork_candidate_apply_report(candidate_id, &kind, created)?,
        })
    }

    async fn resolve_side_effect_media_item(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<MediaItem> {
        match side_effect.target.kind {
            AddonSideEffectTargetKind::MediaItem => {
                let item_id =
                    side_effect
                        .target
                        .id
                        .parse()
                        .map_err(|err| TaruError::InvalidInput {
                            message: format!(
                                "invalid addon side effect media item target id: {err}"
                            ),
                        })?;
                self.store
                    .get_media_item(item_id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "media_item",
                        id: item_id.to_string(),
                    })
            }
            AddonSideEffectTargetKind::MediaSource => {
                let source_id =
                    side_effect
                        .target
                        .id
                        .parse()
                        .map_err(|err| TaruError::InvalidInput {
                            message: format!(
                                "invalid addon side effect media source target id: {err}"
                            ),
                        })?;
                let source = self
                    .store
                    .get_media_source(source_id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "media_source",
                        id: source_id.to_string(),
                    })?;
                self.store
                    .get_media_item(source.item_id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "media_item",
                        id: source.item_id.to_string(),
                    })
            }
        }
    }
}

struct AppliedLibraryFileWrite {
    item_id: MediaItemId,
    source: String,
    report_json: String,
}

struct AppliedArtworkCandidate {
    item_id: MediaItemId,
    source: String,
    report_json: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AddonLibraryFileRole {
    Nfo,
}

impl AddonLibraryFileRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Nfo => "nfo",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AddonNfoExportPolicy {
    CreateMissing,
    ReplaceExistingPreserving,
}

impl AddonNfoExportPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CreateMissing => "create_missing",
            Self::ReplaceExistingPreserving => "replace_existing_preserving",
        }
    }

    const fn force(self) -> bool {
        match self {
            Self::CreateMissing => false,
            Self::ReplaceExistingPreserving => true,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AddonLibraryFileWritePayload {
    file_role: AddonLibraryFileRole,
    policy: AddonNfoExportPolicy,
}

fn parse_addon_library_file_write_payload(
    payload_json: &str,
) -> Result<AddonLibraryFileWritePayload> {
    serde_json::from_str::<AddonLibraryFileWritePayload>(payload_json).map_err(|err| {
        TaruError::InvalidInput {
            message: format!("invalid addon library_file_write payload: {err}"),
        }
    })
}

fn nfo_export_apply_report(
    policy: AddonNfoExportPolicy,
    summary: &NfoExportSourceSummary,
) -> Result<String> {
    let report = serde_json::json!({
        "kind": "nfo_export",
        "file_role": AddonLibraryFileRole::Nfo.as_str(),
        "policy": policy.as_str(),
        "scanned_sources": summary.scanned_sources,
        "exported_items": summary.exported_items,
        "skipped_items": summary.skipped_items,
        "failed_items": summary.failed_items,
        "backed_up_items": summary.backed_up_items,
        "pruned_backup_items": summary.pruned_backup_items,
        "pruned_backups": summary.pruned_backups,
        "prune_failures": summary.prune_failures.len(),
    });

    serde_json::to_string(&report).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to serialize addon NFO export report: {err}"),
    })
}

fn nfo_export_failure_error(kind: NfoFailureKind) -> TaruError {
    match kind {
        NfoFailureKind::StorageRead | NfoFailureKind::StorageWrite => TaruError::storage(
            "nfo_export",
            StorageErrorKind::Unknown,
            "NFO export storage operation failed",
        ),
        NfoFailureKind::StorageBackup => TaruError::storage(
            "nfo_export",
            StorageErrorKind::Backup,
            "NFO export backup failed",
        ),
        NfoFailureKind::StorageUnsupported => {
            TaruError::Unsupported("NFO export storage backend is unsupported")
        }
        NfoFailureKind::MissingMediaItem => TaruError::NotFound {
            entity: "media_item",
            id: "nfo_export_target".to_owned(),
        },
        NfoFailureKind::NfoConflict => TaruError::Conflict {
            message: "NFO export preservation conflict".to_owned(),
        },
        NfoFailureKind::InvalidSidecarPath
        | NfoFailureKind::NfoParse
        | NfoFailureKind::NfoPreservation
        | NfoFailureKind::NfoRender
        | NfoFailureKind::Unknown => TaruError::InvalidInput {
            message: format!("NFO export failed: {kind:?}"),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AddonArtworkIntent {
    ProposeArtwork,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AddonArtworkKind {
    Poster,
    Backdrop,
    Logo,
    Banner,
    Thumbnail,
}

impl AddonArtworkKind {
    fn into_image_kind(self) -> ImageKind {
        match self {
            Self::Poster => ImageKind::Poster,
            Self::Backdrop => ImageKind::Backdrop,
            Self::Logo => ImageKind::Logo,
            Self::Banner => ImageKind::Banner,
            Self::Thumbnail => ImageKind::Thumbnail,
        }
    }
}

fn image_kind_report_value(kind: &ImageKind) -> &'static str {
    match kind {
        ImageKind::Poster => "poster",
        ImageKind::Backdrop => "backdrop",
        ImageKind::Logo => "logo",
        ImageKind::Banner => "banner",
        ImageKind::Thumbnail => "thumbnail",
        ImageKind::Other(_) => "other",
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AddonArtworkSourcePayload {
    kind: ArtworkCandidateSourceKind,
    url: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AddonArtworkWritePayload {
    intent: AddonArtworkIntent,
    kind: AddonArtworkKind,
    source: AddonArtworkSourcePayload,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

fn parse_addon_artwork_write_payload(payload_json: &str) -> Result<AddonArtworkWritePayload> {
    let payload = serde_json::from_str::<AddonArtworkWritePayload>(payload_json).map_err(|_| {
        TaruError::InvalidInput {
            message: "invalid addon artwork_write payload".to_owned(),
        }
    })?;
    match payload.intent {
        AddonArtworkIntent::ProposeArtwork => {}
    }
    match payload.source.kind {
        ArtworkCandidateSourceKind::RemoteUrl => {}
    }
    validate_artwork_dimension("width", payload.width)?;
    validate_artwork_dimension("height", payload.height)?;
    Ok(payload)
}

fn validate_artwork_dimension(field: &str, value: Option<u32>) -> Result<()> {
    if matches!(value, Some(0 | 20001..)) {
        return Err(TaruError::InvalidInput {
            message: format!("addon artwork_write {field} must be between 1 and 20000"),
        });
    }
    Ok(())
}

fn normalize_remote_artwork_url(value: &str) -> Result<String> {
    let value = value.trim();
    if value.len() > 2048 {
        return Err(TaruError::InvalidInput {
            message: "addon artwork_write remote URL must be at most 2048 bytes".to_owned(),
        });
    }
    let url = reqwest::Url::parse(value).map_err(|_| TaruError::InvalidInput {
        message: "invalid addon artwork_write remote URL".to_owned(),
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(TaruError::InvalidInput {
            message: "addon artwork_write remote URL must use http or https".to_owned(),
        });
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(TaruError::InvalidInput {
            message: "addon artwork_write remote URL must not contain credentials".to_owned(),
        });
    }
    Ok(url.to_string())
}

fn normalize_artwork_language(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > 32 {
        return Err(TaruError::InvalidInput {
            message: "addon artwork_write language must be at most 32 bytes".to_owned(),
        });
    }
    Ok(Some(value.to_ascii_lowercase()))
}

fn artwork_candidate_apply_report(
    candidate_id: ArtworkCandidateId,
    kind: &ImageKind,
    created: bool,
) -> Result<String> {
    let report = serde_json::json!({
        "kind": "artwork_candidate",
        "candidate_id": candidate_id.to_string(),
        "image_kind": image_kind_report_value(kind),
        "status": "proposed",
        "candidate_created": u8::from(created),
        "candidate_existing": u8::from(!created),
    });

    serde_json::to_string(&report).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to serialize addon artwork candidate report: {err}"),
    })
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AddonMetadataPatch {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    sort_title: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    runtime_minutes: Option<u32>,
    #[serde(default)]
    tagline: Option<String>,
    #[serde(default)]
    genres: Option<Vec<String>>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

impl AddonMetadataPatch {
    fn apply_to(self, mut metadata: CanonicalMetadata) -> CanonicalMetadata {
        if let Some(title) = self.title.and_then(non_empty_trimmed) {
            metadata.title = title;
        }
        if let Some(value) = self.original_title.map(normalize_optional_text) {
            metadata.original_title = value;
        }
        if let Some(value) = self.sort_title.map(normalize_optional_text) {
            metadata.sort_title = value;
        }
        if let Some(value) = self.overview.map(normalize_optional_text) {
            metadata.overview = value;
        }
        if let Some(value) = self.release_date.map(normalize_optional_text) {
            metadata.release_date = value;
        }
        if let Some(runtime_minutes) = self.runtime_minutes {
            metadata.runtime_minutes = (runtime_minutes > 0).then_some(runtime_minutes);
        }
        if let Some(value) = self.tagline.map(normalize_optional_text) {
            metadata.tagline = value;
        }
        if let Some(genres) = self.genres {
            metadata.genres = normalize_label_list(genres);
        }
        if let Some(tags) = self.tags {
            metadata.tags = normalize_label_list(tags);
        }

        metadata
    }
}

fn normalize_token_label(label: Option<&str>) -> Result<String> {
    let label = label.unwrap_or("default").trim();
    if label.is_empty() {
        return Err(TaruError::InvalidInput {
            message: "addon token label must not be empty".to_owned(),
        });
    }

    Ok(label.to_owned())
}

fn parse_addon_metadata_patch(payload_json: &str) -> Result<AddonMetadataPatch> {
    let patch = serde_json::from_str::<AddonMetadataPatch>(payload_json).map_err(|err| {
        TaruError::InvalidInput {
            message: format!("invalid addon metadata_write payload: {err}"),
        }
    })?;

    patch.validate()?;
    Ok(patch)
}

impl AddonMetadataPatch {
    fn validate(&self) -> Result<()> {
        if !self.has_any_field() {
            return Err(TaruError::InvalidInput {
                message:
                    "addon metadata_write payload must include at least one supported metadata field"
                        .to_owned(),
            });
        }

        self.validate_text_field("title", self.title.as_ref())?;
        self.validate_text_field("original_title", self.original_title.as_ref())?;
        self.validate_text_field("sort_title", self.sort_title.as_ref())?;
        self.validate_text_field("overview", self.overview.as_ref())?;
        self.validate_text_field("release_date", self.release_date.as_ref())?;
        self.validate_text_field("tagline", self.tagline.as_ref())?;
        self.validate_list_field("genres", self.genres.as_ref())?;
        self.validate_list_field("tags", self.tags.as_ref())?;

        if self.runtime_minutes == Some(0) {
            return Err(TaruError::InvalidInput {
                message: "addon metadata_write payload runtime_minutes must be greater than zero"
                    .to_owned(),
            });
        }

        Ok(())
    }

    fn has_any_field(&self) -> bool {
        self.title.is_some()
            || self.original_title.is_some()
            || self.sort_title.is_some()
            || self.overview.is_some()
            || self.release_date.is_some()
            || self.runtime_minutes.is_some()
            || self.tagline.is_some()
            || self.genres.is_some()
            || self.tags.is_some()
    }

    fn catalog_label_selection(&self) -> CatalogLabelHydrationSelection {
        CatalogLabelHydrationSelection {
            genres: self.genres.is_some(),
            tags: self.tags.is_some(),
        }
    }

    fn validate_text_field(&self, field: &str, value: Option<&String>) -> Result<()> {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(TaruError::InvalidInput {
                message: format!("addon metadata_write payload {field} must not be empty"),
            });
        }

        Ok(())
    }

    fn validate_list_field(&self, field: &str, value: Option<&Vec<String>>) -> Result<()> {
        if value.is_some_and(|values| values.iter().any(|value| value.trim().is_empty())) {
            return Err(TaruError::InvalidInput {
                message: format!("addon metadata_write payload {field} entries must not be empty"),
            });
        }

        Ok(())
    }
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn normalize_optional_text(value: String) -> Option<String> {
    non_empty_trimmed(value)
}

fn normalize_label_list(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(non_empty_trimmed)
        .filter(|value| seen.insert(value.to_lowercase()))
        .collect()
}

fn addon_metadata_source_label(addon_id: AddonId) -> String {
    format!("addon:{addon_id}")
}

fn normalize_grants(
    addon_id: AddonId,
    grants: Vec<AddonGrantAssignment>,
) -> Result<Vec<NewAddonGrant>> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(grants.len());

    for grant in grants {
        if !seen.insert((grant.permission, grant.library_id)) {
            continue;
        }
        normalized.push(NewAddonGrant {
            id: AddonGrantId::new(),
            addon_id,
            permission: grant.permission,
            library_id: grant.library_id,
        });
    }

    Ok(normalized)
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

fn side_effect_apply_error_code(error: &TaruError) -> &'static str {
    match error {
        TaruError::InvalidInput { .. } => "invalid_payload",
        TaruError::Forbidden { .. } => "forbidden",
        TaruError::NotFound { .. } => "not_found",
        TaruError::Unauthorized { .. } => "unauthorized",
        TaruError::Conflict { .. } => "conflict",
        TaruError::Unsupported(_) => "unsupported",
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
    }
}
