# Android Client QA Harness — Milestones

Status: Active
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

Exit criteria:

- Harness purpose is separated from product-feature work.
- Emulator, screenshot, fixture, and evidence boundaries are recorded.
- First executable task is selected.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 — Local Smoke Command

Exit criteria:

- A documented local command can build, install, launch, and capture basic
  evidence on an already running emulator.
- The command reports output paths and fails clearly when prerequisites are
  missing.
- Generated evidence is not committed by default.

Primary gates:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`
- local smoke command on an emulator
- `git diff --check`

Status: Completed on 2026-05-18.

Evidence:

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/README.md`
- `apps/android/build/smoke/20260518-204538-emulator-5554/`

## M2 — Fixture And State Strategy

Exit criteria:

- Setup/profile state assumptions are documented.
- Fixture data is token-safe and locator-safe.
- Unsupported server-backed state remains explicit instead of faked.

## M3 — Emulator Surface Coverage

Exit criteria:

- Setup or existing-profile launch is covered.
- Home, Settings, and Server Profile screenshots are captured with stable names.
- Failure modes are visible enough for another agent to continue.

## M4 — Closeout

Exit criteria:

- Harness gates pass fresh.
- Workstream docs reflect shipped harness behavior.
- CI, instrumentation, golden screenshots, server-backed demo data, and
  detail/player coverage follow-ons are split or explicitly deferred.
