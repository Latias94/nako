# Admin Web V2 Product Architecture Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This product-architecture lane is complete. Admin Web V2 now has root product
and design context, a route-first information architecture, stack decisions,
a first `/jobs` implementation proof, shared shadcn-style component
extractions, V2 design tokens, validation evidence, and explicit follow-ons.

This closeout does not claim the whole Admin Web V2 migration is complete.
Most workflows still live behind `/legacy` or placeholder routes and should
move through focused follow-on lanes.

## Delivered

- Root `PRODUCT.md` and `DESIGN.md` for Nako product UI work.
- V2 research, route inventory, and API readiness under this workstream.
- TanStack Router shell with `/jobs`, placeholders, and `/legacy`.
- TanStack Query route-local Jobs server state.
- TanStack Table Jobs table with route-owned URL filters.
- `AdminApiClient.getJobs(query?: AdminJobsQuery)` using generated query DTOs.
- Section-local mock fallback and stable non-JSON Admin API error handling.
- Shared V2 components:
  - `AdminShell`
  - `RoutePage`
  - `RouteNotice`
  - `EmptyRouteState`
  - `DataPanel`
  - `FilterBar`
  - `FilterField`
  - `FilterActions`
  - `RowsSkeleton`
- V2 semantic design tokens in `apps/admin-web/src/design/tokens.css`.
- Focused route, API client, data source, component, redaction, and legacy
  availability tests.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- AWV2-010 through AWV2-050 are complete.
- The target state from `DESIGN.md` is satisfied for this lane's scope.
- Remaining Admin Web V2 work is split as follow-ons rather than hidden in
  this architecture lane.

### Code Quality

- Blocking: none.
- Important: none.
- Shared components no longer import Admin API data-source types.
- Admin API ownership remains in `adminApi` and route/feature modules.
- No direct `shadcn-admin` source was copied; the current code is local
  shadcn-style composition.
- `/legacy` remains available with a narrow route test until workflows migrate.

### Missing Gates

- None. Fresh frontend and browser evidence is recorded in
  `EVIDENCE_AND_GATES.md`.

## Follow-ons Split From This Lane

1. **Jobs workflow lane**
   - Add pagination controls, optional detail route when Admin API detail is
     generated, retry/cancel policy if mutations are accepted, and stronger
     empty/error UX.
2. **Media Libraries route lane**
   - Build the read-only Library list first, then split metadata-profile edit,
     scan/NFO actions, and runtime config semantics by API readiness.
3. **Catalog Governance lane**
   - Route-owned table and detail workflow for low-confidence or unknown Media
     Items; keep repair mutations out until backend semantics exist.
4. **Addons route lane**
   - Split addon list, manifest onboarding, runtime readiness, tokens, grants,
     install guide, and diagnostics out of `/legacy`.
5. **Settings and Network diagnostics lane**
   - Keep read-only until mutation semantics are accepted; preserve reverse
     proxy, tunnel, and lifecycle ownership language.

## Residual Risks

- `/legacy` still carries most V0 workflows. This is intentional until each
  workflow has a V2 route with tests and browser evidence.
- Several routes are placeholders and should not be treated as finished
  product experiences.
- Some Admin API gaps remain: job detail, event/playback/storage query
  support, catalog detail/repair, and settings mutations.
- The V2 CSS tokens coexist with legacy CSS until the legacy route is removed.

## Evidence Anchors

- `PRODUCT.md`
- `DESIGN.md`
- `docs/workstreams/admin-web-v2-product-architecture/EVIDENCE_AND_GATES.md`
- `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/jobs/JobsPage.tsx`
- `apps/admin-web/src/components/layout/`
- `apps/admin-web/src/components/ui/`
- `apps/admin-web/src/design/tokens.css`
