# Playback Runtime Boundary Deepening

Status: Completed
Last updated: 2026-05-28

This workstream is a fearless refactor lane for the server playback runtime
after the source-aware transcode runtime slice. It narrows the playback app
boundary before adaptive HLS, fMP4, rsmpeg, or remote transcode worker work adds
more surface area.

## Goals

- Move HLS artifact serving and lifecycle policy out of the large
  `PlaybackAppService` module.
- Keep session orchestration, artifact serving, support evidence, and runtime
  diagnostics as explicit server-owned boundaries.
- Preserve Public/Admin API behavior, redaction guarantees, and existing HLS,
  remux, direct playback, ticket, cancellation, and support evidence semantics.
- Reduce module coupling before adding adaptive HLS, fMP4, rsmpeg, or remote
  worker execution features.

## Non-Goals

- Add adaptive bitrate ladders, fMP4/CMAF, subtitle burn-in, tone mapping, or
  remote workers.
- Replace FFmpeg CLI with rsmpeg.
- Change public playback routes or wire DTOs.
- Change database schema.
- Copy implementation details from reference projects.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
