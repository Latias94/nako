use std::collections::HashSet;

use taru_addon_protocol::{ensure_scope_grant, validate_manifest};
use taru_api::{
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
    AddonTokenStatus, CanonicalMetadata, LibraryId, LibraryItemRepository, LibraryRepository,
    MediaItem, MediaItemId, MediaRepository, MetadataMergePolicy, MetadataRefreshMode,
    MetadataRepository, MetadataSource, NewAddonGrant, NewAddonRegistration, NewAddonSideEffect,
    NewAddonToken, Result, TaruError, hash_addon_token,
};
use taru_db::SqliteStore;

#[derive(Clone, Debug)]
pub(crate) struct AddonAppService {
    store: SqliteStore,
}

impl AddonAppService {
    pub(crate) fn new(store: SqliteStore) -> Self {
        Self { store }
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
