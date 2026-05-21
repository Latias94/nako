# Android Generated Public Client SDK

Status: Completed
Last updated: 2026-05-21

## Why This Lane Exists

Android now has a cleaner client foundation, but it still mirrors Public Client
API DTOs and HTTP paths by hand. That leaves the app exposed to drift every time
the Public Client API grows. ADR 0031 intentionally defers Rust/UniFFI until the
generated SDK contract is stable, so the next architectural move is an
OpenAPI-backed Kotlin/JVM SDK that Android can consume through narrow adapters.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/api/HTTP_API.md`
- Related workstreams:
  - `docs/workstreams/openapi-client-contract/`
  - `docs/workstreams/sdk-client-scaffold/`
  - `docs/workstreams/typescript-sdk-package/`
  - `docs/workstreams/rust-client-sdk/`
  - `docs/workstreams/android-fearless-client-refactor/`

## Problem

Android still duplicates Public Client API knowledge in handwritten Kotlin
files:

- DTO mirrors live under `apps/android/app/src/main/java/dev/taru/android`
  packages such as `browse`, `media`, `playback`, `userplayback`, and
  `connection`.
- Route path and query construction is repeated in `TaruBrowseClient`,
  `TaruPlaybackClient`, `TaruUserPlaybackClient`, and
  `TaruConnectionClient`.
- Public constants such as `x-taru-api-version`, `x-taru-playback-session-id`,
  `/health`, and the auth probe path are repeated in Android code.
- Android's `MediaSourceDto` and `PlaybackMediaSourceDto` still contain a
  defaulted `locator` field even though the current OpenAPI `MediaSourceDto`
  intentionally does not expose a raw Source Locator.
- Android's catalog DTOs are intentionally partial in places, for example
  `CanonicalMetadataDto` lacks the OpenAPI `credits`, `collections`, `studios`,
  and `external_ids` arrays, and `LibraryOptionsDto` is reduced compared with
  the OpenAPI schema.
- Some Android fields use `JsonElement` escape hatches where the OpenAPI
  contract now uses concrete strings or objects.

These problems are manageable while the app is small, but they create a hidden
compatibility tax before downloads, Android TV, iOS, or shared Rust client core
work begins.

## Target State

When this lane closes:

- `taru-api` generates a checked Kotlin/JVM SDK package from Public OpenAPI v1.
- The generated package owns public DTO mirrors, contract constants, and route
  construction helpers for Public Client API calls.
- Android consumes the generated SDK through app-owned adapters while keeping
  Compose UI state, navigation, Media3, platform permissions, and diagnostics
  presentation outside the SDK.
- Handwritten Android DTO and path mirrors are deleted as their slices migrate.
- Contract drift is caught by Rust generator tests, Kotlin compile tests,
  Android unit tests, and leakage checks.

## Closeout Summary

Closed on 2026-05-21. The target state is satisfied for this lane's scope:

- `taru-api` generates checked Kotlin/JVM SDK source in `sdk/kotlin`.
- Android consumes generated constants, request descriptors, and generated DTOs
  through explicit app adapters.
- Replaced route/path/DTO mirrors were deleted rather than kept in parallel.
- Android UI state, Media3 playback runtime, platform diagnostics, profile and
  token storage, and local persistence remain app-owned.
- Publishing, KMP topology, Rust/UniFFI, unknown-enum/version tolerance, and
  generated runtime ownership are split to follow-ons in `CLOSEOUT.md`.

## In Scope

- A `sdk/kotlin` generated package and Gradle compile/test gate.
- Rust generator support in `taru-api` for Kotlin DTOs and SDK foundation code.
- Tests that prove generated Kotlin output is synchronized with the generator.
- Public Client API leakage checks for the generated Kotlin output.
- Android adapter migration from handwritten DTO/route clients to generated SDK
  slices.
- Documentation of migration order and remaining drift risks.

## Out Of Scope

- Mobile Rust/UniFFI integration.
- Kotlin Multiplatform target-state design.
- iOS client work.
- Android player ownership changes; Media3 remains Android-owned.
- Public Client API semantic expansion unrelated to SDK generation.
- Maven publishing, binary compatibility policy, or external package release.
- Admin API or addon protocol SDK generation.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public OpenAPI v1 is stable enough to generate a first Kotlin compile contract. | High | ADR 0025 and completed OpenAPI/TypeScript/Rust SDK workstreams. | Reopen OpenAPI contract shape before Android adoption. |
| Android should not depend on Rust/UniFFI during ordinary app builds yet. | High | ADR 0031. | Split a Rust/UniFFI target-state workstream before changing Android build topology. |
| A JVM Kotlin SDK package is the clean first artifact for Android consumption. | Medium | Android uses Kotlin serialization and Gradle already has Kotlin tooling. | If KMP becomes mandatory, split package topology before app migration. |
| Existing Android `PublicClientApiExecutor` remains useful as a transport/redaction seam during early migration. | High | Android foundation refactor centralized version, error, TLS, and redaction policy. | Move transport/error runtime into the generated SDK earlier. |

## Architecture Direction

The generated Kotlin SDK is a protocol boundary, not an Android application
framework.

- `taru-api` owns generation from `public_openapi_v1()`.
- `sdk/kotlin` owns generated public DTOs, constants, and eventually route
  builders/client request shapes.
- Android app adapters own user-facing error categories, product copy,
  diagnostics presentation, and profile/token storage.
- Existing Android transport and redaction policy may wrap generated request
  descriptors until a generated runtime proves it can replace them cleanly.
- Media3 playback request execution remains Android-owned; generated code may
  describe safe Public Client API requests, but not player state or sessions.

## Priority Findings

### P0 — Contract drift blockers

- Android has multiple DTO mirrors for OpenAPI schemas.
- Android route construction is scattered across feature clients.
- Android still has defaulted `locator` fields that should not be part of the
  Public Client API contract.

### P1 — Migration risks

- Android's partial DTOs may hide newly required OpenAPI fields until runtime.
- Existing browse/playback/user-playback failure categories contain product
  semantics that should not move into the generated SDK.
- Generated route helpers must preserve token-safe request preview behavior.
- Generated DTO adoption is intentionally stricter than the old Android
  handwritten mirrors. When a fixture lacks required OpenAPI fields such as
  `LibraryOptionsDto.metadata_profile`, `LibraryOptionsDto.naming_strategy`, or
  `CanonicalMetadataDto.external_ids`, fix the fixture/model boundary instead
  of weakening the generated SDK.

### P2 — Later ergonomics

- Generated Kotlin property-specific enum types expose `wireValue` for adapter
  mapping; revisit only if Android needs unknown-enum tolerance.
- Publishing and KMP support should wait until Android proves the package shape.

## Closeout Condition

This lane can close when:

- Android no longer owns handwritten Public Client API DTO mirrors for migrated
  route families.
- Generated Kotlin SDK output is synchronized and compile-checked.
- At least one Android vertical route family consumes the generated SDK.
- Old route/path construction for migrated families is deleted.
- Evidence gates pass and follow-on work is either completed or split.

Satisfied on 2026-05-21. See `CLOSEOUT.md` and `EVIDENCE_AND_GATES.md`.
