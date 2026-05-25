# Admin Web V2 Playback Sessions Route

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Playback operations still live in `/legacy`, while the V2 navigation points to a
placeholder `/playback/sessions` route. The generated Admin API contract already
has a session list response and query DTO, so this route can move as a focused
read-only slice without waiting for playback support-evidence detail UX.

## Relevant Authority

- Product docs:
  - `PRODUCT.md`
  - `DESIGN.md`
- Glossary:
  - `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/admin-web-v2-product-architecture/ROUTE_API_READINESS.md`
  - `docs/workstreams/admin-playback-session-read-model/`
  - `docs/workstreams/admin-playback-runtime-diagnostics/`

## Problem

`/playback/sessions` is currently a placeholder. The legacy dashboard shows a
small active-session summary, but it cannot own URL filters, page state, or
route-specific redaction tests.

## Target State

When this lane closes:

- `/playback/sessions` is a real V2 route.
- `AdminApiClient.getPlaybackSessions()` accepts generated
  `AdminPlaybackSessionsQuery` values.
- `AdminDataSource` exposes route-local live/mock fallback.
- URL search params own `source_id`, `kind`, `state`, `limit`, and `offset`.
- The page renders a safe read-only session table.
- Session detail and support evidence remain follow-ons.

## In Scope

- Route-owned Playback Sessions page.
- Query support in the Admin API client and data-source seam.
- Filter bar for source, kind, state, and limit.
- Tests for query mapping, fallback, route rendering, and unsafe text
  exclusions.
- Frontend validation and browser smoke evidence.

## Out Of Scope

- `/playback/sessions/:sessionId`.
- Support evidence drawer or detail page.
- Runtime diagnostics page.
- Cancelling, retrying, or mutating playback sessions.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The list DTO is enough for the first route. | High | `AdminPlaybackSessionListItem` has session id, source id, kind, state, timing, and failure category. | Split a backend/detail follow-on. |
| Query support requires only frontend client wiring. | High | `AdminPlaybackSessionsQuery` is already generated. | Defer filters or add backend work if the route rejects query params. |
| Support evidence is a separate UX. | High | Route readiness marks support as partial and detail as later. | Open a follow-on after read-only route closeout. |

## Architecture Direction

Follow the Jobs and Catalog route pattern: route search params are normalized in
`App.tsx`, Admin API calls stay inside `adminApi/client.ts`, fallback stays in
`adminApi/dataSource.ts`, and the feature module owns table columns and safe
display mapping.

## Closeout Condition

This lane can close when the route, query mapping, tests, frontend gates, and
browser smoke evidence are recorded, with detail/support-evidence follow-ons
explicitly deferred.

Closeout status: met on 2026-05-25. See `CLOSEOUT.md` and
`EVIDENCE_AND_GATES.md`.
