# Android Relationship Indexes - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

This lane was split from APICI-060 after Android API Contract Integration
completed the Person Detail route and smoke proof. ARI-010 through ARI-050 are
complete. Android now has a server-backed Genres Index from Home, backed by
`GET /genres`, route state, a Material Expressive screen, and focused
`profile-with-media` smoke proof.

## Active Task

None. This workstream is closed.

## Decisions Since Last Update

- Person Detail belongs to the completed API contract lane.
- People, Tags, and Genres indexes are a separate product navigation lane.
- Existing related-items routes should be reused; no local filtering.
- Genres Index is the first slice and should open as a nested route from Home,
  not as a new bottom navigation destination.
- `NakoBrowseClient.listGenres` is the Android typed contract for
  `GET /genres?limit=&offset=`.
- `NakoRoute.RelationshipIndex(RelationshipIndexFamily.Genres)` and
  `RelationshipIndexUiState` are in place.
- `RelationshipIndexRouteContent` replaced the temporary placeholder and Home
  now exposes a Genres anchor into the nested route.
- `profile-with-media` smoke now captures `genre-index` and
  `genre-index-facet`, proving Home -> Genres -> Mystery -> Related Media
  Items against the server-backed `Night Harbor` fixture.
- Tags Index is accepted and split to
  `docs/workstreams/android-tags-index/`.
- Top-level People Index is deferred until the app has a richer role/search IA.

## Blockers

- None for this closed lane.

## Next Recommended Action

- Continue with `docs/workstreams/android-tags-index/` if the next product
  slice should add Tags Index using the proven relationship index shape.
