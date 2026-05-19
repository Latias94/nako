# Android Player Session Architecture - TODO

Status: Closed
Last updated: 2026-05-20

## M0 - Scope And Evidence Freeze

- [x] APSA-010 [owner=codex] [deps=none] [scope=docs/workstreams/android-player-session-architecture]
  Goal: Freeze player session architecture scope after the presentation/runtime adapter lane closes.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: docs/workstreams/android-player-session-architecture/DESIGN.md
  Handoff: DONE. APRA is closed; APSA is now the active player architecture lane.

## M1 - Player State Reducer

- [x] APSA-020 [owner=codex] [deps=APSA-010] [scope=apps/android/app/src/main/java/dev/taru/android/ui/screens/player,apps/android/app/src/test/java/dev/taru/android/ui/screens/player]
  Goal: Extract player display state, error state, retry/back/dispose transitions, and labels into a JVM-testable session/reducer.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel
  Review: Keep Media3 engine calls outside the reducer.
  Evidence: focused player tests.
  Handoff: DONE. `PlayerSession` owns display labels, sanitized error state, retry, and idempotent back/dispose exit requests.

## M2 - Engine And Exit Adapters

- [x] APSA-030 [owner=codex] [deps=APSA-020] [scope=apps/android/app/src/main/java/dev/taru/android/player,apps/android/app/src/main/java/dev/taru/android/ui/screens/player]
  Goal: Put Media3 commands and exit effects behind narrow adapters while preserving current playback behavior.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.* --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel
  Review: Verify exit effects remain idempotent and release ordering is explicit.
  Evidence: focused player and exit tests.
  Handoff: DONE. `PlaybackEngineController` wraps Media3 prepare/snapshot/release; `PlaybackExitEffectRunner` wraps exit side effects.

## M3 - Route Cleanup And Runtime Gate

- [x] APSA-040 [owner=codex] [deps=APSA-030] [scope=apps/android/app/src/main/java/dev/taru/android/ui/screens/player]
  Goal: Reduce `PlaybackPlayerRoute` to Compose rendering/platform glue and decide whether emulator smoke is required.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel; git diff --check
  Review: If Media3 behavior changes materially, add emulator smoke evidence.
  Evidence: final test output and optional smoke report.
  Handoff: DONE. Route now wires session, engine, exit runner, and rendering; no emulator smoke required because Media3 behavior was moved behind adapters without changing playback semantics.

## M4 - Closeout

- [x] APSA-050 [owner=codex] [deps=APSA-040] [scope=docs/workstreams/android-player-session-architecture]
  Goal: Close the lane or split any runtime-only follow-on.
  Validation: final gates recorded in EVIDENCE_AND_GATES.md.
  Review: No blocking findings from workstream review.
  Evidence: WORKSTREAM.json, HANDOFF.md
  Handoff: DONE. Final Android unit tests and `git diff --check` passed; runtime long-media cancellation remains a separate optional validation lane.
