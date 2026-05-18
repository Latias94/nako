# Taru Android

This is the Taru Android client app. It is intentionally kept outside the Rust
Cargo workspace.

## Scope

Current focus: the Android client foundation, V2 UI surfaces, and local QA
harness.

Included:

- single-module Android app under `apps/android`;
- Kotlin, Compose, and Material 3;
- dark-first local debug shell aligned with Design Language v0;
- local token, spacing, type, shape, poster, backdrop, and touch-target roles;
- Gradle Wrapper for local builds;
- server URL plus access-token setup shell;
- `GET /health` preflight and lightweight authenticated Public Client API
  probe;
- API-version and public error-envelope parsing;
- browse, search, detail, source picker, settings, and player UI surfaces;
- playback decision request construction and Media3 player launch smoke;
- multiple server profiles with one active profile;
- Android secure token vault with profile records storing token references only;
- local emulator smoke command for install, launch, and screenshot evidence.

Still out of scope for this Android app baseline:

- UniFFI or shared Rust mobile core;
- downloads/offline playback;
- external player handoff;
- CI device-farm or golden screenshot infrastructure.

## Prerequisites

- JDK 21.
- Android SDK with platform `android-36`.
- Android build tools available from the configured SDK.

The Gradle project uses its own wrapper. Do not add `apps/android` to the Rust
Cargo workspace.

## Commands

From `apps/android`:

```powershell
.\gradlew.bat :app:assembleDebug
.\gradlew.bat :app:testDebugUnitTest
.\scripts\Smoke-Emulator.ps1
```

From the repository root:

```powershell
cargo check --workspace --tests
git diff --check
```

## Local Smoke

Use the smoke script when you want a repeatable emulator sanity check after a
build:

```powershell
.\scripts\Smoke-Emulator.ps1
.\scripts\Smoke-Emulator.ps1 -Serial emulator-5554
.\scripts\Smoke-Emulator.ps1 -SkipBuild
.\scripts\Smoke-Emulator.ps1 -ResetAppData
```

The script builds `:app:assembleDebug` by default, installs the debug APK to a
connected emulator, launches `dev.taru.android/.MainActivity`, and writes
evidence under `apps/android/build/smoke/<timestamp>-<state>-<serial>/`.

The script expects `adb devices` to show exactly one device in `device` state.
If multiple devices are attached, pass `-Serial`. If no devices are attached,
start an emulator first and re-run the command.

Use `-ResetAppData` for a deterministic empty setup state. It clears
`dev.taru.android` app data after installing the debug APK, which removes
stored server profiles and encrypted access tokens before launch.

Fixture and state rules live in `SMOKE_FIXTURES.md`.
