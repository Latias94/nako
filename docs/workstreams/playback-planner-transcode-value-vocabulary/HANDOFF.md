# Handoff

Status: Completed
Last updated: 2026-05-29

Current task: None

## Current State

This lane follows `playback-api-transcode-boundary-cleanup`.

PPTV-010 opened the workstream and linked it from playback architecture
indexes.

PPTV-020 removed the direct `nako-playback -> nako-transcode` dependency.
`nako-playback` now owns planner-facing remux/transcode/HLS/track/constraint
values, and `nako-server` maps those values to transcode execution values close
to playback orchestration.

PPTV-030 verified and closed the lane.

## Follow-On

No follow-on is required for this lane. Future playback policy work should keep
planner intent in `nako-playback` and add server-side adapters only when
execution code actually consumes those values.

## Risks

- Request identity strings must remain stable.
- Server runtime policy may still need transcode-owned execution-only values.
  Keep those in server/transcode, not in playback planner records.
