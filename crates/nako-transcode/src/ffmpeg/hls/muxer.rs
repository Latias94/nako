use std::path::Path;

use crate::{HLS_ADAPTIVE_FMP4_INIT_PATTERN, HlsArtifactManifest, HlsSegmentContainer};

use crate::ffmpeg::FfmpegArg;

use super::seek::HlsSeekCommandPlan;

pub(super) fn hls_muxer_args(
    segment_time_seconds: u32,
    segment_pattern: &Path,
    playlist_path: &Path,
    segment_container: HlsSegmentContainer,
    seek: HlsSeekCommandPlan,
) -> Vec<FfmpegArg> {
    let mut args = seek.timestamp_args();
    args.extend([
        FfmpegArg::raw("-f"),
        FfmpegArg::raw("hls"),
        FfmpegArg::raw("-hls_time"),
        FfmpegArg::raw(segment_time_seconds.max(1).to_string()),
        FfmpegArg::raw("-hls_playlist_type"),
        FfmpegArg::raw("vod"),
    ]);

    args.extend(seek.hls_flags_args());

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

pub(super) fn hls_adaptive_muxer_args(
    segment_time_seconds: u32,
    artifacts: &HlsArtifactManifest,
    seek: HlsSeekCommandPlan,
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

    let mut args = seek.timestamp_args();
    args.extend([
        FfmpegArg::raw("-f"),
        FfmpegArg::raw("hls"),
        FfmpegArg::raw("-hls_time"),
        FfmpegArg::raw(segment_time_seconds.max(1).to_string()),
        FfmpegArg::raw("-hls_playlist_type"),
        FfmpegArg::raw("vod"),
    ]);

    args.extend(seek.hls_flags_args());

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
