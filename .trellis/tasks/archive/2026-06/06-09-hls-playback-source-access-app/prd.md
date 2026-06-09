# refactor: move hls playback source access into app service

## Goal

Move source `Play` Library Access enforcement for HLS playlist and segment
transport from HTTP-local checks into `PlaybackAppService`, so HLS browser and
Public Client playback use the same app-service access boundary before HLS
playlist startup, artifact serving, or playback-session reuse.

## Requirements

- `PlaybackAppService::hls_playlist_playback` enforces source `Play` Library
  Access before HLS playback policy details, resource admission, FFmpeg input
  staging, transcode session startup, playback session creation, or playlist
  response planning.
- `PlaybackAppService::hls_playlist_for_playback_session` rechecks current
  source `Play` Library Access when a browser-ticket-backed or session-backed
  HLS playlist request is used.
- HLS segment serving enforces source `Play` Library Access in an app-service
  method before planning or serving a segment artifact.
- HLS HTTP playlist/segment handlers resolve authentication/tickets and
  delegate source access to the app service instead of using route-local
  `require_source_access` through `resolve_source_playback_context`.
- A principal with `Browse` but not `Play` access receives the standard
  Library Access forbidden response before HLS playback policy, resource, or
  storage details are exposed.
- Browser playback ticket HLS use rechecks current source `Play` access at
  playlist and segment request time, so access revocation after ticket issue
  remains effective.
- Existing HLS playlist/segment behavior, ticket validation, playlist URL
  rewriting, segment manifest protection, resource admission, seek generation,
  cache-control, playback session header, and renderer transport behavior
  remain unchanged.

## Acceptance Criteria

- [x] App-service test proves a browse-only principal is forbidden for
      `hls_playlist_playback` with `required Library Access level 'play'`
      before HLS/transcode policy details.
- [x] HTTP route test proves browse-only HLS playlist requests return `403`
      with the same public message.
- [x] HTTP route tests prove a previously issued HLS browser ticket is rejected
      after source `Play` access is revoked for both playlist and segment use.
- [x] `resolve_source_playback_context` no longer carries a temporary
      route-local source access flag.
- [x] Existing HLS playlist, segment, ticket, and resource-admission tests still
      pass.
- [x] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- HLS HTTP handlers remain responsible for request parsing, auth/ticket
  resolution, URL decoration, response assembly, and headers.
- HLS app-service flow owns source `Play` Library Access and HLS playback
  policy admission before playlist startup, segment planning, or session use.
- No Direct Play, Remux, subtitle, renderer transport, browser ticket issue,
  DTO, schema, playback planner, or FFmpeg command contract changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Use the existing
`PlaybackAppService::effective_playback_policy_for_playable_source_id` helper
from HLS playlist startup/session code so Library Access is checked before HLS
policy or resource details. Add an app-service HLS segment request method that
accepts the resolved principal and session target facts, rechecks current
source `Play` access, then delegates to existing HLS artifact segment planning.

After HLS source access moves into the app service, simplify
`resolve_source_playback_context` by removing its temporary
`require_route_source_access` flag and its route-local `require_source_access`
calls. Direct, Remux, and HLS playlist/segment source routes then share an
auth/ticket-only resolver. Leave `resolve_subtitle_playback_principal` and
renderer transport ticket source access unchanged for dedicated follow-up
slices.

## Decision (ADR-lite)

**Context**: Playback access checks are being migrated from route-local
enforcement into app-service boundaries one transport slice at a time. HLS is
the remaining `resolve_source_playback_context` user after Direct Play and
Remux migration.

**Decision**: Migrate HLS playlist and segment source `Play` access into
`PlaybackAppService`, and remove the temporary source-access flag from
`resolve_source_playback_context`.

**Consequences**: HLS transport becomes safe for non-HTTP app-service callers
while HTTP continues to own playlist/segment response mechanics. Subtitle and
renderer transport retain route-local access until separate tasks cover their
more specialized resolver flows.

## Out of Scope

- Subtitle route access migration.
- Renderer transport route access migration.
- Direct Play or Remux route behavior.
- Browser playback ticket issue behavior, DTO generation, database schema,
  playback planner, HLS artifact manifest shape, or FFmpeg command changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant app-service files:
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/hls_flow.rs`
  - `crates/nako-server/src/app/playback/hls_artifact.rs`
- Related spec scenarios:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Direct Playback
    Access Boundary".
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` "Remux Playback
    Access Boundary".
  - `.trellis/spec/nako-server/backend/directory-structure.md` "Playback HLS
    Lifecycle Orchestration".
- Required specs are recorded in `implement.jsonl` and `check.jsonl`.
