# Android Rust Core Runtime Hardening — TODO

Status: Active
Last updated: 2026-05-21

## M0 — Lane Open

- [x] RCR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-rust-core-runtime-hardening]
  Goal: Open the hardening lane, freeze scope, and order the four follow-ons
  from `generated-sdk-runtime-ownership`.
  Validation: Workstream docs and `WORKSTREAM.json` agree on task order and
  gates.
  Review: Confirm this is a serialized architecture lane, not a catch-all SDK
  publishing or Rust-owned networking lane.
  Evidence: `docs/workstreams/android-rust-core-runtime-hardening/DESIGN.md`
  Handoff: DONE. Start with `RCR-020`.

## M1 — Android Rust/UniFFI Build Ergonomics

- [x] RCR-020 [owner=codex] [deps=RCR-010] [scope=apps/android/app/build.gradle.kts,apps/android/README.md]
  Goal: Split binding generation, host JVM-test library, and Android ABI native
  library packaging so JVM unit tests do not build every Android ABI and APK
  packaging can select ABI sets explicitly.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`; `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PtaruRustAndroidAbis=x86_64 --no-daemon`
  Review: Gradle tasks must be incremental, package-aware, documented, and not
  dependent on global UniFFI bindgen installation.
  Evidence: `apps/android/app/build.gradle.kts`; `apps/android/README.md`;
  `docs/workstreams/android-rust-core-runtime-hardening/EVIDENCE_AND_GATES.md#evidence-log`
  Handoff: DONE. Host library, Kotlin binding generation, and Android ABI libraries are separate tasks. JVM tests no longer depend on Android ABI native library builds. APK assembly builds variant JNI libs and supports focused ABI selection through `-PtaruRustAndroidAbis=...`.

## M2 — Rust Client Adapter Reuse

- [ ] RCR-030 [owner=codex] [deps=RCR-020] [scope=crates/taru-client-core,crates/taru-client]
  Goal: Move portable request construction, bearer injection, API-version
  checks, public error-envelope parsing, and redaction-safe previews into
  `taru-client-core`; make `taru-client` consume that core policy as a
  reqwest/async adapter.
  Validation: `cargo fmt --package taru-client-core --package taru-client --check`; `cargo nextest run -p taru-client-core --no-fail-fast`; `cargo nextest run -p taru-client --no-fail-fast`
  Review: `taru-client` may keep reqwest transport ergonomics, but must not
  keep a second implementation of shared response policy.
  Evidence: `crates/taru-client-core/src/lib.rs`; `crates/taru-client/src/lib.rs`;
  `EVIDENCE_AND_GATES.md`
  Handoff: Pending.

## M3 — Rust Public Wire Tolerance

- [ ] RCR-040 [owner=codex] [deps=RCR-030] [scope=crates/taru-client-protocol,crates/taru-api,crates/taru-client]
  Goal: Make Rust public string-value DTOs preserve unknown additive wire values
  instead of failing deserialization.
  Validation: `cargo fmt --package taru-client-protocol --package taru-api --package taru-client --check`; `cargo nextest run -p taru-client-protocol --no-fail-fast`; `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`; `cargo nextest run -p taru-client --no-fail-fast`
  Review: Known-value ergonomics should stay explicit, while unknown values
  retain their raw strings through decode/encode.
  Evidence: `crates/taru-client-protocol/src/catalog.rs`;
  `EVIDENCE_AND_GATES.md`
  Handoff: Pending.

## M4 — Android Playback Core Tracer

- [ ] RCR-050 [owner=codex] [deps=RCR-040] [scope=crates/taru-client-core,crates/taru-client-uniffi,apps/android/app/src/main/java/dev/taru/android/playback,apps/android/app/src/test/java/dev/taru/android/playback]
  Goal: Use the Rust core / UniFFI boundary for playback decision request
  construction and playback target interpretation while keeping Android-owned
  transport execution, diagnostics, and Media3.
  Validation: `cargo fmt --package taru-client-core --package taru-client-uniffi --check`; `cargo nextest run -p taru-client-core --no-fail-fast`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon`; `apps/android/gradlew.bat -p apps/android :app:assembleDebug -Ptaru.rust.android.abis=x86_64 --no-daemon`
  Review: Rust may choose safe playback request targets, but Android must keep
  Media3, session preflight execution, product errors, and user messages.
  Evidence: `crates/taru-client-core/src/lib.rs`;
  `apps/android/app/src/main/java/dev/taru/android/playback`;
  `EVIDENCE_AND_GATES.md`
  Handoff: Pending.

## M5 — Closeout

- [ ] RCR-090 [owner=planner] [deps=RCR-050] [scope=docs/workstreams/android-rust-core-runtime-hardening]
  Goal: Close the lane with fresh evidence, residual risks, and follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null`; `git diff --check`
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `docs/workstreams/android-rust-core-runtime-hardening/CLOSEOUT.md`
  Handoff: Pending.
