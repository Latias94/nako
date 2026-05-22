# Android Tags Index

Status: Closed
Last updated: 2026-05-20

## Why This Lane Exists

`docs/workstreams/android-relationship-indexes/` proved the reusable
relationship index shape with Genres. Tags are the next accepted relationship
index family, but they deserve a small follow-on lane so the closed Genres work
does not become a mixed bag of future index families.

## Problem

Android can open Tag related Media Items from item detail chips, but users
cannot browse server-backed Tag labels directly from Home. The app should reuse
the proven relationship index route shape instead of adding a separate screen
architecture or locally filtering cached Media Items.

## Target State

- Android has a typed `GET /tags?limit=&offset=` client contract.
- `RelationshipIndexFamily` supports Tags without duplicating the Genres
  route stack.
- Home exposes a Tags anchor only after the client and route state are ready.
- Tag rows open the existing Tag related Media Items route through stable
  server IDs.
- Focused unit gates prove the contract and route behavior.
- Smoke is extended only if the fixture makes the Tags path valuable and
  stable.

## In Scope

- `GET /tags?limit=&offset=` Android DTO and client method.
- Relationship index route state reuse for Tags.
- Material Expressive Tags Index UI using the existing relationship index
  screen family.
- Home nested route entry point.
- Focused tests and docs updates.

## Out Of Scope

- Top-level People Index.
- New server API shape.
- Local tag filtering from cached item lists.
- Advanced tag sorting, clustering, moderation, or multi-select filters.
- New bottom navigation destinations.

## Architecture Direction

Reuse the Genres Index architecture. `NakoBrowseClient` owns the typed Public
Client API call, `ClientBrowseDataSource` maps list rows to stable
`BrowseFacetTarget` values, and `BrowseSession` remains the owner of route
state. The UI should generalize existing relationship index copy/icons where
needed without creating a parallel Tags-only screen.

## Closeout Condition

This lane can close when Tags Index is productized through typed client
contract, route state, Home entry, screen reuse, focused unit evidence, and
either smoke evidence or an explicit rationale for not extending smoke.

This lane closed on 2026-05-20. The closeout condition was met with typed
`GET /tags?limit=&offset=` client coverage, shared relationship index route
state, a Home Tags anchor, family-aware relationship index UI, focused unit
tests, full Android debug unit coverage, and `profile-with-media` smoke
evidence for Home -> Tags -> Lighthouse -> Related Media Items.
