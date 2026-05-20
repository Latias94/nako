# Android Tags Index - Milestones

Status: Active
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

Status: Active

Exit criteria:

- `RelationshipIndexFamily.Tags` reuses the existing relationship index route
  state model.
- Rows open the existing Tag related Media Items route with stable server IDs.

## M3 - Screen And Home Entry

Status: Pending

Exit criteria:

- Tags Index uses the shared Material Expressive relationship index screen
  shape.
- Home exposes Tags as a nested route entry point.
- full Android debug unit gate passes.

## M4 - Evidence And Closeout

Status: Pending

Exit criteria:

- Smoke or explicit non-smoke rationale is recorded.
- Workstream docs reflect the shipped behavior and residual scope.
