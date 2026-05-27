# Playback Transcode Startup Degradation - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

The lane is complete. ADR 0048 defines HLS transcode readiness as runtime
capability instead of a startup invariant, and the implementation now matches
that decision.

## Active Task

- None.

## Decisions

- Keep HLS readiness even when no executable `TranscodePipelinePlan` exists.
- HLS request execution still plans at request time and fails before FFmpeg
  spawn if unavailable.
- Admin diagnostics should use readiness, not require a startup plan.
- `selected_hls_slots` is zero when no executable startup HLS plan exists.

## Blockers

- None.

## Follow-Ons

- Add source-codec-aware decoder requirements.
- Add HLS muxer/protocol probing.
- Consider frontend admin rendering for unavailable HLS transcode readiness.
