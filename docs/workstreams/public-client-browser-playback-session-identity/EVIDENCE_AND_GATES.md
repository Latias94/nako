# Public Client Browser Playback Session Identity - Evidence And Gates

Status: Completed
Last updated: 2026-05-29

## Gate Set

```bash
python -m json.tool docs/workstreams/public-client-browser-playback-session-identity/WORKSTREAM.json
git diff --check -- docs/workstreams/public-client-browser-playback-session-identity
cargo nextest run -p nako-server browser_playback --no-fail-fast
cargo nextest run -p nako-client-protocol playback --no-fail-fast
cargo nextest run -p nako-api playback --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | PBSI-010 | Opened this lane from WDRP-065 after WMLP-040/WMLP-060 proved browser-ticket playback works but lacks web-visible session identity for heartbeat. | Passed. |
| 2026-05-29 | PBSI-020 | Froze `BrowserPlaybackTicketResponse.playback_session_id`, non-subtitle ticket/session binding, heartbeat authority, URL/header safety, and SDK expectations in `CONTRACT.md`; added an HTTP API contract note. Validation: `python -m json.tool`, `git diff --check`, `cargo nextest run -p nako-client-protocol --no-fail-fast`, `cargo nextest run -p nako-api playback --no-fail-fast`, and `cargo fmt --all -- --check`. | Passed. |
| 2026-05-29 | PBSI-030 | Implemented required nullable `BrowserPlaybackTicketResponse.playback_session_id` across protocol/OpenAPI/SDKs, pre-created non-subtitle browser playback sessions, bound opaque source tickets to those sessions, attached direct/remux/HLS media requests to the bound session, kept subtitle tickets sessionless, and enforced heartbeat owner/play access. Validation: `cargo nextest run -p nako-client-protocol public_browser_playback_ticket --no-fail-fast`; `cargo nextest run -p nako-api browser_playback --no-fail-fast`; `cargo nextest run -p nako-api sdk --no-fail-fast`; `cargo nextest run -p nako-client playback_decision_ticket_and_session_cancel_paths_are_stable --no-fail-fast`; `cargo nextest run -p nako-server browser_playback --no-fail-fast`; `cargo nextest run -p nako-server playback_ticket --no-fail-fast`. | Passed. |
| 2026-05-29 | PBSI-040 | Wired `web/` playback plans to carry `BrowserPlaybackTicketResponse.playback_session_id`; added Public Client heartbeat dispatch through `POST /playback/sessions/{session_id}/heartbeat`; updated `VideoPlayer` to emit active/paused/ended/failed heartbeat bodies from the explicit session id instead of media URLs or response headers. Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx`; `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `python -m json.tool docs/workstreams/public-client-browser-playback-session-identity/WORKSTREAM.json`; `git diff --check -- docs/workstreams/public-client-browser-playback-session-identity web`. | Passed. |
| 2026-05-29 | PBSI-050 | Closed the lane after final protocol/API/server/web verification. Target state is met: browser ticket JSON exposes the control-plane session identity, token-safe media URLs remain separate from heartbeat authority, generated SDKs include the field, and `web/` heartbeat uses the explicit session route. Follow-ons are telemetry polish and native/Tauri playback integration, not blockers for this contract lane. Validation: `cargo nextest run -p nako-client-protocol playback --no-fail-fast`; `cargo nextest run -p nako-api playback --no-fail-fast`; `cargo nextest run -p nako-server browser_playback --no-fail-fast`; `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `python -m json.tool docs/workstreams/public-client-browser-playback-session-identity/WORKSTREAM.json`; `git diff --check -- docs/workstreams/public-client-browser-playback-session-identity`. | Passed. |
