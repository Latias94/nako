# HLS Seek Restart Lifecycle - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

HSRL-030 is complete. HLS request variant identity has an optional
`HlsPlaybackGeneration`; default `0 ms` generation preserves current request
identity, non-zero starts isolate request identity plus staging layout, and
`HlsAppService` admission now distinguishes same-generation reuse from
superseding-generation restart.

## Active Task

- HSRL-040: FFmpeg seek command planning.

## Decisions Since Last Update

- Default playback start is `0 ms` and must preserve current request keys.
- Non-zero starts become part of `HlsRequestVariantPlan` identity.
- A same request key active HLS session is still a duplicate conflict.
- A same request key finished HLS output can still be reused when the playlist
  exists.
- A different HLS request key for the same source supersedes active HLS
  sessions by marking them cancellation requested before the new session starts.
- FFmpeg seek flags are a follow-on task.
- No public HTTP seek API was added in HSRL-030.

## Blockers

- None.

## Next Recommended Action

Implement HSRL-040 by passing `HlsPlaybackGeneration` start positions into HLS
command planning with explicit seek/timestamp behavior tests.
