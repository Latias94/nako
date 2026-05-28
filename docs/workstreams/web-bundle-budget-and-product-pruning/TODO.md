# Web Bundle Budget And Product Pruning - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Activation

- [x] WBBP-010 [owner=planner] [deps=WALW-050] [scope=docs/workstreams/web-bundle-budget-and-product-pruning]
  Goal: Activate final bundle/pruning lane after Admin live wiring.
  Validation: WALW complete.
  Evidence: WALW-050 closeout commit `ee6d5cdc`; WORKSTREAM.json status active.
  Handoff: Next task is WBBP-020.

## M1 - Budget Instrumentation

- [x] WBBP-020 [owner=Codex] [deps=WBBP-010] [scope=web/package.json,web/scripts]
  Goal: Add repeatable bundle budget measurement and failure thresholds.
  Validation: npm --prefix web run build and budget script.
  Evidence: `npm --prefix web run build:budget` passed. Budget output: initial JS 442.74 KiB / 136.39 KiB gzip, admin route 212.59 / 45.67, media route 320.37 / 72.94, total JS 1148.54 / 312.15.
  Handoff: DONE. Next task is WBBP-030.

## M2 - Heavy Domain Pruning

- [ ] WBBP-030 [owner=Codex] [deps=WBBP-020] [scope=web/src/features,web/components/nako,web/package.json]
  Goal: Remove, quarantine, or lazy-load deferred domains not accepted as live product.
  Validation: npm --prefix web run test && npm --prefix web run build and budget script.
  Evidence: deleted/quarantined files and bundle diff.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Dependency Diet

- [ ] WBBP-040 [owner=Codex] [deps=WBBP-030] [scope=web/package.json,web/src]
  Goal: Remove unused heavy dependencies or move them behind accepted route boundaries.
  Validation: npm --prefix web run test && npm --prefix web run build and budget script.
  Evidence: package diff and bundle diff.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Final Frontend Closeout

- [ ] WBBP-050 [owner=planner] [deps=WBBP-040] [scope=docs/workstreams/web-bundle-budget-and-product-pruning]
  Goal: Close the final planned frontend refactor lane and summarize remaining non-runtime product work.
  Validation: npm --prefix web run test && npm --prefix web run check && npm --prefix web run build && npm --prefix web run tauri -- build.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Close the six-lane frontend refactor goal if all lanes are complete.
