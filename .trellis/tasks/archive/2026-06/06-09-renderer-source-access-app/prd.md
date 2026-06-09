# Refactor: Move Renderer Source Access Into App Service

## Goal

Move Public Client renderer play-command source `Play` access enforcement out
of the Axum route and into the app-service boundary. This keeps renderer HTTP
handlers thin, aligns renderer command creation with recent access-boundary
cleanups, and prevents non-HTTP callers of the renderer/casting service from
creating playback runtime records for inaccessible sources.

## What I Already Know

- `crates/nako-server/src/http/renderer.rs` currently parses
  `RendererPlayCommandRequest.source_id`, calls
  `require_source_access(... RequiredLibraryAccess::Play)`, then delegates to
  `app.casting().play_on_renderer(...)`.
- `crates/nako-server/src/app/casting.rs` already receives the full
  `AuthenticatedPrincipal` and calls
  `PlaybackAppService::start_renderer_playback_session(...)`.
- `crates/nako-server/src/app/playback/renderer_flow.rs` already loads the
  source and derives effective playback policy from the principal and source
  before creating playback sessions or transcode artifacts.
- Existing renderer route tests cover successful play command creation and
  playback-policy denial, but this slice needs explicit route/app regression
  coverage for source access denial at the app-service boundary.

## Requirements

- Remove renderer play-command route-local source `Play` access enforcement from
  `http::renderer`.
- Enforce source `Play` library access inside the app-service path used by
  renderer play command creation before playback sessions, renderer commands,
  transcode sessions, or HLS artifacts are created.
- Preserve existing renderer ownership, renderer capability, playback-policy,
  transport-ticket, Direct/Remux/HLS, and public error-envelope behavior.
- Keep `http::access::require_source_access` available for route slices that
  have not yet moved access checks into app services, especially playback.
- Keep request/response DTO shapes and public API routes unchanged.

## Acceptance Criteria

- [ ] `crates/nako-server/src/http/renderer.rs` no longer imports or calls
      `require_source_access` / `RequiredLibraryAccess` for
      `/renderers/{renderer_session_id}/commands/play`.
- [ ] Renderer play command app-service execution rejects an ordinary principal
      without source `Play` access with `NakoError::Forbidden`.
- [ ] Rejected renderer play commands create no playback sessions, renderer
      commands, transcode sessions, or HLS artifacts.
- [ ] Existing renderer play success paths and playback-policy denial semantics
      remain green.
- [ ] Focused server check and renderer tests pass.

## Definition Of Done

- `cargo fmt --all`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-server renderer --no-fail-fast`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate 06-09-renderer-source-access-app`
- Task archived and journal recorded after commit/push.

## Technical Approach

Use the existing playback app-service access/policy pipeline as the enforcement
point instead of adding a new renderer-specific route helper. The renderer HTTP
handler should parse the source ID, pass the authenticated principal into
`CastingAppService`, and let `PlaybackAppService::start_renderer_playback_session`
own source access and playback policy.

Add focused app-level coverage around
`PlaybackAppService::start_renderer_playback_session` or the casting service
path, plus route-level coverage proving a browse-only/no-access principal is
forbidden and does not leave runtime records behind.

## Decision (ADR-lite)

**Context**: Source/library access checks are being migrated out of public HTTP
routes so non-HTTP callers cannot bypass permissions and HTTP remains a
translation boundary.

**Decision**: Move renderer play-command source access enforcement to the app
service path already responsible for renderer playback planning. Do not widen
this task into generic playback route access migration.

**Consequences**: Renderer command behavior becomes safer for future adapter or
internal callers. Playback route-local checks remain as deliberate follow-on
cleanup because those routes have multiple ticket/session/media-byte variants.

## Out Of Scope

- Playback HTTP route access migration.
- Browser playback ticket behavior.
- Renderer transport ticket validation behavior.
- Public DTO/schema/route changes.
- New durable runtime, scheduler, or transcode architecture changes.
- API docs changes unless the implementation changes externally visible
  behavior, which this task should avoid.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-server/backend/error-handling.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
- Relevant architecture:
  - `docs/architecture/CONTROL_PLANE.md`
- Main code hotspots inspected:
  - `crates/nako-server/src/http/renderer.rs`
  - `crates/nako-server/src/app/casting.rs`
  - `crates/nako-server/src/app/playback/renderer_flow.rs`
  - `crates/nako-server/src/http/tests/renderer.rs`
  - `crates/nako-server/src/app/tests/playback.rs`
