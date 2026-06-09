# refactor: move playback ticket source access into app service

## Goal

Move source `Play` Library Access enforcement for browser playback ticket issuing
from the HTTP route layer into `PlaybackAppService`, so non-HTTP callers cannot
mint playback tickets without passing the same source access boundary.

## Requirements

- `PlaybackAppService::validate_browser_playback_ticket_request` enforces source
  `Play` Library Access before mode-specific playback policy validation.
- HTTP browser playback ticket issue handlers parse request inputs and delegate
  validation to the app service without route-local `require_source_access`.
- A principal with `Browse` but not `Play` access receives the standard
  Library Access forbidden response before playback policy details are exposed.
- Existing playback ticket modes, ticket signing, administrator access, unknown
  source behavior, and playback policy rejection behavior remain unchanged.

## Acceptance Criteria

- [ ] App-service test proves a browse-only principal is forbidden by
      `validate_browser_playback_ticket_request` with a message containing
      `required Library Access level 'play'`.
- [ ] HTTP route test proves a browse-only principal cannot issue a browser
      playback ticket and receives `403`.
- [ ] Existing mode-specific playback policy denial tests still pass for
      principals that have source `Play` access.
- [ ] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- Tests added or updated for app-service and HTTP route coverage.
- Route handlers stay thin and app service owns the durable access decision.
- No DTO, schema, direct/remux/HLS byte route, session route, or ticket wire
  contract changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Reuse the existing playback app-service source access helper introduced for
renderer playback where possible. The app service should resolve effective
playback policy for a playable source, enforce source `Play` Library Access,
then continue the existing mode-specific checks for direct stream, remux, HLS,
subtitle, and local-download browser ticket modes.

HTTP route handlers should remove only the duplicated
`require_source_access(... RequiredLibraryAccess::Play)` calls from browser
ticket issuing paths and keep request parsing, principal extraction, and ticket
signing behavior unchanged.

## Decision (ADR-lite)

**Context**: Playback access decisions currently still have route-local pockets.
Renderer play command access has already moved into app-service flow, but
browser playback ticket issuing can still be validated only by HTTP route code.

**Decision**: Migrate browser playback ticket source `Play` access to
`PlaybackAppService::validate_browser_playback_ticket_request` in this task.

**Consequences**: Browser ticket issuing becomes consistent with renderer
playback command boundaries. Direct/remux/HLS byte routes remain route-local
until dedicated follow-up slices migrate their larger transport/session flows.

## Out of Scope

- Direct Play byte GET/HEAD route access migration.
- Remux stream route access migration.
- HLS playlist/segment/session route access migration.
- Renderer playback behavior, transport ticket schema, public DTO generation,
  database schema, or playback policy model changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant app-service file: `crates/nako-server/src/app/playback/mod.rs`.
- Existing renderer access precedent: `.trellis/spec/nako-server/backend/http-api-patterns.md`
  scenario "Renderer Play Command Access Boundary".
- Required specs are recorded in `implement.jsonl` and `check.jsonl`.
