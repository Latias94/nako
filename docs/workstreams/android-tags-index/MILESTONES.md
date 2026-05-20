# Android Tags Index - Milestones

Status: Closed
Last updated: 2026-05-20

## M1 - Client Contract

Status: Complete

Exit criteria:

- `GET /tags?limit=&offset=` has typed Android client coverage.
- API coverage matrices point at this lane for the Tags Index work.

Evidence:

- `TagListResponse` and `TaruBrowseClient.listTags` are implemented.
- Focused `TaruBrowseClientTest` coverage proves request construction,
  decoding, bearer auth redaction, safe diagnostics, and unsupported API
  version rejection.
- API matrices now show `GET /tags?limit=&offset=` as connected through the
  typed client contract, with route state remaining in this lane.

## M2 - Route Reuse

Status: Complete

Exit criteria:

- `RelationshipIndexFamily.Tags` reuses the existing relationship index route
  state model.
- Rows open the existing Tag related Media Items route with stable server IDs.

Evidence:

- `RelationshipIndexFamily.Tags` and
  `TagListResponse.toRelationshipIndexContent` map server-backed Tag rows to
  existing Tag related Media Items targets.
- `BrowseSession` opens, loads, retries through the existing route state, and
  opens Tag rows as `TaruRoute.BrowseFacet`.
- `TaruBrowseNavigationStateSaver` restores Tags Index as a safe nested route.
- `ClientBrowseDataSource.loadRelationshipIndex(Tags)` calls
  `TaruBrowseClient.listTags(limit=50, offset=0)`.
- Relationship index presentation copy is family-aware for Genres and Tags.

## M3 - Screen And Home Entry

Status: Complete

Exit criteria:

- Tags Index uses the shared Material Expressive relationship index screen
  shape.
- Home exposes Tags as a nested route entry point.
- full Android debug unit gate passes.

Evidence:

- Home exposes a Tags anchor next to Genres without adding a bottom navigation
  destination.
- `TaruBrowseShell` dispatches the Tags anchor through the shared
  `BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Tags)` path.
- `RelationshipIndexRoute` uses family-aware copy and icons while keeping the
  existing relationship index screen shape.
- Focused host and presentation tests passed, followed by the full Android
  debug unit gate.

## M4 - Evidence And Closeout

Status: Complete

Exit criteria:

- Smoke or explicit non-smoke rationale is recorded.
- Workstream docs reflect the shipped behavior and residual scope.

Evidence:

- `profile-with-media` smoke now captures `tag-index` and `tag-index-facet`,
  proving Home -> Tags -> Lighthouse -> Related Media Items against the
  server-backed `Night Harbor` fixture.
- Closeout docs are updated and `WORKSTREAM.json` is closed.
