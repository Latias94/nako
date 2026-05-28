# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Handoff

Status: Active
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

## Next Step

Complete TOSHL-050: run closeout checks, update final evidence, commit the
adaptive slice, and decide whether any residual adaptive breadth should split
into a follow-on workstream.

## Guardrails

- Commit only coherent verified slices.
- Keep adaptive HLS behind typed output and manifest boundaries.
- Preserve existing single-variant behavior through tests.
- Keep redaction-safe evidence; no raw host paths or command strings in
  Public/Admin surfaces.
