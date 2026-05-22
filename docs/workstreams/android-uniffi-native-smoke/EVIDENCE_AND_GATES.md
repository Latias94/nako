# Android UniFFI Native Smoke — Evidence And Gates

Status: Closed
Last updated: 2026-05-21

## Evidence Anchors

- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `docs/workstreams/android-rust-core-runtime-hardening/CLOSEOUT.md`
- `apps/android/app/build.gradle.kts`
- `apps/android/app/src/androidTest/java/dev/nako/android/uniffi`
- `crates/nako-client-uniffi/src/lib.rs`
- `crates/nako-client-core/src/lib.rs`

## Planning Gates

```powershell
python -m json.tool docs/workstreams/android-uniffi-native-smoke/WORKSTREAM.json > $null
git diff --check
```

## Task Gates

### UNS-020

```powershell
apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
```

### UNS-030

```powershell
apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
```

## Closeout Gates

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
python -m json.tool docs/workstreams/android-uniffi-native-smoke/WORKSTREAM.json > $null
git diff --check
```

## Evidence Log

- 2026-05-21: Opened `UNS-010`.
  - Created a focused lane for Android packaged UniFFI native library runtime
    smoke validation.
- 2026-05-21: Completed `UNS-020` instrumentation smoke.
  - Added AndroidX instrumentation runner/dependencies.
  - Added `NakoUniFfiNativeSmokeTest`, which calls
    `buildPlaybackDecisionRequest` through generated UniFFI bindings.
  - Switched Android runtime JNA packaging to the AAR artifact so Android
    receives `libjnidispatch.so`.
  - Added a host JNA dispatch extraction task so JVM tests keep their host
    `jnidispatch` resource after runtime JNA packaging moved to AAR.
  - `apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon`
    passed.
- 2026-05-21: Completed `UNS-030` device/emulator execution.
  - Initial connected run failed on
    `Pixel_3a_API_34_extension_level_7_x86_64(AVD) - 14` with
    `UnsatisfiedLinkError: Native library (com/sun/jna/android-x86-64/libjnidispatch.so) not found`.
  - After using the JNA AAR artifact and preserving host resources,
    `apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon`
    passed on `Pixel_3a_API_34_extension_level_7_x86_64(AVD) - 14`.
- 2026-05-21: Completed `UNS-090` closeout.
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon`
    passed.
  - `apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon`
    passed.
  - `apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon`
    passed.
  - `python -m json.tool docs/workstreams/android-uniffi-native-smoke/WORKSTREAM.json > $null`
    passed.
  - `git diff --check` passed.
