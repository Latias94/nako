# admin-web Frontend Development Guidelines

`apps/admin-web` is the current validation-oriented Admin Web app. It is not
the release product frontend, but it is still used for Admin API contract,
redaction, route/query, and selected mutation coverage.

## Pre-Development Checklist

- Read [Routes, Forms, Data, and Tests](./routes-forms-and-tests.md) before
  changing `apps/admin-web/src/**`.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Routes, Forms, Data, and Tests](./routes-forms-and-tests.md) | TanStack Router/Query route ownership, native forms, generated Admin API client, Vitest tests | Filled from code and README |

## Authority / Evidence

- ADR 0027: Admin Web uses a separate versioned Admin API boundary.
- ADR 0053: Admin diagnostics must stay redacted and bounded.
- `apps/admin-web/README.md`
- `apps/admin-web/package.json`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/jobs/JobsPage.tsx`
- `apps/admin-web/src/features/settings/SettingsPage.tsx`
- `apps/admin-web/src/surfaces/media/MediaPages.tsx`
- `apps/admin-web/src/surfaces/media/mediaBrowsePlanner.ts`
- `apps/admin-web/src/surfaces/media/mediaBrowsePlanner.test.ts`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
