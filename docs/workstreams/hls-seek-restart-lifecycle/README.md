# HLS Seek Restart Lifecycle

Status: Active
Last updated: 2026-05-29

This workstream makes HLS playback seek-aware without corrupting artifacts,
leaking stale sessions, or turning playback planning into FFmpeg process
control. It starts with request identity and generation modeling, then moves to
runtime restart/cancellation and command planning.

## Goal

When a client seeks into an HLS playback, Nako should be able to create or reuse
the correct playback generation, isolate its artifacts, and later restart FFmpeg
from the requested media position without serving stale segments from a previous
generation.

## Non-Goals

- No HDR tone mapping.
- No audio downmix/normalization.
- No subtitle burn-in policy.
- No public HTTP seek API until the internal identity/lifecycle model is
  stable.
- No new media engine or rsmpeg replacement in this lane.

## Architecture References

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`

