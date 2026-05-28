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

- Task ID: WVRT-050
- Owner: planner
- Files: `docs/workstreams/web-vite-tanstack-runtime`
- Validation: final evidence review, `WORKSTREAM.json` parse, and
  `git diff --check`.
- Status: READY
- Review: Confirm the lane can close and any remaining runtime-only work is
  split or deferred.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Keep `components/`, `lib/`, and `src/api/` in place during this lane.
- WVRT-020 added `/tv` to TanStack Router so the Vite app preserves the static
  route inventory from Next.
- WVRT-030 removed Next app wrappers, Next config/type files, and `next` /
  `next-themes` dependencies.
- WVRT-040 moved Tauri from `../out` to `../dist`, passed static route smoke,
  Tauri tests, and Tauri build.

## Blockers

- None known.

## Next Recommended Action

- Close WVRT-050 after final review and evidence cleanup.
