# HLS Alternate Audio Renditions Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This workstream is open. The next executable task is HAA-020: make selected HLS
audio stream mapping consume `TranscodeTrackSelection.audio_stream`.

## Next Task

Start with HAA-020.

Recommended order:

1. Add a failing `nako-transcode` HLS command builder test for
   `audio_stream: Some(2)` expecting `-map 0:2`.
2. Change single-variant HLS stream map construction to consume track
   selection.
3. Change adaptive HLS stream map construction to repeat the selected audio map
   for each rendition while preserving no-audio behavior.
4. Add or extend a `nako-server` HLS source test so the fake runner records the
   selected audio map for requested audio playback.

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
