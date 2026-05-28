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

- Task ID: WVRT-040
- Owner: Codex
- Files: `web`, `web/src-tauri`, `docs/workstreams/web-vite-tanstack-runtime`
- Validation: `npm --prefix web run build`, static browser smoke,
  `cargo test --manifest-path web/src-tauri/Cargo.toml`, and
  `npm --prefix web run tauri -- build`.
- Status: READY
- Review: Confirm Tauri uses Vite `dist`, route fallback works, and no Node
  sidecar is required.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Keep `components/`, `lib/`, and `src/api/` in place during this lane.
- WVRT-020 added `/tv` to TanStack Router so the Vite app preserves the static
  route inventory from Next.
- WVRT-030 removed Next app wrappers, Next config/type files, and `next` /
  `next-themes` dependencies.
- Tauri should move from `../out` to `../dist` in WVRT-040.

## Blockers

- None known.

## Next Recommended Action

- Implement WVRT-040 by proving static browser and Tauri behavior from Vite
  output.
