# Deepen catalog browse projection scale

## Goal

Deepen the Catalog Browse Module so library item browse filtering, sort keys,
watch-state filtering, and pagination are executed through a repository-backed
query instead of app-layer full-library loading and per-item `User Playback
State` lookups.

This is the first focused slice from the Jellyfin comparison campaign. Jellyfin
has mature query-backed browse behavior, but very wide manager-style
interfaces. Nako should keep the Public Client Interface stable and move the
implementation depth behind Nako repository seams.

## What I Already Know

* Current `LibraryAppService::filtered_library_items` loads all `Media Item`
  records for a `Media Library`, fetches `User Playback State` one item at a
  time, sorts in memory, then slices the requested page.
* ADR 0053 requires bounded, paginated Public and Admin list surfaces.
* ADR 0011 already prefers normalized catalog graph hydration and search
  projections over ad hoc browse records.
* The first slice can avoid schema and DTO changes by adding a repository query
  that joins existing `media_items`, `media_sources`, `library_item_states`,
  and `user_playback_states`.
* The current public route and DTO shape should remain unchanged.

## Requirements

* Preserve the existing `/libraries/{library_id}/items` response shape and query
  parameters.
* Add a bounded repository-level browse query for library items.
* Preserve existing filters:
  * `LibraryItemBrowseFacet::Kind`
  * `LibraryItemWatchStateFilter::{Any, Watched, Unwatched, InProgress}`
* Preserve existing sort keys:
  * `Title`
  * `ReleaseDate`
  * `DateAdded`
  * `LastPlayed`
* Preserve deterministic item-id tie-breaking.
* Avoid app-layer full-library loading for this browse path.
* Avoid per-item `get_user_playback_state` calls in this browse path.
* Keep SQL and row mapping inside `nako-db`; keep HTTP and DTO mapping inside
  `nako-server`/`nako-api`.

## Acceptance Criteria

* [ ] `LibraryAppService::list_library_items` delegates to a repository-backed
      browse method instead of `all_library_items`.
* [ ] SQLite adapter implements the browse query with SQL-level filtering,
      sorting, limit, and offset.
* [ ] PostgreSQL adapter has matching repository contract behavior.
* [ ] Repository contract tests cover facet filtering, watch-state filtering,
      stable pagination, and sort keys.
* [ ] Focused server tests still pass for library item listing.
* [ ] Focused DB tests pass for the new contract coverage.
* [ ] No Public Client DTO, Admin DTO, route, or schema change is introduced.

## Definition Of Done

* Tests added or updated for the repository contract and app route behavior.
* `cargo fmt --all -- --check` or scoped formatting is clean.
* Focused `cargo nextest` / `cargo check` gates pass.
* `python ./.trellis/scripts/task.py validate` passes for this task.
* If a new durable pattern is discovered, update the relevant Trellis spec.

## Technical Approach

Add a domain query result under `nako-core` for library item browse and expose it
through a repository trait. Implement the query in `nako-db` by joining existing
tables and applying SQL-level `WHERE`, `ORDER BY`, `LIMIT`, and `OFFSET`.

Keep the server app service shallow: confirm the `Media Library` exists, call
the repository browse method with the principal and query, then map returned
`Media Item` records to the existing Public Client DTO.

## Decision (ADR-lite)

**Context**: Jellyfin proves that browse scale belongs in repository/query
projections, but its manager interfaces are intentionally not copied into Nako.

**Decision**: For this slice, deepen the existing Nako repository seam rather
than adding a new crate, new DTO, or new SQL schema.

**Consequences**: This directly improves Locality for browse behavior and
removes a current N+1 pattern. It does not yet add richer browse facets,
search-ranking integration, materialized browse projections, or total-count
pagination.

## Out Of Scope

* Changing Public Client DTOs or route query parameter names.
* Adding total counts or cursor pagination.
* Adding new browse facets beyond the existing kind filter.
* Changing search ranking or `nako-search` query behavior.
* Adding schema migrations or materialized browse tables.
* Admin Web or Media Web UI changes.

## Technical Notes

* Related architecture: `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`.
* Control-plane/API scale baseline: `docs/adr/0053-application-control-plane-boundary.md`.
* Relevant specs:
  * `.trellis/spec/nako-core/backend/index.md`
  * `.trellis/spec/nako-db/backend/index.md`
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/nako-catalog/backend/index.md`
* Reference comparison:
  * Jellyfin query-backed browse behavior is useful as behavior coverage.
  * Jellyfin's wide `LibraryManager` style interface should not be copied.
