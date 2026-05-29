# Web Admin Generated Artifact Review Mutations - TODO

Status: Closed
Last updated: 2026-05-29

## M0 - Open Lane

- [x] WGAR-010 [owner=planner] [deps=none] [scope=docs/workstreams/web-admin-generated-artifact-review-mutations,docs/architecture/WORKSTREAM_LINKS.md,docs/workstreams/README.md]
  Goal: Open a durable guarded Generated Artifact review mutation workstream from WAGA closeout.
  Validation: `python -m json.tool docs/workstreams/web-admin-generated-artifact-review-mutations/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-admin-generated-artifact-review-mutations docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md`.
  Review: confirms the lane records `POST review-plan`, redaction rules, route shape, and mutation gates.
  Handoff: DONE. Continue with `WGAR-020`.

## M1 - API And Data-Source Boundary

- [x] WGAR-020 [owner=worker] [deps=WGAR-010] [scope=web/src/api/admin,web/src/test/data-source-contracts.test.ts]
  Goal: Add review-plan and review methods to the Admin client, redacted review-plan read model, domain-specific review mutation result, fixture behavior, and contract tests.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`.
  Review: request method/body/path/auth are asserted; unsafe fields are absent from mapped output; fixture mutation rejects.
  Handoff: DONE. Passed to `WGAR-030`.

## M2 - Guarded Review Route

- [x] WGAR-030 [owner=worker] [deps=WGAR-020] [scope=web/src/features/admin,web/src/shell/nako-router.tsx,web/src/test/route-contracts.test.tsx,web/src/test/route-state-contracts.test.tsx]
  Goal: Add queue-to-review navigation, route-owned `artifact_id`/`decision` state, review-plan display, confirmation controls, mutation result/error rendering, and query invalidation.
  Validation: `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run check`.
  Review: fixture mode cannot mutate; live mode posts the selected decision; result shows idempotent replay and artifact status; unsafe fields never render.
  Handoff: DONE. Passed to `WGAR-040`.

## M3 - Verification And Closeout

- [x] WGAR-040 [owner=planner] [deps=WGAR-020,WGAR-030] [scope=docs/workstreams/web-admin-generated-artifact-review-mutations,web]
  Goal: Run full frontend gates, desktop/mobile browser smoke, bundle budget, closeout docs, and precise commit.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; browser smoke; `git diff --check`; `python -m json.tool docs/workstreams/web-admin-generated-artifact-review-mutations/WORKSTREAM.json`.
  Review: workstream compliance and no blocking code-quality findings.
  Handoff: DONE. Lane closed with `CLOSEOUT.md`.
