use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
    time::Instant,
};

use nako_addon_client::{
    AddonClientError, ReqwestAddonTransport, call_addon_resource_link_check_with_outcome,
    call_addon_resource_search_with_outcome, call_addon_resource_with_outcome, check_addon_health,
};
use nako_addon_protocol::{
    ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
    AddonInstallDescriptor, AddonManifest, AddonResourceLink, AddonResourceLinkCheckRequest,
    AddonResourceSearchRequest, AddonResourceSearchResult, AddonScope,
    addon_install_guide as protocol_addon_install_guide, ensure_scope_grant,
    validate_install_descriptor, validate_manifest,
};
use nako_api::extension::{
    AddonAcquisitionCandidateSummary, AddonGrantsResponse, AddonTokenIssuedResponse,
    AddonTokenResponse, AddonTokenRotationResponse, AddonTokenSummary, AddonTokensResponse,
    AdminAddonConfigurationSchemaSurface, AdminAddonEntryPointSurface,
    AdminAddonEventSubscriptionSurface, AdminAddonHealthCheckResponse, AdminAddonHealthCheckStatus,
    AdminAddonHostedPageSurface, AdminAddonInstallGuideLifecycleBoundary,
    AdminAddonInstallGuidePreviewRequest, AdminAddonInstallGuidePreviewResponse,
    AdminAddonInstallGuideResponse, AdminAddonInstallGuideSecretReference,
    AdminAddonInstallGuideSnippet, AdminAddonInstallGuideStep, AdminAddonManagerPlanRequest,
    AdminAddonManagerPlanResponse, AdminAddonRegistrationDetail, AdminAddonRegistrationResponse,
    AdminAddonRegistrationSummary, AdminAddonRegistrationsResponse,
    AdminAddonResourceCallDiagnosticRequest, AdminAddonResourceCallDiagnosticResponse,
    AdminAddonResourceCallDiagnosticStatus, AdminAddonResourceLinkCheckRequest,
    AdminAddonResourceLinkCheckResponse, AdminAddonResourceSearchDiagnosticRequest,
    AdminAddonResourceSearchDiagnosticResponse, AdminAddonResourceSearchLinkSummary,
    AdminAddonResourceSearchProviderDiagnostic, AdminAddonResourceSearchRequest,
    AdminAddonResourceSearchResponse, AdminAddonResourceSearchResultSummary,
    AdminAddonResourceSearchSelectionRequest, AdminAddonResourceSearchSelectionResponse,
    AdminAddonRoutingPlansResponse, AdminAddonRuntimeReadinessCheck,
    AdminAddonRuntimeReadinessCheckName, AdminAddonRuntimeReadinessDiagnostics,
    AdminAddonRuntimeReadinessReason, AdminAddonRuntimeReadinessResponse,
    AdminAddonSecretReferenceFieldSurface, AdminAddonSourceCatalogEntriesResponse,
    AdminAddonSourceCatalogEntry, AdminAddonSourceCatalogResolveResponse,
    AdminAddonSourceCatalogSource, AdminAddonSourceCatalogSourceKind,
    AdminAddonSourceCatalogSourcesResponse, AdminAddonSurfacesResponse, AdminAddonTaskSurface,
    IssueAddonTokenRequest, RegisterAddonRequest, ReplaceAddonGrantsRequest,
    UpdateAddonStatusRequest,
};
use nako_core::{
    AddonGrantRecord, AddonId, AddonIssuedToken, AddonManifestFingerprint, AddonRegistrationRecord,
    AddonRepository, AddonRoutingDeclarationKind, AddonRoutingPlanId, AddonRoutingPlanStatus,
    AddonRoutingPlanTarget, AddonStatus, AddonTokenId, DomainEventKind, JobKind, NakoError,
    NewAddonGrant, NewAddonRegistration, NewAddonRoutingPlan, NewAddonToken, Result, SecretString,
};
use nako_db::NakoDatabase;
use nako_official_addon_catalog::{chromecast_renderer, metadata_scraper, notification_bridge};
use tokio::sync::{Mutex, Semaphore};

use crate::app::acquisition_intake::{
    AcquisitionIntakeCandidateDiagnostic, RecordResourceSearchSelectionRequest,
};

use super::{runtime::RuntimeSupervisor, storage::StorageBackendRegistry};

mod artwork_write;
mod event_runtime;
mod intake;
mod library_file_write;
mod metadata_write;
mod principal;
mod runtime;
mod scan_metadata;
mod side_effect_apply;
mod target;
mod task_runtime;

use principal::{normalize_grants, normalize_token_label};
pub(crate) use scan_metadata::{
    ScanAddonBulkMetadataScrapeRequest, ScanAddonBulkMetadataScrapeSummary,
};

const RESOURCE_SEARCH_DIAGNOSTIC_DEFAULT_LIMIT: usize = 20;
const RESOURCE_SEARCH_DIAGNOSTIC_MAX_LIMIT: usize = 50;
const RESOURCE_SEARCH_SESSION_TTL_MS: i64 = 15 * 60 * 1_000;
const RESOURCE_SEARCH_SESSION_MAX_COUNT: usize = 64;

#[derive(Clone, Debug)]
struct ResourceSearchSession {
    search_id: String,
    addon_id: AddonId,
    manifest_id: String,
    query: String,
    created_at_ms: i64,
    expires_at_ms: i64,
    selections: HashMap<String, ResourceSearchSelection>,
}

#[derive(Clone, Debug)]
struct ResourceSearchSelection {
    result: AddonResourceSearchResult,
    selected_link: AddonResourceLink,
}

#[derive(Clone, Debug)]
struct ResourceSearchSelectionHandoff {
    manifest_id: String,
    query: String,
    selection: ResourceSearchSelection,
}

#[derive(Debug, Default)]
struct ResourceSearchSessionStore {
    sessions: HashMap<String, ResourceSearchSession>,
}

impl ResourceSearchSessionStore {
    fn insert(&mut self, session: ResourceSearchSession) {
        self.prune(session.created_at_ms);
        self.sessions.insert(session.search_id.clone(), session);
        self.enforce_max_count();
    }

    fn get_selection(
        &mut self,
        addon_id: AddonId,
        search_id: &str,
        selection_id: &str,
        now_ms: i64,
    ) -> Option<ResourceSearchSelectionHandoff> {
        self.prune(now_ms);
        let session = self.sessions.get(search_id)?;
        if session.addon_id != addon_id {
            return None;
        }
        let selection = session.selections.get(selection_id)?.clone();

        Some(ResourceSearchSelectionHandoff {
            manifest_id: session.manifest_id.clone(),
            query: session.query.clone(),
            selection,
        })
    }

    fn prune(&mut self, now_ms: i64) {
        self.sessions
            .retain(|_, session| session.expires_at_ms > now_ms);
    }

    fn enforce_max_count(&mut self) {
        while self.sessions.len() > RESOURCE_SEARCH_SESSION_MAX_COUNT {
            let Some(oldest_search_id) = self
                .sessions
                .iter()
                .min_by_key(|(_, session)| session.created_at_ms)
                .map(|(search_id, _)| search_id.clone())
            else {
                break;
            };
            self.sessions.remove(&oldest_search_id);
        }
    }
}

#[async_trait::async_trait]
trait AddonRegistrationStore: std::fmt::Debug + Send + Sync {
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
    T: AddonRepository + std::fmt::Debug + Send + Sync,
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

#[derive(Clone, Debug)]
pub(crate) struct AddonAppService {
    store: NakoDatabase,
    registration_store: Arc<dyn AddonRegistrationStore>,
    resource_search_sessions: Arc<Mutex<ResourceSearchSessionStore>>,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
}

impl AddonAppService {
    pub(super) fn new(
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Self {
        Self {
            registration_store: Arc::new(store.clone()),
            resource_search_sessions: Arc::new(Mutex::new(ResourceSearchSessionStore::default())),
            store,
            permits,
            storage_backends,
            runtime,
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
        let outbound_task_dispatch_secret_env = normalize_optional_secret_env(
            request.outbound_task_dispatch_secret_env,
            "outbound_task_dispatch_secret_env",
        )?;
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

    pub fn list_addon_source_catalog_sources(
        &self,
    ) -> Result<AdminAddonSourceCatalogSourcesResponse> {
        let entries = builtin_addon_catalog_entries()?;
        Ok(AdminAddonSourceCatalogSourcesResponse {
            sources: vec![AdminAddonSourceCatalogSource {
                id: "nako-official".to_owned(),
                name: "Nako Official Addons".to_owned(),
                description: Some(
                    "Built-in source for official Addon Sidecars published for the current alpha"
                        .to_owned(),
                ),
                kind: AdminAddonSourceCatalogSourceKind::BuiltinOfficial,
                entry_count: entries.len(),
                provides_package_signing: false,
                provides_process_supervision: false,
                provides_provider_breadth: false,
            }],
        })
    }

    pub fn list_addon_source_catalog_entries(
        &self,
    ) -> Result<AdminAddonSourceCatalogEntriesResponse> {
        let entries = builtin_addon_catalog_entries()?;
        Ok(AdminAddonSourceCatalogEntriesResponse {
            source_id: "nako-official".to_owned(),
            entries,
        })
    }

    pub fn resolve_addon_source_catalog_entry(
        &self,
        entry_id: &str,
    ) -> Result<AdminAddonSourceCatalogResolveResponse> {
        let descriptor = builtin_addon_catalog_descriptor(entry_id)?;
        let entry = addon_catalog_entry_from_descriptor("nako-official", entry_id, &descriptor);
        validate_install_descriptor(&descriptor).map_err(|_err| NakoError::InvalidInput {
            message: "invalid addon catalog install descriptor".to_owned(),
        })?;
        let install_guide = protocol_addon_install_guide(&descriptor);

        Ok(AdminAddonSourceCatalogResolveResponse {
            source_id: "nako-official".to_owned(),
            entry,
            descriptor,
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

    pub async fn diagnose_addon_resource_search(
        &self,
        addon_id: AddonId,
        request: AdminAddonResourceSearchDiagnosticRequest,
    ) -> Result<AdminAddonResourceSearchDiagnosticResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let query = request.query.trim().to_owned();
        if query.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "resource search query cannot be empty".to_owned(),
            });
        }
        let limit = normalize_resource_search_diagnostic_limit(request.limit)?;
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let search_request = AddonResourceSearchRequest {
            schema: ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned(),
            intent: request.intent,
            query,
            limit: Some(limit),
            sources: request.sources,
            link_types: request.link_types,
            refresh: request.refresh,
            context: request.context,
        };
        let started = Instant::now();
        let response = call_addon_resource_search_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-resource-search-{addon_id}"),
            search_request,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => {
                let response = outcome.response;
                let link_count = response
                    .results
                    .iter()
                    .map(|result| result.links.len())
                    .sum();
                let merged_link_count = response.merged_by_type.values().map(Vec::len).sum();
                Ok(AdminAddonResourceSearchDiagnosticResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    limit,
                    total: response.total,
                    result_count: response.results.len(),
                    link_count,
                    merged_link_count,
                    provider_executions: response
                        .provider_executions
                        .into_iter()
                        .map(AdminAddonResourceSearchProviderDiagnostic::from)
                        .collect(),
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceSearchDiagnosticResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    limit,
                    total: 0,
                    result_count: 0,
                    link_count: 0,
                    merged_link_count: 0,
                    provider_executions: Vec::new(),
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }

    pub async fn search_addon_resources(
        &self,
        addon_id: AddonId,
        request: AdminAddonResourceSearchRequest,
    ) -> Result<AdminAddonResourceSearchResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let query = request.query.trim().to_owned();
        if query.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "resource search query cannot be empty".to_owned(),
            });
        }
        let limit = normalize_resource_search_diagnostic_limit(request.limit)?;
        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let search_id = new_resource_search_id();
        let search_request = AddonResourceSearchRequest {
            schema: ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned(),
            intent: request.intent,
            query: query.clone(),
            limit: Some(limit),
            sources: request.sources,
            link_types: request.link_types,
            refresh: request.refresh,
            context: request.context,
        };
        let started = Instant::now();
        let response = call_addon_resource_search_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-resource-search-{addon_id}"),
            search_request,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => {
                let response = outcome.response;
                let total = response.total;
                let provider_executions = response
                    .provider_executions
                    .into_iter()
                    .map(AdminAddonResourceSearchProviderDiagnostic::from)
                    .collect();
                let (results, selections) =
                    safe_resource_search_results(&search_id, response.results);
                let result_count = results.len();
                let now_ms = super::current_time_ms()?;
                self.resource_search_sessions
                    .lock()
                    .await
                    .insert(ResourceSearchSession {
                        search_id: search_id.clone(),
                        addon_id,
                        manifest_id: addon.manifest_id.clone(),
                        query,
                        created_at_ms: now_ms,
                        expires_at_ms: now_ms.saturating_add(RESOURCE_SEARCH_SESSION_TTL_MS),
                        selections,
                    });

                Ok(AdminAddonResourceSearchResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    search_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    limit,
                    total,
                    result_count,
                    results,
                    provider_executions,
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceSearchResponse {
                    addon_id,
                    manifest_id: addon.manifest_id,
                    search_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    limit,
                    total: 0,
                    result_count: 0,
                    results: Vec::new(),
                    provider_executions: Vec::new(),
                    http_status: err.http_status(),
                    safe_error_code: Some(safe_resource_diagnostic_error_code(&err).to_owned()),
                })
            }
        }
    }

    pub async fn select_addon_resource_search_result(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonResourceSearchSelectionRequest,
    ) -> Result<AdminAddonResourceSearchSelectionResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let handoff = self
            .resource_search_sessions
            .lock()
            .await
            .get_selection(
                addon_id,
                &search_id,
                &selection_id,
                super::current_time_ms()?,
            )
            .ok_or_else(|| NakoError::NotFound {
                entity: "resource_search_selection",
                id: selection_id.clone(),
            })?;
        if handoff.manifest_id != addon.manifest_id {
            return Err(NakoError::Conflict {
                message: "resource search session belongs to a different addon manifest".to_owned(),
            });
        }

        let diagnostic =
            crate::app::acquisition_intake::AcquisitionIntakeAppService::new_with_storage(
                self.store.clone(),
                self.storage_backends.clone(),
            )
            .record_resource_search_selection(RecordResourceSearchSelectionRequest {
                target_library_id: request.target_library_id,
                addon_id,
                manifest_id: handoff.manifest_id.clone(),
                query: handoff.query,
                result: handoff.selection.result,
                selected_link: handoff.selection.selected_link,
            })
            .await?;

        Ok(AdminAddonResourceSearchSelectionResponse {
            addon_id,
            manifest_id: handoff.manifest_id,
            search_id,
            selection_id,
            candidate: addon_acquisition_candidate_summary(diagnostic.candidate),
            idempotent_replay: diagnostic.idempotent_replay,
        })
    }

    pub async fn check_addon_resource_search_selection_link(
        &self,
        addon_id: AddonId,
        search_id: String,
        selection_id: String,
        request: AdminAddonResourceLinkCheckRequest,
    ) -> Result<AdminAddonResourceLinkCheckResponse> {
        let addon = self.get_addon_registration_or_not_found(addon_id).await?;
        if addon.status == AddonStatus::Unregistered {
            return Err(NakoError::Conflict {
                message: format!("addon registration {addon_id} is unregistered"),
            });
        }
        let handoff = self
            .resource_search_sessions
            .lock()
            .await
            .get_selection(
                addon_id,
                &search_id,
                &selection_id,
                super::current_time_ms()?,
            )
            .ok_or_else(|| NakoError::NotFound {
                entity: "resource_search_selection",
                id: selection_id.clone(),
            })?;
        if handoff.manifest_id != addon.manifest_id {
            return Err(NakoError::Conflict {
                message: "resource search session belongs to a different addon manifest".to_owned(),
            });
        }

        let manifest = self.stored_manifest(&addon)?;
        let granted_scopes = stored_granted_scopes(&addon)?;
        let selected_link = handoff.selection.selected_link.clone();
        let link_type = selected_link.link_type;
        let link_check_request = AddonResourceLinkCheckRequest {
            schema: ADDON_RESOURCE_LINK_CHECK_REQUEST_SCHEMA.to_owned(),
            link: selected_link,
            refresh: request.refresh,
            context: resource_link_check_selection_context(&search_id, &selection_id, &handoff),
        };
        let started = Instant::now();
        let response = call_addon_resource_link_check_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            format!("addon-resource-link-check-{addon_id}"),
            link_check_request,
            None,
        )
        .await;
        let latency_ms = started.elapsed().as_millis();

        match response {
            Ok(outcome) => {
                let response = outcome.response;
                if response.link_type != link_type {
                    return Ok(AdminAddonResourceLinkCheckResponse {
                        addon_id,
                        manifest_id: handoff.manifest_id,
                        search_id,
                        selection_id,
                        status: AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch,
                        latency_ms,
                        attempts: outcome.attempts,
                        link_type,
                        check_status: None,
                        checked_at_ms: None,
                        requires_password: None,
                        retryable: None,
                        retry_after_ms: None,
                        has_safe_message: false,
                        safe_facts: Default::default(),
                        http_status: Some(outcome.http_status),
                        safe_error_code: Some("link_type_mismatch".to_owned()),
                    });
                }

                Ok(AdminAddonResourceLinkCheckResponse {
                    addon_id,
                    manifest_id: handoff.manifest_id,
                    search_id,
                    selection_id,
                    status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
                    latency_ms,
                    attempts: outcome.attempts,
                    link_type,
                    check_status: Some(response.status),
                    checked_at_ms: Some(response.checked_at_ms),
                    requires_password: Some(response.requires_password),
                    retryable: Some(response.retryable),
                    retry_after_ms: response.retry_after_ms,
                    has_safe_message: response.safe_message.is_some(),
                    safe_facts: response.safe_facts,
                    http_status: Some(outcome.http_status),
                    safe_error_code: None,
                })
            }
            Err(failure) => {
                let err = failure.error;
                Ok(AdminAddonResourceLinkCheckResponse {
                    addon_id,
                    manifest_id: handoff.manifest_id,
                    search_id,
                    selection_id,
                    status: resource_diagnostic_status_for_client_error(&err),
                    latency_ms,
                    attempts: failure.attempts,
                    link_type,
                    check_status: None,
                    checked_at_ms: None,
                    requires_password: None,
                    retryable: None,
                    retry_after_ms: None,
                    has_safe_message: false,
                    safe_facts: Default::default(),
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

    pub(super) fn stored_manifest(&self, addon: &AddonRegistrationRecord) -> Result<AddonManifest> {
        serde_json::from_str(&addon.manifest_json).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to parse stored addon manifest: {err}"),
        })
    }
}

pub(super) fn ensure_addon_accepts_runtime_authority(
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
        AddonClientError::Protocol(_)
        | AddonClientError::InvalidRequest { .. }
        | AddonClientError::InvalidResponse { .. }
        | AddonClientError::UnsafeRequestBody => AdminAddonHealthCheckStatus::ProtocolMismatch,
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
        AddonClientError::InvalidRequest { .. }
        | AddonClientError::InvalidResponse { .. }
        | AddonClientError::UnsafeRequestBody => AdminAddonRuntimeReadinessCheckName::Safety,
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
        AddonClientError::InvalidRequest { .. } => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "invalid_request",
        ),
        AddonClientError::InvalidResponse { .. } => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "invalid_response",
        ),
        AddonClientError::UnsafeRequestBody => (
            AdminAddonRuntimeReadinessReason::UnsafeResponse,
            "unsafe_request_body",
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
        AddonClientError::InvalidRequest { .. } => "invalid_request",
        AddonClientError::InvalidResponse { .. } => "invalid_response",
        AddonClientError::UnsafeRequestBody => "unsafe_request_body",
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

fn normalize_resource_search_diagnostic_limit(limit: Option<usize>) -> Result<usize> {
    let limit = limit.unwrap_or(RESOURCE_SEARCH_DIAGNOSTIC_DEFAULT_LIMIT);
    if limit == 0 {
        return Err(NakoError::InvalidInput {
            message: "resource search limit must be greater than zero".to_owned(),
        });
    }

    Ok(limit.min(RESOURCE_SEARCH_DIAGNOSTIC_MAX_LIMIT))
}

fn safe_resource_search_results(
    search_id: &str,
    results: Vec<AddonResourceSearchResult>,
) -> (
    Vec<AdminAddonResourceSearchResultSummary>,
    HashMap<String, ResourceSearchSelection>,
) {
    let mut summaries = Vec::with_capacity(results.len());
    let mut selections = HashMap::new();

    for (result_index, result) in results.into_iter().enumerate() {
        let mut links = Vec::new();
        for (link_index, link) in result.links.iter().enumerate() {
            let Some(source_uri) = resource_search_link_uri(link) else {
                continue;
            };
            let selection_id =
                resource_search_selection_id(search_id, result_index, link_index, &source_uri);
            links.push(AdminAddonResourceSearchLinkSummary {
                selection_id: selection_id.clone(),
                link_type: link.link_type,
                source: link.source.clone(),
                source_ref_redacted: redact_uri(&source_uri),
                has_password: link.password.is_some(),
                has_note: link
                    .note
                    .as_ref()
                    .is_some_and(|note| !note.trim().is_empty()),
            });
            selections.insert(
                selection_id,
                ResourceSearchSelection {
                    result: resource_search_selection_result_snapshot(&result),
                    selected_link: link.clone(),
                },
            );
        }

        summaries.push(AdminAddonResourceSearchResultSummary {
            result_ref_fingerprint: fingerprint_key(&result.id),
            title: result.title,
            content: optional_non_empty(result.content),
            source: result.source,
            tags: result
                .tags
                .into_iter()
                .filter_map(|tag| optional_non_empty(Some(tag)))
                .collect(),
            score: result.score,
            links,
        });
    }

    (summaries, selections)
}

fn resource_search_link_uri(link: &AddonResourceLink) -> Option<String> {
    optional_non_empty(Some(link.normalized_url.clone()))
        .or_else(|| optional_non_empty(Some(link.url.clone())))
}

fn resource_search_selection_result_snapshot(
    result: &AddonResourceSearchResult,
) -> AddonResourceSearchResult {
    AddonResourceSearchResult {
        id: result.id.clone(),
        title: result.title.clone(),
        source: result.source.clone(),
        content: result.content.clone(),
        links: result
            .links
            .iter()
            .map(resource_search_link_count_placeholder)
            .collect(),
        tags: result.tags.clone(),
        images: result.images.iter().map(|_| String::new()).collect(),
        score: result.score,
    }
}

fn resource_search_link_count_placeholder(link: &AddonResourceLink) -> AddonResourceLink {
    AddonResourceLink {
        url: String::new(),
        normalized_url: String::new(),
        link_type: link.link_type,
        source: link.source.clone(),
        password: None,
        note: None,
    }
}

fn resource_search_selection_id(
    search_id: &str,
    result_index: usize,
    link_index: usize,
    source_uri: &str,
) -> String {
    let material = format!(
        "nako.resource-search-selection-id.v1\0{search_id}\0{result_index}\0{link_index}\0{source_uri}"
    );
    format!("sel_{}", &sha256_hex(&material)[..32])
}

fn resource_link_check_selection_context(
    search_id: &str,
    selection_id: &str,
    handoff: &ResourceSearchSelectionHandoff,
) -> serde_json::Value {
    let source_ref_redacted = resource_search_link_uri(&handoff.selection.selected_link)
        .map(|source_uri| redact_uri(&source_uri));

    serde_json::json!({
        "schema": "nako.resource_link_check.selection_context.v1",
        "search_id": search_id,
        "selection_id": selection_id,
        "query_fingerprint": fingerprint_key(&handoff.query),
        "result_ref_fingerprint": fingerprint_key(&handoff.selection.result.id),
        "link_type": handoff.selection.selected_link.link_type.as_str(),
        "source_ref_redacted": source_ref_redacted,
    })
}

fn new_resource_search_id() -> String {
    format!("rs_{}", uuid::Uuid::new_v4().simple())
}

fn optional_non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
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

fn fingerprint_key(value: &str) -> String {
    let digest = sha256_hex(value);
    format!("sha256:{}", &digest[..32])
}

fn sha256_hex(value: &str) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(value.as_bytes());
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn addon_acquisition_candidate_summary(
    diagnostic: AcquisitionIntakeCandidateDiagnostic,
) -> AddonAcquisitionCandidateSummary {
    AddonAcquisitionCandidateSummary {
        id: diagnostic.id,
        target_library_id: diagnostic.target_library_id,
        state: diagnostic.state,
        source_kind: diagnostic.source_kind,
        source_scheme: diagnostic.source_scheme,
        source_ref_redacted: diagnostic.source_uri_redacted,
        source_key_fingerprint: diagnostic.source_key_fingerprint,
        has_display_name: diagnostic.has_display_name,
        has_intended_locator: diagnostic.has_intended_locator,
        size_bytes: diagnostic.size_bytes,
        has_fingerprint: diagnostic.has_fingerprint,
        has_diagnostics: diagnostic.has_diagnostics,
        managed_import_artifact_id: diagnostic.managed_import_artifact_id,
        writes_library: false,
        creates_media_source: false,
        creates_managed_import: false,
        promotion_apply: false,
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
        AddonClientError::InvalidRequest { .. } => {
            AdminAddonResourceCallDiagnosticStatus::ProtocolMismatch
        }
        AddonClientError::InvalidResponse { .. } | AddonClientError::UnsafeRequestBody => {
            AdminAddonResourceCallDiagnosticStatus::UnsafeResponse
        }
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
        AddonClientError::InvalidRequest { .. } => "invalid_request",
        AddonClientError::InvalidResponse { .. } => "invalid_response",
        AddonClientError::UnsafeRequestBody => "unsafe_request_body",
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

pub(super) fn declaration_scopes_granted(required: &[AddonScope], granted: &[AddonScope]) -> bool {
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

fn builtin_addon_catalog_entries() -> Result<Vec<AdminAddonSourceCatalogEntry>> {
    let descriptors = vec![
        (
            metadata_scraper::ADDON_ID,
            official_metadata_scraper_descriptor(),
        ),
        (
            notification_bridge::ADDON_ID,
            official_notification_bridge_descriptor(),
        ),
        (
            chromecast_renderer::ADDON_ID,
            official_chromecast_renderer_descriptor(),
        ),
    ];
    for (_, descriptor) in &descriptors {
        validate_install_descriptor(descriptor).map_err(|_err| NakoError::InvalidInput {
            message: "invalid built-in addon catalog descriptor".to_owned(),
        })?;
    }

    Ok(descriptors
        .into_iter()
        .map(|(entry_id, descriptor)| {
            addon_catalog_entry_from_descriptor("nako-official", entry_id, &descriptor)
        })
        .collect())
}

fn builtin_addon_catalog_descriptor(entry_id: &str) -> Result<AddonInstallDescriptor> {
    match entry_id {
        metadata_scraper::ADDON_ID => Ok(official_metadata_scraper_descriptor()),
        notification_bridge::ADDON_ID => Ok(official_notification_bridge_descriptor()),
        chromecast_renderer::ADDON_ID => Ok(official_chromecast_renderer_descriptor()),
        _ => Err(NakoError::NotFound {
            entity: "addon_catalog_entry",
            id: entry_id.to_owned(),
        }),
    }
}

fn addon_catalog_entry_from_descriptor(
    source_id: &str,
    entry_id: &str,
    descriptor: &AddonInstallDescriptor,
) -> AdminAddonSourceCatalogEntry {
    AdminAddonSourceCatalogEntry {
        source_id: source_id.to_owned(),
        entry_id: entry_id.to_owned(),
        manifest_id: descriptor.manifest.id.clone(),
        addon_name: descriptor.manifest.name.clone(),
        addon_version: descriptor.manifest.version.clone(),
        protocol_version: descriptor.manifest.protocol_version.clone(),
        description: descriptor.manifest.description.clone(),
        runtime_kind: descriptor.runtime.kind,
        resources: descriptor
            .manifest
            .resources
            .iter()
            .map(|resource| resource.kind)
            .collect(),
        scopes: descriptor.manifest.scopes.clone(),
        tasks: descriptor
            .manifest
            .tasks
            .iter()
            .map(|task| task.id.clone())
            .collect(),
        package_signing_verified: false,
        lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary {
            nako_manages_containers: false,
            nako_manages_processes: false,
            nako_manages_packages: false,
            message: "The catalog resolves install metadata only. Operators still own package installation, sidecar process lifecycle, update execution, logs, and rollback outside Nako.".to_owned(),
        },
    }
}

fn official_metadata_scraper_descriptor() -> AddonInstallDescriptor {
    metadata_scraper::container_install_descriptor()
}

fn official_notification_bridge_descriptor() -> AddonInstallDescriptor {
    notification_bridge::container_install_descriptor()
}

fn official_chromecast_renderer_descriptor() -> AddonInstallDescriptor {
    chromecast_renderer::container_install_descriptor()
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

fn normalize_optional_secret_env(
    value: Option<String>,
    field_name: &'static str,
) -> Result<Option<String>> {
    match value {
        Some(value) => {
            let value = value.trim().to_owned();
            if value.is_empty() {
                return Err(NakoError::InvalidInput {
                    message: format!("{field_name} cannot be empty"),
                });
            }
            if !is_valid_environment_name(&value) {
                return Err(NakoError::InvalidInput {
                    message: format!("{field_name} must be a valid environment variable name"),
                });
            }

            Ok(Some(value))
        }
        None => Ok(None),
    }
}

pub(super) fn resolve_outbound_task_dispatch_secret(
    addon: &AddonRegistrationRecord,
) -> Result<Option<SecretString>> {
    resolve_outbound_task_dispatch_secret_with(addon, resolve_outbound_task_dispatch_secret_env)
}

fn resolve_outbound_task_dispatch_secret_env(
    secret_env: &str,
) -> std::result::Result<String, String> {
    #[cfg(test)]
    if let Some(secret) = test_outbound_task_dispatch_secret(secret_env) {
        return Ok(secret);
    }

    env::var(secret_env).map_err(|err| err.to_string())
}

fn resolve_outbound_task_dispatch_secret_with<F, E>(
    addon: &AddonRegistrationRecord,
    resolve_env: F,
) -> Result<Option<SecretString>>
where
    F: FnOnce(&str) -> std::result::Result<String, E>,
    E: std::fmt::Display,
{
    let Some(secret_env) = addon.outbound_task_dispatch_secret_env.as_deref() else {
        return Ok(None);
    };
    let secret = resolve_env(secret_env).map_err(|err| NakoError::InvalidInput {
        message: format!(
            "addon {} references unavailable outbound task-dispatch secret environment variable {secret_env}: {err}",
            addon.id
        ),
    })?;
    if secret.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!(
                "addon {} outbound task-dispatch secret environment variable {secret_env} is empty",
                addon.id
            ),
        });
    }

    Ok(Some(SecretString::new(secret)))
}

fn is_valid_environment_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[cfg(test)]
static TEST_OUTBOUND_TASK_DISPATCH_SECRETS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, String>>,
> = std::sync::OnceLock::new();

#[cfg(test)]
pub(crate) struct TestOutboundTaskDispatchSecretGuard {
    name: String,
}

#[cfg(test)]
pub(crate) fn set_test_outbound_task_dispatch_secret(
    name: &str,
    value: &str,
) -> TestOutboundTaskDispatchSecretGuard {
    let secrets = TEST_OUTBOUND_TASK_DISPATCH_SECRETS
        .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    secrets
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(name.to_owned(), value.to_owned());
    TestOutboundTaskDispatchSecretGuard {
        name: name.to_owned(),
    }
}

#[cfg(test)]
impl Drop for TestOutboundTaskDispatchSecretGuard {
    fn drop(&mut self) {
        if let Some(secrets) = TEST_OUTBOUND_TASK_DISPATCH_SECRETS.get() {
            secrets
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&self.name);
        }
    }
}

#[cfg(test)]
fn test_outbound_task_dispatch_secret(name: &str) -> Option<String> {
    TEST_OUTBOUND_TASK_DISPATCH_SECRETS
        .get()
        .and_then(|secrets| {
            secrets
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(name)
                .cloned()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addon_registration(
        outbound_task_dispatch_secret_env: Option<&str>,
    ) -> AddonRegistrationRecord {
        AddonRegistrationRecord {
            id: AddonId::new(),
            manifest_id: "example.metadata".to_owned(),
            name: "Example Metadata".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            outbound_task_dispatch_secret_env: outbound_task_dispatch_secret_env.map(str::to_owned),
            granted_scopes: Vec::new(),
            status: AddonStatus::Enabled,
            created_at: "2026-05-24T00:00:00.000Z".to_owned(),
            updated_at: "2026-05-24T00:00:00.000Z".to_owned(),
        }
    }

    #[test]
    fn official_metadata_scraper_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            official_metadata_scraper_descriptor(),
            metadata_scraper::container_install_descriptor()
        );
    }

    #[test]
    fn official_notification_bridge_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            official_notification_bridge_descriptor(),
            notification_bridge::container_install_descriptor()
        );
    }

    #[test]
    fn official_chromecast_renderer_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            official_chromecast_renderer_descriptor(),
            chromecast_renderer::container_install_descriptor()
        );
    }

    #[test]
    fn resolves_outbound_task_dispatch_secret_from_env_reference() {
        let addon = addon_registration(Some("NAKO_ADDON_DISPATCH_SECRET"));
        let resolved = resolve_outbound_task_dispatch_secret_with(&addon, |name| match name {
            "NAKO_ADDON_DISPATCH_SECRET" => Ok("super-secret".to_owned()),
            other => Err(format!("missing {other}")),
        })
        .unwrap();

        let secret = resolved.expect("expected resolved outbound secret");
        assert_eq!(secret.expose_secret(), "super-secret");
    }

    #[test]
    fn missing_outbound_task_dispatch_secret_reports_safe_error() {
        let addon = addon_registration(Some("NAKO_ADDON_DISPATCH_SECRET"));
        let err = resolve_outbound_task_dispatch_secret_with(&addon, |_name| {
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_err();
        let text = err.to_string();

        assert!(text.contains("NAKO_ADDON_DISPATCH_SECRET"));
        assert!(!text.contains("super-secret"));
    }

    #[test]
    fn missing_reference_returns_none() {
        let addon = addon_registration(None);

        assert_eq!(
            resolve_outbound_task_dispatch_secret_with(&addon, |_name| {
                Ok::<String, String>("unused".to_owned())
            })
            .unwrap(),
            None
        );
    }

    #[test]
    fn rejects_invalid_outbound_task_dispatch_secret_env_names() {
        let err = normalize_optional_secret_env(
            Some("not-a-valid env".to_owned()),
            "outbound_task_dispatch_secret_env",
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("must be a valid environment variable name")
        );
    }
}
