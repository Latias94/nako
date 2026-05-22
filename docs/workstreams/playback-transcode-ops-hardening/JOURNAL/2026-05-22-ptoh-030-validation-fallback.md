# PTOH-030 — Validation And Fallback Reasons

Date: 2026-05-22
Task: PTOH-030
Status: completed

## Summary

Added typed playback transcode validation before session creation or FFmpeg
execution. Validation ownership stays in `taru-transcode`; `taru-streaming`
exposes Result-returning profile construction seams; `taru-server` now calls
those seams before deriving request identity, staging paths, or creating
playback transcode sessions.

## Code Changes

- Added `TranscodeProfileValidationReason` and
  `TranscodeProfileValidationError` for remux/HLS profile facts.
- Added `TranscodePlanValidationReason` and `TranscodePlanValidationError`
  for playback transcode plan facts.
- Added `PlaybackProfile::try_remux_transcode_profile` and
  `PlaybackProfile::try_hls_transcode_profile`.
- Updated playback app request identity construction to use Result-returning
  profile creation before staging/session work.
- Added tests for invalid remux profile facts, unsupported HLS codecs, invalid
  runtime-selected hardware in playback plans, and redacted operator errors.

## Verification

- `cargo nextest run -p taru-transcode --no-fail-fast`
- `cargo nextest run -p taru-streaming --no-fail-fast`
- `cargo nextest run -p taru-server playback --no-fail-fast`
- `cargo nextest run -p taru-api admin_playback --no-fail-fast`
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`

## Follow-up

Continue with PTOH-040 to classify persisted/session-level playback failures
across validation, staging, runner, timeout, cancellation, and hardware
fallback boundaries.
