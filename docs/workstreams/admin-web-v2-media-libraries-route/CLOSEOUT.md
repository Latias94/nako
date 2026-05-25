# Admin Web V2 Media Libraries Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a real `/libraries` route that
replaces the placeholder with a route-first, read-only Media Libraries page
backed by Admin system config diagnostics and deterministic fallback behavior.

This closeout does not claim full Media Libraries workflow parity. Metadata
profile editing, scan/NFO actions, public inventory, and richer item/source
counts remain follow-ons.

## Delivered

- New `apps/admin-web/src/features/libraries/LibrariesPage.tsx`.
- `AdminDataSource.loadLibraries()` using `GET /admin/v1/system/config` through
  `AdminApiClient.getSystemConfig()`.
- `/libraries` route wiring in `apps/admin-web/src/App.tsx`.
- V2 table rendering for Media Library name, preset, backend, root scheme,
  Secret Reference state, and runtime policy.
- Deterministic mock fallback when the Admin API is unavailable.
- Tests for route rendering, fallback behavior, data-source boundary, and
  redaction-sensitive text output.
- Desktop and mobile browser smoke screenshots under
  `target/admin-web-v2-libraries-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- The route stays within the read-only scope and does not add mutation
  semantics.
- `/legacy` remains available.
- Follow-ons are explicit instead of hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- Admin API access remains behind `AdminDataSource`.
- The page renders only a safe projection of
  `AdminServerConfigDiagnosticsResponse.libraries`.
- No broad design-system expansion was introduced.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Metadata profile route/editing using the existing
   `AdminLibraryMetadataProfileResponse` and update request contract.
2. Library scan and NFO action policy once route-owned mutation semantics are
   accepted.
3. Dedicated Admin library inventory or public library list reconciliation if
   item/source counts become required in V2.
4. Live-backend browser smoke for `/libraries` once a local Admin API server is
   running during frontend verification.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-media-libraries-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/libraries/LibrariesPage.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
