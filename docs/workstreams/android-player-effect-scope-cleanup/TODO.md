# Android Player Effect Scope Cleanup - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

- [x] APESC-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-player-effect-scope-cleanup]
  Goal: Open the refactor lane and define the app-scope boundary.
  Validation: workstream docs exist and agree.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: APESC-020 is ready.

## M1 - Scope Injection

- [x] APESC-020 [owner=codex] [deps=APESC-010] [scope=apps/android/app/src/main/java/dev/taru/android/ui,apps/android/app/src/main/java/dev/taru/android/ui/screens/player,apps/android/app/src/test/java/dev/taru/android/ui/screens/player]
  Goal: Move player exit coroutine ownership to the app shell and inject it into
  `PlaybackPlayerRoute`.
  Validation: focused Android JVM tests cover the scope wiring, and the full
  debug unit suite still passes.
  Evidence: `PlayerExitEffectScopeTest` and full debug unit tests passed on
  2026-05-19.
  Handoff: APESC-030 completed in the same closeout pass.

## M2 - Closeout

- [x] APESC-030 [owner=codex] [deps=APESC-020] [scope=docs/workstreams/android-player-effect-scope-cleanup]
  Goal: Verify no behavior regression and close the workstream.
  Validation: targeted Android JVM tests, full debug unit tests, `git diff --check`.
  Evidence: closeout notes in `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
