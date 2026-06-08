# Continue Watching Repository Projection

## Goal

Move Continue Watching from HTTP-layer per-row orchestration into a repository-backed **User Playback State** projection. The route should keep the public DTO unchanged while the persistence contract owns resumable filtering, **Library Access** filtering before pagination, stable ordering, and bounded hydration of **Media Item** and **Selected Artwork** data.

## What I Already Know

- Current HTTP route `GET /users/me/playback-state/continue-watching` asks `UserPlaybackAppService` for a page of `UserPlaybackState`, then loops per state to check **Library Access**, fetch catalog item detail, and selected images.
- This creates N+1 behavior across access checks and catalog hydration.
- The current access check happens after paging playback states. If a page contains inaccessible items, the route can return fewer visible items even when later accessible candidates exist.
- ADR 0028 requires **User Playback State** to stay principal-scoped and public routes to use `/users/me/...`.
- ADR 0029 and ADR 0030 require SQLite/PostgreSQL parity through repository traits and backend-neutral contract tests.
- Jellyfin's resume route pushes resumable/user/sort semantics into an item query and then uses DTO hydration options; Nako should mirror the architectural shape without copying Jellyfin internals.

## Requirements

- Keep `ContinueWatchingResponse` and `ContinueWatchingItemDto` wire shape unchanged.
- Add a domain-shaped repository projection that returns each continue-watching row with:
  - `UserPlaybackState`
  - the associated `MediaItem`
  - selected artwork records and their managed artwork artifact facts required for public image refs
- Filter to the requested principal before considering candidates.
- Exclude watched states and missing/zero `resume_position_ms`.
- Exclude states whose `MediaItem` no longer exists.
- Exclude items without sufficient **Library Access** before applying `LIMIT/OFFSET`.
- Preserve administrator behavior: administrator principals can browse all candidates, including items with no media sources.
- Preserve ordinary user behavior through user and role **Library Access** policies.
- Keep ordering stable: `last_played_at_ms DESC`, then `item_id ASC`.
- Keep all SQL backend-specific; do not expose SQL or `nako-api` DTOs through `nako-core` or `nako-db`.
- HTTP handler should map repository projection rows to public DTOs and no longer call per-item access checks or `CatalogAppService::get_item` for Continue Watching.

## Acceptance Criteria

- [ ] Public Continue Watching DTOs and OpenAPI shape remain unchanged.
- [ ] SQLite and PostgreSQL adapters implement the same repository contract.
- [ ] Backend-neutral contract covers access filtering before pagination.
- [ ] Backend-neutral contract covers administrator, user policy, and role policy access behavior.
- [ ] Backend-neutral contract covers missing item exclusion and missing artwork artifact tolerance/error behavior as specified by implementation.
- [ ] Existing principal-scoped continue-watching contract still passes.
- [ ] Server route tests prove the route returns visible items, filters revoked access, and preserves DTO shape.
- [ ] Focused checks pass for changed crates.

## Definition Of Done

- Code formatted with `cargo fmt --all`.
- Focused `cargo check` passes for `nako-core`, `nako-db`, and `nako-server` tests.
- Focused `cargo nextest` passes for Continue Watching repository and server route coverage.
- `git diff --check` passes.
- Task context and spec updates record the new repository projection rule if the implementation reveals durable guidance.
- A conventional commit records the slice when implementation and verification are green.

## Technical Approach

Use a deepened `UserPlaybackStateRepository` interface rather than pushing more orchestration into HTTP. The new module Interface should let callers ask for "Continue Watching entries visible to this authenticated principal" and receive fully hydrated domain records for the bounded page.

The repository adapter should:

- Build resumable candidates from `user_playback_states`.
- Join `media_items` and effective library visibility evidence.
- Resolve access through the current authenticated principal fields:
  - `principal_id` for playback state ownership
  - `user_id` and `roles` for **Library Access**
  - administrator roles as full access
- Aggregate item-source/library membership to one candidate row per `MediaItem` before pagination.
- Apply `LIMIT/OFFSET` after access filtering.
- Batch-load one-to-many child data after root pagination:
  - media item external ids
  - selected artwork rows and their managed artwork artifacts

The server app service should expose a Continue Watching projection method and the HTTP handler should only translate query/response shape.

## Decision (ADR-lite)

**Context**: Continue Watching currently has shallow separation: repository lists user states, while HTTP knows resumable visibility, item access, catalog hydration, image hydration, and page construction.

**Decision**: Add a repository-backed Continue Watching projection under the **User Playback State** repository seam. Keep public DTO mapping in server/API crates, but move persistence-dependent filtering, ordering, and page hydration into SQLite/PostgreSQL adapters.

**Consequences**: The repository trait becomes slightly wider, but callers gain leverage and locality. Contract tests can now prove page boundaries, access filtering, and backend parity directly. The projection intentionally changes behavior so inaccessible candidates no longer create empty page holes.

## Out Of Scope

- No public DTO or SDK route shape changes.
- No total count support for Continue Watching.
- No new schema migration unless the existing tables cannot support the projection.
- No Jellyfin compatibility layer or Jellyfin source reuse.
- No broader catalog detail hydration refactor beyond the fields needed by Continue Watching.

## Research References

- [`research/continue-watching-jellyfin-comparison.md`](research/continue-watching-jellyfin-comparison.md) - local comparison of Jellyfin resume-item architecture and Nako's current shallow module.

## Technical Notes

- Main implementation files:
  - `crates/nako-core/src/repository/user_playback.rs`
  - `crates/nako-db/src/sqlite/user_playback.rs`
  - `crates/nako-db/src/postgres/playback_runtime.rs`
  - `crates/nako-db/src/facade.rs`
  - `crates/nako-db/src/contract_tests.rs`
  - `crates/nako-server/src/app/user_playback.rs`
  - `crates/nako-server/src/http/user_playback.rs`
  - `crates/nako-server/src/http/tests/user_playback.rs`
- Related authority:
  - `CONTEXT.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
  - `docs/adr/0029-postgresql-ready-persistence-boundary.md`
  - `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
  - `docs/architecture/STATE_ACCESS.md`
  - `.trellis/spec/nako-db/backend/quality-guidelines.md`
