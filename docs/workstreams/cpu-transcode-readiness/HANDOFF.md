# CPU Transcode Readiness - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

The lane is complete. ADR 0047 records that probe-derived CPU HLS readiness
depends on required software output encoders, and the implementation now matches
that decision.

## Active Task

- None.

## Decisions

- Current HLS CPU readiness requires `libx264` and `aac`.
- Source decoder requirements are a follow-on because the request does not yet
  carry input codec metadata.
- Missing FFmpeg global startup degradation is a follow-on.
- Static `HardwareAccelerationReport::cpu_only()` remains available only as an
  explicit fixture path. Probe-derived reports no longer assume CPU readiness.
- `software_pipeline_unavailable` and `cpu_fallback_unavailable` are distinct
  readiness reasons.

## Blockers

- None.

## Follow-Ons

- Add source-codec-aware decoder requirements when `TranscodePipelineRequest`
  carries input codec facts.
- Probe HLS muxer/protocol capability instead of assuming output support.
- Add startup degradation so admin/browse/direct-play can remain available when
  HLS transcode is unavailable.
