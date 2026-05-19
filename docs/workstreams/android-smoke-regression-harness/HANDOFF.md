# Android Smoke Regression Harness - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

This lane is closed. `ASR-010`, `ASR-020`, `ASR-030`, and `ASR-040` are
complete.

`ASR-020` added `apps/android/scripts/Smoke-Regression.ps1`, a local wrapper
that builds once by default, runs selected stable fixture states through
`Smoke-Emulator.ps1`, and writes a combined report under
`apps/android/build/smoke-regression/<timestamp>/`.

`ASR-040` verified the final gate and closed the local harness scope.

## Closed Task

- Task ID: ASR-040
- Owner: planner
- Files: `docs/workstreams/android-smoke-regression-harness`
- Validation: fresh local regression command, Android unit/build gates if
  touched behavior warrants them, and `git diff --check`.
- Status: DONE
- Review: No blocking workstream compliance or code-quality findings remained
  after final verification.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Use a thin wrapper instead of duplicating ADB or UI navigation code.
- Default stable regression states are `empty-setup`, `profile-missing-token`,
  and `profile-with-media`.
- Keep `Smoke-Emulator.ps1` as the authoritative single-state harness.
- Use `/data/local/tmp` for UI hierarchy dumps instead of `/sdcard`, because
  automated `uiautomator dump` had transient remote-file visibility failures.
- Retry each state once by default in `Smoke-Regression.ps1`; use
  `-RetriesPerState 0` when a stricter no-retry signal is needed.
- Regression reports now include failure category, state attempts, evidence
  path, log path, not-run reason, and a focused single-state rerun command.
- `Smoke-Emulator.ps1` now waits for the Taru app window to be focused before
  UI hierarchy capture, and recovers focus by waking the device and relaunching
  MainActivity before retrying.
- `profile-with-media` debug profile seeding now uses a debug-only
  `ContentProvider.call` path instead of launching
  `DebugSmokeFixtureSeedActivity`. This avoids Activity/window-stack stalls in
  the seed phase while keeping fixture setup inside debug APK code.
- `Smoke-Emulator.ps1` detects Android's `Process system isn't responding`
  dialog, taps `Wait`, recovers app focus, and continues waiting for the target
  UI text. Taru app ANR/crash failures are still treated as failures.
- Do not rewrite the harness in Python under the current local workflow. Python
  is only worth revisiting for CI/device-farm execution, cross-platform
  packaging, structured JSON/JUnit report export, or complex parallelism.
- Keep CI, golden screenshots, and deeper playback validation as follow-ons.

## Blockers

- None known. A local emulator and Android toolchain are expected for full
  validation.

## Next Recommended Action

- Start a new Android development lane. The recommended next scope is a
  developer-friendly test entry point for Android Material Expressive work:
  either a documented smoke/dev workflow around the existing harness, or a
  focused UI implementation task from
  `docs/workstreams/android-material-expressive-ui/`.

## Latest Evidence

Fresh validation passed on 2026-05-19:

- `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup -SkipAppBuild`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token -SkipBuild`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media`
- `git diff --check`

Final report:

- `apps/android/build/smoke-regression/20260519-005118/report.md`

ASR-030 validation passed on 2026-05-19:

- Controlled failure:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup -Serial not-a-device -SkipBuild -RetriesPerState 0`
- Successful local regression:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token -SkipBuild`
- `git diff --check`

ASR-030 reports:

- `apps/android/build/smoke-regression/20260519-080808/report.md`
- `apps/android/build/smoke-regression/20260519-081559/report.md`

ASR-040 closeout validation passed on 2026-05-19:

- `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null"`
- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media`
- `git diff --check`

Final ASR-040 report:

- `apps/android/build/smoke-regression/20260519-093524/report.md`

## Residual Risks And Follow-ons

- CI/device-farm execution is still out of scope for this lane.
- Golden screenshot or visual diff assertions remain out of scope.
- JSON/JUnit report export and a Python rewrite are only worth opening if the
  harness needs cross-platform CI packaging or more complex orchestration.
- Deeper playback validation, session persistence, and stream-state semantics
  belong in a separate playback-focused lane.
