# Web Bundle Budget And Product Pruning - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Admin live wiring is complete. The final frontend lane can now enforce bundle
budgets and prune deferred product surfaces. Bundle budget instrumentation is
available through `npm --prefix web run bundle:budget` and
`npm --prefix web run build:budget`. Heavy domain pruning is complete: accepted
media subviews are lazy-loaded, and the v0-only downloads, playlists, photos,
music, podcasts, AI assistant, and automation pages were removed from the live
runtime graph.

## Active Task

- Task ID: WBBP-040
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test && npm --prefix web run build:budget`

## Next Recommended Action

- Remove unused heavy dependencies and generated components that are no longer
  referenced after WBBP-030.
