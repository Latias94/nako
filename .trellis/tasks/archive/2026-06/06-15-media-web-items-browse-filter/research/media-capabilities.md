# Research: media stack capabilities for browse filters

- Query: Inspect local media web, Public Client SDK/client-core, and API/server contracts for capabilities relevant to browse filters: `facet`, `sort`, `order`, `watch_state`, and search query composition.
- Scope: internal
- Date: 2026-06-15

## Findings

### Files found

- `apps/admin-web/src/surfaces/media/MediaCore.ts` - Media Web route search types; browse pages currently own pagination only, search route owns `facet` and `q`.
- `apps/admin-web/src/surfaces/media/MediaPages.tsx` - Media Web pages; `/media/items` calls top-level `listItems(page)`, while `/media/search` calls `searchItems({ facet, limit, offset, q })`.
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts` - Media Web data source wrapper around generated Public Client SDK; exposes `listItems(page)` and `searchItems(query)`, but not `listLibraryItems`.
- `apps/admin-web/src/surfaces/media/MediaSession.tsx` - lazy data-source proxy; forwards only the current `MediaWebDataSource` methods.
- `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx` - route tests confirm Media Items currently passes only `{ limit, offset }`, and Media Search passes `{ facet, limit, offset, q }`.
- `crates/nako-client/src/lib.rs` - Rust HTTP client exposes richer `LibraryItemsQuery` for `/libraries/{library_id}/items`.
- `crates/nako-client-core/src/browse.rs` - transport-neutral browse builders support top-level `/items` pagination and `/search` `q`/comma-joined `facet`, but no library-items filter builder.
- `crates/nako-api/src/sdk.rs` - generated TypeScript/Kotlin SDK contracts include `LibraryItemsQuery` with `sort`, `order`, `facet`, and `watch_state`; TypeScript `NakoClient.listLibraryItems` forwards it.
- `crates/nako-client-protocol/src/catalog.rs` - Public Client protocol enum wire values for browse sort, order, and watch-state filters.
- `crates/nako-server/src/http/query.rs` - HTTP query parsing for `/libraries/{library_id}/items` supports `sort`, `order`, repeated or comma-separated `facet`, and `watch_state`.
- `crates/nako-server/src/http/library.rs` - route handler wires raw query into `LibraryItemsQuery::into_browse_query()` before calling library service.
- `crates/nako-core/src/media/library.rs` - domain browse query model defines defaults and supported enum variants.
- `crates/nako-db/src/sqlite/library_item.rs` and `crates/nako-db/src/postgres/core_catalog.rs` - DB implementations apply kind facets, watch-state filters, and browse sort/order.
- `crates/nako-db/src/contract_tests.rs` - repository contract tests cover date/title/release/last-played sorting, kind facets, pagination, and watched/unwatched/in-progress filters.

### Code patterns

- `MediaPageSearch` is only `{ limit, offset }`, while `MediaSearchRouteSearch` adds `facet?: string` and `q?: string` in `apps/admin-web/src/surfaces/media/MediaCore.ts:5` and `apps/admin-web/src/surfaces/media/MediaCore.ts:10`.
- `MediaWebDataSource.listItems` only accepts `PageQuery`; `searchItems` accepts `{ facet?: string | string[]; q?: string } & PageQuery` in `apps/admin-web/src/surfaces/media/mediaDataSource.ts:60` and `apps/admin-web/src/surfaces/media/mediaDataSource.ts:67`.
- Live Media Web currently maps `listItems(page)` to `client.listItems(page)` and search to `client.searchItems({ limit: 20, offset: 0, ...query })` in `apps/admin-web/src/surfaces/media/mediaDataSource.ts:126` and `apps/admin-web/src/surfaces/media/mediaDataSource.ts:129`.
- `/media/items` calls `loadMediaItems(source, search, ...)`, and `loadMediaItems` calls `source.listItems(page)`, so browse filters cannot currently reach the backend from that route in `apps/admin-web/src/surfaces/media/MediaPages.tsx:36` and `apps/admin-web/src/surfaces/media/MediaPages.tsx:136`.
- `/media/search` composes `facet`, `limit`, `offset`, and `q` from URL-owned state and sends them to `source.searchItems(...)` in `apps/admin-web/src/surfaces/media/MediaPages.tsx:251` and `apps/admin-web/src/surfaces/media/MediaPages.tsx:260`.
- App route normalization currently gives `/media/items` only page search, and `/media/search` only `facet/q/page`: `validateMediaPageSearch`, `normalizeMediaPageSearch`, `validateMediaSearch`, and `normalizeMediaSearch` in `apps/admin-web/src/App.tsx:1030`, `apps/admin-web/src/App.tsx:1039`, `apps/admin-web/src/App.tsx:1048`, and `apps/admin-web/src/App.tsx:1059`.
- Rust `NakoClient.list_library_items` targets `/libraries/{library_id}/items` and accepts `Option<LibraryItemsQuery<'_>>` in `crates/nako-client/src/lib.rs:194`.
- Rust `LibraryItemsQuery` serializes `sort`, `order`, `facet`, and `watch_state` as query params in `crates/nako-client/src/lib.rs:1104` and `crates/nako-client/src/lib.rs:1113`.
- Rust client test proves expected wire URL `?limit=25&offset=50&sort=last_played&order=desc&facet=kind%3Amovie&watch_state=in_progress` in `crates/nako-client/src/lib.rs:1647`.
- Client-core search builder trims blank `q`, joins `facets` with comma into one `facet` param, and appends page in `crates/nako-client-core/src/browse.rs:191`.
- Client-core top-level list items builder accepts only `CoreBrowsePagedRequestInput` and emits `/items` with page query in `crates/nako-client-core/src/browse.rs:84`; there is no client-core builder for `/libraries/{library_id}/items` filters.
- Server `LibraryItemsQuery::from_raw_query` collects repeated `facet` params and scalar `sort`, `order`, `watch_state` in `crates/nako-server/src/http/query.rs:31` and `crates/nako-server/src/http/query.rs:49`.
- Server parse defaults are `sort=date_added`, `order=desc`, `watch_state=any` in `crates/nako-server/src/http/query.rs:684`, `crates/nako-server/src/http/query.rs:696`, and `crates/nako-server/src/http/query.rs:758`.
- Supported server wire values are `sort`: `title`, `release_date`, `date_added`, `last_played`; `order`: `asc`, `desc`; `watch_state`: `any`, `watched`, `unwatched`, `in_progress`; `facet`: currently only `kind:<movie|series|season|episode|collection|extra|unknown>` in `crates/nako-server/src/http/query.rs:684`, `crates/nako-server/src/http/query.rs:696`, `crates/nako-server/src/http/query.rs:708`, and `crates/nako-server/src/http/query.rs:743`.
- Library route uses `RawQuery` specifically for `/libraries/{library_id}/items` and forwards parsed browse query to the service in `crates/nako-server/src/http/library.rs:119`.
- Domain defaults and supported browse query fields are centralized in `LibraryItemBrowseQuery` in `crates/nako-core/src/media/library.rs:22`.
- SQLite applies each kind facet as an `AND media_items.kind = ...`; multiple different kind facets therefore intersect and can produce an empty result, as shown by DB code in `crates/nako-db/src/sqlite/library_item.rs:107` and contract test in `crates/nako-db/src/contract_tests.rs:2207`.
- SQLite watch-state filter uses joined `user_playback_states`: watched requires `playback.watched = 1`, unwatched includes no row or watched false, in-progress requires watched false plus positive resume position in `crates/nako-db/src/sqlite/library_item.rs:206`.
- SQLite browse ordering maps title/release/date-added/last-played plus asc/desc to explicit `ORDER BY` clauses in `crates/nako-db/src/sqlite/library_item.rs:222`.
- Repository contract tests prove watched, unwatched, in-progress, kind facet, pagination, and sort behavior in `crates/nako-db/src/contract_tests.rs:2180`, `crates/nako-db/src/contract_tests.rs:2229`, and `crates/nako-db/src/contract_tests.rs:2275`.

### External references

- None. User requested local repo only.

### Related specs

- `.trellis/spec/admin-web/frontend/index.md` - points Admin Web work to route/search/data-source patterns.
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md` - requires URL-owned filters, `onSearchChange`, offset reset on filter changes, Admin/Public Client data-source boundaries, and Media Web lazy route separation.
- `.trellis/spec/nako-client-core/backend/index.md` - client-core should build transport-neutral requests, percent encode path/query, and avoid IO.
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` - public DTO/SDK contracts come from `nako-api`; generated artifacts should not be hand-edited.

## Caveats / Not Found

- The current Media Web Items browse route is not wired to the richer `/libraries/{library_id}/items` query. It calls top-level `/items`, whose current frontend and client-core path only supports pagination.
- The richer browse filters are library-scoped in the public API stack. A UI filter implementation needs either a selected library context or a deliberate API/client contract change for top-level `/items`.
- Search facets are supported separately by `/search` and use the search index path, not the library browse query path. Current Media Web Search already forwards `facet/q/page`.
- `facet` semantics differ between paths: search accepts label facets as strings passed to `SearchQuery::from_facet_labels`, while library browse parsing currently supports only `kind:*`.
- Multiple `kind` facets on library browse are currently intersected at DB level, not treated as OR.
- `crates/nako-client-core/src/browse.rs` lacks a transport-neutral library-items filtered request builder even though generated TypeScript/Kotlin SDK and Rust `nako-client` expose the filtered route.
- Admin API contracts do not own these media browse filters; relevant contracts are Public Client API/SDK contracts plus Admin Web's local media surface data-source wrapper.
