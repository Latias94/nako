# Admin Web V2 Addons Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a real `/addons` route that
replaces the placeholder with a read-only Addon operations page backed by
generated Admin API Addon read models and deterministic fallback behavior.

This closeout does not claim mutation parity with the legacy console.

## Delivered

- New `apps/admin-web/src/features/addons/AddonsPage.tsx`.
- `/addons` route wiring in `apps/admin-web/src/App.tsx`.
- Generated `AdminAddonsQuery` status filter ownership in the route URL.
- `AdminDataSource.loadAddons()` with section-local fallback.
- Safe `AddonsRouteSummary` that excludes raw endpoints, paths, env var names,
  snippets, shell commands, raw manifests, diagnostic payloads, and raw token
  values.
- Read-only V2 panels for registry, selected Addon, health, surface counts,
  token prefixes, Addon Permission grants, and install ownership boundary.
- Tests for query mapping, filter updates, route rendering, fallback behavior,
  data-source fallback, and unsafe rendered-text exclusions.
- Desktop and mobile browser smoke screenshots under
  `target/admin-web-v2-addons-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `/addons` is route-owned and no longer uses the placeholder route.
- Addon Admin API calls remain behind `AdminDataSource`.
- Mutation and credential-producing workflows remain deferred.
- No backend contract changes were added.

### Code Quality

- Blocking: none.
- Important: none.
- The route uses an explicit safe read model instead of rendering the legacy
  Addon operations object.
- The install boundary text is derived from safe lifecycle booleans, not from
  raw backend text.
- The UI uses existing route shell, data panel, table, filter, badge, and
  button primitives.
- Redaction tests cover injected raw tokens, env var names, URLs, paths,
  snippets, shell commands, raw manifest payloads, and unsafe lifecycle text.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Addon registration and manifest onboarding route.
2. Addon Token issue, rotation, and revoke workflows.
3. Addon Permission grant replacement workflow.
4. Addon Health Check and resource diagnostic actions.
5. Install Guide snippet presentation or export with explicit redaction rules.
6. Live-backend visual smoke with an independently running Nako server.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-addons-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/addons/AddonsPage.tsx`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/types.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
