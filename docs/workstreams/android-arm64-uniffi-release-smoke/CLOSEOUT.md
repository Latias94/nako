# Android arm64 UniFFI Release Smoke — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane verified `arm64-v8a` APK/JNI packaging for the Android UniFFI runtime.
It did not execute the smoke on a primary arm64 runtime device because only an
x86_64 emulator was attached.

## Device Evidence

```text
adb devices -l
emulator-5554 device product:sdk_gphone64_x86_64 model:sdk_gphone64_x86_64 device:emu64xa

adb shell getprop ro.product.cpu.abi
x86_64

adb shell getprop ro.product.cpu.abilist
x86_64,arm64-v8a
```

This is not arm64 runtime evidence. The device primary ABI is `x86_64`.

## Packaging Evidence

Fresh gates run on 2026-05-21:

```powershell
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=arm64-v8a --no-daemon
```

APK JNI inspection found:

```text
lib/arm64-v8a/libandroidx.graphics.path.so
lib/arm64-v8a/libjnidispatch.so
lib/arm64-v8a/libtaru_client_uniffi.so
arm64 JNI packaging OK
```

No non-arm64 JNI entries were present after focused ABI selection was wired into
Android's `ndk.abiFilters`.

## Finding Fixed In-Lane

Initial arm64 APK inspection showed Taru's Rust library was focused to arm64,
but transitive native libraries from JNA/AndroidX still packaged other ABIs.
The fix was to apply `taruRustAndroidAbis` to Android `ndk.abiFilters`, so the
focused ABI selector governs all native libraries in the APK.

## Final Verification

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=arm64-v8a --no-daemon
python -m json.tool docs/workstreams/android-arm64-uniffi-release-smoke/WORKSTREAM.json > $null
git diff --check
```

All gates passed.

## Residual Risk

`arm64-v8a` runtime loading still needs a real arm64 device or arm64 emulator
whose primary ABI is arm64. This lane only proves packaging correctness for
arm64.

## Recommended Follow-on

When an arm64 device is available, run:

```powershell
apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PtaruRustAndroidAbis=arm64-v8a --no-daemon
```
