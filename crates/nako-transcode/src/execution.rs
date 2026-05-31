use std::path::{Path, PathBuf};

use nako_core::{MediaSourceId, Result};
pub use nako_core::{TranscodeSessionId, TranscodeSessionKind};
use serde::{Deserialize, Serialize};

use super::{
    HlsArtifactManifest, HlsPlaybackGeneration, TranscodeExecutionPolicy, TranscodeTrackSelection,
    ffmpeg::{
        FfmpegCommandBuilder, FfmpegCommandPlan, FfmpegOverwritePolicy, HlsRequest, RemuxContainer,
        RemuxRequest,
    },
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxExecutionPlanRequest {
    pub source_id: MediaSourceId,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub output_container: RemuxContainer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsExecutionPlanRequest {
    pub source_id: MediaSourceId,
    pub input_path: PathBuf,
    pub playback_generation: HlsPlaybackGeneration,
    pub artifacts: HlsArtifactManifest,
    pub segment_time_seconds: u32,
    pub track_selection: TranscodeTrackSelection,
    pub execution_policy: TranscodeExecutionPolicy,
}

#[derive(Clone, Debug)]
pub struct FfmpegExecutionPlanner {
    builder: FfmpegCommandBuilder,
}

impl Default for FfmpegExecutionPlanner {
    fn default() -> Self {
        Self::new("ffmpeg")
    }
}

impl FfmpegExecutionPlanner {
    #[must_use]
    pub fn new(ffmpeg_path: impl Into<PathBuf>) -> Self {
        Self {
            builder: FfmpegCommandBuilder::new(ffmpeg_path),
        }
    }

    pub fn plan_remux(
        &self,
        request: RemuxExecutionPlanRequest,
    ) -> Result<TranscodeExecutionRequest> {
        self.plan_remux_with_id(TranscodeSessionId::new(), request)
    }

    pub fn plan_remux_with_id(
        &self,
        session_id: TranscodeSessionId,
        request: RemuxExecutionPlanRequest,
    ) -> Result<TranscodeExecutionRequest> {
        TranscodeExecutionRequest::plan_remux_with_id(
            session_id,
            RemuxRequest {
                source_id: request.source_id,
                input_path: request.input_path,
                output_path: request.output_path,
                output_container: request.output_container,
                overwrite: FfmpegOverwritePolicy::Never,
            },
            &self.builder,
        )
    }

    pub fn plan_hls(&self, request: HlsExecutionPlanRequest) -> Result<TranscodeExecutionRequest> {
        self.plan_hls_with_id(TranscodeSessionId::new(), request)
    }

    pub fn plan_hls_with_id(
        &self,
        session_id: TranscodeSessionId,
        request: HlsExecutionPlanRequest,
    ) -> Result<TranscodeExecutionRequest> {
        TranscodeExecutionRequest::plan_hls_with_id(
            session_id,
            HlsRequest {
                source_id: request.source_id,
                input_path: request.input_path,
                playback_generation: request.playback_generation,
                artifacts: request.artifacts,
                segment_time_seconds: request.segment_time_seconds,
                track_selection: request.track_selection,
                execution_policy: request.execution_policy,
                overwrite: FfmpegOverwritePolicy::Allow,
            },
            &self.builder,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscodeExecutionRequest {
    pub(crate) session_id: TranscodeSessionId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) kind: TranscodeSessionKind,
    pub(crate) command: FfmpegCommandPlan,
    pub(crate) output_path: PathBuf,
}

impl TranscodeExecutionRequest {
    #[must_use]
    pub const fn session_id(&self) -> TranscodeSessionId {
        self.session_id
    }

    #[must_use]
    pub const fn source_id(&self) -> MediaSourceId {
        self.source_id
    }

    #[must_use]
    pub const fn kind(&self) -> TranscodeSessionKind {
        self.kind
    }

    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    pub(crate) fn plan_remux_with_id(
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

    pub(crate) fn plan_hls_with_id(
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
