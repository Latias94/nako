# Web Bundle Budget And Product Pruning - Handoff

Status: Complete
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

Remaining non-runtime product gaps are explicit follow-ons, not hidden frontend
surfaces: downloads, playlists, photos, music, podcasts, AI assistant, and
automation need real backend contracts, permissions, and product decisions
before they return to the app.

## Active Task

- Task ID: WBBP-050
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test`, `npm --prefix web run check`, `npm --prefix web run build:budget`, `npm --prefix web run tauri -- build`, `git diff --check`

## Next Recommended Action

- Six planned frontend refactor lanes are complete.
