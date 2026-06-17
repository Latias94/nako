# Explicit Playback Profile Selection

## Goal

Let Public Client playback decision and new playback startup requests select a
specific current-user named playback profile by `playback_profile_id`, so a
client can use the correct saved device profile without changing the user's
default profile.

This is the next product step after named playback profile CRUD. Named profiles
should affect actual playback for the device that selects them, not only the
default fallback path.

## Requirements

- Add an optional `playback_profile_id` query parameter to new playback planning
  and startup surfaces that currently accept client capability query fields:
  - `GET /sources/{source_id}/playback/decision`
  - `GET|HEAD /sources/{source_id}/stream`
  - `GET|HEAD /sources/{source_id}/stream/remux`
  - `GET /sources/{source_id}/stream/hls/playlist.m3u8`
- Keep browser playback tickets, renderer transport URLs, existing playback
  sessions, HLS segment requests, heartbeat, and cancellation on their existing
  ticket/session-bound client context. They must not re-resolve
  `playback_profile_id` after the session/ticket is created.
- Selection precedence for new planning/startup requests:
  1. explicit capability query fields are authoritative;
  2. `playback_profile_id` selects that current user's named profile;
  3. the current user's default named profile is used when available;
  4. otherwise use built-in default client capabilities.
- If `playback_profile_id` is present together with explicit capability query
  fields, explicit capability fields win and the selected profile is ignored
  for that request.
- `playback_profile_id` is always current-user scoped. A profile id owned by
  another principal behaves like a missing profile.
- Missing, malformed, or cross-principal `playback_profile_id` must not leak
  another user's profile facts. Return the existing safe not-found/invalid-input
  style used by route/app boundaries.
- Stored profile JSON remains resolved effective capabilities. Invalid stored
  capability JSON must fail loudly and must not silently fall back to defaults.
- Update `nako-client-protocol`, `nako-api` OpenAPI/SDK generators,
  `nako-client-core`, `nako-client`, TypeScript SDK, and Kotlin SDK query
  surfaces so clients can pass `playback_profile_id`.
- Update HTTP/API/code-spec docs to capture the new precedence rule.

## Acceptance Criteria

- [ ] Public Client query DTOs and generated OpenAPI/SDK outputs expose
      optional `playback_profile_id`.
- [ ] Rust client-core and Rust client builders encode `playback_profile_id`
      safely in playback decision, Direct Stream, Remux, and HLS playlist
      request builders.
- [ ] Server playback decision uses the selected named profile when no explicit
      capability query fields are present.
- [ ] Direct Stream and HEAD startup persist selected profile capabilities into
      the new playback session when no explicit capability query fields are
      present.
- [ ] Remux Stream and HEAD startup persist selected profile capabilities into
      the new playback session when no explicit capability query fields are
      present.
- [ ] HLS playlist startup persists selected profile capabilities into the new
      playback session when no explicit capability query fields are present.
- [ ] Explicit capability query fields remain authoritative over
      `playback_profile_id`.
- [ ] Missing/cross-principal profile ids do not fall back to default profile in
      a way that hides the bad id.
- [ ] Browser tickets, renderer transport flows, and existing sessions remain
      session/ticket-bound.

## Definition of Done

- Focused Rust tests pass for `nako-client-protocol`, `nako-api`,
  `nako-client-core`, `nako-client`, and `nako-server` route behavior touched by
  the query contract.
- Generated TypeScript and Kotlin SDK files are refreshed from `nako-api`.
- Specs/docs are updated for the selection precedence.
- No unrelated dirty files are staged.

## Technical Approach

- Extend playback query structs with `playback_profile_id: Option<String>`.
- Treat `playback_profile_id` as a profile-selection hint, not as an explicit
  capability field. This preserves the existing rule that explicit capability
  fields suppress saved profile fallback.
- Add a server app helper that resolves client capabilities with precedence:
  explicit query capabilities, selected current-user profile, default profile,
  built-in defaults.
- Reuse the named profile repository added by the previous task; do not revive
  the legacy single-preference repository hot path.
- Keep the query parameter outside ticket/session transport routes unless a
  later task designs profile selection during ticket creation.

## Decision (ADR-lite)

**Context**: Named playback profiles exist and the default profile fallback
works, but clients with multiple devices still cannot request a non-default
saved profile for a single playback start.

**Decision**: Add an optional `playback_profile_id` query parameter to new
playback planning/startup routes, with explicit capability query fields taking
precedence and existing default fallback preserved when the parameter is
absent.

**Consequences**:

- Existing clients keep working without query changes.
- Device-specific clients can pick a stored profile without mutating the
  current user's default.
- Missing/cross-principal selected ids become visible errors instead of being
  hidden by default fallback.
- Ticket/session-bound media requests remain stable because they use the client
  capabilities captured at creation time.

## Out of Scope

- UI for choosing a profile.
- Admin global device profile catalogs.
- Sharing profiles across users or households.
- Mutating playback sessions to switch profiles after startup.
- Adding `playback_profile_id` to browser-ticket creation or renderer command
  flows in this slice.
- Deleting the old single-profile database table.

## Technical Notes

- Previous task: `.trellis/tasks/archive/2026-06/06-17-named-playback-profile-preferences`.
- Server route/query code: `crates/nako-server/src/http/playback.rs`.
- Server app fallback code: `crates/nako-server/src/app/playback/mod.rs`.
- Public route inventory and DTOs: `crates/nako-client-protocol/src/lib.rs`.
- OpenAPI/SDK generators: `crates/nako-api/src/openapi.rs` and
  `crates/nako-api/src/sdk.rs`.
- Rust client builders: `crates/nako-client-core/src/playback.rs` and
  `crates/nako-client/src/lib.rs`.
- HTTP docs: `docs/api/HTTP_API.md`.
- Code-spec anchors:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`,
  `.trellis/spec/nako-client-protocol/backend/index.md`,
  `.trellis/spec/nako-client-core/backend/index.md`,
  `.trellis/spec/nako-client/backend/index.md`.
