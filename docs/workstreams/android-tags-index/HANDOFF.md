# Android Tags Index - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

This lane is the follow-on from the closed
`docs/workstreams/android-relationship-indexes/` lane. Genres Index proved the
relationship index shape end to end. Tags Index should reuse that shape as the
second accepted relationship index family. ATI-010 is complete:
`TagListResponse` and `TaruBrowseClient.listTags` cover
`GET /tags?limit=&offset=`. ATI-020 is complete: Tags now reuse the
relationship index route state, data-source mapping, and safe navigation
restore path. ATI-030 is complete: Home exposes Tags as a nested relationship
index route, `TaruBrowseShell` dispatches it through the shared Tags
relationship index action, and `RelationshipIndexRoute` keeps one screen shape
with family-aware copy and icons. ATI-040 is complete: `profile-with-media`
smoke now proves Home -> Tags -> Lighthouse -> Related Media Items and captures
`tag-index` plus `tag-index-facet` evidence. This lane is closed.

## Active Task

None. This workstream is closed.

## Decisions

- Tags Index is accepted as a reuse slice after Genres.
- Tags Index should open as a nested Home route, not a new bottom navigation
  destination.
- Tag rows should open existing Tag related Media Items routes with stable
  server IDs.
- Top-level People Index remains outside this lane.
- `TaruBrowseClient.listTags` is the Android typed contract for
  `GET /tags?limit=&offset=`.
- `RelationshipIndexFamily.Tags` is in place and maps to existing Tag related
  Media Items targets.
- Relationship index presentation copy is family-aware for Genres and Tags.
- Tags now has a Home anchor and relationship index icon parity with Genres.
- `profile-with-media` smoke captures Home -> Tags -> Lighthouse -> Related
  Media Items with zero retries.

## Blockers

- None for this closed lane.

## Next Recommended Action

- Open a new workstream only for a different boundary: CI/device-farm smoke
  execution, golden screenshot diffing, or richer Tags IA such as sorting,
  clustering, and multi-select filters.
