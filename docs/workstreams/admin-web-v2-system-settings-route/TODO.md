# Admin Web V2 System Settings Route - TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AWSR-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-system-settings-route]
  Goal: Freeze read-only System Settings route scope, redaction constraints,
  and evidence gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  HANDOFF.md, and WORKSTREAM.json agree.
  Evidence: `docs/workstreams/admin-web-v2-system-settings-route/DESIGN.md`
  Handoff: First executable task is AWSR-020.

## M1 - Route And Data Boundary

- [x] AWSR-020 [owner=codex] [deps=AWSR-010] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/settings]
  Goal: Implement `/settings` as a route-first read-only page with
  deterministic system config fallback.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`
  Review: Admin API calls must remain behind `AdminDataSource`; route output
  must not include env var names, URLs, roots, paths, tokens, provider secrets,
  credentials, or raw config.
  Evidence: `apps/admin-web/src/features/settings/SettingsPage.tsx`
  Handoff: DONE. `/settings` is route-owned and renders safe read-only system
  diagnostics through `AdminDataSource`.

## M2 - Evidence And Closeout

- [x] AWSR-030 [owner=codex] [deps=AWSR-020] [scope=apps/admin-web,docs/workstreams/admin-web-v2-system-settings-route]
  Goal: Run full frontend gates, browser smoke, and update evidence.
  Validation: `npm run generate:admin-api`, `npm run check`, `npm run test`,
  `npm run build`, `git diff --check`, Playwright desktop/mobile smoke.
  Review: review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Mutation and richer configuration workflows are deferred
  follow-ons.
