# Generated SDK Forward Compatibility Tolerance — Design

Status: Completed
Last updated: 2026-05-21

## Context

ADR 0025 makes the Public Client API OpenAPI v1 contract the authority for
generated client SDKs. ADR 0031 intentionally sequences generated SDK adoption
before mobile Rust/UniFFI. The completed Android Generated Public Client SDK
lane moved Android route construction and Public Client API DTO decoding through
`sdk/kotlin`, with app-owned adapters handling presentation and diagnostics.

That target state is cleaner, but before this lane the Kotlin generator emitted
strict Kotlin `enum class` definitions for OpenAPI string enums:

- Schema enums, for example `ErrorResponseCode` and `RemuxOutputContainer`.
- Property-specific enums, for example `HealthResponseVersion`,
  `ClientPlaybackDecisionMode`, `ClientTranscodePlanOutputContainer`,
  `TranscodeSessionDtoState`, and `ClientMediaKind`.

Strict enum decoding is useful when a value is truly invalid. It is harmful when
the Public Client API grows by adding a new public string value that an older
client can still safely preserve, display generically, or classify as
unsupported.

## Problem

The generated Kotlin SDK previously conflated two failure classes:

1. malformed or contract-invalid JSON;
2. well-formed Public Client API JSON containing an unknown future string value.

For a long-lived self-hosted media server, that distinction matters:

- A future server may return `version = "v2"` on `/health`; the Android app
  should report an API compatibility problem, not a generic invalid response.
- A future playback or transcode value may be safe to show as an unknown state
  or generic unavailable decision while preserving the raw wire value for
  diagnostics.
- A future public error code should not prevent decoding the rest of the public
  error envelope.
- Generated SDK callers need a stable way to compare known values while also
  preserving unknown values for logs, diagnostics, or fallback UI.

## Target State

When this lane closes:

- Generated Kotlin Public Client API wire types preserve unknown string-enum
  wire values instead of failing deserialization.
- Known values remain ergonomic, generated, and strongly named.
- Unknown values carry their raw `wireValue` and can be compared or redacted
  safely by app adapters.
- `/health.version` and `x-nako-api-version` observation can produce explicit
  unsupported-version diagnostics instead of generic invalid-response failures.
- Android adapters map unknown values at the app boundary, not in Compose UI or
  Media3 playback runtime.
- Rust generator tests, Kotlin SDK tests, and Android tests prove the new
  behavior without weakening leakage or route-contract checks.

Satisfied on 2026-05-21.

## In Scope

- `crates/nako-api/src/sdk.rs` Kotlin generator changes for public string-enum
  representation and serialization.
- Regenerated checked Kotlin output in `sdk/kotlin`.
- Kotlin SDK tests for known and unknown enum/value decoding.
- Android adapter updates where generated enum strictness shaped app diagnostics
  or presentation models.
- Documentation of the compatibility policy and any follow-on runtime ownership
  questions.

## Out Of Scope

- Changing OpenAPI schemas to remove enum vocabularies.
- Moving HTTP transport, bearer-token redaction, safe request previews, or
  public error parsing into a generated runtime.
- Publishing the Kotlin SDK or changing it to KMP.
- Introducing Rust/UniFFI into Android.
- Server route behavior changes, Admin API behavior, Addon Protocol behavior,
  or Public Client API v2 design.

## Frozen Representation Decision

`SDKFC-010` froze a generated tolerant wire-string value type over Kotlin
`enum class` for Public Client API string enums:

- Generate `@JvmInline @Serializable public value class <Name>(public val wireValue: String)`.
- Generate companion-object known constants for each OpenAPI enum value.
- Decode any JSON string into the wrapper, preserving unknown values.
- Encode the raw `wireValue` back to JSON for request DTOs and round-trip tests.
- Generate `KnownWireValues` and `isKnown` as low-policy helpers for diagnostics
  and SDK tests.

This direction is intentionally different from adding an `Unknown` enum member:
Kotlin `enum class` cannot carry arbitrary future raw values in each unknown
instance, and a single sentinel would discard diagnostics.

TypeScript remains compile-time oriented in this lane. Its generated union
types still document known values, but it does not perform runtime JSON enum
validation. Future runtime TypeScript clients should follow the same principle:
preserve additive public wire strings instead of throwing away raw values.

## Boundary Decisions

- OpenAPI remains the public contract authority. Tolerant SDK decoding is a
  client compatibility policy, not permission for server internals to leak.
- Android app adapters remain responsible for product categories, messages,
  accessibility copy, and presentation fallbacks.
- The generated SDK may expose raw public wire strings, but not local paths,
  storage handles, Source Locators, provider-secret data, or internal server
  type names.
- Known request-side enum values remain generated constants; callers should not
  handwrite contract strings.
- Unknown response-side values should never crash normal app state mapping.
  Adapters choose an explicit fallback or compatibility/error category.

## Priority Findings

### P0 — Decode correctness

- `HealthResponse.version` was the highest-risk surface because the Android
  connection flow must classify future API versions as compatibility failures.
- `ErrorResponse.code` is a public error-envelope surface; future codes should
  not prevent callers from reading `message`.
- Playback/transcode enums shape user-visible decisions and diagnostics; Android
  now maps unknown generated values into app-owned `Unknown`/fallback states.
- The generated Kotlin enum inventory for this lane was:
  `RemuxOutputContainer`, `ClientPlaybackDecisionMode`,
  `ClientTranscodePlanHardwareAcceleration`,
  `ClientTranscodePlanOutputContainer`, `ErrorResponseCode`,
  `HealthResponseVersion`, `LibraryOptionsDtoDomain`,
  `LibraryOptionsDtoNamingStrategy`, `LibraryOptionsDtoPreset`,
  `MetadataProfileDtoLocalMetadataPolicy`, `MetadataProfileDtoRefreshMode`,
  `TranscodeSessionDtoFailureCategory`, `TranscodeSessionDtoKind`,
  `TranscodeSessionDtoState`, and `ClientMediaKind`.

### P1 — Ergonomics and migration risk

- Android adapters used Kotlin enum equality/exhaustive `when` patterns in
  playback. They now compare generated constants by `wireValue` and provide
  app-owned unknown fallbacks where the presentation model needs one.
- Request DTOs and query helpers still use generated known-value constants so
  callers do not handwrite public contract strings.
- Property-specific enum names remain generated and clear.

### P2 — Multi-SDK consistency

- TypeScript's generated contract is compile-time oriented and does not perform
  runtime strict JSON decoding in the same way; its compile gate passed.
- Future KMP, Swift, Dart, or Rust client SDK work should reuse the principle:
  decode additive public wire strings without losing raw values.

## Validation Strategy

- Rust generator tests prove the checked-in Kotlin SDK matches generated output
  and retains leak checks.
- Kotlin SDK tests decode JSON with both known and artificial unknown values for
  representative schema and property enums.
- Android tests prove connection health reports unsupported API versions and
  playback adapters handle unknown public string values safely.
- TypeScript check runs at closeout to guard accidental drift.

## Closeout Summary

Closed on 2026-05-21. The target state is satisfied for this lane:

- `nako-api` emits tolerant Kotlin value classes for generated Public Client API
  string enums.
- `sdk/kotlin` decodes and encodes unknown public wire strings while preserving
  raw `wireValue`.
- Android connection health uses generated `HealthResponse` instead of a bespoke
  raw JSON version probe, while still reporting unsupported API versions as
  app-owned diagnostics.
- Android playback adapters map future unknown generated values into safe
  app-owned fallbacks and avoid Media3/UI ownership changes.
- TypeScript remains unaffected at runtime and passed its compile gate.

## Closeout Condition

This lane can close when the generator, checked-in Kotlin SDK, and Android
consumption tests prove tolerant public string-enum/API-version behavior, the
task ledger is complete, and any remaining runtime ownership or publishing
questions are split as separate follow-ons.

Satisfied on 2026-05-21. See `EVIDENCE_AND_GATES.md` and `CLOSEOUT.md`.
