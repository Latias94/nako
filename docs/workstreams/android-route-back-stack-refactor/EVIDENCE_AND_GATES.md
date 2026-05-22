# Android Route Back-Stack Refactor - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused route stack test:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.NakoRouteStackTest --no-daemon`
- Focused smoke:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
- Regression smoke:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild`
- Diff hygiene:
  `git diff --check`

## Evidence Ledger

### ARB-010 - Boundary Freeze

- Evidence: `docs/workstreams/android-route-back-stack-refactor/DESIGN.md`
- Result: Complete.
- Notes: Lane is scoped to Android browse shell route/back-stack behavior.

### ARB-020 - Route Stack Model And Shell Wiring

- Evidence:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.NakoRouteStackTest --no-daemon`
- Result: PASS on 2026-05-19.
- Notes: Added focused JVM coverage for route stack root behavior,
  Detail -> Facet -> Back -> Detail, Detail -> Player -> Back -> Detail,
  Settings -> Server Profile -> Back -> Settings, top-level destination
  selection clearing nested routes, and nested facet item Back behavior.

### ARB-030 - Smoke Return-Path Evidence

- Evidence:
  - `apps/android/build/smoke/20260519-112014-profile-with-media-emulator-5554/report.md`
  - `apps/android/build/smoke/20260519-112354-profile-missing-token-emulator-5554/report.md`
  - `apps/android/build/smoke-regression/20260519-112540/report.md`
- Result: PASS on 2026-05-19.
- Notes: `profile-with-media` now records `detail-after-facet-back` and
  `detail-after-player-back` surfaces. `profile-missing-token` now records
  `settings-after-profile-back`.

### ARB-040 - Closeout

- Evidence:
  - `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild`
  - `git diff --check`
- Result: PASS on 2026-05-19.
- Notes: Remaining deep links, route serialization, and process-death
  restoration stay as follow-ons outside this closed lane.
