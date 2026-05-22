# Android Generated Public Client SDK — TODO

Status: Complete
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

- [x] AGKS-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-generated-public-client-sdk]
  Goal: Freeze the problem, target state, non-goals, workstream authority, and first proof target.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: `docs/workstreams/android-generated-public-client-sdk/DESIGN.md`
  Handoff: Planner owns this before migration tasks start.

- [x] AGKS-020 [owner=planner] [deps=AGKS-010] [scope=apps/android/app/src/main/java/dev/nako/android,crates/nako-api/src]
  Goal: Record the current Android handwritten DTO/path drift inventory and migration decision spine.
  Validation: DESIGN.md lists priority findings and scoped migration risks.
  Evidence: `docs/workstreams/android-generated-public-client-sdk/DESIGN.md#priority-findings`
  Handoff: Treat the inventory as a starting map; add concrete findings as migration slices touch code.

## M1 — Generated Kotlin SDK Foundation

- [x] AGKS-030 [owner=codex] [deps=AGKS-020] [scope=crates/nako-api,sdk/kotlin,apps/android/gradle]
  Goal: Add the smallest generated Kotlin/JVM SDK package that compiles independently and is synchronized with `nako-api`.
  Validation: `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`; `apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon`
  Review: Confirm generated output is checked in and not edited by hand.
  Evidence: `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
  Handoff: Keep Android app consumption out of this task unless the package shape is proven.

- [x] AGKS-040 [owner=codex] [deps=AGKS-030] [scope=crates/nako-api/src/sdk.rs,sdk/kotlin]
  Goal: Add leakage, sync, and compile evidence for the generated Kotlin SDK foundation.
  Validation: Kotlin package tests plus Rust generator sync tests pass.
  Review: Ensure no admin/internal/addon/storage/provider-secret surfaces leak into generated Kotlin output.
  Evidence: `docs/workstreams/android-generated-public-client-sdk/EVIDENCE_AND_GATES.md`
  Handoff: Split generator ergonomics if inline enum/property handling expands.

## M2 — First Android Consumption Slice

- [x] AGKS-050 [owner=codex] [deps=AGKS-040] [scope=sdk/kotlin,apps/android/app/src/main/java/dev/nako/android/connection]
  Goal: Generate or expose request descriptors for health and auth-probe routes, then adapt Android connection checks to consume them.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.connection.NakoConnectionClientTest --no-daemon`
  Review: Android product error categories and cleartext/TLS policy must remain app-owned.
  Evidence: `apps/android/app/src/test/java/dev/nako/android/connection/NakoConnectionClientTest.kt`
  Handoff: This is the first app-consumption proof; stop if generated runtime ownership is unclear.

- [x] AGKS-060 [owner=codex] [deps=AGKS-050] [scope=sdk/kotlin,apps/android/app/src/main/java/dev/nako/android/browse]
  Goal: Migrate browse/library/item route construction to generated SDK request descriptors and land the first generated-DTO decoding adapter for `listLibraries`.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.* --no-daemon`
  Review: Replaced handwritten browse path/query construction is deleted from `NakoBrowseClient`; remaining handwritten browse DTO mirrors are explicitly split to AGKS-065 instead of hidden as dual-source cleanup.
  Evidence: `apps/android/app/src/test/java/dev/nako/android/browse/NakoBrowseClientTest.kt`; `sdk/kotlin/src/test/kotlin/dev/nako/sdk/NakoClientSdkTest.kt`
  Handoff: `listLibraries` decodes `dev.nako.sdk.LibraryListResponse` and maps into Android UI models. Continue DTO cleanup in AGKS-065 before claiming browse DTO deletion complete.

- [x] AGKS-065 [owner=codex] [deps=AGKS-060] [scope=apps/android/app/src/main/java/dev/nako/android/browse,apps/android/app/src/main/java/dev/nako/android/media,apps/android/app/src/main/java/dev/nako/android/userplayback,sdk/kotlin]
  Goal: Migrate remaining browse/library/item response decoding through generated SDK DTO adapters, then delete replaced handwritten wire DTO mirrors that are no longer app presentation models.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.browse.* --no-daemon`
  Review: Update fixtures to strict OpenAPI payloads instead of weakening generated DTOs; keep Android-owned UI/presentation models where they intentionally differ from wire DTOs.
  Evidence: `apps/android/app/src/main/java/dev/nako/android/browse/BrowseSdkAdapters.kt`; `apps/android/app/src/main/java/dev/nako/android/browse/BrowseModels.kt`; `apps/android/app/src/main/java/dev/nako/android/userplayback/NakoUserPlaybackClient.kt`
  Handoff: Browse/library/item/search/person/genre/tag response decoding now enters through generated SDK DTOs and maps into Android presentation models. User-playback continue-watching also uses generated SDK adapters because it shares browse media/image/page models. Media probe serializers remain only for playback/source-probe until AGKS-070 migrates playback.

- [x] AGKS-070 [owner=codex] [deps=AGKS-065] [scope=sdk/kotlin,apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/main/java/dev/nako/android/userplayback]
  Goal: Migrate playback decision/session and user playback state DTO/route construction through generated SDK boundaries.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --tests dev.nako.android.playback.* --tests dev.nako.android.userplayback.* --no-daemon`
  Review: Media3 player state and playback diagnostics presentation remain Android-owned; source-probe/playback/session responses now decode generated SDK DTOs through adapters, and user-playback request bodies use generated SDK request DTOs.
  Evidence: `apps/android/app/src/main/java/dev/nako/android/playback/PlaybackSdkAdapters.kt`; `apps/android/app/src/main/java/dev/nako/android/media/MediaProbeSdkAdapters.kt`; `apps/android/app/src/test/java/dev/nako/android/playback/NakoPlaybackClientTest.kt`; `apps/android/app/src/test/java/dev/nako/android/userplayback/NakoUserPlaybackClientTest.kt`
  Handoff: Playback and user-playback route construction now uses `NakoPublicClientRequests`; Android-owned presentation/diagnostic/player state remains outside the generated SDK.

## M3 — Deletion And Closeout

- [x] AGKS-080 [owner=codex] [deps=AGKS-070] [scope=apps/android/app/src/main/java/dev/nako/android]
  Goal: Delete replaced handwritten Public Client API DTO mirrors, path constants, and route query helpers.
  Validation: `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
  Review: Verify no migrated family still imports old DTO names.
  Evidence: `rg "locator|/users/me|/sources|/libraries|/items" apps/android/app/src/main/java/dev/nako/android`
  Handoff: `NakoPublicApiContract`, `HealthEnvelope`, `PublicApiUrl`, and route-text 404 classification were removed. Android presentation, persistence, diagnostics, token redaction, Media3 runtime, and local UI state remain app-owned.

- [x] AGKS-090 [owner=codex] [deps=AGKS-080] [scope=docs/workstreams/android-generated-public-client-sdk]
  Goal: Close the lane or split publishing/KMP/Rust-FFI follow-ons.
  Validation: Fresh final gate evidence is recorded.
  Review: `review-workstream` and `verify-rust-workstream` before completion claims.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, `CLOSEOUT.md`
  Handoff: Lane closed. Publishing, KMP, Rust/UniFFI, unknown-enum/version tolerance, generated runtime ownership, and preview fixture ergonomics are split follow-ons.
