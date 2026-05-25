# Admin Web V2 Addons Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 still leaves `/addons` as a placeholder. Addon operations are the
last top-level workflow still anchored in the legacy console after Overview,
Jobs, Libraries, Catalog, Playback, Storage, and Settings were migrated.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/addon-ecosystem-foundation/`
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`

## Problem

Operators need a V2 Addons page that can inspect registered Addons, lifecycle
state, health, grants, tokens, surfaces, and install boundary without returning
to the single-page legacy dashboard. Mutation flows are sensitive because they
create credentials, change grants, and affect sidecar lifecycle, so this lane
keeps them out of scope.

## Target State

When this lane closes:

- `/addons` is a real route-first V2 page.
- `AdminDataSource` exposes route-local live/mock fallback for safe Addon
  operations summaries.
- URL search owns the Addon status filter.
- The page renders safe Addon registration, health, grants, token prefixes,
  install boundary, and surface counts.
- Raw one-time tokens, credential material, manifest JSON, install snippets,
  secret env var names, URLs, paths, payloads, and provider secrets are not
  rendered.
- Mutations remain explicit follow-ons.

## In Scope

- Route-owned Addons page.
- Generated `AdminAddonsQuery` status filter.
- Data-source seam for route-local Addon summary loading.
- Safe read-only registration, health, grant, token, install boundary, and
  surface rendering.
- Tests for query mapping, fallback, route rendering, and unsafe text
  exclusions.
- Frontend validation and browser smoke evidence.

## Out Of Scope

- Addon registration, unregister, enable/disable, health-check action,
  diagnostic action, token issue/rotate/revoke, and grant replacement UI.
- Rendering raw manifest JSON, install snippets, raw tokens, request payloads,
  credential values, env var names, URLs, paths, or hosted page URLs.
- Backend contract changes.
- Removing `/legacy`.

## Architecture Direction

Follow the existing route pattern: `App.tsx` owns URL search normalization,
`adminApi/dataSource.ts` owns section fallback and safe mapping,
`features/addons` owns display-only rendering.

## Closeout Condition

This lane can close when `/addons` is implemented as a read-only route,
validation and browser smoke evidence are recorded, and mutation workflows are
deferred.

## Closeout

Closed on 2026-05-25. `/addons` is now route-owned, read-only, backed by a
safe Addon route summary, and validated through frontend gates plus desktop and
mobile browser smoke. Addon registration, token issue/rotation/revoke, grant
replacement, health-check actions, diagnostics, and install-guide snippets
remain explicit follow-ons.
