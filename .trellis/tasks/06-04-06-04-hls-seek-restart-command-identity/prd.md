# HLS seek restart command identity

## Goal

Make HLS seek/restart FFmpeg command planning derive all seek-related input,
encoder, and muxer arguments from one typed command plan, so start-position
identity cannot drift across `-ss`, keyframe forcing, timestamp reset, and HLS
segment flags.

## What I already know

* `HlsPlaybackGeneration` already carries `start_position_ms` and participates
  in HLS request variant identity.
* Current command planning emits seek-related args from three modules:
  `input.rs`, `encoders.rs`, and `muxer.rs`.
* `seek.rs` already contains helper functions, but each caller still passes
  generation and checks default-start behavior independently.
* Existing exact argv coverage proves `-ss` appears before `-i` and non-default
  starts add timestamp/keyframe/segment flags.
* Admission and subtitle burn-in follow-on slices are already archived.

## Requirements

* Introduce a typed HLS seek command plan derived once per HLS request.
* Use that plan for input seek args, encoder keyframe args, muxer timestamp args,
  and HLS seek segment flags.
* Preserve current command behavior for default-start and non-default-start HLS.
* Preserve single-variant and adaptive HLS command shape.
* Do not change public routes, DTOs, playlist query parsing, session persistence,
  or playback planner selection.

## Acceptance Criteria

* [ ] Non-default HLS start still emits `-ss <seconds.millis>` before `-i`.
* [ ] Non-default HLS start still emits `-force_key_frames`, `-avoid_negative_ts
  make_zero`, and `-hls_flags independent_segments`.
* [ ] Default HLS start emits none of those seek-specific args.
* [ ] Single-variant and adaptive primary HLS output builders both consume the
  same request-derived seek command plan.
* [ ] Focused `nako-transcode` HLS tests and formatting/diff checks pass.

## Definition of Done

* Tests updated and passing.
* `cargo fmt --all -- --check` and `git diff --check` pass.
* `cargo nextest run -p nako-transcode hls --no-fail-fast` passes.
* `cargo check -p nako-transcode --tests` passes.
* Spec updated if the new seek command-plan locality becomes a reusable rule.
* Conventional Commit, task archive, and journal entry are recorded.

## Out of Scope

* Public `start_position_ms` route/query changes.
* Accurate-vs-fast seek policy changes.
* GOP/keyframe policy beyond preserving current forced-keyframe cadence.
* Session supersede/admission behavior changes.
* LL-HLS/CMAF, player UI, or external worker changes.

## Technical Approach

* Replace standalone seek helper inputs with a small `HlsSeekCommandPlan`.
* Construct the seek plan once from `HlsPlaybackGeneration` and
  `segment_time_seconds` in HLS command assembly.
* Pass the plan to input, encoder, and muxer part builders.
* Keep all FFmpeg args as `Vec<FfmpegArg>`.

## Research References

* [`research/current-hls-seek-command-state.md`](research/current-hls-seek-command-state.md)
  - current local command builder state and bounded implementation seam.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/nako-transcode/backend/index.md`
  * `.trellis/spec/nako-transcode/backend/directory-structure.md`
  * `.trellis/spec/nako-transcode/backend/error-handling.md`
  * `.trellis/spec/nako-transcode/backend/quality-guidelines.md`
* Relevant files:
  * `crates/nako-transcode/src/ffmpeg/hls.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/seek.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/input.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/encoders.rs`
  * `crates/nako-transcode/src/ffmpeg/hls/muxer.rs`
  * `crates/nako-transcode/src/lib.rs`
