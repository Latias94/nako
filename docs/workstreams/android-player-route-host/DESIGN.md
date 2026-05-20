# Android Player Route Host

Status: Closed
Last updated: 2026-05-20

## Problem

`PlaybackPlayerRoute` currently owns too much runtime orchestration:

- `PlayerSession` construction and event dispatch.
- Media3 prepare/retry/back/dispose event mapping.
- Playback listener callbacks.
- Exit effect triggering.
- Clipboard and overlay rendering.

The session reducer is tested, but the route-level orchestration is still
embedded in Compose effects and local functions. This makes playback lifecycle
changes hard to test without a UI route.

## Target State

- Introduce `PlayerRouteHost` as the route orchestration module.
- Host owns `PlayerSession`, engine event mapping, prepare/retry/back/dispose,
  and exit effect triggering.
- Compose creates the concrete Media3 engine and exit runner, then renders host
  state and binds `PlayerView` to the engine player.
- Keep visible playback UI and Media3 behavior unchanged.

## Scope

- `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/`
- Focused player route host tests.
- Workstream docs under this directory.

## Non-Goals

- Do not replace Media3.
- Do not change playback start/session server behavior.
- Do not redesign player chrome.
- Do not add new user feedback for copy diagnostics.

## Architecture Direction

`PlayerSession` remains the pure playback route state machine. `PlayerRouteHost`
is a host module around it: it concentrates route lifecycle and effect
orchestration behind a small interface and makes those transitions testable
without Compose.

## Closeout

Closed on 2026-05-20. `PlayerRouteHost` now owns `PlayerSession` dispatch,
Media3 route listener mapping, prepare/retry/back/dispose handling, release
idempotency, and exit effect triggering. `PlaybackPlayerRoute` creates the
host, binds `PlayerView` to the host player, renders overlay state from
`StateFlow`, and forwards UI actions to the host.
