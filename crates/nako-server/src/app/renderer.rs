use std::collections::HashSet;

use nako_core::{
    MediaItemId, MediaSourceId, NakoError, NewRendererCommand, NewRendererSession, PageRequest,
    PlaybackSessionId, PlaybackTargetKind, PlaybackTargetNetworkScope, PlaybackTargetTransportAuth,
    RendererCommandCompletion, RendererCommandId, RendererCommandRecord, RendererCommandState,
    RendererControlCapabilities, RendererControlCommand, RendererSessionHeartbeat,
    RendererSessionId, RendererSessionListFilter, RendererSessionRecord, RendererSessionRepository,
    RendererSessionState, Result, UserPrincipalId,
};
use nako_db::NakoDatabase;
use nako_playback::ClientPlaybackCapabilities;

use super::current_time_ms;

const DEFAULT_RENDERER_SESSION_TTL_MS: i64 = 60_000;
const MAX_RENDERER_SESSION_TTL_MS: i64 = 10 * 60 * 1_000;
const MAX_RENDERER_DISPLAY_NAME_LEN: usize = 128;

#[derive(Clone, Debug)]
pub(crate) struct RegisterRendererRequest {
    pub principal_id: UserPrincipalId,
    pub display_name: String,
    pub target_kind: PlaybackTargetKind,
    pub network_scope: PlaybackTargetNetworkScope,
    pub transport_auth: PlaybackTargetTransportAuth,
    pub media_capabilities: Option<ClientPlaybackCapabilities>,
    pub control_capabilities: RendererControlCapabilities,
    pub ttl_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct RendererHeartbeatUpdate {
    pub principal_id: UserPrincipalId,
    pub renderer_session_id: RendererSessionId,
    pub state: RendererSessionState,
    pub media_capabilities: Option<ClientPlaybackCapabilities>,
    pub control_capabilities: Option<RendererControlCapabilities>,
    pub ttl_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct QueueRendererCommandRequest {
    pub renderer_session_id: RendererSessionId,
    pub controlling_principal_id: UserPrincipalId,
    pub command: RendererControlCommand,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub playback_session_id: Option<PlaybackSessionId>,
    pub position_ms: Option<u64>,
    pub volume_percent: Option<u8>,
    pub payload_json: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct CompleteRendererCommandRequest {
    pub principal_id: UserPrincipalId,
    pub renderer_session_id: RendererSessionId,
    pub command_id: RendererCommandId,
    pub state: RendererCommandState,
    pub failure_message: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct RendererAppService<S = NakoDatabase> {
    store: S,
}

impl<S> RendererAppService<S>
where
    S: RendererSessionRepository + Clone + Send + Sync + std::fmt::Debug,
{
    pub(crate) fn new(store: S) -> Self {
        Self { store }
    }

    pub(crate) async fn register_renderer(
        &self,
        request: RegisterRendererRequest,
    ) -> Result<RendererSessionRecord> {
        let display_name = normalize_display_name(request.display_name)?;
        validate_nako_renderer_target(request.target_kind, request.transport_auth)?;
        let control_capabilities = normalize_control_capabilities(request.control_capabilities)?;
        let now_ms = current_time_ms()?;
        let expires_at_ms = renderer_expiry(now_ms, request.ttl_ms)?;
        let media_capabilities = request.media_capabilities.unwrap_or_default();

        self.store
            .upsert_renderer_session(NewRendererSession {
                id: RendererSessionId::new(),
                owner_principal_id: request.principal_id,
                target_kind: request.target_kind,
                display_name,
                network_scope: request.network_scope,
                transport_auth: request.transport_auth,
                media_capabilities_json: Some(media_capabilities_json(&media_capabilities)?),
                control_capabilities,
                state: RendererSessionState::Online,
                last_seen_at_ms: now_ms,
                expires_at_ms,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            })
            .await
    }

    pub(crate) async fn list_controllable_renderers(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<RendererSessionRecord>> {
        let now_ms = current_time_ms()?;
        let renderers = self
            .store
            .list_renderer_sessions(
                RendererSessionListFilter {
                    owner_principal_id: Some(principal_id.clone()),
                    state: Some(RendererSessionState::Online),
                },
                page,
            )
            .await?;

        Ok(renderers
            .into_iter()
            .filter(|renderer| !renderer_session_expired(renderer, now_ms))
            .collect())
    }

    pub(crate) async fn list_renderer_sessions_for_admin(
        &self,
        page: PageRequest,
    ) -> Result<Vec<RendererSessionRecord>> {
        self.store
            .list_renderer_sessions(RendererSessionListFilter::default(), page)
            .await
    }

    pub(crate) async fn heartbeat_renderer(
        &self,
        request: RendererHeartbeatUpdate,
    ) -> Result<RendererSessionRecord> {
        if matches!(request.state, RendererSessionState::Revoked) {
            return Err(NakoError::InvalidInput {
                message: "renderer heartbeat cannot revoke a renderer session".to_owned(),
            });
        }

        let existing = self
            .require_renderer_owned_by(&request.principal_id, request.renderer_session_id)
            .await?;
        if matches!(existing.state, RendererSessionState::Revoked) {
            return Err(NakoError::Forbidden {
                message: "renderer session is revoked".to_owned(),
            });
        }

        let now_ms = current_time_ms()?;
        let expires_at_ms = renderer_expiry(now_ms, request.ttl_ms)?;
        if request.media_capabilities.is_some() || request.control_capabilities.is_some() {
            let control_capabilities = request
                .control_capabilities
                .map(normalize_control_capabilities)
                .transpose()?
                .unwrap_or_else(|| existing.control_capabilities.clone());
            let media_capabilities_json = request
                .media_capabilities
                .as_ref()
                .map(media_capabilities_json)
                .transpose()?
                .or(existing.media_capabilities_json);

            return self
                .store
                .upsert_renderer_session(NewRendererSession {
                    id: existing.id,
                    owner_principal_id: existing.owner_principal_id,
                    target_kind: existing.target_kind,
                    display_name: existing.display_name,
                    network_scope: existing.network_scope,
                    transport_auth: existing.transport_auth,
                    media_capabilities_json,
                    control_capabilities,
                    state: request.state,
                    last_seen_at_ms: now_ms,
                    expires_at_ms,
                    created_at_ms: existing.created_at_ms,
                    updated_at_ms: now_ms,
                })
                .await;
        }

        self.store
            .record_renderer_session_heartbeat(RendererSessionHeartbeat {
                id: request.renderer_session_id,
                state: request.state,
                last_seen_at_ms: now_ms,
                expires_at_ms,
            })
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_session",
                id: request.renderer_session_id.to_string(),
            })
    }

    pub(crate) async fn poll_next_command(
        &self,
        principal_id: &UserPrincipalId,
        renderer_session_id: RendererSessionId,
    ) -> Result<Option<RendererCommandRecord>> {
        self.require_online_renderer_owned_by(principal_id, renderer_session_id)
            .await?;

        self.store
            .claim_next_renderer_command(renderer_session_id, current_time_ms()?)
            .await
    }

    pub(crate) async fn get_controllable_renderer(
        &self,
        principal_id: &UserPrincipalId,
        renderer_session_id: RendererSessionId,
    ) -> Result<RendererSessionRecord> {
        self.require_online_renderer_owned_by(principal_id, renderer_session_id)
            .await
    }

    pub(crate) async fn attach_playback_session(
        &self,
        principal_id: &UserPrincipalId,
        renderer_session_id: RendererSessionId,
        playback_session_id: PlaybackSessionId,
    ) -> Result<RendererSessionRecord> {
        self.require_online_renderer_owned_by(principal_id, renderer_session_id)
            .await?;

        self.store
            .attach_renderer_playback_session(
                renderer_session_id,
                Some(playback_session_id),
                current_time_ms()?,
            )
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_session",
                id: renderer_session_id.to_string(),
            })
    }

    pub(crate) async fn queue_renderer_command(
        &self,
        request: QueueRendererCommandRequest,
    ) -> Result<RendererCommandRecord> {
        let renderer = self
            .require_online_renderer(request.renderer_session_id)
            .await?;
        if !renderer.control_capabilities.supports(request.command) {
            return Err(NakoError::Forbidden {
                message: format!(
                    "renderer session {} does not support {} command",
                    renderer.id,
                    request.command.as_str()
                ),
            });
        }
        validate_renderer_command_shape(
            request.command,
            request.source_id,
            request.position_ms,
            request.volume_percent,
        )?;

        let now_ms = current_time_ms()?;
        self.store
            .create_renderer_command(NewRendererCommand {
                id: RendererCommandId::new(),
                renderer_session_id: request.renderer_session_id,
                controlling_principal_id: request.controlling_principal_id,
                command: request.command,
                item_id: request.item_id,
                source_id: request.source_id,
                playback_session_id: request.playback_session_id,
                position_ms: request.position_ms,
                volume_percent: request.volume_percent,
                payload_json: request.payload_json,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
            })
            .await
    }

    pub(crate) async fn complete_command(
        &self,
        request: CompleteRendererCommandRequest,
    ) -> Result<RendererCommandRecord> {
        self.require_renderer_owned_by(&request.principal_id, request.renderer_session_id)
            .await?;
        if !request.state.is_terminal() {
            return Err(NakoError::InvalidInput {
                message: "renderer command completion state must be terminal".to_owned(),
            });
        }
        if matches!(request.state, RendererCommandState::Failed)
            && request.failure_message.as_deref().is_none_or(str::is_empty)
        {
            return Err(NakoError::InvalidInput {
                message: "failed renderer command completion requires a failure message".to_owned(),
            });
        }

        let existing = self
            .store
            .get_renderer_command(request.command_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_command",
                id: request.command_id.to_string(),
            })?;
        if existing.renderer_session_id != request.renderer_session_id {
            return Err(NakoError::Forbidden {
                message: "renderer command belongs to a different renderer session".to_owned(),
            });
        }
        if existing.state.is_terminal() {
            if existing.state == request.state {
                return Ok(existing);
            }

            return Err(NakoError::Conflict {
                message: "renderer command already completed with a different state".to_owned(),
            });
        }

        self.store
            .complete_renderer_command(RendererCommandCompletion {
                id: request.command_id,
                state: request.state,
                completed_at_ms: current_time_ms()?,
                failure_message: request.failure_message,
            })
            .await?
            .ok_or_else(|| NakoError::Conflict {
                message: "renderer command could not be completed".to_owned(),
            })
    }

    async fn require_online_renderer(
        &self,
        renderer_session_id: RendererSessionId,
    ) -> Result<RendererSessionRecord> {
        let renderer = self
            .store
            .get_renderer_session(renderer_session_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_session",
                id: renderer_session_id.to_string(),
            })?;
        ensure_renderer_online(&renderer)?;

        Ok(renderer)
    }

    async fn require_online_renderer_owned_by(
        &self,
        principal_id: &UserPrincipalId,
        renderer_session_id: RendererSessionId,
    ) -> Result<RendererSessionRecord> {
        let renderer = self
            .require_renderer_owned_by(principal_id, renderer_session_id)
            .await?;
        ensure_renderer_online(&renderer)?;

        Ok(renderer)
    }

    async fn require_renderer_owned_by(
        &self,
        principal_id: &UserPrincipalId,
        renderer_session_id: RendererSessionId,
    ) -> Result<RendererSessionRecord> {
        let renderer = self
            .store
            .get_renderer_session(renderer_session_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_session",
                id: renderer_session_id.to_string(),
            })?;
        if &renderer.owner_principal_id != principal_id {
            return Err(NakoError::Forbidden {
                message: "renderer session belongs to a different principal".to_owned(),
            });
        }

        Ok(renderer)
    }
}

fn normalize_display_name(display_name: String) -> Result<String> {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "renderer display name cannot be empty".to_owned(),
        });
    }
    if trimmed.len() > MAX_RENDERER_DISPLAY_NAME_LEN {
        return Err(NakoError::InvalidInput {
            message: format!(
                "renderer display name cannot exceed {MAX_RENDERER_DISPLAY_NAME_LEN} bytes"
            ),
        });
    }
    if trimmed.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: "renderer display name cannot contain control characters".to_owned(),
        });
    }

    Ok(trimmed.to_owned())
}

fn validate_nako_renderer_target(
    target_kind: PlaybackTargetKind,
    transport_auth: PlaybackTargetTransportAuth,
) -> Result<()> {
    if !matches!(
        target_kind,
        PlaybackTargetKind::NakoRemoteClient
            | PlaybackTargetKind::NativeDesktop
            | PlaybackTargetKind::NativeMobile
    ) {
        return Err(NakoError::Unsupported(
            "public renderer registration only accepts Nako remote client targets",
        ));
    }
    if !matches!(transport_auth, PlaybackTargetTransportAuth::Bearer) {
        return Err(NakoError::Unsupported(
            "public renderer registration requires bearer-authenticated Nako clients",
        ));
    }

    Ok(())
}

fn normalize_control_capabilities(
    capabilities: RendererControlCapabilities,
) -> Result<RendererControlCapabilities> {
    let mut seen = HashSet::new();
    let commands = capabilities
        .commands
        .into_iter()
        .filter(|command| seen.insert(*command))
        .collect::<Vec<_>>();

    if commands.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "renderer control capabilities must include at least one command".to_owned(),
        });
    }

    Ok(RendererControlCapabilities { commands })
}

fn validate_renderer_command_shape(
    command: RendererControlCommand,
    source_id: Option<MediaSourceId>,
    position_ms: Option<u64>,
    volume_percent: Option<u8>,
) -> Result<()> {
    if matches!(command, RendererControlCommand::Play) && source_id.is_none() {
        return Err(NakoError::InvalidInput {
            message: "play renderer command requires source_id".to_owned(),
        });
    }
    if matches!(command, RendererControlCommand::Seek) && position_ms.is_none() {
        return Err(NakoError::InvalidInput {
            message: "seek renderer command requires position_ms".to_owned(),
        });
    }
    if matches!(command, RendererControlCommand::SetVolume) && volume_percent.is_none() {
        return Err(NakoError::InvalidInput {
            message: "set_volume renderer command requires volume_percent".to_owned(),
        });
    }
    if volume_percent.is_some_and(|value| value > 100) {
        return Err(NakoError::InvalidInput {
            message: "renderer command volume_percent cannot exceed 100".to_owned(),
        });
    }

    Ok(())
}

fn renderer_expiry(now_ms: i64, ttl_ms: Option<u64>) -> Result<Option<i64>> {
    let ttl_ms = ttl_ms
        .map(|value| {
            if value == 0 {
                return Err(NakoError::InvalidInput {
                    message: "renderer ttl_ms must be greater than zero".to_owned(),
                });
            }

            i64::try_from(value).map_err(|err| NakoError::InvalidInput {
                message: format!("renderer ttl_ms does not fit i64: {err}"),
            })
        })
        .transpose()?
        .unwrap_or(DEFAULT_RENDERER_SESSION_TTL_MS);
    if ttl_ms > MAX_RENDERER_SESSION_TTL_MS {
        return Err(NakoError::InvalidInput {
            message: format!("renderer ttl_ms cannot exceed {MAX_RENDERER_SESSION_TTL_MS}"),
        });
    }

    now_ms
        .checked_add(ttl_ms)
        .map(Some)
        .ok_or_else(|| NakoError::InvalidInput {
            message: "renderer expiry timestamp overflowed".to_owned(),
        })
}

fn media_capabilities_json(capabilities: &ClientPlaybackCapabilities) -> Result<String> {
    serde_json::to_string(capabilities).map_err(|err| NakoError::InvalidInput {
        message: format!("renderer media capabilities could not be serialized: {err}"),
    })
}

fn renderer_session_expired(renderer: &RendererSessionRecord, now_ms: i64) -> bool {
    renderer
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_ms)
}

fn ensure_renderer_online(renderer: &RendererSessionRecord) -> Result<()> {
    if matches!(renderer.state, RendererSessionState::Revoked) {
        return Err(NakoError::Forbidden {
            message: "renderer session is revoked".to_owned(),
        });
    }
    if !matches!(renderer.state, RendererSessionState::Online) {
        return Err(NakoError::Conflict {
            message: "renderer session is not online".to_owned(),
        });
    }
    if renderer_session_expired(renderer, current_time_ms()?) {
        return Err(NakoError::Conflict {
            message: "renderer session expired".to_owned(),
        });
    }

    Ok(())
}
