# Web Test Harness And Route Contracts - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Vite/TanStack/Tauri runtime migration is complete. The next frontend refactors
need real tests before moving feature boundaries or deleting fixture-only UI.

## Active Task

- Task ID: WTRC-020
- Owner: Codex
- Files: `web/package.json`, `web/vitest.config.ts`, `web/src/test`
- Validation: `npm --prefix web run test`
- Status: READY
- Review: Keep mocks central and explicit.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Next Recommended Action

- Implement the Vitest harness.
