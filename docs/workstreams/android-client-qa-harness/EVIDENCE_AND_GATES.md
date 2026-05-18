# Android Client QA Harness — Evidence And Gates

Status: Completed
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-missing-token
git diff --check
```

## Gate Set

### Android JVM Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

This proves client DTO, presentation, and token/locator safety tests still pass.

### Android Build Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
```

This proves the debug APK can be built before emulator smoke checks.

### Emulator Smoke Gate

The lane should add a local command that installs and launches the debug APK on
an already running emulator, then captures named screenshots or a structured
report into an untracked output path.

This proves the app can start and expose key surfaces outside JVM tests.

### Diff Hygiene Gate

```powershell
git diff --check
```

This catches whitespace errors and unresolved patch artifacts.

## Evidence Anchors

- `apps/android/README.md`
- `apps/android/SMOKE_FIXTURES.md`
- `apps/android/scripts/`
- `docs/workstreams/android-client-qa-harness/TODO.md`
- `docs/workstreams/android-client-qa-harness/HANDOFF.md`

## Evidence Log

- 2026-05-18: Workstream opened after Android Client Foundation and Android
  Material Expressive UI closeout. `ACQ-010` completed with scope, target
  state, gate set, and first executable task recorded.
- 2026-05-18: `ACQ-020` implementation started. Added
  `apps/android/scripts/Smoke-Emulator.ps1` and updated `apps/android/README.md`
  with local smoke usage. Fresh validation passed for
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
  and `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`.
  Smoke validation is blocked until an emulator is visible to `adb`: `adb
  devices -l` returned no connected devices; a headless start attempt for AVD
  `Pixel_3a_API_34_extension_level_7_x86_64` did not reach `device` state within
  five minutes and was stopped. Re-run
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1` after `adb
  devices` shows a device in `device` state.
- 2026-05-18: `ACQ-020` completed. Fresh validation passed for
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`,
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`,
  and `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1`. The
  smoke command installed the debug APK, launched `dev.taru.android/.MainActivity`,
  captured `launch.png`, and wrote `launch.txt` plus `report.md` under
  `apps/android/build/smoke/20260518-204538-emulator-5554/`.
- 2026-05-18: `ACQ-030` completed fixture/state strategy. Added
  `apps/android/SMOKE_FIXTURES.md` and `-ResetAppData` support to
  `apps/android/scripts/Smoke-Emulator.ps1`. Fresh `empty-setup` smoke passed:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -ResetAppData`
  installed the debug APK, cleared app data, force-stopped the app, launched
  `dev.taru.android/.MainActivity`, captured `launch.png`, and wrote evidence
  under `apps/android/build/smoke/20260518-205953-empty-setup-emulator-5554/`.
  Device inspection after launch showed no `taru_server_profiles.xml`; the
  encrypted token preferences file existed with AndroidX Security keysets only,
  not app token entries.
- 2026-05-18: `ACQ-040` completed first emulator surface coverage. The smoke
  script now captures named screenshots, UI hierarchy dumps, and criteria files
  for `empty-setup` and `profile-missing-token`. Fresh `empty-setup` smoke
  passed at
  `apps/android/build/smoke/20260518-214452-empty-setup-emulator-5554/` with
  `setup.png`, `setup.uiautomator.xml`, and `setup.criteria.txt`. Fresh
  `profile-missing-token` smoke passed at
  `apps/android/build/smoke/20260518-214533-profile-missing-token-emulator-5554/`
  with `home.png`, `settings.png`, `server-profile.png`, matching UI hierarchy
  dumps, and matching criteria files. Criteria results were PASS for all
  required setup, Home, Settings, and Server Profile text/content descriptions.
  Evidence reports and criteria files were checked for token-reference values,
  bearer tokens, localhost, and `10.0.2.2`; none were present.
- 2026-05-18: `ACQ-050` closeout completed. Fresh gates passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`,
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`,
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup`,
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-missing-token`,
  and `git diff --check`. The final `empty-setup` smoke wrote
  `setup.png`, `setup.uiautomator.xml`, and `setup.criteria.txt` under
  `apps/android/build/smoke/20260518-215542-empty-setup-emulator-5554/`.
  The final `profile-missing-token` smoke wrote `home.png`, `settings.png`,
  `server-profile.png`, matching UI hierarchy dumps, and matching criteria
  files under
  `apps/android/build/smoke/20260518-215751-profile-missing-token-emulator-5554/`.
  All criteria files reported `Result: PASS`. A first closeout attempt for
  `profile-missing-token` hit a transient ADB daemon reconnect failure during
  `wm dismiss-keyguard`; after `adb start-server`, the same smoke command
  passed.
