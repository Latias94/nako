# HLS Seek Restart Lifecycle - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

HSRL-050 is complete and the workstream is closed. HLS request variant identity
has an optional
`HlsPlaybackGeneration`; default `0 ms` generation preserves current request
identity, non-zero starts isolate request identity plus staging layout, and
`HlsAppService` admission now distinguishes same-generation reuse from
superseding-generation restart. Non-zero generations now also reach FFmpeg HLS
command planning as explicit seek, timestamp, and keyframe/independent segment
arguments. The public HLS playlist route accepts `start_position_ms`, and
OpenAPI plus generated TypeScript/Kotlin SDKs expose that query surface.

## Active Task

- None. The workstream is closed.

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
- `GET /sources/{source_id}/stream/hls/playlist.m3u8` accepts
  `start_position_ms` for seek/restart.
- Existing playback-session HLS requests can use the same generation when the
  session has not yet been linked to a transcode session; already-linked
  sessions keep serving their current transcode.
- Client-player seek UI/control wiring is not part of this lane.

## Blockers

- None.

## Next Recommended Action

Split follow-ons for client-player seek controls, seek UX/session heartbeat
coordination, and deeper seek accuracy work such as source keyframe indexing or
pre-roll optimization if those become active goals.
