# User Playlist Items Repository Projection

## Goal

Move User Playlist item listing and accessible item counts out of the HTTP
handler's per-row orchestration and into a repository-backed projection.

This should deepen the User Playlist repository module: callers ask for the
playlist item page they need, while the repository adapters own access filtering,
root pagination, and bounded hydration of Media Item and Selected Artwork facts.

## Problem

The current Public Client route for `GET /users/me/playlists/{playlist_id}/items`
does too much work in the HTTP layer:

- It loads the playlist separately.
- It scans every playlist item page.
- It calls per-item Library Access helpers before slicing the public page.
- It paginates in memory after access checks.
- It calls catalog item detail hydration once per visible row.
- Playlist DTO item counts are based on the same unbounded visible-item scan.

That makes the route shallow, hard to test through the right interface, and
vulnerable to N+1 query behavior as playlists grow.

## Requirements

- Keep public `UserPlaylistItemsResponse`, `UserPlaylistDto`, and
  `UserPlaylistItemDto` wire shapes unchanged.
- Add a repository projection returning:
  - the owned `UserPlaylistRecord`;
  - the accessible item count for the current authenticated principal;
  - a bounded page of item entries containing `UserPlaylistItemRecord`,
    associated `MediaItem`, and selected artwork plus managed artifact facts.
- Keep playlist ownership scoped to `principal.principal_id` before access
  filtering. A user cannot list another principal's playlist through access to
  the underlying Media Items.
- Exclude playlist entries whose `MediaItem` no longer exists.
- Exclude ordinary-user entries without sufficient Library Access before
  `LIMIT/OFFSET`.
- Administrator principals may see source-less playlist entries when the
  `MediaItem` exists.
- Ordinary user and Role Library Access policies both grant visibility when
  they allow Browse, Play, or Manage.
- Preserve stable item ordering by `position ASC, item_id ASC`.
- Count accessible items, not raw playlist membership rows.
- Batch-load selected artwork only for the bounded root page.
- The HTTP handler for playlist item listing must not call `item_has_access` or
  `CatalogAppService::get_item`.
- `public_playlist_dto` count behavior may remain conservative for list/get
  playlist routes unless this slice can reuse the projection without expanding
  scope.

## Out Of Scope

- No public DTO, SDK, or route shape change.
- No schema migration unless implementation proves one is required.
- No playlist sharing, collaboration, or visibility expansion.
- No reorder semantics change.
- No broad catalog detail hydration refactor.
- No copying source, tests, migrations, or comments from Jellyfin reference
  code.

## Acceptance

- `nako-core` exposes the projection contract in `UserPlaylistRepository`.
- SQLite and PostgreSQL adapters implement equivalent query semantics.
- Backend-neutral contract tests cover access-before-pagination, count, stable
  ordering, role policy visibility, admin source-less visibility, owner scoping,
  and selected artwork hydration for the bounded page.
- Server code maps projection entries to existing Public Client DTOs without
  per-row access checks or per-row catalog detail lookup in the list route.
- Focused checks pass:
  - `cargo check -p nako-core -p nako-db -p nako-server --tests`
  - `cargo nextest run -p nako-db user_playlist --no-fail-fast`
  - `cargo nextest run -p nako-server user_playlist --no-fail-fast`
  - `cargo fmt --all`
  - `git diff --check`
