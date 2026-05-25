# Admin Web V2 Overview Route - TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AWOV-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-overview-route]
  Goal: Freeze Overview route scope, default-route ownership, and evidence
  gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  HANDOFF.md, and WORKSTREAM.json agree.
  Evidence: `docs/workstreams/admin-web-v2-overview-route/DESIGN.md`
  Handoff: First executable task is AWOV-020.

## M1 - Route And Data Boundary

- [x] AWOV-020 [owner=codex] [deps=AWOV-010] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/overview]
  Goal: Implement `/overview` as the V2 default route with route-local fallback.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`
  Review: Admin API calls must remain behind `AdminDataSource`; route output
  must not include roots, paths, tokens, provider secrets, credentials, or raw
  config.
  Evidence: `apps/admin-web/src/features/overview/OverviewPage.tsx`
  Handoff: DONE. `/overview` is route-owned, `/` redirects to it, and the page
  renders safe overview summary data through `AdminDataSource`.

## M2 - Evidence And Closeout

- [x] AWOV-030 [owner=codex] [deps=AWOV-020] [scope=apps/admin-web,docs/workstreams/admin-web-v2-overview-route]
  Goal: Run full frontend gates, browser smoke, and update evidence.
  Validation: `npm run generate:admin-api`, `npm run check`, `npm run test`,
  `npm run build`, `git diff --check`, Playwright desktop/mobile smoke.
  Review: review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Richer overview cards and new backend fields are deferred
  follow-ons.
