# Admin Web V2 System Settings Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 still leaves `/settings` as a placeholder. The legacy dashboard has
read-only Settings and Network panels backed by redacted Admin system config
diagnostics, so this is the next bounded route-first migration after Overview.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
  - `docs/workstreams/admin-web-v2-media-libraries-route/`

## Problem

Operators need a V2 page that summarizes safe system readiness without sending
them back to the single-page legacy console. The route must not expose raw
config, env var names, URLs, credentials, filesystem paths, storage roots, or
provider secret references.

## Target State

When this lane closes:

- `/settings` is a real route-first V2 page.
- `AdminDataSource` exposes route-local live/mock fallback for
  `AdminServerConfigDiagnosticsResponse`.
- The page renders safe diagnostics for admin auth, network readiness,
  database/runtime capabilities, metadata policy, transcode policy, staging
  policy, playback policy, and artwork policy.
- The page remains read-only and explicitly defers mutation semantics.
- The route has tests, browser evidence, and redaction checks.

## In Scope

- Route-owned System Settings page.
- Data-source seam for route-local system config loading.
- Safe summary cards and diagnostic rows derived from generated Admin system
  config types.
- Tests for rendering, fallback, route data boundary, and unsafe text
  exclusions.
- Frontend validation and browser smoke evidence.

## Out Of Scope

- Settings mutations.
- Rendering raw config or config file text.
- Rendering env var names, URLs, paths, storage roots, credentials, tokens, or
  provider secrets.
- Backend contract changes.
- Removing `/legacy`.

## Architecture Direction

Follow the existing route pattern: `App.tsx` owns route wiring,
`adminApi/dataSource.ts` owns section fallback, and `features/settings` owns
safe display-only summarization of `AdminServerConfigDiagnosticsResponse`.

## Closeout Condition

This lane can close when `/settings` is implemented as a read-only route,
validation and browser smoke evidence are recorded, and mutation/richer
configuration workflows are deferred.

Closeout status: met on 2026-05-25. See `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.
