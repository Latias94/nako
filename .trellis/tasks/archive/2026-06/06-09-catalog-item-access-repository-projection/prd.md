# Catalog item access repository projection

## Goal

Move Public Catalog item-list access filtering out of HTTP post-processing and
into repository-backed projections where possible. The first slice fixes page
holes for browse item lists by applying Library Access before `LIMIT/OFFSET`,
keeps public DTO shapes unchanged, and reduces per-item `item_has_access`
queries on the main browse/search routes.

## What I already know

* Nako is a self-hosted media server. Public Catalog routes expose `Media Item`
  browse/search results to an authenticated principal.
* Current `crates/nako-server/src/http/catalog.rs` filters `/items`,
  `/people/{person_id}/items`, `/tags/{tag_id}/items`,
  `/genres/{genre_id}/items`, and `/search` after the app service has already
  returned a bounded page.
* Post-page filtering creates page holes: inaccessible `Media Item` rows consume
  slots before the visible page is built.
* Jellyfin's comparable pattern injects user/library visibility into the query
  before filtering, ordering, paging, and DTO hydration.
* Recent Nako slices already moved Continue Watching and User Playlist item
  projections to repository-backed access-before-pagination queries.

## Requirements

* Keep public route paths and `nako_api::public_client` DTO shapes unchanged.
* Add bounded repository methods for Public Catalog item browse:
  * all item list;
  * items for a `Person`;
  * items for a `Tag`;
  * items for a `Genre`.
* For non-admin principals, a `Media Item` is visible when at least one of its
  `Media Source` rows belongs to a `Media Library` where the principal has
  `browse`, `play`, or `manage` Library Access through a user or role policy.
* Administrator principals keep the existing item-access semantics, including
  source-less `Media Item` rows being visible.
* Filtering and deduplication must happen before `LIMIT/OFFSET`.
* Preserve existing root ordering: `title ASC, id ASC`.
* Keep app services responsible for mapping domain records to API DTOs.
* Remove per-item `item_has_access` loops from the item-list routes covered by
  the new repository methods.
* For `/search`, use a bounded transitional strategy:
  * retrieve search hits in score order;
  * hydrate/filter through a repository-owned batch access query;
  * avoid per-hit HTTP access checks;
  * record that full access-before-pagination search semantics remain a
    follow-up because the current `SearchIndex` contract paginates inside the
    search evaluator.

## Acceptance Criteria

* [ ] `GET /items` returns only accessible `Media Item` rows and does not let an
  inaccessible item consume the first page slot.
* [ ] `GET /people/{person_id}/items`,
  `GET /tags/{tag_id}/items`, and `GET /genres/{genre_id}/items` apply the same
  access-before-pagination semantics.
* [ ] `GET /search` no longer loops over hits in HTTP with `item_has_access`;
  search response shape stays unchanged.
* [ ] SQLite and PostgreSQL adapters implement the same repository contracts.
* [ ] Backend-neutral contract tests cover ordinary user policy, role policy,
  admin source-less semantics, duplicate-source deduplication, and pagination
  after access filtering.
* [ ] Focused server route tests cover the page-hole regression at the HTTP
  surface.

## Out of Scope

* No public DTO, SDK, or route path changes.
* No full search-index redesign in this slice.
* No root `people`, `tags`, or `genres` list aggregation rewrite; their current
  root visibility checks stay as a later catalog facet projection task.
* No mutation or Library Access policy semantics change.
* No schema migration unless implementation proves an index is required.

## Technical Notes

* Relevant files:
  * `crates/nako-core/src/repository/media.rs`
  * `crates/nako-core/src/repository/catalog.rs`
  * `crates/nako-db/src/sqlite/media.rs`
  * `crates/nako-db/src/sqlite/catalog.rs`
  * `crates/nako-db/src/postgres/core_catalog.rs`
  * `crates/nako-db/src/postgres/metadata_catalog.rs`
  * `crates/nako-db/src/facade.rs`
  * `crates/nako-db/src/contract_tests.rs`
  * `crates/nako-server/src/app/catalog.rs`
  * `crates/nako-server/src/http/catalog.rs`
  * `crates/nako-server/src/http/tests/catalog.rs`
* Existing access SQL pattern is in the User Playlist projection adapters.
* `SearchIndex::search` currently evaluates and pages hits inside
  `nako-search`; this task should not pull `nako-api` or SQL details into
  `nako-search`.

## Definition of Done

* `cargo check -p nako-core -p nako-db -p nako-server --tests`
* `cargo nextest run -p nako-db catalog_access --no-fail-fast`
* `cargo nextest run -p nako-server catalog --no-fail-fast`
* `cargo fmt --all`
* `git diff --check`
