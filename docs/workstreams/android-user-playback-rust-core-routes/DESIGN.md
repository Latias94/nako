# Android User Playback Rust Core Routes

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

Android browse/catalog route construction now flows through `nako-client-core`
and a thin UniFFI adapter. User Playback State is the next Android route family
still constructing runtime Public Client API routes through generated Kotlin SDK
request descriptors.

Leaving this family in Kotlin preserves a second portable route-policy surface
for resume, Continue Watching, playback-progress update, and watched-state
update. That conflicts with ADR 0032's direction that shared Rust client core
own portable request construction while Android owns platform transport and
product behavior.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
  - `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- Existing docs:
  - `docs/workstreams/android-browse-catalog-rust-core-routes/`
  - `docs/workstreams/android-uniffi-boundary-hardening/`

## Problem

Android `NakoUserPlaybackClient` still uses `NakoPublicClientRequests` for:

- `GET /users/me/playback-state/items/{item_id}`;
- `GET /users/me/playback-state/continue-watching`;
- `PUT /users/me/playback-state/items/{item_id}/progress`;
- `PUT /users/me/playback-state/items/{item_id}/watched`.

That means the high-value resume/watch-state route family has not yet adopted
the shared Rust request construction seam.

## Target State

When this lane closes:

- `nako-client-core` owns explicit User Playback State request builders for the
  Android route family in `NakoUserPlaybackClient`.
- `nako-client-uniffi` exposes FFI-safe user-playback request builder records
  and functions only; it does not decode DTOs or execute transport.
- Android `NakoUserPlaybackClient` asks a `UserPlaybackCore` adapter for request
  descriptors and still owns Android transport, generated SDK DTO decode,
  request body serialization, diagnostics, and product error categories.
- Generated Kotlin SDK remains available for DTO/body contract transition, but
  not for runtime User Playback State route construction.
- Boundary guard, Rust package tests, Android user-playback JVM tests, and
  closeout evidence pass.

## In Scope

- Core request builders for the four User Playback State routes used by Android.
- Thin UniFFI bindings for those builders.
- Android `UserPlaybackCore` adapter and `NakoUserPlaybackClient` route
  migration from generated SDK route descriptors to Rust-built requests.
- Targeted tests for stable path/query/auth/redaction/method behavior and
  current user-playback flows.
- Workstream docs and closeout.

## Out Of Scope

- Rust-owned Android networking.
- Rust-side User Playback State DTO decoding.
- Rust-side JSON body construction for update-progress or set-watched requests.
- UI, Compose, navigation, Media3, or server API changes.
- Removing generated Kotlin SDK DTO/body models.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| ADR 0032 still says Rust owns portable request construction while Android owns transport and UI. | High | `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md` | Reopen ADR before moving this lane. |
| ADR 0028 route set remains the authoritative first User Playback State public contract. | High | `docs/adr/0028-user-playback-state-principal-and-public-contract.md` | Update route builders and tests to the accepted contract. |
| Android can continue serializing request bodies through generated SDK DTOs. | High | Current `NakoUserPlaybackClient` body mapping is product-local and stable. | Split a body/DTO Rust migration lane instead of expanding this one. |
| The current UniFFI boundary guard should remain unchanged after adding request builders. | High | Browse route migration proved the pattern. | If it fails, document and justify any guard change. |

## Architecture Direction

Preserve the hardened seam:

```text
nako-client-core
  owns User Playback State request path, query, method, bearer injection,
  content-type header for JSON write requests, and safe preview

nako-client-uniffi
  owns FFI-safe user-playback request builder records/functions

Android UserPlaybackCore adapter
  maps Android profile/page/item inputs and optional JSON body to UniFFI records
  and returns Android request descriptors

NakoUserPlaybackClient
  validates product inputs, serializes generated SDK request bodies, executes
  Android transport, decodes generated Kotlin DTOs, maps product errors/diagnostics
```

The seam is request descriptors, not generic URL helpers. Core builders should
be route-specific: get state, continue watching, update progress, and set
watched state.

## Closeout Condition

This lane can close when:

- TODO tasks UPC-010 through UPC-090 are complete,
- route construction is migrated for Android User Playback State routes,
- targeted Rust, UniFFI, Android user-playback, boundary guard, and docs gates
  pass,
- generated Kotlin SDK remains only for DTO/body contract transition in the
  migrated paths,
- and residual DTO/body/CI work is explicitly deferred.
