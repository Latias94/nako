use nako_core::{AuthenticatedPrincipal, LibraryAccessLevel, LibraryId, NakoError, Result};

use crate::app::NakoApp;

use super::error::ApiResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RequiredLibraryAccess {
    Browse,
    Manage,
}

impl RequiredLibraryAccess {
    fn allows(self, access: LibraryAccessLevel) -> bool {
        match self {
            Self::Browse => access.allows_browse(),
            Self::Manage => access.allows_manage(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Browse => "browse",
            Self::Manage => "manage",
        }
    }
}

pub(super) async fn has_library_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    library_id: LibraryId,
    required: RequiredLibraryAccess,
) -> ApiResult<bool> {
    let effective = app
        .resolve_effective_library_access(principal.user_id, library_id)
        .await?;

    Ok(required.allows(effective.access))
}

pub(super) async fn require_library_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    library_id: LibraryId,
    required: RequiredLibraryAccess,
) -> ApiResult<()> {
    if has_library_access(app, principal, library_id, required).await? {
        return Ok(());
    }

    Err(library_access_forbidden(required).into())
}

pub(super) fn require_administrator(principal: &AuthenticatedPrincipal) -> ApiResult<()> {
    if principal.is_administrator() {
        return Ok(());
    }

    Err(NakoError::Forbidden {
        message: "administrator role is required".to_owned(),
    }
    .into())
}

pub(super) fn parse_public_library_id(value: &str) -> Result<LibraryId> {
    value
        .parse::<LibraryId>()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid library id in public response: {err}"),
        })
}

pub(super) fn page_returned_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn library_access_forbidden(required: RequiredLibraryAccess) -> NakoError {
    NakoError::Forbidden {
        message: format!(
            "required Library Access level '{}' is not available",
            required.label()
        ),
    }
}
