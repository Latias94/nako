use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use nako_core::{MediaSourceId, NakoError, Result};
use serde::{Deserialize, Serialize};

use super::{
    HardwareAcceleration, TranscodeAccelerationPlan, TranscodeExecutionPolicy,
    TranscodeSubtitleStrategy,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FfmpegCommandPlan {
    pub program: PathBuf,
    pub args: Vec<FfmpegArg>,
}

impl FfmpegCommandPlan {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>, args: Vec<FfmpegArg>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    #[must_use]
    pub fn args_as_os_strings(&self) -> Vec<OsString> {
        self.args.iter().map(FfmpegArg::to_os_string).collect()
    }

    #[must_use]
    pub fn argv_lossy(&self) -> Vec<String> {
        let mut argv = Vec::with_capacity(self.args.len() + 1);
        argv.push(self.program.display().to_string());
        argv.extend(self.args.iter().map(FfmpegArg::to_string_lossy));
        argv
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum FfmpegArg {
    Raw(String),
    Path(PathBuf),
}

impl FfmpegArg {
    #[must_use]
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    #[must_use]
    pub fn path(value: impl Into<PathBuf>) -> Self {
        Self::Path(value.into())
    }

    #[must_use]
    pub fn to_os_string(&self) -> OsString {
        match self {
            Self::Raw(value) => OsString::from(value),
            Self::Path(value) => value.as_os_str().to_os_string(),
        }
    }

    #[must_use]
    pub fn to_string_lossy(&self) -> String {
        match self {
            Self::Raw(value) => value.clone(),
            Self::Path(value) => value.display().to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegOverwritePolicy {
    Allow,
    #[default]
    Never,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxRequest {
    pub source_id: MediaSourceId,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub output_container: RemuxContainer,
    pub overwrite: FfmpegOverwritePolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsRequest {
    pub source_id: MediaSourceId,
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
    pub playlist_path: PathBuf,
    pub segment_pattern: PathBuf,
    pub segment_time_seconds: u32,
    pub execution_policy: TranscodeExecutionPolicy,
    pub overwrite: FfmpegOverwritePolicy,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FfmpegCommandBuilder {
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

    #[must_use]
    pub fn ffmpeg_path(&self) -> &Path {
        &self.ffmpeg_path
    }

    pub fn remux(&self, request: &RemuxRequest) -> Result<FfmpegCommandPlan> {
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

        let overwrite_arg = match request.overwrite {
            FfmpegOverwritePolicy::Allow => "-y",
            FfmpegOverwritePolicy::Never => "-n",
        };

        Ok(FfmpegCommandPlan::new(
            self.ffmpeg_path.clone(),
            vec![
                FfmpegArg::raw("-hide_banner"),
                FfmpegArg::raw("-loglevel"),
                FfmpegArg::raw("warning"),
                FfmpegArg::raw(overwrite_arg),
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

    pub fn hls(&self, request: &HlsRequest) -> Result<FfmpegCommandPlan> {
        if request.input_path.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls input path cannot be empty".to_owned(),
            });
        }

        if request.output_dir.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls output directory cannot be empty".to_owned(),
            });
        }

        if request.playlist_path.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls playlist path cannot be empty".to_owned(),
            });
        }

        if request.segment_pattern.as_os_str().is_empty() {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern cannot be empty".to_owned(),
            });
        }

        if !request.playlist_path.starts_with(&request.output_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls playlist path must be inside the output directory".to_owned(),
            });
        }

        if !request.segment_pattern.starts_with(&request.output_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern must be inside the output directory".to_owned(),
            });
        }

        if !request
            .segment_pattern
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.contains('%'))
        {
            return Err(NakoError::InvalidInput {
                message: "hls segment pattern must contain a printf-style segment placeholder"
                    .to_owned(),
            });
        }

        if request.input_path == request.playlist_path {
            return Err(NakoError::InvalidInput {
                message: "hls input and playlist paths must differ".to_owned(),
            });
        }

        let overwrite_arg = match request.overwrite {
            FfmpegOverwritePolicy::Allow => "-y",
            FfmpegOverwritePolicy::Never => "-n",
        };
        let segment_time = request.segment_time_seconds.max(1).to_string();

        validate_hls_subtitle_strategy(request.execution_policy.subtitle_strategy)?;

        let mut args = vec![
            FfmpegArg::raw("-hide_banner"),
            FfmpegArg::raw("-loglevel"),
            FfmpegArg::raw("warning"),
            FfmpegArg::raw(overwrite_arg),
            FfmpegArg::raw("-i"),
            FfmpegArg::path(request.input_path.clone()),
            FfmpegArg::raw("-map"),
            FfmpegArg::raw("0:v:0"),
            FfmpegArg::raw("-map"),
            FfmpegArg::raw("0:a:0?"),
        ];
        append_hls_video_encoder_args(&mut args, request.execution_policy.acceleration);
        append_hls_output_constraint_args(&mut args, request.execution_policy);
        args.extend([
            FfmpegArg::raw("-c:a"),
            FfmpegArg::raw("aac"),
            FfmpegArg::raw("-f"),
            FfmpegArg::raw("hls"),
            FfmpegArg::raw("-hls_time"),
            FfmpegArg::raw(segment_time),
            FfmpegArg::raw("-hls_playlist_type"),
            FfmpegArg::raw("vod"),
            FfmpegArg::raw("-hls_segment_filename"),
            FfmpegArg::path(request.segment_pattern.clone()),
            FfmpegArg::path(request.playlist_path.clone()),
        ]);

        Ok(FfmpegCommandPlan::new(self.ffmpeg_path.clone(), args))
    }
}

fn validate_hls_subtitle_strategy(strategy: TranscodeSubtitleStrategy) -> Result<()> {
    match strategy {
        TranscodeSubtitleStrategy::None | TranscodeSubtitleStrategy::OmitSelected => Ok(()),
        TranscodeSubtitleStrategy::PreserveInContainer
        | TranscodeSubtitleStrategy::BurnInSelected
        | TranscodeSubtitleStrategy::SidecarSelected => Err(NakoError::Unsupported(
            "hls subtitle strategy is not implemented by the ffmpeg adapter",
        )),
    }
}

fn append_hls_video_encoder_args(
    args: &mut Vec<FfmpegArg>,
    acceleration: TranscodeAccelerationPlan,
) {
    match acceleration.encode.accelerator {
        HardwareAcceleration::None => {
            args.push(FfmpegArg::raw("-c:v"));
            args.push(FfmpegArg::raw("libx264"));
        }
        HardwareAcceleration::Vaapi => {
            args.push(FfmpegArg::raw("-hwaccel"));
            args.push(FfmpegArg::raw("vaapi"));
            args.push(FfmpegArg::raw("-vf"));
            args.push(FfmpegArg::raw("format=nv12,hwupload"));
            args.push(FfmpegArg::raw("-c:v"));
            args.push(FfmpegArg::raw("h264_vaapi"));
        }
        HardwareAcceleration::Nvenc => {
            args.push(FfmpegArg::raw("-c:v"));
            args.push(FfmpegArg::raw("h264_nvenc"));
        }
        HardwareAcceleration::QuickSync => {
            args.push(FfmpegArg::raw("-hwaccel"));
            args.push(FfmpegArg::raw("qsv"));
            args.push(FfmpegArg::raw("-c:v"));
            args.push(FfmpegArg::raw("h264_qsv"));
        }
    }
}

fn append_hls_output_constraint_args(args: &mut Vec<FfmpegArg>, policy: TranscodeExecutionPolicy) {
    if let Some(max_video_bitrate) = policy.output_constraints.max_video_bitrate {
        args.push(FfmpegArg::raw("-maxrate"));
        args.push(FfmpegArg::raw(max_video_bitrate.to_string()));
        args.push(FfmpegArg::raw("-bufsize"));
        args.push(FfmpegArg::raw(
            max_video_bitrate.saturating_mul(2).to_string(),
        ));
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
