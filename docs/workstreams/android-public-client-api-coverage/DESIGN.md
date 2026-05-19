# Android Public Client API Coverage

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

The Android client foundation is complete enough to connect, browse, inspect
detail, request playback decisions, and play public stream routes. After the
latest `main` merge, the server Public Client API has grown beyond the Android
consumer surface, especially around selected artwork image serving and richer
catalog read routes.

This lane keeps Android API consumption intentional. It records what Android
already consumes, what should be productized next, what belongs to server or
shared SDK work, and what should remain out of Android scope.

## Relevant Authority

- ADRs:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- Existing docs:
  - `docs/api/HTTP_API.md`
  - `docs/workstreams/android-client-foundation/CLIENT_INTERFACE_DESIGN.md`
  - `docs/workstreams/android-client-foundation/UX_CONTEXT.md`
  - `docs/workstreams/android-client-foundation/HANDOFF.md`
  - `docs/workstreams/public-client-api/`
- Local implementation:
  - `apps/android/app/src/main/java/dev/taru/android/connection/`
  - `apps/android/app/src/main/java/dev/taru/android/browse/`
  - `apps/android/app/src/main/java/dev/taru/android/playback/`
  - `apps/android/app/src/main/java/dev/taru/android/ui/`

## Problem

Android currently mirrors selected Public Client API routes directly in Kotlin.
That was correct for the first playback-first foundation, but the route surface
is now large enough that missing routes can become invisible product debt.

The largest current gaps are:

- selected artwork discovery and byte serving are public, but Android still uses
  generated color placeholders;
- item credits/images and library detail/source list have public routes, but
  Android mostly relies on `GET /items/{item_id}` aggregate detail;
- people, tag, and genre list/detail routes are public, but Android only uses
  their linked-items routes;
- source probe is public, but Android only reads probe facts returned by
  playback decision;
- server-authoritative User Playback State is not public yet, so Android must
  keep local resume clearly device-local.

## Target State

When this lane closes:

- Android has a maintained coverage matrix for every Public Client API v1 route.
- The next Android implementation order is explicit and product-oriented.
- High-priority client-facing gaps are either implemented or split into
  dedicated follow-on lanes.
- Android still avoids admin/internal routes, server-only diagnostics, metadata
  editing, addon management, automation, and storage diagnostics.
- Token redaction, active-server scoping, API-version checks, and public error
  handling remain consistent across all newly consumed routes.

## In Scope

- Audit and maintain the Android route coverage matrix.
- Implement public selected artwork image consumption for browse/detail/player
  surfaces.
- Add first-class client methods only for Public Client API routes that support
  the Android playback-first user experience.
- Add focused unit tests and smoke evidence for newly consumed Android routes.
- Split server contract work when Android needs a route that is not public yet.

## Out Of Scope

- Consuming Admin API routes from the Android phone/tablet app.
- Metadata writeback, NFO import/export, addon management, webhook management,
  automation provider management, or storage diagnostics.
- Publishing an Android SDK or replacing the Kotlin clients with UniFFI.
- Cross-device Continue Watching until a public User Playback State contract
  exists.
- Downloads/offline playback.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Android should keep direct Kotlin HTTP for this lane. | High | Android foundation deferred UniFFI until duplication is large enough. | If duplication grows, split a shared SDK/FFI lane before adding more client logic. |
| `GET /images/{image_id}` and `HEAD /images/{image_id}` are Public Client API routes. | High | `docs/api/HTTP_API.md` lists them in the public v1 set and OpenAPI/SDK include them. | If the route becomes non-public, Android must keep generated placeholders. |
| Item detail already includes enough sources, credits, relations, and images for the first detail screen. | Medium | `ItemDetailResponse` includes sources, credits, relations, and images. | If route payload stays too large or stale, Android should add narrower `/credits` or `/images` calls. |
| Authoritative playback progress is still missing from Public Client API. | High | Android foundation ACF-060 and current HTTP API only expose session inspect/cancel, not watch-state progress reporting. | If a route exists later, split a User Playback State lane. |

## Architecture Direction

Keep Android route clients small and vertical:

- `connection` owns setup, health, auth probe, API-version handling, and token
  redaction.
- `browse` owns JSON catalog routes and selected artwork discovery metadata.
- a new image request layer should build authenticated image URLs/headers
  without leaking bearer tokens into diagnostics.
- `playback` owns playback decisions, stream request construction, session
  inspection, and session cancellation.
- UI code consumes presentation models instead of hard-coding route facts.

Do not use server/internal Rust crates from Android. Public route facts should
continue to come from `docs/api/HTTP_API.md`, `taru-client-protocol`, generated
OpenAPI/SDK evidence, and Android's own thin clients.

## Closeout Condition

This lane can close when:

- `API_COVERAGE_MATRIX.md` has been checked against the current public route
  inventory;
- the selected artwork/image route decision is implemented or split;
- remaining route gaps are classified as product backlog, server contract gaps,
  or explicit non-goals;
- Android unit/smoke gates pass for any code added in this lane;
- docs and handoff point to the next executable task.

## Closeout

Closed on 2026-05-19.

Android now has intentional coverage or explicit deferral for every current
Public Client API v1 route. Selected artwork, Library Detail/source inventory,
and direct source probe coverage are implemented. Server-authoritative **User
Playback State** is split to
`docs/workstreams/user-playback-state-contract/`.

Final gates:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `git diff --check`
- public route inventory check against `docs/api/HTTP_API.md`
