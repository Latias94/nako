# Public Library Browse Scale Contract

## Goal

Harden the Public Client Media Library browse, search, and selected artwork delivery contracts so frontend and SDK consumers can combine sorting, filtering, search text, pagination, access boundaries, and cache validators without relying on fixture-only behavior.

## What I Already Know

- The backend maturity execution plan identifies this as U2: Public Browse, Search, And Cache Contract Hardening.
- Public browse already has database contract coverage for title, release date, date-added, and last-played sort behavior.
- Public HTTP route coverage now includes stable library item sort pagination and selected artwork private cache validators through commit `f417e3f6`.
- Search access filtering exists in `nako-db` through `AccessibleSearchIndex` and is surfaced by `CatalogAppService::search_accessible_items`.
- The remaining useful gap is narrower than the original U2 scope: prove combined search text, repeated/comma facets, bounded pagination, and Library Access filtering through route and repository contracts.

## Requirements

- Public browse/list responses must remain bounded by `PageInfo { limit, offset, returned }` and must not imply total-count semantics.
- Library item browse must preserve stable ordering for supported sort keys and access-filter before pagination.
- Search must support text plus repeated and comma-separated facet filters.
- Search and browse must never return items outside effective Library Access.
- Selected artwork byte routes must keep private cache-control, ETag, and `304 Not Modified` behavior scoped behind the existing access check.
- This task may add focused characterization tests and small parser/contract fixes. It must not add new route families, schema migrations, Admin-only fields, or frontend behavior.

## Acceptance Criteria

- [x] Library item browse route proves stable `title` and `last_played` ordering across pagination.
- [x] Selected artwork image route proves `Cache-Control: private, max-age=86400`, ETag, GET/HEAD, and `If-None-Match` `304` behavior.
- [ ] Search route proves combined text plus repeated/comma facets return only matching accessible items.
- [ ] Search route proves access filtering happens before public pagination for hidden hits.
- [ ] Repository-level search/access contract remains green for SQLite and the optional PostgreSQL contract family where available.
- [ ] No public DTO exposes source locators, local paths, principal IDs, raw storage identity, provider payloads, tokens, or backend URLs.

## Definition of Done

- Focused `nako-server` catalog/library HTTP tests pass.
- Focused `nako-db` catalog/search/access contract tests pass where affected.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- `python ./.trellis/scripts/task.py validate '.trellis/tasks/06-16-public-library-browse-scale-contract'` passes.
- API or protocol contract checks run if DTOs or public route inventories change.

## Technical Approach

1. Treat current browse/cache tests as committed baseline evidence.
2. Inspect existing search route, query parser, `AccessibleSearchIndex`, and DB contract tests before changing behavior.
3. Add focused HTTP and DB tests for the missing combined-search/facet/access cases.
4. Only change implementation if those tests reveal a real contract gap.
5. Keep all route responses redaction-safe and bounded.

## Decision (ADR-lite)

**Context**: The frontend already combines sorting, filtering, and search. The backend has most underlying pieces, but the product contract needs explicit evidence for self-hosted large-library behavior.

**Decision**: Continue U2 as contract hardening, not as a broad feature expansion. Start from characterization tests around current query shapes, then make minimal backend fixes only where behavior contradicts the contract.

**Consequences**: This protects frontend/SDK consumers and keeps the public API stable without creating new surface area. Broader search semantics such as richer facets, full text ranking changes, total counts, or external search engines stay separate follow-ons.

## Out of Scope

- Full-text search engine replacement.
- Public total-count semantics.
- New genre/tag/provider/year filters beyond currently supported facets unless a failing contract proves the parser already intended them.
- New Admin Web or media web UI changes.
- Schema migrations.
- Public exposure of provider governance, raw search projection rows, job internals, source locators, or storage identities.

## Verification Evidence So Far

- Commit `f417e3f6` added route tests for selected artwork private cache validators and library item stable sort pagination.
- `cargo nextest run -p nako-server library_items_route_uses_stable_sort_keys_with_pagination --no-fail-fast` passed.
- `cargo nextest run -p nako-server catalog_selected_artwork_image_route_uses_private_cache_validators --no-fail-fast` passed.
- `cargo fmt --all` passed for the committed browse/cache slice.

## Technical Notes

- Primary plan: `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`.
- Relevant architecture: `docs/architecture/STATE_ACCESS.md`, `docs/architecture/CONTROL_PLANE.md`.
- Relevant specs: `.trellis/spec/nako-server/backend/index.md`, `.trellis/spec/nako-api/backend/index.md`, `.trellis/spec/nako-client-protocol/backend/index.md`, `.trellis/spec/nako-db/backend/index.md`.
- Likely code areas: `crates/nako-server/src/http/catalog.rs`, `crates/nako-server/src/http/library.rs`, `crates/nako-server/src/http/query.rs`, `crates/nako-server/src/app/catalog.rs`, `crates/nako-db/src/accessible_search.rs`, `crates/nako-db/src/sqlite/search.rs`, `crates/nako-db/src/postgres/core_catalog.rs`, `crates/nako-db/src/contract_tests.rs`.
