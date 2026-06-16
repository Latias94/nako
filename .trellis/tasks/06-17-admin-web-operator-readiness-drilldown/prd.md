# feat: Admin Web operator readiness drilldown

## Goal

Turn the new backend `GET /admin/v1/operator-readiness` read model into an
operator-facing Admin Web page so a self-hosting administrator can move from
overview status to concrete, redaction-safe reasons and next actions.

## Requirements

- Add a read-only Admin Web route at `/operator-readiness`.
- Add a typed `AdminApiClient.getOperatorReadiness()` method using
  `NAKO_ADMIN_ROUTES.operatorReadiness`.
- Add `AdminDataSource.loadOperatorReadiness()` with deterministic mock
  fallback data.
- Render the operator readiness summary plus all detail sections:
  setup, media library scan, playback, durable jobs, storage, network, and
  backup.
- Keep the page dense and operational, matching existing Admin Web layout,
  panels, badges, notices, refresh buttons, and i18n patterns.
- Link Overview's readiness panel to the new drilldown route without changing
  backend contracts.
- Do not render raw paths, source locators, tokens, token env names, backend
  URLs, etags, fingerprints, raw job payload names, or raw errors.
- Provide English and zh-Hans copy for new route/nav/page text.

## Acceptance Criteria

- [x] `/operator-readiness` renders with live data source and mock fallback.
- [x] The Admin API client uses the generated `operatorReadiness` route key.
- [x] The data source returns live result when available and mock fallback on
      failure.
- [x] The page renders all seven readiness areas with status, reason, safe
      facts, and action route hints where present.
- [x] Route tests prove zh-Hans copy appears.
- [x] Route tests prove unsafe field families are not rendered.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Focused Admin Web tests for client, data source, and route pass.

## Verification

- `python .\.trellis\scripts\task.py validate 06-17-admin-web-operator-readiness-drilldown`
- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web -- src/adminApi/client.test.ts src/adminApi/dataSource.test.ts src/adminApi/lazyDataSource.test.ts src/App.test.tsx`
- `npm run test --prefix apps/admin-web`
- `npm run build --prefix apps/admin-web`

## Definition of Done

- Tests added or updated for client, data source, route rendering, fallback,
  i18n, and redaction.
- Generated Admin contract remains source-of-truth; no hand-written route
  literal replaces generated route keys.
- Trellis task context is valid.
- Work is committed separately from unrelated CRLF-only file changes.

## Technical Approach

- Use a new lazy route module and page component:
  `routes/OperatorReadinessRouteModule.tsx` and
  `features/overview/OperatorReadinessPage.tsx`.
- Reuse the existing `overview` namespace for readiness labels where useful,
  and add a dedicated `operatorReadiness` namespace for page-specific copy.
- Keep data loading in React Query through `AdminDataSource`, not direct fetch.
- Reuse existing `Badge`, `Button`, `DataPanel`, `RoutePage`, `RouteNotice`,
  `RowsSkeleton`, and existing overview readiness label concepts.
- Keep the first implementation read-only. Action hints are route labels/paths,
  not mutations.

## Decision (ADR-lite)

**Context**: The backend now exposes a rich drilldown contract, while Overview
only shows compact checks.

**Decision**: Add a dedicated Admin Web route instead of expanding Overview
into a large mixed dashboard. Overview remains the compact landing page and
links to drilldown.

**Consequences**: This keeps the operational page scan-friendly, makes test
coverage focused, and leaves room for future per-area remediation flows without
turning Overview into a mutation surface.

## Out of Scope

- No new backend endpoints.
- No scan/repair/playback/network mutation controls.
- No browser visual redesign or new component system.
- No public Media Web changes.

## Technical Notes

- Existing Admin Web spec:
  `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.
- Backend contract spec:
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`.
- Existing patterns:
  `OverviewPage`, `IncidentBundlePage`, `AdminApiClient`, `AdminDataSource`,
  `mockData`, `RouteI18n`, and `App.test.tsx`.
