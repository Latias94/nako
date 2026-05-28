# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

TOSHL-020 through TOSHL-040 are implemented. `TranscodeProfile` now owns typed
`TranscodeOutputShape`, so remux output container and HLS output requirements
cannot be combined incorrectly. HLS runtime uses `HlsArtifactManifest` /
`TranscodeArtifactSet` boundaries for playlist, init segment, media segment,
content type, cleanup, and reuse. The first adaptive fMP4 HLS ladder slice now
has typed renditions, `hls_adaptive` request identity, FFmpeg master/variant
command planning, server staging, artifact serving, and master playlist rewrite
coverage.

## Follow-Ons

- Make the adaptive ladder source-aware: derive rendition count, resolution,
  bitrate, no-audio stream maps, and no-upscale decisions from
  `MediaProbeResult`, client constraints, and playback policy.
- Add adaptive MPEG-TS only if a client requirement proves it is needed; the
  closed lane intentionally ships adaptive fMP4 first.
- Add subtitle renditions, alternate audio renditions, LL-HLS/CMAF/DRM, and
  richer bitrate policy as separate vertical slices.
- Evaluate rsmpeg or another typed FFmpeg adapter after the CLI model has a
  second real adapter pressure point.

## Guardrails

- Commit only coherent verified slices.
- Keep adaptive HLS behind typed output and manifest boundaries.
- Preserve existing single-variant behavior through tests.
- Keep redaction-safe evidence; no raw host paths or command strings in
  Public/Admin surfaces.
