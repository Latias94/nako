# Android UniFFI Boundary Hardening — TODO

Status: Active
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

- [x] UBF-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-uniffi-boundary-hardening]
  Goal: Freeze the hardening target state, non-goals, task order, and gates for
  the current UniFFI seam.
  Validation: `python -m json.tool docs/workstreams/android-uniffi-boundary-hardening/WORKSTREAM.json > $null`; docs agree on scope and task IDs.
  Review: Confirm this lane hardens the existing ADR 0032 seam instead of
  expanding Rust-owned networking or browse/catalog DTO decode.
  Evidence: `docs/workstreams/android-uniffi-boundary-hardening/DESIGN.md`
  Handoff: DONE. Start with UBF-020.

## M1 — Android Adapter Encapsulation

- [x] UBF-020 [owner=codex] [deps=UBF-010] [scope=apps/android/app/src/main/java/dev/taru/android/connection,apps/android/app/src/test/java/dev/taru/android/connection]
  Goal: Hide generated UniFFI connection request/outcome types behind
  Android-owned `ConnectionCore` result types so product connection logic no
  longer switches over `uniffi.taru_client_uniffi.*` types directly.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`
  Review: `TaruConnectionClient` should know only Android-owned connection
  models plus `ConnectionCore`; generated UniFFI imports should remain isolated
  to `RustConnectionCore.kt` and native smoke tests.
  Evidence: `apps/android/app/src/main/java/dev/taru/android/connection/RustConnectionCore.kt`; connection unit tests.
  Handoff: DONE. `TaruConnectionClient` now consumes Android-owned
  `ConnectionCoreOutcome`, `ConnectionCoreRequest`, and `ConnectionCoreSuccess`
  instead of generated UniFFI request/outcome types. Generated UniFFI imports
  are isolated to `RustConnectionCore.kt` for the connection seam, while product
  mapping, transport, and diagnostics remain Android-owned.

## M2 — Core Module Split

- [x] UBF-030 [owner=codex] [deps=UBF-020] [scope=crates/taru-client-core]
  Goal: Split `taru-client-core/src/lib.rs` into request, response, redaction,
  connection, playback, and encoding modules while preserving the current
  public Rust API and behavior.
  Validation: `cargo fmt --package taru-client-core --check`; `cargo nextest run -p taru-client-core --no-fail-fast`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; `cargo nextest run -p taru-client --no-fail-fast`
  Review: Module split must improve locality without adding pass-through files
  that make the interface shallower.
  Evidence: `crates/taru-client-core/src/*.rs`; Rust package tests.
  Handoff: DONE. `taru-client-core` now exposes the same public API from
  `lib.rs` while implementation locality lives in `ids`, `encoding`,
  `redaction`, `request`, `response`, `connection`, and `playback` modules.
  Rust core, UniFFI, and reqwest client package gates passed.

## M3 — Boundary Drift Guards

- [x] UBF-040 [owner=codex] [deps=UBF-030] [scope=scripts,crates/taru-client-uniffi,apps/android]
  Goal: Add local validation guard(s) that reject accidental UniFFI dependency
  creep and document the expected generated surface.
  Validation: boundary guard command passes; `cargo nextest run -p taru-client-uniffi --no-fail-fast` passes.
  Review: Guard should fail if `taru-client-uniffi` starts depending on
  `reqwest`, `tokio`, Android platform crates, or other runtime transports.
  Evidence: guard script/test path and command output in `EVIDENCE_AND_GATES.md`.
  Handoff: DONE. Added `scripts/guard-uniffi-boundary.ps1`, which checks
  `taru-client-uniffi` direct dependencies against an allowlist and fails if
  forbidden runtime/platform dependencies such as reqwest, Tokio, hyper, tower,
  axum, SQL, JNI, or Android platform crates appear in the dependency tree.
  Guard and UniFFI package tests passed.

## M4 — Native Smoke Script

- [x] UBF-050 [owner=codex] [deps=UBF-040] [scope=apps/android/scripts,apps/android/README.md,docs/workstreams/android-uniffi-boundary-hardening]
  Goal: Add a reusable PowerShell script that builds selected ABI APKs,
  installs app/test APKs on a selected connected device, and runs
  `TaruUniFfiNativeSmokeTest`.
  Validation: run the script against the connected OPPO arm64 device when
  available, or record a device-unavailable reason plus a dry command check.
  Review: Script must be opt-in, serial-aware, ABI-aware, non-destructive, and
  not make ordinary local JVM validation require a device.
  Evidence: `apps/android/scripts/Validate-UniFfiNativeSmoke.ps1`; README usage;
  device command output.
  Handoff: DONE. Added `apps/android/scripts/Validate-UniFfiNativeSmoke.ps1`
  and README usage. The original OPPO serial was no longer connected during
  this task, so the script recorded that failure path, then successfully built,
  installed, and ran `TaruUniFfiNativeSmokeTest` on connected `emulator-5554`
  with `-Abi x86_64`.

## M5 — Closeout

- [x] UBF-090 [owner=planner] [deps=UBF-050] [scope=docs/workstreams/android-uniffi-boundary-hardening]
  Goal: Close the lane with fresh evidence, residual risks, and recommended
  follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-uniffi-boundary-hardening/WORKSTREAM.json > $null`; `git diff --check`; final targeted gates from `EVIDENCE_AND_GATES.md`.
  Review: Confirm no blocking architecture or code-quality findings remain.
  Evidence: `docs/workstreams/android-uniffi-boundary-hardening/EVIDENCE_AND_GATES.md`; closeout notes.
  Handoff: DONE. Lane closed with `CLOSEOUT.md`, final targeted gates, native
  smoke script evidence, and residual risks. Browse/catalog migration remains a
  follow-on, not hidden scope in this lane.
