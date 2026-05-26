# Browser Playback Auth Transport - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

This lane was opened after Media Web Client Foundation closed. Media Web has a
safe watch shell, but no real browser player because bearer-only `<video src>`
cannot attach Authorization headers.

ADR 0036 accepts short-lived browser playback tickets as the first transport.

## Active Task

- Task ID: BPAT-020
- Owner: unassigned
- Files: `crates/nako-api`, `sdk/typescript`,
  `docs/workstreams/browser-playback-auth-transport`
- Validation: `cargo test -p nako-api public_openapi -- --nocapture`;
  `cargo run -q -p nako-api --example emit-typescript-sdk -- --output sdk/typescript/src/index.ts`;
  `git diff --check`
- Status: READY
- Review: Add the Public Client API/OpenAPI/SDK contract for issuing browser
  playback tickets without exposing raw locators, local paths, bearer tokens,
  or permanent privileged URLs.
- Evidence: update `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Short-lived browser playback tickets are the accepted MVP transport.
- ADR 0036 records the durable auth boundary decision.
- Ticket validation must protect direct stream, remux, HLS playlist, and HLS
  segment requests.
- Ticket values are secrets and must be redacted.
- Cookie/session auth and JavaScript HLS/MSE with headers are deferred.

## Next Recommended Action

Run BPAT-020 and add the Public Client contract plus generated SDK shape for
issuing browser playback tickets.
