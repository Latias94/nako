# Android Client Follow-On Hardening — Evidence And Gates

Status: Complete
Last updated: 2026-05-29

## Gate Set

### Smoke / Local Validation

```powershell
apps\android\scripts\Validate-AndroidLocal.ps1
apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
apps\android\scripts\Smoke-Regression.ps1
adb devices
```

### Android JVM / Build

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel --console=plain
apps\android\gradlew.bat -p apps\android :app:assembleDebug -PnakoRustAndroidAbis=x86_64 --no-daemon --no-parallel --console=plain
```

### Focused Gates

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.connection.* --no-daemon --no-parallel --console=plain
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel --console=plain
apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.nako.android.connection.*" --tests "dev.nako.android.ui.screens.player.*"
```

### Hygiene

```powershell
git diff --check
python -m json.tool docs/workstreams/android-client-follow-on-hardening/WORKSTREAM.json > $null
```

## Evidence Log

### ACFH-010 — Lane Open

Status: DONE
Date: 2026-05-22
Evidence:

- Created `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
  `HANDOFF.md`, and `WORKSTREAM.json`.
- The lane follows `android-client-architecture-deepening/CLOSEOUT.md` and does
  not reopen that closed workstream.

### ACFH-020 — Device Or Emulator Smoke Evidence

Status: DONE_WITH_CONCERNS
Date: 2026-05-22
Evidence:

- `adb devices` initially found `emulator-5554` in `device` state.
- `apps\android\scripts\Validate-AndroidLocal.ps1 -Serial emulator-5554`
  returned FAIL. Evidence:
  `apps/android/build/validation/20260522-161437/report.md`.
  - Android JVM tests: PASS.
  - Android debug assemble: PASS.
  - Android smoke regression: FAIL.
- Delegated smoke report:
  `apps/android/build/smoke-regression/20260522-161747/report.md`.
  - `empty-setup`: PASS.
  - `profile-missing-token`: PASS.
  - `profile-with-media`: FAIL after an Android ANR dialog interfered with
    surface capture.
- Focused `profile-with-media` smoke attempt timed out, but retained partial
  evidence under
  `apps/android/build/smoke/20260522-164355-profile-with-media-emulator-5554/`.
- After restarting the emulator and using `-AdbServerPort 5038`, this state
  passed:
  `apps\android\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup -SkipAppBuild -AdbServerPort 5038 -Serial 127.0.0.1:5555`.
  Report:
  `apps/android/build/smoke/20260522-174816-empty-setup-127.0.0.1-5555/report.md`.
- Later local validation reruns with `127.0.0.1:5555` and ADB port `5038`
  failed because the local ADB/Windows socket environment could not reliably
  start or connect to the daemon. Evidence:
  `apps/android/build/validation/20260522-175027/report.md`,
  `apps/android/build/smoke-regression/20260522-175027/report.md`,
  `apps/android/build/validation/20260522-180048/report.md`, and
  `apps/android/build/smoke-regression/20260522-180052/report.md`.
- Script hardening added:
  - `-AdbServerPort` support through `Validate-AndroidLocal.ps1`,
    `Smoke-Regression.ps1`, and `Smoke-Emulator.ps1`.
  - fresh-evidence directory selection in smoke regression reruns.
  - ANR dialog dismissal during UI dump collection.
  - serial-safe smoke output directory naming.
- Fresh parser gate passed:
  `PowerShell smoke scripts parse successfully.`

Residual risk:

- End-to-end smoke is not claimed PASS. The lane records smoke as
  environment-blocked with partial PASS evidence and rerunnable diagnostics.

### ACFH-030 — TokenVault Migration

Status: DONE
Date: 2026-05-22
Evidence:

- Replaced `AndroidSecureTokenVault` implementation:
  - removed deprecated AndroidX Security `EncryptedSharedPreferences`,
    `MasterKey`, and `@file:Suppress("DEPRECATION")`;
  - removed `androidx.security:security-crypto` from the version catalog and
    Android app dependencies;
  - introduced Android Keystore AES-GCM record encryption for new installs;
  - hashes token references before using them as SharedPreferences keys;
  - purges invalid or undecryptable records instead of surfacing partial token
    state;
  - added `TokenVaultMigrationSource` read-through seam so compatible legacy
    providers can migrate into the new vault without retaining deprecated
    AndroidX Security dependencies in the default app build.
- Added `SharedPreferencesTokenVaultTest`.
- Fresh grep gate passed:

```powershell
git grep -n -e 'androidx.security.crypto' -e 'EncryptedSharedPreferences' -e 'MasterKey' -e 'Suppress("DEPRECATION")' -- apps/android ':(exclude)apps/android/app/build' ':(exclude)apps/android/build' ':(exclude)**/.gradle'
```

Result: no deprecated token-vault crypto references in tracked Android source.

- Fresh focused gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.nako.android.connection.*" --tests "dev.nako.android.ui.screens.player.*"
```

Result: BUILD SUCCESSFUL in 25s.

### ACFH-040 — PlayerRuntime Platform Capability Slice

Status: DONE
Date: 2026-05-22
Evidence:

- Added `PlayerPlatformSessionFactory` / `PlayerPlatformSession` seam owned by
  `PlaybackSessionRuntime`.
- Added Android framework `MediaSession` implementation in
  `AndroidMediaSessionPlayerPlatformSession.kt` without introducing a new Maven
  dependency.
- `PlayerRouteHost` now creates the platform session on attach, publishes
  playback-state changes to it, and releases it exactly once on dispose.
- Added PiP capability slice:
  - manifest declares `android:supportsPictureInPicture="true"` and
    `android:resizeableActivity="true"`;
  - `PlaybackPictureInPictureGateway` safely unwraps an `Activity` from Compose
    context and enters PiP only when API level and activity are available;
  - player top overlay exposes an "Enter picture-in-picture" entrypoint when
    supported.
- Added/updated focused player tests:
  - `PlayerRouteHostTest`;
  - `PlaybackSessionRuntimeTest`;
  - `PlaybackPictureInPictureTest`.
- Fresh focused gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.nako.android.connection.*" --tests "dev.nako.android.ui.screens.player.*"
```

Result: BUILD SUCCESSFUL in 25s.

### Broad Gates — 2026-05-22

Status: PASS_WITH_SMOKE_CONCERN
Evidence:

- Full Android JVM gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:testDebugUnitTest
```

Result: BUILD SUCCESSFUL in 13s.

- x86_64 debug assemble gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --offline --no-daemon --no-parallel --console=plain :app:assembleDebug -PnakoRustAndroidAbis=x86_64
```

Result: BUILD SUCCESSFUL in 1m 38s.

- Hygiene gate passed:

```powershell
git diff --check
```

Result: exit code 0. Git emitted CRLF normalization warnings only.

### ACFH-090 — Closeout And Public SDK Contract Alignment

Status: DONE
Date: 2026-05-29
Evidence:

- During closeout the focused Android JVM gate initially exposed public SDK
  contract drift in the playback adapter. Fixed the Android boundary by:
  - mapping `ClientPlaybackDecisionReason` through `wireValue`;
  - representing `denied` playback decisions and denial details explicitly;
  - removing the legacy client transcode hardware-acceleration field from the
    Android playback model because the public `ClientTranscodePlan` no longer
    exposes it.
- Refreshed Android public API fixtures to the current SDK contract:
  - playback decision responses now include `denial`, `report`, and `target`;
  - media stream fixtures include `disposition` and `origin`;
  - library metadata profile fixtures include `scan`.
- Offline focused gate was not a valid final signal on this machine because the
  Gradle plugin was not present in the offline cache. The online gate below is
  the accepted final evidence.

Fresh focused JVM gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:testDebugUnitTest --tests "dev.nako.android.connection.*" --tests "dev.nako.android.ui.screens.player.*"
```

Result: BUILD SUCCESSFUL in 1m 57s.

Fresh full Android JVM gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:testDebugUnitTest
```

Result: BUILD SUCCESSFUL in 41s.

Fresh x86_64 debug assemble gate passed:

```powershell
apps\android\gradlew.bat -p apps\android --no-daemon --no-parallel --console=plain :app:assembleDebug -PnakoRustAndroidAbis=x86_64
```

Result: BUILD SUCCESSFUL in 24s. The first x86_64 assemble attempt in this
closeout failed because the local Rust toolchain was missing the
`x86_64-linux-android` target; after installing it, assemble passed. Android
packaging emitted non-blocking strip warnings for native debug symbols on the
first passing run.

Fresh hygiene gates passed:

```powershell
python -m json.tool docs\workstreams\android-client-follow-on-hardening\WORKSTREAM.json > $null
git diff --check
```

Result: both exited 0. Git emitted CRLF normalization warnings only.

Residual risk:

- End-to-end smoke remains the ACFH-020 environment-blocked concern and is not
  claimed PASS.
