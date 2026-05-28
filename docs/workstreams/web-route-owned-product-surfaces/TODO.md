# Web Route-Owned Product Surfaces - TODO

Status: Queued
Last updated: 2026-05-28

## M0 - Activation

- [ ] WROP-010 [owner=planner] [deps=WFBR-050] [scope=docs/workstreams/web-route-owned-product-surfaces]
  Goal: Activate after feature boundaries close.
  Validation: WFBR complete.
  Evidence: WORKSTREAM.json
  Handoff: Next task is WROP-020.

## M1 - Media Routes

- [ ] WROP-020 [owner=Codex] [deps=WROP-010] [scope=web/src/features/media,web/components/nako/nako-router.tsx]
  Goal: Add route-owned Media search/detail/library surfaces.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: route definitions and route tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M2 - Admin Routes

- [ ] WROP-030 [owner=Codex] [deps=WROP-020] [scope=web/src/features/admin,web/components/nako/nako-router.tsx]
  Goal: Add route-owned Admin libraries/users/tasks/logs/settings surfaces.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: route definitions and route tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Navigation And URL State

- [ ] WROP-040 [owner=Codex] [deps=WROP-030] [scope=web/src/features,web/components/nako]
  Goal: Move durable filters/search/page state into route params/search params.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: route state tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Closeout

- [ ] WROP-050 [owner=planner] [deps=WROP-040] [scope=docs/workstreams/web-route-owned-product-surfaces]
  Goal: Close route-owned surface lane.
  Validation: npm --prefix web run test && npm --prefix web run build plus static smoke.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Activate `web-connection-auth-tauri-profile`.
