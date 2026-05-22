# Android UniFFI Boundary Hardening

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

The Android client now uses `nako-client-core` through UniFFI for connection
probing and playback request/target construction. That is the correct long-term
ownership direction from ADR 0032, but the seam is still young. Without a
hardening lane, generated UniFFI types can leak upward into Android product
modules, `nako-client-core` can grow into a single shallow file, and native
smoke verification can remain a manual release-risk recipe.

## Relevant Authority

- ADRs:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- Existing docs:
  - `CONTEXT.md`
  - `AGENTS.md`
- Related workstreams:
  - `docs/workstreams/android-rust-core-runtime-hardening/`
  - `docs/workstreams/android-uniffi-native-smoke/`
  - `docs/workstreams/android-arm64-uniffi-release-smoke/`

## Problem

The current Rust core / UniFFI direction is right, but several seams are not yet
hardened enough to be safe growth points for browse/catalog/playback expansion:

- Android connection flow directly handles generated UniFFI outcome/request
  types in a product module.
- `nako-client-core/src/lib.rs` mixes request construction, response policy,
  redaction, connection probing, playback planning, encoding, and tests.
- There is no automated guard that rejects accidental `nako-client-uniffi`
  dependency creep into reqwest/Tokio/platform runtime code.
- The OPPO arm64 native smoke has evidence, but no reusable validation script.

## Target State

When this lane closes:

- Android product modules call Android-owned core interfaces and do not switch
  over generated UniFFI outcome/request types directly.
- `nako-client-core` is split into deep modules with stable re-exports from
  `lib.rs`; public Rust callers and UniFFI continue to use the same core API.
- UniFFI surface and dependency guard tests/scripts prevent boundary drift.
- A reusable PowerShell validation script builds, installs, and runs the UniFFI
  native smoke on a selected Android device/ABI.
- Workstream evidence records targeted Rust, Android JVM, APK packaging, and
  device-smoke gates.

## In Scope

- Android connection-core adapter cleanup.
- `nako-client-core` module split with no behavior expansion.
- UniFFI dependency/surface guard suitable for local validation and future CI.
- Android UniFFI native smoke script for connected devices/emulators.
- Docs, README references, and workstream evidence.

## Out Of Scope

- Rust-owned Android networking.
- Moving browse/catalog/user-playback DTO decode behind UniFFI.
- Media3/player/session UI changes.
- Server API shape changes.
- Public protocol DTO changes beyond what is required to preserve the current
  API during refactor.
- Broad physical-device matrix beyond a selectable single-device script.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| ADR 0032 remains the owner decision: Rust core owns portable policy, Android owns platform runtime. | High | `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md` | Reopen ADR before changing this lane. |
| UniFFI has already passed x86_64 emulator and OPPO arm64 smoke. | High | `docs/workstreams/android-uniffi-native-smoke/`; `docs/workstreams/android-arm64-uniffi-release-smoke/` | Script work may need diagnosis before hardening can close. |
| `nako-client-core` public API should remain source-compatible for current `nako-client` and `nako-client-uniffi` callers. | High | `android-rust-core-runtime-hardening` closeout | If a better public API is needed, split a follow-on ADR/workstream. |
| Generated UniFFI Kotlin bindings are still allowed inside adapter modules. | High | ADR 0032 | If zero generated-type imports are required in Android, this lane is too large and needs a different architecture. |

## Architecture Direction

Keep the seam deep and directional:

```text
nako-client-core
  portable request/response/redaction/connection/playback policy

nako-client-uniffi
  generated-binding adapter only

Android adapter modules
  translate UniFFI records/outcomes into Android-owned interfaces

Android product/runtime modules
  transport, token/profile state, diagnostics, product copy, Media3, UI
```

The hardening strategy is not to hide Rust. It is to make the seam explicit:
Android should depend on Android-owned `ConnectionCore` / `PlaybackCore`
interfaces, while only the adapter files import generated UniFFI packages.
`nako-client-core` should become easier to expand by making request, response,
redaction, connection, playback, and encoding each locally understandable.

## Closeout Condition

This lane can close when:

- TODO tasks UBF-010 through UBF-090 are complete,
- targeted Rust and Android JVM gates pass,
- arm64 or selected-device UniFFI smoke script has been exercised or a concrete
  device-unavailable reason is recorded,
- docs reflect the shipped seam,
- and follow-on browse/catalog migration work is explicitly deferred.
