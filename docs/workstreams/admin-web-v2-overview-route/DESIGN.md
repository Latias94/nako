# Admin Web V2 Overview Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 currently redirects `/` to `/jobs`, because Jobs was the first
route-first proof. With several operational routes now migrated, the default
entry should become `/overview`, backed by the generated Admin overview read
model.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`

## Problem

`/overview` is still a placeholder. The legacy dashboard has an overview band,
but the V2 shell does not yet have a route-owned summary of health, storage,
metadata, runtime, and startup facts.

## Target State

When this lane closes:

- `/overview` is a real V2 route.
- `/` redirects to `/overview`.
- `AdminDataSource` exposes route-local live/mock fallback for
  `AdminOverviewResponse`.
- The page renders safe summary metrics plus storage backend and metadata
  provider status.
- The route has tests, browser evidence, and redaction checks.

## In Scope

- Route-owned Overview page.
- Root redirect change from `/jobs` to `/overview`.
- Data-source seam for route-local overview loading.
- Tests for default redirect, fallback, route rendering, and unsafe text
  exclusions.
- Frontend validation and browser smoke evidence.

## Out Of Scope

- Overview mutations.
- Replacing all remaining legacy dashboard sections.
- Adding new backend overview fields.
- Rendering raw config, roots, paths, credentials, tokens, or provider secrets.

## Architecture Direction

Follow the route-first pattern already used by Jobs, Libraries, Catalog,
Playback, and Storage. Keep API access in `adminApi`, fallback in the
data-source seam, and display-only summarization in `features/overview`.

## Closeout Condition

This lane can close when `/overview` is implemented as the default route,
validation and browser smoke evidence are recorded, and remaining overview
expansion work is deferred.

Closeout status: met on 2026-05-25. See `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.
