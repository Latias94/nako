use crate::HlsPlaybackGeneration;

use crate::ffmpeg::FfmpegArg;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct HlsSeekCommandPlan {
    playback_generation: HlsPlaybackGeneration,
    segment_time_seconds: u32,
}

impl HlsSeekCommandPlan {
    #[must_use]
    pub(super) fn new(
        playback_generation: HlsPlaybackGeneration,
        segment_time_seconds: u32,
    ) -> Self {
        Self {
            playback_generation,
            segment_time_seconds: segment_time_seconds.max(1),
        }
    }

    #[must_use]
    pub(super) fn input_args(self) -> Vec<FfmpegArg> {
        if self.playback_generation.is_default_start() {
            return Vec::new();
        }

        vec![
            FfmpegArg::raw("-ss"),
            FfmpegArg::raw(format_ffmpeg_timestamp_ms(
                self.playback_generation.start_position_ms(),
            )),
        ]
    }

    #[must_use]
    pub(super) fn keyframe_args(self) -> Vec<FfmpegArg> {
        if self.playback_generation.is_default_start() {
            return Vec::new();
        }

        vec![
            FfmpegArg::raw("-force_key_frames"),
            FfmpegArg::raw(format!(
                "expr:gte(t,n_forced*{})",
                self.segment_time_seconds
            )),
        ]
    }

    #[must_use]
    pub(super) fn timestamp_args(self) -> Vec<FfmpegArg> {
        if self.playback_generation.is_default_start() {
            return Vec::new();
        }

        vec![
            FfmpegArg::raw("-avoid_negative_ts"),
            FfmpegArg::raw("make_zero"),
        ]
    }

    #[must_use]
    pub(super) fn hls_flags_args(self) -> Vec<FfmpegArg> {
        if self.playback_generation.is_default_start() {
            return Vec::new();
        }

        vec![
            FfmpegArg::raw("-hls_flags"),
            FfmpegArg::raw("independent_segments"),
        ]
    }
}

fn format_ffmpeg_timestamp_ms(position_ms: u64) -> String {
    format!("{}.{:03}", position_ms / 1_000, position_ms % 1_000)
}
