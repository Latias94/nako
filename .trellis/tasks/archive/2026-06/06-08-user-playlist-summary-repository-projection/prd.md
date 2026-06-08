# User Playlist Summary Repository Projection

## Goal

Move user playlist list/get summary counts behind a repository-backed projection so the Public Client playlist surface can return accessible item counts without issuing one item projection per playlist.

This deepens the user playlist repository module: ownership, Library Access filtering, missing Media Item handling, ordering, and count semantics live behind one repository interface instead of being rebuilt in HTTP handlers.

## Requirements

- Preserve existing Public Client DTOs and routes for:
  - `GET /users/me/playlists`
  - `GET /users/me/playlists/{playlist_id}`
  - playlist mutation responses that return `UserPlaylistDto`.
- Add a repository-backed playlist summary projection that returns a `UserPlaylistRecord` plus the current principal's accessible item count.
- Keep playlist ownership scope before item access filtering.
- Keep list ordering stable: `updated_at_ms DESC, id ASC`.
- Apply `PageRequest` pagination to root playlist rows, not to joined playlist item rows.
- Count only playlist entries whose `Media Item` row still exists.
- Honor ordinary User and Role `Library Access` before counting.
- Preserve existing admin source-less semantics used by the item projection: admin can count source-less existing Media Items, ordinary users cannot access them without allowed source membership.
- Keep mutation behavior unchanged.

## Acceptance Criteria

- [ ] `list_user_playlists` no longer loops over playlists and calls `get_items_projection(... PageRequest::new(1, 0))` per playlist.
- [ ] Single-playlist response mapping can use the same summary projection path where practical.
- [ ] SQLite and PostgreSQL adapters implement the new repository contract with parity.
- [ ] Backend-neutral repository contract tests cover ordering, pagination, ordinary user Library Access counts, missing Media Item exclusion, and admin source-less count semantics.
- [ ] Server behavior tests still prove route DTO shape and item projection behavior.
- [ ] Focused checks pass:
  - `cargo check -p nako-core -p nako-db -p nako-server --tests`
  - `cargo nextest run -p nako-db user_playlist --no-fail-fast`
  - `cargo nextest run -p nako-server user_playlist --no-fail-fast`
  - `cargo fmt --all`
  - `git diff --check`

## Definition of Done

- Tests are added or updated at the repository/app/route level according to risk.
- SQLite and PostgreSQL implementations remain structurally comparable.
- No public DTO, route, or migration changes are introduced.
- Task context is recorded in `implement.jsonl` and `check.jsonl`.
- Work is committed with a Conventional Commit message.

## Technical Approach

Add a small core record, tentatively `UserPlaylistSummaryProjection`, and a repository method for:

- Listing playlist summaries for a principal and page.
- Reading one playlist summary for a principal and playlist id.

The database adapters should first select owner-scoped playlist root rows, then aggregate accessible item counts using the same Library Access predicate family as the existing item projection. The HTTP layer should map summary projections directly to `UserPlaylistDto`.

## Decision (ADR-lite)

**Context**: The existing route-level mapping calls the item projection once per playlist just to obtain `total_items`, which duplicates access/count knowledge across layers and makes playlist list pages scale with playlist count.

**Decision**: Move count semantics into the repository interface, adjacent to the item projection SQL and contract tests.

**Consequences**: Repository SQL becomes slightly deeper, but callers get a smaller and more reliable interface. Contract tests become the primary test surface for count/access behavior, while HTTP tests stay focused on route shape and DTO mapping.

## Out of Scope

- No route, SDK, or DTO shape changes.
- No playlist sharing, collaboration, or visibility semantics beyond current owner-scoped behavior.
- No schema migrations.
- No change to playlist item mutation semantics.
- No copying code or tests from Jellyfin reference sources.

## Technical Notes

- Relevant prior slice:
  `.trellis/tasks/archive/2026-06/06-08-user-playlist-items-repository-projection`
- Current likely files:
  - `crates/nako-core/src/repository/user_playlist.rs`
  - `crates/nako-db/src/sqlite/user_playlist.rs`
  - `crates/nako-db/src/postgres/user_playlist.rs`
  - `crates/nako-db/src/facade.rs`
  - `crates/nako-db/src/contract_tests.rs`
  - `crates/nako-server/src/app/user_playlist.rs`
  - `crates/nako-server/src/http/user_playlist.rs`
- Architecture vocabulary:
  - This is a Module deepening: the repository interface hides access-filtered count implementation and improves locality.
  - The repository interface is the test surface.
