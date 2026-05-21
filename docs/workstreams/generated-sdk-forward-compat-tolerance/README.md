# Generated SDK Forward Compatibility Tolerance

Status: Completed
Last updated: 2026-05-21

## Why This Lane Exists

The Android Generated Public Client SDK lane closed with Android consuming the
OpenAPI-backed Kotlin/JVM SDK for route construction and DTO decoding. That
removed handwritten drift, but it also made a remaining compatibility risk more
visible: generated Kotlin string enums are currently strict `enum class` types.
An additive server enum value or future `/health.version` value can fail DTO
decoding before the app can produce useful Public Client API compatibility
diagnostics.

This lane exists to make generated Public Client SDK decoding tolerant where the
wire contract is intentionally versioned or additive, while preserving strict
contract leakage checks and keeping Android UI, Media3, and product diagnostics
outside the generated SDK.

## Goals

- Define the generated SDK policy for unknown public string-enum values and
  API-version observation.
- Update the Kotlin SDK generator and checked-in package so generated public
  wire types can preserve unknown raw string values without becoming invalid
  responses.
- Keep known-value ergonomics good enough for Android adapters and future SDK
  consumers.
- Prove Android can map unknown/generated-tolerant values into safe
  app-owned diagnostics or presentation fallbacks.
- Preserve Public Client API leakage checks: no admin/internal/provider-secret,
  raw Source Locator, storage URI, local path, or server-domain type exposure.

## Non-Goals

- No Public Client API v2 semantic redesign.
- No server behavior or route expansion unless tests need fixture-only examples.
- No generated runtime ownership decision for HTTP execution, bearer redaction,
  error parsing, or product diagnostics.
- No Maven publishing, binary compatibility policy, or KMP topology change.
- No Rust/UniFFI mobile client core.
- No Android UI or Media3 playback ownership changes.

## Authoritative Docs

- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
- `docs/workstreams/android-generated-public-client-sdk/CLOSEOUT.md`
- `docs/workstreams/generated-sdk-forward-compat-tolerance/DESIGN.md`
- `docs/workstreams/generated-sdk-forward-compat-tolerance/TODO.md`

## Closeout Summary

Closed on 2026-05-21. Generated Kotlin Public Client API string enums now use
tolerant value-class wrappers that preserve unknown raw `wireValue`s, expose
generated known constants, and keep Android diagnostics/presentation policy
app-owned.

## First Executable Slice

Completed as `SDKFC-010`: inventory every generated Kotlin enum and strict
decode surface, then freeze the tolerant representation decision before
modifying the generator.
