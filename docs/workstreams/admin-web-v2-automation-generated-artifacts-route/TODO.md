# Admin Web V2 Automation Generated Artifacts Route TODO

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] AGA-010 [owner=codex] [deps=none] [scope=docs/workstreams/admin-web-v2-automation-generated-artifacts-route]
  Goal: Freeze the read-only route migration scope, non-goals, and evidence gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: this workstream.
  Handoff: Continue with AGA-020.

## M1 - Route-First Generated Artifacts Page

- [x] AGA-020 [owner=codex] [deps=AGA-010] [scope=apps/admin-web/src/App.tsx, apps/admin-web/src/features/automation, apps/admin-web/src/adminApi]
  Goal: Implement `/automation/generated-artifacts` as a read-only V2 route with URL-owned pagination and route-local fallback.
  Validation: `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/dataSource.test.ts`.
  Review: verify route ownership, Admin API boundary, and redaction-sensitive rendering.
  Evidence: `apps/admin-web/src/features/automation/GeneratedArtifactsPage.tsx`.
  Handoff: Continue with AGA-030.

## M2 - Evidence And Browser Smoke

- [x] AGA-030 [owner=codex] [deps=AGA-020] [scope=apps/admin-web, docs/workstreams/admin-web-v2-automation-generated-artifacts-route]
  Goal: Run package gates and browser smoke for the route.
  Validation: frontend package gate, `git diff --check`, and Playwright smoke for desktop/mobile.
  Review: no unsafe rendered prompt bodies, payload bodies, provider raw data, local paths, or credentials.
  Evidence: `EVIDENCE_AND_GATES.md` and screenshots under `target/admin-web-v2-generated-artifacts-smoke/`.
  Handoff: Continue with AGA-040.

## M3 - Closeout

- [x] AGA-040 [owner=codex] [deps=AGA-030] [scope=docs/workstreams/admin-web-v2-automation-generated-artifacts-route]
  Goal: Close the lane and record follow-ons.
  Validation: WORKSTREAM.json, TODO.md, HANDOFF.md, and CLOSEOUT.md agree.
  Review: no blocking findings.
  Evidence: closeout notes.
  Handoff: Continue Admin Web V2 migration with the next legacy workflow.
