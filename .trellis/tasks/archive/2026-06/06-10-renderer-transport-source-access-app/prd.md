# refactor: move renderer transport source access into app service

## Goal

Remove the last playback route-local source `Play` Library Access check from
renderer transport ticket use, so Direct, Remux, and HLS renderer transport
requests rely on the existing `PlaybackAppService` session-use boundaries for
current access rechecks.

## Requirements

- `resolve_renderer_transport_principal` and
  `resolve_renderer_transport_principal_for_session` validate renderer
  transport ticket identity, renderer ownership, network scope, source, mode,
  and playback session identity only.
- Renderer transport ticket use delegates source `Play` Library Access to the
  downstream `PlaybackAppService` method:
  - Direct: `direct_playback_session_stream` /
    `direct_playback_session_preflight`
  - Remux: `remux_playback_session_stream`
  - HLS playlist: `hls_playlist_for_playback_session`
  - HLS segment: `hls_segment_playback`
- Access revocation after renderer transport ticket issue remains effective at
  use time for Direct, Remux, and HLS renderer transport paths.
- A revoked or Browse-only principal receives the standard Library Access
  forbidden response before playback session reuse, lazy artifact startup,
  segment planning, or byte/playlist response details.
- Existing renderer ticket validation, wrong-ticket unauthorized behavior,
  renderer ownership checks, network-scope checks, URL shape, and response
  behavior remain unchanged.

## Acceptance Criteria

- [x] `resolve_renderer_transport_principal_for_session` no longer calls
      `require_source_access`.
- [x] HTTP renderer transport test proves a previously issued Direct renderer
      transport ticket is rejected after source `Play` access is revoked.
- [x] HTTP renderer transport test proves a previously issued Remux renderer
      transport ticket is rejected after source `Play` access is revoked before
      serving bytes or starting extra work.
- [x] HTTP renderer transport test proves a previously issued HLS renderer
      transport ticket is rejected after source `Play` access is revoked for
      playlist use.
- [x] Existing renderer transport ticket tests for Remux, HLS playlist/segment,
      Direct, invalid ticket, and transport URL redaction still pass.
- [x] `cargo fmt --all`, focused `cargo nextest`, `cargo check -p nako-server
      --tests`, `git diff --check`, and Trellis task validation pass or any
      skipped gate is recorded.

## Definition of Done

- HTTP renderer transport resolver remains responsible for ticket parsing,
  renderer lookup, ticket validation, and owner/network-scope checks.
- `PlaybackAppService` remains the source `Play` access authority for renderer
  transport use paths through the existing session-use methods.
- No renderer command issuing behavior, transport DTOs, browser playback
  tickets, Direct/Remux/HLS ordinary route behavior, schema, generated API, or
  playback planner changes.
- Task is validated, archived, recorded in journal, committed, and pushed.

## Technical Approach

Delete the route-local `require_source_access(... Play)` call from
`resolve_renderer_transport_principal_for_session`. The resolver already
returns a principal and playback session identity to downstream app-service
methods that now own access checks for Direct, Remux, HLS playlist, and HLS
segments.

Add renderer HTTP regression coverage by issuing cast-ticket transports while
the principal still has source `Play` access, deleting the principal's library
access policy, then using the previously issued transport URL. The expected
failure is `403 forbidden` with `required Library Access level 'play'`.

## Decision (ADR-lite)

**Context**: Direct, Remux, HLS, browser ticket use, and sidecar subtitle
playback have been migrated to app-service source `Play` access boundaries.
Renderer transport use is the remaining playback HTTP resolver that still
checks source `Play` access route-local.

**Decision**: Make renderer transport principal resolution auth/ticket-only and
rely on the playback app-service session-use methods for source `Play` access
at use time.

**Consequences**: Renderer transport behavior matches the rest of playback
access ownership while retaining route ownership for renderer ticket parsing
and renderer network/owner validation.

## Out of Scope

- Renderer transport ticket issue behavior.
- Renderer command queue payload shape or public DTOs.
- Direct, Remux, HLS ordinary browser playback behavior.
- Browser playback ticket behavior.
- Playback planner, storage, transcode, schema, generated API, or client
  changes.

## Technical Notes

- Relevant route file: `crates/nako-server/src/http/playback.rs`.
- Relevant renderer route tests:
  - `renderer_play_command_with_cast_ticket_remux_returns_ticketed_transport`
  - `renderer_play_command_with_cast_ticket_hls_protects_playlist_and_segments`
  - `synthetic_external_adapter_play_command_receives_cast_safe_transport_envelope`
- App-service methods already rechecking source `Play` access:
  - `direct_playback_session_stream`
  - `direct_playback_session_preflight`
  - `remux_playback_session_stream`
  - `hls_playlist_for_playback_session`
  - `hls_segment_playback`

## Verification

- `cargo fmt --all`
- `cargo nextest run -p nako-server renderer_transport_direct_rejects_revoked_source_play_access_at_use --no-fail-fast`
- `cargo nextest run -p nako-server renderer_transport_remux_rejects_revoked_source_play_access_at_use --no-fail-fast`
- `cargo nextest run -p nako-server renderer_transport_hls_rejects_revoked_source_play_access_at_playlist_use --no-fail-fast`
- `cargo nextest run -p nako-server renderer_play_command_with_cast_ticket --no-fail-fast`
- `cargo nextest run -p nako-server synthetic_external_adapter_play_command_receives_cast_safe_transport_envelope --no-fail-fast`
- `cargo check -p nako-server --tests`
- `git diff --check`
- `python .\.trellis\scripts\task.py validate 06-10-renderer-transport-source-access-app`
