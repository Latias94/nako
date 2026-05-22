# Generated SDK Forward Compatibility Tolerance — Closeout

Status: Closed
Closed: 2026-05-21

## Closeout Claim

The generated SDK forward-compatibility tolerance lane is complete. Generated
Kotlin Public Client API string enums now preserve unknown future public wire
values instead of failing deserialization, and Android consumes those tolerant
values through app-owned adapters and diagnostics.

## Delivered

- `nako-api` Kotlin generator now emits generated string-enum surfaces as
  `@JvmInline @Serializable value class` wrappers.
- Generated wrappers expose:
  - raw `wireValue`;
  - generated known constants;
  - `KnownWireValues`;
  - `isKnown`.
- Checked-in `sdk/kotlin` output was regenerated from `nako-api`.
- Kotlin SDK tests prove known and unknown public wire string decode/encode
  behavior.
- Android connection health now decodes generated `HealthResponse` and still
  maps future body/header API versions to `UnsupportedApiVersion`.
- Android playback generated-SDK adapters compare known generated constants by
  `wireValue` and map unknown future values into safe app-owned fallbacks.
- Android UI and Media3 ownership did not move into the generated SDK.
- TypeScript remains compile-time oriented and runtime-unaffected in this lane;
  its compile gate passed.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- All five tasks are complete: `SDKFC-010`, `SDKFC-020`, `SDKFC-030`,
  `SDKFC-040`, and `SDKFC-090`.
- Fresh closeout gate evidence is recorded in `EVIDENCE_AND_GATES.md`.

### Code Quality

- Blocking: none.
- Important: none.
- Generated Kotlin source is synchronized from `nako-api` rather than edited by
  hand.
- Unknown public wire values remain public strings only; no admin/internal, raw
  Source Locator, storage URI, local path, provider-secret, or server-domain
  surfaces were introduced.
- Android app models keep product/presentation fallback policy outside the SDK.

### Missing Gates

- None.

## Residual Risks And Follow-Ons

These remain intentionally out of scope and should stay separate:

1. **Generated runtime ownership**
   - Whether HTTP execution, public error parsing, API-version header checks,
     and redaction should move from Android into a generated/runtime SDK layer.
2. **SDK publishing and binary policy**
   - Maven coordinates, artifact versioning, source/binary compatibility, and
     external consumer docs.
3. **Kotlin Multiplatform target state**
   - Whether this JVM-first package should become KMP before iOS or desktop
     clients consume it.
4. **Shared Rust client core / UniFFI**
   - Still gated by ADR 0031 triggers such as iOS, offline/download cache, or
     another native shell.
5. **Wider multi-SDK runtime tolerance**
   - TypeScript currently has compile-time unions but no runtime JSON decoder;
     future runtime SDKs should preserve additive public strings.

## Evidence Anchors

- `docs/workstreams/generated-sdk-forward-compat-tolerance/EVIDENCE_AND_GATES.md`
- `crates/nako-api/src/sdk.rs`
- `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`
- `sdk/kotlin/src/test/kotlin/dev/nako/sdk/NakoClientSdkTest.kt`
- `apps/android/app/src/main/java/dev/nako/android/connection/NakoConnectionClient.kt`
- `apps/android/app/src/main/java/dev/nako/android/playback/PlaybackSdkAdapters.kt`
