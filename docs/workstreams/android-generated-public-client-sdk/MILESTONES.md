# Android Generated Public Client SDK — Milestones

Status: Complete
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- ADR 0025, ADR 0026, and ADR 0031 are linked.
- Android handwritten DTO/path drift is recorded.
- The first proof target is the generated Kotlin/JVM package, not Android app
  migration or Rust/UniFFI.

Primary evidence:

- `docs/workstreams/android-generated-public-client-sdk/DESIGN.md`
- `docs/workstreams/android-generated-public-client-sdk/TODO.md`

## M1 — Generated Kotlin SDK Foundation

Status: Complete as of 2026-05-21.

Exit criteria:

- `taru-api` can emit Kotlin SDK source from Public OpenAPI v1.
- `sdk/kotlin` contains checked-in generated source and local package metadata.
- Rust tests prove generator/package synchronization.
- Kotlin/Gradle tests compile and smoke-check serialization behavior.
- Generated output passes public-surface leakage checks.

Primary gates:

- `cargo nextest run -p taru-api kotlin_sdk --no-fail-fast`
- `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`

## M2 — First Android Consumption Slice

Status: Complete as of 2026-05-21.

Exit criteria:

- Android consumes the generated SDK for at least one low-risk Public Client API
  vertical route family.
- Product diagnostics, profile storage, token vaulting, and UI state remain
  Android-owned.
- Replaced handwritten route/DTO code is deleted rather than kept in parallel.

Progress:

- AGKS-050 moved connection health/auth-probe paths to generated request
  descriptors through `TaruPublicApiContract`.
- AGKS-060 moved browse route construction to generated
  `TaruPublicClientRequests` descriptors and landed the first generated DTO
  adapter for `listLibraries`.
- AGKS-065 moved remaining browse/library/item/search/person/genre/tag response
  decoding through generated SDK DTO adapters, kept Android presentation models
  app-owned, and fixed strict OpenAPI-aligned fixtures for required
  `LibraryOptionsDto` and `CanonicalMetadataDto` fields.
- Continue-watching response decoding moved with AGKS-065 because it shares the
  same browse media/page/image presentation models.
- AGKS-070 moved playback decision/source-probe/session response decoding and
  playback/user-playback route construction through generated SDK descriptors
  and DTO adapters. Android Media3 launch/session diagnostics remain app-owned,
  while `PlaybackMediaSourceDto.locator` and media/user-playback wire
  serializers were removed from app presentation models.

Primary gates:

- Focused Android unit tests for the migrated family.
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`

## M3 — Deletion And Closeout

Status: Complete as of 2026-05-21.

Exit criteria:

- Migrated route families have no duplicate handwritten wire DTOs.
- Workstream gates are fresh and recorded.
- Remaining publishing, KMP, route-helper, or Rust/UniFFI work is split into
  follow-ons instead of hidden in this lane.
- `WORKSTREAM.json` status reflects reality.

Primary gates:

- `cargo nextest run -p taru-api --no-fail-fast`
- `apps/android/gradlew.bat -p apps/android :taru-public-client-sdk:test --no-daemon`
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon`
- `git diff --check`

Closeout:

- `CLOSEOUT.md` records the final review, follow-ons, and residual risks.
- `WORKSTREAM.json` is closed.
