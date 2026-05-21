# Generated SDK Forward Compatibility Tolerance — TODO

Status: Complete
Last updated: 2026-05-21

## M0 — Compatibility Contract Freeze

- [x] SDKFC-010 [owner=planner] [deps=none] [scope=docs/workstreams/generated-sdk-forward-compat-tolerance,crates/taru-api/src/sdk.rs,sdk/kotlin,apps/android/app/src/main/java/dev/taru/android]
  Goal: Inventory generated Kotlin string-enum decode surfaces, Android adapter usage, and API-version/error-envelope tolerance needs, then freeze the tolerant representation decision.
  Validation: `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` agree; inventory links concrete code surfaces.
  Review: Confirm the decision preserves raw `wireValue`, known-value ergonomics, and public-surface leak checks.
  Evidence: `docs/workstreams/generated-sdk-forward-compat-tolerance/DESIGN.md#frozen-representation-decision`
  Handoff: Frozen as generated Kotlin `@JvmInline @Serializable value class` wrappers with companion-object known constants, `KnownWireValues`, `isKnown`, and raw `wireValue` preservation.

## M1 — Generated Kotlin Tolerant Wire Values

- [x] SDKFC-020 [owner=codex] [deps=SDKFC-010] [scope=crates/taru-api/src/sdk.rs,crates/taru-api/examples/emit-kotlin-sdk.rs,sdk/kotlin]
  Goal: Replace strict generated Kotlin enum deserialization with the accepted tolerant wire-string representation for Public Client API string enums.
  Validation: `cargo fmt --package taru-api --check`; `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`; `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  Review: Generated output must be regenerated from `taru-api`, not edited by hand. Known constants must remain generated; unknown values must preserve raw `wireValue`.
  Evidence: `crates/taru-api/src/sdk.rs`; `sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`; `sdk/kotlin/src/test/kotlin/dev/taru/sdk/TaruClientSdkTest.kt`
  Handoff: `taru-api` now generates tolerant Kotlin value classes; `sdk/kotlin` was regenerated from the generator and SDK tests cover unknown decode/encode behavior.

- [x] SDKFC-030 [owner=codex] [deps=SDKFC-020] [scope=apps/android/app/src/main/java/dev/taru/android,apps/android/app/src/test/java/dev/taru/android]
  Goal: Update Android generated-SDK adapters and diagnostics to consume tolerant wire values without leaking SDK policy into UI or Media3 runtime.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --tests dev.taru.android.playback.* --tests dev.taru.android.browse.* --no-daemon`
  Review: Android product categories, copy, token redaction, safe request previews, and Media3 ownership remain app-owned.
  Evidence: `apps/android/app/src/main/java/dev/taru/android/connection/TaruConnectionClient.kt`; `apps/android/app/src/main/java/dev/taru/android/playback/PlaybackSdkAdapters.kt`; `apps/android/app/src/main/java/dev/taru/android/browse/BrowseSdkAdapters.kt`
  Handoff: Android connection health decodes generated tolerant `HealthResponse`; playback adapters map unknown generated values into app-owned `Unknown`/fallback states and focused tests cover unsupported body API version plus unknown playback mode.

## M2 — Cross-SDK Contract And Regression Gates

- [x] SDKFC-040 [owner=codex] [deps=SDKFC-030] [scope=crates/taru-api,sdk/kotlin,sdk/typescript,apps/android]
  Goal: Run and record the full compatibility gate set, including leakage checks and cross-SDK drift checks.
  Validation: Gate set in `EVIDENCE_AND_GATES.md` passes with fresh evidence.
  Review: Confirm TypeScript is either unaffected or has an explicit documented compatibility stance. Confirm no admin/internal/raw-locator/storage/local-path terms leak into generated Kotlin output.
  Evidence: `docs/workstreams/generated-sdk-forward-compat-tolerance/EVIDENCE_AND_GATES.md`
  Handoff: Full closeout gate passed on 2026-05-21; TypeScript remains runtime-unaffected and compile-checked.

## M3 — Closeout

- [x] SDKFC-090 [owner=codex] [deps=SDKFC-040] [scope=docs/workstreams/generated-sdk-forward-compat-tolerance]
  Goal: Close the lane or split any remaining SDK runtime/publishing/KMP follow-ons.
  Validation: `WORKSTREAM.json`, `TODO.md`, `HANDOFF.md`, and `EVIDENCE_AND_GATES.md` agree; closeout note records residual risks and follow-ons.
  Review: Run workstream review before marking complete.
  Evidence: `docs/workstreams/generated-sdk-forward-compat-tolerance/CLOSEOUT.md`
  Handoff: Lane closed on 2026-05-21. Remaining SDK publishing, KMP, generated runtime ownership, Rust/UniFFI, and wider multi-SDK runtime tolerance stay split.
