# Android Generated Public Client SDK — Closeout

Status: Closed
Closed: 2026-05-21

## Closeout Claim

The Android Generated Public Client SDK lane is complete. Android now consumes
an OpenAPI-backed generated Kotlin/JVM SDK for Public Client API DTOs, constants,
and route construction across the migrated connection, browse, media-probe,
playback, artwork, and user-playback route families.

The lane intentionally keeps Android-owned UI state, Compose navigation,
Media3 playback runtime, profile/token storage, product diagnostics, safe
request previews, and local persistence outside the generated SDK.

## Delivered

- `nako-api` emits checked Kotlin/JVM SDK source into `sdk/kotlin`.
- Android Gradle includes the generated package as `:nako-public-client-sdk`.
- Rust generator tests prove the checked Kotlin source matches the generator
  output and excludes admin/internal/secret/raw-locator surfaces.
- Kotlin SDK tests prove request descriptors and generated DTO serialization.
- Android clients build public route paths from `NakoPublicClientRequests`.
- Android response decoding enters through generated SDK DTOs and maps through
  explicit app adapters:
  - `BrowseSdkAdapters.kt`
  - `MediaProbeSdkAdapters.kt`
  - `PlaybackSdkAdapters.kt`
- Replaced Android handwritten public API mirrors were deleted:
  - `NakoPublicApiContract`
  - `HealthEnvelope`
  - `PublicApiUrl`
  - old route query/path helpers
  - route-string 404 classification
  - migrated wire serializers and raw `locator` fields.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- The target state from `DESIGN.md` is satisfied for this lane's scope.
- All AGKS tasks are complete and evidence is recorded in
  `EVIDENCE_AND_GATES.md`.
- Remaining work is split as follow-ons rather than hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- Android app models that remain serializable are local diagnostics, profile
  persistence, preferences, playback-position, or UI navigation state, not
  generated Public Client API wire DTO mirrors.
- The retained `PublicClientApiExecutor` and `NakoRequestDescriptor.urlOn`
  helper are Android transport/redaction seams, not public route authorities.

### Missing Gates

- None. Closeout gate evidence is recorded in `EVIDENCE_AND_GATES.md`.

## Follow-ons Split From This Lane

1. **Kotlin SDK publishing and binary policy**
   - Decide Maven coordinates, generated artifact versioning, release workflow,
     binary/source compatibility, and consumer docs.
2. **Kotlin Multiplatform SDK target state**
   - Decide whether `sdk/kotlin` should stay JVM-first or become KMP before iOS
     or desktop clients consume it.
3. **Shared Rust client core / UniFFI target state**
   - Start only when ADR 0031 triggers are met: iOS shell, offline/download
     cache coordination, Android TV/native second shell, or duplicated portable
     logic across Kotlin and Swift.
4. **Generated SDK unknown-enum and version-tolerance ergonomics**
   - Evaluate strict generated enums versus tolerant wrappers for API version
     negotiation and forward-compatible diagnostics.
5. **Generated runtime ownership**
   - Decide whether public error parsing, API-version header checks, request
     execution, and redaction should remain Android-owned or move into a
     generated/runtime SDK layer.
6. **Android preview and fixture DSL**
   - Reduce hand-authored strict OpenAPI JSON fixtures in previews/tests now
     that generated DTOs enforce full payload shapes.

## Residual Risks

- The generated Kotlin SDK is checked in and synchronized by tests, but package
  publishing is not designed yet.
- Android still owns transport execution and product diagnostics. That is
  intentional for this lane, but future multi-client work may justify moving a
  subset into shared SDK/runtime code.
- Connection health uses a small Android-owned tolerant JSON version probe
  instead of the generated strict `HealthResponse` enum DTO so future
  unsupported versions can produce compatibility diagnostics.

## Evidence Anchors

- `docs/workstreams/android-generated-public-client-sdk/EVIDENCE_AND_GATES.md`
- `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
- `sdk/kotlin/src/test/kotlin/dev/nako/sdk/NakoClientSdkTest.kt`
- `crates/nako-api/src/sdk.rs`
- `apps/android/app/src/main/java/dev/nako/android`
