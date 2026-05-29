# HLS Master Renditions Authoring Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This workstream is open. Nako can generate and serve selected subtitle WebVTT
sidecar artifacts, but public HLS entry playlists do not yet advertise those
sidecars through standard HLS media rendition tags.

## Next Task

Start with HMA-020.

Recommended order:

1. Inspect `HlsArtifactManifest`, `HlsMediaRenditionPlan`,
   `hls_artifact_manifest_for_session`, and `rewrite_hls_playlist`.
2. Decide whether the authoring boundary belongs in `nako-server` only or
   whether `nako-transcode` should expose additional typed playlist facts.
3. Add the smallest authoring helper that can generate subtitle media tags for
   single-variant and adaptive HLS without changing raw artifact serving.

## Validation To Preserve

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Cautions

- A media playlist cannot itself attach subtitle renditions in the same way a
  master playlist can; single-variant HLS may need a generated master entry
  point that references the existing media playlist.
- Keep session reuse deterministic: authored public playlists must be
  reconstructible from persisted request identity and output paths.
- Preserve adaptive no-audio stream-map behavior.
