# Android Settings Connection State Architecture - TODO

Status: Closed
Last updated: 2026-05-20

## M0 - Scope And Evidence Freeze

- [x] ASCSA-010 [owner=codex] [deps=none] [scope=docs/workstreams/android-settings-connection-state-architecture]
  Goal: Open the lane and freeze the connection/settings state architecture target.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: docs/workstreams/android-settings-connection-state-architecture/DESIGN.md
  Handoff: DONE. Implementation starts with ConnectionSession.

## M1 - Connection Session

- [x] ASCSA-020 [owner=codex] [deps=ASCSA-010] [scope=apps/android/app/src/main/java/dev/taru/android/ui/connection,apps/android/app/src/test/java/dev/taru/android/ui/connection]
  Goal: Move connection form state, async test, save, switch, failure recording, and token persistence into a testable `ConnectionSession`.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.connection.* --tests dev.taru.android.connection.* --no-daemon --no-parallel
  Review: Compose connection screen should render state and dispatch actions only.
  Evidence: focused JVM tests and diff.
  Handoff: DONE. Connection form/test/save/switch/failure recording now live in `ConnectionSession`; UI renders state and dispatches actions.

## M2 - Settings Session

- [x] ASCSA-030 [owner=codex] [deps=ASCSA-020] [scope=apps/android/app/src/main/java/dev/taru/android/ui/screens/settings,apps/android/app/src/test/java/dev/taru/android/ui/screens/settings]
  Goal: Move server profile switching and sign-out token deletion out of settings visual rendering into a testable session/runtime adapter.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.settings.* --no-daemon --no-parallel
  Review: Settings screens should receive state and callbacks, not token vault mutation.
  Evidence: focused JVM tests and diff.
  Handoff: DONE. `SettingsSession` owns switch/sign-out actions and settings screens no longer mutate token vault or repository directly.

## M3 - Root App State

- [x] ASCSA-040 [owner=codex] [deps=ASCSA-030] [scope=apps/android/app/src/main/java/dev/taru/android/ui,apps/android/app/src/test/java/dev/taru/android/ui]
  Goal: Make root app connection visibility an explicit tested state transition after save, switch, sign-out, and reconnect actions.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.* --tests dev.taru.android.ui.connection.* --tests dev.taru.android.ui.screens.settings.* --no-daemon --no-parallel
  Review: Root should coordinate sessions, not duplicate repository decisions.
  Evidence: focused JVM tests and diff.
  Handoff: DONE. `TaruAppSession` owns root snapshot and connection visibility transitions without changing Browse/Player architecture.

## M4 - Closeout

- [x] ASCSA-050 [owner=codex] [deps=ASCSA-040] [scope=docs/workstreams/android-settings-connection-state-architecture]
  Goal: Verify final gates, close the lane, and record follow-ons.
  Validation: apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel; git diff --check
  Review: No direct repository/token mutation remains in connection/settings visual screens.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. Final Android unit tests and `git diff --check` passed; auth/session/RBAC remain separate future work.
