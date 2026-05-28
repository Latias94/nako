# Web Test Harness And Route Contracts - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

Vite/TanStack/Tauri runtime migration is complete. Vitest is active and the
top-level route inventory has rendering contract coverage.

## Active Task

- Task ID: WTRC-040
- Owner: Codex
- Files: `web/src/api`
- Validation: `npm --prefix web run test && npm --prefix web run check`
- Status: READY
- Review: Shared UI must remain DTO-free.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Next Recommended Action

- Add fixture fallback and live mapping tests around public/admin data-source seams.
