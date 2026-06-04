# HLS Subtitle Burn-In Planning

## Goal

Add the first bounded HLS subtitle burn-in planning slice so unsupported
subtitle formats can be represented as typed playback/transcode requirements
without reintroducing ad hoc FFmpeg command assembly.

## Requirements

* Keep subtitle compatibility decisions typed in playback/transcode boundaries.
* Preserve the existing sidecar subtitle behavior for supported text subtitle
  outputs.
* Add a narrow burn-in planning path for unsupported subtitles without changing
  public routes in this slice.
* Keep FFmpeg command construction inside `nako-transcode`.
* Avoid server playback orchestration changes unless a blocker proves glue is
  unavoidable.

## Acceptance Criteria

* [ ] Playback/transcode can distinguish sidecar-capable subtitle output from
      burn-in-required output.
* [ ] HLS plan or pipeline tests cover the new burn-in planning decision.
* [ ] Exact command or planner tests prove no unsupported strategy silently
      falls through.
* [ ] Existing HLS sidecar behavior remains covered.
* [ ] Focused playback/transcode checks pass.

## Definition Of Done

* `cargo check -p nako-playback -p nako-transcode --tests`
* `cargo nextest run -p nako-transcode hls --no-fail-fast`
* `cargo nextest run -p nako-playback --no-fail-fast`
* `cargo fmt --all -- --check`
* `git diff --check`

## Technical Approach

Work within the existing Playback Capability, Transcode Pipeline, and HLS
FFmpeg planner seams. Prefer adding typed vocabulary and tests before expanding
runtime behavior. Do not modify server HLS orchestration unless the PRD is
revised to widen the task.

## Out Of Scope

* Public API, route, or generated client contract changes.
* HLS seek/restart behavior.
* LL-HLS/CMAF.
* HEVC/AV1 executable output.
* Full image-subtitle support beyond the first planning slice.

## Technical Notes

* Lane: `playback-transcode`.
* Authorized write scope:
  * `crates/nako-playback/src/**`
  * `crates/nako-transcode/src/**`
  * `docs/architecture/PLAYBACK.md`
* Forbidden scope:
  * `crates/nako-server/src/app/playback/**` unless a blocker is recorded first
  * Admin Web files
  * generated contracts

