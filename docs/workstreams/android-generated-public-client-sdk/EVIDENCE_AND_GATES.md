# Android Generated Public Client SDK — Evidence And Gates

Status: Complete
Last updated: 2026-05-21

## Smallest Current Repro

```powershell
cargo nextest run -p taru-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon
```

These commands prove that the Rust generator and checked-in Kotlin package stay
in sync and that the generated Kotlin source compiles with serialization tests.

## Gate Set

### Targeted Iteration Gate

```powershell
cargo nextest run -p taru-api kotlin_sdk --no-fail-fast
apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon
```

### Android Consumption Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
```

Use a narrower `--tests` filter while migrating one Android route family, then
run the full app unit-test task before claiming the slice is done.

### Package And Closeout Gate

```powershell
cargo nextest run -p taru-api --no-fail-fast
npm run check --prefix sdk/typescript
apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon
git diff --check
```

`npm run check --prefix sdk/typescript` is included when shared generator or
OpenAPI changes might affect the existing TypeScript SDK contract.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to the review
note.

## Evidence Anchors

- `docs/workstreams/android-generated-public-client-sdk/DESIGN.md`
- `docs/workstreams/android-generated-public-client-sdk/TODO.md`
- `crates/taru-api/src/sdk.rs`
- `crates/taru-api/examples/emit-kotlin-sdk.rs`
- `sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`
- `sdk/kotlin/src/test/kotlin/dev/taru/sdk/TaruClientSdkTest.kt`
- `apps/android/settings.gradle.kts`

## Evidence Log

Record fresh command results here as tasks land.

### 2026-05-21 — AGKS-030/040 generated Kotlin SDK foundation

- PASS: `cargo run -q -p taru-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`
  - Proves `taru-api` can emit the checked Kotlin SDK package entry.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves generated Kotlin includes Public Client API constants/paths/wire
    types, excludes admin/internal/secret/raw-locator surfaces, and matches the
    checked-in package entry.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  - Proves the standalone generated Kotlin/JVM package compiles and decodes a
    representative generated DTO with kotlinx.serialization.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the wider `taru-api` OpenAPI, TypeScript SDK, admin contract, and
    Kotlin SDK tests still pass together.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves existing TypeScript SDK compile contract was not broken.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves Android JVM tests still pass after adding the sibling SDK module.
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - Proves Android debug assembly still works after Gradle settings changes.

### 2026-05-21 — AGKS-050 Android connection consumption slice

- PASS: `cargo run -q -p taru-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`
  - Regenerated the Kotlin SDK after adding request descriptor helpers.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves the checked Kotlin SDK still matches `taru-api`, includes generated
    request descriptor helpers, and excludes forbidden public-surface leaks.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  - Proves generated request descriptors compile and build `/health` plus
    `/libraries?limit=1&offset=0` with Kotlin tests.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`
  - Proves Android connection checks are still behaviorally correct while
    consuming generated SDK constants and low-risk request helpers through
    `TaruPublicApiContract`.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves the full Android JVM test suite still passes after adding the SDK
    module dependency.
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - Proves Android debug assembly still works with the app depending on
    `:taru-public-client-sdk`.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the wider API/OpenAPI/SDK contract tests remain green.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves TypeScript SDK compile contract remains green after shared SDK
    generator changes.
- PASS: `python -m json.tool docs/workstreams/android-generated-public-client-sdk/WORKSTREAM.json > $null`
  - Proves workstream machine-readable metadata remains valid JSON.
- PASS: `git diff --check`
  - Proves no whitespace errors; Git reported only expected Windows line-ending
    warnings.

### 2026-05-21 — Goal completion audit

- PASS: Current-state inspection of `WORKSTREAM.json`, `TODO.md`, `DESIGN.md`,
  `EVIDENCE_AND_GATES.md`, and `HANDOFF.md`
  - Proves the workstream exists, is active, records the fearless refactor plan,
    lists Android DTO/route drift findings, and carries the next executable
    task after the first consumption slice.
- PASS: `python -m json.tool docs/workstreams/android-generated-public-client-sdk/WORKSTREAM.json > $null`
  - Proves workstream machine-readable metadata is valid JSON.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves the Kotlin SDK generator/package sync, public-surface coverage, and
    leakage checks remain green.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  - Proves the generated Kotlin/JVM SDK package still compiles and its request
    descriptor/serialization tests pass.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves Android app unit tests still pass with the app consuming
    `:taru-public-client-sdk`.
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - Proves Android debug assembly still succeeds with the new SDK module.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the broader `taru-api` OpenAPI, public/admin contract, TypeScript
    SDK, and Kotlin SDK test set remains green.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves the existing TypeScript SDK compile contract remains green after the
    shared generator change.
- PASS: `git diff --check`
  - Proves no whitespace errors; Git reported only expected Windows line-ending
    warnings.

### 2026-05-21 — AGKS-060 browse route + listLibraries generated DTO slice

- PASS: `cargo run -q -p taru-api --example emit-kotlin-sdk -- --output sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`
  - Regenerated the checked Kotlin SDK after adding browse request descriptors
    and enum `wireValue` support.
- PASS: `cargo fmt --package taru-api --check`
  - Proves the touched Rust generator package remains formatted.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves Kotlin SDK generator/package sync, public-surface coverage, enum
    wire-value support, and leakage checks remain green.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the wider API/OpenAPI/SDK contract tests remain green after shared
    generator changes.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  - Proves generated browse request descriptors and strict
    `LibraryListResponse` decoding compile and pass Kotlin SDK tests.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon`
  - Proves Android browse tests still pass after all browse route construction
    moved to generated `TaruPublicClientRequests` and `listLibraries` decodes
    through generated DTOs before mapping to Android models.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves the full Android JVM unit-test suite remains green.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves TypeScript SDK compile contract remains green after shared generator
    changes.
- PASS: `git diff --check`
  - Proves no whitespace errors; Git reported only expected Windows line-ending
    warnings.

Notes:

- Replaced handwritten browse route construction was deleted from
  `TaruBrowseClient`; remaining route text matching is Android-owned failure
  classification.
- Remaining browse DTO mirror deletion is split to AGKS-065 because generated
  DTOs correctly require strict OpenAPI payloads that several historical Android
  fixtures do not yet provide.

### 2026-05-21 — AGKS-065 browse generated DTO adapter cleanup

- PASS: `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon`
  - Proves Android app source compiles after browse models stopped being
    serialization DTOs and browse/user-playback response clients map generated
    SDK DTOs into app presentation models.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon`
  - Proves browse/library/item/search/person/genre/tag success responses decode
    through strict generated SDK DTOs and map into Android models; fixtures now
    include full OpenAPI-required `LibraryOptionsDto` and
    `CanonicalMetadataDto` fields.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --no-daemon`
  - Proves the focused browse route family remains green after adapter
    extraction and fixture cleanup.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.userplayback.* --no-daemon`
  - Proves continue-watching response decoding moved through generated SDK DTOs
    because it shares browse media/page/image presentation models.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.* --tests dev.taru.android.userplayback.* --no-daemon`
  - Proves the combined migrated browse plus shared user-playback response
    surface remains green with fresh command evidence.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  - Proves the generated Kotlin/JVM SDK package still compiles and its local
    serialization/request-descriptor tests remain green.
- PASS: `cargo fmt --package taru-api --check`
  - Proves the touched Rust generator package remains formatted.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves Kotlin SDK generator/package sync, public-surface coverage, and
    leakage checks remain green after Android adapter migration.
- PASS: `git diff --check`
  - Proves no whitespace errors after removing a trailing blank line; Git
    reported only expected Windows line-ending warnings.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves the full Android JVM unit-test suite remains green after app model
    serializer removal, generated DTO adapters, and strict fixture cleanup.
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - Proves Android debug assembly still succeeds.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the broader `taru-api` OpenAPI, public/admin contract, TypeScript
    SDK, and Kotlin SDK test set remains green.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves the TypeScript SDK compile contract remains green after shared
    generator work.
- PASS: `python -m json.tool docs/workstreams/android-generated-public-client-sdk/WORKSTREAM.json > $null`
  - Proves workstream machine-readable metadata remains valid JSON.

Notes:

- Added `BrowseSdkAdapters.kt` as the explicit generated-SDK-to-Android-model
  seam; app UI state still imports Android-owned browse models.
- Removed `@Serializable`, `@SerialName`, `JsonElement`, and raw
  `MediaSourceDto.locator` from browse presentation models.
- Media probe/source-probe serializers intentionally remain until AGKS-070
  migrates playback response decoding; removing them in AGKS-065 would break the
  current playback client.

### 2026-05-21 — AGKS-070 playback and user-playback generated SDK boundary

- PASS: `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon`
  - Proves Android source compiles after playback/source-probe/session and
    user-playback request/response boundaries moved to generated SDK DTO
    adapters and after app presentation models stopped carrying wire
    serialization annotations.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.playback.* --tests dev.taru.android.userplayback.* --no-daemon`
  - Proves the focused AGKS-070 Android playback plus user-playback route
    families remain behaviorally green through public client seams.
  - The rerun after fixture hardening proves remux target construction no
    longer emits contract-invalid `output_container=hls`.
- PASS: `cargo fmt --package taru-api --check`
  - Proves the touched Rust generator package remains formatted.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves generated Kotlin SDK output still matches `taru-api`, includes the
    playback/user-playback request descriptors, and keeps leakage checks green.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --rerun-tasks --no-daemon`
  - Proves the generated Kotlin/JVM SDK package compiles from a clean rerun and
    its serialization/request-descriptor tests execute, not merely reuse cached
    Gradle results.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves the full Android JVM test suite remains green after shared
    media/playback/user-playback presentation model cleanup.
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - Proves Android debug assembly still succeeds.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the broader `taru-api` OpenAPI, public/admin contract, TypeScript
    SDK, and Kotlin SDK test set remains green after request descriptor
    additions.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves the TypeScript SDK compile contract remains green after shared
    generator work.
- PASS: `python -m json.tool docs/workstreams/android-generated-public-client-sdk/WORKSTREAM.json > $null`
  - Proves workstream machine-readable metadata remains valid JSON.
- PASS: `git diff --check`
  - Proves no whitespace errors; Git reported only expected Windows line-ending
    warnings.
- PASS: `rg "@Serializable|@SerialName|JsonElement|locator|PublicApiUrl\.encodePathSegment|PublicApiUrl\.pageQuery|PublicApiUrl\.queryString|/sources/|/users/me/playback-state|/playback/sessions|capabilitiesQuery|remuxQuery" apps/android/app/src/main/java/dev/taru/android/playback apps/android/app/src/main/java/dev/taru/android/media apps/android/app/src/main/java/dev/taru/android/userplayback apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt apps/android/app/src/main/java/dev/taru/android/browse/BrowseSdkAdapters.kt`
  - Only `PlaybackPreferences.kt` still contains `@Serializable`, which is an
    Android-owned persisted preferences model and not a Public Client API wire
    DTO.
  - Proves the AGKS-070 migrated playback/media/user-playback app models no
    longer carry wire serializers or raw Source Locator fields, and migrated
    route families no longer build source/session/user-playback paths by hand.

Notes:

- Added `PlaybackSdkAdapters.kt` and `MediaProbeSdkAdapters.kt` as generated
  SDK DTO to Android presentation seams.
- `TaruPlaybackClient` now builds source-probe, playback decision, stream,
  remux, HLS playlist/segment, playback session inspection, and cancellation
  requests from generated `TaruPublicClientRequests` descriptors.
- `TaruUserPlaybackClient` now builds state, continue-watching, progress, and
  watched routes from generated descriptors and encodes progress/watched bodies
  through generated SDK request DTOs.
- Removed `PlaybackMediaSourceDto.locator`; playback strict fixtures and preview
  JSON now align with OpenAPI `MediaSourceDto` by omitting raw locators and
  including required fields such as `fingerprint`.
- Kept Media3 launch/session state, safe request previews, failure categories,
  and product diagnostic copy Android-owned.

### 2026-05-21 — AGKS-080 deletion audit and public route mirror cleanup

- PASS: `apps/android/gradlew.bat -p apps/android :app:compileDebugKotlin --no-daemon`
  - Proves Android debug source compiles after deleting
    `TaruPublicApiContract`, `HealthEnvelope`, and the old `PublicApiUrl`
    route/query helper object.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`
  - Proves connection health/auth-probe still map success, unauthorized,
    unsupported-version, invalid-url/token, cleartext, TLS, transport, and
    sanitized diagnostics while using generated descriptors/constants directly.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --tests dev.taru.android.connection.TaruConnectionClientTest --no-daemon`
  - Proves browse plus connection behavior remains green after route-text 404
    classification was replaced with explicit app-owned request categories.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.taru.android.artwork.PublicArtworkTest --tests dev.taru.android.ui.artwork.ArtworkRequestResolverTest --no-daemon`
  - Proves artwork request creation still scopes URLs/tokens to the active
    server and now validates public image refs against generated
    `TaruPublicClientRequests.image(image.id)` descriptors.
- PASS: `cargo fmt --package taru-api --check`
  - Proves the touched Rust generator package remains formatted.
- PASS: `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
  - 3 tests passed.
  - Proves generated Kotlin SDK output still matches `taru-api` after Android
    deletion cleanup and generated descriptor consumption.
- PASS: `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
  - Proves the generated Kotlin/JVM SDK package still compiles and its
    descriptor/serialization tests remain green.
- PASS: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  - Proves the full Android JVM unit-test suite remains green after deleting
    the remaining replaced public API mirrors and helpers.
- PASS: `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
  - Proves Android debug assembly still succeeds after main/debug source route
    literal cleanup.
- PASS: `cargo nextest run -p taru-api --no-fail-fast`
  - 45 tests passed.
  - Proves the broader `taru-api` OpenAPI, public/admin contract, TypeScript
    SDK, and Kotlin SDK test set remains green.
- PASS: `npm run check --prefix sdk/typescript`
  - Proves the TypeScript SDK compile contract remains green.
- PASS: `python -m json.tool docs/workstreams/android-generated-public-client-sdk/WORKSTREAM.json > $null`
  - Proves workstream machine-readable metadata remains valid JSON.
- PASS: `rg "locator|/users/me|/sources|/libraries|/items" apps/android/app/src/main/java/dev/taru/android`
  - No matches.
  - Proves the AGKS-080 evidence pattern no longer finds migrated public route
    text or raw locator leakage in Android main source.
- PASS: `rg "TaruPublicApiContract|PublicApiUrl\.|pageQuery|queryString|encodePathSegment|capabilitiesQuery|remuxQuery|HealthEnvelope|JsonElement|parseToJsonElement" apps/android/app/src/main/java/dev/taru/android apps/android/app/src/debug/java/dev/taru/android apps/android/app/src/test/java/dev/taru/android -g "*.kt"`
  - No matches.
  - Proves the replaced public contract facade, handwritten route helpers, old
    connection health DTO mirror, and migrated DTO leak probes are gone from
    Android main/debug/test Kotlin sources.
- PASS: `rg '"/[^"]*(libraries|items|sources|users|playback|continue|health|people|genres|tags|images)' apps/android/app/src/main/java/dev/taru/android apps/android/app/src/debug/java/dev/taru/android -g "*.kt"`
  - No matches after the artwork and preview cleanup.
  - Proves main/debug Kotlin no longer keeps public route literals for the
    migrated families.
- PASS: `git diff --check`
  - Proves no whitespace errors; Git reported only expected Windows
    line-ending warnings.

Notes:

- Kept `PublicErrorEnvelope`, `ServerProfile`, `ServerProfileSnapshot`,
  `PlaybackPreferences`, `DevicePlaybackPosition`, and
  `TaruBrowseNavigationStateSaver` serializers because they are Android-owned
  diagnostics, profile persistence, preferences, local playback-position, or UI
  state persistence rather than generated Public Client API wire DTO mirrors.
- Connection health still uses an app-owned tolerant JSON version probe instead
  of the generated `HealthResponse` enum DTO so future unsupported server
  versions can surface compatibility diagnostics before strict enum decoding.

### 2026-05-21 — AGKS-090 closeout review and lane split

- PASS: Closeout review of `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, ADR 0025, ADR 0026,
  ADR 0031, and current `git diff --name-status`.
  - Workstream Compliance: no blocking or important findings.
  - Code Quality: no blocking or important findings.
  - Missing Gates: none; fresh AGKS-080 closeout gates are recorded above.
  - Residual Risk: publishing, KMP topology, Rust/UniFFI, unknown-enum/version
    tolerance, generated runtime ownership, and preview fixture ergonomics are
    split to follow-ons in `CLOSEOUT.md`.
- PASS: `python -m json.tool docs/workstreams/android-generated-public-client-sdk/WORKSTREAM.json > $null`
  - Proves the closed workstream metadata remains valid JSON.
- PASS: `git diff --check`
  - Proves no whitespace errors after closeout doc updates; Git reported only
    expected Windows line-ending warnings.

Notes:

- AGKS-090 is documentation and closeout only. The implementation gates were
  already rerun fresh in AGKS-080 before lane closeout.
- `WORKSTREAM.json` is now `closed`, `TODO.md` is complete, and `CLOSEOUT.md`
  records follow-ons and residual risks.

## Notes

- Fresh verification is required before marking a task, Codex goal, or lane
  complete.
- Generated Kotlin source must be refreshed through the generator command, not
  edited by hand.
- Android app migration should delete replaced handwritten mirrors; do not keep
  dual wire DTO sources after a route family moves.
