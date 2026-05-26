use nako_api::public_client::{
    ClientManagementAction, ClientManagementDisabledReason, ClientManagementHttpMethod,
    ClientManagementRequiredAccess, ClientManagementSurface, ManagementContextDto,
    ManagementContextLinkDto, ManagementContextLinksResponse,
};
use nako_core::{
    AuthenticatedPrincipal, IdentityAccessRepository, LibraryId, LibraryRepository, MediaItemId,
    MediaRepository, MediaSourceId, NakoError, PageRequest, PlaybackSessionId,
    PlaybackSessionRepository, Result,
};
use nako_db::NakoDatabase;

const ROUTE_LIBRARY_SCAN: &str = "library.scan";
const ROUTE_LIBRARY_METADATA_PROFILE: &str = "library.metadata_profile";
const ROUTE_ITEM_METADATA_REFRESH: &str = "item.metadata_refresh";
const ROUTE_JOBS_FILTERED: &str = "jobs.filtered";
const ROUTE_PLAYBACK_SUPPORT: &str = "playback.support";
const ROUTE_PLAYBACK_RUNTIME: &str = "playback.runtime";
const ROUTE_ACCESS_LIBRARY_POLICIES: &str = "access.library_policies";

#[derive(Clone, Debug)]
pub(crate) struct ManagementContextAppService {
    store: NakoDatabase,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ManagementContextRequest {
    pub(crate) library_id: Option<LibraryId>,
    pub(crate) item_id: Option<MediaItemId>,
    pub(crate) source_id: Option<MediaSourceId>,
    pub(crate) playback_session_id: Option<PlaybackSessionId>,
}

#[derive(Clone, Copy, Debug, Default)]
struct ResolvedManagementContext {
    library_id: Option<LibraryId>,
    item_id: Option<MediaItemId>,
    source_id: Option<MediaSourceId>,
    playback_session_id: Option<PlaybackSessionId>,
}

#[derive(Clone, Copy, Debug, Default)]
struct ManagementAuthority {
    can_browse_context: bool,
    can_manage_library_context: bool,
    can_administer_server: bool,
}

impl ManagementContextAppService {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub(crate) async fn context_links(
        &self,
        principal: &AuthenticatedPrincipal,
        request: ManagementContextRequest,
    ) -> Result<ManagementContextLinksResponse> {
        let context = self.resolve_context(request).await?;
        let authority = self.resolve_authority(principal, context).await?;
        if context.has_resource_scope() && !authority.can_browse_context {
            return Err(NakoError::Forbidden {
                message: "required Library Access level 'browse' is not available".to_owned(),
            });
        }

        Ok(ManagementContextLinksResponse {
            context: context.to_dto(),
            links: self.links_for_context(context, authority),
        })
    }

    async fn resolve_context(
        &self,
        request: ManagementContextRequest,
    ) -> Result<ResolvedManagementContext> {
        let mut context = ResolvedManagementContext {
            library_id: request.library_id,
            item_id: request.item_id,
            source_id: request.source_id,
            playback_session_id: request.playback_session_id,
        };

        if let Some(playback_session_id) = request.playback_session_id {
            let session =
                PlaybackSessionRepository::get_playback_session(&self.store, playback_session_id)
                    .await?
                    .ok_or_else(|| NakoError::NotFound {
                        entity: "playback_session",
                        id: playback_session_id.to_string(),
                    })?;
            merge_optional_id(&mut context.source_id, session.source_id, "source_id")?;
            merge_optional_id(&mut context.item_id, session.item_id, "item_id")?;
        }

        if let Some(source_id) = context.source_id {
            let source = MediaRepository::get_media_source(&self.store, source_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_source",
                    id: source_id.to_string(),
                })?;
            merge_optional_id(&mut context.library_id, source.library_id, "library_id")?;
            merge_optional_id(&mut context.item_id, source.item_id, "item_id")?;
        }

        if let Some(item_id) = context.item_id {
            let item = MediaRepository::get_media_item(&self.store, item_id).await?;
            if item.is_none() {
                return Err(NakoError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                });
            }
            if context.library_id.is_none() {
                let sources = MediaRepository::list_item_sources(
                    &self.store,
                    item_id,
                    PageRequest::new(PageRequest::MAX_LIMIT, 0),
                )
                .await?;
                if sources.len() == 1 {
                    context.library_id = Some(sources[0].library_id);
                }
            }
        }

        if let Some(library_id) = context.library_id {
            let library = LibraryRepository::get_library(&self.store, library_id).await?;
            if library.is_none() {
                return Err(NakoError::NotFound {
                    entity: "library",
                    id: library_id.to_string(),
                });
            }
        }

        Ok(context)
    }

    async fn resolve_authority(
        &self,
        principal: &AuthenticatedPrincipal,
        context: ResolvedManagementContext,
    ) -> Result<ManagementAuthority> {
        let mut authority = ManagementAuthority {
            can_browse_context: !context.has_resource_scope(),
            can_manage_library_context: false,
            can_administer_server: principal.is_administrator(),
        };

        if let Some(library_id) = context.library_id {
            let effective = self
                .store
                .resolve_effective_library_access(principal.user_id, library_id)
                .await?;
            authority.can_browse_context = effective.access.allows_browse();
            authority.can_manage_library_context = effective.access.allows_manage();
            return Ok(authority);
        }

        if let Some(item_id) = context.item_id {
            let sources = MediaRepository::list_item_sources(
                &self.store,
                item_id,
                PageRequest::new(PageRequest::MAX_LIMIT, 0),
            )
            .await?;
            if sources.is_empty() {
                authority.can_browse_context = principal.is_administrator();
                authority.can_manage_library_context = principal.is_administrator();
                return Ok(authority);
            }

            for source in sources {
                let effective = self
                    .store
                    .resolve_effective_library_access(principal.user_id, source.library_id)
                    .await?;
                authority.can_browse_context |= effective.access.allows_browse();
                authority.can_manage_library_context |= effective.access.allows_manage();
            }
        }

        Ok(authority)
    }

    fn links_for_context(
        &self,
        context: ResolvedManagementContext,
        authority: ManagementAuthority,
    ) -> Vec<ManagementContextLinkDto> {
        let target = context.to_dto();
        let has_library = context.library_id.is_some();
        let has_item = context.item_id.is_some();
        let has_playback_subject =
            context.source_id.is_some() || context.playback_session_id.is_some();

        vec![
            context_link(
                ROUTE_LIBRARY_SCAN,
                ClientManagementHttpMethod::Post,
                ClientManagementAction::ScanLibrary,
                ClientManagementRequiredAccess::LibraryManage,
                target.clone(),
                has_library,
                authority.can_manage_library_context,
            ),
            context_link(
                ROUTE_LIBRARY_METADATA_PROFILE,
                ClientManagementHttpMethod::Get,
                ClientManagementAction::UpdateLibraryMetadataProfile,
                ClientManagementRequiredAccess::Administrator,
                target.clone(),
                has_library,
                authority.can_administer_server,
            ),
            context_link(
                ROUTE_ITEM_METADATA_REFRESH,
                ClientManagementHttpMethod::Post,
                ClientManagementAction::RefreshItemMetadata,
                ClientManagementRequiredAccess::LibraryManage,
                target.clone(),
                has_item,
                authority.can_manage_library_context,
            ),
            context_link(
                ROUTE_JOBS_FILTERED,
                ClientManagementHttpMethod::Get,
                ClientManagementAction::ViewJobs,
                ClientManagementRequiredAccess::Administrator,
                target.clone(),
                true,
                authority.can_administer_server,
            ),
            context_link(
                ROUTE_PLAYBACK_SUPPORT,
                ClientManagementHttpMethod::Get,
                ClientManagementAction::ViewPlaybackDiagnostics,
                ClientManagementRequiredAccess::Administrator,
                target.clone(),
                has_playback_subject,
                authority.can_administer_server,
            ),
            context_link(
                ROUTE_PLAYBACK_RUNTIME,
                ClientManagementHttpMethod::Get,
                ClientManagementAction::ViewPlaybackRuntime,
                ClientManagementRequiredAccess::Administrator,
                target.clone(),
                true,
                authority.can_administer_server,
            ),
            context_link(
                ROUTE_ACCESS_LIBRARY_POLICIES,
                ClientManagementHttpMethod::Get,
                ClientManagementAction::ManageLibraryAccess,
                ClientManagementRequiredAccess::Administrator,
                target,
                has_library,
                authority.can_administer_server,
            ),
        ]
    }
}

impl ResolvedManagementContext {
    const fn has_resource_scope(self) -> bool {
        self.library_id.is_some()
            || self.item_id.is_some()
            || self.source_id.is_some()
            || self.playback_session_id.is_some()
    }

    fn to_dto(self) -> ManagementContextDto {
        ManagementContextDto {
            library_id: self.library_id.map(|id| id.to_string()),
            item_id: self.item_id.map(|id| id.to_string()),
            source_id: self.source_id.map(|id| id.to_string()),
            playback_session_id: self.playback_session_id.map(|id| id.to_string()),
        }
    }
}

fn context_link(
    route_name: &str,
    method: ClientManagementHttpMethod,
    action: ClientManagementAction,
    required_access: ClientManagementRequiredAccess,
    target: ManagementContextDto,
    has_required_context: bool,
    has_permission: bool,
) -> ManagementContextLinkDto {
    let enabled = has_required_context && has_permission;
    let disabled_reason = if enabled {
        None
    } else if !has_required_context {
        Some(ClientManagementDisabledReason::MissingContext)
    } else {
        Some(ClientManagementDisabledReason::InsufficientPermission)
    };

    ManagementContextLinkDto {
        route_name: route_name.to_owned(),
        method,
        surface: ClientManagementSurface::Management,
        action,
        target,
        enabled,
        required_access,
        disabled_reason,
    }
}

fn merge_optional_id<T>(target: &mut Option<T>, incoming: T, field: &str) -> Result<()>
where
    T: Copy + Eq + ToString,
{
    if let Some(existing) = *target {
        if existing != incoming {
            return Err(NakoError::InvalidInput {
                message: format!("conflicting {field} in management context"),
            });
        }
    } else {
        *target = Some(incoming);
    }

    Ok(())
}
