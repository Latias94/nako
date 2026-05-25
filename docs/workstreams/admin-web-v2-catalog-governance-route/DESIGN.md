# Admin Web V2 Catalog Governance Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 still leaves Catalog Governance in `/legacy`, even though the
generated Admin API contract already exposes a list response and query DTO for
unknown or low-confidence Media Items. This lane moves that workflow into a
route-first V2 page while preserving Admin API contract ownership and redaction
safety.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
  - `docs/workstreams/admin-web-v2-product-architecture/CLOSEOUT.md`

## Problem

`/catalog/governance` is currently a placeholder. The legacy panel shows a
compact table, but it is buried inside a monolithic dashboard and cannot own URL
state, focused fallback handling, pagination, or route-specific redaction tests.

## Target State

When this lane closes:

- `/catalog/governance` is a real V2 route.
- The route reads `AdminCatalogGovernanceItemListResponse` through
  `AdminDataSource`.
- `AdminApiClient.getCatalogGovernanceItems()` accepts generated
  `AdminCatalogGovernanceItemsQuery` values.
- URL search params own `library_id`, `max_confidence_milli`, `limit`, and
  `offset`.
- The page uses existing V2 shell/table/filter primitives and deterministic
  fallback.
- Repair/detail workflows remain explicitly deferred.

## In Scope

- Route-owned Catalog Governance list page.
- Query support in the Admin API client and data-source seam.
- Filter bar for library and maximum local inference confidence.
- Focused tests for query mapping, fallback, route rendering, and unsafe text
  exclusions.
- Frontend validation and browser smoke evidence.

## Out Of Scope

- Catalog governance detail route.
- Repair, accept, split, merge, or provider-mapping mutations.
- Backend contract changes.
- Removing the `/legacy` Catalog panel before deeper workflow parity exists.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The current list DTO is sufficient for a read-only route. | High | `AdminCatalogGovernanceItemListResponse` includes item, library, local inference, issue, source, and mapping facts. | Split a backend contract follow-on for missing detail data. |
| Query params can be wired without backend changes. | High | `AdminCatalogGovernanceItemsQuery` is already generated. | Defer filters or add backend query support before closeout. |
| Repair workflows are separate. | High | Route readiness marks detail/repair routes as missing. | Open a separate lane for mutation semantics. |

## Architecture Direction

Keep Admin API calls in `adminApi/client.ts` and live/mock fallback in
`adminApi/dataSource.ts`. The route module owns display columns, filter controls,
search-param adapters, and safe rendering. Shared components may be reused, but
new abstractions should only be introduced if Catalog and Jobs prove the same
pattern twice.

## Closeout Condition

This lane can close when the route, query mapping, tests, frontend gates, and
browser smoke evidence are recorded, with detail/repair follow-ons explicitly
deferred.

Closeout status: met on 2026-05-25. See `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.
