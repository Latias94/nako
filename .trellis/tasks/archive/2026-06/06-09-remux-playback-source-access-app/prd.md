# refactor: move remux playback source access into app service

## Goal

Move source `Play` Library Access enforcement for Remux media byte transport
from HTTP-local checks into `PlaybackAppService`, so Remux GET/HEAD and
browser-ticket-backed Remux use the same app-service access boundary before
artifact startup, byte response planning, or playback-session reuse.

## Requirements

- `PlaybackAppService::remux_playback_stream` and
  `PlaybackAppService::remux_playback_preflight` enforce source `Play` Library
  Access before remux policy details, transcode session startup, playback
  session creation, or response planning.
- Remux HTTP GET/HEAD handlers resolve authentication/tickets and delegate
  source access to the app service instead of using route-local
  `require_source_access`.
- A principal with `Browse` but not `Play` access receives the standard
  Library Access forbidden response before Remux playback policy or storage
  details are exposed.
- Browser playback ticket Remux use rechecks current source `Play` access at
  request time, so access revocation after ticket issue remains effective.
- Remux ticket/session use preserves the existing policy behavior for sessions
  that still need to start a remux artifact: source `Play` access is checked
  first, then Remux policy can still deny artifact startup when applicable.
- Existing Remux GET/HEAD/range/session reuse, cache-control, playback session
  header, ticket validation, unauthorized behavior, resource admission, and
  playback policy denial behavior remain unchanged.

## Acceptance Criteria

- [ ] App-service tests prove browse-only principals are forbidden for both
      `remux_playback_stream` and `remux_playback_preflight` with
      `required Library Access level 'play'` before `remux` policy details.
- [ ] HTTP route test proves browse-only Remux stream requests return `403`
      with the same public message.
- [ ] HTTP route test proves a previously issued Remux browser ticket is
      rejected after source `Play` access is revoked.
- [ ] Existing Remux GET/HEAD/range/session reuse/resource tests still pass.
- [ ] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- Remux HTTP handlers remain responsible for request parsing, auth/ticket
  resolution, range parsing, response assembly, and headers.
- Remux app-service flow owns source `Play` Library Access and Remux playback
  policy admission before artifact startup or response planning.
- No Direct Play, HLS, subtitle, renderer, browser ticket issue, DTO, schema,
  playback planner, or FFmpeg command contract changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Add an app-service helper that resolves an effective playback policy only after
checking source `Play` Library Access, without changing the existing
`effective_playback_policy_for_source_id` behavior used by HLS. Use that helper
from ordinary Remux stream/preflight flows. For ticket-backed
`remux_playback_session_stream`, recheck current source `Play` access before
session lookup; if the session does not yet have a linked remux transcode
session, keep the current Remux policy check before starting the artifact.

Update `resolve_source_playback_context` call sites so Direct and Remux pass
`false` for route-local source access while HLS still passes `true`. Leave
subtitle and renderer transport access checks unchanged for dedicated follow-up
slices.

## Decision (ADR-lite)

**Context**: Playback access checks are being migrated from route-local
enforcement into app-service boundaries one transport slice at a time. Remux is
the next byte transport after decision, ticket issuing, renderer command, and
Direct Play migration.

**Decision**: Migrate Remux GET/HEAD source `Play` access into
`PlaybackAppService::remux_playback_stream`,
`PlaybackAppService::remux_playback_preflight`, and ticket-backed
`PlaybackAppService::remux_playback_session_stream`.

**Consequences**: Remux transport becomes safe for non-HTTP callers while HTTP
continues to own byte response mechanics. HLS, subtitle, and renderer
transport retain route-local access until separate tasks cover their broader
session/artifact flows.

## Out of Scope

- HLS playlist/segment/session route access migration.
- Subtitle route access migration.
- Renderer transport route access migration.
- Direct Play route behavior.
- Browser playback ticket issue behavior, DTO generation, database schema,
  playback planner, or FFmpeg command changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant app-service files:
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/remux_flow.rs`
- Related spec scenarios:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Direct Playback
    Access Boundary".
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Browser Playback
    Ticket Access Boundary".
- Required specs are recorded in `implement.jsonl` and `check.jsonl`.
