use std::path::Path;

use nako_core::{NakoError, Result};

use crate::HlsSubtitleBurnInPlan;
use crate::{
    HardwareAcceleration, TranscodeAccelerationPlan, TranscodeAudioDownmixRequirement,
    TranscodeAudioNormalizationRequirement, TranscodeAudioOutputRequirement,
    TranscodeColorPipelineRequirement, TranscodeExecutionPolicy, TranscodeSubtitleStrategy,
};

use crate::ffmpeg::FfmpegArg;

const HLS_HDR_TO_SDR_TONE_MAPPING_FILTER: &str = "zscale=transfer=linear:npl=100,tonemap=tonemap=hable:desat=0,zscale=transfer=bt709:matrix=bt709:primaries=bt709:range=tv,format=yuv420p";

pub(super) fn hls_filter_graph_args(
    policy: TranscodeExecutionPolicy,
    input_path: &Path,
    subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
) -> Result<Vec<FfmpegArg>> {
    if let Some(filter_graph) = hls_video_filter_graph(policy, input_path, subtitle_burn_in)? {
        return Ok(vec![FfmpegArg::raw("-vf"), FfmpegArg::raw(filter_graph)]);
    }

    Ok(hls_hardware_filter_graph_args(policy.acceleration))
}

pub(super) fn hls_audio_filter_args(
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

pub(super) fn hls_audio_filter_graph(
    audio_output: TranscodeAudioOutputRequirement,
) -> Result<Option<String>> {
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

fn hls_hardware_filter_graph_args(acceleration: TranscodeAccelerationPlan) -> Vec<FfmpegArg> {
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

fn hls_color_filter_graph(
    color_pipeline: TranscodeColorPipelineRequirement,
    acceleration: TranscodeAccelerationPlan,
) -> Result<Option<String>> {
    if color_pipeline.is_deferred_unsupported() {
        return Err(NakoError::Unsupported(
            "hls hdr tone mapping for deferred dynamic hdr formats is not implemented",
        ));
    }

    if !color_pipeline.requires_hdr_to_sdr_tone_mapping() {
        return Ok(None);
    }

    if !acceleration.is_software_only() {
        return Err(NakoError::Unsupported(
            "hdr-to-sdr tone mapping requires the software transcode pipeline",
        ));
    }

    Ok(Some(HLS_HDR_TO_SDR_TONE_MAPPING_FILTER.to_owned()))
}

fn hls_video_filter_graph(
    policy: TranscodeExecutionPolicy,
    input_path: &Path,
    subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
) -> Result<Option<String>> {
    let mut filters = Vec::new();
    if let Some(filter) = hls_color_filter_graph(policy.color_pipeline, policy.acceleration)? {
        filters.push(filter);
    }
    if let Some(filter) = hls_subtitle_burn_in_filter(
        policy.subtitle_strategy,
        input_path,
        subtitle_burn_in,
        policy.acceleration,
    )? {
        filters.push(filter);
    }

    if filters.is_empty() {
        Ok(None)
    } else {
        Ok(Some(filters.join(",")))
    }
}

fn hls_subtitle_burn_in_filter(
    subtitle_strategy: TranscodeSubtitleStrategy,
    input_path: &Path,
    subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
    acceleration: TranscodeAccelerationPlan,
) -> Result<Option<String>> {
    if subtitle_strategy != TranscodeSubtitleStrategy::BurnInSelected {
        return Ok(None);
    }
    if !acceleration.is_software_only() {
        return Err(NakoError::Unsupported(
            "hls subtitle burn-in requires the software transcode pipeline",
        ));
    }
    let Some(subtitle_burn_in) = subtitle_burn_in else {
        return Err(NakoError::InvalidInput {
            message: "hls subtitle burn-in requires a filter stream plan".to_owned(),
        });
    };

    Ok(Some(format!(
        "subtitles='{}':si={}",
        escaped_subtitles_filter_path(input_path),
        subtitle_burn_in.filter_stream_index
    )))
}

fn escaped_subtitles_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace(':', "\\:")
        .replace(',', "\\,")
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
