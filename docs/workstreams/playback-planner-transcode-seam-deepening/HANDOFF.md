# Playback Planner Transcode Seam Deepening - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

The lane is closed. Remux/HLS `TranscodeProfile` construction moved from
`PlaybackTargetProfile` to transcode-owned playback profile request builders.

## Active Task

- None. PPTS-010 through PPTS-040 are complete.

## Decisions Since Last Update

- Keep playback compatibility decisions in `nako-playback`.
- Keep `TranscodeProfile`, validation, identity, and execution policy assembly
  in `nako-transcode`.
- Keep server playback services as the composition layer that combines planner
  output with runtime acceleration/source facts.
- No wire/API/schema behavior changed in this lane.

## Blockers

- None.

## Next Recommended Action

Open separate lanes for HLS seek/restart, HDR tone mapping, audio
downmix/normalization, subtitle burn-in policy, or runtime resource scheduling
when product pressure justifies them.
