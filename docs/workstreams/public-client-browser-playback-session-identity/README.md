# Public Client Browser Playback Session Identity

Status: Completed
Last updated: 2026-05-29

This lane fixed the WMLP playback follow-on: browser playback tickets can start
safe media URLs, and the web client can now heartbeat playback through an
explicit Public Client playback session identity exposed in the browser ticket
JSON response.

## Authoritative Docs

- `DESIGN.md` - problem, scope, non-goals, and architecture direction.
- `CONTRACT.md` - frozen browser playback session identity contract.
- `CONTRACT_READINESS.md` - contract choices and readiness checks.
- `TODO.md` - executable task ledger.
- `EVIDENCE_AND_GATES.md` - validation commands and evidence log.
- `HANDOFF.md` - current state and next action.

## Closeout State

PBSI is closed. The shipped contract adds required nullable
`BrowserPlaybackTicketResponse.playback_session_id`; direct, remux, and HLS
browser tickets return non-null playback session ids; subtitle tickets return
`null`; ticketed media requests attach to the same server-side playback
session; and `web/` heartbeat uses
`POST /playback/sessions/{session_id}/heartbeat` through that explicit session
id instead of parsing media URLs, playlists, or diagnostic headers.
