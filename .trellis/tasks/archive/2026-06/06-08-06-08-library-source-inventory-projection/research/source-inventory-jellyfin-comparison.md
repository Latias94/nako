# Source Inventory Jellyfin Comparison Notes

## Summary

Two read-only exploration passes found the same local issue:
`LibraryAppService::list_library_sources` is still a shallow projection module.
It returns a page of `Media Source` rows, then performs per-source `Media Item`
and `Media Technical Facts` lookups before building the Public Client response.

## Nako Evidence

- `crates/nako-server/src/app/library.rs`
  - `list_library_sources` calls `list_media_sources`.
  - It then loops each source and calls `get_media_item(source.item_id)`.
  - It also calls `get_media_probe(source.id)`.
- `crates/nako-core/src/repository/media.rs`
  - `MediaRepository` exposes source, item, and probe reads as separate
    interfaces, but has no source inventory projection.
- `crates/nako-db/src/sqlite/media.rs` and
  `crates/nako-db/src/postgres/core_catalog.rs`
  - source listing is already bounded and ordered by locator/source identity;
    the missing piece is repository-level hydration.

## Jellyfin Reference Boundary

Jellyfin reference areas for architecture comparison:

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ItemsController.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/Dto/DtoService.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Dto/BaseItemDto.cs`

Use these only to study capability boundaries and DTO hydration shape. Do not
copy code, comments, schema, tests, or generated artifacts.

## Architecture Assessment

The current Nako interface is shallow:

- Caller complexity is almost as large as the implementation complexity.
- Deleting the app-layer loop would force the same source + item + probe join
  logic to reappear in another caller.
- The repository adapter is the module with the right locality for joining
  persisted source, item, and probe facts.

Expected deeper module:

- repository method takes `LibraryId` and `PageRequest`;
- adapter performs bounded source inventory hydration;
- app service maps domain projection records to existing DTOs.

## Recommended Slice

Implement a read-only `Library Source Inventory` repository projection.

Keep scope narrow:

- no API shape changes;
- no schema migrations;
- no new source sorting mode;
- no access-control changes;
- no raw locator/path diagnostic expansion.

## Follow-On Findings

Other candidates discovered by exploration:

- Continue Watching projection currently filters/hydrates after pagination and
  should become a repository-backed browse projection.
- Search hit hydration currently does per-hit `get_media_item`.
- Catalog governance summary currently paginates full records to count summary
  fields.
- User Playlist writes use full-list read/rewrite patterns that can be
  atomized in smaller follow-up slices.
