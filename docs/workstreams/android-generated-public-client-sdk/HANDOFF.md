# Android Generated Public Client SDK — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The workstream is closed. ADR 0031 selected generated Kotlin SDK work before
mobile Rust/UniFFI. The initial inventory showed Android-owned handwritten
Public Client API DTOs and route paths across connection, browse, playback,
media probe, and user playback packages.

`taru-api` now emits a checked Kotlin/JVM SDK entry at
`sdk/kotlin/src/main/kotlin/dev/taru/sdk/TaruClientSdk.kt`. The package is
included in the Android Gradle build as `:taru-public-client-sdk` and has local
serialization/contract tests.

Android app code now consumes generated SDK constants and low-risk request
descriptors directly for connection health/auth-probe; the temporary
`TaruPublicApiContract` wrapper has been deleted. Browse/library/item/search/person/genre/tag routes are built through generated
`TaruPublicClientRequests`; their success responses decode generated SDK DTOs
first and then map into Android presentation models in
`BrowseSdkAdapters.kt`. Continue-watching responses also decode generated SDK
DTOs because they reuse browse media/image/page models.

AGKS-070 extends that boundary to playback and user playback. Source-probe,
playback-decision, and playback-session responses decode generated SDK DTOs
through `MediaProbeSdkAdapters.kt` and `PlaybackSdkAdapters.kt`. Playback
stream, remux, HLS playlist/segment, session inspection/cancel, and
user-playback state, continue-watching, progress, and watched routes are built
by generated `TaruPublicClientRequests`. User-playback progress/watched request
bodies are encoded from generated SDK request DTOs. Product diagnostics,
cleartext/TLS policy, profile storage, token redaction, Compose state, and
Media3 playback runtime remain Android-owned.

AGKS-080 completed the repo-wide deletion audit. `HealthEnvelope`,
`TaruPublicApiContract`, `PublicApiUrl`, and route-text 404 classification were
removed. Image requests, debug smoke seeding, previews, diagnostics defaults,
safe token redaction, and tests now use generated SDK constants/request
descriptors where public API route shape is involved.

## Active Task

- None. The lane is closed.
- Evidence: AGKS-090 closeout evidence is recorded in
  `EVIDENCE_AND_GATES.md` and `CLOSEOUT.md`.

## Decisions Since Last Update

- Use `sdk/kotlin` as the generated package location so the artifact is not
  Android-app-private.
- Keep Android `PublicClientApiExecutor` and product diagnostics app-owned for
  the first consumption slice.
- Do not introduce Rust/UniFFI or KMP in this lane's first milestone.
- The first generated package is JVM/Kotlin, not Android-library or KMP, to keep
  the compile proof small and avoid platform resource/package complexity.
- Android connection consumption initially went through `TaruPublicApiContract`
  as a migration seam; AGKS-080 deleted that wrapper after generated descriptor
  adoption was broad enough.
- Generated Kotlin enums now expose `wireValue` so Android adapters can map
  strict generated enums back into app presentation models without reflecting on
  `@SerialName`.
- `TaruBrowseClient` now builds browse/library/item/search paths through
  generated `TaruPublicClientRequests`; 404 request classification is explicit
  and no longer pattern-matches path text.
- `listLibraries` now decodes `dev.taru.sdk.LibraryListResponse` and maps into
  Android `LibraryListResponse` so UI state does not depend directly on wire
  DTOs.
- AGKS-065 created `BrowseSdkAdapters.kt` as the explicit generated-SDK-to-app
  boundary. Android browse models no longer carry `@Serializable`,
  `@SerialName`, `JsonElement`, or raw `MediaSourceDto.locator`; they are now
  presentation models, not wire DTO mirrors.
- Strict fixture cleanup fixed full `LibraryOptionsDto` and
  `CanonicalMetadataDto` payloads instead of weakening generated DTOs.
- Continue-watching response decoding moved through generated SDK adapters as a
  consequence of removing serializers from shared browse media/page/image app
  models.
- AGKS-070 removed media probe/source-probe serializers after playback switched
  to generated DTO decoding.
- AGKS-080 deletes the temporary generated-SDK facade
  `TaruPublicApiContract`; app code now imports generated constants and request
  descriptors directly when it needs public contract values.
- `HealthEnvelope` was removed. Connection health now uses the generated
  `/health` descriptor and a small app-owned tolerant JSON version probe so a
  future unsupported API version can still map to compatibility diagnostics
  instead of strict enum decode failure.
- `PublicApiUrl` and the old page/query/path encoding helpers were removed.
  App code uses generated descriptors plus the small `TaruRequestDescriptor.urlOn`
  transport join helper.
- Browse 404 category selection no longer pattern-matches route strings;
  request methods pass explicit Android-owned not-found categories.
- Artwork requests now validate server-supplied image refs against generated
  `TaruPublicClientRequests.image(image.id)` descriptors instead of manually
  constructing `/images/...` URLs.

## Blockers

- None known.

## Follow-ons

- Kotlin SDK publishing and binary policy.
- Kotlin Multiplatform SDK target state.
- Shared Rust client core / UniFFI target state after ADR 0031 triggers.
- Generated SDK unknown-enum and API-version tolerance ergonomics.
- Generated runtime ownership for error parsing/version checks/redaction.
- Android preview and fixture DSL for strict OpenAPI payloads.

Keep Android presentation models, Media3 runtime, diagnostics, and UI state
app-owned unless a future ADR changes that.
