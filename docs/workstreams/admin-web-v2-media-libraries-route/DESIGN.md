# Admin Web V2 Media Libraries Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 has a route-first shell and a proven `/jobs` slice, but Media
Libraries still live in `/legacy` as a mostly mock table. Operators need a V2
route that shows configured Media Libraries from the Admin API boundary without
leaking storage roots, credentials, or unfinished mutation semantics.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
  - `docs/workstreams/admin-web-v2-product-architecture/CLOSEOUT.md`
  - `docs/workstreams/admin-library-metadata-profile-configuration/`

## Problem

The `/libraries` V2 route is currently a placeholder, while `/legacy` shows a
Media Libraries panel backed by deterministic mock rows. The generated Admin API
contract does not yet expose a dedicated Admin library list route, but
`GET /admin/v1/system/config` already carries redacted library configuration
diagnostics suitable for a read-only V2 route.

## Target State

When this lane closes:

- `/libraries` is a real route-first page, not a placeholder.
- The page reads Media Library configuration diagnostics through the Admin API
  data-source boundary.
- Fallback data remains deterministic and section-local when the Admin API is
  unavailable.
- Rendered output contains no raw storage roots, filesystem paths, WebDAV
  passwords, tokens, or secret values.
- The route uses the existing V2 shell, shadcn-style primitives, and shared data
  panel/table components already proven by `/jobs`.
- Metadata profile editing, scans, and NFO actions are either split into
  follow-ons or explicitly deferred.

## In Scope

- Read-only `/libraries` route using `AdminServerConfigDiagnosticsResponse`.
- A route-local Media Libraries feature module under `apps/admin-web/src/features`.
- Data-source method for route-local library diagnostics with live/mock fallback.
- Tests for route availability, data-source fallback, and redaction-sensitive
  rendering.
- Fresh frontend validation and browser smoke evidence.

## Out Of Scope

- Creating, editing, or deleting Media Libraries.
- Running scans or NFO import/export from V2.
- Editing `AdminMetadataProfile` from this lane.
- Adding a new backend Admin library-list contract unless the existing config
  diagnostics prove insufficient.
- Removing the `/legacy` Media Libraries panel before equivalent workflows exist.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `GET /admin/v1/system/config` is safe for a read-only Media Libraries list. | High | Generated `AdminServerConfigDiagnosticsResponse.libraries` contains IDs, names, preset, backend kind, root scheme, and credential-presence booleans. | Add a dedicated Admin library diagnostics route before shipping the page. |
| The V2 route should not show public API library item counts yet. | Medium | Current route readiness notes list `/libraries` as partial and legacy rows are mock. | Split a follow-on to reconcile public library list data with Admin-only diagnostics. |
| Metadata profile editing is a separate workflow. | High | Existing admin metadata-profile contract has its own workstream and mutation semantics. | Extend this lane only after the read-only route is validated. |
| The existing V2 shared components are sufficient. | High | `/jobs` extracted shell, route header, filters, data panel, table, badges, and skeletons. | Add only narrowly proven shared pieces, not a broader design system. |

## Architecture Direction

`apps/admin-web/src/adminApi` owns Admin API calls and live/mock fallback. The
new route should depend on `AdminDataSource`, not on `AdminApiClient` directly.
The feature module owns display mapping and route UI. It can use generated
contract types for the response shape, but it must keep unsafe fields out of
rendered text and tests.

The first implementation should use `AdminServerConfigDiagnosticsResponse` as
the route data model because it is already generated, Admin-scoped, and
redaction-aware. Public library routes can be reconciled later when item counts
or Media Source inventory become route requirements.

## Closeout Condition

This lane can close when:

- `/libraries` is implemented and tested as a V2 route;
- fallback, empty, loading, and redaction behavior are covered;
- `npm run check`, `npm run test`, `npm run build`, `git diff --check`, and a
  browser smoke pass are recorded;
- remaining metadata-profile, scan, NFO, and library inventory work is split or
  explicitly deferred.

Closeout status: met on 2026-05-25. See `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.
