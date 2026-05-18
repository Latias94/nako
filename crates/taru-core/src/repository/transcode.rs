use async_trait::async_trait;

use crate::{
    MediaSourceId, NewTranscodeSession, PageRequest, Result, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionState,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TranscodeSessionListFilter {
    pub source_id: Option<MediaSourceId>,
    pub kind: Option<TranscodeSessionKind>,
    pub state: Option<TranscodeSessionState>,
}

#[async_trait]
pub trait TranscodeSessionRepository: Send + Sync {
    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord>;

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>>;

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord>;

    async fn request_transcode_session_cancellation(
        &self,
        id: TranscodeSessionId,
        failure_message: String,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn fail_stale_transcode_sessions(
        &self,
        failure_category: TranscodeFailureCategory,
        failure_message: String,
    ) -> Result<u64>;
}
