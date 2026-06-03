use std::path::Path;

use crate::{HardwareAcceleration, TranscodeAccelerationPlan, TranscodeTrackSelection};

use crate::ffmpeg::{FfmpegArg, FfmpegOverwritePolicy, common::overwrite_arg};

use super::seek::HlsSeekCommandPlan;

pub(super) fn hls_global_args(overwrite: FfmpegOverwritePolicy) -> Vec<FfmpegArg> {
    vec![
        FfmpegArg::raw("-hide_banner"),
        FfmpegArg::raw("-loglevel"),
        FfmpegArg::raw("warning"),
        FfmpegArg::raw("-nostats"),
        FfmpegArg::raw("-progress"),
        FfmpegArg::raw("pipe:1"),
        FfmpegArg::raw(overwrite_arg(overwrite)),
    ]
}

pub(super) fn hls_device_input_args(acceleration: TranscodeAccelerationPlan) -> Vec<FfmpegArg> {
    match acceleration.decode.accelerator {
        HardwareAcceleration::None | HardwareAcceleration::Nvenc | HardwareAcceleration::Amf => {
            Vec::new()
        }
        HardwareAcceleration::Vaapi => hwaccel_args("vaapi"),
        HardwareAcceleration::QuickSync => hwaccel_args("qsv"),
        HardwareAcceleration::VideoToolbox => hwaccel_args("videotoolbox"),
    }
}

pub(super) fn hls_input_args(input_path: &Path, seek: HlsSeekCommandPlan) -> Vec<FfmpegArg> {
    let mut args = seek.input_args();
    args.extend([
        FfmpegArg::raw("-i"),
        FfmpegArg::path(input_path.to_path_buf()),
    ]);
    args
}

pub(super) fn hls_stream_map_args(
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

pub(super) fn hls_adaptive_stream_map_args(
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

fn hwaccel_args(kind: &'static str) -> Vec<FfmpegArg> {
    vec![FfmpegArg::raw("-hwaccel"), FfmpegArg::raw(kind)]
}
