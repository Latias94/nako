use async_trait::async_trait;

use crate::{
    MediaSourceId, NewPlaybackSession, PageRequest, PlaybackSessionHeartbeat, PlaybackSessionId,
    PlaybackSessionRecord, PlaybackSessionState, Result, TranscodeSessionId, UserPrincipalId,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlaybackSessionListFilter {
    pub principal_id: Option<UserPrincipalId>,
    pub source_id: Option<MediaSourceId>,
    pub state: Option<PlaybackSessionState>,
}

#[async_trait]
pub trait PlaybackSessionRepository: Send + Sync {
    async fn create_playback_session(
        &self,
        session: NewPlaybackSession,
    ) -> Result<PlaybackSessionRecord>;

    async fn get_playback_session(
        &self,
        id: PlaybackSessionId,
    ) -> Result<Option<PlaybackSessionRecord>>;

    async fn list_playback_sessions(
        &self,
        filter: PlaybackSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackSessionRecord>>;

    async fn find_latest_playback_session_by_transcode_session(
        &self,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<Option<PlaybackSessionRecord>>;

    async fn link_playback_session_transcode(
        &self,
        id: PlaybackSessionId,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<PlaybackSessionRecord>;

    async fn record_playback_session_heartbeat(
        &self,
        heartbeat: PlaybackSessionHeartbeat,
    ) -> Result<Option<PlaybackSessionRecord>>;

    async fn set_playback_session_state(
        &self,
        id: PlaybackSessionId,
        state: PlaybackSessionState,
        ended_at_ms: Option<i64>,
    ) -> Result<Option<PlaybackSessionRecord>>;
}
