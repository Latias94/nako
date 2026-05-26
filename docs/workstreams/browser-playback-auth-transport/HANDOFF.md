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

BPAT-030 is complete. The server now issues opaque in-memory playback tickets
and validates ticketed direct stream, remux, HLS playlist, and HLS segment
requests before serving bytes.

## Active Task

- Task ID: BPAT-040
- Owner: unassigned
- Files: `apps/admin-web/src/surfaces/media`, `sdk/typescript`,
  `docs/workstreams/browser-playback-auth-transport`
- Validation: `cd apps/admin-web && npm run check && npm run test -- mediaSurface.test.tsx mediaDataSource.test.ts`; boundary grep under `apps/admin-web/src/surfaces/media`; browser smoke when a dev server is available.
- Status: READY
- Review: Replace the safe watch shell with a real browser player that calls
  the Public Client ticket route and uses only browser-safe playback URLs.
  The UI must not render bearer tokens, raw stream internals, raw Source
  Locators, local paths, or Admin API diagnostics.
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
- BPAT-030 uses an in-memory opaque ticket service rather than reversible
  signed claims. The server stores only hashed token lookup keys and scoped
  ticket records.
- Ticket validation rechecks current Library Access at use, not only at
  issuance.
- Auth middleware bypass is intentionally narrow: GET/HEAD media byte routes
  with a `ticket` query are allowed through to route-level ticket validation;
  Admin/Public JSON routes stay bearer-protected.

## Next Recommended Action

Run BPAT-040. Wire Media Web's watch surface to request browser playback
tickets and render a real player without exposing bearer tokens or raw media
internals.
