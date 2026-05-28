# Adaptive HLS Source-Aware Ladder Runtime Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This workstream is open. The previous adaptive runtime lane implemented a fixed
two-rendition fMP4 adaptive slice with explicit HLS artifact manifests and
server artifact serving. This lane now targets the first source-aware runtime
deepening:

- adaptive ladder planning should derive from source and client facts;
- adaptive FFmpeg planning should support sources with and without audio.

## Next Task

Start with AHSL-020.

Recommended order:

1. Inspect `nako-transcode` HLS artifact/profile identity code,
   `nako-playback` transcode requirement facts, and `nako-server` playback
   staging/artifact reconstruction.
2. Add focused failing tests for source-aware ladder output and identity.
3. Implement the typed ladder plan and wire it through server staging.

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
