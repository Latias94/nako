# Generated SDK Runtime Ownership

Status: Active
Last updated: 2026-05-21

## Why This Lane Exists

The Android Generated Public Client SDK lane moved route construction and DTO
mirrors into the OpenAPI-backed Kotlin/JVM SDK. The forward-compatibility lane
then made generated public string values tolerant. After those changes, Android
still owns the reusable Public Client API runtime seam:

- HTTP request execution;
- bearer-token header injection and redaction;
- public error-envelope parsing;
- API-version header checks;
- JSON decode failure mapping;
- transport failure mapping;
- token-safe request previews.

That ownership is safe for one Android shell, but it is now a durable boundary
question. If future JVM, desktop, Swift, Dart, or Rust/UniFFI clients need the
same protocol-level runtime behavior, keeping all policy in Android will
reintroduce drift. Moving too much into the SDK would also be wrong: product
diagnostics, Android security policy, token storage, Compose state, and Media3
playback must remain app-owned.

## Goals

- Inventory the current split between generated SDK surfaces, Android runtime
  execution, product diagnostics, and platform policy.
- Freeze an ownership matrix for Public Client API runtime responsibilities.
- Treat early shared Rust client core / UniFFI as a first-class candidate
  because the product direction prefers paying integration complexity now if it
  prevents future architecture debt.
- Decide whether the next move is no code, a Kotlin SDK runtime tracer, an
  Android-only cleanup, a shared Rust client-core tracer, or an ADR before
  implementation.
- If runtime movement is accepted, define the smallest tracer that proves the
  boundary without moving Android UI, navigation, token storage, or playback.

## Non-Goals

- No Maven publishing, binary compatibility, or external SDK release policy.
- No Kotlin Multiplatform topology change.
- No full Android migration to Rust/UniFFI before an ADR update and tracer
  contract. This lane may decide to pull shared Rust client-core work forward.
- No server route, OpenAPI shape, Admin API, or Public Client API v2 change.
- No Compose UI, Media3, media-session, or playback presentation ownership
  change.
- No token vault, profile persistence, cleartext policy, or Android permission
  redesign.

## Authoritative Docs

- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0031-android-client-sdk-before-mobile-rust-ffi.md`
- `docs/workstreams/android-generated-public-client-sdk/CLOSEOUT.md`
- `docs/workstreams/generated-sdk-forward-compat-tolerance/CLOSEOUT.md`
- `docs/workstreams/generated-sdk-runtime-ownership/DESIGN.md`
- `docs/workstreams/generated-sdk-runtime-ownership/TODO.md`
- `docs/workstreams/generated-sdk-runtime-ownership/EVIDENCE_AND_GATES.md`
- `docs/workstreams/generated-sdk-runtime-ownership/HANDOFF.md`
- `docs/workstreams/generated-sdk-runtime-ownership/WORKSTREAM.json`

## First Executable Slice

`SDKRT-010` is complete. It selected early shared Rust client core / UniFFI
target state with app-supplied Android transport as the durable direction.

Next executable slice: `SDKRT-020`. Create or supersede the ADR for pulling
Rust client core forward, then define the FFI-safe core API, crate topology,
Android build topology, and first connection-flow tracer. Implementation should
not start before `SDKRT-020` freezes that contract.
