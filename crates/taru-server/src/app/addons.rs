use std::collections::HashSet;

use taru_addon_protocol::{ensure_scope_grant, validate_manifest};
use taru_api::{
    AddonGrantAssignment, AddonGrantsResponse, AddonRegistrationResponse,
    AddonRegistrationsResponse, AddonTokenIssuedResponse, AddonTokenResponse,
    AddonTokenRotationResponse, AddonTokenSummary, AddonTokensResponse, IssueAddonTokenRequest,
    RegisterAddonRequest, ReplaceAddonGrantsRequest,
};
use taru_core::{
    AddonGrantId, AddonId, AddonIssuedToken, AddonRegistrationRecord, AddonRepository, AddonStatus,
    AddonTokenId, NewAddonGrant, NewAddonRegistration, NewAddonToken, Result, TaruError,
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

fn normalize_token_label(label: Option<&str>) -> Result<String> {
    let label = label.unwrap_or("default").trim();
    if label.is_empty() {
        return Err(TaruError::InvalidInput {
            message: "addon token label must not be empty".to_owned(),
        });
    }

    Ok(label.to_owned())
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
