# Public Client Browser Playback Session Identity - Contract Readiness

Status: Active
Last updated: 2026-05-28

## WDRP-065 Decision

Decision: open this Public Client contract lane now.

WMLP can play browser-ticket URLs, but heartbeat remains blocked until the web
client can learn a stable session id through a Public Client JSON contract.

## Contract Questions

PBSI-020 must freeze:

| Question | Initial recommendation |
| --- | --- |
| Identity field | Add `playback_session_id` to browser playback ticket response if the server can guarantee it. |
| Heartbeat route | Reuse `POST /playback/sessions/{session_id}/heartbeat`. |
| Session inspection | Reuse `GET /playback/sessions/{session_id}`. |
| URL safety | Media/subtitle URLs stay ticketed and contain no bearer token. |
| Timing | Session identity must be available before playback heartbeat starts. |
| Error behavior | Missing or expired session id returns public error envelope, not fallback local state. |

## Required Gates

```bash
cargo nextest run -p nako-client-protocol playback --no-fail-fast
cargo nextest run -p nako-api playback --no-fail-fast
cargo nextest run -p nako-server browser_playback --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```
