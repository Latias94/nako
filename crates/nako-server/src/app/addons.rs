use std::{collections::HashSet, sync::Arc, time::Instant};

use nako_addon_client::{
    AddonClientError, ReqwestAddonTransport, call_addon_resource_with_outcome, check_addon_health,
};
use nako_addon_protocol::{
    AddonManifest, AddonScope, addon_install_guide as protocol_addon_install_guide,
    ensure_scope_grant, validate_install_descriptor, validate_manifest,
};
use nako_api::extension::{
    AddonGrantsResponse, AddonTokenIssuedResponse, AddonTokenResponse, AddonTokenRotationResponse,
    AddonTokenSummary, AddonTokensResponse, AdminAddonConfigurationSchemaSurface,
    AdminAddonEntryPointSurface, AdminAddonEventSubscriptionSurface, AdminAddonHealthCheckResponse,
    AdminAddonHealthCheckStatus, AdminAddonHostedPageSurface,
    AdminAddonInstallGuideLifecycleBoundary, AdminAddonInstallGuidePreviewRequest,
    AdminAddonInstallGuidePreviewResponse, AdminAddonInstallGuideResponse,
    AdminAddonInstallGuideSecretReference, AdminAddonInstallGuideSnippet,
    AdminAddonInstallGuideStep, AdminAddonRegistrationDetail, AdminAddonRegistrationResponse,
    AdminAddonRegistrationSummary, AdminAddonRegistrationsResponse,
    AdminAddonResourceCallDiagnosticRequest, AdminAddonResourceCallDiagnosticResponse,
    AdminAddonResourceCallDiagnosticStatus, AdminAddonRoutingPlansResponse,
    AdminAddonRuntimeReadinessCheck, AdminAddonRuntimeReadinessCheckName,
    AdminAddonRuntimeReadinessDiagnostics, AdminAddonRuntimeReadinessReason,
    AdminAddonRuntimeReadinessResponse, AdminAddonSecretReferenceFieldSurface,
    AdminAddonSurfacesResponse, AdminAddonTaskSurface, IssueAddonTokenRequest,
    RegisterAddonRequest, ReplaceAddonGrantsRequest, UpdateAddonStatusRequest,
};
use nako_core::{
    AddonId, AddonIssuedToken, AddonManifestFingerprint, AddonRegistrationRecord, AddonRepository,
    AddonRoutingDeclarationKind, AddonRoutingPlanId, AddonRoutingPlanStatus,
    AddonRoutingPlanTarget, AddonStatus, AddonTokenId, DomainEventKind, JobKind, NakoError,
    NewAddonRegistration, NewAddonRoutingPlan, NewAddonToken, Result,
};
use nako_db::NakoDatabase;
use tokio::sync::Semaphore;

use super::storage::StorageBackendRegistry;

mod artwork_write;
mod intake;
mod library_file_write;
mod metadata_write;
mod principal;
mod runtime;
mod side_effect_apply;
mod target;

use principal::{normalize_grants, normalize_token_label};

#[derive(Clone, Debug)]
pub(crate) struct AddonAppService {
    store: NakoDatabase,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
}

impl AddonAppService {
    pub(super) fn new(
        store: NakoDatabase,
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
            .store
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

        let addon = self.store.upsert_addon_registration(addon).await?;

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
            .store
            .list_addon_registrations(status)
            .await?
            .iter()
            .map(AdminAddonRegistrationSummary::from_record)
            .collect();

        Ok(AdminAddonRegistrationsResponse { addons })
    }

    pub fn preview_addon_install_guide(
        &self,
        request: AdminAddonInstallGuidePreviewRequest,
    ) -> Result<AdminAddonInstallGuidePreviewResponse> {
        validate_install_descriptor(&request.descriptor).map_err(|_err| {
            NakoError::InvalidInput {
                message: "invalid addon install descriptor".to_owned(),
            }
        })?;

        Ok(AdminAddonInstallGuidePreviewResponse {
            guide: protocol_addon_install_guide(&request.descriptor),
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
            .store
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
            .store
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

    pub async fn check_addon_health(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonHealthCheckResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let manifest = self.stored_manifest(&addon)?;
        let started = Instant::now();
        let response = check_addon_health(
            &ReqwestAddonTransport::default(),
            &manifest,
            format!("addon-health-{addon_id}"),
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(response) => Ok(AdminAddonHealthCheckResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                status: response.status.into(),
                latency_ms,
                protocol_version: Some(response.protocol_version),
                addon_version: Some(response.manifest.addon_version),
                resource_count: Some(response.manifest.resource_count),
                protocol_checked_at: Some(response.checked_at),
                safe_error_code: None,
            }),
            Err(err) => Ok(AdminAddonHealthCheckResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                status: health_status_for_client_error(&err),
                latency_ms,
                protocol_version: None,
                addon_version: None,
                resource_count: None,
                protocol_checked_at: None,
                safe_error_code: Some(safe_health_error_code(&err).to_owned()),
            }),
        }
    }

    pub async fn check_addon_runtime_readiness(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonRuntimeReadinessResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }

        let manifest = self.stored_manifest(&addon)?;
        let mut checks = Vec::new();
        let mut should_call_sidecar = true;

        if let Err(err) = validate_manifest(&manifest) {
            return Ok(AdminAddonRuntimeReadinessResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                readiness: AdminAddonRuntimeReadinessDiagnostics::from_checks(vec![
                    AdminAddonRuntimeReadinessCheck::unavailable(
                        AdminAddonRuntimeReadinessCheckName::Manifest,
                        manifest_runtime_readiness_reason(&err),
                        "invalid_manifest",
                    ),
                ]),
            });
        }

        let granted_scopes = stored_granted_scopes(&addon)?;
        let missing_grant = manifest
            .resources
            .iter()
            .any(|resource| ensure_scope_grant(&manifest, resource.kind, &granted_scopes).is_err());
        if missing_grant {
            should_call_sidecar = false;
            checks.push(AdminAddonRuntimeReadinessCheck::degraded(
                AdminAddonRuntimeReadinessCheckName::Grants,
                AdminAddonRuntimeReadinessReason::MissingGrant,
                "missing_grant",
            ));
        } else {
            checks.push(AdminAddonRuntimeReadinessCheck::ready(
                AdminAddonRuntimeReadinessCheckName::Grants,
                AdminAddonRuntimeReadinessReason::Ready,
            ));
        }

        if manifest
            .secret_reference_fields
            .iter()
            .any(|field| field.required)
        {
            should_call_sidecar = false;
            checks.push(AdminAddonRuntimeReadinessCheck::degraded(
                AdminAddonRuntimeReadinessCheckName::SecretReferences,
                AdminAddonRuntimeReadinessReason::MissingSecretReference,
                "missing_secret_reference",
            ));
        } else {
            checks.push(AdminAddonRuntimeReadinessCheck::ready(
                AdminAddonRuntimeReadinessCheckName::SecretReferences,
                AdminAddonRuntimeReadinessReason::Ready,
            ));
        }

        if addon_runtime_network_policy_blocked(&manifest.base_url) {
            should_call_sidecar = false;
            checks.push(AdminAddonRuntimeReadinessCheck::unavailable(
                AdminAddonRuntimeReadinessCheckName::Network,
                AdminAddonRuntimeReadinessReason::NetworkPolicyBlocked,
                "network_policy_blocked",
            ));
        } else {
            checks.push(AdminAddonRuntimeReadinessCheck::ready(
                AdminAddonRuntimeReadinessCheckName::Network,
                AdminAddonRuntimeReadinessReason::Ready,
            ));
        }

        if should_call_sidecar {
            let health = check_addon_health(
                &ReqwestAddonTransport::default(),
                &manifest,
                format!("addon-readiness-{addon_id}"),
            )
            .await;
            match health {
                Ok(response) => {
                    checks.push(runtime_readiness_check_for_health_status(response.status));
                    checks.extend([
                        AdminAddonRuntimeReadinessCheck::ready(
                            AdminAddonRuntimeReadinessCheckName::Protocol,
                            AdminAddonRuntimeReadinessReason::Ready,
                        ),
                        AdminAddonRuntimeReadinessCheck::ready(
                            AdminAddonRuntimeReadinessCheckName::Manifest,
                            AdminAddonRuntimeReadinessReason::Ready,
                        ),
                        AdminAddonRuntimeReadinessCheck::ready(
                            AdminAddonRuntimeReadinessCheckName::Safety,
                            AdminAddonRuntimeReadinessReason::Ready,
                        ),
                    ]);
                }
                Err(err) => checks.push(runtime_readiness_check_for_client_error(&err)),
            }
        }

        Ok(AdminAddonRuntimeReadinessResponse {
            addon_id,
            manifest_id: addon.manifest_id,
            readiness: AdminAddonRuntimeReadinessDiagnostics::from_checks(checks),
        })
    }

    pub async fn get_addon_surfaces(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonSurfacesResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        Ok(AdminAddonSurfacesResponse {
            addon_id,
            manifest_id: addon.manifest_id,
            entry_points: manifest
                .entry_points
                .into_iter()
                .map(|entry_point| AdminAddonEntryPointSurface {
                    id: entry_point.id,
                    kind: entry_point.kind,
                    label: entry_point.label,
                    path: entry_point.path,
                    hosted_page_id: entry_point.hosted_page_id,
                    required_scopes: entry_point.required_scopes,
                })
                .collect(),
            hosted_pages: manifest
                .hosted_pages
                .into_iter()
                .map(|hosted_page| {
                    let url = addon_surface_url(&addon.base_url, &hosted_page.path);
                    AdminAddonHostedPageSurface {
                        id: hosted_page.id,
                        title: hosted_page.title,
                        path: hosted_page.path,
                        url,
                        required_scopes: hosted_page.required_scopes,
                    }
                })
                .collect(),
            configuration_schema: manifest
                .configuration_schema
                .map(AdminAddonConfigurationSchemaSurface::from),
            secret_reference_fields: manifest
                .secret_reference_fields
                .into_iter()
                .map(|field| AdminAddonSecretReferenceFieldSurface {
                    id: field.id,
                    label: field.label,
                    description: field.description,
                    required: field.required,
                })
                .collect(),
            tasks: manifest
                .tasks
                .into_iter()
                .map(AdminAddonTaskSurface::from)
                .collect(),
            event_subscriptions: manifest
                .event_subscriptions
                .into_iter()
                .map(|subscription| AdminAddonEventSubscriptionSurface {
                    id: subscription.id,
                    event_kind: subscription.event_kind,
                    path: subscription.path,
                    required_scopes: subscription.required_scopes,
                    filters: subscription.filters,
                })
                .collect(),
        })
    }

    pub async fn sync_addon_routing_plans(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonRoutingPlansResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        ensure_addon_accepts_runtime_authority(&addon, "sync addon routing plans")?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        let granted_scopes = stored_granted_scopes(&addon)?;
        let manifest_fingerprint = AddonManifestFingerprint::new(&addon.manifest_json);
        let plans = build_addon_routing_plans(
            addon_id,
            addon.status,
            &addon.manifest_id,
            &manifest.version,
            &manifest_fingerprint,
            &manifest,
            &granted_scopes,
        )?;
        let records = self
            .store
            .replace_addon_routing_plans(addon_id, plans)
            .await?;

        Ok(AdminAddonRoutingPlansResponse::from_records(
            addon_id,
            addon.manifest_id,
            manifest.version,
            manifest_fingerprint.to_string(),
            records,
        ))
    }

    pub async fn get_addon_install_guide(
        &self,
        addon_id: AddonId,
    ) -> Result<AdminAddonInstallGuideResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;

        Ok(addon_install_guide(&addon, &manifest))
    }

    pub async fn diagnose_addon_resource_call(
        &self,
        addon_id: AddonId,
        request: AdminAddonResourceCallDiagnosticRequest,
    ) -> Result<AdminAddonResourceCallDiagnosticResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let started = Instant::now();
        let response = call_addon_resource_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            request.resource,
            &granted_scopes,
            format!("addon-diagnostic-{addon_id}"),
            request.payload,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => Ok(AdminAddonResourceCallDiagnosticResponse {
                addon_id,
                manifest_id: addon.manifest_id,
                resource: request.resource,
                status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                latency_ms,
                attempts: outcome.attempts,
                http_status: Some(outcome.http_status),
                safe_error_code: None,
            }),
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceCallDiagnosticResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    resource: request.resource,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
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
        let grants = self.store.replace_addon_grants(addon_id, grants).await?;

        Ok(AddonGrantsResponse { grants })
    }

    pub async fn list_addon_grants(&self, addon_id: AddonId) -> Result<AddonGrantsResponse> {
        self.get_addon_registration_or_not_found(addon_id).await?;
        let grants = self.store.list_addon_grants(addon_id).await?;

        Ok(AddonGrantsResponse { grants })
    }

    async fn get_addon_registration_or_not_found(
        &self,
        addon_id: AddonId,
    ) -> Result<AddonRegistrationRecord> {
        self.store
            .get_addon_registration(addon_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_registration",
                id: addon_id.to_string(),
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

    fn stored_manifest(&self, addon: &AddonRegistrationRecord) -> Result<AddonManifest> {
        serde_json::from_str(&addon.manifest_json).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to parse stored addon manifest: {err}"),
        })
    }
}

fn ensure_addon_accepts_runtime_authority(
    addon: &AddonRegistrationRecord,
    operation: &'static str,
) -> Result<()> {
    if addon.status == AddonStatus::Unregistered {
        return Err(NakoError::Conflict {
            message: format!(
                "cannot {operation} for unregistered addon registration {}",
                addon.id
            ),
        });
    }

    Ok(())
}

fn health_status_for_client_error(err: &AddonClientError) -> AdminAddonHealthCheckStatus {
    match err {
        AddonClientError::Protocol(_) => AdminAddonHealthCheckStatus::ProtocolMismatch,
        AddonClientError::HttpStatus { .. } | AddonClientError::Http { .. } => {
            AdminAddonHealthCheckStatus::Unreachable
        }
    }
}

fn runtime_readiness_check_for_client_error(
    err: &AddonClientError,
) -> AdminAddonRuntimeReadinessCheck {
    let name = runtime_readiness_check_name_for_client_error(err);
    let (reason, code) = runtime_readiness_reason_and_code(err);
    AdminAddonRuntimeReadinessCheck::unavailable(name, reason, code)
}

fn runtime_readiness_check_for_health_status(
    status: nako_addon_protocol::AddonHealthStatus,
) -> AdminAddonRuntimeReadinessCheck {
    match status {
        nako_addon_protocol::AddonHealthStatus::Ok => AdminAddonRuntimeReadinessCheck::ready(
            AdminAddonRuntimeReadinessCheckName::Reachability,
            AdminAddonRuntimeReadinessReason::Ready,
        ),
        nako_addon_protocol::AddonHealthStatus::Degraded => {
            AdminAddonRuntimeReadinessCheck::degraded(
                AdminAddonRuntimeReadinessCheckName::Reachability,
                AdminAddonRuntimeReadinessReason::SidecarDegraded,
                "sidecar_degraded",
            )
        }
        nako_addon_protocol::AddonHealthStatus::Unhealthy => {
            AdminAddonRuntimeReadinessCheck::unavailable(
                AdminAddonRuntimeReadinessCheckName::Reachability,
                AdminAddonRuntimeReadinessReason::SidecarUnhealthy,
                "sidecar_unhealthy",
            )
        }
    }
}

fn runtime_readiness_check_name_for_client_error(
    err: &AddonClientError,
) -> AdminAddonRuntimeReadinessCheckName {
    match err {
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::UnsupportedProtocolVersion { .. },
        ) => AdminAddonRuntimeReadinessCheckName::Protocol,
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            message,
        }) if health_envelope_manifest_mismatch(message) => {
            AdminAddonRuntimeReadinessCheckName::Manifest
        }
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => AdminAddonRuntimeReadinessCheckName::Safety,
        AddonClientError::Protocol(_) => AdminAddonRuntimeReadinessCheckName::Safety,
        AddonClientError::HttpStatus { .. } | AddonClientError::Http { .. } => {
            AdminAddonRuntimeReadinessCheckName::Reachability
        }
    }
}

fn runtime_readiness_reason_and_code(
    err: &AddonClientError,
) -> (AdminAddonRuntimeReadinessReason, &'static str) {
    match err {
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::UnsupportedProtocolVersion { .. },
        ) => (
            AdminAddonRuntimeReadinessReason::ProtocolMismatch,
            "protocol_mismatch",
        ),
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            message,
        }) if health_envelope_manifest_mismatch(message) => (
            AdminAddonRuntimeReadinessReason::ManifestMismatch,
            "manifest_mismatch",
        ),
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "unsafe_response",
        ),
        AddonClientError::Protocol(_) => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "unsafe_response",
        ),
        AddonClientError::HttpStatus { .. } => (
            AdminAddonRuntimeReadinessReason::Unavailable,
            safe_health_error_code(err),
        ),
        AddonClientError::Http { .. } => (
            AdminAddonRuntimeReadinessReason::Unavailable,
            "transport_failure",
        ),
    }
}

fn manifest_runtime_readiness_reason(
    err: &nako_addon_protocol::AddonManifestError,
) -> AdminAddonRuntimeReadinessReason {
    match err {
        nako_addon_protocol::AddonManifestError::UnsupportedProtocolVersion { .. } => {
            AdminAddonRuntimeReadinessReason::ProtocolMismatch
        }
        _ => AdminAddonRuntimeReadinessReason::UnsafeResponse,
    }
}

fn health_envelope_manifest_mismatch(message: &str) -> bool {
    message.contains("manifest_id")
        || message.contains("addon_version")
        || message.contains("resource_count")
}

fn addon_runtime_network_policy_blocked(base_url: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(base_url) else {
        return true;
    };
    if url.username() != "" || url.password().is_some() {
        return true;
    }
    if url.scheme() == "https" {
        return false;
    }
    if url.scheme() != "http" {
        return true;
    }

    let Some(host) = url.host_str() else {
        return true;
    };
    !matches!(host, "127.0.0.1" | "::1" | "localhost")
}

fn safe_health_error_code(err: &AddonClientError) -> &'static str {
    match err {
        AddonClientError::Protocol(_) => "protocol_mismatch",
        AddonClientError::HttpStatus { status, .. } if *status == 401 || *status == 403 => {
            "unauthorized"
        }
        AddonClientError::HttpStatus { status, .. } if *status == 404 => "health_endpoint_missing",
        AddonClientError::HttpStatus { status, .. } if *status == 408 => "timeout",
        AddonClientError::HttpStatus { status, .. } if *status == 429 => "rate_limited",
        AddonClientError::HttpStatus { status, .. } if (500..600).contains(status) => {
            "sidecar_unhealthy"
        }
        AddonClientError::HttpStatus { .. } => "http_failure",
        AddonClientError::Http { .. } => "transport_failure",
    }
}

fn resource_diagnostic_status_for_client_error(
    err: &AddonClientError,
) -> AdminAddonResourceCallDiagnosticStatus {
    match err {
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::MissingAuthToken {
            ..
        }) => AdminAddonResourceCallDiagnosticStatus::AuthorizationGap,
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::ResourceNotDeclared { .. },
        ) => AdminAddonResourceCallDiagnosticStatus::MissingResource,
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::MissingDeclaredScope { .. },
        ) => AdminAddonResourceCallDiagnosticStatus::MissingGrant,
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => AdminAddonResourceCallDiagnosticStatus::UnsafeResponse,
        AddonClientError::Protocol(_) => AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch,
        AddonClientError::HttpStatus {
            retryable: true, ..
        } => AdminAddonResourceCallDiagnosticStatus::RetryableHttpFailure,
        AddonClientError::HttpStatus {
            retryable: false, ..
        } => AdminAddonResourceCallDiagnosticStatus::HttpFailure,
        AddonClientError::Http { .. } => AdminAddonResourceCallDiagnosticStatus::Unreachable,
    }
}

fn safe_resource_diagnostic_error_code(err: &AddonClientError) -> &'static str {
    match err {
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::MissingAuthToken {
            ..
        }) => "authorization_gap",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::ResourceNotDeclared { .. },
        ) => "missing_resource",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::MissingDeclaredScope { .. },
        ) => "missing_grant",
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => "unsafe_response",
        AddonClientError::Protocol(_) => "protocol_mismatch",
        AddonClientError::HttpStatus {
            retryable: true, ..
        } => "retryable_http_failure",
        AddonClientError::HttpStatus {
            retryable: false, ..
        } => "http_failure",
        AddonClientError::Http { .. } => "transport_failure",
    }
}

fn addon_surface_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

fn build_addon_routing_plans(
    addon_id: AddonId,
    addon_status: AddonStatus,
    manifest_id: &str,
    manifest_version: &str,
    manifest_fingerprint: &AddonManifestFingerprint,
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
) -> Result<Vec<NewAddonRoutingPlan>> {
    let mut plans = Vec::new();
    let addon_enabled = addon_status == AddonStatus::Enabled;

    for task in &manifest.tasks {
        let granted = declaration_scopes_granted(&task.required_scopes, granted_scopes);
        let reason = if !addon_enabled {
            Some("addon_disabled")
        } else if !granted {
            Some("missing_grant")
        } else {
            None
        };
        let executable = reason.is_none();
        let status = if executable {
            AddonRoutingPlanStatus::Executable
        } else {
            AddonRoutingPlanStatus::Deferred
        };
        let target = if executable {
            AddonRoutingPlanTarget::AddonTaskJob
        } else {
            AddonRoutingPlanTarget::None
        };
        let job_kind = executable.then_some(JobKind::AddonTask);
        plans.push(NewAddonRoutingPlan {
            id: AddonRoutingPlanId::new(),
            addon_id,
            manifest_id: manifest_id.to_owned(),
            manifest_version: manifest_version.to_owned(),
            manifest_fingerprint: manifest_fingerprint.clone(),
            declaration_kind: AddonRoutingDeclarationKind::Task,
            declaration_id: task.id.clone(),
            status,
            target,
            safe_reason_code: reason.map(ToOwned::to_owned),
            job_kind,
            event_kind: None,
            plan_json: routing_plan_json(RoutingPlanJson {
                declaration_kind: AddonRoutingDeclarationKind::Task,
                declaration_id: &task.id,
                target,
                status,
                safe_reason_code: reason,
                job_kind: job_kind.map(JobKind::as_str),
                event_kind: None,
                required_scope_count: task.required_scopes.len(),
                filter_configured: false,
                timeout_ms: task.timeout_ms,
                max_attempts: task.max_attempts,
            })?,
        });
    }

    for subscription in &manifest.event_subscriptions {
        let granted = declaration_scopes_granted(&subscription.required_scopes, granted_scopes);
        let parsed_event_kind = DomainEventKind::parse(&subscription.event_kind).ok();
        let reason = if !addon_enabled {
            Some("addon_disabled")
        } else if !granted {
            Some("missing_grant")
        } else if parsed_event_kind.is_none() {
            Some("unsupported_event_kind")
        } else {
            None
        };
        let status = if reason.is_none() {
            AddonRoutingPlanStatus::Executable
        } else {
            AddonRoutingPlanStatus::Deferred
        };
        let target = if reason.is_none() {
            AddonRoutingPlanTarget::EventOutbox
        } else {
            AddonRoutingPlanTarget::None
        };
        let event_kind = parsed_event_kind.map(|kind| kind.as_str().to_owned());
        plans.push(NewAddonRoutingPlan {
            id: AddonRoutingPlanId::new(),
            addon_id,
            manifest_id: manifest_id.to_owned(),
            manifest_version: manifest_version.to_owned(),
            manifest_fingerprint: manifest_fingerprint.clone(),
            declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
            declaration_id: subscription.id.clone(),
            status,
            target,
            safe_reason_code: reason.map(ToOwned::to_owned),
            job_kind: None,
            event_kind: event_kind.clone(),
            plan_json: routing_plan_json(RoutingPlanJson {
                declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
                declaration_id: &subscription.id,
                target,
                status,
                safe_reason_code: reason,
                job_kind: None,
                event_kind: event_kind.as_deref(),
                required_scope_count: subscription.required_scopes.len(),
                filter_configured: !subscription.filters.is_null(),
                timeout_ms: None,
                max_attempts: None,
            })?,
        });
    }

    Ok(plans)
}

fn declaration_scopes_granted(required: &[AddonScope], granted: &[AddonScope]) -> bool {
    required.iter().all(|scope| granted.contains(scope))
}

struct RoutingPlanJson<'a> {
    declaration_kind: AddonRoutingDeclarationKind,
    declaration_id: &'a str,
    target: AddonRoutingPlanTarget,
    status: AddonRoutingPlanStatus,
    safe_reason_code: Option<&'static str>,
    job_kind: Option<&'static str>,
    event_kind: Option<&'a str>,
    required_scope_count: usize,
    filter_configured: bool,
    timeout_ms: Option<u64>,
    max_attempts: Option<u32>,
}

fn routing_plan_json(plan: RoutingPlanJson<'_>) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "schema": "nako.addon.routing_plan.v1",
        "declaration_kind": plan.declaration_kind,
        "declaration_id": plan.declaration_id,
        "target": plan.target,
        "status": plan.status,
        "safe_reason_code": plan.safe_reason_code,
        "job_kind": plan.job_kind,
        "event_kind": plan.event_kind,
        "required_scope_count": plan.required_scope_count,
        "filter_configured": plan.filter_configured,
        "timeout_ms": plan.timeout_ms,
        "max_attempts": plan.max_attempts,
    }))
    .map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize addon routing plan: {err}"),
    })
}

fn addon_install_guide(
    addon: &AddonRegistrationRecord,
    manifest: &AddonManifest,
) -> AdminAddonInstallGuideResponse {
    let service_name = addon_service_name(&manifest.id);
    let health_url = addon_surface_url(&manifest.base_url, "/health");
    let secret_references = manifest
        .secret_reference_fields
        .iter()
        .map(|field| AdminAddonInstallGuideSecretReference {
            id: field.id.clone(),
            label: field.label.clone(),
            description: field.description.clone(),
            required: field.required,
            env_var: secret_reference_env_var(&field.id),
            placeholder: format!("secret-reference:{}", field.id),
        })
        .collect::<Vec<_>>();

    AdminAddonInstallGuideResponse {
        addon_id: addon.id,
        manifest_id: addon.manifest_id.clone(),
        addon_name: addon.name.clone(),
        addon_version: addon.version.clone(),
        protocol_version: addon.protocol_version.clone(),
        base_url: addon.base_url.clone(),
        status: addon.status,
        docker_compose: docker_compose_install_snippet(manifest, &service_name, &secret_references),
        systemd: systemd_install_snippet(manifest, &service_name, &secret_references),
        secret_references,
        health_check_steps: vec![
            AdminAddonInstallGuideStep {
                title: "Check the Addon Sidecar health contract directly".to_owned(),
                command: format!(
                    "curl -fsS -X POST {} -H {} -d {}",
                    shell_quote(&health_url),
                    shell_quote("Content-Type: application/json"),
                    shell_quote(&serde_json::json!({
                        "protocol_version": manifest.protocol_version,
                        "manifest_id": manifest.id,
                        "request_id": "manual-health-check",
                        "expected_addon_version": manifest.version,
                        "expected_resource_count": manifest.resources.len()
                    })
                    .to_string())
                ),
                expected_result: "The sidecar returns matching protocol, manifest, addon version, and resource-count facts.".to_owned(),
            },
            AdminAddonInstallGuideStep {
                title: "Check the Addon through Nako Admin API".to_owned(),
                command: format!(
                    "curl -fsS -X POST \"$NAKO_BASE_URL/admin/v1/addons/{}/health-check\" -H {}",
                    addon.id,
                    shell_quote("Authorization: <admin-auth-header>")
                ),
                expected_result: "Nako returns a redaction-safe Addon Health Check status without sending Admin credentials or resolved secrets to the sidecar.".to_owned(),
            },
        ],
        registration_verification_steps: vec![
            AdminAddonInstallGuideStep {
                title: "Verify the registered Addon manifest snapshot".to_owned(),
                command: format!(
                    "curl -fsS \"$NAKO_BASE_URL/admin/v1/addons/{}\" -H {}",
                    addon.id,
                    shell_quote("Authorization: <admin-auth-header>")
                ),
                expected_result: format!(
                    "The response summary contains manifest_id `{}` and status `{}`.",
                    addon.manifest_id,
                    addon.status.as_str()
                ),
            },
            AdminAddonInstallGuideStep {
                title: "Verify declared Addon surfaces".to_owned(),
                command: format!(
                    "curl -fsS \"$NAKO_BASE_URL/admin/v1/addons/{}/surfaces\" -H {}",
                    addon.id,
                    shell_quote("Authorization: <admin-auth-header>")
                ),
                expected_result: "The response lists Entry Points, Hosted Pages, Secret Reference fields, Tasks, and Event Subscriptions as declarations only.".to_owned(),
            },
        ],
        lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary {
            nako_manages_containers: false,
            nako_manages_processes: false,
            nako_manages_packages: false,
            message: "Nako generates this guide only. The operator owns Addon Sidecar installation, start/stop, upgrades, logs, and removal outside Nako.".to_owned(),
        },
    }
}

fn docker_compose_install_snippet(
    manifest: &AddonManifest,
    service_name: &str,
    secret_references: &[AdminAddonInstallGuideSecretReference],
) -> AdminAddonInstallGuideSnippet {
    let mut environment = vec![
        format!(
            "      NAKO_ADDON_BASE_URL: {}",
            yaml_quote(&manifest.base_url)
        ),
        format!(
            "      NAKO_ADDON_PROTOCOL_VERSION: {}",
            yaml_quote(&manifest.protocol_version)
        ),
        format!("      NAKO_ADDON_MANIFEST_ID: {}", yaml_quote(&manifest.id)),
    ];
    if secret_references.is_empty() {
        environment
            .push("      # No Secret Reference fields are declared by this manifest.".to_owned());
    } else {
        environment.extend(secret_references.iter().map(|secret| {
            format!(
                "      {}: {}",
                secret.env_var,
                yaml_quote(&secret.placeholder)
            )
        }));
    }

    let content = format!(
        r#"services:
  {service_name}:
    image: {image}
    restart: unless-stopped
    environment:
{environment}
    healthcheck:
      test: ["CMD-SHELL", {healthcheck}]
      interval: 30s
      timeout: 5s
      retries: 5
      start_period: 20s
"#,
        image = yaml_quote(&format!(
            "<replace-with-{}-image>:{}",
            service_name, manifest.version
        )),
        environment = environment.join("\n"),
        healthcheck = yaml_quote(&format!(
            "curl -fsS {} >/dev/null",
            addon_surface_url(&manifest.base_url, "/health")
        )),
    );

    AdminAddonInstallGuideSnippet {
        title: "Docker Compose sidecar snippet".to_owned(),
        filename: format!("compose.{service_name}.yml"),
        content,
        notes: vec![
            "Run this Addon Sidecar as a separate service on a network Nako can reach.".to_owned(),
            "Replace the image placeholder with the Addon author's published image.".to_owned(),
            "Nako does not mount the Docker socket or manage this container lifecycle.".to_owned(),
        ],
    }
}

fn systemd_install_snippet(
    manifest: &AddonManifest,
    service_name: &str,
    secret_references: &[AdminAddonInstallGuideSecretReference],
) -> AdminAddonInstallGuideSnippet {
    let mut environment = vec![
        systemd_environment("NAKO_ADDON_BASE_URL", &manifest.base_url),
        systemd_environment("NAKO_ADDON_PROTOCOL_VERSION", &manifest.protocol_version),
        systemd_environment("NAKO_ADDON_MANIFEST_ID", &manifest.id),
    ];
    if secret_references.is_empty() {
        environment.push("# No Secret Reference fields are declared by this manifest.".to_owned());
    } else {
        environment.extend(
            secret_references
                .iter()
                .map(|secret| systemd_environment(&secret.env_var, &secret.placeholder)),
        );
    }

    let content = format!(
        r#"[Unit]
Description={name} Addon Sidecar
After=network-online.target

[Service]
Type=simple
{environment}
ExecStart=<addon-sidecar-command> --listen 0.0.0.0:{port}
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
"#,
        name = manifest.name,
        environment = environment.join("\n"),
        port = addon_base_url_port(&manifest.base_url),
    );

    AdminAddonInstallGuideSnippet {
        title: "systemd sidecar unit snippet".to_owned(),
        filename: format!("{service_name}.service"),
        content,
        notes: vec![
            "Replace <addon-sidecar-command> with the Addon author's binary and arguments.".to_owned(),
            "Keep Secret Reference placeholders out of this unit until your host secret policy resolves them safely.".to_owned(),
            "Nako does not call systemd or supervise this process.".to_owned(),
        ],
    }
}

fn addon_service_name(manifest_id: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;
    for character in manifest_id.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            output.push('-');
            last_was_dash = true;
        }
    }
    let output = output.trim_matches('-').to_owned();
    if output.is_empty() {
        "nako-addon-sidecar".to_owned()
    } else {
        output
    }
}

fn secret_reference_env_var(id: &str) -> String {
    let mut output = String::from("ADDON_SECRET_");
    let mut last_was_underscore = false;
    for character in id.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_uppercase());
            last_was_underscore = false;
        } else if !last_was_underscore {
            output.push('_');
            last_was_underscore = true;
        }
    }

    while output.ends_with('_') {
        output.pop();
    }
    if output == "ADDON_SECRET" {
        "ADDON_SECRET_VALUE".to_owned()
    } else {
        output
    }
}

fn addon_base_url_port(base_url: &str) -> u16 {
    reqwest::Url::parse(base_url)
        .ok()
        .and_then(|url| url.port_or_known_default())
        .unwrap_or(8080)
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn systemd_environment(key: &str, value: &str) -> String {
    format!(
        "Environment=\"{key}={}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn stored_granted_scopes(addon: &AddonRegistrationRecord) -> Result<Vec<AddonScope>> {
    addon
        .granted_scopes
        .iter()
        .map(|scope| {
            serde_json::from_value::<AddonScope>(serde_json::Value::String(scope.clone())).map_err(
                |err| NakoError::InvalidInput {
                    message: format!("invalid stored addon scope `{scope}`: {err}"),
                },
            )
        })
        .collect()
}
