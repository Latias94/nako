# Public Client Library Browse Query Contract - Design

Status: Completed
Last updated: 2026-05-29

## Problem

WMLP made `/media/library` truthful: it shows library metadata/source readiness
but does not fake scoped live items. The missing contract is Public Client
library-scoped item browse. A related gap blocks accurate home rails and view
filters: `listItems` does not expose stable sort/filter keys for Recently
Added, watched/unwatched, or library-scoped browse.

## Target State

When this lane closes:

- Public Client API exposes a stable library-scoped item browse contract.
- Public Client API exposes explicit sort and filter keys rather than raw DB
  columns or ad hoc query strings.
- Effective Library Access is enforced before returning item rows.
- TypeScript/Rust SDKs expose the query shape.
- `web/` can render `/media/library` scoped live items and home rails without
  fixture-only claims.

## Scope

In scope:

- Public route/query DTO design for library item browse.
- Stable browse sort/filter keys needed by current video-first web surfaces.
- Server repository/app/API support and SDK generation.
- Web data-source and route integration for library pages and rails.

Out of scope:

- Non-video domain browse contracts.
- Arbitrary SQL-like query language.
- Admin catalog governance filters.
- Recommendation ranking.
- Playlist membership.

## Architecture Direction

Start with explicit video-first browse contracts. Prefer a route such as
`GET /libraries/{library_id}/items` or an explicitly accepted `library_id`
filter on `GET /items`; do not support both unless the contract freeze justifies
it. Sort/filter keys should be named enums in the public protocol, with server
behavior backed by tests.

## Closeout Summary

PLBQ closes with `GET /libraries/{library_id}/items` as the accepted route. The
server/API/SDK path is implemented, effective Library Access hides inaccessible
libraries, `kind:` facets and watch-state filters are covered, and `web/` uses
scoped live browse for supported `/media/library` states.

The broader browse vocabulary remains intentionally split: additional facet
prefixes, richer pagination UX, and home rail read models should enter through
new bounded lanes instead of extending this contract lane.
