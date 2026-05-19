# Android Presentation Runtime Adapters - Design

Status: Closed
Last updated: 2026-05-20

## Why This Lane Exists

The Browse unidirectional state workstream moved route state, asynchronous
loading, source selection, playback decision, and playback start into
`BrowseSession`. `TaruBrowseShell` is now mostly a Compose adapter, but it still
passes runtime credentials and concrete runtime collaborators into visual
surfaces:

- `PublicArtworkSource` is built in the shell and passed through Home and
  Libraries.
- Media Item Detail receives `ServerProfile` plus raw `accessToken` only so it
  can construct artwork requests.
- The shell still passes `TokenVault` and playback clients directly into the
  Player route. Player lifecycle cleanup belongs to the player architecture
  lane, but the Browse shell should stop knowing player internals as early as
  possible.

That shape is functional, but it is not the cleanest architecture for extending
presentation, previews, screenshot evidence, dynamic color, and future server
profile switching.

## Relevant Authority

- ADRs:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- Existing docs:
  - `docs/workstreams/android-unidirectional-state-architecture/`
  - `docs/workstreams/android-material-expressive-ui/`
  - `docs/workstreams/android-player-session-integrity/`
- Related follow-on:
  - `docs/workstreams/android-player-session-architecture/`

## Problem

`TaruBrowseShell` and detail rendering still expose shallow runtime seams:

- visual screens can see credential-shaped values even when they only need
  already-built artwork requests;
- shell tests and previews must know runtime objects that are not part of the
  visual contract;
- player route construction is a concrete call site instead of a small Browse
  runtime interface;
- token reads are duplicated at presentation edges, making future refresh or
  profile-switch behavior harder to reason about.

## Target State

- Browse presentation receives presentation-safe inputs:
  - artwork requests or an artwork resolver,
  - no raw access token on detail visual APIs,
  - no direct `TokenVault` reads inside `TaruBrowseShell` for artwork.
- Player route launch from Browse goes through a small runtime adapter interface
  so the shell no longer depends on the concrete player Composable signature.
- Existing user-visible behavior stays unchanged.
- The public client, playback start, playback exit semantics, and Media3 player
  lifecycle remain unchanged in this lane.

## In Scope

- `apps/android/app/src/main/java/dev/taru/android/ui/browse/`
- `apps/android/app/src/main/java/dev/taru/android/ui/screens/detail/`
- `apps/android/app/src/main/java/dev/taru/android/ui/artwork/`
- a narrow adapter under `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/`
- focused JVM tests for adapter contracts and presentation APIs
- workstream evidence and closeout docs

## Out Of Scope

- Rewriting Media3 player lifecycle, ExoPlayer ownership, or playback exit side
  effects.
- Changing Public Client API, playback session identity, or server contracts.
- Visual redesign beyond removing runtime-shaped props from presentation
  surfaces.
- Settings or connection onboarding state architecture.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Artwork requests can be built once at the Browse runtime edge and passed to visual surfaces without losing dynamic fallback behavior. | High | `PublicArtworkSource.requestFor` already returns request objects consumed by `TaruArtworkImage`. | If false, keep a small `ArtworkRequestResolver` interface instead of concrete requests. |
| Player lifecycle deserves its own workstream. | High | `PlaybackPlayerRoute` owns ExoPlayer, listener state, errors, and exit effects. | If false, this lane would become too broad and block clean closeout. |
| Existing JVM tests can cover the adapter contracts without emulator smoke. | Medium | Previous UDF lane covered runtime orchestration with JVM tests. | If behavior risk appears, run focused emulator smoke as an additional gate. |

## Architecture Direction

Introduce narrow presentation/runtime adapters where the shell currently leaks
runtime implementation details:

- an artwork presentation adapter that turns profile + token vault + image
  metadata into presentation-safe `PublicArtworkRequest` inputs;
- a detail route API that accepts those inputs or a resolver rather than
  `ServerProfile` and raw `accessToken`;
- a Browse-owned player route renderer interface so `TaruBrowseShell` can render
  a `PlaybackLaunchRequest` without knowing the concrete player dependency
  list.

The key rule: presentation modules should render explicit state and dispatch
explicit callbacks. Runtime modules may know tokens, clients, stores, and
platform objects.

## Closeout Condition

This lane can close when:

- Browse artwork and detail visual APIs no longer accept raw access tokens;
- Browse shell no longer reads tokens for artwork presentation;
- Player route rendering is behind a narrow Browse runtime adapter;
- focused JVM tests and final diff checks pass;
- `android-player-session-architecture` remains the explicit follow-on for
  player lifecycle/session ownership.

## Closeout Notes

- `ArtworkRequestResolver` is now the presentation-safe artwork request seam.
  `TokenVaultArtworkRequestResolver` is the production adapter that reads the
  current profile token at request time.
- Home, Libraries, Media Poster rows/cards, and Media Item Detail now receive
  resolver-shaped inputs rather than `PublicArtworkSource` or raw access
  tokens.
- `PlayerRouteRenderer` is now the Browse-to-Player runtime seam. Browse can
  render a `PlaybackLaunchRequest` without knowing the concrete player route
  dependency list.
- Media3 lifecycle, player reducer state, and exit idempotency remain in the
  dedicated `android-player-session-architecture` follow-on.
