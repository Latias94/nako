# HLS Seek Restart Lifecycle - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

HSRL-040 is complete. HLS request variant identity has an optional
`HlsPlaybackGeneration`; default `0 ms` generation preserves current request
identity, non-zero starts isolate request identity plus staging layout, and
`HlsAppService` admission now distinguishes same-generation reuse from
superseding-generation restart. Non-zero generations now also reach FFmpeg HLS
command planning as explicit seek, timestamp, and keyframe/independent segment
arguments.

## Active Task

- HSRL-050: public playback integration and closeout.

## Decisions Since Last Update

- Default playback start is `0 ms` and must preserve current request keys.
- Non-zero starts become part of `HlsRequestVariantPlan` identity.
- A same request key active HLS session is still a duplicate conflict.
- A same request key finished HLS output can still be reused when the playlist
  exists.
- A different HLS request key for the same source supersedes active HLS
  sessions by marking them cancellation requested before the new session starts.
- FFmpeg seek planning uses input `-ss` before `-i`, keeps default generation
  argv unchanged, avoids negative timestamps for non-default generations, and
  forces segment-boundary keyframes with independent HLS segments.
- No public HTTP seek API was added in HSRL-040.

## Blockers

- None.

## Next Recommended Action

Implement HSRL-050 by deciding whether this workstream should expose a public
seek request surface now or explicitly close with internal-only playback
generation support and split client-player work into a follow-on.
