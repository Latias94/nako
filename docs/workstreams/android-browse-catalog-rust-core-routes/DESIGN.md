# Android Browse/Catalog Rust Core Routes

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

The Android client now has a hardened Rust core / UniFFI seam for connection and
playback request construction. Browse/catalog remains the largest Android route
family still constructing Public Client API routes through the generated Kotlin
SDK runtime. That is acceptable today, but it preserves a second portable route
policy surface that will grow with library, item, search, person, genre, and tag
flows.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/workstreams/android-uniffi-boundary-hardening/`
  - `docs/workstreams/android-rust-core-runtime-hardening/`

## Problem

Android browse/catalog route construction still depends on
`NakoPublicClientRequests` in Kotlin for high-value runtime paths:

- libraries and library sources;
- items, item images, and item detail;
- people and related items;
- genres/tags and facet item lists;
- search query/facet pagination.

This duplicates portable path/query/auth construction outside `nako-client-core`
and creates future drift risk as more native clients or offline/cache flows are
added.

## Target State

When this lane closes:

- `nako-client-core` owns browse/catalog request builders for the Android route
  family currently handled by `NakoBrowseClient`.
- `nako-client-uniffi` exposes explicit FFI-safe browse request builder records
  and functions only; it does not decode browse DTOs or execute transport.
- Android `NakoBrowseClient` asks a `BrowseCore` adapter for request descriptors
  and still owns transport execution, public error mapping, DTO-to-product
  mapping, diagnostics, UI state, and copy.
- Generated Kotlin SDK remains available for DTO decode and transition tests,
  but not for runtime browse route construction.
- Boundary guard, Rust package tests, Android browse JVM tests, and workstream
  closeout evidence pass.

## In Scope

- Core request builders for browse/catalog GET routes used by
  `NakoBrowseClient`.
- Thin UniFFI bindings for those request builders.
- Android browse adapter and `NakoBrowseClient` migration from Kotlin SDK route
  descriptors to Rust core request descriptors.
- Tests for stable path/query/auth/redaction behavior and current browse flows.
- Workstream docs and closeout.

## Out Of Scope

- Rust-owned Android networking.
- Browse/catalog DTO decode behind UniFFI.
- UI state, Compose screens, navigation, or Media3 changes.
- Server API shape changes.
- Removing generated Kotlin SDK DTOs.
- Offline/download/cache behavior.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| ADR 0032 still says Rust owns portable request construction while Android owns transport and UI. | High | `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md` | Reopen ADR before moving this lane. |
| Kotlin SDK should remain DTO/route-inventory transition tooling, not durable Android runtime-policy owner. | High | ADR 0032 and previous closeouts | If generated SDK must own routes, this lane should stop. |
| Browse/catalog DTO decoding can stay Kotlin-side for this lane. | High | `NakoBrowseClient` currently maps SDK DTOs to Android models | If Rust decode is required, split a separate DTO boundary workstream. |
| The current UniFFI boundary guard can be reused after adding browse builders. | High | `scripts/guard-uniffi-boundary.ps1` | If guard fails due expected dependencies, update the guard with a documented rationale. |

## Architecture Direction

Preserve the hardened seam:

```text
nako-client-core
  owns browse/catalog request path, query, bearer injection, safe preview

nako-client-uniffi
  owns FFI-safe browse request builder records/functions

Android BrowseCore adapter
  maps Android Page/Search inputs to UniFFI records and returns Android request descriptors

NakoBrowseClient
  executes Android transport, decodes generated Kotlin DTOs, maps product errors/diagnostics
```

The first-class interface should be request descriptors, not generic URL string
helpers. That keeps Rust core deep: callers ask for domain routes such as list
libraries, list genre items, or search items, and get a complete HTTP request
with safe preview.

## Closeout Condition

This lane can close when:

- TODO tasks BCR-010 through BCR-090 are complete,
- route construction is migrated for the high-value browse/catalog family,
- targeted Rust, UniFFI, Android browse, boundary guard, and docs gates pass,
- generated Kotlin SDK remains only for DTO decode/contract transition in the
  migrated `NakoBrowseClient` paths,
- and residual DTO migration or CI work is explicitly deferred.
