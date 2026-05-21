# Generated SDK Runtime Ownership — TODO

Status: Active
Last updated: 2026-05-21

## M0 — Runtime Ownership Freeze

- [x] SDKRT-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-sdk-runtime-ownership,apps/android/app/src/main/java/dev/taru/android/connection,apps/android/app/src/main/java/dev/taru/android/playback,sdk/kotlin,crates/taru-api/src/sdk.rs]
  Goal: Inventory current SDK/runtime/app/Rust-client responsibilities, freeze the ownership matrix, and decide whether runtime behavior stays Android-owned, moves into a Kotlin SDK/runtime seam, or is pulled forward into a shared Rust client core / UniFFI target state.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` agree on the selected option and ADR impact.
  Review: Confirm protocol-level policy is separated from product/platform policy, and that no UI, Media3, token vault, profile persistence, or cleartext/TLS behavior is moved by accident.
  Evidence: `docs/workstreams/generated-sdk-runtime-ownership/DESIGN.md#sdkrt-010-frozen-decision`
  Handoff: DONE. Selected early shared Rust client core / UniFFI target state with app-supplied Android transport as the first tracer. `SDKRT-020` must create or supersede the ADR and define the FFI-safe core API before code.

## M1 — Runtime Contract Decision

- [ ] SDKRT-020 [owner=codex] [deps=SDKRT-010] [scope=docs/adr,docs/workstreams/generated-sdk-runtime-ownership,crates/taru-client,crates/taru-client-protocol,crates/taru-api/src/sdk.rs,sdk/kotlin]
  Goal: Define the smallest selected runtime/core API shape and record whether ADR 0031 is amended or superseded. If early Rust core is selected, define crate topology, FFI-safe data shapes, app-supplied versus Rust-owned transport, and the first Android tracer. If no runtime move is selected, close or split the lane without code.
  Validation: ADR/workstream docs name the selected option, rejected alternatives, compatibility expectations, build topology, and first tracer.
  Review: Runtime/core API must consume Public Client API contract surfaces without becoming an Android product API or hidden portable application framework.
  Evidence: `docs/workstreams/generated-sdk-runtime-ownership/DESIGN.md`
  Handoff: Do not generate, hand-write, or FFI-bind runtime code until this task chooses the contract shape.

## M2 — Small Runtime/Core Tracer

- [ ] SDKRT-030 [owner=codex] [deps=SDKRT-020] [scope=crates/taru-client,crates/taru-client-protocol,crates/taru-api/src/sdk.rs,sdk/kotlin/src/main/kotlin,sdk/kotlin/src/test/kotlin]
  Goal: Implement the smallest selected runtime/core tracer around protocol-level request construction, response decoding, public error parsing, API-version observation, and redaction-safe request previewing.
  Validation: `cargo fmt --package taru-api --package taru-client --package taru-client-protocol --check`; `cargo nextest run -p taru-client --no-fail-fast`; `cargo nextest run -p taru-client-protocol --no-fail-fast`; if Kotlin generator changes, also `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast` and `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  Review: Generated output must remain mechanically synchronized from `taru-api` if generator-owned code changes. Rust core must expose FFI-safe boundaries if it is selected; Kotlin runtime code must be clearly separated from generated DTOs if that fallback is chosen.
  Evidence: selected runtime/core source and tests
  Handoff: Keep the tracer narrow; split publishing, KMP, or multi-SDK runtime work.

## M3 — Android Consumption Tracer

- [ ] SDKRT-040 [owner=codex] [deps=SDKRT-030] [scope=apps/android/app/src/main/java/dev/taru/android/connection,apps/android/app/src/main/java/dev/taru/android/playback,apps/android/app/src/test/java/dev/taru/android]
  Goal: Migrate one low-risk Android flow to the selected runtime/core tracer while keeping product diagnostics, cleartext policy, token storage, and Media3 ownership in Android.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`; add playback focused tests only if playback code changes.
  Review: Android failure categories and user messages remain app-owned; token-safe request previews remain redaction-safe. If UniFFI is used, ordinary Android build and test commands must document the new Rust/NDK prerequisites.
  Evidence: Android focused unit tests and `EVIDENCE_AND_GATES.md`
  Handoff: If the tracer increases coupling or worsens diagnostics, revert the design direction by patching forward rather than broadening.

## M4 — Broaden Or Split

- [ ] SDKRT-050 [owner=codex] [deps=SDKRT-040] [scope=sdk/kotlin,apps/android/app/src/main/java/dev/taru/android]
  Goal: Decide whether to broaden the tracer across repeated route families or split follow-ons. Do not move platform/product policy merely to reduce local code.
  Validation: Focused SDK and Android tests for every migrated family; `git diff --check`.
  Review: Broadening must remove duplication without hiding Android-specific behavior in SDK code.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: Prefer closing and splitting over turning this lane into publishing, KMP, or full-platform Rust/UniFFI migration work.

## M5 — Closeout

- [ ] SDKRT-090 [owner=planner] [deps=SDKRT-050] [scope=docs/workstreams/generated-sdk-runtime-ownership]
  Goal: Close the lane, record final ownership, evidence, residual risks, and split follow-ons.
  Validation: Fresh final gate evidence is recorded; `WORKSTREAM.json`, `TODO.md`, `HANDOFF.md`, and `EVIDENCE_AND_GATES.md` agree.
  Review: Run workstream review for compliance and code-quality findings before marking complete.
  Evidence: `docs/workstreams/generated-sdk-runtime-ownership/CLOSEOUT.md`
  Handoff: Remaining SDK publishing, KMP, Rust/UniFFI, and multi-SDK runtime tolerance stay separate.
