# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

TOSHL-020 is implemented. `TranscodeProfile` now owns typed
`TranscodeOutputShape`, so remux output container and HLS output requirements
cannot be combined incorrectly. HLS runtime still derives artifact serving from
a primary playlist path.

## Next Step

Implement TOSHL-030: introduce explicit HLS artifact manifest/layout records for
playlist, init segment, media segments, content types, cleanup, and session
reuse.

## Guardrails

- Commit only coherent verified slices.
- Keep adaptive HLS behind typed output and manifest boundaries.
- Preserve existing single-variant behavior through tests.
- Keep redaction-safe evidence; no raw host paths or command strings in
  Public/Admin surfaces.
