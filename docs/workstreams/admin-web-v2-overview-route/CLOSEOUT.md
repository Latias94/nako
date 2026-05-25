# Admin Web V2 Overview Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a real `/overview` route that
replaces the placeholder and becomes the default entry for `/`, backed by the
generated Admin overview read model and deterministic fallback behavior.

This closeout does not claim full dashboard parity. Richer overview cards,
workflow shortcuts, or new backend overview fields remain follow-ons.

## Delivered

- New `apps/admin-web/src/features/overview/OverviewPage.tsx`.
- `/overview` route wiring in `apps/admin-web/src/App.tsx`.
- Root route redirect from `/` to `/overview`.
- `AdminDataSource.loadOverview()` with section-local fallback.
- V2 metric cards for server status, storage readiness, runtime tasks, failed
  jobs, configured libraries, and recovered jobs.
- V2 tables for storage backend and metadata provider status.
- Tests for default redirect, route rendering, fallback behavior,
  data-source boundary, and redaction-sensitive text output.
- Desktop and mobile browser smoke screenshots for `/overview` and `/` under
  `target/admin-web-v2-overview-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `/overview` is route-owned and `/` redirects to it.
- No backend overview fields were added.
- `/legacy` remains available while remaining workflows migrate.
- Follow-ons are explicit instead of hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- Admin API access remains behind `AdminDataSource`.
- The page renders only safe `AdminOverviewResponse` summary fields.
- Raw config, roots, paths, credentials, tokens, and provider secrets are not
  rendered.
- No broad design-system expansion was introduced.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Richer Overview cards only after required backend read-model fields are
   accepted.
2. Route-owned operational shortcuts once mutation semantics are designed.
3. Live-backend browser smoke for `/overview` once a local Admin API server is
   running during frontend verification.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-overview-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/overview/OverviewPage.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
