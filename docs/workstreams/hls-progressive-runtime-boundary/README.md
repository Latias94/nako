# HLS Progressive Runtime Boundary

Status: Completed
Last updated: 2026-05-29

This workstream owns the next HLS runtime refactor after fMP4, adaptive
ladders, media renditions, audio sidecars, and seek generation identity have
landed.

Before this lane, the HLS model had strong typed planning and manifest
vocabulary, but the executable runtime still behaved like a whole-output
materialization path: FFmpeg wrote to a temporary directory, the runner promoted
artifacts only after process exit, and the public playlist path was returned
after the transcode had finished. That shape was safe for early VOD slices, but
not the right future boundary for a Jellyfin/Plex-class self-hosted media
server.

This lane made HLS session start, artifact visibility, playlist readiness,
segment serving, and cancellation explicit runtime behavior while preserving
the FFmpeg CLI-first media-engine decision in ADR 0052.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
