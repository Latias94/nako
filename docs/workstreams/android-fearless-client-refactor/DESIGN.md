# Android Fearless Client Refactor

Status: Complete
Last updated: 2026-05-21

## Why This Lane Exists

The Android client has a strong foundation: it connects to Nako through the
Public Client API, keeps tokens out of visible diagnostics, uses Media3 for
native playback, has unidirectional state modules, and ships with local smoke
evidence. The next phase should not preserve early-client scaffolding merely
because it works.

This lane records a fearless refactor of the Android client toward the cleanest
long-term seams before downloads, external player handoff, richer playback
controls, account management, offline cache, Android TV, or iOS shared-client
work make the current seams harder to change.

## Relevant Authority

- `CONTEXT.md`
- ADRs:
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- Existing docs:
  - `docs/workstreams/android-client-foundation/`
  - `docs/workstreams/android-api-contract-integration/`
  - `docs/workstreams/android-material-expressive-ui/`
  - `docs/workstreams/android-unidirectional-state-architecture/`
  - `docs/workstreams/android-player-session-architecture/`
  - `docs/workstreams/android-client-qa-harness/`
- Local implementation:
  - `apps/android/app/src/main/java/dev/nako/android/connection/`
  - `apps/android/app/src/main/java/dev/nako/android/browse/`
  - `apps/android/app/src/main/java/dev/nako/android/playback/`
  - `apps/android/app/src/main/java/dev/nako/android/userplayback/`
  - `apps/android/app/src/main/java/dev/nako/android/player/`
  - `apps/android/app/src/main/java/dev/nako/android/ui/`

## Problem

The Android client is past the tracer stage, but several early seams will
become expensive if feature work continues on top of them:

1. Public Client API clients repeat transport, API-version, error-envelope,
   JSON decode, query construction, and redaction logic.
2. `BrowseSession` is now a broad orchestration module for navigation, catalog
   loading, relationship browsing, Media Item Detail, source probe, Playback
   Source Selection, and playback start.
3. Playback launch objects can carry raw authorization headers through route
   state and player inputs, relying on disciplined `toString`/safe previews
   rather than a hard token boundary.
4. Android networking still uses a minimal `HttpURLConnection` adapter and
   globally permits cleartext traffic.
5. Paging is still a first-page pattern, not a reusable state model for large
   Media Libraries.
6. User-facing UI copy and semantics still contain developer-facing terms,
   hard-coded English strings, and incomplete accessibility/i18n seams.
7. The single Gradle `:app` module remains acceptable, but internal package
   seams should be deep enough before any future module split.

## Target State

When this lane closes:

- Android has one deep Public Client API adapter that owns transport execution,
  request authentication, API-version checks, public error-envelope parsing,
  JSON decode, safe diagnostics, and route-independent redaction.
- Route-specific clients become small modules that describe Public Client API
  routes and DTO mapping instead of re-implementing protocol policy.
- Browse state is divided into deep modules with clear interfaces:
  navigation, catalog/home/search, relationship browsing, Media Item Detail,
  Playback Source Selection, and playback launch.
- Playback launch route state is token-safe by construction. UI routes and
  saveable state never carry raw bearer tokens, raw source locators, FFmpeg
  commands, or local paths.
- Player runtime adapters inject authorization only at the final platform
  boundary and remain compatible with a future short-lived playback handoff
  API.
- Network security distinguishes debug/local self-hosted convenience from
  release safety. Cleartext use is explicit and user-visible.
- Paging, loading, retry, stale-response handling, and empty/error states are
  reusable across Home, Search, Library Detail, relationship indexes, and facet
  results.
- UI strings, settings copy, source picker labels, error copy, and accessibility
  semantics are product-grade and prepared for localization.
- The smoke harness continues to prove setup, Home, detail, relationship
  browsing, source picker, playback, exit effects, and token-safe evidence.

## In Scope

- Android production code under `apps/android/app/src/main/java/dev/nako/android/`.
- Android JVM tests under `apps/android/app/src/test/java/dev/nako/android/`.
- Android smoke scripts only when new gates or evidence are required.
- Workstream docs under this directory.
- Deleting or replacing obsolete Android code when the new seam supersedes it.
- Updating related Android API integration docs if route ownership changes.

## Out Of Scope

- Changing server Public Client API contracts unless the refactor proves a
  concrete contract gap.
- Consuming Admin API, Addon, Automation, storage diagnostics, or metadata
  maintenance routes from Android.
- Implementing downloads/offline playback, Android TV, external player handoff,
  Cast, PiP, full track picker, or account management in this lane.
- Introducing a Rust-owned player abstraction.
- Copying code, comments, layouts, assets, tests, or schemas from Jellyfin,
  Plex, Findroid, or other reference projects.
- Splitting Gradle modules before package seams are clean enough to justify the
  build and dependency overhead.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The Android client should keep direct Kotlin HTTP for the first refactor slice. | Medium | Existing Android foundation deferred UniFFI until duplication became costly. | If route/protocol duplication remains high after adapter deepening, split a generated SDK or UniFFI lane. |
| Most protocol drift can be fixed without changing server routes. | High | Current clients call public v1 routes successfully. | If a route shape is missing, split a Public Client API workstream before inventing client-only semantics. |
| Token-safe playback launch can be introduced without changing Media3 playback behavior. | High | Player already receives launch metadata plus token vault access through runtime adapters. | If Media3 requires raw headers earlier, keep raw requests inside a non-saveable platform adapter only. |
| Paging needs reusable state before large-library features. | High | Current screens use fixed first pages. | If paging is deferred, new browse features will duplicate loading and retry policy. |
| UI copy and a11y can be improved without a visual redesign. | High | V2 Material baseline is already accepted. | If visual evidence shows structural problems, split a UI polish lane rather than hiding it in architecture work. |

## Architecture Direction

This lane should deepen modules, not merely split files.

### Public Client API adapter

Create a deep adapter module with a small interface:

- input: method, public path/query, optional typed body, auth policy, expected
  response type;
- output: typed success or safe diagnostics with stable failure category.

This module owns API-version policy, bearer redaction, public error-envelope
parsing, sanitized transport failures, JSON decode errors, request previews,
and final authenticated request construction. Route clients own only route
shape and DTO conversion.

### Browse and playback state

`BrowseSession` should become a composition root for smaller state modules
instead of the owner of every async route policy. Each deep module should have
one reason to change and an independently testable interface.

### Token-safe playback launch

Playback route state should carry a token-safe launch descriptor. The player
runtime adapter should resolve the active token reference and inject
authorization immediately before Media3 request construction. External player
handoff must remain deferred until a short-lived public handoff contract
exists.

### Network safety

Keep self-hosted and local-development ergonomics, but stop treating global
cleartext as the final app policy. Debug and release behavior may differ.

### Product UI hardening

Preserve the current Material 3 V2 visual baseline while improving copy,
semantics, localization seams, and large-library behavior.

## Closeout Condition

This lane can close when:

- P0 and P1 architecture tasks are implemented or explicitly split with
  accepted rationale;
- no Android UI route or saveable state carries raw bearer tokens;
- duplicated Public Client API protocol policy is removed from route clients;
- Browse state no longer requires one broad module to understand every product
  route;
- paging and product copy/a11y foundations are in place;
- fresh Android unit, build, smoke, and diff gates are recorded;
- remaining product features are either completed, deferred, or split into
  follow-on workstreams.

## Closeout Summary

Closed on 2026-05-21.

The lane reached the target state for the Android foundation:

- Public Client API execution policy is centralized in a deep Android adapter.
- Playback launch route state is token-safe by construction.
- Browse state is split into deeper modules with focused test surfaces.
- Android network cleartext policy distinguishes debug/local development from
  production release behavior.
- Search, relationship indexes, and public-backed facets have reusable
  server-backed paging.
- Product copy, first localization seams, and key accessibility semantics are
  in place.
- Smoke assertions now track the product language introduced by AFCR-060.

Generated Kotlin SDK, shared Rust/UniFFI client core, Gradle module split,
artwork descriptors, broader Home/Library Detail paging, downloads/offline,
external player handoff, and Android TV are not hidden residual work in this
lane; they are explicit follow-on targets.
