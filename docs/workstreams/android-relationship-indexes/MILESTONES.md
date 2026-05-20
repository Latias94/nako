# Android Relationship Indexes - Milestones

Status: Active
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

Status: Pending

Exit criteria:

- `GET /genres?limit=&offset=` has typed Android client coverage.
- Focused client tests prove safe request construction and decoding.

## M3 - Genre Route And Screen

Status: Pending

Exit criteria:

- Genre Index has route state, save/restore, loading, retry, and UI.
- Rows open server-backed Genre related Media Items routes.

## M4 - Evidence And Closeout

Status: Pending

Exit criteria:

- Full Android unit gate passes.
- Smoke or explicit non-smoke rationale is recorded.
- Remaining index families are complete, split, or deferred.
