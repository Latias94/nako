# HLS Alternate Audio Renditions Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This workstream is closed. HLS selected audio stream mapping now consumes
`TranscodeTrackSelection.audio_stream`.

## Next Task

Recommended follow-on: open an audio sidecar artifact lane before emitting
`EXT-X-MEDIA:TYPE=AUDIO`.

Recommended order:

1. Extend source facts/planner output to expose publishable audio rendition
   candidates, not only one selected audio stream.
2. Add `HlsAudioRendition` identity and artifact naming to
   `HlsMediaRenditionPlan`.
3. Generate audio-only HLS playlists and segments with FFmpeg.
4. Advertise only generated audio artifacts with `EXT-X-MEDIA:TYPE=AUDIO`.

## Validation To Preserve

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Cautions

- Requested audio stream indexes are source stream indexes, not per-kind audio
  ordinals.
- Keep `0:a:0?` only for the no-explicit-selection fallback.
- No-audio adaptive HLS must keep omitting audio maps and audio encoders.
