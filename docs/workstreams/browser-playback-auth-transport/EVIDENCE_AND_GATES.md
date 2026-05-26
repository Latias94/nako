# Browser Playback Auth Transport - Evidence And Gates

Status: Active
Last updated: 2026-05-26

## Smallest Current Repro

```bash
python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json
git diff --check -- docs/workstreams/browser-playback-auth-transport docs/workstreams/README.md
```

## Gate Set

### Planning Gate

```bash
python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json
git diff --check -- docs/workstreams/browser-playback-auth-transport docs/workstreams/README.md
```

### Public Client Contract Gate

```bash
cargo test -p nako-api public_openapi -- --nocapture
cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts
git diff --check
```

### Rust Playback Gate

Use focused `cargo nextest run` commands for the touched packages and tests.
Expected coverage includes:

- ticket or accepted transport issuance;
- expiry;
- scope mismatch;
- Library Access denial;
- Range requests;
- remux;
- HLS playlist and segment protection;
- redaction-safe errors and logs.

### Media Web Package Gate

```bash
cd apps/admin-web && npm run check && npm run test && npm run build
```

### Boundary Leakage Gate

```bash
rg -n "admin/v1|AdminApi|adminApi|source_locator|local path|ffmpeg|Bearer |Authorization" apps/admin-web/src/surfaces/media
```

Any match must be reviewed and either removed or justified as fixture/test-only
safe text.

### Browser Smoke Gate

Use a local server and deterministic test media to verify:

- `/media/watch/:itemId` renders a real player;
- direct/remux/HLS behavior matches the accepted MVP scope;
- Range seeking works where supported;
- progress writes update User Playback State;
- desktop and mobile viewport states are nonblank and do not expose tokens.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | BPAT-010 transport decision | `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, `docs/adr/0036-short-lived-browser-playback-tickets.md`; `python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json`; `git diff --check -- docs/workstreams/browser-playback-auth-transport docs/workstreams/README.md docs/adr` | DONE. Short-lived browser playback tickets are accepted as the first browser playback transport. Cookie/session auth is deferred to credential/session UX, and JavaScript HLS/MSE with headers remains a later playback layer rather than the MVP transport. |

## Notes

Fresh verification is required before marking any task, goal, or lane complete.

## BPAT-010 Decision Matrix

| Option | Result | Rationale |
| --- | --- | --- |
| Short-lived playback tickets | Accepted | Works with native media elements, Range requests, remux URLs, and HLS playlist/segment URLs without putting the long-lived bearer token in browser-visible media URLs. |
| Cookie/session auth | Deferred | Good long-term browser model, but it depends on credential/session semantics, CSRF/same-site policy, reverse proxy behavior, logout, and account switching. |
| JavaScript HLS/MSE with headers | Deferred | Useful for advanced HLS/MSE playback, but it does not solve direct native `<video src>` and adds browser compatibility and buffering complexity. |
| Bearer token in media URL | Rejected | Leaks the long-lived inbound credential through history, logs, referrers, copied URLs, and devtools. |
