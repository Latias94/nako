# Android arm64 UniFFI Release Smoke

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

`android-uniffi-native-smoke` proved the packaged UniFFI library on an x86_64
emulator. Release risk is still highest on `arm64-v8a`, because real Android
phones are predominantly arm64 and JNI packaging/linking mistakes can be
ABI-specific.

## Target State

- Prefer running the existing UniFFI instrumentation smoke on an attached
  `arm64-v8a` device.
- If no arm64 device is attached, verify the `arm64-v8a` debug APK packaging
  contains both Taru's UniFFI library and JNA's Android dispatch library.
- Keep the evidence boundary explicit: packaging verification is not runtime
  execution on arm64.

## Non-Goals

- No playback E2E or Media3 launch validation.
- No server fixture.
- No broad ABI matrix.
- No release signing or Play distribution work.
- No Rust-owned Android networking.

## Architecture Direction

This lane should not add another test when the existing
`TaruUniFfiNativeSmokeTest` already exercises the right runtime seam. The only
new code, if needed, should be validation documentation or a tiny reusable
packaging inspection script/task. Prefer Gradle/APK evidence over manual claims.

## Guardrails

- Do not claim arm64 runtime smoke unless an attached arm64 device executes the
  instrumentation test successfully.
- If only x86_64 emulator is present, run arm64 packaging verification and
  record the runtime gap.
- Keep focused ABI selection with `-PtaruRustAndroidAbis=arm64-v8a`.
