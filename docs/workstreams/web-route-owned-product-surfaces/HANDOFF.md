# Web Route-Owned Product Surfaces - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Feature boundaries are complete. Product surfaces live under
`web/src/features/*`, and route shell code lives under `web/src/shell`.

## Active Task

- Task ID: WROP-040
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`, `npm --prefix web run check`, `npm --prefix web run test`, `npm --prefix web run build`, and `git diff --check`.

## Next Recommended Action

- Close WROP-050, then activate `web-connection-auth-tauri-profile`.
