use std::{collections::HashSet, fmt::Debug};

use nako_addon_protocol::{ensure_scope_grant, validate_manifest};
use nako_api::extension::{
    AddonGrantsResponse, AddonTokenIssuedResponse, AddonTokenResponse, AddonTokenRotationResponse,
    AddonTokenSummary, AddonTokensResponse, AdminAddonManagerPlanRequest,
    AdminAddonManagerPlanResponse, AdminAddonRegistrationDetail, AdminAddonRegistrationResponse,
    AdminAddonRegistrationSummary, AdminAddonRegistrationsResponse, IssueAddonTokenRequest,
    RegisterAddonRequest, ReplaceAddonGrantsRequest, UpdateAddonStatusRequest,
};
use nako_core::{
    AddonGrantRecord, AddonId, AddonIssuedToken, AddonRegistrationRecord, AddonRepository,
    AddonStatus, AddonTokenId, NakoError, NewAddonGrant, NewAddonRegistration, NewAddonToken,
    Result,
};

use super::{
    AddonAppService,
    helpers::{
        ensure_addon_accepts_runtime_authority, normalize_optional_secret_env,
        stored_granted_scopes,
    },
    principal::{normalize_grants, normalize_token_label},
};

#[async_trait::async_trait]
pub(super) trait AddonRegistrationStore: Debug + Send + Sync {
    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>>;

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>>;

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>>;

    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord>;

    async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        grants: Vec<NewAddonGrant>,
    ) -> Result<Vec<AddonGrantRecord>>;

    async fn list_addon_grants(&self, addon_id: AddonId) -> Result<Vec<AddonGrantRecord>>;

    async fn update_addon_registration_status(
        &self,
        id: AddonId,
        status: AddonStatus,
    ) -> Result<Option<AddonRegistrationRecord>>;

    async fn unregister_addon_registration(
        &self,
        id: AddonId,
    ) -> Result<Option<AddonRegistrationRecord>>;
}

#[async_trait::async_trait]
impl<T> AddonRegistrationStore for T
where
    T: AddonRepository + Debug + Send + Sync,
{
    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>> {
        AddonRepository::find_addon_registration_by_manifest_id(self, manifest_id).await
    }

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>> {
        AddonRepository::get_addon_registration(self, id).await
    }

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>> {
        AddonRepository::list_addon_registrations(self, status).await
    }

    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord> {
        AddonRepository::upsert_addon_registration(self, addon).await
    }

    async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        grants: Vec<NewAddonGrant>,
    ) -> Result<Vec<AddonGrantRecord>> {
        AddonRepository::replace_addon_grants(self, addon_id, grants).await
    }

    async fn list_addon_grants(&self, addon_id: AddonId) -> Result<Vec<AddonGrantRecord>> {
        AddonRepository::list_addon_grants(self, addon_id).await
    }

    async fn update_addon_registration_status(
        &self,
        id: AddonId,
        status: AddonStatus,
    ) -> Result<Option<AddonRegistrationRecord>> {
        AddonRepository::update_addon_registration_status(self, id, status).await
    }

    async fn unregister_addon_registration(
        &self,
        id: AddonId,
    ) -> Result<Option<AddonRegistrationRecord>> {
        AddonRepository::unregister_addon_registration(self, id).await
    }
}

impl AddonAppService {
    fn normalize_addon_registration(
        &self,
        request: RegisterAddonRequest,
    ) -> Result<nako_core::NewAddonRegistration> {
        validate_manifest(&request.manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        let mut seen = HashSet::new();
        let granted_scopes = request
            .granted_scopes
            .into_iter()
            .filter(|scope| seen.insert(*scope))
            .collect::<Vec<_>>();
        let status = request.status.unwrap_or(AddonStatus::Disabled);

        if status == AddonStatus::Enabled {
            for resource in &request.manifest.resources {
                ensure_scope_grant(&request.manifest, resource.kind, &granted_scopes).map_err(
                    |err| NakoError::InvalidInput {
                        message: err.to_string(),
                    },
                )?;
            }
        }

        let manifest_json =
            serde_json::to_string(&request.manifest).map_err(|err| NakoError::InvalidInput {
                message: format!("failed to serialize addon manifest: {err}"),
            })?;
        let outbound_task_dispatch_secret_env = normalize_optional_secret_env(
            request.outbound_task_dispatch_secret_env,
            "outbound_task_dispatch_secret_env",
        )?;
        let granted_scopes = granted_scopes
            .into_iter()
            .map(|scope| scope.as_str().to_owned())
            .collect();

        Ok(nako_core::NewAddonRegistration {
            id: request.id.unwrap_or_else(AddonId::new),
            manifest_id: request.manifest.id,
            name: request.manifest.name,
            version: request.manifest.version,
            protocol_version: request.manifest.protocol_version,
            base_url: request.manifest.base_url,
            manifest_json,
            outbound_task_dispatch_secret_env,
            granted_scopes,
            status,
        })
    }

    pub async fn register_addon(
        &self,
        request: RegisterAddonRequest,
    ) -> Result<AdminAddonRegistrationResponse> {
        let request_id = request.id;
        let manifest_id = request.manifest.id.clone();
        let addon = self.normalize_addon_registration(request)?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::InvalidInput {
                message: "addon registration cannot start as unregistered".to_owned(),
            });
        }

        let existing = self
            .registration_store
            .find_addon_registration_by_manifest_id(&manifest_id)
            .await?;
        if let Some(existing) = existing {
            if existing.status == AddonStatus::Unregistered {
                if request_id.is_some_and(|id| id == existing.id) {
                    return Err(NakoError::Conflict {
                        message: format!(
                            "addon manifest {} is unregistered; register a new addon id",
                            existing.manifest_id
                        ),
                    });
                }
                if addon.status != AddonStatus::Disabled {
                    return Err(NakoError::InvalidInput {
                        message: format!(
                            "addon manifest {manifest_id} was previously unregistered; re-registration must start disabled"
                        ),
                    });
                }
            } else if request_id.is_some_and(|id| id != existing.id) || request_id.is_none() {
                return Err(NakoError::Conflict {
                    message: format!(
                        "addon manifest {} is already registered as {}",
                        existing.manifest_id, existing.id
                    ),
                });
            }
        }

        let addon = self
            .registration_store
            .upsert_addon_registration(addon)
            .await?;

        Ok(AdminAddonRegistrationResponse {
            addon: self.admin_addon_detail(&addon)?,
        })
    }

    pub async fn get_addon_registration(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonRegistrationResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;

        Ok(AdminAddonRegistrationResponse {
            addon: self.admin_addon_detail(&addon)?,
        })
    }

    pub async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<AdminAddonRegistrationsResponse> {
        let addons = self
            .registration_store
            .list_addon_registrations(status)
            .await?
            .iter()
            .map(AdminAddonRegistrationSummary::from_record)
            .collect();

        Ok(AdminAddonRegistrationsResponse { addons })
    }

    pub async fn get_addon_manager_plan(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonManagerPlanResponse> {
        self.addon_manager_plan_snapshot(addon_id).await
    }

    pub async fn plan_addon_manager_lifecycle(
        &self,
        addon_id: AddonId,
        request: AdminAddonManagerPlanRequest,
    ) -> Result<AdminAddonManagerPlanResponse> {
        if !request.operator_confirmed {
            return Err(NakoError::InvalidInput {
                message: "operator confirmation is required for addon manager lifecycle plans"
                    .to_owned(),
            });
        }

        let snapshot = self.addon_manager_plan_snapshot(addon_id).await?;

        Ok(AdminAddonManagerPlanResponse {
            intent: Some(request.intent),
            operator_confirmed: true,
            ..snapshot
        })
    }

    async fn addon_manager_plan_snapshot(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonManagerPlanResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        let source = self.admin_addon_detail(&addon)?;
        let health_check = self.check_addon_health(addon_id).await?;
        let tokens = self.list_addon_tokens(addon_id).await?;
        let grants = self.list_addon_grants(addon_id).await?;
        let install_guide = self.get_addon_install_guide(addon_id).await?;

        Ok(AdminAddonManagerPlanResponse {
            addon_id,
            intent: None,
            operator_confirmed: false,
            source,
            health_check,
            tokens,
            grants,
            install_guide,
        })
    }

    pub async fn update_addon_status(
        &self,
        addon_id: AddonId,
        request: UpdateAddonStatusRequest,
    ) -> Result<AdminAddonRegistrationResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if request.status == AddonStatus::Unregistered {
            return Err(NakoError::InvalidInput {
                message: "use addon unregister command for terminal unregistration".to_owned(),
            });
        }
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        if request.status == AddonStatus::Enabled {
            self.validate_registered_addon_can_be_enabled(&addon)?;
        }

        let addon = self
            .registration_store
            .update_addon_registration_status(addon_id, request.status)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_registration",
                id: addon_id.to_string(),
            })?;

        Ok(AdminAddonRegistrationResponse {
            addon: self.admin_addon_detail(&addon)?,
        })
    }

    pub async fn unregister_addon(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonRegistrationResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let addon = self
            .registration_store
            .unregister_addon_registration(addon_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_registration",
                id: addon_id.to_string(),
            })?;

        Ok(AdminAddonRegistrationResponse {
            addon: self.admin_addon_detail(&addon)?,
        })
    }

    pub async fn issue_addon_token(
        &self,
        addon_id: AddonId,
        request: IssueAddonTokenRequest,
    ) -> Result<AddonTokenIssuedResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "issue addon token")?;
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
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "rotate addon token")?;
        let existing =
            self.store
                .get_addon_token(token_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "addon_token",
                    id: token_id.to_string(),
                })?;
        if existing.addon_id != addon_id {
            return Err(NakoError::NotFound {
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
                .ok_or_else(|| NakoError::NotFound {
                    entity: "addon_token",
                    id: token_id.to_string(),
                })?;
        if existing.addon_id != addon_id {
            return Err(NakoError::NotFound {
                entity: "addon_token",
                id: token_id.to_string(),
            });
        }

        let token = self
            .store
            .revoke_addon_token(token_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
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
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "replace addon grants")?;
        let grants = normalize_grants(addon_id, request.grants)?;
        let grants = self
            .registration_store
            .replace_addon_grants(addon_id, grants)
            .await?;

        Ok(AddonGrantsResponse { grants })
    }

    pub async fn list_addon_grants(&self, addon_id: AddonId) -> Result<AddonGrantsResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let grants = self.registration_store.list_addon_grants(addon_id).await?;

        Ok(AddonGrantsResponse { grants })
    }

    pub(super) async fn get_addon_registration_or_not_found(
        &self,
        addon_id: AddonId,
    ) -> Result<AddonRegistrationRecord> {
        self.registration_store
            .get_addon_registration(addon_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_registration",
                id: addon_id.to_string(),
            })
    }

    pub(super) fn stored_manifest(
        &self,
        addon: &AddonRegistrationRecord,
    ) -> Result<nako_addon_protocol::AddonManifest> {
        serde_json::from_str(&addon.manifest_json).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to parse stored addon manifest: {err}"),
        })
    }

    fn admin_addon_detail(
        &self,
        addon: &AddonRegistrationRecord,
    ) -> Result<AdminAddonRegistrationDetail> {
        AdminAddonRegistrationDetail::from_record(addon).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to parse stored addon manifest: {err}"),
        })
    }

    fn validate_registered_addon_can_be_enabled(
        &self,
        addon: &AddonRegistrationRecord,
    ) -> Result<()> {
        let manifest = self.stored_manifest(addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        let granted_scopes = stored_granted_scopes(addon)?;

        for resource in &manifest.resources {
            ensure_scope_grant(&manifest, resource.kind, &granted_scopes).map_err(|err| {
                NakoError::InvalidInput {
                    message: err.to_string(),
                }
            })?;
        }

        Ok(())
    }
}
