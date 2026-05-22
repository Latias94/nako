# Generated SDK Forward Compatibility Tolerance — Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

The lane is closed. `nako-api` now generates tolerant Kotlin Public Client API
string enum wrappers instead of strict `enum class` definitions. Checked-in
`sdk/kotlin` output was regenerated from the generator.

Generated string enum surfaces are now `@JvmInline @Serializable value class`
types with:

- `wireValue: String`
- companion-object known constants
- `KnownWireValues`
- `isKnown`

Unknown future public string values decode and encode without losing raw
`wireValue`.

## Active Task

None. `SDKFC-010`, `SDKFC-020`, `SDKFC-030`, `SDKFC-040`, and `SDKFC-090` are
complete.

## Decisions

- This lane stayed about generated SDK forward compatibility, not SDK
  publishing, KMP topology, generated runtime ownership, or Rust/UniFFI.
- The chosen representation is a generated tolerant wire-string value type with
  known constants and raw `wireValue` preservation.
- Android adapters remain responsible for product diagnostics and UI fallback
  semantics.
- OpenAPI remains the authority; tolerant decoding must not permit internal
  surface leakage.
- Android connection health now decodes generated `HealthResponse` and maps
  future body versions to `UnsupportedApiVersion`.
- Android playback maps unknown generated values into app-owned fallback states;
  unknown playback mode prepares no target and yields `UnsupportedSource`.

## Blockers

None.

## Verification

```powershell
cargo fmt --package nako-api --check
cargo nextest run -p nako-api kotlin_sdk --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
npm run check --prefix sdk/typescript
apps/android/gradlew.bat -p apps/android :nako-public-client-sdk:test --no-daemon
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon
python -m json.tool docs/workstreams/generated-sdk-forward-compat-tolerance/WORKSTREAM.json > $null
git diff --check
```

All passed on 2026-05-21; see `EVIDENCE_AND_GATES.md`.

## Follow-ons

- Generated runtime ownership for HTTP execution, public error parsing,
  API-version header checks, and redaction.
- SDK publishing and binary/source compatibility policy.
- Kotlin Multiplatform target state.
- Shared Rust client core / UniFFI after ADR 0031 triggers.
- Wider multi-SDK runtime tolerance for future TypeScript, Swift, Dart, or Rust
  SDK runtime decoders.
