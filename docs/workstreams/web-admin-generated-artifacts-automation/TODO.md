# Web Admin Generated Artifacts Automation - TODO

Status: Closed
Last updated: 2026-05-29

## M0 - Open Lane

- [x] WAGA-010 [owner=planner] [deps=WDRP-040] [scope=docs/workstreams/web-admin-generated-artifacts-automation]
  Goal: Open the new `web/` Admin Generated Artifacts / Automation lane with scope, route/API readiness, task ledger, gates, and handoff.
  Validation: `python -m json.tool docs/workstreams/web-admin-generated-artifacts-automation/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-admin-generated-artifacts-automation`.
  Evidence: Initial design, route/API readiness, task ledger, and WDRP-040 update.
  Handoff: DONE. Next task is WAGA-020.

## M1 - Admin API And Read-Model Audit

- [x] WAGA-020 [owner=Codex] [deps=WAGA-010] [scope=web/src/api/admin,web/src/test,docs/workstreams/web-admin-generated-artifacts-automation]
  Goal: Audit generated Admin proposal/review contracts and define the `web/` read-model boundary for proposal list, pagination state, fixture fallback, review-plan display, and redaction assertions.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`; `ROUTE_API_READINESS.md` updated.
  Review: verify that generated artifacts remain Admin-only and do not revive Media AI/Automation chrome.
  Evidence: `ROUTE_API_READINESS.md`, `AdminApiClient.getGeneratedArtifactProposals`, `loadGeneratedArtifacts`, fixture fallback, and data-source contract tests.
  Handoff: DONE. Next task is WAGA-030.

## M2 - Read-Only Proposal Route

- [x] WAGA-030 [owner=Codex] [deps=WAGA-020] [scope=web/src/api/admin,web/src/features/admin,web/src/shell,web/src/test]
  Goal: Implement `/admin/automation/generated-artifacts` as a read-only Admin route with route-owned pagination and fixture/live proposal data.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Review: assert no prompt text, payload bodies, provider raw data, local paths, Source Locators, tokens, or credentials are rendered.
  Evidence: route component, route contract, route-state contract, and data-source contract tests.
  Handoff: DONE. Next task is WAGA-040.

## M3 - Review Mutation Guard Decision

- [x] WAGA-040 [owner=Codex] [deps=WAGA-030] [scope=web/src/api/admin,web/src/features/admin,docs/workstreams/web-admin-generated-artifacts-automation]
  Goal: Decide whether review-plan and accept/reject belong in this lane as guarded actions or must split into a follow-on.
  Validation: decision recorded with review-plan display, confirmation, idempotency, boundary flags, redaction, and mutation-result requirements; if implemented, `npm --prefix web run test` and `npm --prefix web run check` pass.
  Review: no action may imply direct canonical metadata, sidecar, or library-file writes.
  Evidence: `MUTATION_BOUNDARY_DECISION.md` and updated `ROUTE_API_READINESS.md`.
  Handoff: DONE. Review mutation controls are split to a future guarded mutation lane. Next task is WAGA-050.

## M4 - Closeout

- [x] WAGA-050 [owner=planner] [deps=WAGA-030,WAGA-040] [scope=docs/workstreams/web-admin-generated-artifacts-automation]
  Goal: Close the lane with browser/Tauri readiness evidence, bundle budget output, and follow-ons for provider adapters, local runtime, review mutations, or metadata-authority apply.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke; `git diff --check`.
  Review: workstream compliance and no blocking code-quality findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, and `CLOSEOUT.md`.
  Handoff: DONE. Lane closed. Return to WDRP or start the selected follow-on.
