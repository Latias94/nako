# Web Feature Boundary Reshape - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Activation

- [x] WFBR-010 [owner=planner] [deps=WTRC-050] [scope=docs/workstreams/web-feature-boundary-reshape]
  Goal: Activate this lane after the test harness closes.
  Validation: WTRC is complete and current docs still match `web/`.
  Evidence: WORKSTREAM.json
  Handoff: DONE. WTRC is complete and this lane is active; next task is WFBR-020.

## M1 - Media Boundary

- [x] WFBR-020 [owner=Codex] [deps=WFBR-010] [scope=web/src/features/media,web/components/nako]
  Goal: Move Media surface components into a feature-owned boundary without behavior changes.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Review: Shared UI remains DTO-free.
  Evidence: moved files and import diff.
  Handoff: DONE. Media surface and its current internal product pages live under `web/src/features/media`.

## M2 - Admin Boundary

- [ ] WFBR-030 [owner=Codex] [deps=WFBR-020] [scope=web/src/features/admin,web/components/nako]
  Goal: Move Admin surface components into a feature-owned boundary.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Review: Admin DTO imports remain inside admin API/feature boundary only.
  Evidence: moved files and import diff.
  Handoff: READY. Move Admin surface and admin-only child components next.

## M3 - Shell And Deferred Boundaries

- [ ] WFBR-040 [owner=Codex] [deps=WFBR-030] [scope=web/src/features,web/components/nako]
  Goal: Move setup/account/notifications/TV and isolate deferred copied domains.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Review: Deferred domains do not enter initial route imports.
  Evidence: feature directory map.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Closeout

- [ ] WFBR-050 [owner=planner] [deps=WFBR-040] [scope=docs/workstreams/web-feature-boundary-reshape]
  Goal: Close the feature-boundary lane.
  Validation: npm --prefix web run test && npm --prefix web run check && npm --prefix web run build.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Activate `web-route-owned-product-surfaces`.
