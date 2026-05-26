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

BPAT-040 is complete. Media Web watch pages now request browser playback
tickets through the Public Client data source and render an HTML5 player from
the browser-safe URL envelope without exposing bearer tokens, raw Source
Locators, raw stream paths, or ticket values in visible UI text.

## Active Task

- Task ID: BPAT-050
- Owner: unassigned
- Files: `apps/admin-web/src/surfaces/media`,
  `docs/workstreams/browser-playback-auth-transport`
- Validation: focused Media Web tests for progress throttling, pause/end
  updates, source-aware state, and no writes when playback is not active.
- Status: READY
- Review: Wire real player events to User Playback State progress writes using
  Public Client `/users/me/playback-state` routes. Do not depend on Admin API
  state or write playback progress before playback is active.
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
- BPAT-040 keeps ticket values out of visible UI text. The raw ticket remains
  only inside the media element URL attribute, which is the accepted browser
  transport boundary for this lane.
- Fixture browser smoke uses `https://fixture.nako.test/...` URLs and therefore
  produces an expected media-load console error without a real fixture media
  service.

## Next Recommended Action

Run BPAT-050. Wire the real player to playback progress writes with throttled
`timeupdate` handling, pause/end flushes, source-aware state, and no writes
until playback has actually started.
