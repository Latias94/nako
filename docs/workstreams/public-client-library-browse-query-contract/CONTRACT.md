# Public Client Library Browse Query Contract

Status: Frozen for PLBQ-020
Last updated: 2026-05-29

## Purpose

This document freezes the first Public Client contract for library-scoped item
browse and stable catalog query keys. It is the implementation target for
PLBQ-030 and PLBQ-040.

The contract follows:

- ADR-0021 for video-first media-server scope and explicit browse facets/sort
  keys;
- ADR-0023 for the public v1 compatibility and error envelope;
- ADR-0025 for generated Public Client SDK ownership;
- `identity-and-library-access-contract` for effective Library Access;
- `user-playlists-contract-and-web-slice` remains separate from browse/query.

## Route Decision

Add one scoped browse route:

```text
GET /libraries/{library_id}/items
```

Do not add `library_id` to `GET /items` in the first implementation slice. The
scoped route keeps library access, route ownership, and web readiness states
unambiguous while preserving the existing all-visible-items route.

## Query Contract

```rust
pub struct LibraryItemsQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<ClientBrowseSortKey>,
    pub order: Option<ClientSortOrder>,
    pub facet: Vec<ClientBrowseFacetToken>,
    pub watch_state: Option<ClientWatchStateFilter>,
}
```

Wire shape:

```text
/libraries/{library_id}/items?limit=50&offset=0&sort=date_added&order=desc&facet=kind:movie,genre:sci-fi&watch_state=unwatched
```

`facet` may be repeated or comma-separated; the server normalizes both forms to
the same token list.

## Sort Keys

```rust
pub enum ClientBrowseSortKey {
    Title,       // sort_title fallback to title
    ReleaseDate,
    DateAdded,   // library/item state or accepted source-added proxy
    LastPlayed,  // current principal User Playback State
}

pub enum ClientSortOrder {
    Asc,
    Desc,
}
```

Default sort is `date_added desc`.

`last_played` is valid only for current-principal authenticated requests. Rows
with no user playback state sort after rows with a timestamp for `desc` and
after deterministic item id tie-breaks for stable pagination.

## Facet Tokens

Facet tokens use explicit public prefixes, not raw database column names:

```text
kind:<ClientMediaKind>
genre:<genre_id_or_slug>
tag:<tag_id_or_slug>
collection:<collection_id_or_slug>
studio:<studio_id_or_slug>
year:<yyyy>
content_rating:<region_or_source>:<value>
```

Unknown facet prefixes return the public error envelope with
`invalid_query_parameter`. Unsupported but known facets may return the same
error until the backend implements them.

## Watch State Filter

```rust
pub enum ClientWatchStateFilter {
    Any,
    Watched,
    Unwatched,
    InProgress,
}
```

Default is `any`. This filter is principal-scoped and must use the same
authenticated principal resolution as User Playback State. It must not expose
internal principal ids or user-state rows.

## Response Contract

```rust
pub struct LibraryItemsResponse {
    pub library: LibraryDto,
    pub items: Vec<MediaItemDto>,
    pub page: PageInfo,
}
```

The response intentionally reuses `MediaItemDto` rather than creating an Admin
or database-row DTO. Additional readiness/detail fields should be added only
when a concrete web/client workflow needs them.

## Access Behavior

- `browse` access to the library is required.
- If the authenticated principal cannot see the library, return `404 not_found`
  to avoid exposing library existence.
- Returned item rows must be filtered by effective Library Access before
  pagination is finalized.
- Public responses must not expose Admin policy rows, internal principal ids,
  raw Source Locators, local filesystem paths, provider raw payloads, or hidden
  item facts.

## SDK Expectations

Generated SDKs should expose:

```typescript
listLibraryItems(libraryId: string, query?: LibraryItemsQuery): Promise<LibraryItemsResponse>
```

Rust client support should expose an equivalent JSON request helper. The route
belongs to the Public Client route inventory as a JSON method.

## Web Expectations

`web/` may remove the current library-scoped browse readiness gap only after
PLBQ-030 implements the server/API/SDK contract. Until then,
`/media/library` must continue to show truthful readiness instead of fixture-only
scoped live items.
