use nako_core::{AuthenticatedPrincipal, NakoError};

use super::error::ApiResult;

pub(super) fn require_administrator(principal: &AuthenticatedPrincipal) -> ApiResult<()> {
    if principal.is_administrator() {
        return Ok(());
    }

    Err(NakoError::Forbidden {
        message: "administrator role is required".to_owned(),
    }
    .into())
}
