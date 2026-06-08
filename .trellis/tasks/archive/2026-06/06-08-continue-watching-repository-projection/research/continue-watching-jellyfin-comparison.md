# Continue Watching Jellyfin Comparison

## Scope

This note compares Nako's Continue Watching implementation with Jellyfin's resume-item flow as reference architecture only. Code under `repo-ref/jellyfin` is not copied or translated; it is used to identify mature media-server module shape and failure modes.

## Jellyfin Reference Shape

Observed files:

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ItemsController.cs`
  - `GetResumeItems`
  - legacy route wrapper `GetResumeItemsLegacy`
- `repo-ref/jellyfin/Emby.Server.Implementations/Library/LibraryManager.cs`
  - `GetItemsResult`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Item/BaseItemRepository.TranslateQuery.cs`
  - `IsResumable` query translation
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Item/OrderMapper.cs`
  - `DatePlayed` ordering
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Item/BaseItemRepository.QueryBuilding.cs`
  - filtered/sorted query paging
- `repo-ref/jellyfin/MediaBrowser.Controller/Entities/UserViewBuilder.cs`
  - `GetMovieResume`
  - `GetTvResume`
- `repo-ref/jellyfin/Emby.Server.Implementations/Dto/DtoService.cs`
- `repo-ref/jellyfin/MediaBrowser.Controller/Library/IUserDataManager.cs`

Key architectural observations:

- Resume listing is not assembled by fetching arbitrary user data rows and then filtering in the controller. Jellyfin routes build an item query with resumable/user/sort constraints.
- `GetResumeItems` uses an item query shape with `IsResumable = true`, user identity, paging, sorting by recently played state, and `DtoOptions`.
- `LibraryManager.GetItemsResult` folds user-visible scope into the item query instead of making the controller rediscover visibility row by row.
- `BaseItemRepository.TranslateQuery` translates resumable state into repository-level user playback-position conditions.
- `OrderMapper` maps `DatePlayed` to current-user playback state ordering, so sorting and paging are query-level behavior.
- `BaseItemRepository.QueryBuilding` applies filtering and ordering before paging the root item set.
- `UserViewBuilder` applies the same conceptual query shape for movie and TV resume views.
- DTO hydration is an explicit stage after the item query returns a bounded result set. The controller does not manually loop through each candidate to rediscover visibility and item data.

## Nako Current Shape

Observed files:

- `crates/nako-server/src/http/user_playback.rs`
  - `list_continue_watching`
  - `continue_watching_item`
- `crates/nako-server/src/app/user_playback.rs`
  - `UserPlaybackAppService::list_continue_watching`
- `crates/nako-core/src/repository/user_playback.rs`
  - `UserPlaybackStateRepository::list_continue_watching_states`
- `crates/nako-db/src/sqlite/user_playback.rs`
- `crates/nako-db/src/postgres/playback_runtime.rs`

Current flow:

1. Repository returns a paged list of `UserPlaybackState`.
2. HTTP loops over states.
3. For each state, HTTP calls `item_has_access`.
4. `item_has_access` lists item sources and resolves effective **Library Access** per source.
5. For each visible state, HTTP calls `CatalogAppService::get_item`.
6. Catalog detail loads item, sources, credits, genres, tags, collections, studios, and selected images, even though Continue Watching only needs `item` and `images`.

Problems:

- The repository Interface is shallow: it exposes only playback state rows while every caller must know how to turn those rows into a visible Continue Watching row.
- Page boundaries are wrong for access filtering. Inaccessible rows are removed after pagination, so visible candidates after the page window do not backfill the response.
- Access logic is duplicated by composition rather than represented as query behavior.
- Hydration is too broad because `get_item` is a full item detail path.
- The route has poor locality: changing Continue Watching visibility or hydration requires reasoning across HTTP, app services, catalog service, access helper, and DB adapters.

## Deepening Opportunity

Create a deep module at the `UserPlaybackStateRepository` seam:

- Interface: list continue-watching entries visible to an `AuthenticatedPrincipal`.
- Implementation: backend-specific SQL filters resumable user state, joins Media Item membership, applies **Library Access**, orders and pages root rows, then batch-hydrates selected artwork.
- Leverage: server route gets a single bounded, correctly filtered list with the exact domain data needed for DTO mapping.
- Locality: page, access, and hydration behavior is contract-tested once against SQLite/PostgreSQL.

## Recommended Contract

The projection should return a domain record, not a public DTO:

- `UserPlaybackState`
- `MediaItem`
- selected artwork rows paired with managed artwork artifact facts

Repository contract cases:

- Current principal rows only.
- Exclude watched rows.
- Exclude `None` or zero resume positions.
- Exclude missing Media Items.
- User policy allows browse/play/manage.
- Role policy allows browse/play/manage.
- Administrator sees all candidates, including items without sources.
- Inaccessible first-page candidate does not create a page hole.
- Stable ordering by `last_played_at_ms DESC, item_id ASC`.
- Selected artwork hydration is bounded to returned page items.

## Risks

- SQL must avoid joining one-to-many selected artwork before root pagination.
- `last_played_at_ms` is nullable in the existing state model. Existing Continue Watching rows generally come from positive progress reports, but the projection should keep explicit ordering semantics. If a row has resume position but null last-played time, it should sort after rows with a timestamp while preserving the item-id tie-break.
- Effective **Library Access** depends on both user policies and role policies. The repository method needs enough principal data to resolve it without server-side loops.
- PostgreSQL ignored tests compile locally but require `NAKO_TEST_POSTGRES_URL` to execute.
