# Android UniFFI Native Smoke — TODO

Status: Active
Last updated: 2026-05-21

## M0 — Lane Open

- [x] UNS-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-uniffi-native-smoke]
  Goal: Open a focused lane for proving Android packaged UniFFI native library
  loading on device/emulator.
  Validation: `python -m json.tool docs/workstreams/android-uniffi-native-smoke/WORKSTREAM.json > $null`; `git diff --check`
  Review: Confirm this lane is runtime native smoke only, not playback E2E.
  Evidence: `docs/workstreams/android-uniffi-native-smoke/DESIGN.md`
  Handoff: DONE. Implement `UNS-020` next.

## M1 — Instrumentation Smoke

- [x] UNS-020 [owner=codex] [deps=UNS-010] [scope=apps/android/app/build.gradle.kts,apps/android/app/src/androidTest]
  Goal: Add a minimal Android instrumentation test that calls a deterministic
  UniFFI function backed by the packaged Android native library.
  Validation: `apps/android/gradlew.bat -p apps/android :app:assembleDebug :app:assembleDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon`
  Review: The test must not use host-library overrides, network, server state,
  Media3, or profile/token persistence.
  Evidence: `apps/android/app/src/androidTest/java/dev/nako/android/uniffi/NakoUniFfiNativeSmokeTest.kt`
  Handoff: DONE. Added AndroidX instrumentation dependencies, switched Android
  runtime JNA packaging to the AAR artifact, preserved JVM host JNA resources,
  and added a deterministic UniFFI request-builder smoke.

## M2 — Device/Emulator Execution

- [x] UNS-030 [owner=codex] [deps=UNS-020] [scope=apps/android,docs/workstreams/android-uniffi-native-smoke]
  Goal: Run the instrumentation smoke on an attached device/emulator when
  available; otherwise record the exact blocker and keep build evidence.
  Validation: `apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=x86_64 --no-daemon`
  Review: Do not claim real Android runtime verification unless the connected
  instrumentation command passes.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. The first connected run exposed missing Android JNA
  `libjnidispatch.so`; after switching runtime packaging to the JNA AAR and
  preserving host resources for JVM tests, connected instrumentation passed on
  `Pixel_3a_API_34_extension_level_7_x86_64(AVD) - 14`.

## M3 — Closeout

- [x] UNS-090 [owner=planner] [deps=UNS-030] [scope=docs/workstreams/android-uniffi-native-smoke]
  Goal: Close the lane if device/emulator execution passed, or mark it blocked
  with a precise external dependency if no device/emulator is available.
  Validation: `python -m json.tool docs/workstreams/android-uniffi-native-smoke/WORKSTREAM.json > $null`; `git diff --check`
  Review: Final status must distinguish buildable smoke from executed runtime
  smoke.
  Evidence: `docs/workstreams/android-uniffi-native-smoke/CLOSEOUT.md`
  Handoff: DONE. Runtime native smoke passed on emulator and the lane is closed.
