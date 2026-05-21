# Generated SDK Runtime Ownership — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Evidence Anchors

- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `docs/workstreams/android-generated-public-client-sdk/CLOSEOUT.md`
- `docs/workstreams/generated-sdk-forward-compat-tolerance/CLOSEOUT.md`
- `crates/taru-api/src/sdk.rs`
- `crates/taru-client/src/lib.rs`
- `crates/taru-client-protocol/src/lib.rs`
- `crates/taru-client-protocol/src/catalog.rs`
- `sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`
- `sdk/kotlin/src/test/kotlin/dev/taru/sdk/TaruClientSdkTest.kt`
- `apps/android/app/src/main/java/dev/taru/android/connection/PublicClientApiExecutor.kt`
- `apps/android/app/src/main/java/dev/taru/android/connection/TaruHttpTransport.kt`
- `apps/android/app/src/main/java/dev/taru/android/connection/SensitiveText.kt`
- `apps/android/app/src/main/java/dev/taru/android/connection/TaruConnectionClient.kt`
- `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`

## Planning Gates

Run after opening or updating the planning docs:

```powershell
python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null
git diff --check
```

## Inventory Commands

Useful commands for `SDKRT-010`:

```powershell
rg "PublicClientApiExecutor|PublicApiFailure|SafeRequestPreview|SensitiveText|TaruHttpTransport" apps/android/app/src/main/java/dev/taru/android
rg "TaruRequestDescriptor|TaruPublicClientRequests|ErrorResponse|TARU_API_VERSION" sdk/kotlin crates/taru-api/src/sdk.rs
rg "TaruClient|ClientTransport|TaruClientError|UnsupportedApiVersion|ErrorResponse|ClientPlaybackMode" crates/taru-client crates/taru-client-protocol
```

## Implementation Gates

Only run these if the lane moves runtime code:

```powershell
cargo fmt --package taru-api --package taru-client --package taru-client-protocol --check
cargo nextest run -p taru-client --no-fail-fast
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon
```

If playback runtime consumption changes, also run focused playback tests:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --no-daemon
```

For the current Rust core / UniFFI tracer checkpoint, use:

```powershell
cargo fmt --package taru-client-core --package taru-client-uniffi --check
cargo nextest run -p taru-client-core --no-fail-fast
cargo nextest run -p taru-client-uniffi --no-fail-fast
python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null
git diff --check
```

## Closeout Gates

Use the broader gate set before closing an implementation lane:

```powershell
cargo fmt --package taru-api --check
cargo fmt --package taru-client --package taru-client-protocol --check
cargo nextest run -p taru-client --no-fail-fast
cargo nextest run -p taru-client-protocol --no-fail-fast
cargo nextest run -p taru-api kotlin_sdk --no-fail-fast
cargo nextest run -p taru-api --no-fail-fast
npm run check --prefix sdk/typescript
apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon
python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null
git diff --check
```

## Evidence Log

- 2026-05-21: Opened planning lane for `SDKRT-010`.
  - `python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null`
    passed.
  - `git diff --check` passed. Git reported the existing line-ending warning
    that `docs/workstreams/README.md` will be normalized to CRLF when Git next
    touches it.
- 2026-05-21: Completed `SDKRT-010` ownership freeze.
  - Inventory evidence reviewed:
    - `apps/android/app/src/main/java/dev/taru/android/connection/PublicClientApiExecutor.kt`
      owns execution, public error parsing, version checks, decode failure
      mapping, transport failure mapping, and redaction.
    - `apps/android/app/src/main/java/dev/taru/android/connection/TaruHttpTransport.kt`
      and `JdkTaruHttpTransport.kt` own Android-side HTTP execution.
    - `sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt` owns generated
      request descriptors, API constants, DTOs, and tolerant public wire string
      wrappers.
    - `crates/taru-client/src/lib.rs` already owns Rust request construction,
      reqwest execution, public error parsing, API-version checks, streaming
      request builders, and route tests, but uses non-FFI-safe `reqwest`, `Url`,
      `HeaderMap`, `StatusCode`, async traits, and `reqwest::Error`.
    - `crates/taru-client-protocol/src/catalog.rs` still uses strict serde
      enums for public string values such as playback mode, transcode state,
      output container, and hardware acceleration.
  - Frozen decision: pull shared Rust client core forward now, but start with an
    FFI-safe no-socket core and app-supplied Android transport rather than full
    Rust-owned Android networking.
  - ADR impact: `SDKRT-020` must create a new ADR or explicitly supersede the
    mobile-FFI sequencing portion of ADR 0031 before implementation.
  - `python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null`
    passed after the `SDKRT-010` updates.
  - `git diff --check` passed after the `SDKRT-010` updates. Git reported
    line-ending normalization warnings for touched workstream docs.

No implementation evidence yet. The lane is now ready for `SDKRT-020` target
ADR and FFI-safe core API definition.
- 2026-05-21: Completed `SDKRT-020` contract decision.
  - Added
    `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`.
  - Marked ADR 0031 superseded for post-generated-SDK mobile Rust/UniFFI
    sequencing.
  - Updated ADR index.
  - Frozen target: new no-socket `taru-client-core`, existing `taru-client` as
    reqwest adapter, future thin `taru-client-uniffi`, and first connection
    tracer with Android-supplied transport.
  - `python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null`
    passed after the `SDKRT-020` updates.
  - `git diff --check` passed after the `SDKRT-020` updates. Git reported
    line-ending normalization warnings for touched ADR/workstream docs.

No implementation evidence yet. The lane is now ready for `SDKRT-030` Rust
core tracer implementation.
- 2026-05-21: Completed `SDKRT-030` no-socket Rust client core tracer.
  - Added `crates/taru-client-core` with FFI-safe request, response, preview,
    outcome, success, public-error, and failure records.
  - The core tracer builds unauthenticated health requests, advances to an
    authenticated library auth probe, interprets app-supplied responses, checks
    API-version observations, parses public error envelopes, classifies invalid
    response bodies, and redacts bearer tokens in safe previews.
  - Android transport, token storage, product diagnostics, UI, and Media3
    ownership were not changed.
  - `cargo fmt --package taru-client-core --package taru-client-uniffi --check`
    passed after the Rust core and UniFFI scaffold updates.
  - `cargo nextest run -p taru-client-core --no-fail-fast` passed with 7 tests.
- 2026-05-21: Completed `SDKRT-035` UniFFI compile-only scaffold.
  - Added `crates/taru-client-uniffi` as a thin binding crate over
    `taru-client-core`.
  - The binding crate exposes FFI-safe records/enums/functions only and delegates
    behavior to `taru-client-core`.
  - No Android app behavior was wired in this milestone.
  - `cargo nextest run -p taru-client-uniffi --no-fail-fast` passed with 1 test.
  - `python -m json.tool docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json > $null`
    passed.
  - `git diff --check` passed. Git reported line-ending normalization warnings
    for touched workspace/workstream files.
