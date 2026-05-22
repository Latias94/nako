# Android arm64 UniFFI Release Smoke — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane verified `arm64-v8a` APK/JNI packaging and then executed the Android
UniFFI runtime smoke on a real arm64 OPPO device.

## Device Evidence

```text
adb devices -l
emulator-5554 device product:sdk_gphone64_x86_64 model:sdk_gphone64_x86_64 device:emu64xa

adb shell getprop ro.product.cpu.abi
x86_64

adb shell getprop ro.product.cpu.abilist
x86_64,arm64-v8a
```

This initial probe was not arm64 runtime evidence. The device primary ABI was
`x86_64`, so the lane first used arm64 packaging verification.

After OPPO was connected:

```text
adb devices -l
3B15BC01DH500000 device product:PLG110 model:PLG110 device:OP5E11L1

adb -s 3B15BC01DH500000 shell getprop ro.product.cpu.abi
arm64-v8a

adb -s 3B15BC01DH500000 shell getprop ro.product.cpu.abilist
arm64-v8a

adb -s 3B15BC01DH500000 shell getprop ro.product.model
PLG110

adb -s 3B15BC01DH500000 shell getprop ro.build.version.release
16
```

## Packaging Evidence

Fresh gates run on 2026-05-21:

```powershell
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PnakoRustAndroidAbis=arm64-v8a --no-daemon
```

APK JNI inspection found:

```text
lib/arm64-v8a/libandroidx.graphics.path.so
lib/arm64-v8a/libjnidispatch.so
lib/arm64-v8a/libnako_client_uniffi.so
arm64 JNI packaging OK
```

No non-arm64 JNI entries were present after focused ABI selection was wired into
Android's `ndk.abiFilters`.

## arm64 Runtime Evidence

Fresh OPPO runtime smoke run on 2026-05-21:

```powershell
apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=arm64-v8a --no-daemon
adb -s 3B15BC01DH500000 install -r apps/android/app/build/outputs/apk/debug/app-debug.apk
adb -s 3B15BC01DH500000 install -r apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
adb -s 3B15BC01DH500000 shell am instrument -w -r -e class dev.nako.android.uniffi.NakoUniFfiNativeSmokeTest dev.nako.android.test/androidx.test.runner.AndroidJUnitRunner
```

Result:

```text
dev.nako.android.uniffi.NakoUniFfiNativeSmokeTest:
.
Time: 0.037
OK (1 test)
```

## Finding Fixed In-Lane

Initial arm64 APK inspection showed Nako's Rust library was focused to arm64,
but transitive native libraries from JNA/AndroidX still packaged other ABIs.
The fix was to apply `nakoRustAndroidAbis` to Android `ndk.abiFilters`, so the
focused ABI selector governs all native libraries in the APK.

## Final Verification

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug -PnakoRustAndroidAbis=arm64-v8a --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=arm64-v8a --no-daemon
adb -s 3B15BC01DH500000 shell am instrument -w -r -e class dev.nako.android.uniffi.NakoUniFfiNativeSmokeTest dev.nako.android.test/androidx.test.runner.AndroidJUnitRunner
python -m json.tool docs/workstreams/android-arm64-uniffi-release-smoke/WORKSTREAM.json > $null
git diff --check
```

All gates passed.

## Residual Risk

This lane now proves a single real arm64 device, not a device matrix. Broader
release confidence should still cover at least one additional Android vendor or
CI/device-farm target before public beta.

## Recommended Follow-on

Keep this command as the focused real-device regression:

```powershell
apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=arm64-v8a --no-daemon
adb -s <arm64-serial> install -r apps/android/app/build/outputs/apk/debug/app-debug.apk
adb -s <arm64-serial> install -r apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
adb -s <arm64-serial> shell am instrument -w -r -e class dev.nako.android.uniffi.NakoUniFfiNativeSmokeTest dev.nako.android.test/androidx.test.runner.AndroidJUnitRunner
```
