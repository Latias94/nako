# Media Library Items Browse Capability

## Scope

Local repo research for adding library-scoped item browsing to Media Web.

## Findings

- `MediaLibraryDetailPage` currently renders library metadata and a paginated
  "Library sources" panel using `listLibrarySources(libraryId, search)`.
- `/media/libraries/$libraryId` currently validates only `limit` and `offset`.
- The generated Public Client TypeScript SDK already exposes
  `listLibraryItems(libraryId, query?: LibraryItemsQuery)`.
- `LibraryItemsQuery` supports `limit`, `offset`, `sort`, `order`, `facet`, and
  `watch_state`.
- Server `/libraries/{library_id}/items` parses raw query with:
  - repeated or comma-separated `facet`;
  - `sort`: `title`, `release_date`, `date_added`, `last_played`;
  - `order`: `asc`, `desc`;
  - `watch_state`: `any`, `watched`, `unwatched`, `in_progress`.
- Existing top-level `/media/items` now owns the same browse controls, but live
  top-level `/items` remains pagination-only.
- For library-scoped browse, live forwarding should use `client.listLibraryItems`
  and pass the rich query directly.

## Constraints

- Keep route search URL-owned.
- Do not render source/backend error strings from Media Web read failures.
- Keep existing library sources panel behavior unless deliberately changed.
- Avoid adding a new route for the MVP; reuse the library detail route surface.

