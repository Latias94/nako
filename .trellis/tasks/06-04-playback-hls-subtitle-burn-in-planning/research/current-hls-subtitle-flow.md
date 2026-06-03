# Current HLS Subtitle Flow

## Observed Code Path

* `crates/nako-playback/src/lib.rs`
  * `build_transcode_requirement` currently maps any selected subtitle stream to
    `PlaybackSubtitleStrategy::OmitSelected`.
* `crates/nako-server/src/app/playback/selection.rs`
  * `hls_runtime_plan_request` forwards the playback transcode requirement into
    the transcode runtime request.
* `crates/nako-transcode/src/pipeline.rs`
  * `HlsRuntimePlanRequest` does not yet carry subtitle strategy.
  * `plan_hls_runtime` currently sets `SidecarSelected` whenever subtitle
    renditions are present, regardless of playback intent.
* `crates/nako-transcode/src/ffmpeg/hls.rs`
  * HLS FFmpeg planning currently allows sidecar subtitles only and rejects
    `BurnInSelected`.
* `crates/nako-server/src/app/playback/hls.rs`
  * HLS runtime execution is still built from `track_selection` plus execution
    policy, not from explicit subtitle intent.

## Practical Consequence

* The runtime boundary is still reconstructing subtitle behavior from track
  selection and media renditions.
* That is the seam this task is meant to split.

## Relevant Local Files

* `crates/nako-playback/src/lib.rs`
* `crates/nako-playback/src/capability.rs`
* `crates/nako-transcode/src/pipeline.rs`
* `crates/nako-transcode/src/ffmpeg/hls.rs`
* `crates/nako-server/src/app/playback/selection.rs`
* `crates/nako-server/src/app/playback/hls_flow.rs`
* `crates/nako-server/src/app/playback/hls.rs`
