use nako_core::{
    AuthenticatedPrincipal, LibraryAccessLevel, LibraryId, MediaItemId, MediaSourceId, NakoError,
    Result, SelectedArtworkId,
};

use crate::app::NakoApp;

use super::error::ApiResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RequiredLibraryAccess {
    Browse,
    Play,
    Manage,
}

impl RequiredLibraryAccess {
    fn allows(self, access: LibraryAccessLevel) -> bool {
        match self {
            Self::Browse => access.allows_browse(),
            Self::Play => access.allows_play(),
            Self::Manage => access.allows_manage(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Browse => "browse",
            Self::Play => "play",
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

pub(super) async fn item_has_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    item_id: MediaItemId,
    required: RequiredLibraryAccess,
) -> ApiResult<bool> {
    let sources = app.list_item_media_sources(item_id).await?;
    if sources.is_empty() {
        return Ok(principal.is_administrator());
    }

    for source in sources {
        if has_library_access(app, principal, source.library_id, required).await? {
            return Ok(true);
        }
    }

    Ok(false)
}

pub(super) async fn require_item_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    item_id: MediaItemId,
    required: RequiredLibraryAccess,
) -> ApiResult<()> {
    if item_has_access(app, principal, item_id, required).await? {
        return Ok(());
    }

    Err(library_access_forbidden(required).into())
}

pub(super) async fn require_source_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    source_id: MediaSourceId,
    required: RequiredLibraryAccess,
) -> ApiResult<()> {
    let source = app
        .get_media_source_record(source_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "media_source",
            id: source_id.to_string(),
        })?;

    require_library_access(app, principal, source.library_id, required).await
}

pub(super) async fn require_selected_artwork_access(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    selected_id: SelectedArtworkId,
    required: RequiredLibraryAccess,
) -> ApiResult<()> {
    let selected = app
        .get_selected_artwork_record(selected_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "selected_artwork",
            id: selected_id.to_string(),
        })?;

    require_library_access(app, principal, selected.library_id, required).await
}

pub(super) fn parse_public_library_id(value: &str) -> Result<LibraryId> {
    value
        .parse::<LibraryId>()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid library id in public response: {err}"),
        })
}

pub(super) fn parse_public_item_id(value: &str) -> Result<MediaItemId> {
    value
        .parse::<MediaItemId>()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid item id in public response: {err}"),
        })
}

pub(super) fn parse_public_source_id(value: &str) -> Result<MediaSourceId> {
    value
        .parse::<MediaSourceId>()
        .map_err(|err| NakoError::InvalidInput {
            message: format!("invalid source id in public response: {err}"),
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
