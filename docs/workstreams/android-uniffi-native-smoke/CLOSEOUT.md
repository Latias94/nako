# Android UniFFI Native Smoke — Closeout

Status: Closed
Closed on: 2026-05-21

## Outcome

This lane proved that the Android debug APK can load the packaged UniFFI native
library on a real Android runtime and call a Rust-owned request builder through
generated Kotlin bindings.

The smoke stays intentionally narrow:

- no network;
- no server fixture;
- no profile/token persistence;
- no UI or Media3;
- one deterministic UniFFI call into `nako-client-core` via
  `nako-client-uniffi`.

## Implementation Notes

- Added `NakoUniFfiNativeSmokeTest` under `androidTest`.
- Added AndroidX instrumentation runner/dependencies.
- Switched Android runtime JNA dependency resolution to the JNA AAR artifact so
  Android receives `libjnidispatch.so`.
- Added `extractHostJnaDispatch` so JVM tests still have the host JNA native
  dispatch resource after Android runtime packaging uses the AAR.

## Final Verification

Fresh gates run on 2026-05-21:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
python -m json.tool docs/workstreams/android-uniffi-native-smoke/WORKSTREAM.json > $null
git diff --check
```

All gates passed. The connected instrumentation smoke passed on
`Pixel_3a_API_34_extension_level_7_x86_64(AVD) - 14`.

## Finding Fixed In-Lane

The first connected run failed before Nako's Rust library could be exercised:

```text
UnsatisfiedLinkError: Native library (com/sun/jna/android-x86-64/libjnidispatch.so) not found
```

Root cause: Android runtime used the JNA jar shape, which contains host native
resources but not Android packaged JNI libs. The fix is to consume the JNA AAR
for Android runtime packaging and separately extract host JNA resources for JVM
unit tests.

## Residual Risks

- The smoke uses x86_64 emulator ABI evidence. Other Android ABIs still depend
  on the existing ABI build/packaging path and should be covered by release or
  CI matrix gates.
- The smoke proves UniFFI load and pure request-builder calls, not Media3
  playback launch or server-backed playback.

## Recommended Follow-ons

1. Add an optional CI/emulator lane for x86_64 connected smoke if CI supports
   Android emulators.
2. Add release-matrix ABI packaging verification for `arm64-v8a` before a
   real-device beta.
3. Keep playback launch E2E as a separate lane so this smoke remains fast and
   diagnostic.
