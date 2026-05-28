# Web Route-Owned Product Surfaces - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Activation

- [x] WROP-010 [owner=planner] [deps=WFBR-050] [scope=docs/workstreams/web-route-owned-product-surfaces]
  Goal: Activate after feature boundaries close.
  Validation: WFBR complete.
  Evidence: WORKSTREAM.json
  Handoff: DONE. WFBR is complete and this lane is active; next task is WROP-020.

## M1 - Media Routes

- [x] WROP-020 [owner=Codex] [deps=WROP-010] [scope=web/src/features/media,web/src/shell/nako-router.tsx]
  Goal: Add route-owned Media search/detail/library surfaces.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: route definitions and route tests.
  Handoff: DONE. `/media/search`, `/media/detail`, and `/media/library` are route-owned and covered by route contract tests.

## M2 - Admin Routes

- [ ] WROP-030 [owner=Codex] [deps=WROP-020] [scope=web/src/features/admin,web/src/shell/nako-router.tsx]
  Goal: Add route-owned Admin libraries/users/tasks/logs/settings surfaces.
  Validation: npm --prefix web run test && npm --prefix web run build.
  Evidence: route definitions and route tests.
  Handoff: READY.

## M3 - Navigation And URL State

- [ ] WROP-040 [owner=Codex] [deps=WROP-030] [scope=web/src/features,web/src/shell]
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
