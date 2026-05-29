# Public Client Browser Playback Session Identity - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

This lane is closed. PBSI completed the browser playback heartbeat authority
contract across protocol, API/OpenAPI, server behavior, SDKs, and `web/`:

- server/API/SDK contract is implemented with required nullable
  `BrowserPlaybackTicketResponse.playback_session_id`;
- non-subtitle browser playback plans in `web/` retain the returned playback
  session id;
- Public media data sources send heartbeat through
  `POST /playback/sessions/{session_id}/heartbeat`;
- `VideoPlayer` emits active/paused/ended/failed heartbeat bodies with position and
  duration snapshots through the explicit playback session id;
- web tests cover data-source routing and player behavior that rejects URL
  ticket inference;
- media URLs remain token-safe and do not expose `playback_session_id`.

## Active Task

- Task ID: PBSI-050
- Owner: planner
- Status: DONE
- Validation: final protocol/API/server/web/document gates are recorded in
  `EVIDENCE_AND_GATES.md`.

## Blockers

- None known.

## Follow-Ons

- Add unload/pagehide cancellation or final heartbeat behavior for browser
  playback if product telemetry needs explicit session shutdown.
- Add buffering/seek-specific heartbeat states only after the playback product
  model needs them; the current contract intentionally covers stable session
  identity, not rich QoE telemetry.
- Wire native Tauri/desktop playback through its own playback-session control
  path instead of reusing browser-ticket assumptions.

## Next Recommended Action

Keep this lane closed. Start a new focused lane when browser playback telemetry
polish or native/Tauri playback integration becomes the next product priority.
