# Admin Web V2 Acquisition Intake Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

The legacy dashboard still owns the Acquisition Intake panel. This is a real
operator workflow, not a placeholder: it shows watch-folder candidates before
Managed Import and promotion apply. The generated Admin API already exposes
`GET /admin/v1/acquisition/intake/candidates`, so the workflow can move to a
route-first V2 page without backend changes.

## Relevant Authority

- `PRODUCT.md`
- `DESIGN.md`
- `CONTEXT.md`
- `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`

## Problem

Operators cannot inspect Acquisition Intake candidates from the V2 navigation
without returning to the single-page legacy console. The legacy panel also has
no route-owned filters, shareable URL state, route-local fallback, or focused
redaction tests.

## Target State

When this lane closes:

- `/acquisition/intake` is a real route-first V2 page.
- The route owns safe search params for `library_id`, `state`, `source_kind`,
  `managed_import_artifact_id`, `limit`, and `offset`.
- `AdminDataSource` exposes route-local live/mock fallback for
  `AdminAcquisitionIntakeCandidateListResponse`.
- The page renders candidate readiness, source type, redacted source facts,
  size, diagnostics availability, Managed Import linkage, and timestamps
  without raw locators or filesystem paths.
- The route has focused tests, redaction checks, frontend gates, and browser
  smoke evidence.

## In Scope

- Route wiring and navigation entry for `/acquisition/intake`.
- Route-owned query normalization and generated query DTO mapping.
- Read-only V2 table/panel UI for intake candidates.
- Route-local `AdminDataSource.loadAcquisitionIntake`.
- Tests for rendering, fallback, search params, and unsafe text exclusions.
- Workstream evidence and closeout updates.

## Out Of Scope

- Watch-folder discovery mutation UI.
- Managed Import promotion or apply mutations.
- Backend/Admin API contract changes.
- Raw source references, local paths, credentials, or unredacted locator data.
- Removing `/legacy`.

## Architecture Direction

Follow the existing route pattern used by Jobs, Catalog Governance, Playback
Sessions, Storage Staging, and Addons: `App.tsx` owns route wiring and URL
normalization, `adminApi/dataSource.ts` owns live/mock fallback, and
`features/acquisition` owns display-only workflow UI.

## Closeout Condition

The lane can close when `/acquisition/intake` has route-first behavior,
validation evidence, browser smoke evidence, and explicit follow-ons for any
future mutation workflows.

Status: satisfied on 2026-05-25. See `CLOSEOUT.md`.
