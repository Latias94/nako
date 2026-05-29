# HLS Audio Sidecar Artifacts Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This workstream is open. The next executable task is HAS-020: add typed audio
rendition artifact identity and manifest membership.

## Next Task

Start with HAS-020.

Recommended order:

1. Add `HlsAudioRendition` naming and identity to `nako-transcode`.
2. Add audio playlist and `.aac` segment artifact membership.
3. Add FFmpeg audio sidecar output args.
4. Add server planning and master playlist publication.

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
