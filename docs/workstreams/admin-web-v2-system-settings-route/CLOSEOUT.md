# Admin Web V2 System Settings Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a real `/settings` route that
replaces the placeholder with a route-first, read-only system diagnostics page
backed by redacted Admin system config and deterministic fallback behavior.

This closeout does not claim settings workflow parity. Mutations, raw config
editing, and richer configuration workflows remain follow-ons.

## Delivered

- New `apps/admin-web/src/features/settings/SettingsPage.tsx`.
- `/settings` route wiring in `apps/admin-web/src/App.tsx`.
- `AdminDataSource.loadSettings()` using `GET /admin/v1/system/config`.
- V2 summary cards for admin auth, network, database, metadata providers,
  transcode policy, and staging budget.
- V2 diagnostics panels for network readiness, database capabilities, metadata
  policy, and runtime policies.
- Tests for route rendering, fallback behavior, data-source boundary, and
  redaction-sensitive text output.
- Desktop and mobile browser smoke screenshots under
  `target/admin-web-v2-settings-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- The route stays read-only and does not add mutation semantics.
- `/legacy` remains available.
- Follow-ons are explicit instead of hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- Admin API access remains behind `AdminDataSource`.
- The page renders only safe summaries from
  `AdminServerConfigDiagnosticsResponse`.
- Env var names, URLs, host fingerprints, listen addresses, paths, roots,
  credentials, tokens, and provider secret references are not rendered.
- No broad design-system expansion was introduced.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Settings mutation workflows after route-owned mutation semantics are
   accepted.
2. Richer configuration diagnostics if new backend read-model fields are
   accepted.
3. Live-backend browser smoke for `/settings` once a local Admin API server is
   running during frontend verification.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-system-settings-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/settings/SettingsPage.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
