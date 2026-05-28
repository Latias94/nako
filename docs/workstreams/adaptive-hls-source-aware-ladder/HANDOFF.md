# Adaptive HLS Source-Aware Ladder Runtime Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

This workstream is closed. The previous adaptive runtime lane implemented a
fixed two-rendition fMP4 adaptive slice with explicit HLS artifact manifests and
server artifact serving. This lane completed the first source-aware runtime
deepening:

- adaptive ladder planning should derive from source and client facts;
- adaptive FFmpeg planning should support sources with and without audio.

## Completed State

- `nako-transcode` owns `HlsAdaptiveLadderPlan`, request-variant identity
  material, and audio-presence-aware adaptive FFmpeg maps.
- `nako-playback` carries client max width/height into
  `TranscodeOutputConstraints`.
- `nako-server` derives adaptive plans from selected source facts, binds the
  plan into request identity, stages from that plan, and reconstructs artifacts
  from the persisted request key.

## Validation To Preserve

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Cautions

- Do not use the fixed default ladder as the server runtime source of truth once
  source-aware planning is added.
- Do not allow adaptive no-audio sources to emit `var_stream_map` entries that
  reference audio.
- Keep dynamic ladder decisions deterministic and reconstructable from the
  persisted request/session boundary.

## Follow-Ons

- Adaptive MPEG-TS only if a concrete client/server need appears.
- Alternate audio and subtitle renditions as a separate playlist-manifest lane.
- LL-HLS, CMAF, DRM, and a future rsmpeg/second-engine adapter evaluation.
