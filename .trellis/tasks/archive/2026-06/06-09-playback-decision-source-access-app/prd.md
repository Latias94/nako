# refactor: move playback decision source access into app service

## Goal

Move source `Play` Library Access enforcement for public playback decision
lookup from the HTTP route layer into `PlaybackAppService`, so all callers of
the decision planner receive the same access boundary and error ordering.

## Requirements

- `PlaybackAppService::get_source_playback_decision` enforces source `Play`
  Library Access before playback planning or policy detail exposure.
- The `/sources/{source_id}/playback/decision` HTTP handler parses query data
  and delegates to the app service without route-local `require_source_access`.
- A principal with `Browse` but not `Play` access receives the standard
  Library Access forbidden response before playback policy details are exposed.
- Existing administrator access, unknown source behavior, client capability
  parsing, planning output, and playback policy denial behavior remain
  unchanged.

## Acceptance Criteria

- [ ] App-service test proves a browse-only principal calling
      `get_source_playback_decision` receives `NakoError::Forbidden` with
      `required Library Access level 'play'`.
- [ ] HTTP route test proves a browse-only principal calling
      `/sources/{source_id}/playback/decision` receives `403` with the same
      public message.
- [ ] Existing playback policy decision tests still pass for principals with
      source `Play` access.
- [ ] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- Route handlers stay thin and app service owns the durable access decision.
- Tests cover app-service and HTTP route behavior.
- No DTO, schema, playback planner, direct/remux/HLS transport, session, or
  ticket contract changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Reuse the existing `PlaybackAppService::effective_playback_policy_for_playable_source`
helper, which resolves effective playback policy and rejects principals without
source `Play` Library Access before policy checks are surfaced. Replace the
plain policy lookup inside `get_source_playback_decision` with this helper and
remove only the duplicated `require_source_access(... Play)` call from the HTTP
decision handler.

## Decision (ADR-lite)

**Context**: Playback route-local source access checks are being moved into
app-service boundaries one slice at a time. Browser playback tickets and
renderer play commands already enforce source `Play` access in app-service
paths.

**Decision**: Migrate public playback decision source `Play` access to
`PlaybackAppService::get_source_playback_decision` in this task.

**Consequences**: Playback decision planning becomes safe for non-HTTP callers.
Direct/remux/HLS byte and session routes remain route-local until dedicated
follow-up slices migrate those larger transport flows.

## Out of Scope

- Direct Play byte GET/HEAD route access migration.
- Remux stream route access migration.
- HLS playlist/segment/session route access migration.
- Browser playback ticket, renderer playback, DTO generation, database schema,
  or playback planner behavior changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant app-service file: `crates/nako-server/src/app/playback/mod.rs`.
- Related spec scenarios:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Renderer Play
    Command Access Boundary".
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Browser Playback
    Ticket Access Boundary".
- Required specs are recorded in `implement.jsonl` and `check.jsonl`.
