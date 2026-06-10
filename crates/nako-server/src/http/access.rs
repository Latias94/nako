use nako_core::{AuthenticatedPrincipal, LibraryId, NakoError};

use crate::app::NakoApp;

use super::error::ApiResult;

pub(super) async fn require_library_manage_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    library_id: LibraryId,
) -> ApiResult<()> {
    let effective = app
        .resolve_effective_library_access(principal.user_id, library_id)
        .await?;

    if effective.access.allows_manage() {
        return Ok(());
    }

    Err(library_access_forbidden("manage").into())
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

fn library_access_forbidden(required: &'static str) -> NakoError {
    NakoError::Forbidden {
        message: format!(
            "required Library Access level '{}' is not available",
            required
        ),
    }
}
