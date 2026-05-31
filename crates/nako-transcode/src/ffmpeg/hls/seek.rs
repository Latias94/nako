use crate::HlsPlaybackGeneration;

use crate::ffmpeg::FfmpegArg;

pub(super) fn hls_seek_keyframe_args(
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

pub(super) fn hls_seek_timestamp_args(
    playback_generation: HlsPlaybackGeneration,
) -> Vec<FfmpegArg> {
    if playback_generation.is_default_start() {
        return Vec::new();
    }

    vec![
        FfmpegArg::raw("-avoid_negative_ts"),
        FfmpegArg::raw("make_zero"),
    ]
}

pub(super) fn format_ffmpeg_timestamp_ms(position_ms: u64) -> String {
    format!("{}.{:03}", position_ms / 1_000, position_ms % 1_000)
}
