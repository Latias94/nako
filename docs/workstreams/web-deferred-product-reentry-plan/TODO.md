# Web Deferred Product Reentry Plan - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Open Lane

- [x] WDRP-010 [owner=planner] [deps=WBBP-050] [scope=docs/workstreams/web-deferred-product-reentry-plan]
  Goal: Convert deferred frontend gaps into an explicit reentry plan.
  Validation: `python -m json.tool docs/workstreams/web-deferred-product-reentry-plan/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-deferred-product-reentry-plan`.
  Evidence: Initial design, reentry matrix, and task ledger.
  Handoff: DONE. Next task is WDRP-020.

## M1 - Video-First Reentry

- [x] WDRP-020 [owner=planner] [deps=WDRP-010] [scope=docs/workstreams]
  Goal: Open or update the next Media Web lane for live Public Client browsing/detail/playback parity in the new `web/` shell.
  Validation: new or updated workstream docs reference Public Client routes, route contracts, browser/Tauri gates, and bundle budget gates.
  Evidence: Opened `docs/workstreams/web-media-live-public-client-parity`; first executable task is WMLP-020 Public Client readiness audit.
  Handoff: DONE. Next task is WDRP-030.

## M2 - Admin Operations Reentry

- [x] WDRP-030 [owner=planner] [deps=WDRP-010] [scope=docs/workstreams]
  Goal: Open or update a new `web/` Admin Acquisition Intake route lane, reusing the completed backend/Admin V2 work instead of restoring a Media downloads page.
  Validation: workstream docs name Admin API contracts, fixture/live data-source tests, route-state tests, and `build:budget`.
  Evidence: Opened `docs/workstreams/web-admin-acquisition-intake`; first executable task is WAAI-020 Admin API and read-model audit.
  Handoff: DONE. Next task is WDRP-040.

- [x] WDRP-040 [owner=planner] [deps=WDRP-010] [scope=docs/workstreams]
  Goal: Open or update a new `web/` Admin Generated Artifacts / Automation route lane.
  Validation: workstream docs reference generated artifacts, review-plan semantics, mutation guards, and Admin API contract generation.
  Evidence: Opened `docs/workstreams/web-admin-generated-artifacts-automation`; first executable task is WAGA-020 Admin API and read-model audit.
  Handoff: DONE. Next task is WDRP-050.

## M3 - User Media State

- [x] WDRP-050 [owner=planner] [deps=WDRP-020] [scope=docs/workstreams]
  Goal: Decide whether playlists are ready for a backend contract lane or should remain deferred behind user playback state.
  Validation: decision recorded with contract prerequisites and no frontend UI before API shape.
  Evidence: Opened `docs/workstreams/user-playlists-contract-and-web-slice`; first executable task is UPCW-020 Public Contract Freeze.
  Handoff: DONE. Next task is WDRP-060.

## M4 - Non-Video Media Domains

- [x] WDRP-060 [owner=planner] [deps=WDRP-010] [scope=docs/workstreams]
  Goal: Decide when photos, music, and podcasts deserve a non-video media-domain baseline lane.
  Validation: decision references ADR-0021 and avoids UI-first implementation.
  Evidence: `NON_VIDEO_DOMAIN_DECISION.md` records the deferred decision and reentry triggers.
  Handoff: DONE. Next task is WDRP-065.

## M5 - Closeout

- [ ] WDRP-065 [owner=planner] [deps=WDRP-020] [scope=docs/workstreams]
  Goal: Route WMLP closeout follow-ons for browser playback session identity, library browse, catalog sort/filter, and desktop native playback into explicit workstream decisions.
  Validation: follow-on planning references the WMLP closeout evidence and names whether each item is a new lane, existing lane task, or deferred trigger.
  Evidence: Public Client follow-on plan.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

- [ ] WDRP-070 [owner=planner] [deps=WDRP-020,WDRP-030,WDRP-040,WDRP-050,WDRP-060,WDRP-065] [scope=docs/workstreams/web-deferred-product-reentry-plan]
  Goal: Close this planning lane after follow-on implementation lanes are opened or explicitly deferred.
  Validation: `python -m json.tool docs/workstreams/web-deferred-product-reentry-plan/WORKSTREAM.json`; `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Close or return to the selected implementation lane.
