# Web Admin Live Wiring - TODO

Status: Queued
Last updated: 2026-05-28

## M0 - Activation

- [ ] WALW-010 [owner=planner] [deps=WCAT-050] [scope=docs/workstreams/web-admin-live-wiring]
  Goal: Activate after connection/auth closes.
  Validation: WCAT complete.
  Evidence: WORKSTREAM.json
  Handoff: Next task is WALW-020.

## M1 - Read Models

- [ ] WALW-020 [owner=Codex] [deps=WALW-010] [scope=web/src/features/admin,web/src/api/admin]
  Goal: Wire libraries/users/tasks/logs/settings read models through Admin API modules.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: admin data-source tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M2 - Mutations And Safety

- [ ] WALW-030 [owner=Codex] [deps=WALW-020] [scope=web/src/features/admin]
  Goal: Add accepted Admin mutations with confirmation, error, and permission states.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: mutation tests and UI states.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Addon Manager Slice

- [ ] WALW-040 [owner=Codex] [deps=WALW-030] [scope=web/src/features/admin/addons,web/src/api/admin]
  Goal: Replace copied plugin fixture UI with a Nako Addon Manager first slice.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: Addon Manager route/data tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Closeout

- [ ] WALW-050 [owner=planner] [deps=WALW-040] [scope=docs/workstreams/web-admin-live-wiring]
  Goal: Close Admin live-wiring lane.
  Validation: npm --prefix web run test && npm --prefix web run check && npm --prefix web run build.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Activate `web-bundle-budget-and-product-pruning`.
