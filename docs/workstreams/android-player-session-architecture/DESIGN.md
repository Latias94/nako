# Android Player Session Architecture - Design

Status: Closed
Last updated: 2026-05-20

## Why This Lane Exists

Android playback now has stronger public contracts for playback source
selection, playback start, session identity, exit reporting, and active session
cancellation. The remaining weakness is local architecture: `PlaybackPlayerRoute`
still owns Media3 setup, player state labels, error presentation, retry
behavior, exit effect triggering, and ExoPlayer release in one Composable.

That is acceptable for a first implementation, but it is not the cleanest shape
for long-term playback correctness.

## Relevant Authority

- ADRs:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- Existing docs:
  - `docs/workstreams/android-playback-session-integrity/`
  - `docs/workstreams/android-active-playback-session-cancellation/`
  - `docs/workstreams/android-player-exit-effects-coordinator/`
  - `docs/workstreams/android-playback-start-flow-coordinator/`
  - `docs/workstreams/android-device-local-playback-position/`
- Predecessor:
  - `docs/workstreams/android-presentation-runtime-adapters/`

## Problem

`PlaybackPlayerRoute` is a shallow module: callers pass many runtime
dependencies, and the Composable internally coordinates player creation,
preparation, listener state, error conversion, retry, back handling, exit
effects, and release ordering. That makes it hard to test ordering-sensitive
playback lifecycle behavior without a UI or emulator.

## Target State

- Player lifecycle is represented by a testable player session module:
  - immutable player UI/runtime state,
  - explicit player actions/events,
  - a small player engine interface for Media3,
  - a small exit effect interface for progress/session cleanup.
- `PlaybackPlayerRoute` becomes a Compose renderer and platform adapter.
- Retry, back, dispose, error, and exit-effect idempotency are covered by JVM
  tests where possible.
- Existing playback launch, progress reporting, session cancellation, and
  visual behavior are preserved.

## In Scope

- `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/`
- `apps/android/app/src/main/java/dev/taru/android/player/`
- focused player JVM tests
- optional emulator smoke if runtime risk warrants it
- workstream evidence and closeout docs

## Out Of Scope

- Changing server playback APIs or Public Client contracts.
- Changing playback source selection or Browse playback start orchestration.
- Redesigning the player visual surface.
- Implementing long-media runtime fixtures unless required to verify a bug.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Media3 can sit behind a small player engine adapter while most ordering logic becomes JVM-testable. | Medium | Current route already separates `preparePlayer` and `persistPositionAndCancelSession` helpers. | If false, keep Media3 inside the route but move reducer/exit idempotency into pure modules. |
| Exit side effects should remain idempotent and external to Compose disposal timing. | High | Existing `PlaybackExitCoordinator` and tests cover side-effect behavior. | If wrong, session cancellation/progress reporting could regress under back/dispose races. |
| Browse should not own player runtime dependencies after the predecessor lane. | High | Presentation runtime adapter workstream explicitly narrows the Browse call site. | If predecessor is incomplete, start with the player route renderer adapter. |

## Architecture Direction

Deepen the player module around a `PlayerSession` concept:

- the session accepts launch input and explicit events such as prepared,
  buffering, playing, paused, error, retry, back, and disposed;
- Media3-specific creation and commands live behind a player engine adapter;
- exit side effects are triggered once by session policy rather than scattered
  through Compose callbacks;
- the Compose route observes/render state and forwards platform events.

This follows the same direction as Browse UDF: put state and lifecycle policy
behind a small test surface, keep Compose as rendering and platform glue.

## Closeout Condition

This lane can close when:

- player lifecycle policy is testable without a Composable;
- `PlaybackPlayerRoute` no longer owns reducer-like player state and exit
  idempotency policy directly;
- focused player tests and final diff checks pass;
- any remaining emulator-only runtime evidence is split or documented.

## Closeout Notes

- `PlayerSession` now owns player display state, sanitized playback error
  state, retry state, and idempotent back/dispose exit requests.
- `PlaybackEngineController` now hides Media3 prepare, listener attachment,
  snapshot, and release commands from route policy.
- `PlaybackExitEffectRunner` now hides coroutine dispatch and
  `PlaybackExitCoordinator` invocation from the route.
- `PlaybackPlayerRoute` remains the Compose/platform renderer: it wires the
  session, engine, exit runner, Android `PlayerView`, and UI overlays.
- No emulator smoke was required for this lane because playback semantics were
  preserved and covered by focused/full JVM tests. Runtime long-media
  cancellation remains a separate validation lane if needed.
