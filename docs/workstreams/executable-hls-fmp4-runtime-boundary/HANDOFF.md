# Executable HLS fMP4 Runtime Boundary - Handoff

Status: Completed
Last updated: 2026-05-28

## Current State

The first executable fMP4 single-variant HLS slice is complete. MPEG-TS remains
the default, while fMP4 now has explicit runtime identity, `.m4s` staging,
FFmpeg muxer planning, playlist init-segment rewrite support, and artifact
content-type handling.

## Next Step

Open a follow-on adaptive ladder lane before adding multi-variant master
playlists, bitrate ladder planning, or per-variant segment directories. That
lane should start from `HlsVariantPolicy::Adaptive`, not from the single-variant
runtime path.

Useful starting points:

- `crates/nako-transcode/src/profile.rs`
- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-server/src/app/playback/staging_policy.rs`
- `crates/nako-server/src/app/playback/playlist.rs`

## Guardrails

- Keep adaptive ladder runtime out of this completed slice.
- Preserve MPEG-TS as the default HLS behavior.
- Keep Public/Admin evidence redaction-safe.
- Do not copy Jellyfin or FFmpeg source/tests; use them only as behavior
  references.
