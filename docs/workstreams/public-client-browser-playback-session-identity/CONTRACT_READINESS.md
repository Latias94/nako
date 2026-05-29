# Public Client Browser Playback Session Identity - Contract Readiness

Status: Frozen
Last updated: 2026-05-29

## WDRP-065 Decision

Decision: open this Public Client contract lane now.

WMLP can play browser-ticket URLs, but heartbeat remains blocked until the web
client can learn a stable session id through a Public Client JSON contract.

## Contract Decisions

PBSI-020 freezes:

| Question | Initial recommendation |
| --- | --- |
| Identity field | Add required nullable `playback_session_id` to `BrowserPlaybackTicketResponse`. |
| Mode semantics | `direct`, `remux`, and `hls` return non-null ids; `subtitle` returns `null`. |
| Ticket binding | Non-subtitle browser tickets allocate and bind a durable playback session before JSON response. |
| Heartbeat route | Reuse `POST /playback/sessions/{session_id}/heartbeat`; bearer/session principal, not media ticket, authorizes heartbeat. |
| Session inspection | Reuse `GET /playback/sessions/{session_id}` for the exposed id. |
| URL safety | Media/subtitle URLs stay ticketed and contain no bearer token, raw locator, local path, or renderer/cast transport fields. |
| Timing | Session identity is available in JSON before playback heartbeat starts. |
| Error behavior | Missing/inaccessible session returns `404 not_found`; malformed ids return `400 invalid_input`; terminal heartbeat returns `409 conflict`; expired media tickets return `401 unauthorized`. |

The full frozen contract lives in `CONTRACT.md`.

## Required Gates

```bash
cargo nextest run -p nako-client-protocol playback --no-fail-fast
cargo nextest run -p nako-api playback --no-fail-fast
cargo nextest run -p nako-server browser_playback --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```
