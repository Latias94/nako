# Code Context

## Existing Patterns

- Routes are declared in `apps/admin-web/src/App.tsx`, then rendered through
  lazy `routes/*RouteModule.tsx` files.
- Route modules wrap pages with `RouteI18n` and pass broad `AdminDataSource`
  into feature pages.
- Read-only Admin pages use React Query, `SourceLabel`, `RoutePage`,
  `RouteNotice`, `DataPanel`, `RowsSkeleton`, and mock fallback.
- `OverviewPage` already has readiness area, status, reason, and action label
  mappings that can be reused or mirrored.
- `IncidentBundlePage` shows how to render dense redaction-safe diagnostics and
  reject unsafe material in tests.

## Affected Files

- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
- `apps/admin-web/src/adminApi/lazyDataSource.ts`
- `apps/admin-web/src/adminApi/mockData.ts`
- `apps/admin-web/src/adminApi/types.ts`
- `apps/admin-web/src/features/overview/OverviewPage.tsx`
- `apps/admin-web/src/features/overview/OperatorReadinessPage.tsx`
- `apps/admin-web/src/routes/OperatorReadinessRouteModule.tsx`
- `apps/admin-web/src/i18n/catalogs/base.ts`
- `apps/admin-web/src/i18n/catalogs/operatorReadiness.ts`
- `apps/admin-web/src/i18n/catalogLoader.ts`
- `apps/admin-web/src/i18n/messages.ts`
- `apps/admin-web/src/App.test.tsx`

## Constraints

- Use `NAKO_ADMIN_ROUTES.operatorReadiness` rather than a literal route string
  in client code.
- Keep the route read-only and redaction-safe.
- Do not render raw unsafe strings even if fixture data is polluted.
- Follow existing route-level bundle splitting: new page should be loaded
  through a lazy route module and dedicated catalog namespace.
