use std::{collections::HashMap, path::PathBuf};

use nako_core::{MediaSourceId, NakoError, Result, TranscodeSessionRuntimeMetrics};
pub use nako_core::{TranscodeSessionId, TranscodeSessionKind, TranscodeSessionState};
use serde::{Deserialize, Serialize};

use super::ffmpeg::{FfmpegCommandBuilder, FfmpegCommandPlan, HlsRequest, RemuxRequest};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeSession {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub state: TranscodeSessionState,
    pub command: FfmpegCommandPlan,
    pub output_path: PathBuf,
    pub failure_message: Option<String>,
    pub runtime_metrics: TranscodeSessionRuntimeMetrics,
}

#[derive(Debug, Default)]
pub struct TranscodeSessionManager {
    sessions: HashMap<TranscodeSessionId, TranscodeSession>,
}

impl TranscodeSessionManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn plan_remux(
        &mut self,
        request: RemuxRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        self.plan_remux_with_id(TranscodeSessionId::new(), request, builder)
    }

    pub fn plan_remux_with_id(
        &mut self,
        session_id: TranscodeSessionId,
        request: RemuxRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        let command = builder.remux(&request)?;
        let session = TranscodeSession {
            id: session_id,
            source_id: request.source_id,
            kind: TranscodeSessionKind::Remux,
            state: TranscodeSessionState::Planned,
            command,
            output_path: request.output_path,
            failure_message: None,
            runtime_metrics: TranscodeSessionRuntimeMetrics::default(),
        };

        self.sessions.insert(session.id, session.clone());
        Ok(session)
    }

    pub fn plan_hls(
        &mut self,
        request: HlsRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        self.plan_hls_with_id(TranscodeSessionId::new(), request, builder)
    }

    pub fn plan_hls_with_id(
        &mut self,
        session_id: TranscodeSessionId,
        request: HlsRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<TranscodeSession> {
        let command = builder.hls(&request)?;
        let session = TranscodeSession {
            id: session_id,
            source_id: request.source_id,
            kind: TranscodeSessionKind::HlsTranscode,
            state: TranscodeSessionState::Planned,
            command,
            output_path: request.playlist_path,
            failure_message: None,
            runtime_metrics: TranscodeSessionRuntimeMetrics::default(),
        };

        self.sessions.insert(session.id, session.clone());
        Ok(session)
    }

    pub fn mark_starting(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[TranscodeSessionState::Planned],
            TranscodeSessionState::Starting,
            None,
        )
    }

    pub fn mark_running(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[
                TranscodeSessionState::Planned,
                TranscodeSessionState::Starting,
            ],
            TranscodeSessionState::Running,
            None,
        )
    }

    pub fn request_cancel(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[
                TranscodeSessionState::Planned,
                TranscodeSessionState::Starting,
                TranscodeSessionState::Running,
            ],
            TranscodeSessionState::CancelRequested,
            None,
        )
    }

    pub fn mark_cancelled(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[TranscodeSessionState::CancelRequested],
            TranscodeSessionState::Cancelled,
            None,
        )
    }

    pub fn mark_finished(&mut self, session_id: TranscodeSessionId) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[TranscodeSessionState::Running],
            TranscodeSessionState::Finished,
            None,
        )
    }

    pub fn mark_failed(
        &mut self,
        session_id: TranscodeSessionId,
        message: impl Into<String>,
    ) -> Result<TranscodeSession> {
        self.transition(
            session_id,
            &[
                TranscodeSessionState::Starting,
                TranscodeSessionState::Running,
                TranscodeSessionState::CancelRequested,
            ],
            TranscodeSessionState::Failed,
            Some(message.into()),
        )
    }

    pub fn update_runtime_metrics(
        &mut self,
        session_id: TranscodeSessionId,
        metrics: TranscodeSessionRuntimeMetrics,
    ) -> Result<TranscodeSession> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        session.runtime_metrics = metrics;
        Ok(session.clone())
    }

    #[must_use]
    pub fn get(&self, session_id: TranscodeSessionId) -> Option<&TranscodeSession> {
        self.sessions.get(&session_id)
    }

    fn transition(
        &mut self,
        session_id: TranscodeSessionId,
        allowed_from: &[TranscodeSessionState],
        target: TranscodeSessionState,
        failure_message: Option<String>,
    ) -> Result<TranscodeSession> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })?;

        if !allowed_from.contains(&session.state) {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "cannot transition transcode session {session_id} from {:?} to {:?}",
                    session.state, target
                ),
            });
        }

        session.state = target;
        session.failure_message = failure_message;
        Ok(session.clone())
    }
}
