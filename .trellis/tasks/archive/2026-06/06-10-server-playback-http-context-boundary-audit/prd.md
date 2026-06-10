# refactor: audit playback HTTP context boundaries

## Goal

Move one remaining playback HTTP context-resolution responsibility out of
`crates/nako-server/src/http/playback.rs` and into the playback app boundary, so
HTTP routes remain request/response translators while renderer transport ticket,
renderer ownership, playback session, source, and mode validation live in app
logic.

## What I Already Know

- The previous server cleanup removed obsolete HTTP access helpers and left
  `http/playback.rs` as the next area with route-local business context logic.
- `resolve_renderer_transport_principal_for_session` currently parses renderer
  transport inputs, loads renderer state, validates the renderer transport
  ticket scope, and checks owner principal consistency inside the HTTP module.
- `app/playback/renderer_flow.rs` already owns renderer playback session
  orchestration for source/probe context, `RemoteControl` policy enforcement,
  playback planning, and transport plan construction.
- `docs/architecture/PLAYBACK.md` states that renderer playback transport
  orchestration is a server-side app boundary, while ticket issuance and URL
  authoring can remain in HTTP renderer routes.

## Assumptions

- Query string extraction and string ID parsing may remain in HTTP for this
  slice, because `http-api-patterns.md` treats query/path parsing as an HTTP
  boundary.
- The app service should receive typed IDs and the raw renderer ticket token,
  then own renderer lookup, ticket scope construction, ticket validation, and
  owner-principal validation.
- Invalid renderer transport tickets must continue returning the existing
  public `401 unauthorized` body text: `invalid renderer transport ticket`.

## Requirements

- Add a focused playback app-service entry point that resolves a renderer
  transport ticket into the authenticated principal and playback session context
  needed by Direct, Remux, and HLS session transport routes.
- Keep `http/playback.rs` responsible for Axum extractors, optional query
  presence semantics, typed path/query parsing, response streaming, and response
  header insertion.
- Preserve existing renderer transport behavior:
  - no `playback_session_id` and no renderer ticket means normal browser/auth
    flow continues;
  - no `playback_session_id` with a renderer ticket is rejected;
  - typed `playback_session_id` with no renderer ticket continues as non-renderer
    session flow where currently supported;
  - blank or mismatched renderer transport tickets are rejected;
  - ticket validation includes renderer session id, playback session id,
    source id, playback mode, and renderer network scope;
  - ticket principal must match the renderer owner principal.
- Add or update focused tests to prove the app boundary validates successful and
  rejected renderer transport tickets without relying on route-local business
  logic.
- Run focused server checks after implementation.

## Acceptance Criteria

- [x] `http/playback.rs` no longer constructs `ValidateRendererTransportTicketRequest`
      or checks renderer owner-principal equality directly.
- [x] A playback app-service method owns renderer transport principal/session
      resolution from typed request data.
- [x] Existing Direct, Remux, HLS playlist, and HLS segment renderer transport
      routes preserve their public status/body behavior.
- [x] Focused tests cover successful renderer ticket resolution and at least one
      invalid-scope/owner mismatch rejection at the app boundary.
- [x] `cargo fmt --all`, focused `cargo nextest`, and `cargo check -p nako-server --tests`
      are run or any inability is documented.

## Out of Scope

- Reworking all browser playback ticket flows.
- Changing public API DTOs, generated SDK shape, URL query names, ticket token
  format, ticket TTL, or ticket storage.
- Moving subtitle playback ticket resolution in this task.
- Changing renderer ticket issuance or renderer command URL authoring.
- Changing playback planner, transcode runtime, storage, or database schema.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-server/backend/error-handling.md`
  - `.trellis/spec/nako-server/backend/logging-guidelines.md`
  - `.trellis/spec/nako-server/backend/quality-guidelines.md`
  - `.trellis/spec/nako-server/backend/directory-structure.md`
  - `.trellis/spec/nako-playback/backend/index.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
- Relevant architecture:
  - `docs/architecture/PLAYBACK.md`
- Candidate implementation files:
  - `crates/nako-server/src/http/playback.rs`
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/renderer_flow.rs`
  - `crates/nako-server/src/app/renderer_transport_ticket.rs`
- No external research is required; the choice is governed by existing project
  architecture and Trellis specs.
