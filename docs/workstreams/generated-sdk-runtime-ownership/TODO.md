# Generated SDK Runtime Ownership — TODO

Status: Closed
Last updated: 2026-05-21

## M0 — Runtime Ownership Freeze

- [x] SDKRT-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-sdk-runtime-ownership,apps/android/app/src/main/java/dev/taru/android/connection,apps/android/app/src/main/java/dev/taru/android/playback,sdk/kotlin,crates/taru-api/src/sdk.rs]
  Goal: Inventory current SDK/runtime/app/Rust-client responsibilities, freeze the ownership matrix, and decide whether runtime behavior stays Android-owned, moves into a Kotlin SDK/runtime seam, or is pulled forward into a shared Rust client core / UniFFI target state.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` agree on the selected option and ADR impact.
  Review: Confirm protocol-level policy is separated from product/platform policy, and that no UI, Media3, token vault, profile persistence, or cleartext/TLS behavior is moved by accident.
  Evidence: `docs/workstreams/generated-sdk-runtime-ownership/DESIGN.md#sdkrt-010-frozen-decision`
  Handoff: DONE. Selected early shared Rust client core / UniFFI target state with app-supplied Android transport as the first tracer. `SDKRT-020` must create or supersede the ADR and define the FFI-safe core API before code.

## M1 — Runtime Contract Decision

- [x] SDKRT-020 [owner=codex] [deps=SDKRT-010] [scope=docs/adr,docs/workstreams/generated-sdk-runtime-ownership,crates/taru-client,crates/taru-client-protocol,crates/taru-api/src/sdk.rs,sdk/kotlin]
  Goal: Define the smallest selected runtime/core API shape and record whether ADR 0031 is amended or superseded. If early Rust core is selected, define crate topology, FFI-safe data shapes, app-supplied versus Rust-owned transport, and the first Android tracer. If no runtime move is selected, close or split the lane without code.
  Validation: ADR/workstream docs name the selected option, rejected alternatives, compatibility expectations, build topology, and first tracer.
  Review: Runtime/core API must consume Public Client API contract surfaces without becoming an Android product API or hidden portable application framework.
  Evidence: `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`; `docs/workstreams/generated-sdk-runtime-ownership/DESIGN.md#sdkrt-020-contract-decision`
  Handoff: DONE. ADR 0032 supersedes ADR 0031's post-generated-SDK mobile Rust/UniFFI sequencing. `SDKRT-030` may implement the smallest no-socket `taru-client-core` tracer and tests, but must not add Android UniFFI consumption before the core API is proven.

## M2 — Small Runtime/Core Tracer

- [x] SDKRT-030 [owner=codex] [deps=SDKRT-020] [scope=crates/taru-client-core,crates/taru-client-protocol,docs/workstreams/generated-sdk-runtime-ownership]
  Goal: Implement the smallest selected runtime/core tracer around protocol-level request construction, response decoding, public error parsing, API-version observation, and redaction-safe request previewing.
  Validation: `cargo fmt --package taru-client-core --package taru-client --package taru-client-protocol --check`; `cargo nextest run -p taru-client-core --no-fail-fast`; if protocol/client compatibility changes, also `cargo nextest run -p taru-client --no-fail-fast` and `cargo nextest run -p taru-client-protocol --no-fail-fast`
  Review: Generated output must remain mechanically synchronized from `taru-api` if generator-owned code changes. Rust core must expose FFI-safe boundaries if it is selected; Kotlin runtime code must be clearly separated from generated DTOs if that fallback is chosen.
  Evidence: `crates/taru-client-core/src/lib.rs`; `docs/workstreams/generated-sdk-runtime-ownership/JOURNAL/2026-05-21-sdkrt-030.md`
  Handoff: DONE. Added the no-socket connection probe state machine in `taru-client-core`. It builds health/auth-probe requests, interprets app-supplied responses, classifies HTTP/version/decode/token failures, and emits redaction-safe request previews. No Android or Rust-owned socket behavior was added.

## M3 — UniFFI Compile-Only Scaffold

- [x] SDKRT-035 [owner=codex] [deps=SDKRT-030] [scope=crates/taru-client-uniffi,crates/taru-client-core,Cargo.toml,docs/workstreams/generated-sdk-runtime-ownership]
  Goal: Add the thinnest UniFFI binding crate over the proven `taru-client-core` tracer and prove it compiles without wiring Android app behavior.
  Validation: `cargo fmt --package taru-client-uniffi --package taru-client-core --check`; `cargo nextest run -p taru-client-uniffi --no-fail-fast`; if binding generation is added, run the documented bindgen command.
  Review: Binding crate must not contain runtime policy. It may expose FFI-safe core records/functions only, and must not introduce Rust-owned Android networking.
  Evidence: `crates/taru-client-uniffi/src/lib.rs`; `docs/workstreams/generated-sdk-runtime-ownership/JOURNAL/2026-05-21-sdkrt-035.md`
  Handoff: DONE. Added a compile-only UniFFI binding crate over the core tracer. Binding generation and Android consumption stay in `SDKRT-040`.

## M4 — Android Consumption Tracer

- [x] SDKRT-040 [owner=codex] [deps=SDKRT-035] [scope=apps/android/app/src/main/java/dev/taru/android/connection,apps/android/app/build.gradle.kts,apps/android/gradle/libs.versions.toml,apps/android/app/src/test/java/dev/taru/android/connection,apps/android/README.md,crates/taru-uniffi-bindgen]
  Goal: Migrate one low-risk Android flow to the selected runtime/core tracer while keeping product diagnostics, cleartext policy, token storage, and Media3 ownership in Android.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`; add playback focused tests only if playback code changes.
  Review: Android failure categories and user messages remain app-owned; token-safe request previews remain redaction-safe. If UniFFI is used, ordinary Android build and test commands must document the new Rust/NDK prerequisites.
  Evidence: `apps/android/app/src/main/java/dev/taru/android/connection/RustConnectionCore.kt`; `apps/android/app/src/main/java/dev/taru/android/connection/TaruConnectionClient.kt`; `apps/android/app/src/test/java/dev/taru/android/connection/TaruConnectionClientTest.kt`; `docs/workstreams/generated-sdk-runtime-ownership/JOURNAL/2026-05-21-sdkrt-040.md`
  Handoff: DONE. Android connection checks now use the Rust core / UniFFI binding boundary for probe request construction and response interpretation while Android still executes transport and owns token/profile/security/diagnostic/UI behavior.

## M5 — Broaden Or Split

- [x] SDKRT-050 [owner=codex] [deps=SDKRT-040] [scope=sdk/kotlin,apps/android/app/src/main/java/dev/taru/android]
  Goal: Decide whether to broaden the tracer across repeated route families or split follow-ons. Do not move platform/product policy merely to reduce local code.
  Validation: Focused SDK and Android tests for every migrated family; `git diff --check`.
  Review: Broadening must remove duplication without hiding Android-specific behavior in SDK code.
  Evidence: `docs/workstreams/generated-sdk-runtime-ownership/CLOSEOUT.md#sdkrt-050-decision-split-not-broaden`; `docs/workstreams/generated-sdk-runtime-ownership/EVIDENCE_AND_GATES.md#evidence-log`
  Handoff: DONE. The lane closes and splits follow-ons instead of broadening browse, playback, SDK publishing, KMP, Rust-owned networking, or multi-SDK runtime work here. The connection tracer is enough to prove the Rust core / UniFFI boundary; wider route-family migrations need their own tolerance, product-policy, and validation lanes.

## M6 — Closeout

- [x] SDKRT-090 [owner=planner] [deps=SDKRT-050] [scope=docs/workstreams/generated-sdk-runtime-ownership]
  Goal: Close the lane, record final ownership, evidence, residual risks, and split follow-ons.
  Validation: Fresh final gate evidence is recorded; `WORKSTREAM.json`, `TODO.md`, `HANDOFF.md`, and `EVIDENCE_AND_GATES.md` agree.
  Review: Run workstream review for compliance and code-quality findings before marking complete.
  Evidence: `docs/workstreams/generated-sdk-runtime-ownership/CLOSEOUT.md`
  Handoff: DONE. Closeout records no blocking review findings, passed closeout gates, residual risks, and separate follow-on lanes.
