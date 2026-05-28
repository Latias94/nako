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
runtime graph. Dependency diet is complete: unused AI/chat, chart, calendar,
carousel, drawer, form, command, OTP, resizable, toast, and unused Radix/shadcn
prototype components and packages were removed.

## Active Task

- Task ID: WBBP-050
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test && npm --prefix web run check && npm --prefix web run build && npm --prefix web run tauri -- build`

## Next Recommended Action

- Run final frontend closeout gates, document remaining non-runtime product
  gaps, close this lane, and close the six-lane frontend refactor goal.
