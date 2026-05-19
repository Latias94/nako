# Android Player Effect Scope Cleanup - Design

Status: Closed
Last updated: 2026-05-19

## Problem

`PlaybackPlayerRoute` no longer owns user playback and cancellation wiring, but
it still creates its own detached `CoroutineScope(SupervisorJob() +
Dispatchers.Main.immediate)` for exit effects. That keeps lifecycle ownership in
the route and makes the exit path slightly harder to reason about.

We want a stable app-level effect scope that is created once at the app shell
layer and passed down to the player route.

## Target State

- App root owns the player exit effect scope.
- `PlaybackPlayerRoute` accepts an injected scope and launches exit work
  through it.
- No route-local detached scope creation remains.
- Existing exit semantics remain unchanged.

## Architecture Direction

- Keep the scope anchored at `TaruAndroidAppContent` or an equivalent app
  shell boundary.
- Use the existing Compose lifecycle to own the scope, not a global singleton.
- Keep the change mechanical and narrow: no new business logic.

## Non-Goals

- Redesign playback exit semantics.
- Change `applyPlaybackExitEffects`.
- Rework playback player UI.

## Assumptions

- The previously extracted `PlaybackExitCoordinator` remains the business
  boundary for exit behavior.

## Outcome

- Added a small player exit effect launcher that uses an injected scope.
- `TaruAndroidApp` now owns the player exit effect scope and passes it through
  `TaruBrowseShell`.
- `PlaybackPlayerRoute` no longer constructs a detached `SupervisorJob`.
