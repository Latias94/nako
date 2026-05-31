use crate::{HlsArtifactManifest, TranscodeSubtitleStrategy};

use crate::ffmpeg::FfmpegArg;

use super::filters::hls_audio_filter_args;

pub(super) fn hls_audio_sidecar_args(
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

pub(super) fn hls_subtitle_args(
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
