# Public Client Browser Playback Session Identity - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. PBSI-020 froze the Public Client contract for browser
playback heartbeat authority:

- add required nullable `playback_session_id` to `BrowserPlaybackTicketResponse`;
- return non-null ids for `direct`, `remux`, and `hls`;
- return `null` for `subtitle`;
- bind non-subtitle opaque browser tickets to the same durable playback session;
- use `POST /playback/sessions/{session_id}/heartbeat` with the authenticated
  owner principal, not the media ticket.

## Active Task

- Task ID: PBSI-030
- Owner: Codex
- Status: READY
- Validation: focused playback route tests, OpenAPI/SDK generation checks, and
  `cargo nextest run -p nako-server browser_playback --no-fail-fast`.

## Next Recommended Action

Start PBSI-030. Implement the frozen server/API/SDK contract before wiring web
heartbeat.
