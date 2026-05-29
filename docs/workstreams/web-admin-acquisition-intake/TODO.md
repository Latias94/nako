# Web Admin Acquisition Intake - TODO

Status: Active
Last updated: 2026-05-29

## M0 - Open Lane

- [x] WAAI-010 [owner=planner] [deps=WDRP-030] [scope=docs/workstreams/web-admin-acquisition-intake]
  Goal: Open the new `web/` Admin Acquisition Intake lane with scope, route/API readiness, task ledger, gates, and handoff.
  Validation: `python -m json.tool docs/workstreams/web-admin-acquisition-intake/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-admin-acquisition-intake`.
  Evidence: Initial design, route/API readiness, task ledger, and WDRP-030 update.
  Handoff: DONE. Next task is WAAI-020.

## M1 - Admin API And Read-Model Audit

- [x] WAAI-020 [owner=Codex] [deps=WAAI-010] [scope=web/src/api/admin,web/src/test,docs/workstreams/web-admin-acquisition-intake]
  Goal: Audit generated Admin acquisition contracts and define the `web/` read-model boundary for candidate list, route query state, fixture fallback, and redaction assertions.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`; `ROUTE_API_READINESS.md` updated.
  Review: verify that the route uses Admin contracts only and does not revive a Media Downloads surface.
  Evidence: `ROUTE_API_READINESS.md`, `AdminApiClient.getAcquisitionIntakeCandidates`, `loadAcquisitionIntake`, fixture fallback, and data-source contract tests.
  Handoff: DONE. Next task is WAAI-030.

## M2 - Route-First Intake Page

- [ ] WAAI-030 [owner=Codex] [deps=WAAI-020] [scope=web/src/api/admin,web/src/features/admin,web/src/shell,web/src/test]
  Goal: Implement `/admin/acquisition/intake` as a read-only Admin route with route-owned query state and fixture/live data-source behavior.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Review: assert no raw locators, local paths, credentials, prompt bodies, or downloader internals are rendered.
  Evidence: route component, route contract, route-state contract, and data-source contract tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Mutation Boundary Decision

- [ ] WAAI-040 [owner=Codex] [deps=WAAI-030] [scope=web/src/api/admin,web/src/features/admin,docs/workstreams/web-admin-acquisition-intake]
  Goal: Decide whether watch-folder discovery belongs in this lane as a guarded mutation or must split into a follow-on.
  Validation: decision recorded with Admin API route, idempotency, permission, redaction, and UI guard requirements; if implemented, `npm --prefix web run test` and `npm --prefix web run check` pass.
  Review: no mutation may imply promotion/apply or direct library writes.
  Evidence: updated `ROUTE_API_READINESS.md`, `TODO.md`, and tests if code changes are made.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Closeout

- [ ] WAAI-050 [owner=planner] [deps=WAAI-030,WAAI-040] [scope=docs/workstreams/web-admin-acquisition-intake]
  Goal: Close the lane with browser/Tauri readiness evidence, bundle budget output, and follow-ons for downloader protocols or Managed Import mutations.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke; `git diff --check`.
  Review: workstream compliance and no blocking code-quality findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md`.
  Handoff: DONE. Return to WDRP or start the selected follow-on.
