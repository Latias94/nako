# Web Test Harness And Route Contracts - TODO

Status: Complete
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] WTRC-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-test-harness-and-route-contracts]
  Goal: Freeze the test-harness target, route contract scope, and gates.
  Validation: Workstream docs exist and agree.
  Evidence: DESIGN.md
  Handoff: DONE. First executable task is WTRC-020.

## M1 - Vitest Harness

- [x] WTRC-020 [owner=Codex] [deps=WTRC-010] [scope=web/package.json,web/vitest.config.ts,web/src/test]
  Goal: Add Vitest/Testing Library/jsdom setup and replace the `npm test` alias.
  Validation: npm --prefix web run test.
  Review: Test setup should mock browser APIs centrally without hiding real errors.
  Evidence: vitest config and setup file.
  Handoff: DONE. Vitest/Testing Library/jsdom setup is in place, `npm test` runs real tests, and the next task is route contracts.

## M2 - Route Contracts

- [x] WTRC-030 [owner=Codex] [deps=WTRC-020] [scope=web/src,web/components/nako/nako-router.tsx]
  Goal: Add top-level route rendering contract tests for the shipped route inventory.
  Validation: npm --prefix web run test.
  Review: Tests must avoid brittle full-page snapshots.
  Evidence: route contract test files.
  Handoff: DONE. Top-level routes now render through an injectable TanStack router and semantic assertions.

## M3 - Data Source Contracts

- [x] WTRC-040 [owner=Codex] [deps=WTRC-020] [scope=web/src/api]
  Goal: Add Public Client and Admin data-source tests for fixture fallback and live mapping boundaries.
  Validation: npm --prefix web run test && npm --prefix web run check.
  Review: Shared UI must remain DTO-free.
  Evidence: public/admin data-source test files.
  Handoff: DONE. Public media and Admin dashboard data sources now have fixture fallback and live mapping contract coverage.

## M4 - Closeout

- [x] WTRC-050 [owner=planner] [deps=WTRC-030,WTRC-040] [scope=docs/workstreams/web-test-harness-and-route-contracts]
  Goal: Close the lane after test, build, and smoke evidence are recorded.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Review: No blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json.
  Handoff: DONE. Lane gates passed and `web-feature-boundary-reshape` is active.
