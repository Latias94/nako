# Web Modern Frontend And Tauri Foundation - TODO

Status: Complete
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] WMFT-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-modern-frontend-and-tauri-foundation]
  Goal: Freeze the replacement direction: `web/` is the product frontend,
  `apps/admin-web` is validation/prototype only, and v0 is UX reference rather
  than a shippable Next.js import.
  Validation: DESIGN.md, README.md, TODO.md, MILESTONES.md,
  EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/web-modern-frontend-and-tauri-foundation/DESIGN.md
  Handoff: DONE. First implementation task is WMFT-020.

## M1 - Product App Scaffold

- [x] WMFT-020 [owner=codex] [deps=WMFT-010] [scope=web]
  Goal: Create the `web/` Vite React product app with TypeScript, React 19,
  Tailwind v4, shadcn-style configuration, lucide icons, TanStack Router,
  TanStack Query, package scripts, and a Tauri shell skeleton targeting the new
  frontend.
  Validation: npm --prefix web run verify; cargo check --manifest-path web/src-tauri/Cargo.toml
  Review: review-workstream for dependency shape, package scripts, and absence
  of Next.js/Vercel/TMDb route assumptions.
  Evidence: web/package.json, web/src route scaffold, web/src-tauri, command output in
  EVIDENCE_AND_GATES.md.
  Handoff: DONE. No Next.js/Vercel/TMDb route assumptions were introduced; Tauri
  uses the new web frontend as its shell. Native playback remains a follow-on.

## M2 - Design System And Route Shell

- [x] WMFT-030 [owner=codex] [deps=WMFT-020] [scope=web/src]
  Goal: Re-author the v0 visual direction into Nako-owned tokens, layout,
  surface switcher, navigation, empty states, and route-owned Media/Admin
  shells.
  Validation: npm --prefix web run check && npm --prefix web run test &&
  npm --prefix web run build, plus Browser/Playwright smoke screenshots for
  desktop and mobile viewports.
  Review: UX review for route clarity, responsive behavior, text fit, icon use,
  and no unsupported feature claims.
  Evidence: web/src/components, web/src/routes, visual evidence linked from
  EVIDENCE_AND_GATES.md.
  Handoff: DONE. Shell uses Nako-owned tokens, active navigation, truthful
  disconnected empty states, generated CSS artwork slots, and no v0 assets or
  Next.js/Vercel assumptions. First smoke caught and fixed a media CSS token
  self-reference that made dark-surface text unreadable.

## M3 - API Boundary Integration

- [x] WMFT-040 [owner=codex] [deps=WMFT-020] [scope=web/src/api,sdk/typescript,crates/nako-api]
  Goal: Connect the new frontend to Public Client SDK and generated Admin API
  contract through surface-specific data modules and fixture/live seams.
  Validation: SDK/Admin contract generation commands, npm --prefix web run
  check && npm --prefix web run test, and focused Rust contract tests if
  generator outputs change.
  Review: Boundary review for Admin DTO leakage into Media routes, Public DTO
  leakage into Admin-only operations, and redaction safety.
  Evidence: web/src/api, generated contract path, EVIDENCE_AND_GATES.md.
  Handoff: DONE. `web/` now depends on `@nako/sdk`, generates its own Admin
  API contract at `web/src/api/admin/generated/contract.ts`, and has
  surface-specific Media/Admin API modules with live/fixture section results.
  Rust contract drift tests now compare the generator output against both
  `apps/admin-web` and `web/`.

## M4 - Media Web First Product Slice

- [x] WMFT-050 [owner=codex] [deps=WMFT-030,WMFT-040] [scope=web/src/surfaces/media]
  Goal: Implement the first API-backed Media Web slice: libraries, library
  detail, item detail, source/version picker, browser playback ticket player,
  Continue Watching basics, and fixture/live parity tests.
  Validation: npm --prefix web run check && npm --prefix web run test &&
  npm --prefix web run build; Browser/Playwright smoke for browse, detail, and
  player routes.
  Review: Product/API review for Public Client API only, playback ticket safety,
  and no raw Source Locator/local path exposure.
  Evidence: web/src/surfaces/media, browser smoke evidence.
  Handoff: DONE. Media routes now consume `web/src/api/media` through TanStack
  Query, including Continue Watching, Libraries, Library Detail, Item Detail,
  source selection via `sourceId` search state, and browser playback ticket
  request handling. Fixture tickets are not attached to the video element;
  live tickets are the only source assigned to `<video src>`.

## M5 - Admin Product Slice

- [x] WMFT-060 [owner=codex] [deps=WMFT-030,WMFT-040] [scope=web/src/surfaces/admin]
  Goal: Implement the first API-backed Admin route family in the new product
  frontend: overview, libraries, jobs/sessions, Addons, settings/readiness, and
  safe links back into Media routes.
  Validation: npm --prefix web run check && npm --prefix web run test &&
  npm --prefix web run build; Browser/Playwright smoke for `/admin/*`.
  Review: Admin boundary review for redaction, operation confirmation, and
  truthful unsupported controls.
  Evidence: web/src/surfaces/admin, route tests, smoke evidence.
  Handoff: DONE. Admin Overview, Libraries, Jobs, Addons, and Settings now read
  from `web/src/api/admin` through TanStack Query. The slice remains read-only:
  broad operations and destructive controls stay out until Admin API authority
  and confirmation UX are added.

## M6 - Tauri Foundation

- [x] WMFT-070 [owner=codex] [deps=WMFT-050] [scope=web/src-tauri,web/package.json]
  Goal: Deepen the Tauri desktop shell beyond the WMFT-020 skeleton with server
  connection bootstrap, desktop packaging smoke, and explicit native playback
  split evidence.
  Validation: npm --prefix web run check && npm --prefix web run build plus
  Tauri check/build smoke accepted by the task.
  Review: Desktop architecture review against ADR 0026 and ADR 0032.
  Evidence: web/src-tauri, Tauri command output in EVIDENCE_AND_GATES.md.
  Handoff: Split native playback core integration before claiming desktop
  playback quality. DONE. Tauri now exposes no-secret server profile bootstrap
  commands, the web runtime verifies `/health` before storing a server profile,
  and `npm --prefix web run tauri -- build` produces the Windows desktop shell.
  Native playback core remains explicitly unavailable in the bootstrap payload.

## M7 - Old Admin Web Retirement Plan

- [x] WMFT-080 [owner=codex] [deps=WMFT-050,WMFT-060] [scope=apps/admin-web,web,docs]
  Goal: Define and execute the safe retirement or archive plan for
  `apps/admin-web` after `web/` owns equivalent validation coverage.
  Validation: parity matrix, replacement test gates, and explicit deletion or
  archive decision recorded before any removal.
  Review: review-workstream for user-change safety and contract validation
  continuity.
  Evidence: parity matrix and final diff.
  Handoff: Never delete the old frontend only because the new app scaffold
  exists. DONE. `ADMIN_WEB_RETIREMENT_PLAN.md` records the parity matrix,
  retention rules, deletion-over-archive target, and blocking gaps. Current
  decision: keep `apps/admin-web` as validation-only until every row is ready,
  moved to CI, or split to an accepted follow-on.

## M8 - Closeout

- [x] WMFT-090 [owner=codex] [deps=WMFT-020,WMFT-030,WMFT-040,WMFT-050,WMFT-060,WMFT-070,WMFT-080] [scope=docs/workstreams/web-modern-frontend-and-tauri-foundation]
  Goal: Close this lane or split remaining work to narrower product/frontend
  lanes.
  Validation: verify-rust-workstream records fresh final gate evidence,
  including frontend package checks and any Tauri smoke evidence that exists.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md.
  Handoff: Split Addon Manager UI, notifications, acquisition/downloads,
  native playback, mobile packaging, and AI workflows to separate lanes. DONE.
  `CLOSEOUT.md` closes this foundation lane and records the remaining product
  lanes without claiming full frontend parity.
