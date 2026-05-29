use std::path::PathBuf;

use nako_core::{Result, TranscodeSessionRuntimeMetrics};
use serde::{Deserialize, Serialize};

use super::{
    CancellationToken, TranscodeExecutionRequest, TranscodeSessionId, TranscodeSessionKind,
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
    pub execution: TranscodeExecutionRequest,
    pub cancel: CancellationToken,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranscodeEngineStartOutcome {
    Finished {
        session_id: TranscodeSessionId,
        artifact_kind: TranscodeEngineArtifactKind,
        output_path: PathBuf,
        runtime_metrics: TranscodeSessionRuntimeMetrics,
    },
    Cancelled {
        session_id: TranscodeSessionId,
        artifact_kind: TranscodeEngineArtifactKind,
        temporary_output_path: PathBuf,
        runtime_metrics: TranscodeSessionRuntimeMetrics,
    },
}

impl TranscodeEngineStartOutcome {
    #[must_use]
    pub const fn runtime_metrics(&self) -> &TranscodeSessionRuntimeMetrics {
        match self {
            Self::Finished {
                runtime_metrics, ..
            }
            | Self::Cancelled {
                runtime_metrics, ..
            } => runtime_metrics,
        }
    }
}

pub trait TranscodeEngineAdapter {
    fn adapter_kind(&self) -> TranscodeEngineAdapterKind;

    #[allow(async_fn_in_trait)]
    async fn start(
        &self,
        command: TranscodeEngineStartCommand,
    ) -> Result<TranscodeEngineStartOutcome>;
}
