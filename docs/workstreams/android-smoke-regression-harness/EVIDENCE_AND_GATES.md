# Android Smoke Regression Harness - Evidence And Gates

Status: Active
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
- `profile-with-media` built/prepared `taru-server`; Rust emitted pre-existing
  unused-code warnings in `taru-server`, but the fixture and smoke state passed.
- `git diff --check` passed with Git line-ending normalization warnings for
  edited Windows-tracked files only.

## Notes

Do not commit generated screenshots, UI hierarchy dumps, fixture databases,
or regression reports by default. Evidence paths should be recorded here and in
handoff documents, but generated artifacts stay local.

Fresh verification is required before marking a task, Codex goal, or lane
complete.
