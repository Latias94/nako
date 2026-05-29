# HLS Progressive Runtime Boundary

Status: Active
Last updated: 2026-05-29

This workstream owns the next HLS runtime refactor after fMP4, adaptive
ladders, media renditions, audio sidecars, and seek generation identity have
landed.

The current HLS model has strong typed planning and manifest vocabulary, but
the executable runtime still behaves like a whole-output materialization path:
FFmpeg writes to a temporary directory, the runner promotes artifacts only after
process exit, and the public playlist path is returned after the transcode has
finished. That shape is safe for early VOD slices, but it is not the right
future boundary for a Jellyfin/Plex-class self-hosted media server.

This lane makes HLS session start, artifact visibility, playlist readiness,
segment serving, and cancellation explicit runtime behavior while preserving
the FFmpeg CLI-first media-engine decision in ADR 0052.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
