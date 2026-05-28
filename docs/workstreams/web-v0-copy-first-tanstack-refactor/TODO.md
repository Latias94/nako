# Web V0 Copy-First TanStack Refactor - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] WVTR-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-v0-copy-first-tanstack-refactor]
  Goal: Freeze the copy-first frontend refactor direction, target stack, non-goals, and gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, HANDOFF.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/web-v0-copy-first-tanstack-refactor/DESIGN.md
  Handoff: DONE. First executable task is WVTR-020.

## M1 - Copy Baseline

- [x] WVTR-020 [owner=Codex] [deps=WVTR-010] [scope=web,repo-ref/nako-admin-web]
  Goal: Copy `repo-ref/nako-admin-web` into `web/` as the product baseline while preserving reusable Tauri and Nako API boundary assets for later reattachment.
  Validation: npm --prefix web run build or an explicitly recorded blocker if the copied Next shell cannot build before runtime quarantine.
  Review: Confirm copy source is v0 reference only and no Jellyfin/Plex source/assets are introduced.
  Evidence: web/package.json, web/app, web/components, web/lib, web/src-tauri.
  Handoff: DONE. Copy-first baseline is in place and build-validated; next task is WVTR-030.

## M2 - Runtime Quarantine

- [x] WVTR-030 [owner=Codex] [deps=WVTR-020] [scope=web]
  Goal: Remove or quarantine Next server runtime assumptions, Vercel assumptions, TMDB API route, frontend provider secrets, and third-party artwork hotlinks from the copied shell.
  Validation: npm --prefix web run check && npm --prefix web run build.
  Review: Confirm Tauri does not require a bundled Next/Node server sidecar.
  Evidence: web/next.config.mjs or replacement build config, removed app/api/tmdb route, fixture data notes.
  Handoff: DONE. The copied shell now builds as a static export with local fixture media and local artwork resolution.

## M3 - TanStack Route Ownership

- [x] WVTR-040 [owner=Codex] [deps=WVTR-030] [scope=web]
  Goal: Replace the copied single-page view-state navigation with route-owned TanStack Router surfaces for Media, Admin, Setup, and Account.
  Validation: npm --prefix web run check && npm --prefix web run test && npm --prefix web run build; Browser smoke for desktop and mobile.
  Review: Confirm major surfaces are deep-linkable and code-split.
  Evidence: web/src/routes or accepted route directory, route tests, smoke screenshots.
  Handoff: DONE. Top-level Media/Admin/Setup/Account plus Settings/Notifications are deep-linkable through TanStack Router inside the static Next shell. Current `test` script is a type-check gate until real UI/E2E tests are added.

## M4 - Nako API Boundary Reattachment

- [x] WVTR-050 [owner=Codex] [deps=WVTR-040] [scope=web/src/api,sdk/typescript,crates/nako-api]
  Goal: Reattach Nako Public Client and Admin API boundaries to the copied/refactored shell, replacing TMDB/mock hooks with Nako-owned live/fixture seams.
  Validation: npm --prefix web run check && npm --prefix web run test && npm --prefix web run build; cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output when generated contracts change.
  Review: Confirm Media routes do not import Admin DTOs and shared UI imports no API DTOs.
  Evidence: web/src/api, web/src/surfaces/media, web/src/surfaces/admin.
  Handoff: DONE. Media home/search/detail now use Public Client data with fixture fallback, and the Admin dashboard uses Admin API read-model data with fixture fallback. Deeper copied v0 pages remain fixture/planned for WVTR-070 classification.

## M5 - Performance And Desktop Gates

- [ ] WVTR-060 [owner=Codex] [deps=WVTR-040,WVTR-050] [scope=web,web/src-tauri]
  Goal: Add route-level code splitting, virtualize large grids/tables where needed, record bundle-size evidence, and restore Tauri static packaging.
  Validation: npm --prefix web run build; cargo test --manifest-path web/src-tauri/Cargo.toml; npm --prefix web run tauri -- build or documented platform-specific smoke blocker.
  Review: Confirm no Next/Node server sidecar is required by Tauri and initial route chunks have a recorded budget.
  Evidence: bundle output, web/src-tauri/tauri.conf.json, Browser/Tauri smoke evidence.
  Handoff: Native playback core remains a separate lane.

## M6 - Feature Gap Ledger And Closeout

- [ ] WVTR-070 [owner=planner] [deps=WVTR-050,WVTR-060] [scope=docs/workstreams/web-v0-copy-first-tanstack-refactor,web]
  Goal: Map copied v0 features to live, fixture, planned, blocked-by-backend, or deferred-domain status and close or split remaining work.
  Validation: EVIDENCE_AND_GATES.md includes final command evidence and gap ledger.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, HANDOFF.md, WORKSTREAM.json.
  Handoff: Split Addon Manager UI, downloads/acquisition, notifications, native playback, i18n, and old Admin Web deletion if still open.
