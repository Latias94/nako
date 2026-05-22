# Android Rust Core Runtime Hardening

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

`generated-sdk-runtime-ownership` proved the first no-socket Rust client core
and UniFFI Android tracer for connection checks. Its closeout deliberately
split the next four risks into follow-ons:

1. Android Rust/UniFFI build ergonomics.
2. `nako-client` reuse of `nako-client-core`.
3. Rust public wire tolerance.
4. Android playback core tracer.

These four belong in one serialized hardening lane because each depends on the
previous boundary being clean. Playback must not move behind UniFFI until Rust
wire values are tolerant, and broader Rust reuse should happen before Android
and `nako-client` grow two versions of request/error/version/redaction policy.

## Target State

- Android Gradle no longer builds every Rust Android ABI during every ordinary
  `preBuild` or JVM unit-test path. Binding generation, host test library, and
  packageable Android ABI libraries are separate, incremental, documented
  tasks.
- `nako-client-core` owns portable request-spec construction, bearer injection,
  API-version observation, public error-envelope parsing, and safe request
  previews for core-owned routes.
- `nako-client` becomes the reqwest/async adapter over that core-owned
  request/response policy instead of a second Rust implementation.
- Rust Public Client API string-value DTOs preserve unknown additive wire
  values instead of failing deserialization.
- Android playback uses the Rust core / UniFFI boundary for playback decision
  request construction and playback target interpretation, while Android still
  owns transport execution, product diagnostics, Media3, player/session UI,
  cleartext/TLS policy, token vaults, and profile persistence.

## Non-Goals

- No Rust-owned Android socket/TLS/proxy behavior.
- No Media3, player session, Compose state, navigation, or product copy move
  into Rust.
- No Kotlin Multiplatform, Maven publishing, iOS binding, or external SDK
  release policy.
- No server route shape or OpenAPI v2 change.
- No browse/search tracer in this lane unless it is explicitly split after
  playback is verified.

## Architecture Direction

The work should deepen ADR 0032, not replace it.

### Android Build Topology

The Android app needs three distinct Rust artifacts:

1. generated Kotlin UniFFI bindings;
2. host native library for JVM unit tests;
3. Android ABI native libraries for APK packaging.

Only the first two are needed for JVM unit tests. Android ABI libraries should
be built by package/merge JNI tasks and should be configurable by ABI set so
local development can target an emulator ABI without paying all-target cost.

### Rust Core Reuse

The core should expose explicit FFI-safe request and response records. The
reqwest client may still return `reqwest::Url`, `HeaderMap`, and async results
to Rust callers, but it should obtain request specs and response policy from
`nako-client-core`.

### Public Wire Tolerance

Generated Kotlin already preserves unknown public string values. Rust must not
regress this before playback/browse DTOs can safely move behind UniFFI. Public
string enums that can grow must deserialize unknown strings into explicit
`Other(String)` values and serialize them back unchanged.

### Playback Core Tracer

The first playback tracer should be narrow:

- Rust core builds the playback decision request.
- Rust core performs generic status/API-version/public-error interpretation for
  the response.
- Android may still decode the generated Kotlin DTO and map product models.
- Rust core interprets the Android-provided playback decision summary into
  direct, remux, or HLS request targets.
- Android executes preflight requests, reads the public playback session header,
  and launches Media3.

## Guardrails

- Do not add Android profile persistence, token vault, cleartext/TLS,
  platform-networking, UI, or Media3 dependencies to Rust crates.
- Do not put runtime policy in `nako-client-uniffi`; it remains a binding
  adapter over `nako-client-core`.
- Do not hide unknown public wire strings as generic `Unknown` when the raw
  wire value can be preserved.
- Do not make JVM unit tests build all Android Rust ABIs.
