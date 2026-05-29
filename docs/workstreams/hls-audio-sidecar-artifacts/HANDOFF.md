# HLS Audio Sidecar Artifacts Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This workstream is closed. Nako now represents generated HLS audio sidecars as
typed media renditions, produces them through FFmpeg command planning, serves
them through manifest-backed HLS segment routes, and publishes `TYPE=AUDIO`
only for generated artifacts.

## Next Task

Open a new workstream for the next boundary. Recommended follow-ons:

1. Stop duplicating the selected audio stream in both main mux and audio
   sidecar for clients that can consume alternate audio groups cleanly.
2. Add language preference/default selection policy instead of using the
   requested or first audio stream only.
3. Add codec-copy or codec-aware sidecar generation for already HLS-compatible
   audio sources.
4. Revisit adaptive video-only variants once alternate audio groups are
   mature.

## Validation To Preserve

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Cautions

- Do not publish `TYPE=AUDIO` for sources with no generated audio sidecars.
- Keep selected-audio main mux behavior intact.
- Treat source stream indexes as global source stream indexes.
