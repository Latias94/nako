use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use nako_core::{MediaSourceId, NakoError, Result};
use serde::{Deserialize, Serialize};

use super::{
    HLS_ADAPTIVE_FMP4_INIT_PATTERN, HardwareAcceleration, HlsArtifactManifest,
    HlsPlaybackGeneration, HlsRendition, HlsSegmentContainer, HlsVariantPolicy,
    TranscodeAccelerationPlan, TranscodeAudioDownmixRequirement,
    TranscodeAudioNormalizationRequirement, TranscodeAudioOutputRequirement,
    TranscodeExecutionPolicy, TranscodeSubtitleStrategy, TranscodeTrackSelection,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FfmpegCommandPlan {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<FfmpegArg>,
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

    #[cfg(test)]
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
pub(crate) enum FfmpegArg {
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

    #[cfg(test)]
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
pub(crate) enum FfmpegOverwritePolicy {
    Allow,
    #[default]
    Never,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RemuxRequest {
    pub(crate) source_id: MediaSourceId,
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) output_container: RemuxContainer,
    pub(crate) overwrite: FfmpegOverwritePolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HlsRequest {
    pub(crate) source_id: MediaSourceId,
    pub(crate) input_path: PathBuf,
    pub(crate) playback_generation: HlsPlaybackGeneration,
    pub(crate) artifacts: HlsArtifactManifest,
    pub(crate) segment_time_seconds: u32,
    pub(crate) track_selection: TranscodeTrackSelection,
    pub(crate) execution_policy: TranscodeExecutionPolicy,
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
        validate_hls_request(request)?;

        let parts = plan_hls_command_parts(request)?;
        Ok(FfmpegCommandPlan::new(
            self.ffmpeg_path.clone(),
            parts.into_args(),
        ))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FfmpegHlsCommandParts {
    pub global: Vec<FfmpegArg>,
    pub device_input: Vec<FfmpegArg>,
    pub input: Vec<FfmpegArg>,
    pub stream_map: Vec<FfmpegArg>,
    pub filter_graph: Vec<FfmpegArg>,
    pub audio_filter: Vec<FfmpegArg>,
    pub video_encoder: Vec<FfmpegArg>,
    pub audio_encoder: Vec<FfmpegArg>,
    pub audio_sidecar: Vec<FfmpegArg>,
    pub subtitle: Vec<FfmpegArg>,
    pub muxer: Vec<FfmpegArg>,
}

impl FfmpegHlsCommandParts {
    #[must_use]
    pub fn into_args(self) -> Vec<FfmpegArg> {
        let Self {
            global,
            device_input,
            input,
            stream_map,
            filter_graph,
            audio_filter,
            video_encoder,
            audio_encoder,
            audio_sidecar,
            subtitle,
            muxer,
        } = self;
        let capacity = global.len()
            + device_input.len()
            + input.len()
            + stream_map.len()
            + filter_graph.len()
            + audio_filter.len()
            + video_encoder.len()
            + audio_encoder.len()
            + audio_sidecar.len()
            + subtitle.len()
            + muxer.len();
        let mut args = Vec::with_capacity(capacity);
        args.extend(global);
        args.extend(device_input);
        args.extend(input);
        args.extend(stream_map);
        args.extend(filter_graph);
        args.extend(audio_filter);
        args.extend(video_encoder);
        args.extend(audio_encoder);
        args.extend(muxer);
        args.extend(audio_sidecar);
        args.extend(subtitle);
        args
    }
}

fn validate_hls_request(request: &HlsRequest) -> Result<()> {
    if request.input_path.as_os_str().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "hls input path cannot be empty".to_owned(),
        });
    }
    request.artifacts.validate()?;

    if request.input_path == request.artifacts.primary_playlist_path() {
        return Err(NakoError::InvalidInput {
            message: "hls input and playlist paths must differ".to_owned(),
        });
    }

    validate_hls_subtitle_strategy(request)
}

fn plan_hls_command_parts(request: &HlsRequest) -> Result<FfmpegHlsCommandParts> {
    if request.artifacts.output().variant_policy == HlsVariantPolicy::Adaptive {
        return plan_adaptive_hls_command_parts(request);
    }

    plan_single_variant_hls_command_parts(request)
}

fn plan_single_variant_hls_command_parts(request: &HlsRequest) -> Result<FfmpegHlsCommandParts> {
    let main_output_has_audio = request.artifacts.main_output_has_audio();
    let audio_filter_graph = hls_audio_filter_graph(request.execution_policy.audio_output)?;
    Ok(FfmpegHlsCommandParts {
        global: hls_global_args(request.overwrite),
        device_input: hls_device_input_args(request.execution_policy.acceleration),
        input: hls_input_args(&request.input_path, request.playback_generation),
        stream_map: hls_stream_map_args(request.track_selection, main_output_has_audio),
        filter_graph: hls_filter_graph_args(request.execution_policy.acceleration),
        audio_filter: hls_audio_filter_args(main_output_has_audio, audio_filter_graph.as_deref()),
        video_encoder: hls_video_encoder_args(
            request.execution_policy,
            request.playback_generation,
            request.segment_time_seconds,
        ),
        audio_encoder: hls_audio_encoder_args(main_output_has_audio),
        audio_sidecar: hls_audio_sidecar_args(
            &request.artifacts,
            request.segment_time_seconds,
            audio_filter_graph.as_deref(),
        ),
        subtitle: hls_subtitle_args(
            request.execution_policy.subtitle_strategy,
            &request.artifacts,
            request.segment_time_seconds,
        ),
        muxer: hls_muxer_args(
            request.segment_time_seconds,
            request.artifacts.media_segment_pattern(),
            request.artifacts.primary_playlist_path(),
            request.artifacts.output().segment_container,
            request.playback_generation,
        ),
    })
}

fn plan_adaptive_hls_command_parts(request: &HlsRequest) -> Result<FfmpegHlsCommandParts> {
    let main_output_has_audio = request.artifacts.main_output_has_audio();
    let audio_filter_graph = hls_audio_filter_graph(request.execution_policy.audio_output)?;
    Ok(FfmpegHlsCommandParts {
        global: hls_global_args(request.overwrite),
        device_input: hls_device_input_args(request.execution_policy.acceleration),
        input: hls_input_args(&request.input_path, request.playback_generation),
        stream_map: hls_adaptive_stream_map_args(
            request.artifacts.renditions().len(),
            main_output_has_audio,
            request.track_selection,
        ),
        filter_graph: hls_filter_graph_args(request.execution_policy.acceleration),
        video_encoder: hls_adaptive_video_encoder_args(
            request.execution_policy,
            request.artifacts.renditions(),
            request.playback_generation,
            request.segment_time_seconds,
        ),
        audio_filter: hls_audio_filter_args(main_output_has_audio, audio_filter_graph.as_deref()),
        audio_encoder: hls_adaptive_audio_encoder_args(
            request.artifacts.renditions(),
            main_output_has_audio,
        ),
        audio_sidecar: hls_audio_sidecar_args(
            &request.artifacts,
            request.segment_time_seconds,
            audio_filter_graph.as_deref(),
        ),
        subtitle: hls_subtitle_args(
            request.execution_policy.subtitle_strategy,
            &request.artifacts,
            request.segment_time_seconds,
        ),
        muxer: hls_adaptive_muxer_args(
            request.segment_time_seconds,
            &request.artifacts,
            request.playback_generation,
        ),
    })
}

fn validate_hls_subtitle_strategy(request: &HlsRequest) -> Result<()> {
    match request.execution_policy.subtitle_strategy {
        TranscodeSubtitleStrategy::None | TranscodeSubtitleStrategy::OmitSelected => {
            if request.artifacts.media_renditions().has_subtitles() {
                return Err(NakoError::InvalidInput {
                    message: "hls subtitle artifacts require sidecar-selected subtitle strategy"
                        .to_owned(),
                });
            }
            Ok(())
        }
        TranscodeSubtitleStrategy::SidecarSelected => {
            if request.artifacts.media_renditions().has_subtitles() {
                Ok(())
            } else {
                Err(NakoError::InvalidInput {
                    message: "sidecar-selected hls subtitle strategy requires subtitle artifacts"
                        .to_owned(),
                })
            }
        }
        TranscodeSubtitleStrategy::PreserveInContainer
        | TranscodeSubtitleStrategy::BurnInSelected => Err(NakoError::Unsupported(
            "hls subtitle strategy is not implemented by the ffmpeg adapter",
        )),
    }
}

fn hls_global_args(overwrite: FfmpegOverwritePolicy) -> Vec<FfmpegArg> {
    let overwrite_arg = match overwrite {
        FfmpegOverwritePolicy::Allow => "-y",
        FfmpegOverwritePolicy::Never => "-n",
    };

    vec![
        FfmpegArg::raw("-hide_banner"),
        FfmpegArg::raw("-loglevel"),
        FfmpegArg::raw("warning"),
        FfmpegArg::raw("-nostats"),
        FfmpegArg::raw("-progress"),
        FfmpegArg::raw("pipe:1"),
        FfmpegArg::raw(overwrite_arg),
    ]
}

fn hls_device_input_args(acceleration: TranscodeAccelerationPlan) -> Vec<FfmpegArg> {
    match acceleration.decode.accelerator {
        HardwareAcceleration::None | HardwareAcceleration::Nvenc | HardwareAcceleration::Amf => {
            Vec::new()
        }
        HardwareAcceleration::Vaapi => hwaccel_args("vaapi"),
        HardwareAcceleration::QuickSync => hwaccel_args("qsv"),
        HardwareAcceleration::VideoToolbox => hwaccel_args("videotoolbox"),
    }
}

fn hls_input_args(input_path: &Path, playback_generation: HlsPlaybackGeneration) -> Vec<FfmpegArg> {
    let mut args = Vec::new();
    if !playback_generation.is_default_start() {
        args.extend([
            FfmpegArg::raw("-ss"),
            FfmpegArg::raw(format_ffmpeg_timestamp_ms(
                playback_generation.start_position_ms(),
            )),
        ]);
    }
    args.extend([
        FfmpegArg::raw("-i"),
        FfmpegArg::path(input_path.to_path_buf()),
    ]);
    args
}

fn hls_stream_map_args(
    track_selection: TranscodeTrackSelection,
    main_output_has_audio: bool,
) -> Vec<FfmpegArg> {
    let mut args = vec![FfmpegArg::raw("-map"), FfmpegArg::raw("0:v:0")];
    if main_output_has_audio {
        args.extend([
            FfmpegArg::raw("-map"),
            FfmpegArg::raw(hls_audio_stream_map(track_selection)),
        ]);
    }
    args
}

fn hls_filter_graph_args(acceleration: TranscodeAccelerationPlan) -> Vec<FfmpegArg> {
    match acceleration.filter.accelerator {
        HardwareAcceleration::None
        | HardwareAcceleration::Nvenc
        | HardwareAcceleration::QuickSync
        | HardwareAcceleration::Amf
        | HardwareAcceleration::VideoToolbox => Vec::new(),
        HardwareAcceleration::Vaapi => vec![
            FfmpegArg::raw("-vf"),
            FfmpegArg::raw("format=nv12,hwupload"),
        ],
    }
}

fn hls_audio_filter_args(
    has_audio_output: bool,
    audio_filter_graph: Option<&str>,
) -> Vec<FfmpegArg> {
    if !has_audio_output {
        return Vec::new();
    }

    let Some(filter_graph) = audio_filter_graph else {
        return Vec::new();
    };

    vec![FfmpegArg::raw("-af"), FfmpegArg::raw(filter_graph)]
}

fn hls_audio_filter_graph(audio_output: TranscodeAudioOutputRequirement) -> Result<Option<String>> {
    let mut filters = Vec::new();

    if audio_output.downmix == TranscodeAudioDownmixRequirement::Required {
        filters.push(hls_audio_downmix_filter(audio_output)?);
    }

    if audio_output.normalization == TranscodeAudioNormalizationRequirement::Requested {
        filters.push("loudnorm=I=-16:TP=-1.5:LRA=11".to_owned());
    }

    if filters.is_empty() {
        Ok(None)
    } else {
        Ok(Some(filters.join(",")))
    }
}

fn hls_audio_downmix_filter(audio_output: TranscodeAudioOutputRequirement) -> Result<String> {
    let target_channels = audio_output
        .target_channels
        .or(audio_output.max_supported_channels)
        .ok_or_else(|| NakoError::InvalidInput {
            message: "hls audio downmix requires a target channel count".to_owned(),
        })?;
    let layout =
        hls_audio_channel_layout(target_channels).ok_or_else(|| NakoError::InvalidInput {
            message: format!(
                "hls audio downmix target channel count {target_channels} is not supported by ffmpeg filter planning"
            ),
        })?;

    Ok(format!("aformat=channel_layouts={layout}"))
}

fn hls_audio_channel_layout(target_channels: u32) -> Option<&'static str> {
    match target_channels {
        1 => Some("mono"),
        2 => Some("stereo"),
        3 => Some("2.1"),
        4 => Some("quad"),
        5 => Some("5.0"),
        6 => Some("5.1"),
        7 => Some("6.1"),
        8 => Some("7.1"),
        _ => None,
    }
}

fn hls_video_encoder_args(
    policy: TranscodeExecutionPolicy,
    playback_generation: HlsPlaybackGeneration,
    segment_time_seconds: u32,
) -> Vec<FfmpegArg> {
    let encoder = hls_video_encoder_name(policy.acceleration.encode.accelerator);
    let mut args = vec![FfmpegArg::raw("-c:v"), FfmpegArg::raw(encoder)];

    if let Some(max_video_bitrate) = policy.output_constraints.max_video_bitrate {
        args.push(FfmpegArg::raw("-maxrate"));
        args.push(FfmpegArg::raw(max_video_bitrate.to_string()));
        args.push(FfmpegArg::raw("-bufsize"));
        args.push(FfmpegArg::raw(
            max_video_bitrate.saturating_mul(2).to_string(),
        ));
    }

    args.extend(hls_seek_keyframe_args(
        playback_generation,
        segment_time_seconds,
    ));
    args
}

fn hls_adaptive_stream_map_args(
    rendition_count: usize,
    has_audio: bool,
    track_selection: TranscodeTrackSelection,
) -> Vec<FfmpegArg> {
    let mut args =
        Vec::with_capacity(rendition_count.saturating_mul(if has_audio { 4 } else { 2 }));
    let audio_stream_map = hls_audio_stream_map(track_selection);
    for _ in 0..rendition_count {
        args.extend([FfmpegArg::raw("-map"), FfmpegArg::raw("0:v:0")]);
        if has_audio {
            args.extend([
                FfmpegArg::raw("-map"),
                FfmpegArg::raw(audio_stream_map.clone()),
            ]);
        }
    }
    args
}

fn hls_audio_stream_map(track_selection: TranscodeTrackSelection) -> String {
    track_selection
        .audio_stream
        .map_or_else(|| "0:a:0?".to_owned(), |stream| format!("0:{stream}"))
}

fn hls_adaptive_video_encoder_args(
    policy: TranscodeExecutionPolicy,
    renditions: &[HlsRendition],
    playback_generation: HlsPlaybackGeneration,
    segment_time_seconds: u32,
) -> Vec<FfmpegArg> {
    let encoder = hls_video_encoder_name(policy.acceleration.encode.accelerator);
    let mut args = vec![FfmpegArg::raw("-c:v"), FfmpegArg::raw(encoder)];

    for (stream_index, rendition) in renditions.iter().enumerate() {
        let target_bitrate = policy
            .output_constraints
            .max_video_bitrate
            .map_or(rendition.video_bitrate, |max| {
                rendition.video_bitrate.min(max)
            });
        args.extend([
            FfmpegArg::raw(format!("-b:v:{stream_index}")),
            FfmpegArg::raw(target_bitrate.to_string()),
            FfmpegArg::raw(format!("-maxrate:v:{stream_index}")),
            FfmpegArg::raw(target_bitrate.to_string()),
            FfmpegArg::raw(format!("-bufsize:v:{stream_index}")),
            FfmpegArg::raw(target_bitrate.saturating_mul(2).to_string()),
            FfmpegArg::raw(format!("-s:v:{stream_index}")),
            FfmpegArg::raw(format!("{}x{}", rendition.width, rendition.height)),
        ]);
    }

    args.extend(hls_seek_keyframe_args(
        playback_generation,
        segment_time_seconds,
    ));
    args
}

fn hls_audio_encoder_args(main_output_has_audio: bool) -> Vec<FfmpegArg> {
    if !main_output_has_audio {
        return Vec::new();
    }

    vec![FfmpegArg::raw("-c:a"), FfmpegArg::raw("aac")]
}

fn hls_adaptive_audio_encoder_args(renditions: &[HlsRendition], has_audio: bool) -> Vec<FfmpegArg> {
    if !has_audio {
        return Vec::new();
    }

    let mut args = vec![FfmpegArg::raw("-c:a"), FfmpegArg::raw("aac")];
    for (stream_index, rendition) in renditions.iter().enumerate() {
        args.extend([
            FfmpegArg::raw(format!("-b:a:{stream_index}")),
            FfmpegArg::raw(rendition.audio_bitrate.to_string()),
        ]);
    }
    args
}

fn hls_audio_sidecar_args(
    artifacts: &HlsArtifactManifest,
    segment_time_seconds: u32,
    audio_filter_graph: Option<&str>,
) -> Vec<FfmpegArg> {
    let mut args = Vec::new();
    for audio in artifacts.media_renditions().audios() {
        args.extend([
            FfmpegArg::raw("-map"),
            FfmpegArg::raw(format!("0:{}", audio.source_stream_index)),
            FfmpegArg::raw("-vn"),
        ]);
        args.extend(hls_audio_filter_args(true, audio_filter_graph));
        args.extend([
            FfmpegArg::raw("-c:a"),
            FfmpegArg::raw("aac"),
            FfmpegArg::raw("-f"),
            FfmpegArg::raw("segment"),
            FfmpegArg::raw("-segment_time"),
            FfmpegArg::raw(segment_time_seconds.max(1).to_string()),
            FfmpegArg::raw("-segment_list"),
            FfmpegArg::path(audio.playlist_path(artifacts.output_dir())),
            FfmpegArg::raw("-segment_format"),
            FfmpegArg::raw("adts"),
            FfmpegArg::path(audio.segment_pattern_path(artifacts.output_dir())),
        ]);
    }
    args
}

fn hls_subtitle_args(
    strategy: TranscodeSubtitleStrategy,
    artifacts: &HlsArtifactManifest,
    segment_time_seconds: u32,
) -> Vec<FfmpegArg> {
    match strategy {
        TranscodeSubtitleStrategy::None | TranscodeSubtitleStrategy::OmitSelected => Vec::new(),
        TranscodeSubtitleStrategy::SidecarSelected => {
            hls_sidecar_subtitle_args(artifacts, segment_time_seconds)
        }
        TranscodeSubtitleStrategy::PreserveInContainer
        | TranscodeSubtitleStrategy::BurnInSelected => unreachable!(
            "unsupported hls subtitle strategy must be rejected before command construction"
        ),
    }
}

fn hls_sidecar_subtitle_args(
    artifacts: &HlsArtifactManifest,
    segment_time_seconds: u32,
) -> Vec<FfmpegArg> {
    let mut args = Vec::new();
    for subtitle in artifacts.media_renditions().subtitles() {
        args.extend([
            FfmpegArg::raw("-map"),
            FfmpegArg::raw(format!("0:{}", subtitle.source_stream_index)),
            FfmpegArg::raw("-c:s"),
            FfmpegArg::raw("webvtt"),
            FfmpegArg::raw("-f"),
            FfmpegArg::raw("segment"),
            FfmpegArg::raw("-segment_time"),
            FfmpegArg::raw(segment_time_seconds.max(1).to_string()),
            FfmpegArg::raw("-segment_list"),
            FfmpegArg::path(subtitle.playlist_path(artifacts.output_dir())),
            FfmpegArg::raw("-segment_format"),
            FfmpegArg::raw("webvtt"),
            FfmpegArg::path(subtitle.segment_pattern_path(artifacts.output_dir())),
        ]);
    }
    args
}

fn hls_muxer_args(
    segment_time_seconds: u32,
    segment_pattern: &Path,
    playlist_path: &Path,
    segment_container: HlsSegmentContainer,
    playback_generation: HlsPlaybackGeneration,
) -> Vec<FfmpegArg> {
    let mut args = hls_seek_timestamp_args(playback_generation);
    args.extend([
        FfmpegArg::raw("-f"),
        FfmpegArg::raw("hls"),
        FfmpegArg::raw("-hls_time"),
        FfmpegArg::raw(segment_time_seconds.max(1).to_string()),
        FfmpegArg::raw("-hls_playlist_type"),
        FfmpegArg::raw("vod"),
    ]);

    if !playback_generation.is_default_start() {
        args.extend([
            FfmpegArg::raw("-hls_flags"),
            FfmpegArg::raw("independent_segments"),
        ]);
    }

    if segment_container == HlsSegmentContainer::Fmp4 {
        args.extend([
            FfmpegArg::raw("-hls_segment_type"),
            FfmpegArg::raw("fmp4"),
            FfmpegArg::raw("-hls_fmp4_init_filename"),
            FfmpegArg::raw("init.mp4"),
        ]);
    }

    args.extend([
        FfmpegArg::raw("-hls_segment_filename"),
        FfmpegArg::path(segment_pattern.to_path_buf()),
        FfmpegArg::path(playlist_path.to_path_buf()),
    ]);
    args
}

fn hls_adaptive_muxer_args(
    segment_time_seconds: u32,
    artifacts: &HlsArtifactManifest,
    playback_generation: HlsPlaybackGeneration,
) -> Vec<FfmpegArg> {
    let master_playlist_name = artifacts
        .primary_playlist_path()
        .file_name()
        .and_then(|value| value.to_str())
        .expect("validated adaptive hls manifest must have a primary playlist file name")
        .to_owned();
    let variant_playlist_pattern = artifacts
        .variant_playlist_pattern()
        .expect("validated adaptive hls manifest must have a variant playlist pattern");
    let stream_map = artifacts
        .renditions()
        .iter()
        .enumerate()
        .map(|(stream_index, _)| {
            if artifacts.main_output_has_audio() {
                format!("v:{stream_index},a:{stream_index}")
            } else {
                format!("v:{stream_index}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut args = hls_seek_timestamp_args(playback_generation);
    args.extend([
        FfmpegArg::raw("-f"),
        FfmpegArg::raw("hls"),
        FfmpegArg::raw("-hls_time"),
        FfmpegArg::raw(segment_time_seconds.max(1).to_string()),
        FfmpegArg::raw("-hls_playlist_type"),
        FfmpegArg::raw("vod"),
    ]);

    if !playback_generation.is_default_start() {
        args.extend([
            FfmpegArg::raw("-hls_flags"),
            FfmpegArg::raw("independent_segments"),
        ]);
    }

    args.extend([
        FfmpegArg::raw("-hls_segment_type"),
        FfmpegArg::raw("fmp4"),
        FfmpegArg::raw("-hls_fmp4_init_filename"),
        FfmpegArg::raw(HLS_ADAPTIVE_FMP4_INIT_PATTERN),
        FfmpegArg::raw("-hls_segment_filename"),
        FfmpegArg::path(artifacts.media_segment_pattern().to_path_buf()),
        FfmpegArg::raw("-master_pl_name"),
        FfmpegArg::raw(master_playlist_name),
        FfmpegArg::raw("-var_stream_map"),
        FfmpegArg::raw(stream_map),
        FfmpegArg::path(variant_playlist_pattern.to_path_buf()),
    ]);
    args
}

fn hls_seek_keyframe_args(
    playback_generation: HlsPlaybackGeneration,
    segment_time_seconds: u32,
) -> Vec<FfmpegArg> {
    if playback_generation.is_default_start() {
        return Vec::new();
    }

    vec![
        FfmpegArg::raw("-force_key_frames"),
        FfmpegArg::raw(format!(
            "expr:gte(t,n_forced*{})",
            segment_time_seconds.max(1)
        )),
    ]
}

fn hls_seek_timestamp_args(playback_generation: HlsPlaybackGeneration) -> Vec<FfmpegArg> {
    if playback_generation.is_default_start() {
        return Vec::new();
    }

    vec![
        FfmpegArg::raw("-avoid_negative_ts"),
        FfmpegArg::raw("make_zero"),
    ]
}

fn format_ffmpeg_timestamp_ms(position_ms: u64) -> String {
    format!("{}.{:03}", position_ms / 1_000, position_ms % 1_000)
}

fn hls_video_encoder_name(acceleration: HardwareAcceleration) -> &'static str {
    match acceleration {
        HardwareAcceleration::None => "libx264",
        HardwareAcceleration::Vaapi => "h264_vaapi",
        HardwareAcceleration::Nvenc => "h264_nvenc",
        HardwareAcceleration::QuickSync => "h264_qsv",
        HardwareAcceleration::Amf => "h264_amf",
        HardwareAcceleration::VideoToolbox => "h264_videotoolbox",
    }
}

fn hwaccel_args(kind: &'static str) -> Vec<FfmpegArg> {
    vec![FfmpegArg::raw("-hwaccel"), FfmpegArg::raw(kind)]
}

pub(crate) fn stderr_message(stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_owned();

    if message.is_empty() {
        "ffmpeg remux process failed".to_owned()
    } else {
        message
    }
}
