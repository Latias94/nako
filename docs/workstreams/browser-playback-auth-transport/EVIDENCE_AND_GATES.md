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
cargo test -p nako-client-protocol public -- --nocapture
cargo test -p nako-client playback_decision_ticket_and_session_cancel_paths_are_stable -- --nocapture
cargo test -p nako-client sdk_inventory -- --nocapture
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
| 2026-05-26 | BPAT-020 public contract and SDK | `crates/nako-client-protocol/src/catalog.rs`, `crates/nako-client-protocol/src/lib.rs`, `crates/nako-client/src/lib.rs`, `crates/nako-api/src/openapi.rs`, `crates/nako-api/src/sdk.rs`, `sdk/typescript/src/index.ts`, `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`; `cargo test -p nako-client-protocol public -- --nocapture`; `cargo test -p nako-client playback_decision_ticket_and_session_cancel_paths_are_stable -- --nocapture`; `cargo test -p nako-client sdk_inventory -- --nocapture`; `cargo test -p nako-api public_openapi -- --nocapture`; `cargo test -p nako-api typescript_sdk -- --nocapture`; `cargo test -p nako-api kotlin_sdk -- --nocapture`; `cargo test -p nako-api sdk`; `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`; `cargo run -q -p nako-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`; `git diff --check` | PASS. Public contract now issues browser playback ticket envelopes without raw locators, local paths, bearer-token media URLs, permanent privileged URLs, Admin API state, or provider payloads. |
| 2026-05-26 | BPAT-030 server validation and stream use | `crates/nako-server/src/app/playback_ticket.rs`, `crates/nako-server/src/app.rs`, `crates/nako-server/src/app/composition.rs`, `crates/nako-server/src/http/auth.rs`, `crates/nako-server/src/http/playback.rs`, `crates/nako-server/src/http/tests/playback.rs`; `cargo nextest run -p nako-server browser_playback_ticket playback_ticket_bypass_is_limited_to_media_byte_routes source_ticket_is_opaque_and_validates_scope_and_expiry`; `cargo test -p nako-server playback -- --nocapture`; `cargo test -p nako-server bearer_auth_protects_non_health_routes_and_keeps_health_public -- --nocapture`; `cargo fmt --all --check`; `python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json`; `git diff --check` | PASS. Server issues opaque short-lived browser playback tickets, validates direct/remux/HLS requests before bytes are served, limits auth bypass to ticketed media byte routes, rechecks Library Access at use, preserves Range handling, protects HLS segment URLs, rejects scope mismatch and expired tickets, and redacts raw ticket values from client-safe errors and Debug output. |
| 2026-05-26 | BPAT-040 Media Web real player | `apps/admin-web/src/surfaces/media/MediaPages.tsx`, `apps/admin-web/src/surfaces/media/mediaDataSource.ts`, `apps/admin-web/src/surfaces/media/mediaDataSource.test.ts`, `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`, `apps/admin-web/src/styles.css`; `cd apps/admin-web && npm run test -- mediaDataSource.test.ts mediaSurface.test.tsx`; `cd apps/admin-web && npm run check`; `cd apps/admin-web && npm run build`; `rg -n "admin/v1|AdminApi|adminApi|source_locator|local path|ffmpeg|Bearer |Authorization" apps/admin-web/src/surfaces/media`; Playwright smoke at `http://127.0.0.1:4173/media` desktop and 390x844 mobile | PASS. Watch pages request browser playback tickets through the Public Client data source and render an HTML5 player from the browser-safe URL envelope. Visible page text did not contain `nako_bpt_fixture`, `/sources/`, `Bearer`, or `Authorization`; the only boundary grep hits were test-only SDK header assertions. Vite build passed with existing chunk-size/plugin-timing warnings. Fixture smoke produced the expected media-load console error because `fixture.nako.test` is not a real media server. |

## Notes

Fresh verification is required before marking any task, goal, or lane complete.

## BPAT-020 Route Matrix

| Route | Method | Request | Response | Notes |
| --- | --- | --- | --- | --- |
| `/sources/{source_id}/playback/browser-ticket` | `POST` | `BrowserPlaybackTicketRequest` with `mode` (`direct`, `remux`, `hls`) and optional browser capability fields | `BrowserPlaybackTicketResponse` with `source_id`, optional `item_id`, selected `mode`, `expires_at`, and one or more browser-safe URL descriptors | JSON-only issuance contract. It does not serve bytes and does not expose raw Source Locators, local paths, bearer tokens, Admin API state, or provider payloads. Direct/remux/HLS request validation is BPAT-030. |

## BPAT-010 Decision Matrix

| Option | Result | Rationale |
| --- | --- | --- |
| Short-lived playback tickets | Accepted | Works with native media elements, Range requests, remux URLs, and HLS playlist/segment URLs without putting the long-lived bearer token in browser-visible media URLs. |
| Cookie/session auth | Deferred | Good long-term browser model, but it depends on credential/session semantics, CSRF/same-site policy, reverse proxy behavior, logout, and account switching. |
| JavaScript HLS/MSE with headers | Deferred | Useful for advanced HLS/MSE playback, but it does not solve direct native `<video src>` and adds browser compatibility and buffering complexity. |
| Bearer token in media URL | Rejected | Leaks the long-lived inbound credential through history, logs, referrers, copied URLs, and devtools. |
