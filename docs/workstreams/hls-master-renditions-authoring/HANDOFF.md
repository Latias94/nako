# HLS Master Renditions Authoring Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This workstream is closed. Nako can generate, serve, reuse, and now advertise
selected subtitle WebVTT sidecar artifacts through standard HLS master playlist
media rendition tags.

## Next Task

Recommended follow-on: split alternate audio rendition authoring into a new
workstream once the playback planner and track selection model are ready to
carry alternate audio groups.

Likely order:

1. Extend `HlsMediaRenditionPlan` with audio rendition facts.
2. Teach playback selection to expose alternate audio tracks without forcing a
   selected-only sidecar model.
3. Reuse the HLS master playlist authoring boundary for `TYPE=AUDIO` groups.

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
