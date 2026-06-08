# User Playlist Jellyfin Comparison

## Reference Scope

This note records architecture observations only. `repo-ref/` is reference
material; Nako must use original implementation, naming, tests, SQL, comments,
and migrations.

## Jellyfin Lessons To Adapt

Jellyfin treats playlist listing as an owned collection read rather than a route
that manually re-derives item visibility and presentation one item at a time.
The useful architectural lesson for Nako is not a specific class or query. It is
the module split:

- collection ownership is checked before item expansion;
- item query semantics are centralized below the request handler;
- DTO creation consumes already-selected item and image facts;
- list routes avoid reimplementing access and pagination policy locally.

For Nako, the matching deep module is the User Playlist repository projection,
because Nako already has two real adapters at that seam: SQLite and PostgreSQL.

## Nako Gap

Current `crates/nako-server/src/http/user_playlist.rs` spreads repository,
Library Access, pagination, and catalog hydration knowledge across route-local
helpers:

- `accessible_playlist_item_records` scans raw playlist items in pages of
  `PageRequest::MAX_LIMIT`;
- each raw item calls `item_has_access`;
- `page_visible_items` slices after that scan;
- the visible page calls `app.catalog().get_item` once per row;
- `accessible_playlist_item_count` repeats the scan just to count visible rows.

This is a shallow HTTP module: deleting these helpers would push the same logic
into another caller. Moving the behavior behind the repository interface gives
callers more leverage and improves locality for access/pagination bugs.

## Projection Contract

The repository projection should return a single page object:

- `playlist`: owner-scoped `UserPlaylistRecord`;
- `accessible_item_count`: count after Media Item existence and Library Access
  filtering;
- `items`: bounded page of entries ordered by `position ASC, item_id ASC`.

Each entry should include:

- the original `UserPlaylistItemRecord`;
- the joined `MediaItem`;
- selected artwork rows plus managed artwork artifact facts for image refs.

The root playlist item rows must be filtered and paginated before selected
artwork is batch-loaded. Joining selected artwork before pagination risks page
boundary shifts when a Media Item has multiple selected images.

## Test Risks

Backend-neutral contract tests should prove:

- private playlist ownership is scoped to `principal_id`;
- ordinary users only see playlist items with user or role Library Access;
- inaccessible rows before the requested page do not consume offset slots;
- accessible count matches visible rows, not raw membership rows;
- source-less Media Items are visible to administrators and hidden from ordinary
  users;
- missing Media Items are excluded;
- selected artwork is hydrated only for visible page entries;
- stable ordering survives same or sparse positions.
