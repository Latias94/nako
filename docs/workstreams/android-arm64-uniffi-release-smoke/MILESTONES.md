# Android arm64 UniFFI Release Smoke — Milestones

Status: Closed
Last updated: 2026-05-21

## M0 — Lane Open

Status: Complete on 2026-05-21.

Exit when arm64 UniFFI release risk is separated from the x86_64 emulator smoke.

## M1 — arm64 Device Detection

Status: Complete on 2026-05-21.

Exit when attached device ABI evidence is recorded and the validation path is
chosen.

Result: The initial attached device was only an x86_64 emulator, so the lane
correctly chose arm64 packaging verification instead of claiming arm64 runtime
execution. After OPPO `PLG110` was attached, primary ABI `arm64-v8a` was
confirmed.

## M2 — arm64 Runtime Or Packaging Verification

Status: Complete on 2026-05-21.

Exit when either:

- connected instrumentation passes on an arm64 device; or
- arm64-v8a APK packaging is built and inspected, with runtime risk still
  explicitly documented.

Result: `arm64-v8a` APK packaging was built and inspected. The APK contains
`lib/arm64-v8a/libnako_client_uniffi.so` and
`lib/arm64-v8a/libjnidispatch.so`, with no non-arm64 JNI entries after
threading focused ABI selection into Android's `ndk.abiFilters`. The same
debug APK and test APK then passed the UniFFI native smoke on OPPO `PLG110`.

## M3 — Closeout

Status: Complete on 2026-05-21.

Exit when status, evidence, and residual risk agree.

Result: Lane closed as arm64 runtime smoke verified on a real OPPO `PLG110`
device.
