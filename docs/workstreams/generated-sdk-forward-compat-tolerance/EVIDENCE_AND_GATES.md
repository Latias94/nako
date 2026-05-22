# Generated SDK Forward Compatibility Tolerance — Evidence And Gates

Status: Complete
Last updated: 2026-05-21

## Smallest Current Repro

```powershell
cargo nextest run -p nako-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon
```

These commands prove the generator/package sync and standalone generated Kotlin
SDK tests.

## Gate Set

### Planning Gate

```powershell
python -m json.tool docs/workstreams/generated-sdk-forward-compat-tolerance/WORKSTREAM.json > $null
git diff --check
```

### Generator And SDK Gate

```powershell
cargo fmt --package nako-api --check
cargo nextest run -p nako-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon
git diff --check
```

### Android Consumption Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --tests dev.nako.android.playback.* --tests dev.nako.android.browse.* --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
```

### Closeout Gate

```powershell
cargo fmt --package nako-api --check
cargo nextest run -p nako-api kotlin_sdk --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
npm run check --prefix sdk/typescript
apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon
python -m json.tool docs/workstreams/generated-sdk-forward-compat-tolerance/WORKSTREAM.json > $null
git diff --check
```

`npm run check --prefix sdk/typescript` is included at closeout because the
compatibility principle affects generated SDK contract thinking even when the
implementation is Kotlin-only.

## Review Gate

Closeout review found no blocking findings, no important findings, and no
missing gates.

## Evidence Anchors

- `docs/workstreams/generated-sdk-forward-compat-tolerance/DESIGN.md`
- `docs/workstreams/generated-sdk-forward-compat-tolerance/TODO.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
- `crates/nako-api/src/sdk.rs`
- `crates/nako-api/examples/emit-kotlin-sdk.rs`
- `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
- `sdk/kotlin/src/test/kotlin/dev/nako/sdk/NakoClientSdkTest.kt`
- `apps/android/app/src/main/java/dev/nako/android`

## Evidence Log

### 2026-05-21 — Workstream opened

- PASS: Current-state inspection of ADR 0025, ADR 0031, the Android Generated
  Public Client SDK closeout, `crates/nako-api/src/sdk.rs`, generated
  `sdk/kotlin`, and Android connection health handling.
  - Confirms the follow-on risk: strict generated Kotlin enums cannot preserve
    future unknown public string values.
  - Confirms this lane should be scoped to generated SDK compatibility
    tolerance, not SDK publishing, KMP, generated runtime ownership, or
    Rust/UniFFI.

### 2026-05-21 — SDKFC-010 compatibility contract freeze

- PASS: Inventory of generated Kotlin enum surfaces in
  `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`.
  - Found 15 generated strict enum surfaces before implementation:
    `RemuxOutputContainer`, `ClientPlaybackDecisionMode`,
    `ClientTranscodePlanHardwareAcceleration`,
    `ClientTranscodePlanOutputContainer`, `ErrorResponseCode`,
    `HealthResponseVersion`, `LibraryOptionsDtoDomain`,
    `LibraryOptionsDtoNamingStrategy`, `LibraryOptionsDtoPreset`,
    `MetadataProfileDtoLocalMetadataPolicy`, `MetadataProfileDtoRefreshMode`,
    `TranscodeSessionDtoFailureCategory`, `TranscodeSessionDtoKind`,
    `TranscodeSessionDtoState`, and `ClientMediaKind`.
- PASS: Android adapter usage review of `NakoConnectionClient.kt`,
  `PlaybackSdkAdapters.kt`, `BrowseSdkAdapters.kt`, and UI playback formatters.
  - Frozen decision: generated Kotlin string enums become
    `@JvmInline @Serializable value class` wrappers with generated known
    constants, `KnownWireValues`, `isKnown`, and raw `wireValue` preservation.

### 2026-05-21 — SDKFC-020 generated Kotlin tolerant wire values

- RED then GREEN: `apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --tests dev.nako.sdk.NakoClientSdkTest.decodesUnknownPublicWireValuesWithoutLosingRawValue --no-daemon`
  - Initial run failed because strict `enum class` types had no `isKnown` and
    could not be instantiated with unknown raw values.
  - After generator changes, the focused test passed.
- PASS: `cargo run -q -p nako-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
  - Regenerated checked Kotlin SDK output from `nako-api`.
- PASS: `cargo fmt --package nako-api --check`
- PASS: `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
- PASS: `apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon`
  - Proves generated Kotlin SDK compiles and tests known plus unknown public
    wire string decode/encode behavior.

### 2026-05-21 — SDKFC-030 Android generated-SDK tolerance consumption

- PASS: `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon`
  - Proves Android source compiles after generated enum value classes replaced
    strict enums.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --tests dev.nako.android.playback.* --tests dev.nako.android.browse.* --no-daemon`
  - Proves connection, playback, and browse focused tests remain green.
  - New coverage proves future `/health.version` body values become
    `UnsupportedApiVersion`, and future playback modes decode to an app-owned
    unsupported fallback instead of invalid JSON.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves the full Android JVM unit-test suite remains green after app-owned
    unknown fallback additions.

### 2026-05-21 — SDKFC-040 closeout gate

- PASS: `cargo fmt --package nako-api --check`
- PASS: `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
- PASS: `cargo nextest run -p nako-api --no-fail-fast`
  - 45 tests passed.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves the existing TypeScript SDK compile contract remains green; this
    lane does not add TypeScript runtime JSON enum validation.
- PASS: `apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon`
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
- PASS: `python -m json.tool docs/workstreams/generated-sdk-forward-compat-tolerance/WORKSTREAM.json > $null`
- PASS: `git diff --check`
  - No whitespace errors; Git reported only expected Windows line-ending
    warnings.

### 2026-05-21 — SDKFC-090 closeout

- PASS: Closeout review of `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, ADR 0025, ADR 0031,
  `crates/nako-api/src/sdk.rs`, `sdk/kotlin`, and Android adapter changes.
  - Workstream Compliance: no blocking or important findings.
  - Code Quality: no blocking or important findings.
  - Missing Gates: none; fresh SDKFC-040 evidence is recorded above.
  - Residual Risk: SDK publishing, KMP, generated runtime ownership,
    Rust/UniFFI, and wider multi-SDK runtime tolerance remain separate
    follow-ons.

## Notes

- Generated Kotlin source must be refreshed through the generator command, not
  edited by hand.
- Do not weaken OpenAPI leakage checks to make unknown values easier.
- Preserve Android app-owned diagnostics and presentation boundaries.
