# Browser Playback Auth Transport - TODO

Status: Active
Last updated: 2026-05-26

Task IDs use the `BPAT` prefix.

## M0 - Transport Decision

- [ ] BPAT-010 [owner=planner] [deps=none] [scope=docs/workstreams/browser-playback-auth-transport,docs/adr]
  Goal: Freeze the browser playback auth transport decision, threat model, and
  API shape before implementation.
  Validation: `python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json`; `git diff --check -- docs/workstreams/browser-playback-auth-transport docs/workstreams/README.md`
  Review: Compare short-lived playback tickets, cookie/session auth, and
  JavaScript HLS/MSE with headers. If the selected transport changes a durable
  auth contract, add or update an ADR.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`, and optional ADR.
  Handoff: Do not implement stream routes until ticket/session/header semantics
  are accepted.

## M1 - Public Contract And SDK

- [ ] BPAT-020 [owner=unassigned] [deps=BPAT-010] [scope=crates/nako-api,sdk/typescript,docs/workstreams/browser-playback-auth-transport]
  Goal: Add the accepted Public Client API/OpenAPI/SDK contract for browser
  playback transport issuance.
  Validation: `cargo test -p nako-api public_openapi -- --nocapture`; `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`; `git diff --check`
  Review: The contract must not expose raw Source Locators, local paths,
  bearer tokens, permanent privileged URLs, Admin API state, or provider
  payloads.
  Evidence: OpenAPI test, SDK diff, route matrix notes.
  Handoff: Split if the contract requires credential/session prerequisites.

## M2 - Server Validation And Stream Use

- [ ] BPAT-030 [owner=unassigned] [deps=BPAT-020] [scope=crates/nako-server,crates/nako-api,crates/nako-core,crates/nako-db]
  Goal: Implement server-side ticket/session/header validation for stream,
  remux, playlist, and segment requests according to the accepted transport.
  Validation: focused Rust tests for ticket issuance, expiry, scope mismatch,
  Library Access denial, Range handling, remux/HLS behavior, and redaction.
  Review: Validation must happen on every protected playback request. Ticket
  values must not be logged or surfaced in client-safe errors.
  Evidence: focused nextest output and security-case notes.
  Handoff: If HLS segment protection needs a separate playback-session model,
  split the smallest backend task before frontend player work.

## M3 - Media Web Real Player

- [ ] BPAT-040 [owner=unassigned] [deps=BPAT-030] [scope=apps/admin-web/src/surfaces/media,sdk/typescript]
  Goal: Replace the Media Web safe watch shell with a real browser player for
  the accepted MVP transport.
  Validation: `cd apps/admin-web && npm run check && npm run test -- mediaSurface.test.tsx mediaDataSource.test.ts`; boundary grep under `apps/admin-web/src/surfaces/media`.
  Review: The UI must not render bearer tokens, raw stream internals, raw
  Source Locators, local paths, or Admin API diagnostics.
  Evidence: player tests, data-source tests, redaction checks.
  Handoff: Keep codec/HDR/subtitle/hardware decode limitations explicit.

## M4 - Playback Progress Writes

- [ ] BPAT-050 [owner=unassigned] [deps=BPAT-040] [scope=apps/admin-web/src/surfaces/media]
  Goal: Wire real player events to User Playback State progress writes with
  sane throttling and end-of-play watched behavior.
  Validation: focused Media Web tests for progress throttling, pause/end
  updates, source-aware state, and no writes when playback is not active.
  Review: Progress writes must use Public Client `/users/me/playback-state`
  routes and must not depend on Admin API state.
  Evidence: route tests and browser smoke.
  Handoff: Split offline sync or multi-device conflict resolution if needed.

## M5 - Closeout

- [ ] BPAT-060 [owner=planner] [deps=BPAT-050] [scope=docs/workstreams/browser-playback-auth-transport]
  Goal: Verify final gates, record browser playback evidence, and close or
  split follow-ons for desktop native playback, subtitles, advanced codecs,
  and credential/session UX.
  Validation: relevant Rust gates, package-local Media Web check/test/build,
  `git diff --check`, browser desktop/mobile smoke, and review-workstream.
  Review: close-workstream before completion claims.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`
  Handoff: Do not expand this lane into Tauri/native playback or account UX.
