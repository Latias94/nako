# Android Smoke Regression Harness - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane is open. `ASR-010`, `ASR-020`, and `ASR-030` are complete.

`ASR-020` added `apps/android/scripts/Smoke-Regression.ps1`, a local wrapper
that builds once by default, runs selected stable fixture states through
`Smoke-Emulator.ps1`, and writes a combined report under
`apps/android/build/smoke-regression/<timestamp>/`.

## Active Task

- Task ID: ASR-040
- Owner: planner
- Files: `docs/workstreams/android-smoke-regression-harness`
- Validation: fresh local regression command, Android unit/build gates if
  touched behavior warrants them, and `git diff --check`.
- Status: READY
- Review: Not run yet.
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
- Do not rewrite the harness in Python under the current local workflow. Python
  is only worth revisiting for CI/device-farm execution, cross-platform
  packaging, structured JSON/JUnit report export, or complex parallelism.
- Keep CI, golden screenshots, and deeper playback validation as follow-ons.

## Blockers

- None known. A local emulator and Android toolchain are expected for full
  validation.

## Next Recommended Action

- Execute `ASR-040`: close the lane if a fresh final gate set passes, or split
  CI, golden screenshot, or deeper playback validation into follow-on
  workstreams.

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
