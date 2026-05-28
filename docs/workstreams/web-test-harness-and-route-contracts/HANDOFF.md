# Web Test Harness And Route Contracts - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Vite/TanStack/Tauri runtime migration is complete. The next frontend refactors
need real tests before moving feature boundaries or deleting fixture-only UI.

## Active Task

- Task ID: WTRC-030
- Owner: Codex
- Files: `web/src`, `web/components/nako/nako-router.tsx`
- Validation: `npm --prefix web run test`
- Status: READY
- Review: Route tests should avoid brittle full-page snapshots.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Next Recommended Action

- Add route rendering contract tests for shipped top-level routes.
