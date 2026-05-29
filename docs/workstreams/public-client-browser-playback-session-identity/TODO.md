# Public Client Browser Playback Session Identity - TODO

Status: Completed
Last updated: 2026-05-29

## M0 - Open Lane

- [x] PBSI-010 [owner=planner] [deps=WDRP-065,WMLP-060] [scope=docs/workstreams/public-client-browser-playback-session-identity]
  Goal: Open the browser playback session identity contract lane from WMLP closeout.
  Validation: `python -m json.tool docs/workstreams/public-client-browser-playback-session-identity/WORKSTREAM.json`; `git diff --check -- docs/workstreams/public-client-browser-playback-session-identity`.
  Evidence: Initial design, contract readiness, task ledger, and WDRP-065 update.
  Handoff: DONE. Next task is PBSI-020.

## M1 - Contract Freeze

- [x] PBSI-020 [owner=Codex] [deps=PBSI-010] [scope=crates/nako-client-protocol,crates/nako-api,docs/api/HTTP_API.md,docs/workstreams/public-client-browser-playback-session-identity]
  Goal: Freeze how browser ticket responses expose playback session identity and heartbeat authority.
  Validation: protocol/API tests or snapshots; HTTP API docs updated; `cargo fmt --all -- --check`; `git diff --check`.
  Review: no bearer tokens, source locators, local paths, or transcode internals in public DTOs.
  Evidence: `CONTRACT.md`, `CONTRACT_READINESS.md`, HTTP API contract note, and existing protocol/API playback tests.
  Handoff: DONE. Next task is PBSI-030.

## M2 - Server And SDK Implementation

- [x] PBSI-030 [owner=Codex] [deps=PBSI-020] [scope=crates/nako-server,crates/nako-api,sdk/typescript,crates/nako-client]
  Goal: Implement the accepted DTO/server mapping and regenerate SDKs.
  Validation: focused playback route tests; SDK generation check; `cargo nextest run -p nako-server browser_playback --no-fail-fast`.
  Review: browser media URLs stay token-safe.
  Evidence: DONE. Protocol/API/Rust client/server playback tests pass; TypeScript and Kotlin SDK package entries were regenerated and matched generator output.
  Handoff: DONE. Next task is PBSI-040.

## M3 - Web Heartbeat Integration

- [x] PBSI-040 [owner=Codex] [deps=PBSI-030] [scope=web/src/api/public,web/src/features/media,web/src/test]
  Goal: Wire web playback heartbeat through the stable session identity.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Review: playback-state writes remain separate from session heartbeat.
  Evidence: DONE. Public media data source stores
  `BrowserPlaybackTicketResponse.playback_session_id` on playback plans and
  sends heartbeat through `POST /playback/sessions/{session_id}/heartbeat`;
  `VideoPlayer` emits heartbeat using the explicit session id and never parses
  media URLs or diagnostic headers.
  Handoff: DONE. Next task is PBSI-050.

## M4 - Closeout

- [x] PBSI-050 [owner=planner] [deps=PBSI-040] [scope=docs/workstreams/public-client-browser-playback-session-identity]
  Goal: Close the lane with backend/API/SDK/web evidence and remaining playback follow-ons.
  Validation: final backend/frontend gates, JSON validation, and `git diff --check`.
  Review: no blocking findings.
  Evidence: DONE. Final protocol/API/server/web/document gates are recorded in
  `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json` marks the lane completed; and
  `HANDOFF.md` records follow-ons.
  Handoff: DONE. Lane is closed.
