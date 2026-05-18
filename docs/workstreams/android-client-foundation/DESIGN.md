# Android Client Foundation

Status: Proposed
Last updated: 2026-05-17

## Why This Lane Exists

ADR 0026 establishes Taru's flagship playback-client direction: native
platform shells with a shared Rust client core. Android is the first
implementation target so Taru can validate real playback behavior, server
connection flows, Public Client API ergonomics, and shared client-core
boundaries before splitting attention across iOS, TV, desktop, or web clients.

This lane records the Android-first implementation plan without changing the
long-term client architecture. iOS remains a peer target; Android-specific
choices must not leak back into the Public Client API or shared Rust model.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/client-sdk-contract/`
- `docs/workstreams/rust-client-sdk/`
- `docs/workstreams/client-cli/`
- `docs/workstreams/android-client-foundation/UX_CONTEXT.md`

## Clarified Scope

The first client scope is Android phone and tablet. TV, desktop, web, and iOS
implementation are deferred, though the architecture must remain compatible
with them.

The product slice is playback-first with a minimal media-library browsing loop.
The first useful client should let a user connect to Taru, browse a library,
open a media item, select a playable source through Taru's playback decision,
and play through Android's native media stack.

The first user-facing page set is defined in `UX_CONTEXT.md`. It intentionally
borrows Findroid's broad page families and playback checklist as reference
material while preserving Taru's own domain model, design language, and public
API boundary.

The Home screen is a playback launchpad. Libraries and Search are the stable
first-version anchors; Continue Watching, Latest, and Next Up are enhancements
when the Public Client API exposes the needed state.

Resume UX is part of the intended playback loop, but authoritative User
Playback State belongs to the Public Client API. Android may keep only
active-server-scoped device-local transient position as a temporary convenience
when server API support is incomplete.

The Android client may store multiple server profiles, but one server is active
at a time. Home, Search, cache, playback, and future Downloads are scoped to
the active server. A server profile is client-side connection state, not a Taru
User.

First-version auth is server URL plus access token, matching the current
inbound bearer-token boundary. Username/password, OAuth/OIDC, QR login,
device-code login, user picker, and permission management are deferred.

First-version errors must be actionable and client-safe. User-facing copy
should explain the state and next action, while raw protocol/server codes stay
limited to sanitized diagnostics.

Media Item Detail is a playback decision surface. It should prioritize
Play/Resume, source/version choice, basic metadata, hierarchy navigation, and
explainable playback errors over metadata management or server diagnostics.

Search first provides global title-oriented discovery and safe result
navigation. Advanced facets, filters, and sort controls must follow explicit
Public Client API support instead of inventing client-only filtering semantics.

Phone and tablet share one touch-first navigation model. Tablet layouts may use
extra width for grids and detail context, but they must not fork the product
model or become Android TV.

The visual direction is Findroid-inspired and expressive-leaning: immersive,
artwork-led, dark-first, playback-confident, and source-clear. It uses
Material 3 as the interaction foundation and defines principles, component
semantics, and a small token set rather than a full brand manual.

Source / Version Picker is part of the first playback loop whenever multiple
playable Media Sources or Source Variants exist. It should show client-safe
source facts and playback-mode consequences, not server-local paths or raw
diagnostics.

The first Player is a reliability surface. It must make playback state,
buffering, seeking, exit behavior, cancellation, and errors understandable
before Taru adds advanced gestures, PiP, trickplay, skip controls, Cast, or
alternate player backends.

ACF-060 audits the current Public Client API boundary for playback state. The
public API already exposes playback session inspection and cancellation, but
does not yet expose progress reporting, resume lookup, or authoritative
**User Playback State**. Android can keep active-server-scoped device-local
transient position for route-local resume convenience, but this state must stay
separate from cross-device Continue Watching.

External player handoff is a deferred compatibility feature, not a
first-version playback path. It must not expose long-lived bearer tokens and
may require a short-lived public playback handoff API before implementation.

Offline playback and Downloads are deferred from the first playback loop but
treated as a second-phase core client capability. They require a separate
storage, permission, source-selection, and lifecycle design before
implementation.

Settings first serves client identity, connection, theme, and basic playback
preferences. Server administration, provider configuration, NFO policy,
addon/webhook/automation configuration, and advanced transcode controls belong
outside the first mobile client.

Android-first is an implementation order, not an Android-only product
strategy. Shared protocol, DTO, cache, and playback-decision code must remain
portable enough for a later iOS shell.

## Target State

- Android app code lives under `apps/android/`.
- The Android UI uses Kotlin and Compose.
- Playback uses AndroidX Media3 ExoPlayer.
- The app consumes Taru through the Public Client API and existing public SDK
  boundaries.
- Shared Rust client-core code, if introduced, lives in a permissive client
  crate and is exposed through a narrow FFI boundary such as UniFFI.
- Shared Rust code may own server URL handling, token handling, API-version
  checks, public error-envelope handling, DTO hydration, browse/search state,
  playback decision interpretation, and streaming request construction.
- Kotlin/native Android code owns player instances, player lifecycle, Media3
  integration, track selection UI, subtitle rendering, media sessions, audio
  focus, PiP, notifications, permissions, and Android-specific diagnostics.

## In Scope

- Android project scaffold under `apps/android`.
- A minimal Android app shell for connecting to a Taru server.
- A basic dark-first Material 3 theme aligned with the visual baseline in
  `UX_CONTEXT.md`.
- Public Client API connection and bearer-token handling.
- Server URL plus access-token setup flow with secure storage, connection
  preflight, and token-redaction behavior.
- Multiple server profiles with one active server at a time.
- Active-server scoped Home, Search, cache, playback, and future Downloads.
- First page set from `UX_CONTEXT.md`: setup flow, main shell, content detail
  flow, player, track/subtitle sheet, and playback error sheet.
- Library list, media item list, media item detail, and search/browse state
  needed for a playback loop.
- Home sections anchored by Libraries and Search, with resume/latest sections
  only when backed by Public Client API data.
- Resume UI placeholders that use authoritative Public Client API state when
  available and do not promote device-local transient state to cross-device
  facts.
- Media Item Detail as a playback decision surface.
- Global keyword search over Media Items with basic result grouping and detail
  navigation.
- Shared phone/tablet navigation model with responsive grid and detail layouts.
- Playback decision query and request construction for direct, remux, and HLS
  routes.
- Source / Version Picker for multiple playable sources or variants.
- Media3 ExoPlayer integration for the first playable route.
- Player controls for play/pause, seek, loading/buffering, elapsed/remaining
  time, full-screen/orientation behavior, exit behavior, playback errors,
  track/subtitle sheet entry, and supported session cancellation on exit.
- Settings for server identity, re-authentication, connection diagnostics,
  theme, basic playback preference, basic subtitle preference,
  mobile-network warning or restriction, About, license, and version
  information.
- Basic error UX for server unreachable, unauthorized, unsupported API
  version, not found, and playback failure.
- Client-safe empty/error states for setup, auth, browse, search, detail, and
  playback, each with an actionable recovery path.
- Dependency gates proving Android/shared client code does not depend on AGPL
  server/internal crates.

## Out Of Scope

- iOS implementation.
- Android TV, tvOS, desktop, or web UI.
- Tablet-only product behavior or TV-style focus navigation in the phone/tablet
  client.
- Cross-server Home, Search, Continue Watching, cache, playback, or Downloads
  aggregation.
- Client-only authoritative watch-state models, cross-device Continue Watching
  from local transient state, offline progress sync without reconciliation, or
  reliable external-player progress without a later handoff design.
- Server-admin workflows.
- Metadata editing, NFO import/export control, provider diagnostics, addon
  management, webhook management, automation management, storage diagnostics,
  and job administration.
- Advanced transcode controls, hardware acceleration selection, provider
  settings, NFO settings, home-section customization, detailed gesture tuning,
  alternate-player parameter editing, and broad experimental flag lists.
- Full offline download manager, durable local media cache, or background sync.
- Downloads tab and download actions until offline source-selection, storage,
  lifecycle, and permission behavior are designed.
- Favorites, person detail, collections, and advanced settings as complete
  feature families.
- Recommendation algorithms, personalized ranking, review/comment systems, and
  metadata-management detail surfaces.
- Advanced search facets, multi-condition filters, sort-control UI, saved
  searches, search history, provider-specific search, and cross-server search.
- Cast, AirPlay, DLNA, or remote playback device integration.
- Advanced player gestures, chapters, trickplay, skip intro/outro, PiP,
  background playback, playback speed, mpv fallback, external player handoff,
  and complex lock-screen controls.
- OAuth/OIDC, multi-user session management, and RBAC UI beyond the existing
  bearer-token boundary.
- Username/password login, QR login, device-code login, user picker, and
  client-side permission management.
- A Rust-owned cross-platform player abstraction.

## Architecture Direction

Keep the Android client thin at the platform boundary and strict at the Taru
boundary.

Android should treat Taru as a server reached through Public Client API routes,
not as a local Rust library. The app may reuse Rust client code through FFI
once that reduces duplication, but the first scaffold should preserve the same
contract discipline as `taru-client-cli`: clients consume public routes and
public DTOs only.

Do not expose server-local paths, raw provider state, storage internals, or
admin diagnostics to Android because they are convenient for UI development.
If the Android client needs new data, add a Public Client API affordance first
and keep it protocol-owned.

Playback should remain native. Rust can construct the HLS/direct/remux request
and interpret a playback decision; Media3 owns playback execution and
Android-specific lifecycle behavior.

## ACF-020 Client Connection Decision

ACF-020 starts with direct Kotlin HTTP for the Android setup/auth slice.

Rationale:

- It validates Android UX, secure token-reference handling, active-server
  scoping, and Public Client API error semantics without adding FFI packaging
  risk to the first connection screen.
- The slice only needs `GET /health` and a lightweight authenticated Public
  Client API probe. Duplicated protocol surface is intentionally tiny.
- The implementation keeps protocol facts in a small Android connection layer:
  expected API version `v1`, `x-taru-api-version`, `ErrorResponse`, and safe
  diagnostics. It must not import `taru-api`, `taru-server`, `taru-core`,
  `taru-streaming`, or `taru-transcode`.
- UniFFI remains the likely path once browse/search/playback request
  construction grows enough that duplicating SDK logic becomes expensive.

The auth check uses unauthenticated `GET /health` for reachability and API
version preflight, then an authenticated lightweight Public Client API probe
against `/libraries?limit=1&offset=0` because `GET /health` intentionally
bypasses auth. The probe does not expose a browse UI or parse library data in
ACF-020; it only verifies token acceptance and public error handling.

## Open Questions

- When should Android move from direct Kotlin HTTP to a shared Rust/UniFFI
  client core: during browse/search request construction, playback decision
  construction, or later cache/download work?
- What local persistence layer should Android use for client-only state:
  DataStore, Room, or a small Rust-owned cache exposed through FFI?
- Which playback route should be the first smoke-test target: HLS playlist,
  direct stream, or remux stream?
- What Public Client API contract should own progress reporting, resume lookup,
  watched state, and other authoritative **User Playback State**?
- Should playback stream responses expose a public session id header or
  structured launch envelope so native clients can cancel remux/HLS sessions
  on exit without parsing playlists?
- After ACF-010, `apps/android` starts as one `:app` module. When should the
  app split into `core/*` and `feature/*` Gradle modules?

## Closeout Condition

This lane can close when Android has a validated foundation for connecting to
Taru, browsing a minimal library surface, requesting playback decisions, and
playing at least one public playback route through Media3 without depending on
server/internal crates.
