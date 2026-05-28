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

- Task ID: WVRT-030
- Owner: Codex
- Files: `web/app`, `web/next.config.mjs`, `web/next-env.d.ts`,
  `web/package.json`, `web/package-lock.json`, `web/tsconfig.json`
- Validation: `npm --prefix web run check && npm --prefix web run build`, then
  `rg` confirms no Next release imports remain.
- Status: READY
- Review: Confirm deleted files are Next-only and `/tv` remains routable through
  TanStack Router.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Keep `components/`, `lib/`, and `src/api/` in place during this lane.
- WVRT-020 added `/tv` to TanStack Router so the Vite app preserves the static
  route inventory from Next.
- Tauri should move from `../out` to `../dist` after Vite build ownership lands.

## Blockers

- None known.

## Next Recommended Action

- Implement WVRT-030 by deleting the remaining Next runtime surface.
