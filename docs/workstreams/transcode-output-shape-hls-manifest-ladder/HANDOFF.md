# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

TOSHL-020 is implemented. `TranscodeProfile` now owns typed
`TranscodeOutputShape`, so remux output container and HLS output requirements
cannot be combined incorrectly. HLS runtime now uses
`HlsArtifactManifest` / `TranscodeArtifactSet` boundaries for playlist, init
segment, media segment, content type, cleanup, and reuse.

## Next Step

Implement TOSHL-040: type the first adaptive HLS ladder slice on top of the new
manifest boundary, including rendition identity, master/variant planning, and
playlist rewrite.

## Guardrails

- Commit only coherent verified slices.
- Keep adaptive HLS behind typed output and manifest boundaries.
- Preserve existing single-variant behavior through tests.
- Keep redaction-safe evidence; no raw host paths or command strings in
  Public/Admin surfaces.
