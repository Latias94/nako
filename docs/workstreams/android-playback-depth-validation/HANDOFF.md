# Android Playback Depth Validation - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

APDV-010 is complete. The lane is open and scoped to Direct Play depth
validation on top of the existing `profile-with-media` smoke fixture.

## Next Task

Run APDV-020:

- inspect the current player UI/evidence available during `profile-with-media`;
- extend smoke evidence so playback advancement is proven beyond the seeded
  server resume point;
- keep the check local, deterministic, and token-safe.

## Constraints

- Do not expand the first slice into HLS/remux/transcode validation.
- Do not introduce golden screenshot comparison here.
- Do not rely on Android device-local resume as the authority.
- Preserve existing smoke regression behavior unless the new evidence is
  deterministic enough for local regression.
