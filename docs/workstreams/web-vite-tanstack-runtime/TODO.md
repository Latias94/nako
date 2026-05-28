# Web Vite TanStack Runtime - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] WVRT-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-vite-tanstack-runtime]
  Goal: Freeze the Next-to-Vite runtime migration target, non-goals, and gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, HANDOFF.md, WORKSTREAM.json exist and agree.
  Review: Confirm this lane does not hide feature wiring or visual redesign.
  Evidence: docs/workstreams/web-vite-tanstack-runtime/DESIGN.md
  Handoff: DONE. First executable task is WVRT-020.

## M1 - Vite Runtime Proof

- [x] WVRT-020 [owner=Codex] [deps=WVRT-010] [scope=web/package.json,web/vite.config.ts,web/index.html,web/src,web/tsconfig.json]
  Goal: Add the Vite React entry, move root layout responsibilities into Vite-owned files, and make check/build pass without relying on Next page wrappers.
  Validation: npm --prefix web run check && npm --prefix web run build.
  Review: Confirm the proof keeps Nako API seams and does not introduce a server runtime.
  Evidence: web/index.html, web/vite.config.ts, web/src/main.tsx.
  Handoff: DONE. Vite now owns the runnable web entry, `web/dist` build output is produced, and the next task is deleting the remaining Next runtime surface.

## M2 - Delete Next Runtime Surface

- [ ] WVRT-030 [owner=Codex] [deps=WVRT-020] [scope=web/app,web/next.config.mjs,web/next-env.d.ts,web/package.json,web/package-lock.json]
  Goal: Remove Next-only wrappers, config, type plugins, scripts, and dependencies after Vite owns build/runtime behavior.
  Validation: npm --prefix web run check && npm --prefix web run build; rg confirms no Next release source imports remain.
  Review: Confirm `/tv` is preserved through TanStack Router and deleted files are truly Next-only.
  Evidence: package diff, removed app directory, route smoke.
  Handoff: Final status must be DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.

## M3 - Browser And Tauri Static Proof

- [ ] WVRT-040 [owner=Codex] [deps=WVRT-030] [scope=web,web/src-tauri]
  Goal: Point Tauri at Vite `dist`, verify desktop/static browser behavior, and record bundle evidence.
  Validation: npm --prefix web run build; Browser/Playwright smoke for `/media`, `/admin`, `/setup`, `/account`, `/tv`, and mobile `/media`; cargo test --manifest-path web/src-tauri/Cargo.toml; npm --prefix web run tauri -- build.
  Review: Confirm no Next/Node server sidecar is required and route fallback works in static hosting.
  Evidence: EVIDENCE_AND_GATES.md, web/src-tauri/tauri.conf.json.
  Handoff: Final status must be DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.

## M4 - Closeout

- [ ] WVRT-050 [owner=planner] [deps=WVRT-040] [scope=docs/workstreams/web-vite-tanstack-runtime]
  Goal: Close the runtime migration lane or split any remaining runtime-only follow-ons.
  Validation: EVIDENCE_AND_GATES.md includes final command evidence and bundle notes.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, HANDOFF.md, WORKSTREAM.json.
  Handoff: Summarize remaining risks and next frontend lane.
