# Admin Web V2 Addons Route - TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AWAD-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-addons-route]
  Goal: Freeze read-only Addons route scope, status filter ownership, and
  credential redaction constraints.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  HANDOFF.md, and WORKSTREAM.json agree.
  Evidence: `docs/workstreams/admin-web-v2-addons-route/DESIGN.md`
  Handoff: First executable task is AWAD-020.

## M1 - Route And Data Boundary

- [x] AWAD-020 [owner=codex] [deps=AWAD-010] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/addons]
  Goal: Implement `/addons` as a route-first read-only page with generated
  status query params and deterministic Addon summary fallback.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`
  Review: Admin API calls must remain behind `AdminDataSource`; route output
  must not include raw tokens, env var names, URLs, paths, install snippets,
  credential values, raw manifest JSON, or diagnostic payloads.
  Evidence: `apps/admin-web/src/features/addons/AddonsPage.tsx`
  Handoff: DONE. Route output uses `AddonsRouteSummary`, URL status filtering,
  section-local fallback, and tests for unsafe rendered text exclusions.

## M2 - Evidence And Closeout

- [x] AWAD-030 [owner=codex] [deps=AWAD-020] [scope=apps/admin-web,docs/workstreams/admin-web-v2-addons-route]
  Goal: Run full frontend gates, browser smoke, and update evidence.
  Validation: `npm run generate:admin-api`, `npm run check`, `npm run test`,
  `npm run build`, `git diff --check`, Playwright desktop/mobile smoke.
  Review: review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Mutation and credential-producing workflows remain follow-ons.
