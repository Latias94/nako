# Android Rust Core Runtime Hardening — TODO

Status: Closed
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
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon`; `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PnakoRustAndroidAbis=x86_64 --no-daemon`
  Review: Gradle tasks must be incremental, package-aware, documented, and not
  dependent on global UniFFI bindgen installation.
  Evidence: `apps/android/app/build.gradle.kts`; `apps/android/README.md`;
  `docs/workstreams/android-rust-core-runtime-hardening/EVIDENCE_AND_GATES.md#evidence-log`
  Handoff: DONE. Host library, Kotlin binding generation, and Android ABI libraries are separate tasks. JVM tests no longer depend on Android ABI native library builds. APK assembly builds variant JNI libs and supports focused ABI selection through `-PnakoRustAndroidAbis=...`.

## M2 — Rust Client Adapter Reuse

- [x] RCR-030 [owner=codex] [deps=RCR-020] [scope=crates/nako-client-core,crates/nako-client]
  Goal: Move portable request construction, bearer injection, API-version
  checks, public error-envelope parsing, and redaction-safe previews into
  `nako-client-core`; make `nako-client` consume that core policy as a
  reqwest/async adapter.
  Validation: `cargo fmt --package nako-client-core --package nako-client --check`; `cargo nextest run -p nako-client-core --no-fail-fast`; `cargo nextest run -p nako-client --no-fail-fast`
  Review: `nako-client` may keep reqwest transport ergonomics, but must not
  keep a second implementation of shared response policy.
  Evidence: `crates/nako-client-core/src/lib.rs`; `crates/nako-client/src/lib.rs`;
  `docs/workstreams/android-rust-core-runtime-hardening/EVIDENCE_AND_GATES.md#evidence-log`
  Handoff: DONE. `nako-client-core` now exposes generic request spec construction, query/path encoding, bearer injection, safe previews, and generic response policy. `nako-client` builds reqwest requests from core specs and maps core response-policy failures back to Rust client errors.

## M3 — Rust Public Wire Tolerance

- [x] RCR-040 [owner=codex] [deps=RCR-030] [scope=crates/nako-client-protocol,crates/nako-api,crates/nako-client]
  Goal: Make Rust public string-value DTOs preserve unknown additive wire values
  instead of failing deserialization.
  Validation: `cargo fmt --package nako-client-protocol --package nako-api --package nako-client --check`; `cargo nextest run -p nako-client-protocol --no-fail-fast`; `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`; `cargo nextest run -p nako-client --no-fail-fast`
  Review: Known-value ergonomics should stay explicit, while unknown values
  retain their raw strings through decode/encode.
  Evidence: `crates/nako-client-protocol/src/catalog.rs`;
  `crates/nako-client-protocol/src/lib.rs`;
  `docs/workstreams/android-rust-core-runtime-hardening/EVIDENCE_AND_GATES.md#evidence-log`
  Handoff: DONE. Public Rust string-value DTOs now preserve unknown additive strings through `Other(String)` and re-serialize raw wire values. Known-value ergonomics use `wire_value()` and `is_known()`.

## M4 — Android Playback Core Tracer

- [x] RCR-050 [owner=codex] [deps=RCR-040] [scope=crates/nako-client-core,crates/nako-client-uniffi,apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/test/java/dev/nako/android/playback]
  Goal: Use the Rust core / UniFFI boundary for playback decision request
  construction and playback target interpretation while keeping Android-owned
  transport execution, diagnostics, and Media3.
  Validation: `cargo fmt --package nako-client-core --package nako-client-uniffi --check`; `cargo nextest run -p nako-client-core --no-fail-fast`; `cargo nextest run -p nako-client-uniffi --no-fail-fast`; `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.* --no-daemon`; `apps/android/gradlew.bat -p apps/android :app:assembleDebug -PnakoRustAndroidAbis=x86_64 --no-daemon`
  Review: Rust may choose safe playback request targets, but Android must keep
  Media3, session preflight execution, product errors, and user messages.
  Evidence: `crates/nako-client-core/src/lib.rs`;
  `apps/android/app/src/main/java/dev/nako/android/playback`;
  `EVIDENCE_AND_GATES.md`
  Handoff: DONE. Rust core now owns playback decision request construction,
  explicit direct/remux/HLS target builders, recommended target interpretation,
  HLS segment route construction, and session-preflight request descriptors.
  Android still owns transport execution, token/profile state, DTO-to-product
  mapping, diagnostics, session header handling, and Media3 launch policy.

## M5 — Closeout

- [x] RCR-090 [owner=planner] [deps=RCR-050] [scope=docs/workstreams/android-rust-core-runtime-hardening]
  Goal: Close the lane with fresh evidence, residual risks, and follow-ons.
  Validation: `python -m json.tool docs/workstreams/android-rust-core-runtime-hardening/WORKSTREAM.json > $null`; `git diff --check`
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `docs/workstreams/android-rust-core-runtime-hardening/CLOSEOUT.md`
  Handoff: DONE. Lane closed after fresh closeout gates passed. Residual risks
  are documented in `CLOSEOUT.md` and should be handled by new workstreams.
