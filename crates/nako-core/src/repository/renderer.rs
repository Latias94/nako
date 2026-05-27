use async_trait::async_trait;

use crate::{
    PageRequest, RendererCommandCompletion, RendererCommandId, RendererCommandRecord,
    RendererCommandState, RendererSessionHeartbeat, RendererSessionId, RendererSessionRecord,
    RendererSessionState, Result, UserPrincipalId,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RendererSessionListFilter {
    pub owner_principal_id: Option<UserPrincipalId>,
    pub state: Option<RendererSessionState>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RendererCommandListFilter {
    pub renderer_session_id: Option<RendererSessionId>,
    pub state: Option<RendererCommandState>,
}

#[async_trait]
pub trait RendererSessionRepository: Send + Sync {
    async fn upsert_renderer_session(
        &self,
        session: crate::NewRendererSession,
    ) -> Result<RendererSessionRecord>;

    async fn get_renderer_session(
        &self,
        id: RendererSessionId,
    ) -> Result<Option<RendererSessionRecord>>;

    async fn list_renderer_sessions(
        &self,
        filter: RendererSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<RendererSessionRecord>>;

    async fn record_renderer_session_heartbeat(
        &self,
        heartbeat: RendererSessionHeartbeat,
    ) -> Result<Option<RendererSessionRecord>>;

    async fn attach_renderer_playback_session(
        &self,
        id: RendererSessionId,
        playback_session_id: Option<crate::PlaybackSessionId>,
        updated_at_ms: i64,
    ) -> Result<Option<RendererSessionRecord>>;

    async fn create_renderer_command(
        &self,
        command: crate::NewRendererCommand,
    ) -> Result<RendererCommandRecord>;

    async fn get_renderer_command(
        &self,
        id: RendererCommandId,
    ) -> Result<Option<RendererCommandRecord>>;

    async fn list_renderer_commands(
        &self,
        filter: RendererCommandListFilter,
        page: PageRequest,
    ) -> Result<Vec<RendererCommandRecord>>;

    async fn claim_next_renderer_command(
        &self,
        renderer_session_id: RendererSessionId,
        delivered_at_ms: i64,
    ) -> Result<Option<RendererCommandRecord>>;

    async fn complete_renderer_command(
        &self,
        completion: RendererCommandCompletion,
    ) -> Result<Option<RendererCommandRecord>>;
}
