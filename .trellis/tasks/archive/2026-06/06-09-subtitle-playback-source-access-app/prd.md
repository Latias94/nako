# refactor: move subtitle playback source access into app service

## Goal

Move source `Play` Library Access enforcement for sidecar subtitle playback
from HTTP-local subtitle ticket/principal resolution into
`PlaybackAppService::subtitle_playback`, so ordinary and browser-ticket-backed
subtitle requests use the same app-service access boundary before sidecar
probe, file-name, storage, or policy details are exposed.

## Requirements

- `PlaybackAppService::subtitle_playback` enforces source `Play` Library Access
  before subtitle playback policy details, probe lookup, sidecar file-name
  resolution, storage backend lookup, sidecar stat/read, or response planning.
- Subtitle HTTP handlers resolve authentication or subtitle browser tickets and
  delegate source access to `PlaybackAppService` instead of calling
  route-local `require_source_access`.
- Subtitle browser ticket use rechecks current source `Play` access at use time
  through the app service, so access revocation after ticket issue remains
  effective.
- A principal with `Browse` but not `Play` access receives the standard Library
  Access forbidden response before subtitle stream, sidecar, storage, remote
  playback, or media-playback policy details are exposed.
- Existing subtitle ticket validation, subtitle stream scoping, content type,
  sidecar path redaction, byte limit, unauthenticated bearer bypass behavior,
  and subtitle response body behavior remain unchanged.

## Acceptance Criteria

- [x] App-service test proves a browse-only principal is forbidden for
      `subtitle_playback` with `required Library Access level 'play'` before
      media-playback or subtitle sidecar details.
- [x] HTTP route test proves browse-only subtitle requests return `403` with
      the same public Library Access message.
- [x] HTTP route test proves a previously issued subtitle browser ticket is
      rejected after source `Play` access is revoked.
- [x] `resolve_subtitle_playback_principal` no longer calls
      `require_source_access` and only resolves auth/ticket principal identity.
- [x] Existing sidecar subtitle route and browser-ticket tests still pass.
- [x] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- Subtitle HTTP handlers remain responsible for request parsing, auth/ticket
  resolution, response assembly, and headers.
- `PlaybackAppService::subtitle_playback` owns source `Play` Library Access,
  subtitle playback policy admission, probe lookup, sidecar selection, storage
  reads, and output shaping.
- No Direct Play, Remux, HLS, renderer transport, browser ticket issue, DTO,
  schema, playback planner, or HLS rendition contract changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Reuse the existing `ensure_source_play_access` helper before
`ensure_subtitle_playback_allowed_for_source` inside
`PlaybackAppService::subtitle_playback`. This preserves the current subtitle
policy checks for `RemotePlayback` and `MediaPlayback`, while guaranteeing
Library Access denial happens first.

After subtitle source access moves into the app service, simplify
`resolve_subtitle_playback_principal` so it validates subtitle browser tickets
or extracts the authenticated principal only. Leave
`resolve_renderer_transport_principal` and
`resolve_renderer_transport_principal_for_session` unchanged for a dedicated
follow-up slice.

## Decision (ADR-lite)

**Context**: Direct Play, Remux, and HLS source access have been migrated from
route-local enforcement into `PlaybackAppService`. Subtitle playback is the
next remaining route-local `Play` access path in `http/playback.rs`.

**Decision**: Migrate sidecar subtitle playback source `Play` access into
`PlaybackAppService::subtitle_playback`, and make
`resolve_subtitle_playback_principal` an auth/ticket-only resolver.

**Consequences**: Subtitle playback becomes safe for non-HTTP app-service
callers while HTTP continues to own ticket parsing and subtitle response
mechanics. Renderer transport keeps route-local access until a separate task
covers its dedicated resolver flow.

## Out of Scope

- Renderer transport route access migration.
- Direct Play, Remux, or HLS route behavior.
- HLS sidecar subtitle rendition artifact behavior.
- Browser playback ticket issue behavior, DTO generation, database schema,
  playback planner, or storage sidecar contract changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant app-service file: `crates/nako-server/src/app/playback/mod.rs`.
- Existing route tests:
  - `subtitle_route_serves_sidecar_text_without_exposing_locator`
  - `subtitle_route_requires_play_library_access`
  - `browser_playback_ticket_streams_sidecar_subtitle_without_bearer`
- Related spec scenarios:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md` playback access
    boundary precedents for Direct, Remux, HLS, and browser tickets.
  - `.trellis/spec/nako-server/backend/error-handling.md` forbidden response
    mapping and public message behavior.

## Verification

- `cargo fmt --all`
- `cargo nextest run -p nako-server subtitle_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`
- `cargo nextest run -p nako-server subtitle_route_requires_play_library_access --no-fail-fast`
- `cargo nextest run -p nako-server subtitle_browser_playback_ticket_rejects_revocation_at_use --no-fail-fast`
- `cargo nextest run -p nako-server subtitle --no-fail-fast`
- `cargo nextest run -p nako-server browser_playback_ticket_streams_sidecar_subtitle_without_bearer --no-fail-fast`
- `cargo check -p nako-server --tests`
- `git diff --check`
- `python .\.trellis\scripts\task.py validate 06-09-subtitle-playback-source-access-app`
