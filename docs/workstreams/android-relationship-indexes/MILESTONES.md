# Android Relationship Indexes - Milestones

Status: Closed
Last updated: 2026-05-20

## M1 - Index Decision

Status: Complete

Exit criteria:

- People, Tags, and Genres each have an explicit product decision.
- The first index implementation slice is selected.
- Android API integration matrix no longer marks list indexes as ambiguous
  `next` routes.

Decision:

- Genres accepted as first implementation slice.
- Tags accepted as the second reuse slice.
- Top-level People index deferred; Person Detail remains the primary People
  path.

## M2 - Genre Client Contract

Status: Complete

Exit criteria:

- `GET /genres?limit=&offset=` has typed Android client coverage.
- Focused client tests prove safe request construction and decoding.

Evidence:

- `NakoBrowseClient.listGenres` builds the paginated `GET /genres` request
  through the shared authenticated Public Client API pipeline.
- Focused tests cover decoding, bearer token redaction, and API version
  rejection.

## M3 - Genre Route And Screen

Status: Complete

Exit criteria:

- Genre Index has route state, save/restore, loading, retry, and UI.
- Rows open server-backed Genre related Media Items routes.

Evidence:

- Route state, save/restore, loading, retry, back behavior, and row targets
  are complete under ARI-030.
- Material Expressive screen and Home entry point are complete under ARI-040.
- Full Android debug unit gate passed after the screen landed.

## M4 - Evidence And Closeout

Status: Complete

Exit criteria:

- Full Android unit gate passes.
- Smoke or explicit non-smoke rationale is recorded.
- Remaining index families are complete, split, or deferred.

Evidence:

- Full Android debug unit gate passed under ARI-040.
- Focused smoke passed on 2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0`.
- Smoke surfaces include `genre-index` and `genre-index-facet`, proving Home
  -> Genres -> Mystery -> Related Media Items against the server-backed
  `Night Harbor` fixture.
- Tags Index is split to `docs/workstreams/android-tags-index/`.
- Top-level People Index remains deferred.
