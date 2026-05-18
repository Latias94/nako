# Admin Playback Session Read Model

Status: Completed
Last updated: 2026-05-18

This workstream tracks M55: add a safe Admin API v1 playback session
list/filter read model for the web console.

## Why This Lane Exists

Taru already exposes Public Client API session detail and cancellation routes
for known playback session IDs. The admin web console needs a broader
Playback & Transcode operational view: active and historical sessions, filtered
by source, kind, state, and pagination.

The existing `TranscodeSessionRecord` includes server-owned implementation
details such as local `output_path`. M55 must add an admin read model without
leaking local paths or weakening the Public Client API boundary.

## Outcome

- `GET /admin/v1/playback/sessions` lists playback sessions with source, kind,
  state, and pagination filters.
- `AdminPlaybackSessionListItem` is a redacted admin list DTO. It excludes
  `output_path`, local staging roots, and raw failure messages.
- Public Client API playback session detail/cancel routes remain unchanged.
- Public OpenAPI, TypeScript SDK, and `taru-client-protocol` boundaries remain
  unchanged.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
