# Android UniFFI Native Smoke — Milestones

Status: Closed
Last updated: 2026-05-21

## M0 — Lane Open

Status: Complete on 2026-05-21.

Exit when the runtime native smoke risk is split into a focused lane with a
clear non-goal boundary.

## M1 — Instrumentation Smoke

Status: Complete on 2026-05-21.

Exit when:

- app instrumentation dependencies are present;
- an androidTest smoke calls generated UniFFI bindings;
- debug app and debug androidTest APKs build with focused ABI selection.

Result: Added AndroidX instrumentation dependencies and
`NakoUniFfiNativeSmokeTest`, which calls generated UniFFI bindings without
network, server, UI, profile, token, or Media3 dependencies.

## M2 — Device/Emulator Execution

Status: Complete on 2026-05-21.

Exit when:

- `connectedDebugAndroidTest` passes on a device/emulator; or
- no device/emulator is available and the lane records the exact external
  blocker without claiming runtime success.

Result: `connectedDebugAndroidTest` passed on
`Pixel_3a_API_34_extension_level_7_x86_64(AVD) - 14` after the Android runtime
dependency was switched to the JNA AAR artifact.

## M3 — Closeout

Status: Complete on 2026-05-21.

Exit when docs, gates, and status agree on passed or blocked runtime evidence.

Result: The lane closed with real emulator runtime evidence for packaged UniFFI
library loading and a preserved JVM host-test path.
