# Android Fearless Client Refactor — Handoff

Status: Complete
Last updated: 2026-05-21

## Current State

AFCR-010 through AFCR-080 are implemented, locally validated, and closed. The
Android route clients now use a shared Public Client API executor, playback
launch route state is token-safe by construction, `BrowseSession` is a
composition shell over deeper browse state modules, and network cleartext
behavior is explicit rather than globally permissive. Search, relationship
indexes, and public-backed facets support server-backed load-more paging
without local filtering or invented totals. Product-facing copy, first Android
string-resource seams, key accessibility semantics, and smoke assertions for
the updated product language are in place for Settings, connection, browse,
detail, source picker, player, artwork fallback, and sanitized diagnostics
surfaces.

AFCR-070 keeps the current Kotlin package seams as the closeout shape for this
lane, and splits generated SDK, shared Rust/UniFFI client-core, Gradle module,
artwork, broader paging, downloads/offline, external player handoff, and
Android TV work into explicit follow-ons rather than hiding them inside this
refactor.

The architecture review found the main refactor targets:

- duplicated Public Client API protocol policy across Android clients;
- broad `BrowseSession` orchestration;
- raw bearer headers entering playback launch route state;
- minimal transport and global cleartext policy;
- first-page-only large-library behavior;
- developer-facing copy, missing localization seams, and incomplete
  accessibility semantics. This is complete for the first key Android surfaces
  in AFCR-060; remaining non-blocking UI polish should be split after closeout
  if new surfaces appear.

The full issue register is in `REFACTOR_REGISTER.md`.

## Completed In This Session

- Added `apps/android/app/src/main/java/dev/taru/android/connection/PublicClientApiExecutor.kt`.
- Migrated `TaruConnectionClient`, `TaruBrowseClient`, `TaruPlaybackClient`,
  and `TaruUserPlaybackClient` to the shared executor.
- Centralized API-version checks, HTTP error handling, public error-envelope
  parsing, JSON decode failure mapping, transport failure mapping, safe request
  previews, bearer redaction, and path/query helper policy.
- Kept route clients focused on route path/query/body construction, DTO decode
  target selection, and route-specific failure category mapping.
- Added playback regression tests proving session-preflight HTTP errors and
  unsupported API versions are handled through the shared executor and remain
  token-safe.
- Added `PlaybackRequestDescriptor` and migrated `PlaybackRequestTarget` plus
  `PlaybackLaunchRequest` away from storing route-level raw `TaruHttpRequest`
  values with bearer headers.
- Moved playback Authorization construction to explicit final-request builders
  and the non-saveable Media3 runtime boundary.
- Added regressions that descriptors reject Authorization/bearer header values,
  launch/request `toString` output remains redacted, direct-play preparation
  still requires a token, Player route save payloads stay transient, and
  playback/player diagnostics remain token-safe.
- Extracted browse state modules:
  - `BrowseSessionNavigation`
  - `BrowseRouteStatePolicy`
  - `BrowseRouteLoadingSession`
  - `BrowseItemDetailSession`
  - `BrowseSearchSession`
  - `BrowsePlaybackSession`
  - `BrowseSessionStore`
- Added `BrowseSessionRouteStateTest` plus shared browse test fixtures to
  verify route-family preparation, stale request invalidation, unsupported
  facet API-gap handling, and transient Player routes at the module seam.
- Added `ConnectionSecurityPolicy` and `CleartextHttpNotPermittedException`.
- Removed `android:usesCleartextTraffic="true"` from the main manifest and
  moved cleartext opt-in to the debug manifest for local development.
- Wired production `AndroidTaruAppEnvironmentFactory` and
  `JdkTaruHttpTransport` through explicit security policy.
- Added connection tests for production cleartext rejection, explicit local
  development allowance, and final transport guard behavior.
- Added debug/release `BuildConfig.TARU_ALLOW_CLEARTEXT_HTTP` wiring so debug
  builds explicitly allow local HTTP while release builds retain production
  cleartext denial.
- Added shared browse paging helpers for `PageInfo.nextPageRequestOrNull`,
  Search append, relationship index append, and facet append.
- Added `LoadMoreSearch`, `LoadMoreRelationshipIndex`, and `LoadMoreFacet`
  actions plus visible Load more footers for Search, relationship indexes, and
  public-backed facets.
- Threaded explicit `PageRequest` through `BrowseDataSource` and
  `ClientBrowseDataSource` for Search, relationship index, and facet routes.
- Added regressions for Search, relationship index, and facet page appending
  using only server `limit`, `offset`, and `returned` semantics.
- Added Android string resources plus `TaruStrings` indirection for stable
  common actions and labels: Back, Retry, Change server, Search, Load more,
  Copy diagnostics, server access key, and player session accessibility.
- Rewrote visible copy away from protocol/internal terms such as API gaps,
  route, access token, Media Source, User Playback State, current response, and
  source facts toward media-client language: server compatibility, sign-in key,
  titles, versions, watch progress, and server-backed lists.
- Added accessibility semantics for settings rows, active profile card,
  sign-out, status chips/pills, pressable media/library cards, relationship
  rows, source picker radio rows, library source rows, and player session
  status.
- Added regressions for source picker version fallback and accessibility copy,
  detail relationship copy, API-gap user language, player session a11y
  redaction, artwork fallback labels, and settings diagnostics labels.
- Updated `apps/android/scripts/Smoke-Emulator.ps1` so smoke criteria assert
  the AFCR-060 product language instead of stale developer-facing copy:
  server access key, sign-in required, From server, Related Titles, Check
  version, Version, ready playback labels, and Resume from server.

## Architecture Reassessment — AFCR-070

The post-refactor architecture review applied the deletion test to the major
modules touched by this lane.

### Keep for closeout: Kotlin package seams

The current Android foundation should close with package-level seams inside
`:app`, not with an immediate Gradle split.

- `PublicClientApiExecutor` is a deep module: deleting it would push
  API-version checks, public error-envelope parsing, JSON decode failure
  mapping, transport failure mapping, safe request previews, bearer redaction,
  and URL helper policy back into connection, browse, playback, and
  User Playback State clients.
- `BrowseSession` is now a composition shell over deeper modules for
  navigation, route state, route loading, search, Media Item Detail/source
  selection, and playback start policy. Deleting those modules would
  re-concentrate unrelated route-family rules in one broad session class.
- `PlaybackRequestDescriptor` is a deep token-safety module: deleting it would
  reintroduce raw request/header knowledge into saveable route state and
  diagnostics.

These seams now provide useful locality and leverage without creating a new
build graph. A Gradle module split would add interface overhead before there is
a second adapter, generated SDK, or measurable build/dependency pressure.

### Split later: generated Kotlin SDK

Generated Kotlin SDK work is justified, but not inside this lane.

Decision:

- Wait until the Public Client API OpenAPI v1 artifact from ADR-0025 is stable
  enough to become the contract authority.
- Generate route DTOs, path/query builders, error-envelope handling, and
  version-header handling from that contract.
- Replace Android's handwritten DTO mirrors and route clients through a
  separate target-state workstream, then delete the now-superseded Kotlin
  mirrors.

Reason:

- Android still has many `@Serializable` public DTO mirrors, so drift risk is
  real.
- However, AFCR-010 already centralized generic protocol policy; generating
  SDK code now would mix contract generation, app refactor, and UI hardening in
  one lane.

### Split later: shared Rust/UniFFI client core

ADR-0026 remains the long-term direction, but the first Android foundation
should not move to UniFFI yet.

Decision:

- Start a shared Rust/UniFFI client-core workstream only after a target-state
  document names the narrow portable interface.
- Good candidates for that interface are Public Client API calls, bearer-token
  handling, API-version checks, public error-envelope handling, DTO hydration,
  browse/search query state, playback decision interpretation, streaming
  request construction, and portable cache/download coordination metadata.
- Keep Media3 player instances, media sessions, subtitles/audio presentation,
  PiP, remote controls, background behavior, platform permissions, and
  Android-specific playback diagnostics in Kotlin.

Reason:

- There is still only one production native shell in this repo.
- FFI would currently have one adapter, so the seam would be hypothetical.
- Moving the player into Rust would contradict ADR-0026 and reduce platform
  playback depth.

### Defer: Gradle module split

Decision:

- Keep one `:app` module for this closeout.
- Consider a future split only when there is a concrete dependency graph such
  as `:client-public`, `:browse-ui`, `:player`, and `:connection`, or when
  generated SDK/FFI work creates a real second adapter.

Reason:

- The package seams now carry most of the locality benefit.
- A premature module split would make every test and preview pay build-graph
  cost without changing the public product behavior.

### Follow-on workstreams

Create separate workstreams when these become the next target state:

1. `android-generated-public-client-sdk` — replace handwritten Kotlin DTOs and
   route clients with OpenAPI-backed generated SDK code after ADR-0025 output
   stabilizes.
2. `shared-rust-client-core-uniffi` — design the narrow portable client-core
   interface and validation matrix across Android plus at least one other
   shell.
3. `android-gradle-module-split` — split only after the generated SDK/FFI
   decision or measurable build/dependency pressure.
4. `android-artwork-request-descriptor` — introduce token-safe Managed Artwork
   request descriptors when image caching, offline, or shared-core artwork
   routing begins.
5. `android-browse-paging-expansion` — extend the proven server-backed paging
   policy to Home and Library Detail when product inventory scale requires it.
6. `android-downloads-offline`, `android-external-player-handoff`, and
   `android-tv-shell` — product lanes with their own UX, permissions, and
   validation criteria.

## Active Goal

Complete the Android client fearless refactor by creating the cleanest
long-term seams and then fixing all registered issues in priority order.

## Validation Evidence

Fresh 2026-05-21 evidence is recorded in `EVIDENCE_AND_GATES.md`.

Validated:

- focused connection, browse, playback, User Playback State, connection UI,
  browse UI, settings, source picker, detail, player, and artwork JVM tests;
- full `:app:testDebugUnitTest`;
- debug `:app:assembleDebug`;
- full `apps/android/scripts/Validate-AndroidLocal.ps1`, including smoke states
  `empty-setup`, `profile-missing-token`, and `profile-with-media`;
- `git diff --check`.

Final reports:

- `apps/android/build/validation/20260521-032251/report.md`
- `apps/android/build/smoke-regression/20260521-032326/report.md`

## Next Task

None in this workstream. It is complete.

Rationale:

- M1 architecture boundaries, M2 production hardening, M3 product UI
  hardening, M4 architecture reassessment, and M5 closeout are locally
  complete.
- Remaining product work is split into follow-on workstreams.

## Guardrails

- Do not consume Admin/internal server routes from Android.
- Do not keep obsolete code for compatibility with old Android internals.
- Do not allow raw bearer tokens, local source locators, local paths, or FFmpeg
  command strings into visible UI, diagnostics, route saveable state, or smoke
  reports.
- Do not add downloads, external player handoff, Android TV, Cast, PiP, or
  account management inside this lane.
- Do not introduce a Rust-owned player abstraction.
- Do not split Gradle modules until a separate target-state workstream proves
  the dependency graph and migration path.

## Parallel Work

This lane is closed. Do not add new scope here; open a new workstream for the
follow-ons listed in AFCR-070.

## Validation Reminder

Final closeout gates are recorded in `EVIDENCE_AND_GATES.md`. If another agent
continues from this handoff, start from a new target-state workstream rather
than reopening AFCR.
