use std::{collections::HashSet, env, sync::Arc};

use nako_addon_protocol::{AddonManifest, AddonScope, ensure_scope_grant, validate_manifest};
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
    Result, SecretString,
};
use nako_db::NakoDatabase;
use tokio::sync::{Mutex, Semaphore};

use super::{runtime::RuntimeSupervisor, storage::StorageBackendRegistry};

mod artwork_write;
mod catalog;
mod diagnostics;
mod event_runtime;
mod intake;
mod library_file_write;
mod metadata_write;
mod principal;
mod resource_search;
mod routing;
mod runtime;
mod scan_metadata;
mod side_effect_apply;
mod subtitles;
mod surfaces;
mod target;
mod task_runtime;

use principal::{normalize_grants, normalize_token_label};
use routing::declaration_scopes_granted;
pub(crate) use scan_metadata::{
    ScanAddonBulkMetadataScrapeRequest, ScanAddonBulkMetadataScrapeSummary,
};

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
    resource_search_sessions: Arc<Mutex<resource_search::ResourceSearchSessionStore>>,
    subtitle_search_sessions: Arc<Mutex<subtitles::SubtitleSearchSessionStore>>,
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
            resource_search_sessions: Arc::new(Mutex::new(
                resource_search::ResourceSearchSessionStore::default(),
            )),
            subtitle_search_sessions: Arc::new(Mutex::new(
                subtitles::SubtitleSearchSessionStore::default(),
            )),
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

fn addon_surface_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
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
            catalog::official_metadata_scraper_descriptor(),
            nako_official_addon_catalog::metadata_scraper::container_install_descriptor()
        );
    }

    #[test]
    fn official_notification_bridge_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            catalog::official_notification_bridge_descriptor(),
            nako_official_addon_catalog::notification_bridge::container_install_descriptor()
        );
    }

    #[test]
    fn official_chromecast_renderer_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            catalog::official_chromecast_renderer_descriptor(),
            nako_official_addon_catalog::chromecast_renderer::container_install_descriptor()
        );
    }

    #[test]
    fn official_resource_search_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            catalog::official_resource_search_descriptor(),
            nako_official_addon_catalog::resource_search::container_install_descriptor()
        );
    }

    #[test]
    fn official_external_acquisition_runner_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            catalog::official_external_acquisition_runner_descriptor(),
            nako_official_addon_catalog::external_acquisition_runner::container_install_descriptor(
            )
        );
    }

    #[test]
    fn official_subtitle_provider_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            catalog::official_subtitle_provider_descriptor(),
            nako_official_addon_catalog::subtitle_provider::container_install_descriptor()
        );
    }

    #[test]
    fn official_dlna_renderer_catalog_descriptor_uses_shared_catalog_facts() {
        assert_eq!(
            catalog::official_dlna_renderer_descriptor(),
            nako_official_addon_catalog::dlna_renderer::container_install_descriptor()
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
