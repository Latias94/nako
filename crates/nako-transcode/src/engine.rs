use std::path::PathBuf;

use nako_core::{NakoError, Result};
use serde::{Deserialize, Serialize};

use super::{
    CancellationToken, TranscodeSession, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionManager, TranscodeSessionState,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeEngineAdapterKind {
    FfmpegCli,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeEngineArtifactKind {
    RemuxFile,
    HlsPlaylist,
}

impl TranscodeEngineArtifactKind {
    #[must_use]
    pub const fn from_session_kind(kind: TranscodeSessionKind) -> Self {
        match kind {
            TranscodeSessionKind::Remux => Self::RemuxFile,
            TranscodeSessionKind::HlsTranscode => Self::HlsPlaylist,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TranscodeEngineStartCommand {
    pub session_id: TranscodeSessionId,
    pub cancel: CancellationToken,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscodeEngineProgress {
    pub session_id: TranscodeSessionId,
    pub adapter_kind: TranscodeEngineAdapterKind,
    pub artifact_kind: TranscodeEngineArtifactKind,
    pub state: TranscodeSessionState,
    pub output_path: PathBuf,
    pub failure_message: Option<String>,
}

impl TranscodeEngineProgress {
    #[must_use]
    pub fn from_session(
        adapter_kind: TranscodeEngineAdapterKind,
        session: &TranscodeSession,
    ) -> Self {
        Self {
            session_id: session.id,
            adapter_kind,
            artifact_kind: TranscodeEngineArtifactKind::from_session_kind(session.kind),
            state: session.state,
            output_path: session.output_path.clone(),
            failure_message: session.failure_message.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranscodeEngineStartOutcome {
    Finished {
        session_id: TranscodeSessionId,
        artifact_kind: TranscodeEngineArtifactKind,
        output_path: PathBuf,
    },
    Cancelled {
        session_id: TranscodeSessionId,
        artifact_kind: TranscodeEngineArtifactKind,
        temporary_output_path: PathBuf,
    },
}

pub trait TranscodeEngineAdapter {
    fn adapter_kind(&self) -> TranscodeEngineAdapterKind;

    fn progress(
        &self,
        manager: &TranscodeSessionManager,
        session_id: TranscodeSessionId,
    ) -> Result<TranscodeEngineProgress> {
        let session = manager.get(session_id).ok_or_else(|| NakoError::NotFound {
            entity: "transcode_session",
            id: session_id.to_string(),
        })?;

        Ok(TranscodeEngineProgress::from_session(
            self.adapter_kind(),
            session,
        ))
    }

    #[allow(async_fn_in_trait)]
    async fn start(
        &self,
        manager: &mut TranscodeSessionManager,
        command: TranscodeEngineStartCommand,
    ) -> Result<TranscodeEngineStartOutcome>;
}
