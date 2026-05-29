# Public Client Browser Playback Session Identity - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Gate Set

```bash
python -m json.tool docs/workstreams/public-client-browser-playback-session-identity/WORKSTREAM.json
git diff --check -- docs/workstreams/public-client-browser-playback-session-identity
cargo nextest run -p nako-server browser_playback --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/video-player.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | PBSI-010 | Opened this lane from WDRP-065 after WMLP-040/WMLP-060 proved browser-ticket playback works but lacks web-visible session identity for heartbeat. | Passed. |
| 2026-05-29 | PBSI-020 | Froze `BrowserPlaybackTicketResponse.playback_session_id`, non-subtitle ticket/session binding, heartbeat authority, URL/header safety, and SDK expectations in `CONTRACT.md`; added an HTTP API contract note. Validation: `python -m json.tool`, `git diff --check`, `cargo nextest run -p nako-client-protocol --no-fail-fast`, `cargo nextest run -p nako-api playback --no-fail-fast`, and `cargo fmt --all -- --check`. | Passed. |
