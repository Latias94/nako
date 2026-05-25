# Admin Web V2 Storage Staging Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Storage staging diagnostics still live in `/legacy`, while `/storage/staging`
is a placeholder. The Admin API already exposes a staging diagnostics response
and generated query DTO, making it suitable for the next read-only V2 route.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
  - `docs/workstreams/storage-vfs/`

## Problem

The legacy dashboard only shows a compact Storage summary. Operators need a V2
route that can inspect staging records with URL-owned filters, deterministic
fallback, and explicit redaction safety.

## Target State

When this lane closes:

- `/storage/staging` is a real V2 route.
- `AdminApiClient.getStorageStaging()` accepts generated
  `AdminStorageStagingQuery` values.
- `AdminDataSource` exposes route-local live/mock fallback.
- URL search params own `purpose`, `state`, `limit`, and `offset`.
- The page renders summary facts plus a safe staging records table.
- Cleanup/delete/lease mutation workflows remain follow-ons.

## In Scope

- Route-owned Storage Staging page.
- Query support in the Admin API client and data-source seam.
- Filter bar for purpose, state, and limit.
- Tests for query mapping, fallback, route rendering, and unsafe text
  exclusions.
- Frontend validation and browser smoke evidence.

## Out Of Scope

- Staging cleanup mutations.
- VFS cache repair or deletion.
- Backend contract changes.
- Displaying raw Source Locators, storage roots, cache paths, or credentials.

## Architecture Direction

Follow the existing route pattern: `App.tsx` owns search normalization,
`adminApi/client.ts` owns Admin API query construction, `adminApi/dataSource.ts`
owns section fallback, and `features/storage` owns safe rendering.

## Closeout Condition

This lane can close when the route, query mapping, tests, frontend gates, and
browser smoke evidence are recorded, with cleanup and mutation workflows
explicitly deferred.

Closeout status: met on 2026-05-25. See `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.
