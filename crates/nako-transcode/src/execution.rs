use std::path::PathBuf;

use nako_core::{MediaSourceId, Result};
pub use nako_core::{TranscodeSessionId, TranscodeSessionKind};
use serde::{Deserialize, Serialize};

use super::ffmpeg::{FfmpegCommandBuilder, FfmpegCommandPlan, HlsRequest, RemuxRequest};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeExecutionRequest {
    pub session_id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub command: FfmpegCommandPlan,
    pub output_path: PathBuf,
}

impl TranscodeExecutionRequest {
    pub fn plan_remux(request: RemuxRequest, builder: &FfmpegCommandBuilder) -> Result<Self> {
        Self::plan_remux_with_id(TranscodeSessionId::new(), request, builder)
    }

    pub fn plan_remux_with_id(
        session_id: TranscodeSessionId,
        request: RemuxRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<Self> {
        let command = builder.remux(&request)?;
        Ok(Self {
            session_id,
            source_id: request.source_id,
            kind: TranscodeSessionKind::Remux,
            command,
            output_path: request.output_path,
        })
    }

    pub fn plan_hls(request: HlsRequest, builder: &FfmpegCommandBuilder) -> Result<Self> {
        Self::plan_hls_with_id(TranscodeSessionId::new(), request, builder)
    }

    pub fn plan_hls_with_id(
        session_id: TranscodeSessionId,
        request: HlsRequest,
        builder: &FfmpegCommandBuilder,
    ) -> Result<Self> {
        let command = builder.hls(&request)?;
        Ok(Self {
            session_id,
            source_id: request.source_id,
            kind: TranscodeSessionKind::HlsTranscode,
            command,
            output_path: request.artifacts.primary_playlist_path().to_path_buf(),
        })
    }
}
