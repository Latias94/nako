use std::path::PathBuf;

use nako_core::Result;
use serde::{Deserialize, Serialize};

mod common;
mod hls;
mod remux;

pub(crate) use common::{FfmpegArg, FfmpegCommandPlan, FfmpegOverwritePolicy};
pub(crate) use hls::HlsRequest;
pub use remux::RemuxContainer;
pub(crate) use remux::RemuxRequest;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FfmpegCommandBuilder {
    ffmpeg_path: PathBuf,
}

impl Default for FfmpegCommandBuilder {
    fn default() -> Self {
        Self::new("ffmpeg")
    }
}

impl FfmpegCommandBuilder {
    #[must_use]
    pub fn new(ffmpeg_path: impl Into<PathBuf>) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
        }
    }

    pub fn remux(&self, request: &RemuxRequest) -> Result<FfmpegCommandPlan> {
        remux::plan_remux_command(&self.ffmpeg_path, request)
    }

    pub fn hls(&self, request: &HlsRequest) -> Result<FfmpegCommandPlan> {
        hls::plan_hls_command(&self.ffmpeg_path, request)
    }
}

pub(crate) fn stderr_message(stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_owned();

    if message.is_empty() {
        "ffmpeg remux process failed".to_owned()
    } else {
        message
    }
}
