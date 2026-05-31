use std::path::{Path, PathBuf};

use nako_core::{MediaSourceId, NakoError, Result};
use serde::{Deserialize, Serialize};

use crate::{
    HlsArtifactManifest, HlsPlaybackGeneration, HlsVariantPolicy, TranscodeExecutionPolicy,
    TranscodeSubtitleStrategy, TranscodeTrackSelection,
};

use super::{FfmpegArg, FfmpegCommandPlan, FfmpegOverwritePolicy, common::command_plan};

mod encoders;
mod filters;
mod input;
mod muxer;
mod seek;
mod sidecars;

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

pub(super) fn plan_hls_command(
    ffmpeg_path: &Path,
    request: &HlsRequest,
) -> Result<FfmpegCommandPlan> {
    validate_hls_request(request)?;

    let parts = plan_hls_command_parts(request)?;
    Ok(command_plan(ffmpeg_path, parts.into_args()))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct FfmpegHlsCommandParts {
    global: Vec<FfmpegArg>,
    device_input: Vec<FfmpegArg>,
    input: Vec<FfmpegArg>,
    stream_map: Vec<FfmpegArg>,
    filter_graph: Vec<FfmpegArg>,
    audio_filter: Vec<FfmpegArg>,
    video_encoder: Vec<FfmpegArg>,
    audio_encoder: Vec<FfmpegArg>,
    audio_sidecar: Vec<FfmpegArg>,
    subtitle: Vec<FfmpegArg>,
    muxer: Vec<FfmpegArg>,
}

impl FfmpegHlsCommandParts {
    #[must_use]
    fn into_args(self) -> Vec<FfmpegArg> {
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
    let audio_filter_graph =
        filters::hls_audio_filter_graph(request.execution_policy.audio_output)?;
    Ok(FfmpegHlsCommandParts {
        global: input::hls_global_args(request.overwrite),
        device_input: input::hls_device_input_args(request.execution_policy.acceleration),
        input: input::hls_input_args(&request.input_path, request.playback_generation),
        stream_map: input::hls_stream_map_args(request.track_selection, main_output_has_audio),
        filter_graph: filters::hls_filter_graph_args(request.execution_policy)?,
        audio_filter: filters::hls_audio_filter_args(
            main_output_has_audio,
            audio_filter_graph.as_deref(),
        ),
        video_encoder: encoders::hls_video_encoder_args(
            request.execution_policy,
            request.playback_generation,
            request.segment_time_seconds,
        ),
        audio_encoder: encoders::hls_audio_encoder_args(main_output_has_audio),
        audio_sidecar: sidecars::hls_audio_sidecar_args(
            &request.artifacts,
            request.segment_time_seconds,
            audio_filter_graph.as_deref(),
        ),
        subtitle: sidecars::hls_subtitle_args(
            request.execution_policy.subtitle_strategy,
            &request.artifacts,
            request.segment_time_seconds,
        ),
        muxer: muxer::hls_muxer_args(
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
    let audio_filter_graph =
        filters::hls_audio_filter_graph(request.execution_policy.audio_output)?;
    Ok(FfmpegHlsCommandParts {
        global: input::hls_global_args(request.overwrite),
        device_input: input::hls_device_input_args(request.execution_policy.acceleration),
        input: input::hls_input_args(&request.input_path, request.playback_generation),
        stream_map: input::hls_adaptive_stream_map_args(
            request.artifacts.renditions().len(),
            main_output_has_audio,
            request.track_selection,
        ),
        filter_graph: filters::hls_filter_graph_args(request.execution_policy)?,
        video_encoder: encoders::hls_adaptive_video_encoder_args(
            request.execution_policy,
            request.artifacts.renditions(),
            request.playback_generation,
            request.segment_time_seconds,
        ),
        audio_filter: filters::hls_audio_filter_args(
            main_output_has_audio,
            audio_filter_graph.as_deref(),
        ),
        audio_encoder: encoders::hls_adaptive_audio_encoder_args(
            request.artifacts.renditions(),
            main_output_has_audio,
        ),
        audio_sidecar: sidecars::hls_audio_sidecar_args(
            &request.artifacts,
            request.segment_time_seconds,
            audio_filter_graph.as_deref(),
        ),
        subtitle: sidecars::hls_subtitle_args(
            request.execution_policy.subtitle_strategy,
            &request.artifacts,
            request.segment_time_seconds,
        ),
        muxer: muxer::hls_adaptive_muxer_args(
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
