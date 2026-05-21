# Android arm64 UniFFI Release Smoke — Evidence And Gates

Status: Closed
Last updated: 2026-05-21

## Evidence Anchors

- `docs/workstreams/android-uniffi-native-smoke/CLOSEOUT.md`
- `apps/android/app/src/androidTest/java/dev/taru/android/uniffi/TaruUniFfiNativeSmokeTest.kt`
- `apps/android/app/build.gradle.kts`
- `apps/android/app/build/outputs/apk/debug/app-debug.apk`

## Planning Gates

```powershell
python -m json.tool docs/workstreams/android-arm64-uniffi-release-smoke/WORKSTREAM.json > $null
git diff --check
```

## Task Gates

### A64-020

```powershell
adb devices -l
adb shell getprop ro.product.cpu.abi
adb shell getprop ro.product.cpu.abilist
```

### A64-030 Preferred Runtime Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PtaruRustAndroidAbis=arm64-v8a --no-daemon
```

### A64-030 Fallback Packaging Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=arm64-v8a --no-daemon
# Inspect app-debug.apk for lib/arm64-v8a/libtaru_client_uniffi.so and lib/arm64-v8a/libjnidispatch.so.
```

## Evidence Log

- 2026-05-21: Opened `A64-010`.
  - Device probe showed one attached emulator: `emulator-5554`, model
    `sdk_gphone64_x86_64`.
  - `adb shell getprop ro.product.cpu.abi` returned `x86_64`.
  - `adb shell getprop ro.product.cpu.abilist` returned `x86_64,arm64-v8a`.
  - This is not arm64 runtime evidence; it only indicates an x86_64 emulator
    with arm64 listed in ABI support.
- 2026-05-21: Completed `A64-020` arm64 device detection.
  - No attached primary arm64 runtime device was available.
  - Chose fallback `arm64-v8a` APK/JNI packaging verification.
- 2026-05-21: Completed `A64-030` arm64 packaging verification.
  - First APK inspection showed Taru's `libtaru_client_uniffi.so` was focused
    to arm64, but transitive JNI libraries from JNA/AndroidX were still present
    for non-arm64 ABIs.
  - Added Android `ndk.abiFilters` from `taruRustAndroidAbis` so focused ABI
    selection applies to all packaged native libraries, not only Taru's Rust
    output.
  - `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=arm64-v8a --no-daemon`
    passed.
  - APK inspection passed with:
    - `lib/arm64-v8a/libandroidx.graphics.path.so`
    - `lib/arm64-v8a/libjnidispatch.so`
    - `lib/arm64-v8a/libtaru_client_uniffi.so`
  - APK inspection found no non-arm64 JNI entries.
- 2026-05-21: Completed `A64-090` closeout.
  - `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`
    passed.
  - `python -m json.tool docs/workstreams/android-arm64-uniffi-release-smoke/WORKSTREAM.json > $null`
    passed.
  - `git diff --check` passed.
