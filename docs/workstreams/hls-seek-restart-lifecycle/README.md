# HLS Seek Restart Lifecycle

Status: Completed
Last updated: 2026-05-29

This workstream made HLS playback seek-aware without corrupting artifacts,
leaking stale sessions, or turning playback planning into FFmpeg process
control. It starts with request identity and generation modeling, then moves to
runtime restart/cancellation, command planning, and the minimal public playlist
query surface.

## Goal

When a client seeks into an HLS playback, Nako can create or reuse the correct
playback generation, isolate its artifacts, and restart FFmpeg from the
requested media position without serving stale segments from a previous
generation.

## Non-Goals

- No HDR tone mapping.
- No audio downmix/normalization.
- No subtitle burn-in policy.
- No client-player seek controls or heartbeat/seek UX coordination in this
  lane.
- No new media engine or rsmpeg replacement in this lane.

## Architecture References

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/adr/0049-source-aware-transcode-runtime.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
