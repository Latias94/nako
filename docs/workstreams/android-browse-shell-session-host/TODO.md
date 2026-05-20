# Android Browse Shell Session Host - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] ABSH-010 - Freeze host boundary and validation gates.
  - Owner: Codex
  - Scope: Workstream docs only.
  - Validation: Docs exist and describe the host boundary.
  - Handoff: Boundary is recorded in `DESIGN.md`.

- [x] ABSH-020 - Add browse shell host state module.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/BrowseShellHost.kt`
    - `apps/android/app/src/test/java/dev/taru/android/ui/browse/BrowseShellHostTest.kt`
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --no-daemon --no-parallel`
  - Handoff: Host owns initial load, route display, dispatch persistence, async persistence, and settings action forwarding.

- [x] ABSH-030 - Move client runtime assembly out of `TaruBrowseShell`.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/ClientBrowseShellRuntime.kt`
    - `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
  - Validation:
    - Focused browse tests pass.
  - Handoff: Compose only constructs runtime/host and renders collected state.

- [x] ABSH-040 - Verify integration and close lane.
  - Owner: Codex
  - Scope: Workstream docs and validation.
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel`
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
    - `git diff --check`
  - Handoff: Close or split follow-ons.
