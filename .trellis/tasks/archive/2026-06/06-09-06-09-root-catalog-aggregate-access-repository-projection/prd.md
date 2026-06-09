# Root catalog aggregate access repository projection

## Goal

Move Public Catalog root aggregate visibility for `/people`, `/tags`, and `/genres` from HTTP-layer access-after-pagination helper loops into repository-backed access-before-pagination projections. This continues the catalog access refactor by removing page holes and HTTP N+1 access checks from aggregate list routes.

## What I already know

- `/items`, `/people/{id}/items`, `/tags/{id}/items`, and `/genres/{id}/items` already use repository-backed accessible item projections.
- `crates/nako-server/src/http/catalog.rs` still handles `/people`, `/tags`, and `/genres` by fetching a plain page, looping through returned DTOs, hydrating relation items with `PageRequest::MAX_LIMIT`, then calling `item_has_access` per item.
- Those root list handlers can return short pages when inaccessible aggregate rows are filtered after pagination.
- `CatalogRepository` currently exposes plain `list_people`, `list_tags`, `list_genres` and accessible relation item methods, but not accessible root aggregate list methods.
- SQLite and PostgreSQL adapters now have shared access helpers in `crates/nako-db/src/sqlite/access.rs` and `crates/nako-db/src/postgres/access.rs`.
- Existing adapter-local `list_accessible_catalog_item_rows` helpers show the intended SQL shape: join the relation table, apply `push_media_item_access_filter`, then paginate.

## Requirements

- Add repository methods for access-filtered root aggregate pages:
  - `list_accessible_people(principal, page)`
  - `list_accessible_tags(principal, page)`
  - `list_accessible_genres(principal, page)`
- Apply access filtering before `ORDER BY`, `LIMIT`, and `OFFSET` for ordinary and role users.
- Preserve administrator semantics by returning the same root aggregate rows admins get from the plain list methods, including aggregate rows that currently have no related item.
- Preserve deterministic ordering:
  - people: `name ASC, id ASC`
  - tags/genres: `name ASC, id ASC`
- Update app service and HTTP route handlers so `/people`, `/tags`, and `/genres` call repository-backed accessible projections directly.
- Remove the root list post-filter loops and the route-level `page_returned_len` repair for these routes.
- Keep DTO shapes and public response contracts unchanged.

## Acceptance Criteria

- [ ] Ordinary users only see people/tags/genres that are linked to at least one item with browse-or-higher access through user or role policy.
- [ ] Ordinary users do not get page holes when the first plain aggregate rows are inaccessible.
- [ ] Role-based library access grants aggregate visibility without user-specific policy.
- [ ] Administrators see all root aggregate rows, including aggregate rows with no related media item, matching current admin route behavior.
- [ ] SQLite and PostgreSQL adapters expose equivalent repository behavior through shared contract tests.
- [ ] HTTP catalog tests cover `/people`, `/tags`, and `/genres` hidden aggregate filtering and page-hole behavior.

## Definition of Done

- Tests added or updated at the repository contract layer and HTTP route layer.
- `cargo fmt --all` run after edits.
- Focused gates pass:
  - `cargo check -p nako-core -p nako-db -p nako-server --tests`
  - `cargo nextest run -p nako-db catalog_access --no-fail-fast`
  - `cargo nextest run -p nako-server catalog --no-fail-fast`
  - `git diff --check`
- Relevant Trellis spec updated only if the implementation discovers a new durable rule beyond the existing Public Catalog Library Access projection guidance.

## Technical Approach

Extend `CatalogRepository` in `nako-core` with three accessible root aggregate methods. Implement them in SQLite and PostgreSQL using adapter-local SQL builders that select the aggregate table, then apply an `EXISTS` relation-to-accessible-item predicate for non-admin principals before pagination.

For people, the predicate should use `item_credits.person_id`. For genres and tags, use `item_genres.genre_id` and `item_tags.tag_id`. The implementation should reuse the shared adapter access helpers rather than duplicating user/role policy SQL.

`CatalogAppService` should gain principal-aware list methods that map returned domain records to the existing public DTO responses. HTTP handlers should become thin calls to those app methods, following the recently completed `/items` and relation-item route pattern.

## Decision (ADR-lite)

**Context**: Current root aggregate list routes filter after pagination and perform bounded N+1 access checks in HTTP. This is inconsistent with the repository projection direction and can produce short pages.

**Decision**: Add repository-backed accessible root aggregate list methods and route `/people`, `/tags`, and `/genres` through them.

**Consequences**: This adds three repository trait methods and adapter implementations, but keeps access policy in the persistence boundary where pagination can be correct. Single-record aggregate getters remain unchanged for this slice.

## Out of Scope

- Full `/search` access-before-pagination redesign.
- Single-record `get_person`, `get_tag`, and `get_genre` access helper replacement.
- Collections, studios, or any aggregate type not currently exposed as a root Public Catalog list route.
- Schema or migration changes.

## Technical Notes

- Before modifying Rust code, invoke `rust-best-practices`.
- Relevant files identified during preparation:
  - `crates/nako-core/src/repository/catalog.rs`
  - `crates/nako-db/src/sqlite/catalog.rs`
  - `crates/nako-db/src/postgres/metadata_catalog.rs`
  - `crates/nako-db/src/sqlite/access.rs`
  - `crates/nako-db/src/postgres/access.rs`
  - `crates/nako-db/src/contract_tests.rs`
  - `crates/nako-server/src/app/catalog.rs`
  - `crates/nako-server/src/http/catalog.rs`
  - `crates/nako-server/src/http/tests/catalog.rs`
- See `research/initial-code-context.md` for the code inspection summary captured before implementation.
