use std::collections::HashSet;

use taru_addon_protocol::{ensure_scope_grant, validate_manifest};
use taru_api::{AddonRegistrationResponse, AddonRegistrationsResponse, RegisterAddonRequest};
use taru_core::{
    AddonId, AddonRegistrationRecord, AddonRepository, AddonStatus, NewAddonRegistration, Result,
    TaruError,
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
}
