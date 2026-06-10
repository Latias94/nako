use nako_core::{AuthenticatedPrincipal, IdentityAccessRepository, LibraryId, NakoError, Result};
use nako_db::NakoDatabase;

pub(crate) async fn ensure_library_manage_access(
    store: &NakoDatabase,
    principal: &AuthenticatedPrincipal,
    library_id: LibraryId,
) -> Result<()> {
    let effective = store
        .resolve_effective_library_access(principal.user_id, library_id)
        .await?;

    if effective.access.allows_manage() {
        Ok(())
    } else {
        Err(library_manage_access_forbidden())
    }
}

fn library_manage_access_forbidden() -> NakoError {
    NakoError::Forbidden {
        message: "required Library Access level 'manage' is not available".to_owned(),
    }
}
