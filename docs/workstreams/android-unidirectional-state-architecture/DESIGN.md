# Android Unidirectional State Architecture - Design

Status: Active
Last updated: 2026-05-19

## Problem

`TaruBrowseShell` still owns too much state and orchestration. It stores route
state, page state, selection state, search state, retry keys, source probe
state, playback decision state, and playback-start orchestration directly in a
Composable. Multiple `LaunchedEffect` blocks coordinate asynchronous work by
watching route and refresh keys.

That shape was reasonable for the first client foundation, but it is no longer
the cleanest architecture for extending Browse Facets, Media Item Detail,
Media Source selection, User Playback State, and Playback Source Selection.

## Target State

- Browse becomes a unidirectional state module:
  - immutable `BrowseShellState`,
  - explicit `BrowseAction`,
  - optional one-shot `BrowseEffect`,
  - a testable `BrowseSession` that owns state transitions and asynchronous
    orchestration.
- Compose screens render state and dispatch actions. They do not call Taru
  clients, read tokens, manage refresh counters, or construct playback launch
  requests.
- Existing deep modules stay in place:
  - `PlaybackStartCoordinator`,
  - `PlaybackExitCoordinator`,
  - `resolvePlaybackResumePosition`,
  - navigation stack saver and pure route model.
- The UI remains Compose and Material 3; this lane is architectural, not a
  visual redesign.

## Architecture Direction

- Start with a pure Kotlin `BrowseSession` rather than immediately introducing
  AndroidX ViewModel. The session is the state machine and test surface.
- Add lifecycle adapters later if needed. AndroidX ViewModel and StateFlow are
  useful adapters, not the core architecture.
- Move one vertical slice at a time:
  1. navigation and selected destination,
  2. home/search/library loading,
  3. detail/source/playback selection,
  4. playback start route opening,
  5. UI shell cleanup.
- Use explicit actions instead of `refreshKey += 1`.
- Guard async responses so stale route loads cannot overwrite current state.

## Non-Goals

- Changing Public Client API contracts.
- Replacing Compose with another UI framework.
- Adding Hilt or a large dependency-injection framework.
- Redesigning visual surfaces.
- Changing playback semantics already proven by smoke and coordinator tests.

## Assumptions

- Current Public Client API coverage remains the runtime source of truth.
- `TaruBrowseNavigationState` is already a useful pure Kotlin route model and
  should be reused rather than replaced in this lane.
- `output/` and `tmp/` are generated or local work directories and must remain
  untouched.
