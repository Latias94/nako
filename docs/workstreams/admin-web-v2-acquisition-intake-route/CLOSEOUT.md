# Admin Web V2 Acquisition Intake Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a route-first, read-only
`/acquisition/intake` page backed by generated Admin API query and response
types, route-local fallback, URL-owned filters, focused tests, and browser
smoke evidence.

This closeout does not claim mutation workflow parity. Watch-folder discovery,
Managed Import promotion, and promotion apply flows remain follow-ons.

## Delivered

- New `apps/admin-web/src/features/acquisition/AcquisitionIntakePage.tsx`.
- `/acquisition/intake` route wiring and navigation entry in
  `apps/admin-web/src/App.tsx`.
- `AdminDataSource.loadAcquisitionIntake()` using
  `GET /admin/v1/acquisition/intake/candidates`.
- URL-owned filters for library, state, source kind, Managed Import artifact,
  limit, and offset.
- Tests for route rendering, search params, fallback behavior, data-source
  query mapping, and unsafe rendered text exclusions.
- Desktop and mobile browser smoke screenshots under
  `target/admin-web-v2-acquisition-intake-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- The route is read-only and does not add backend or mutation semantics.
- `/legacy` remains available.

### Code Quality

- Blocking: none.
- Important: none.
- Admin API access remains behind `AdminDataSource`.
- Rendering avoids raw source refs, source URIs, raw locators, local paths,
  tokens, and credentials.
- The page follows existing route/table/filter component patterns.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Watch-folder discovery route after mutation UX and safety policy are
   accepted.
2. Managed Import review and promotion workflows after backend mutation
   semantics are ready.
3. Live-backend smoke once a local Admin API server is attached during
   frontend verification.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-acquisition-intake-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/acquisition/AcquisitionIntakePage.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
