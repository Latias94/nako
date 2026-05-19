# Android Playback Depth Validation - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

APDV-010, APDV-020, APDV-030, and APDV-040 are complete. The lane is closed.
The existing `profile-with-media` smoke fixture now proves Direct Play
advancement and server **User Playback State** readback after player exit.

## Closeout Evidence

- `apps/android/build/smoke-apdv/20260519-174200-profile-with-media-emulator-5554/player.criteria.txt`
- `apps/android/build/smoke-apdv/20260519-174200-profile-with-media-emulator-5554/profile-with-media-server-readback.txt`
- `git diff --check`

## Follow-Ons

Split these into new lanes if needed:

- HLS/remux/session cancellation validation.
- A longer fixture for watched-threshold policy that is not dominated by a
  two-second smoke clip.
- Playback quality, subtitle, audio, and PiP depth checks.

## Constraints

- Do not expand the first slice into HLS/remux/transcode validation.
- Do not introduce golden screenshot comparison here.
- Do not rely on Android device-local resume as the authority.
- Preserve existing smoke regression behavior unless the new evidence is
  deterministic enough for local regression.
