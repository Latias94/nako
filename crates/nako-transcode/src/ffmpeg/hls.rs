use std::path::{Path, PathBuf};

use nako_core::{MediaSourceId, NakoError, Result};
use serde::{Deserialize, Serialize};

use crate::{
    HlsArtifactManifest, HlsPlaybackGeneration, HlsSubtitleBurnInPlan, HlsVariantPolicy,
    TranscodeExecutionPolicy, TranscodeSubtitleStrategy, TranscodeTrackSelection,
};

use super::{FfmpegArg, FfmpegCommandPlan, FfmpegOverwritePolicy, common::command_plan};

mod encoders;
mod filters;
mod input;
mod muxer;
mod seek;
mod sidecars;

use seek::HlsSeekCommandPlan;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HlsRequest {
    pub(crate) source_id: MediaSourceId,
    pub(crate) input_path: PathBuf,
    pub(crate) playback_generation: HlsPlaybackGeneration,
    pub(crate) artifacts: HlsArtifactManifest,
    pub(crate) segment_time_seconds: u32,
    pub(crate) track_selection: TranscodeTrackSelection,
    pub(crate) subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
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
    fn from_request(request: &HlsRequest, context: &HlsCommandAssemblyContext) -> Self {
        Self {
            global: input::hls_global_args(request.overwrite),
            device_input: input::hls_device_input_args(request.execution_policy.acceleration),
            input: input::hls_input_args(&request.input_path, context.seek),
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
    fn from_request(request: &HlsRequest, context: &HlsCommandAssemblyContext) -> Self {
        Self {
            audio_sidecar: sidecars::hls_audio_sidecar_args(
                &request.artifacts,
                request.segment_time_seconds,
                context.audio_filter_graph(),
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
struct HlsCommandAssemblyContext {
    main_output_has_audio: bool,
    audio_filter_graph: Option<String>,
    seek: HlsSeekCommandPlan,
}

impl HlsCommandAssemblyContext {
    fn from_request(request: &HlsRequest) -> Result<Self> {
        Ok(Self {
            main_output_has_audio: request.artifacts.main_output_has_audio(),
            audio_filter_graph: filters::hls_audio_filter_graph(
                request.execution_policy.audio_output,
            )?,
            seek: HlsSeekCommandPlan::new(
                request.playback_generation,
                request.segment_time_seconds,
            ),
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
    let context = HlsCommandAssemblyContext::from_request(request)?;
    Ok(FfmpegHlsCommandParts {
        input: FfmpegHlsInputParts::from_request(request, &context),
        primary_output: single_variant_primary_output_parts(request, &context)?,
        sidecar_outputs: FfmpegHlsSidecarOutputParts::from_request(request, &context),
    })
}

fn plan_adaptive_hls_command_parts(request: &HlsRequest) -> Result<FfmpegHlsCommandParts> {
    let context = HlsCommandAssemblyContext::from_request(request)?;
    Ok(FfmpegHlsCommandParts {
        input: FfmpegHlsInputParts::from_request(request, &context),
        primary_output: adaptive_primary_output_parts(request, &context)?,
        sidecar_outputs: FfmpegHlsSidecarOutputParts::from_request(request, &context),
    })
}

fn single_variant_primary_output_parts(
    request: &HlsRequest,
    context: &HlsCommandAssemblyContext,
) -> Result<FfmpegHlsPrimaryOutputParts> {
    Ok(FfmpegHlsPrimaryOutputParts {
        stream_map: input::hls_stream_map_args(
            request.track_selection,
            context.main_output_has_audio,
        ),
        filter_graph: filters::hls_filter_graph_args(
            request.execution_policy,
            &request.input_path,
            request.subtitle_burn_in,
        )?,
        audio_filter: filters::hls_audio_filter_args(
            context.main_output_has_audio,
            context.audio_filter_graph(),
        ),
        video_encoder: encoders::hls_video_encoder_args(request.execution_policy, context.seek),
        audio_encoder: encoders::hls_audio_encoder_args(context.main_output_has_audio),
        muxer: muxer::hls_muxer_args(
            request.segment_time_seconds,
            request.artifacts.media_segment_pattern(),
            request.artifacts.primary_playlist_path(),
            request.artifacts.output().segment_container,
            context.seek,
        ),
    })
}

fn adaptive_primary_output_parts(
    request: &HlsRequest,
    context: &HlsCommandAssemblyContext,
) -> Result<FfmpegHlsPrimaryOutputParts> {
    Ok(FfmpegHlsPrimaryOutputParts {
        stream_map: input::hls_adaptive_stream_map_args(
            request.artifacts.renditions().len(),
            context.main_output_has_audio,
            request.track_selection,
        ),
        filter_graph: filters::hls_filter_graph_args(
            request.execution_policy,
            &request.input_path,
            request.subtitle_burn_in,
        )?,
        audio_filter: filters::hls_audio_filter_args(
            context.main_output_has_audio,
            context.audio_filter_graph(),
        ),
        video_encoder: encoders::hls_adaptive_video_encoder_args(
            request.execution_policy,
            request.artifacts.renditions(),
            context.seek,
        ),
        audio_encoder: encoders::hls_adaptive_audio_encoder_args(
            request.artifacts.renditions(),
            context.main_output_has_audio,
        ),
        muxer: muxer::hls_adaptive_muxer_args(
            request.segment_time_seconds,
            &request.artifacts,
            context.seek,
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
        TranscodeSubtitleStrategy::BurnInSelected => {
            if request.artifacts.media_renditions().has_subtitles() {
                return Err(NakoError::InvalidInput {
                    message: "hls subtitle artifacts require sidecar-selected subtitle strategy"
                        .to_owned(),
                });
            }
            if request.track_selection.subtitle_stream.is_none() {
                return Err(NakoError::InvalidInput {
                    message: "hls subtitle burn-in requires a selected subtitle stream".to_owned(),
                });
            }
            let Some(subtitle_burn_in) = request.subtitle_burn_in else {
                return Err(NakoError::InvalidInput {
                    message: "hls subtitle burn-in requires a filter stream plan".to_owned(),
                });
            };
            if subtitle_burn_in.source_stream_index
                != request
                    .track_selection
                    .subtitle_stream
                    .expect("selected subtitle was checked above")
            {
                return Err(NakoError::InvalidInput {
                    message: "hls subtitle burn-in filter stream does not match selected subtitle"
                        .to_owned(),
                });
            }
            if !request.execution_policy.acceleration.is_software_only() {
                return Err(NakoError::Unsupported(
                    "hls subtitle burn-in requires the software transcode pipeline",
                ));
            }
            Ok(())
        }
        TranscodeSubtitleStrategy::PreserveInContainer => Err(NakoError::Unsupported(
            "hls subtitle strategy is not implemented by the ffmpeg adapter",
        )),
    }
}
