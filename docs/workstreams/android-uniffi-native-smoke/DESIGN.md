# Android UniFFI Native Smoke

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

`android-rust-core-runtime-hardening` proved the Rust client core boundary with
JVM tests and APK assembly, but it did not prove that the packaged Android
`libnako_client_uniffi.so` loads inside a real Android runtime. That is the
highest remaining risk after moving connection and playback request builders
behind UniFFI.

## Target State

- The Android app has a minimal instrumentation smoke test that loads the
  packaged UniFFI native library on device/emulator.
- The smoke calls a tiny, deterministic Rust core route builder through the
  generated Kotlin binding so symbol resolution, JNI loading, UniFFI scaffolding,
  and packaged ABI wiring are all exercised.
- Gradle has a clear validation path for building the ABI-specific APK and
  running the instrumentation smoke.
- JVM unit tests and ordinary APK assembly keep the split host/Android ABI
  behavior established by the previous lane.

## Non-Goals

- No Media3 playback launch verification in this lane.
- No server-backed end-to-end playback fixture.
- No Android profile/token persistence changes.
- No Rust-owned Android networking or socket/TLS policy.
- No broad UI automation suite.

## Architecture Direction

The smoke should intentionally be smaller than an end-to-end app test. It
should prove one thing: Android can load the packaged UniFFI library and call a
Rust-owned pure request builder through generated Kotlin bindings.

Use instrumentation tests rather than JVM tests because JVM tests use the host
library override, while this lane must exercise the Android packaged `.so`.

The test should use an ABI-focused Gradle path such as:

```powershell
apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon
```

When no emulator/device is attached, build and unit-level gates can still pass,
but closeout must clearly mark device runtime execution as unavailable rather
than pretending it was verified.

## Guardrails

- Keep this as a smoke test, not a product flow test.
- Do not add network, server, token vault, or Media3 dependencies to the smoke.
- Do not make JVM unit tests depend on Android ABI libraries.
- Do not require all Android ABIs for local smoke validation; use focused ABI
  selection unless a release gate explicitly asks for more.
