# Android Client QA Harness — Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
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
- `apps/android/scripts/`
- `docs/workstreams/android-client-qa-harness/TODO.md`
- `docs/workstreams/android-client-qa-harness/HANDOFF.md`

## Evidence Log

- 2026-05-18: Workstream opened after Android Client Foundation and Android
  Material Expressive UI closeout. `ACQ-010` completed with scope, target
  state, gate set, and first executable task recorded.
