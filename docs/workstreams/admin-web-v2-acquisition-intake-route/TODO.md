# Admin Web V2 Acquisition Intake Route TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AIR-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-acquisition-intake-route]
  Goal: Freeze the read-only route migration scope, non-goals, and evidence gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: this workstream.
  Handoff: Continue with AIR-020.

## M1 - Route-First Intake Page

- [x] AIR-020 [owner=codex] [deps=AIR-010] [scope=apps/admin-web/src/App.tsx, apps/admin-web/src/features/acquisition, apps/admin-web/src/adminApi]
  Goal: Implement `/acquisition/intake` as a read-only V2 route with URL-owned filters and route-local fallback.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`.
  Review: verify route ownership, Admin API boundary, and redaction-sensitive rendering.
  Evidence: `apps/admin-web/src/features/acquisition/AcquisitionIntakePage.tsx`.
  Handoff: Continue with AIR-030.

## M2 - Evidence And Browser Smoke

- [x] AIR-030 [owner=codex] [deps=AIR-020] [scope=apps/admin-web, docs/workstreams/admin-web-v2-acquisition-intake-route]
  Goal: Run package gates and browser smoke for the route.
  Validation: frontend package gate, `git diff --check`, and Playwright smoke for desktop/mobile.
  Review: no unsafe rendered source refs, local paths, raw locators, or credentials.
  Evidence: `EVIDENCE_AND_GATES.md` and screenshots under `target/admin-web-v2-acquisition-intake-smoke/`.
  Handoff: Continue with AIR-040.

## M3 - Closeout

- [x] AIR-040 [owner=codex] [deps=AIR-030] [scope=docs/workstreams/admin-web-v2-acquisition-intake-route]
  Goal: Close the lane and record follow-ons.
  Validation: WORKSTREAM.json, TODO.md, HANDOFF.md, and CLOSEOUT.md agree.
  Review: no blocking findings.
  Evidence: closeout notes.
  Handoff: Continue Admin Web V2 migration with the next legacy workflow.
