# Android Playback Residual Rust Core Routes

Status: Closed
Last updated: 2026-05-21

## Why This Lane Exists

Android connection, browse/catalog, user-playback, playback decision, streaming
targets, remux/HLS targets, and HLS segments now flow through the shared Rust
client core. `TaruPlaybackClient` still has a small but important runtime route
island that constructs routes through generated Kotlin SDK descriptors.

This lane closes that island and removes confirmed redundant compatibility code.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- Existing docs:
  - `docs/workstreams/android-browse-catalog-rust-core-routes/`
  - `docs/workstreams/android-user-playback-rust-core-routes/`
  - `docs/workstreams/android-uniffi-boundary-hardening/`

## Problem

Android `TaruPlaybackClient` still uses `TaruPublicClientRequests` for:

- `GET /sources/{source_id}/probe`;
- `GET /playback/sessions/{session_id}`;
- `POST /playback/sessions/{session_id}/cancel`.

After the user-playback migration, `PageRequest.toSdkPageQuery()` is also a
confirmed dead compatibility helper.

## Target State

When this lane closes:

- `taru-client-core` owns explicit request builders for source probe, playback
  session inspection, and playback session cancellation.
- `taru-client-uniffi` exposes thin FFI-safe builder records/functions for those
  routes.
- Android `TaruPlaybackClient` uses `PlaybackCore`/`RustPlaybackCore` for all
  runtime playback route construction and no longer imports generated SDK route
  descriptors.
- Generated Kotlin SDK remains available for playback DTO decode and contract
  tests, but not runtime playback route construction.
- Confirmed dead Kotlin compatibility helpers are deleted.
- Boundary guard, Rust package tests, Android playback JVM tests, route-owner
  scans, and closeout evidence pass.

## In Scope

- Core request builders for source probe, playback session inspect, and playback
  session cancel.
- Thin UniFFI bindings for those builders.
- Android `PlaybackCore`/`RustPlaybackCore` extension and `TaruPlaybackClient`
  migration from generated SDK route descriptors to Rust-built request
  descriptors.
- Deletion of `PageRequest.toSdkPageQuery()` if no callers remain.
- Tests and docs.

## Out Of Scope

- Rust-owned Android networking.
- Rust-side playback DTO decoding.
- Media3, player UI, session UI, PiP, cast, or navigation changes.
- Server API shape changes.
- Removing generated Kotlin SDK DTOs or request descriptors globally; artwork
  and previews are separate follow-ons unless proven dead.

## Architecture Direction

Preserve the hardened seam:

```text
taru-client-core
  owns playback route path, method, bearer injection, and safe preview

taru-client-uniffi
  owns FFI-safe playback residual request builder records/functions

RustPlaybackCore
  converts UniFFI CoreHttpRequest to Android PlaybackRequestDescriptor

TaruPlaybackClient
  validates product inputs, authenticates descriptors through Android policy,
  executes Android transport, decodes generated SDK DTOs, maps diagnostics
```

The durable API is route-specific request descriptors, not generic URL helpers.

## Closeout Condition

This lane can close when:

- tasks PRR-010 through PRR-090 are complete,
- `TaruPlaybackClient` has no generated SDK route descriptor usage,
- confirmed dead compatibility code is deleted,
- targeted Rust, UniFFI, Android playback, boundary guard, route-owner scan, and
  docs gates pass,
- and residual artwork/preview/generated SDK use is explicitly documented as a
  separate follow-on rather than playback runtime debt.
