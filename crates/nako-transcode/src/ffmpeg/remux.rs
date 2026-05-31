use std::path::{Path, PathBuf};

use nako_core::{MediaSourceId, NakoError, Result};
use serde::{Deserialize, Serialize};

use super::{
    FfmpegArg, FfmpegCommandPlan, FfmpegOverwritePolicy,
    common::{command_plan, overwrite_arg},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RemuxRequest {
    pub(crate) source_id: MediaSourceId,
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) output_container: RemuxContainer,
    pub(crate) overwrite: FfmpegOverwritePolicy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemuxContainer {
    Mp4,
    Mkv,
}

impl RemuxContainer {
    #[must_use]
    pub const fn ffmpeg_format(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "matroska",
        }
    }

    #[must_use]
    pub const fn file_extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

pub(super) fn plan_remux_command(
    ffmpeg_path: &Path,
    request: &RemuxRequest,
) -> Result<FfmpegCommandPlan> {
    validate_remux_request(request)?;

    Ok(command_plan(
        ffmpeg_path,
        vec![
            FfmpegArg::raw("-hide_banner"),
            FfmpegArg::raw("-loglevel"),
            FfmpegArg::raw("warning"),
            FfmpegArg::raw(overwrite_arg(request.overwrite)),
            FfmpegArg::raw("-i"),
            FfmpegArg::path(request.input_path.clone()),
            FfmpegArg::raw("-map"),
            FfmpegArg::raw("0"),
            FfmpegArg::raw("-c"),
            FfmpegArg::raw("copy"),
            FfmpegArg::raw("-f"),
            FfmpegArg::raw(request.output_container.ffmpeg_format()),
            FfmpegArg::path(request.output_path.clone()),
        ],
    ))
}

fn validate_remux_request(request: &RemuxRequest) -> Result<()> {
    if request.input_path.as_os_str().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "remux input path cannot be empty".to_owned(),
        });
    }

    if request.output_path.as_os_str().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "remux output path cannot be empty".to_owned(),
        });
    }

    if request.input_path == request.output_path {
        return Err(NakoError::InvalidInput {
            message: "remux input and output paths must differ".to_owned(),
        });
    }

    Ok(())
}
