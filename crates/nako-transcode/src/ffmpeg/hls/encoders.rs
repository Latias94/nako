use crate::{HardwareAcceleration, HlsPlaybackGeneration, HlsRendition, TranscodeExecutionPolicy};

use crate::ffmpeg::FfmpegArg;

use super::seek::hls_seek_keyframe_args;

pub(super) fn hls_video_encoder_args(
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

pub(super) fn hls_adaptive_video_encoder_args(
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

pub(super) fn hls_audio_encoder_args(main_output_has_audio: bool) -> Vec<FfmpegArg> {
    if !main_output_has_audio {
        return Vec::new();
    }

    vec![FfmpegArg::raw("-c:a"), FfmpegArg::raw("aac")]
}

pub(super) fn hls_adaptive_audio_encoder_args(
    renditions: &[HlsRendition],
    has_audio: bool,
) -> Vec<FfmpegArg> {
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
