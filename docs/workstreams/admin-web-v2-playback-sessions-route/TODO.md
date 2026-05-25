# Admin Web V2 Playback Sessions Route - TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AWPS-010 [owner=planner] [deps=none] [scope=docs/workstreams/admin-web-v2-playback-sessions-route]
  Goal: Freeze read-only Playback Sessions route scope, query ownership, and
  evidence gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  HANDOFF.md, and WORKSTREAM.json agree.
  Evidence: `docs/workstreams/admin-web-v2-playback-sessions-route/DESIGN.md`
  Handoff: First executable task is AWPS-020.

## M1 - Route And Data Boundary

- [x] AWPS-020 [owner=codex] [deps=AWPS-010] [scope=apps/admin-web/src/App.tsx,apps/admin-web/src/adminApi,apps/admin-web/src/features/playback]
  Goal: Implement `/playback/sessions` as a route-first read-only page with
  generated query params and deterministic fallback.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`
  Review: Admin API calls must remain behind `AdminDataSource`; route output
  must not include request keys, paths, FFmpeg commands, stderr, credentials, or
  filesystem roots.
  Evidence: `apps/admin-web/src/features/playback/PlaybackSessionsPage.tsx`
  Handoff: DONE. `/playback/sessions` now renders a route-first read-only page
  with generated query DTO mapping.

## M2 - Evidence And Closeout

- [x] AWPS-030 [owner=codex] [deps=AWPS-020] [scope=apps/admin-web,docs/workstreams/admin-web-v2-playback-sessions-route]
  Goal: Run full frontend gates, browser smoke, and update evidence.
  Validation: `npm run generate:admin-api`, `npm run check`, `npm run test`,
  `npm run build`, `git diff --check`, Playwright desktop/mobile smoke.
  Review: review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Support evidence and detail workflows are deferred follow-ons.
