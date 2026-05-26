# Admin Web V2 I18n Expansion - Design

Status: Closed
Last updated: 2026-05-26

## Problem

Admin Web V2 has a dependency-free i18n boundary with English and Simplified
Chinese catalogs, but only the app shell and Media Library management routes
use it broadly. Newer routes still contain hard-coded English UI copy. This
creates an uneven operator experience and makes later translation work a broad
cleanup instead of a route-owned maintenance habit.

## Target State

- Route-owned visible UI copy is moved into `apps/admin-web/src/i18n/messages.ts`.
- English and Simplified Chinese catalogs stay key-complete.
- Routes use `useI18n()` for titles, descriptions, labels, buttons, loading
  labels, fallback copy, empty states, static badges, and confirmation text.
- API enum values, ids, provider names, Media Library names, job ids, route
  query values, timestamps, and redaction-safe diagnostic facts remain stable.
- Tests prove at least one localized rendering path for every migrated route.
- Browser smoke verifies that localized pages do not overflow on desktop or
  mobile widths.

## First Slice

The first implementation slice migrates:

- `/overview`, because it is the default Admin Web V2 entry route;
- `/access`, because it is the newest management slice and should not ship
  English-only copy after the i18n boundary exists.

## Scope

- `apps/admin-web/src/i18n/messages.ts`
- `apps/admin-web/src/features/overview/OverviewPage.tsx`
- `apps/admin-web/src/features/access/AccessPage.tsx`
- `apps/admin-web/src/App.test.tsx`
- this workstream's evidence and handoff docs

## Non-Goals

- Replacing the existing lightweight i18n boundary with an external library.
- Translating API enum values, route search params, identifiers, or diagnostic
  payload values.
- Date, number, plural-rule, or locale-specific collation infrastructure.
- Full translation of every remaining Admin Web route in the first slice.
- Public Client API, Admin API, or generated contract changes.

## Architecture Direction

Localization should stay route-owned and incremental. Each route migration must
keep its public behavior stable, route tests should exercise localized copy,
and fallback/mock/live source truth must continue to use the existing
`SourceLabel` boundary.

The implementation should not localize backend vocabulary that operators may
copy into logs, tests, or support reports, such as `ready`, `sqlite`,
`single_admin`, or job ids. Static interpretation labels, headings, help copy,
and action labels are UI copy and should be localized.
