# playback hls subtitle burn in planning

## Goal

Carry subtitle intent through the playback -> transcode -> HLS planning
boundary so HLS runtime can distinguish subtitle sidecar output from burn-in
intent, keep request identity stable, and stop reconstructing subtitle
strategy only from track selection.

## What I already know

* `nako-playback` already owns `PlaybackSubtitleStrategy` with
  `OmitSelected`, `BurnInSelected`, and `SidecarSelected`.
* `build_transcode_requirement` currently defaults to `OmitSelected` whenever a
  subtitle stream is selected.
* `nako-server` HLS orchestration currently rebuilds subtitle handling from
  `track_selection` and promotes subtitle renditions to `SidecarSelected`
  whenever the media rendition plan contains subtitles.
* `nako-transcode::HlsRuntimePlanRequest` does not yet carry subtitle strategy.
* `nako-transcode::ffmpeg::hls` rejects `BurnInSelected`.
* `PlaybackTargetProfile` already tracks subtitle support and includes subtitle
  capability in profile identity.
* Transcode profile identity already includes `subtitle_strategy`, so request
  identity can distinguish different subtitle execution intents.
* FFmpeg subtitles filter docs expose `filename`, `si`, and `original_size`,
  which is enough for a later executable burn-in slice.

## Assumptions

* This slice is HLS-only.
* External subtitle download, OCR, and image-subtitle conversion stay out of
  scope.
* The first pass should create the seam for burn-in intent without widening the
  server API surface.
* Subtitle strategy should come from playback capability and request planning,
  not from ad hoc reconstruction at the HLS runtime boundary.

## Research References

* [`research/ffmpeg-subtitles-filter.md`](research/ffmpeg-subtitles-filter.md)
  - official FFmpeg subtitles filter options relevant to future burn-in
* [`research/current-hls-subtitle-flow.md`](research/current-hls-subtitle-flow.md)
  - current local code path and the runtime seam that still needs to be split

## Requirements (evolving)

* Carry subtitle strategy from playback requirement into HLS runtime planning.
* Keep `SidecarSelected` generating subtitle media renditions.
* Keep `BurnInSelected` from generating subtitle sidecar artifacts.
* Preserve subtitle strategy in execution policy, profile identity, and
  request identity.
* Add focused tests around strategy propagation and identity drift.

## Acceptance Criteria (evolving)

* A subtitle-selected playback requirement produces a subtitle strategy that
  distinguishes sidecar and burn-in intent.
* HLS runtime planning no longer reconstructs subtitle intent only from
  `track_selection`.
* Burn-in requests do not create subtitle sidecar artifacts in the HLS
  manifest/layout.
* Request identity changes when subtitle strategy or subtitle support changes.
* Focused tests cover the new strategy flow.

## Definition of Done

* Tests updated and passing.
* Formatting and diff checks passing.
* No unrelated API or route changes.

## Out of Scope

* Actual FFmpeg subtitle burn-in execution.
* Subtitle OCR or image-subtitle conversion.
* Public API or Admin contract changes.
* UI work.

## Technical Notes

* Relevant files inspected:
  * `crates/nako-playback/src/lib.rs`
  * `crates/nako-playback/src/capability.rs`
  * `crates/nako-transcode/src/pipeline.rs`
  * `crates/nako-transcode/src/ffmpeg/hls.rs`
  * `crates/nako-server/src/app/playback/selection.rs`
  * `crates/nako-server/src/app/playback/hls_flow.rs`
  * `crates/nako-server/src/app/playback/hls.rs`
* Local docs inspected:
  * `docs/architecture/PLAYBACK.md`
  * `docs/adr/0044-playback-capability-profile-planner.md`
  * `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  * `docs/adr/0049-source-aware-transcode-runtime.md`
  * `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
  * `docs/workstreams/playback-planner-transcode-value-vocabulary/DESIGN.md`
