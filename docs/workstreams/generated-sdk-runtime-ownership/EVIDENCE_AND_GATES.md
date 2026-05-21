# Generated SDK Runtime Ownership — Evidence And Gates

Status: Active
Last updated: 2026-05-21

## Evidence Anchors

- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
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

No implementation evidence yet. The lane is open for `SDKRT-010` planning and
ownership freeze.
