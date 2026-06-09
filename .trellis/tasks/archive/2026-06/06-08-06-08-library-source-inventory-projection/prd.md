# Library Source Inventory Repository Projection

## Goal

Deepen the Media Library source inventory path by moving source hydration from
`LibraryAppService::list_library_sources` into a repository-backed projection.
The Public Client route should keep the same DTO shape and pagination behavior
while avoiding per-source `Media Item` and `Media Technical Facts` lookups.

## Background

The previous slice pushed Media Library item browse filtering, sorting, and
pagination into `LibraryItemRepository::list_library_items_for_browse`. The
adjacent source inventory route still uses a shallow app-layer projection:

- list one page of `Media Source` records;
- for every source, call `get_media_item(source.item_id)`;
- for every source, call `get_media_probe(source.id)`;
- map the hydrated values into `LibrarySourceResponse`.

Jellyfin keeps item/source inventory style responses behind query and DTO
hydration modules instead of forcing route code to coordinate every related
record. Nako should follow the same architectural direction without copying
Jellyfin implementation details.

## Requirements

- Add a Nako-owned repository projection for Media Library source inventory.
- The projection must return one bounded page of records containing:
  - the `Media Source`;
  - the optional `Media Item` for `source.item_id`;
  - the optional `Media Technical Facts` / probe for `source.id`.
- Preserve current route behavior:
  - library existence is still checked before listing sources;
  - source rows are ordered as the existing `list_media_sources` contract orders
    them;
  - `PageRequest` is clamped and pagination applies to sources, not hydrated
    child rows;
  - missing item or missing probe remains represented as `None`;
  - the existing `LibrarySourcesResponse` and `LibrarySourceResponse` DTOs stay
    unchanged.
- Implement SQLite and PostgreSQL adapter parity.
- Add backend-neutral contract coverage for the new projection.
- Update the server app service to delegate hydration to the repository
  projection and only perform DTO mapping.
- Do not add schema migrations, new HTTP routes, new API DTO fields, or
  destructive behavior.

## Architecture Notes

- This is a read-only repository projection, similar in spirit to Media Library
  item browse. The repository interface should be domain-shaped, not SQL-shaped.
- The projection belongs close to `MediaRepository` unless implementation
  evidence shows a better existing repository trait.
- Use `PageRequest` and domain records from `nako-core`; do not expose adapter
  row structs through `nako-core`.
- SQL joins should be parameterized. Dynamic SQL is not needed for this slice.
- The app service should remain thin: check the Media Library, call the
  repository projection, map the returned records into existing DTOs.

## Acceptance Criteria

- `LibraryAppService::list_library_sources` no longer loops over source rows
  and calls per-source `get_media_item` / `get_media_probe`.
- SQLite and PostgreSQL adapters implement the same repository projection.
- Contract tests cover:
  - hydrated source with existing item and probe;
  - source with missing probe still returns the source and item;
  - source whose item is missing still returns `item = None`;
  - library isolation;
  - stable ordering and pagination by the source ordering contract.
- Focused validation passes or the skipped reason is recorded:
  - `cargo check -p nako-core -p nako-db -p nako-server --tests`;
  - `cargo nextest run -p nako-db <source-inventory-filter> --no-fail-fast`;
  - focused server library/catalog route test for `/libraries/{id}/sources`;
  - `cargo fmt --all -- --check`;
  - `git diff --check`.

## Follow-On Candidates

- Continue Watching projection down into a repository-backed browse query that
  combines `User Playback State`, `Library Access`, `Media Item`, and selected
  artwork without handler-level filtering after pagination.
- Search hit hydration down into repository/search projection to remove
  per-hit `get_media_item` calls.
- User Playlist write path atomics for append/insert/remove before tackling full
  reorder semantics.
