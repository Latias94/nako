# HEVC AV1 HLS Output Policy Evidence

Date: 2026-06-04

## Scope

- Added typed HLS video output policy vocabulary in `nako-transcode`.
- Recognized H264, HEVC/H265, and AV1 in profile validation.
- Kept H264/AAC as the only executable HLS output profile.
- Kept HEVC/AV1 FFmpeg encoder argv, server runtime, public API, schema, and
  playback defaults unchanged.

## Verification

- `cargo fmt --all`: passed after formatting.
- `cargo fmt --all -- --check`: passed.
- `cargo check -p nako-transcode --tests`: passed.
- `cargo nextest run -p nako-transcode hls_video_output --no-fail-fast`:
  passed, 1 test.
- `cargo nextest run -p nako-transcode transcode_profile_validation --no-fail-fast`:
  passed, 7 tests.
- `cargo nextest run -p nako-transcode hls_video_output transcode_profile_validation --no-fail-fast`:
  passed, 8 tests after the final readability adjustment.
- `cargo nextest run -p nako-transcode hls --no-fail-fast`: passed, 73 tests.
- `git diff --check`: passed with LF/CRLF normalization warnings only.
- `python .\.trellis\scripts\task.py validate 06-04-06-04-hevc-av1-hls-output-policy-first-slice`:
  passed.

## Spec And Architecture Sync

- `.trellis/spec/nako-transcode/backend/quality-guidelines.md` now records the
  HLS output codec policy contract.
- `docs/architecture/PLAYBACK.md` now records the shipped typed policy seam and
  keeps HEVC/AV1 FFmpeg execution as a follow-on.
