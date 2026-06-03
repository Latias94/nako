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

#[derive(Clone, Debug, Eq, PartialEq)]
struct FfmpegHlsCommandParts {
    input: FfmpegHlsInputParts,
    primary_output: FfmpegHlsPrimaryOutputParts,
    sidecar_outputs: FfmpegHlsSidecarOutputParts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FfmpegHlsInputParts {
    global: Vec<FfmpegArg>,
    device_input: Vec<FfmpegArg>,
    input: Vec<FfmpegArg>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FfmpegHlsPrimaryOutputParts {
    stream_map: Vec<FfmpegArg>,
    filter_graph: Vec<FfmpegArg>,
    audio_filter: Vec<FfmpegArg>,
    video_encoder: Vec<FfmpegArg>,
    audio_encoder: Vec<FfmpegArg>,
    muxer: Vec<FfmpegArg>,
}

impl FfmpegHlsCommandParts {
    #[must_use]
    fn into_args(self) -> Vec<FfmpegArg> {
        let capacity = self.input.len() + self.primary_output.len() + self.sidecar_outputs.len();
        let mut args = Vec::with_capacity(capacity);
        self.input.append_to(&mut args);
        self.primary_output.append_to(&mut args);
        self.sidecar_outputs.append_to(&mut args);
        args
    }
}

impl FfmpegHlsInputParts {
    #[must_use]
    fn from_request(request: &HlsRequest) -> Self {
        Self {
            global: input::hls_global_args(request.overwrite),
            device_input: input::hls_device_input_args(request.execution_policy.acceleration),
            input: input::hls_input_args(&request.input_path, request.playback_generation),
        }
    }

    #[must_use]
    fn len(&self) -> usize {
        self.global.len() + self.device_input.len() + self.input.len()
    }

    fn append_to(self, args: &mut Vec<FfmpegArg>) {
        let Self {
            global,
            device_input,
            input,
        } = self;
        args.extend(global);
        args.extend(device_input);
        args.extend(input);
    }
}

impl FfmpegHlsPrimaryOutputParts {
    #[must_use]
    fn len(&self) -> usize {
        self.stream_map.len()
            + self.filter_graph.len()
            + self.audio_filter.len()
            + self.video_encoder.len()
            + self.audio_encoder.len()
            + self.muxer.len()
    }

    fn append_to(self, args: &mut Vec<FfmpegArg>) {
        let Self {
            stream_map,
            filter_graph,
            audio_filter,
            video_encoder,
            audio_encoder,
            muxer,
        } = self;
        args.extend(stream_map);
        args.extend(filter_graph);
        args.extend(audio_filter);
        args.extend(video_encoder);
        args.extend(audio_encoder);
        args.extend(muxer);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FfmpegHlsSidecarOutputParts {
    audio_sidecar: Vec<FfmpegArg>,
    subtitle: Vec<FfmpegArg>,
}

impl FfmpegHlsSidecarOutputParts {
    #[must_use]
    fn from_request(request: &HlsRequest, output: &HlsOutputAssemblyContext) -> Self {
        Self {
            audio_sidecar: sidecars::hls_audio_sidecar_args(
                &request.artifacts,
                request.segment_time_seconds,
                output.audio_filter_graph(),
            ),
            subtitle: sidecars::hls_subtitle_args(
                request.execution_policy.subtitle_strategy,
                &request.artifacts,
                request.segment_time_seconds,
            ),
        }
    }

    #[must_use]
    fn len(&self) -> usize {
        self.audio_sidecar.len() + self.subtitle.len()
    }

    fn append_to(self, args: &mut Vec<FfmpegArg>) {
        let Self {
            audio_sidecar,
            subtitle,
        } = self;
        args.extend(audio_sidecar);
        args.extend(subtitle);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HlsOutputAssemblyContext {
    main_output_has_audio: bool,
    audio_filter_graph: Option<String>,
}

impl HlsOutputAssemblyContext {
    fn from_request(request: &HlsRequest) -> Result<Self> {
        Ok(Self {
            main_output_has_audio: request.artifacts.main_output_has_audio(),
            audio_filter_graph: filters::hls_audio_filter_graph(
                request.execution_policy.audio_output,
            )?,
        })
    }

    #[must_use]
    fn audio_filter_graph(&self) -> Option<&str> {
        self.audio_filter_graph.as_deref()
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
    let output = HlsOutputAssemblyContext::from_request(request)?;
    Ok(FfmpegHlsCommandParts {
        input: FfmpegHlsInputParts::from_request(request),
        primary_output: single_variant_primary_output_parts(request, &output)?,
        sidecar_outputs: FfmpegHlsSidecarOutputParts::from_request(request, &output),
    })
}

fn plan_adaptive_hls_command_parts(request: &HlsRequest) -> Result<FfmpegHlsCommandParts> {
    let output = HlsOutputAssemblyContext::from_request(request)?;
    Ok(FfmpegHlsCommandParts {
        input: FfmpegHlsInputParts::from_request(request),
        primary_output: adaptive_primary_output_parts(request, &output)?,
        sidecar_outputs: FfmpegHlsSidecarOutputParts::from_request(request, &output),
    })
}

fn single_variant_primary_output_parts(
    request: &HlsRequest,
    output: &HlsOutputAssemblyContext,
) -> Result<FfmpegHlsPrimaryOutputParts> {
    Ok(FfmpegHlsPrimaryOutputParts {
        stream_map: input::hls_stream_map_args(
            request.track_selection,
            output.main_output_has_audio,
        ),
        filter_graph: filters::hls_filter_graph_args(request.execution_policy)?,
        audio_filter: filters::hls_audio_filter_args(
            output.main_output_has_audio,
            output.audio_filter_graph(),
        ),
        video_encoder: encoders::hls_video_encoder_args(
            request.execution_policy,
            request.playback_generation,
            request.segment_time_seconds,
        ),
        audio_encoder: encoders::hls_audio_encoder_args(output.main_output_has_audio),
        muxer: muxer::hls_muxer_args(
            request.segment_time_seconds,
            request.artifacts.media_segment_pattern(),
            request.artifacts.primary_playlist_path(),
            request.artifacts.output().segment_container,
            request.playback_generation,
        ),
    })
}

fn adaptive_primary_output_parts(
    request: &HlsRequest,
    output: &HlsOutputAssemblyContext,
) -> Result<FfmpegHlsPrimaryOutputParts> {
    Ok(FfmpegHlsPrimaryOutputParts {
        stream_map: input::hls_adaptive_stream_map_args(
            request.artifacts.renditions().len(),
            output.main_output_has_audio,
            request.track_selection,
        ),
        filter_graph: filters::hls_filter_graph_args(request.execution_policy)?,
        audio_filter: filters::hls_audio_filter_args(
            output.main_output_has_audio,
            output.audio_filter_graph(),
        ),
        video_encoder: encoders::hls_adaptive_video_encoder_args(
            request.execution_policy,
            request.artifacts.renditions(),
            request.playback_generation,
            request.segment_time_seconds,
        ),
        audio_encoder: encoders::hls_adaptive_audio_encoder_args(
            request.artifacts.renditions(),
            output.main_output_has_audio,
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
