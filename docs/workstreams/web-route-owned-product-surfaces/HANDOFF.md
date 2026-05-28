# Web Route-Owned Product Surfaces - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Feature boundaries are complete. Product surfaces live under
`web/src/features/*`, and route shell code lives under `web/src/shell`.

## Active Task

- Task ID: WROP-030
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx`, `npm --prefix web run check`, `npm --prefix web run test`, and `npm --prefix web run build`.

## Next Recommended Action

- Start WROP-040 by moving durable search/filter/page state into TanStack route search params where the behavior is already product-level state.
