# Public Client Browser Playback Session Identity - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. PBSI-030 implemented the Public Client server/API/SDK
contract for browser playback heartbeat authority:

- `BrowserPlaybackTicketResponse.playback_session_id` is required nullable in
  protocol/OpenAPI/generated SDKs;
- `direct`, `remux`, and `hls` browser ticket responses return non-null session
  ids allocated before the JSON response;
- `subtitle` browser ticket responses return `null`;
- opaque non-subtitle browser tickets are bound to the same durable playback
  session used by direct/remux/HLS media requests;
- heartbeat uses `POST /playback/sessions/{session_id}/heartbeat` with the
  authenticated owner principal, not the media ticket;
- media URLs remain token-safe and do not expose `playback_session_id`.

## Active Task

- Task ID: PBSI-040
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`, `npm --prefix web run check`, and
  `npm --prefix web run build:budget`.

## Next Recommended Action

Start PBSI-040. Wire `web/` playback heartbeat from
`BrowserPlaybackTicketResponse.playback_session_id`; do not infer heartbeat
identity from media URLs or `x-nako-playback-session-id` headers.
