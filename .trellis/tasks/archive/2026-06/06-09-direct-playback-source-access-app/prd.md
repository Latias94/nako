# refactor: move direct playback source access into app service

## Goal

Move source `Play` Library Access enforcement for Direct Play media byte
transport from HTTP-local checks into `PlaybackAppService`, so Direct Play
GET/HEAD and browser-ticket-backed Direct Play use the same app-service access
boundary before byte plans or playback sessions are created.

## Requirements

- `PlaybackAppService::direct_playback_stream` and
  `PlaybackAppService::direct_playback_preflight` enforce source `Play` Library
  Access before direct byte planning, playback session creation, or response
  planning.
- Direct Play HTTP GET/HEAD handlers resolve authentication/tickets and delegate
  source access to the app service instead of using route-local
  `require_source_access`.
- A principal with `Browse` but not `Play` access receives the standard
  Library Access forbidden response before playback policy or storage details
  are exposed.
- Browser playback ticket Direct Play use still rechecks current source `Play`
  access at request time, so access revocation after ticket issue remains
  effective.
- Existing direct stream status, HEAD/no-body behavior, range handling,
  cache-control, playback session header, ticket validation, unauthorized
  behavior, and playback policy denial behavior remain unchanged.

## Acceptance Criteria

- [ ] App-service tests prove browse-only principals are forbidden for both
      `direct_playback_stream` and `direct_playback_preflight` with
      `required Library Access level 'play'`.
- [ ] HTTP route test proves browse-only direct stream requests return `403`
      with the same public message.
- [ ] HTTP route test proves a previously issued Direct Play browser ticket is
      rejected after source `Play` access is revoked.
- [ ] Existing Direct Play GET/HEAD/range/session tests still pass.
- [ ] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- Direct Play HTTP handlers remain responsible for request parsing, auth/ticket
  resolution, range parsing, response assembly, and headers.
- Direct Play app-service flow owns source `Play` Library Access and playback
  policy admission before byte plans or sessions.
- No remux, HLS, renderer, browser ticket issue, DTO, schema, or playback
  planner contract changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Use the existing `PlaybackAppService::ensure_direct_playback_allowed` path and
update it to share the same Library Access-first helper used by renderer,
browser ticket, and playback decision flows. Remove route-local
`require_source_access(... Play)` only from Direct Play principal/ticket
resolution for `/sources/{source_id}/stream`; leave remux, HLS, subtitle, and
renderer transport access checks for dedicated follow-up slices.

## Decision (ADR-lite)

**Context**: Playback access checks are being migrated from route-local
enforcement into app-service boundaries one transport slice at a time. Direct
Play is the smallest byte transport after decision/ticket migration.

**Decision**: Migrate Direct Play GET/HEAD source `Play` access into
`PlaybackAppService::direct_playback_stream` and
`PlaybackAppService::direct_playback_preflight`.

**Consequences**: Direct Play transport becomes safe for non-HTTP callers while
HTTP continues to own byte response mechanics. Remux and HLS retain route-local
access until separate tasks cover their broader session/artifact flows.

## Out of Scope

- Remux stream route access migration.
- HLS playlist/segment/session route access migration.
- Subtitle route access migration.
- Renderer transport route access migration.
- Browser playback ticket issue behavior, DTO generation, database schema, or
  playback planner changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant app-service file: `crates/nako-server/src/app/playback/mod.rs`.
- Related spec scenarios:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Playback Decision
    Access Boundary".
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Browser Playback
    Ticket Access Boundary".
- Required specs are recorded in `implement.jsonl` and `check.jsonl`.
