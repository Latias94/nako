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
- shared Rust client-core / UniFFI connection probe with Android-supplied
  transport;
- API-version and public error-envelope parsing;
- browse, search, detail, source picker, settings, and player UI surfaces;
- playback decision request construction and Media3 player launch smoke;
- multiple server profiles with one active profile;
- Android secure token vault with profile records storing token references only;
- local emulator smoke command for install, launch, and screenshot evidence.

Still out of scope for this Android app baseline:

- Rust-owned Android networking;
- downloads/offline playback;
- external player handoff;
- CI device-farm or golden screenshot infrastructure.

## Prerequisites

- JDK 21.
- Android SDK with platform `android-36`.
- Android build tools available from the configured SDK.
- Rust toolchain with the installed Android targets:
  `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`,
  and `x86_64-linux-android`.
- Android NDK. Set `ANDROID_NDK_HOME`, `NDK_HOME`, or Gradle property
  `android.ndk.home`.

The Gradle project uses its own wrapper. Do not add `apps/android` to the Rust
Cargo workspace.

## Rust / UniFFI Build Topology

The app uses three separate Rust/UniFFI artifacts:

1. host `taru-client-uniffi` shared library for JVM tests and binding
   generation;
2. generated Kotlin bindings under `app/build/generated/source/uniffi/`;
3. Android ABI `libtaru_client_uniffi.so` libraries under
   `app/build/generated/jniLibs/<variant>/`.

JVM unit tests depend only on the host library and generated Kotlin bindings.
APK packaging builds Android ABI libraries through the variant JNI merge path,
not through every ordinary `preBuild`.

By default Android packaging builds all supported ABIs. For focused local
iteration, pass a comma-separated ABI set:

```powershell
.\gradlew.bat :app:assembleDebug -PtaruRustAndroidAbis=x86_64
.\gradlew.bat :app:assembleDebug -PtaruRustAndroidAbis=arm64-v8a,x86_64
```

Supported ABI names are `arm64-v8a`, `armeabi-v7a`, `x86`, and `x86_64`.
Generated binding sources and JNI libraries live under `app/build/generated/`
and are not committed.

The connection flow uses the Rust core only for protocol-level request
construction, response interpretation, API-version checks, public error parsing,
and redaction-safe previews. Android still owns HTTP execution, cleartext/TLS
policy, token vaults, profile persistence, product diagnostics, UI, and Media3.

## Commands

From `apps/android`:

```powershell
.\gradlew.bat :app:assembleDebug
.\gradlew.bat :app:testDebugUnitTest
.\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
.\scripts\Smoke-Emulator.ps1
```

From the repository root:

```powershell
cargo check --workspace --tests
git diff --check
```

## App Icon

The Android launcher icons are generated platform resources under
`app/src/main/res/mipmap-*` and are referenced from `app/src/main/AndroidManifest.xml`.

The canonical product icon source asset lives at
[`../../assets/brand/taru-app-icon-1024.png`](../../assets/brand/taru-app-icon-1024.png).
Regenerate the Android launcher resources from that source asset when the product
icon changes.

## Local Smoke

Use the local validation entrypoint when you want one developer-facing handoff
command:

```powershell
.\scripts\Validate-AndroidLocal.ps1
.\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
.\scripts\Validate-AndroidLocal.ps1 -Serial emulator-5554
```

The default command runs Android JVM tests, assembles the debug APK, then
delegates the stable smoke state set to `Smoke-Regression.ps1`. Reports are
written under `apps/android/build/validation/<timestamp>/` as `report.md` for
human handoff, `report.json` for automation, and `report.junit.xml` for CI
test reporting. Use `-SkipSmoke` when no emulator is available and you only
need the local JVM/build gate.

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
.\scripts\Smoke-Emulator.ps1 -FixtureState profile-active-remux
.\scripts\Smoke-Regression.ps1
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
detail facet navigation, source picker, server-backed resume, Continue
Watching, Direct Play advancement, and player-exit server readback evidence
from real Public Client API responses. The script prepares and starts the
server-backed `Night Harbor` fixture, writes a User Playback State progress
record through the Public Client API, applies `adb reverse`, and seeds the debug
APK through its real profile store and encrypted token vault. Generated
screenshots, criteria files, and server readback artifacts remain local under
`apps/android/build/smoke/`. This fixture also creates a token-safe remux
session readback artifact with the public playback session header and
`/playback/sessions/{session_id}` route. The visible player still uses the
short MP4 Direct Play path.

Use `-FixtureState profile-active-remux` when you need a focused playback
session lifetime gate. It prepares a fresh MKV fixture, forces the debug
profile to choose Remux, starts the remux session only when playback begins,
exits the player before the slow remux wrapper completes, and records a
token-safe `/playback/sessions/{session_id}` readback artifact showing
terminal `cancelled` state.

Fixture and state rules live in `SMOKE_FIXTURES.md`.

Use the regression wrapper when you want the stable local Android confidence
gate before handing work to another developer or agent:

```powershell
.\scripts\Smoke-Regression.ps1
.\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token
.\scripts\Smoke-Regression.ps1 -States profile-active-remux
.\scripts\Smoke-Regression.ps1 -SkipBuild
.\scripts\Smoke-Regression.ps1 -RetriesPerState 0
```

The wrapper builds the debug APK once by default, then runs the selected smoke
fixture states through `Smoke-Emulator.ps1` and writes a combined report under
`apps/android/build/smoke-regression/<timestamp>/` as `report.md`,
`report.json`, and `report.junit.xml`. The default state set is `empty-setup`,
`profile-missing-token`, and `profile-with-media`. If a state fails, the report
includes the failed state, failure category, evidence path, log path, and a
focused `Smoke-Emulator.ps1` rerun command.

`profile-active-remux` is intentionally opt-in for regression because it starts
a slow server-side remux fixture and is heavier than the default confidence
set.

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
