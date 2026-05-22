# Android Artwork And Preview Rust Core Routes — Design

Status: Closed
Opened: 2026-05-22

## Problem

After browse, user-playback, and playback route migrations, two Android-side
uses still make the generated Kotlin SDK look like a runtime route owner:

1. `PublicArtworkSource` validates and builds selected artwork image requests
   through `NakoPublicClientRequests.image(...)` and `NakoRequestDescriptor`.
2. `NakoBrowseShellPreview` matches fake transport requests through generated
   SDK request descriptors.

ADR 0032 says the durable runtime owner is the shared Rust client core behind an
Android-supplied transport. The generated Kotlin SDK remains useful for DTOs,
wire constants, and contract tests, but it should not be the application route
construction policy in runtime code or previews that mirror runtime ownership.

## Target State

- `nako-client-core` owns selected artwork image request construction.
- `nako-client-uniffi` exposes a thin FFI-safe artwork request builder over the
  Rust core.
- Android artwork code asks an `ArtworkCore` seam for image requests and keeps
  Android-owned responsibilities: active profile/token lookup, UI image loading,
  DTO selection, and local product behavior.
- Android validates that a `PublicImageRefDto` URL points exactly to the Rust
  core route for its image id before loading it. This preserves the previous
  safety property without using generated SDK descriptors.
- Compose preview fake transport uses fixture-owned URL matching helpers instead
  of generated SDK route descriptors.
- No Android `src/main` runtime code imports `NakoPublicClientRequests` or
  `NakoRequestDescriptor` after this lane.

## Scope

In scope:

- Add Rust core selected artwork image GET builder.
- Expose that builder through UniFFI.
- Add Android `ArtworkCore` / `RustArtworkCore` adapter.
- Refactor `PublicArtworkSource` and `preferredPublicArtwork` validation to use
  the Android artwork core seam.
- Remove now-dead `PublicApiRequestDescriptors.kt` if no callers remain.
- Replace `NakoBrowseShellPreview` generated SDK path matching with stable
  preview-local route helpers.
- Update Android docs and workstream evidence.

## Non-goals

- Do not move image byte fetching into Rust.
- Do not move Coil/Compose image loading or Android HTTP transport into Rust.
- Do not decode image DTOs in Rust.
- Do not change server API shape, generated SDK generation, or OpenAPI contract.
- Do not remove generated SDK DTO usage or contract-test usage.
- Do not add image variant UI unless an existing caller already supplies width
  or height.

## Architecture Direction

### Rust core

Add an explicit artwork request input, for example:

```rust
CoreArtworkImageRequestInput {
    base_url: String,
    access_token: String,
    image_id: String,
    width: Option<u32>,
    height: Option<u32>,
}
```

The builder returns a `CoreHttpRequest` for `GET /images/{image_id}` with
optional `width` and `height` query parameters, bearer auth, encoded image ids,
and redacted safe previews.

### Android

`PublicArtworkSource` should not inspect generated descriptors. Its safety check
should be:

1. Reject blank token or blank image id.
2. Ask `ArtworkCore` to build the canonical request for the image id.
3. Compare the image DTO `url` against the canonical core-built route's path and
   query, after stripping the active profile base URL from the core URL.
4. If the DTO URL matches, return the core-built `NakoHttpRequest` and
   `SafeRequestPreview`; otherwise reject the image as unsafe/unusable.

This keeps provider-controlled or stale URLs from causing cross-origin, admin,
query-token, `..`, or mismatched-id requests. It also avoids duplicating percent
encoding in Kotlin.

### Preview fixtures

Preview fake transports are not runtime policy, but they teach future readers.
They should use local helper constants/functions such as:

- `previewRouteListLibraries()`
- `previewRouteContinueWatching(limit = 12)`
- `previewRouteItem(itemId)`

These helpers must not import generated SDK request descriptors.

## Assumptions

- Public image DTO `url` remains a relative public path such as
  `/images/{image_id}`.
- Android selected artwork loading currently uses no width/height variant; the
  Rust builder supports variants for parity, but the Android runtime may pass
  `null`.
- `NakoConnectionClientTest` may continue using generated SDK constants and
  route descriptors as a contract/inventory assertion, because it is a test, not
  app runtime policy.

## Risks And Guards

- Risk: comparing full URLs instead of route paths could reject profiles with a
  base path. Guard: derive canonical path/query by removing the active profile
  base URL prefix from the core-built URL.
- Risk: silently accepting absolute or admin URLs. Guard: route comparison must
  require exact canonical path/query match.
- Risk: duplicated preview path formatting drifts. Guard: preview helpers are
  intentionally local-only and covered by an Android compile gate; production
  runtime stays Rust-owned.
