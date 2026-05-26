# Browser Playback Auth Transport - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

This lane was opened after Media Web Client Foundation closed. Media Web has a
safe watch shell, but no real browser player because bearer-only `<video src>`
cannot attach Authorization headers.

ADR 0036 accepts short-lived browser playback tickets as the first transport.

BPAT-020 is complete. The Public Client contract now exposes
`POST /sources/{source_id}/playback/browser-ticket` with protocol-owned
request/response DTOs, Rust/TypeScript client methods, and generated SDK
coverage.

## Active Task

- Task ID: BPAT-030
- Owner: unassigned
- Files: `crates/nako-server`, `crates/nako-api`, `crates/nako-core`,
  `crates/nako-db`, `docs/workstreams/browser-playback-auth-transport`
- Validation: focused Rust tests for ticket issuance, expiry, source/mode
  scope mismatch, Library Access denial, Range handling, remux/HLS behavior,
  and redaction; use `cargo nextest run` where practical.
- Status: READY
- Review: Implement server-side ticket issuance and per-request validation for
  direct stream, remux, HLS playlist, and HLS segment routes. Ticket values
  must not be logged or surfaced in client-safe errors.
- Evidence: update `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Short-lived browser playback tickets are the accepted MVP transport.
- ADR 0036 records the durable auth boundary decision.
- Ticket validation must protect direct stream, remux, HLS playlist, and HLS
  segment requests.
- Ticket values are secrets and must be redacted.
- Cookie/session auth and JavaScript HLS/MSE with headers are deferred.
- BPAT-020 chose a JSON issuance route under the source playback namespace:
  `POST /sources/{source_id}/playback/browser-ticket`.
- The issuance contract returns browser-safe URL descriptors only; actual byte
  serving remains protected by BPAT-030 validation.

## Next Recommended Action

Run BPAT-030. Add the backend ticket model and validation boundary before
Media Web renders a real player.
