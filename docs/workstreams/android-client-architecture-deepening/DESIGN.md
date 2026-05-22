# Android Client Architecture Deepening

Status: Draft
Last updated: 2026-05-22

## Why This Lane Exists

The Android client foundation is usable and several earlier refactor lanes have
closed, but the next product wave will make remaining shallow seams expensive:
Public Client API runtime policy is spread across route-family clients, browse
route side effects are coordinated indirectly through `BrowseShellHost`, player
lifecycle is still route-scoped, UI design-system ownership is blurred by large
screen/component files, Home loads as one coarse read model, and local client
state/build hygiene needs a clearer target before downloads, offline playback,
PiP/Cast, Android TV, and richer Managed Artwork grow on top.

This lane records a fearless architecture-deepening pass. It may delete
obsolete adapters, wrappers, duplicated components, and transition code when a
deeper module or interface makes them unnecessary. The goal is the best long-
term architecture, not the smallest patch.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/adr/0032-shared-rust-client-core-app-supplied-transport.md`
- `docs/workstreams/android-client-foundation/`
- `docs/workstreams/android-fearless-client-refactor/`
- `docs/workstreams/generated-sdk-runtime-ownership/`
- `docs/workstreams/android-rust-core-runtime-hardening/`
- `docs/workstreams/android-browse-catalog-rust-core-routes/`
- `docs/workstreams/android-user-playback-rust-core-routes/`
- `docs/workstreams/android-artwork-preview-rust-core-routes/`
- `docs/workstreams/android-playback-residual-rust-core-routes/`
- `docs/workstreams/android-browse-shell-session-host/`
- `docs/workstreams/android-player-session-architecture/`
- `docs/workstreams/android-player-route-host/`
- `docs/workstreams/android-material-expressive-ui/`
- Local architecture review artifact generated during planning:
  `C:\Users\Frankorz\AppData\Local\Temp\nako-android-architecture-review-20260522-091745.html`

## Problem

The current app has many good modules, but some modules are still too shallow or
wide for the next wave:

1. Route-family clients repeat the same execution pattern around token lookup,
   Rust-core request construction, `PublicClientApiExecutor`, generated SDK
   decode, product model mapping, safe request previews, and route-specific
   failure categories.
2. Browse route loading and persistence side effects are triggered indirectly by
   host state collection. This works today, but the effect ownership is not
   explicit enough for deeper route families, partial Home loading, and future
   lifecycle-aware restoration.
3. Player lifecycle remains tied to a route host and Compose disposal. It is
   not yet a durable Android-owned playback runtime ready for audio focus,
   MediaSession, PiP, Cast handoff, lifecycle resume, or robust exit effects.
4. UI design-system components and media-specific browse components overlap,
   and large screen files hide display-model/presentation logic inside broad
   composables.
5. Home loading is a coarse all-or-nothing read model. Managed Artwork
   enrichment, Continue Watching, Media Library sections, and Media Item grids
   cannot degrade independently.
6. Local persistence and build/validation hygiene are documented but not yet
   expressed as a target architecture for future offline/download/client-cache
   work.

## Target State

When this workstream closes:

- Android has a deeper Public Client runtime seam that centralizes generic
  execution policy without erasing route-family product semantics.
- Browse route side effects are explicit, testable, and separated from pure
  state reduction and rendering. Route loading, saveable-state publishing, and
  stale-response handling have clear ownership.
- Player has an Android-owned runtime seam that can survive feature growth and
  owns Media3 lifecycle orchestration while Rust remains limited to portable
  request/decision semantics.
- UI generic design-system components, media-specific components, and screen
  route composition have clear module ownership. Redundant pass-through wrappers
  and obsolete transition code are removed.
- Home uses a section-oriented read model that can render partial content,
  skeleton/degraded states, and progressive Managed Artwork enrichment.
- Remaining persistence/build/validation debt is either implemented if small and
  in-scope, or split into narrower follow-ons with explicit gates.
- All changes preserve token safety: bearer tokens, local source locators,
  local paths, and FFmpeg/server internals do not enter visible UI, saved route
  state, diagnostics, smoke evidence, or logs.

## In Scope

- Android app code under `apps/android/app/src/main/java/dev/nako/android`.
- Android unit tests under `apps/android/app/src/test/java/dev/nako/android`.
- Focused Rust client-core / UniFFI changes only when required to support an
  Android runtime seam already endorsed by ADR 0032.
- Workstream docs and validation evidence under this directory.
- Deleting redundant Android wrappers, obsolete transition code, and duplicated
  UI components when covered by tests and not owned by active follow-on lanes.
- Updating Android README or validation scripts when the refactor changes local
  development or gate expectations.

## Out Of Scope

- No server route shape changes and no Public Client API v2 contract changes.
- No Rust-owned Android networking, profile persistence, token vault, Compose UI,
  Media3 player, media session, PiP, Cast, or platform permission behavior.
- No downloads/offline playback implementation.
- No Android TV shell, external player handoff, or subtitle/audio-track UX.
- No KMP/iOS/Maven SDK publishing policy.
- No migration away from single `:app` Gradle module unless this lane proves
  concrete dependency pressure and records a split decision first.
- No broad server-side refactor.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Earlier Android refactor lanes are closed and should not be reopened as catch-all lanes. | High | Multiple `WORKSTREAM.json` files are closed/completed. | Reopening them would blur closeout evidence; create follow-ons instead. |
| Package-level seams still provide better leverage than an immediate Gradle module split. | Medium | AFCR-070 kept Kotlin package seams; current app has one production shell. | If build/dependency pressure is measured, split a dedicated Gradle-module workstream. |
| Route-family clients duplicate enough execution policy to justify a deeper runtime seam. | High | Architecture review found repeated token/core/executor/decode/error flow. | If implementation proves duplication is lower than expected, record deletion/keep decision and move to Browse/Player seams. |
| Player runtime should remain Android-owned. | High | ADR 0026/0032 keep Media3 and platform behavior in the shell. | If a future platform-independent player model is needed, it requires a new ADR/follow-on. |
| Home can improve UX without server API changes by changing client read-model orchestration. | Medium | Current Public Client API already exposes library/item/search/user-playback/artwork routes. | If API gaps block partial sections, split server/API work instead of inventing local semantics. |

## Architecture Direction

### Deep module criteria

A module introduced in this lane must hide enough complexity that deleting it
would make callers substantially worse. Shallow pass-through wrappers should be
removed or folded into a deeper owner.

### Public Client runtime

Create a runtime seam above `PublicClientApiExecutor` only for generic
execution policy:

- token lookup and missing-token classification;
- Rust-core descriptor to Android request conversion;
- executor invocation and safe request preview propagation;
- generated SDK decode orchestration;
- common diagnostics and redaction primitives;
- optional future retry/metrics hooks.

Keep product semantics in route-family clients:

- browse/search/facet failure categories;
- playback target and session-preflight decisions;
- User Playback State progress/watched semantics;
- user-facing copy and UI state.

### Browse effects

Browse state reducers should not rely on implicit host state collection to
perform route side effects. Prefer an explicit effect or load-intent interface
that `BrowseShellHost` or a successor coordinator executes. This keeps route
state deterministic while preserving Android-owned transport, token, and UI.

### Player runtime

Introduce a PlayerRuntime/PlaybackSessionRuntime seam only on the Android side.
It may own Media3 lifecycle orchestration, event mapping, exit effect dispatch,
resume seek policy, and future MediaSession/PiP/Cast seams. Rust remains limited
to portable playback decision/request construction and public response
interpretation.

### UI and read models

Separate:

- generic reusable design-system surfaces in `ui/components`;
- Nako media-specific rows/cards/chips in a media/browse component module;
- screen-level route composition in `ui/screens` and `ui/browse`;
- display-model builders and copy decisions outside large composable bodies when
  they are independently testable.

Home should move from a monolithic load to section-level read models with
partial/degraded states and progressive Managed Artwork enrichment.

## Closeout Condition

This lane can close when:

- all accepted tasks in `TODO.md` are complete or explicitly split;
- stale wrappers and obsolete transition code introduced by earlier Android
  foundation lanes are removed where safe;
- focused and full Android validation gates pass with fresh evidence;
- token/redaction and route-state safety regressions pass;
- workstream docs reflect the shipped behavior;
- remaining large product scopes are split into follow-on workstreams instead of
  hidden in this lane.
