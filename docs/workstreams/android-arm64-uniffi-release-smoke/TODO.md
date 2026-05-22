# Android arm64 UniFFI Release Smoke — TODO

Status: Closed
Last updated: 2026-05-21

## M0 — Lane Open

- [x] A64-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-arm64-uniffi-release-smoke]
  Goal: Open a focused arm64 release-risk lane and record the device ABI
  evidence boundary.
  Validation: `python -m json.tool docs/workstreams/android-arm64-uniffi-release-smoke/WORKSTREAM.json > $null`; `git diff --check`
  Review: Confirm this lane is arm64 UniFFI risk only, not playback E2E.
  Evidence: `docs/workstreams/android-arm64-uniffi-release-smoke/DESIGN.md`
  Handoff: DONE. Continue with `A64-020`.

## M1 — arm64 Device Detection

- [x] A64-020 [owner=codex] [deps=A64-010] [scope=docs/workstreams/android-arm64-uniffi-release-smoke]
  Goal: Detect attached Android device ABI and choose runtime smoke or packaging
  verification.
  Validation: `adb devices -l`; `adb shell getprop ro.product.cpu.abi`; `adb shell getprop ro.product.cpu.abilist`
  Review: Do not treat an x86_64 emulator that lists arm64 translation support
  as an arm64 runtime device.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. The initial run only had `emulator-5554` with primary ABI
  `x86_64`, so the lane first fell back to arm64-v8a packaging verification.
  After an OPPO `PLG110` device was attached, primary ABI `arm64-v8a` was
  confirmed.

## M2 — arm64 Runtime Or Packaging Verification

- [x] A64-030 [owner=codex] [deps=A64-020] [scope=apps/android,docs/workstreams/android-arm64-uniffi-release-smoke]
  Goal: Run connected UniFFI smoke on arm64 if possible; otherwise assemble and
  inspect arm64-v8a APK JNI contents.
  Validation: preferred `apps/android/gradlew.bat -p apps/android :app:connectedDebugAndroidTest -PnakoRustAndroidAbis=arm64-v8a --no-daemon`; fallback `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PnakoRustAndroidAbis=arm64-v8a --no-daemon` plus APK JNI inspection.
  Review: Runtime and packaging evidence must be labeled separately.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE. `arm64-v8a` debug APK builds and contains only arm64 JNI
  entries, including `libnako_client_uniffi.so` and `libjnidispatch.so`.
  The same APK/test APK pair was then installed on OPPO `PLG110`, and
  `NakoUniFfiNativeSmokeTest` passed through Android instrumentation.

## M3 — Closeout

- [x] A64-090 [owner=planner] [deps=A64-030] [scope=docs/workstreams/android-arm64-uniffi-release-smoke]
  Goal: Close or explicitly boundary the lane with residual runtime risk.
  Validation: `python -m json.tool docs/workstreams/android-arm64-uniffi-release-smoke/WORKSTREAM.json > $null`; `git diff --check`
  Review: Final status must be honest about whether arm64 runtime was executed.
  Evidence: `docs/workstreams/android-arm64-uniffi-release-smoke/CLOSEOUT.md`
  Handoff: DONE. Lane closed as arm64 runtime smoke verified on OPPO `PLG110`.
