# Web Vite TanStack Runtime - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

The previous copy-first lane is closed. `web/` now has TanStack-owned top-level
routes and static Tauri packaging, but Next remains as a temporary bootstrap
shell.

This lane owns removing that shell and making Vite the only browser/Tauri web
runtime.

## Active Task

- Task ID: WVRT-020
- Owner: Codex
- Files: `web/package.json`, `web/vite.config.ts`, `web/index.html`,
  `web/src`, `web/tsconfig.json`
- Validation: `npm --prefix web run check && npm --prefix web run build`
- Status: READY
- Review: Confirm no product feature wiring or visual redesign sneaks into the
  runtime proof.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Keep `components/`, `lib/`, and `src/api/` in place during this lane.
- Add `/tv` to TanStack Router during migration so the Vite app preserves the
  static route inventory from Next.
- Tauri should move from `../out` to `../dist` after Vite build ownership lands.

## Blockers

- None known.

## Next Recommended Action

- Implement WVRT-020 as the first Vite runtime proof.
