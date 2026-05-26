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

BPAT-050 is complete. Media Web watch players now write User Playback State
progress after playback starts, throttle normal `timeupdate` writes, flush on
pause, and mark the selected source watched on ended.

## Active Task

- Task ID: BPAT-060
- Owner: planner
- Files: `docs/workstreams/browser-playback-auth-transport`,
  `apps/admin-web/src/surfaces/media`, `crates/nako-server`
- Validation: relevant Rust gates, package-local Media Web check/test/build,
  `git diff --check`, browser desktop/mobile smoke, and review-workstream.
- Status: READY
- Review: close or split follow-ons for desktop native playback, subtitles,
  advanced codec/HDR capability mapping, credential/session UX, and broader
  account/admin role work.
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
- BPAT-050 writes progress through Public Client playback-state routes only.
  It does not use Admin API state or expose transport credentials in the UI.
- Progress throttling is position-based, not wall-clock-based: normal
  `timeupdate` writes require at least 30 seconds of playback-position delta;
  pause writes force a flush.

## Next Recommended Action

Run BPAT-060. Verify the lane end to end, record final evidence, then close or
split follow-ons rather than expanding this lane into desktop native playback,
subtitles, advanced codecs, sessions, or account UX.
