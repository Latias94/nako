# Web Bundle Budget And Product Pruning - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Admin live wiring is complete. The final frontend lane can now enforce bundle
budgets and prune deferred product surfaces. Bundle budget instrumentation is
available through `npm --prefix web run bundle:budget` and
`npm --prefix web run build:budget`.

## Active Task

- Task ID: WBBP-030
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test && npm --prefix web run build:budget`

## Next Recommended Action

- Remove, quarantine, or lazy-load deferred domains not accepted as live product.
