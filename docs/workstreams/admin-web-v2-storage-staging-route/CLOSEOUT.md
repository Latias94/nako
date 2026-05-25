# Admin Web V2 Storage Staging Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a real `/storage/staging` route
that replaces the placeholder with a route-first, read-only staging diagnostics
page backed by generated Admin API query DTOs and deterministic fallback
behavior.

This closeout does not claim Storage workflow parity. Cleanup, deletion,
repair, lease management, and VFS cache mutation semantics remain follow-ons.

## Delivered

- New `apps/admin-web/src/features/storage/StorageStagingPage.tsx`.
- `/storage/staging` route wiring with URL search validation.
- `AdminApiClient.getStorageStaging(query?: AdminStorageStagingQuery)`.
- `AdminDataSource.loadStorageStaging(query)` with section-local fallback.
- V2 filter bar for `purpose`, `state`, and `limit`.
- V2 table for record, state, source scheme, size, leases, validation, and
  expiry.
- Tests for query mapping, fallback, route rendering, and unsafe text output.
- Desktop and mobile browser smoke screenshots under
  `target/admin-web-v2-storage-staging-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- The route stays read-only and does not add cleanup, deletion, repair, or lease
  semantics.
- `/legacy` remains available.
- Follow-ons are explicit instead of hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- Admin API access remains behind `AdminDataSource`.
- Generated contract types are reused rather than duplicated.
- Raw paths, Source Locators, cache URIs, storage roots, and credentials are not
  rendered.
- No broad design-system expansion was introduced.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Staging cleanup/delete policy and route-owned mutation UX.
2. VFS cache repair diagnostics.
3. Lease management diagnostics if operators need ownership visibility.
4. Live-backend browser smoke for `/storage/staging` once a local Admin API
   server is running during frontend verification.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-storage-staging-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/storage/StorageStagingPage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
