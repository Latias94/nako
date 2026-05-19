# Android Detail Facet Smoke Evidence - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

- [x] ADF-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-detail-facet-smoke-evidence]
  Goal: Open the lane and freeze the first smoke target to API-backed detail
  facets: Genre, Tag, and Person.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-detail-facet-smoke-evidence/DESIGN.md`
  Handoff: Completed on 2026-05-19. First implementation slice should extend
  `profile-with-media` smoke without changing Public Client API or server
  schema.

## M1 - Detail Facet Navigation Smoke Slice

- [x] ADF-020 [owner=codex] [deps=ADF-010] [scope=apps/android/scripts,apps/android/SMOKE_FIXTURES.md,apps/android/README.md]
  Goal: Make `profile-with-media` smoke prove detail-page Genre, Tag, and
  Person navigation to server-backed facet result routes.
  Validation:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
  plus
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild`
  and `git diff --check`.
  Review: Confirm the test proves UI navigation through Public Client API
  relationships and does not claim unsupported families as implemented.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE on 2026-05-19. Reused existing `Night Harbor` metadata and
  extended `profile-with-media` smoke to prove `Mystery`, `Lighthouse`, and
  `Actor / as Keeper` open API-backed facet result routes that return `Night
  Harbor`. Latest focused report:
  `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/report.md`.

## M2 - Closeout

- [x] ADF-030 [owner=planner] [deps=ADF-020] [scope=docs/workstreams/android-detail-facet-smoke-evidence]
  Goal: Verify evidence, close this lane, and keep broader relationship
  families split.
  Validation: fresh ADF-020 gates and closeout doc updates.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: DONE on 2026-05-19. Lane is closed. Collection, Studio,
  Series/Hierarchy, Year, Item Kind, and richer role-specific navigation remain
  follow-ons.
