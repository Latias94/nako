# Admin Web i18n Route Catalog Chunk Splitting

## Goal

Reduce Admin Web route-agnostic i18n download cost by splitting the current
single messages catalog into route or feature namespaces that can be loaded on
demand, while preserving typed message IDs and stable locale switching.

## What I already know

- `apps/admin-web` is the validation-oriented Admin Web app, and bundle size is
  being tightened after route-level splitting work.
- The latest build still emitted a large `messages-*.js` chunk around 200 kB.
- Admin Web uses an in-app `I18nProvider` and `messages.ts` catalog.
- The previous media route split already moved browse/detail/watch pages into
  separate lazy route modules.

## Requirements

- Split the monolithic i18n catalog into a small base catalog plus feature or
  route catalogs.
- Load only the catalogs needed by the active route surface, rather than
  downloading all route text for every Admin Web entry.
- Preserve `AdminLocale`, `MessageId`, and translator type safety as much as the
  current architecture allows.
- Keep existing Admin Web route behavior unchanged.
- Locale switching must not render stale translated content after async catalog
  loading.
- Keep the change scoped to Admin Web frontend code and associated Trellis
  documentation.

## Acceptance Criteria

- [x] `messages` is no longer emitted as a single route-agnostic chunk near the
      previous 200 kB size.
- [x] Admin Web routes can render translated copy for their active namespace
      after navigation and after locale changes.
- [x] Existing Admin Web tests pass.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] `npm run build --prefix apps/admin-web` passes and shows smaller,
      separated i18n-related chunks.
- [x] Any new route/i18n loading convention is recorded in the Admin Web
      frontend Trellis spec if it becomes a reusable pattern.

## Definition of Done

- Typecheck, tests, build, and `git diff --check` are green.
- The Trellis task is archived or ready to archive.
- A Conventional Commit records the implementation.

## Technical Approach

Inspect the current `I18nProvider`, `messages.ts`, and route lazy imports first.
Prefer a conservative namespace loader that keeps the translator API stable for
call sites while changing the underlying catalog delivery model. Use route-level
metadata or wrappers only where it avoids broad page churn.

## Out of Scope

- Introducing an external i18n framework.
- Translating new product copy beyond what is needed to preserve existing text.
- Reworking Admin Web routing or Admin API clients.
- Backend changes.

## Technical Notes

- Relevant spec entrypoints:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/index.md`
- Relevant code entrypoints:
  - `apps/admin-web/src/i18n/I18nProvider.tsx`
  - `apps/admin-web/src/i18n/messages.ts`
  - `apps/admin-web/src/App.tsx`
  - `apps/admin-web/src/App.test.tsx`
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
