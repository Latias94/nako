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
.\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup
.\scripts\Smoke-Emulator.ps1 -FixtureState profile-missing-token -SkipBuild
.\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
```

The script builds `:app:assembleDebug` by default, installs the debug APK to a
connected emulator, launches `dev.taru.android/.MainActivity`, and writes
evidence under `apps/android/build/smoke/<timestamp>-<state>-<serial>/`.
Surface checks write named screenshots, UI hierarchy dumps, and criteria files
such as `setup.png`, `home.png`, `settings.png`, and
`server-profile.criteria.txt`.

The script expects `adb devices` to show exactly one device in `device` state.
If multiple devices are attached, pass `-Serial`. If no devices are attached,
start an emulator first and re-run the command.

Use `-ResetAppData` for a deterministic empty setup state. It clears
`dev.taru.android` app data after installing the debug APK, which removes
stored server profiles and encrypted access tokens before launch.

Use `-FixtureState profile-missing-token` when you need repeatable Home,
Settings, and Server Profile shell screenshots without a real server. It seeds
one local Server Profile with no token value, so Home intentionally shows the
safe re-authentication state instead of fake media data.

Use `-FixtureState profile-with-media` when you need repeatable Home, detail,
source picker, and player-safe launch evidence from real Public Client API
responses. The script prepares and starts the server-backed `Night Harbor`
fixture, applies `adb reverse`, and seeds the debug APK through its real profile
store and encrypted token vault. Generated screenshots and reports remain local
under `apps/android/build/smoke/`.

Fixture and state rules live in `SMOKE_FIXTURES.md`.

## Server-Backed Demo Fixture

Use the demo fixture server directly when Android work needs real Public Client
API media responses outside the full emulator smoke flow:

```powershell
.\scripts\Start-DemoFixtureServer.ps1 -PrepareOnly
.\scripts\Start-DemoFixtureServer.ps1 -AdbReverse
```

The script prepares a generated local Movies library with `Night Harbor`, runs
`taru-server scan` and `import-nfo`, and starts a loopback server at
`http://127.0.0.1:3018` unless `-PrepareOnly` is passed. Generated fixture data
is written under `apps/android/build/demo-fixtures/server-backed/` and should
not be committed.
