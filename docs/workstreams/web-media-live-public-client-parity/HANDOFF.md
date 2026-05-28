# Web Media Live Public Client Parity - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

The new `web/` Media surface has a first live data-source seam for
`listItems`, `searchItems`, and `getItem`, but browse/detail still need truthful
route-owned live read models and playback entry is still mock-only. The
generated TypeScript SDK exposes playback decision, browser ticket, playback
session, heartbeat, stream, continue-watching, progress, and watched-state
methods, but this lane must verify those contracts before wiring UI.

## Active Task

- Task ID: WMLP-020
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`

## Next Recommended Action

- Run WMLP-020: audit generated Public Client route/SDK readiness and update
  `ROUTE_API_READINESS.md` before making code changes.

