# Android Local Resume Smoke Evidence - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused debug fixture test:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.smoke.DebugSmokeFixtureSeedActivityTest --no-daemon`
- Focused smoke:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
- Diff hygiene:
  `git diff --check`

## Evidence Ledger

### ALR-010 - Boundary Freeze

- Evidence: `docs/workstreams/android-local-resume-smoke-evidence/DESIGN.md`
- Result: Complete.
- Notes: Lane is scoped to device-local Android smoke evidence. Public Client
  API changes, cross-device Continue Watching, and server-authoritative
  **User Playback State** remain out of scope.

### ALR-020 - Device-Local Resume Smoke Slice

- Evidence:
  - Focused debug fixture test:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.smoke.DebugSmokeFixtureSeedActivityTest --no-daemon`
  - Focused smoke report:
    `apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/report.md`
  - Regression smoke report:
    `apps/android/build/smoke-regression/20260519-102943/report.md`
  - Smoke criteria:
    `apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/source-picker-local-resume.criteria.txt`
    `apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/source-picker.criteria.txt`
    `apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/player.criteria.txt`
- Result: Complete.
- Notes: Smoke proved `Resume on this device`, local-only source picker copy,
  `Start resume`, player `Local resume 0:01`, and absence of `Continue
  Watching` / `User Playback State` fragments on the local resume surfaces. A
  three-state regression also passed for `empty-setup`, `profile-missing-token`,
  and `profile-with-media`.

### ALR-030 - Closeout

- Evidence: this document, `TODO.md`, `DESIGN.md`, `HANDOFF.md`, and
  `WORKSTREAM.json`.
- Result: Complete.
- Notes: CI/device-farm execution, golden screenshot diffing, and deeper
  playback duration/seek validation remain follow-ons.
