# Android Smoke Regression Harness - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup,profile-missing-token
```

This proves the wrapper can build once, run multiple stable fixture states, and
write a combined summary without requiring the server-backed media fixture.

## Gate Set

### Targeted Iteration Gate

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup,profile-missing-token -SkipBuild
```

Use after `:app:assembleDebug` has already produced a fresh debug APK.

### Full Local Regression Gate

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media
```

This is the default local Android confidence gate for UI/client changes that
can affect setup, shell, settings, detail, source picker, or player launch.

### Android Build And Unit Gates

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
```

Run when Android app code, Gradle files, or debug fixture seeding behavior
changes. Script-only wrapper changes may use the full local regression gate as
the main behavioral proof.

### Diff Hygiene

```powershell
git diff --check
```

## Evidence Anchors

- `docs/workstreams/android-smoke-regression-harness/DESIGN.md`
- `docs/workstreams/android-smoke-regression-harness/TODO.md`
- `docs/workstreams/android-smoke-regression-harness/MILESTONES.md`
- `apps/android/scripts/Smoke-Regression.ps1`
- `apps/android/build/smoke-regression/<timestamp>/report.md`

## ASR-020 Evidence

Validated on 2026-05-19:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup -SkipAppBuild
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token -SkipBuild
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media
git diff --check
```

Final full regression report:

- `apps/android/build/smoke-regression/20260519-005118/report.md`

Final state evidence:

- `empty-setup`:
  `apps/android/build/smoke-regression/20260519-005118/states/empty-setup/20260519-005134-empty-setup-emulator-5554/`
- `profile-missing-token`:
  `apps/android/build/smoke-regression/20260519-005118/states/profile-missing-token/20260519-005336-profile-missing-token-emulator-5554/`
- `profile-with-media`:
  `apps/android/build/smoke-regression/20260519-005118/states/profile-with-media/20260519-005635-profile-with-media-emulator-5554/`

What this proves:

- `Smoke-Regression.ps1` builds the Android debug APK once by default, then
  runs selected fixture states through `Smoke-Emulator.ps1` with app-build
  reuse.
- The stable state set covers setup, missing-token shell/settings/profile, and
  server-backed Home/detail/source-picker/player surfaces.
- The report records state status, attempts, evidence directories, and per-state
  logs.
- UI hierarchy capture now writes through `/data/local/tmp`, cleans stale
  `uiautomator` processes between attempts, includes dump output in failures,
  and the wrapper retries each state once by default to absorb transient
  UiAutomation empty-root failures.

Notes:

- The full run built `:app:assembleDebug` successfully.
- `profile-with-media` built/prepared `nako-server`; Rust emitted pre-existing
  unused-code warnings in `nako-server`, but the fixture and smoke state passed.
- `git diff --check` passed with Git line-ending normalization warnings for
  edited Windows-tracked files only.

## ASR-030 Evidence

Validated on 2026-05-19:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup -Serial not-a-device -SkipBuild -RetriesPerState 0
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token -SkipBuild
git diff --check
```

Controlled failure report:

- `apps/android/build/smoke-regression/20260519-080808/report.md`

Successful local regression report:

- `apps/android/build/smoke-regression/20260519-081559/report.md`

What this proves:

- A device-selection failure is classified as `device-automation` and the
  report records the failed state, attempts, log path, and focused rerun
  command.
- State rows now include a category column; not-run states receive a reason
  such as `blocked-by-earlier-state` or `android-build`.
- The wrapper records Android build status separately from smoke state status.
- `Smoke-Emulator.ps1` waits for the Nako app window to be focused before
  `uiautomator dump`, and it recovers focus by waking the device and relaunching
  MainActivity before retrying a UI hierarchy capture.
- A two-state local regression passed after the focus recovery change:
  `empty-setup` and `profile-missing-token`, both in one attempt.

Decision:

- Do not rewrite the harness in Python under ASR-030. The current problem is
  Android/ADB orchestration and emulator focus recovery, not PowerShell string
  manipulation. Python remains a follow-on option for CI/device-farm,
  cross-platform packaging, structured JSON/JUnit export, or more complex
  concurrency.

## ASR-040 Closeout Evidence

Validated on 2026-05-19:

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null"
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media
git diff --check
```

Final full regression report:

- `apps/android/build/smoke-regression/20260519-093524/report.md`

Final state evidence:

- `empty-setup`:
  `apps/android/build/smoke-regression/20260519-093524/states/empty-setup/20260519-093541-empty-setup-emulator-5554/`
- `profile-missing-token`:
  `apps/android/build/smoke-regression/20260519-093524/states/profile-missing-token/20260519-093555-profile-missing-token-emulator-5554/`
- `profile-with-media`:
  `apps/android/build/smoke-regression/20260519-093524/states/profile-with-media/20260519-093628-profile-with-media-emulator-5554/`

What this proves:

- The default stable regression state set passed with `:app:assembleDebug`
  enabled and one attempt per state.
- Android debug unit tests pass after changing debug fixture seeding behavior.
- Debug server-backed profile seeding no longer depends on launching the seed
  Activity. The smoke script calls a debug-only `ContentProvider` and records
  provider status in `profile-with-media-seed.txt`.
- `Smoke-Emulator.ps1` tolerates Android system ANR wait dialogs by tapping
  `Wait` and retrying UI text capture, without suppressing Nako app failures.
- `git diff --check` passed with Git line-ending normalization warnings for
  edited Windows-tracked files only.

Closeout decision:

- Close this local harness lane. CI/device-farm execution, golden visual
  diffing, JSON/JUnit export, Python rewrite, and deeper playback/session
  validation remain follow-ons outside ASR-040.

## Notes

Do not commit generated screenshots, UI hierarchy dumps, fixture databases,
or regression reports by default. Evidence paths should be recorded here and in
handoff documents, but generated artifacts stay local.

Fresh verification is required before marking a task, Codex goal, or lane
complete.
