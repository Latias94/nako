use std::collections::HashSet;

use taru_api::extension::{
    AddonAccessCheckRequest, AddonAccessCheckResponse, AddonGrantAssignment,
};
use taru_core::{
    AddonGrantId, AddonPermission, AddonPrincipal, AddonRepository, AddonTokenStatus, LibraryId,
    NewAddonGrant, Result, TaruError, hash_addon_token,
};

use super::AddonAppService;

impl AddonAppService {
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

        if addon.status != taru_core::AddonStatus::Enabled {
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
}

pub(super) fn normalize_token_label(label: Option<&str>) -> Result<String> {
    let label = label.unwrap_or("default").trim();
    if label.is_empty() {
        return Err(TaruError::InvalidInput {
            message: "addon token label must not be empty".to_owned(),
        });
    }

    Ok(label.to_owned())
}

pub(super) fn normalize_grants(
    addon_id: taru_core::AddonId,
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
