# Browser Playback Auth Transport - TODO

Status: Completed
Last updated: 2026-05-26

Task IDs use the `BPAT` prefix.

## M0 - Transport Decision

- [x] BPAT-010 [owner=codex] [deps=none] [scope=docs/workstreams/browser-playback-auth-transport,docs/adr]
  Goal: Freeze the browser playback auth transport decision, threat model, and
  API shape before implementation.
  Validation: `python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json`; `git diff --check -- docs/workstreams/browser-playback-auth-transport docs/workstreams/README.md`
  Review: Compare short-lived playback tickets, cookie/session auth, and
  JavaScript HLS/MSE with headers. If the selected transport changes a durable
  auth contract, add or update an ADR.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`, and optional ADR.
  Handoff: Do not implement stream routes until ticket/session/header semantics
  are accepted.
  Result: DONE 2026-05-26. ADR 0036 accepts short-lived browser playback
  tickets as the first transport. Cookie/session auth and JavaScript HLS/MSE
  with headers remain alternatives or later layers, not the MVP transport.

## M1 - Public Contract And SDK

- [x] BPAT-020 [owner=codex] [deps=BPAT-010] [scope=crates/nako-api,crates/nako-client-protocol,crates/nako-client,sdk/typescript,sdk/kotlin,docs/workstreams/browser-playback-auth-transport]
  Goal: Add the accepted Public Client API/OpenAPI/SDK contract for browser
  playback transport issuance.
  Validation: `cargo test -p nako-client-protocol public -- --nocapture`; `cargo test -p nako-client playback_decision_ticket_and_session_cancel_paths_are_stable -- --nocapture`; `cargo test -p nako-client sdk_inventory -- --nocapture`; `cargo test -p nako-api public_openapi -- --nocapture`; `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`; `git diff --check`
  Review: The contract must not expose raw Source Locators, local paths,
  bearer tokens, permanent privileged URLs, Admin API state, or provider
  payloads.
  Evidence: OpenAPI test, SDK diff, route matrix notes.
  Handoff: Split if the contract requires credential/session prerequisites.
  Result: DONE 2026-05-26. Added protocol-owned browser playback ticket
  request/response DTOs, `POST /sources/{source_id}/playback/browser-ticket`,
  TypeScript and Rust client methods, and refreshed generated SDK entries.
  Server issuance and stream validation remain BPAT-030.

## M2 - Server Validation And Stream Use

- [x] BPAT-030 [owner=codex] [deps=BPAT-020] [scope=crates/nako-server,crates/nako-api,crates/nako-core,crates/nako-db]
  Goal: Implement server-side ticket/session/header validation for stream,
  remux, playlist, and segment requests according to the accepted transport.
  Validation: focused Rust tests for ticket issuance, expiry, scope mismatch,
  Library Access denial, Range handling, remux/HLS behavior, and redaction.
  Review: Validation must happen on every protected playback request. Ticket
  values must not be logged or surfaced in client-safe errors.
  Evidence: focused nextest output and security-case notes.
  Handoff: If HLS segment protection needs a separate playback-session model,
  split the smallest backend task before frontend player work.
  Result: DONE 2026-05-26. Added an opaque in-memory browser playback ticket
  service with hashed token lookup, 6-hour expiry, source/mode/principal
  binding, issuance-time and use-time Library Access checks, and protected
  direct stream, remux, HLS playlist, and HLS segment routes. Ticket query
  auth bypass is limited to media byte routes; token values are not surfaced
  in client-safe errors or Debug output.

## M3 - Media Web Real Player

- [x] BPAT-040 [owner=codex] [deps=BPAT-030] [scope=apps/admin-web/src/surfaces/media,sdk/typescript]
  Goal: Replace the Media Web safe watch shell with a real browser player for
  the accepted MVP transport.
  Validation: `cd apps/admin-web && npm run check && npm run test -- mediaSurface.test.tsx mediaDataSource.test.ts`; boundary grep under `apps/admin-web/src/surfaces/media`.
  Review: The UI must not render bearer tokens, raw stream internals, raw
  Source Locators, local paths, or Admin API diagnostics.
  Evidence: player tests, data-source tests, redaction checks.
  Handoff: Keep codec/HDR/subtitle/hardware decode limitations explicit.
  Result: DONE 2026-05-26. Media Web watch pages now request browser playback
  tickets through the Public Client data source and render an HTML5 player
  from the browser-safe URL envelope. Tests cover live SDK ticket issuance,
  fixture ticketing, source selection, and visible-text redaction for ticket
  values, bearer credentials, and raw stream paths. Browser smoke covered
  desktop and mobile watch states with the expected fixture media load error.

## M4 - Playback Progress Writes

- [x] BPAT-050 [owner=codex] [deps=BPAT-040] [scope=apps/admin-web/src/surfaces/media]
  Goal: Wire real player events to User Playback State progress writes with
  sane throttling and end-of-play watched behavior.
  Validation: focused Media Web tests for progress throttling, pause/end
  updates, source-aware state, and no writes when playback is not active.
  Review: Progress writes must use Public Client `/users/me/playback-state`
  routes and must not depend on Admin API state.
  Evidence: route tests and browser smoke.
  Handoff: Split offline sync or multi-device conflict resolution if needed.
  Result: DONE 2026-05-26. Media Web watch players now write progress through
  the Public Client data source only after playback starts, throttle
  `timeupdate` writes by playback position, flush progress on pause, and mark
  the selected source watched on ended. Tests cover no pre-start writes,
  source-aware progress payloads, pause flushes, and end-of-play watched
  behavior. Fixture browser smoke confirmed visible state updates and no
  transport secret text leakage.

## M5 - Closeout

- [x] BPAT-060 [owner=codex] [deps=BPAT-050] [scope=docs/workstreams/browser-playback-auth-transport]
  Goal: Verify final gates, record browser playback evidence, and close or
  split follow-ons for desktop native playback, subtitles, advanced codecs,
  and credential/session UX.
  Validation: relevant Rust gates, package-local Media Web check/test/build,
  `git diff --check`, browser desktop/mobile smoke, and review-workstream.
  Review: close-workstream before completion claims.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`
  Handoff: Do not expand this lane into Tauri/native playback or account UX.
  Result: DONE 2026-05-26. Final contract, server, frontend, formatting,
  boundary grep, and browser smoke gates passed. The workstream is closed with
  follow-ons split for desktop native playback, subtitles, codec/HDR
  capability mapping, credential/session UX, and account/admin roles.
