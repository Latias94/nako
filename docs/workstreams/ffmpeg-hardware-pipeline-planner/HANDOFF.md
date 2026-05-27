# FFmpeg Hardware Pipeline Planner - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

ADR 0045 records the shipped ownership split: `nako-playback` owns playback
decisions, `nako-transcode` owns hardware inventory, pipeline planning, FFmpeg
command planning, and transcode profile identity, `nako-server` adapts
config/runtime state, and Admin diagnostics expose only redaction-safe evidence.

The HLS runtime now consumes a `TranscodePipelinePlan` through a
pipeline-derived `TranscodeExecutionPolicy`. The old production
selected-accelerator helper chain was removed.

## Active Task

- None. This lane is closed.

## Decisions

- The old selected-accelerator model is not a compatibility contract.
- Hardware capability reporting should be stage-aware before command planning
  grows more FFmpeg branches.
- HLS H.264/AAC remains the executable output during this lane.
- Jellyfin is reference pressure only.

## Blockers

- None.

## Next Action

Recommended follow-ons:

- Extend FFmpeg probing beyond encoder lists into decoder, hwaccel, filter, and
  bitstream-filter parsers so diagnostics can distinguish "encoder exists" from
  "end-to-end hardware pipeline is executable" on real hosts.
- Add HDR tone mapping and subtitle burn-in requirements to
  `TranscodePipelineRequest` before implementing those features in command
  builders.
- Add adaptive HLS ladder planning as a separate workstream; the current lane
  intentionally keeps single-variant HLS as the executable output.
- Connect Admin Web to the new pipeline diagnostics after the frontend lane is
  ready, preserving the admin/media browsing coexistence model from the
  playback UX workstreams.
