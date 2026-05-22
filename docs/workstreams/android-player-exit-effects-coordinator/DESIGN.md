# Android Player Exit Effects Coordinator - Design

Status: Closed
Last updated: 2026-05-19

## Problem

`PlaybackPlayerRoute` still owns too much exit behavior. It snapshots Media3
state, starts a detached coroutine, reads the server token, wires
`NakoUserPlaybackClient` progress/watched calls, wires
`NakoPlaybackClient.cancelPlaybackSession`, and persists device-local position.

The underlying rules live in `applyPlaybackExitEffects`, but the UI route still
assembles the side-effect graph. That keeps networking and playback lifecycle
coordination in a Compose file.

## Target State

- A focused player exit coordinator/use case owns the client wiring for:
  - device-local position persistence,
  - User Playback State progress/watched reports,
  - active playback session cancellation.
- `PlaybackPlayerRoute` keeps only Media3 snapshot capture, once-only guarding,
  and route navigation.
- Existing exit semantics remain unchanged:
  - unfinished session playback saves progress and cancels the session,
  - ended playback clears local position and reports watched,
  - missing token preserves local position and skips network side effects.

## Architecture Direction

- Keep this Android-side only. Do not change server contracts.
- Keep `applyPlaybackExitEffects` as the domain rule boundary.
- Add a small coordinator around real Android clients instead of passing client
  lambdas from UI.
- Preserve detached exit execution semantics unless a safer app-level scope is
  introduced in a separate lane.

## Non-Goals

- Redesign player controls or overlays.
- Change Media3 player construction.
- Change progress/watched thresholds.
- Change session cancellation server behavior.
- Add emulator smoke states unless targeted tests reveal a behavior risk.

## Assumptions

- Previous playback session lanes already proved end-to-end cancellation
  behavior; this lane is a maintainability refactor around the same contract.

## Outcome

- Added `PlaybackExitCoordinator` as the player exit client-wiring boundary.
- `PlaybackPlayerRoute` now captures Media3 snapshots and delegates exit
  effects to the coordinator.
- `applyPlaybackExitEffects` remains the domain rule boundary.
