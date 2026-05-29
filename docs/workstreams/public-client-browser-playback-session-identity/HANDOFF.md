# Public Client Browser Playback Session Identity - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. PBSI-040 completed the browser playback heartbeat wiring
across the Public Client web path:

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
- Status: READY
- Validation: final backend/frontend gates, JSON validation, and
  `git diff --check`.

## Next Recommended Action

Start PBSI-050. Close the lane after final verification, preserve PBSI-040
frontend evidence, and split any remaining browser playback telemetry polish
into a follow-on lane instead of expanding this contract lane.
