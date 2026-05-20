# Android Settings Connection State Architecture - Handoff

Status: Closed
Last updated: 2026-05-20

## Current Task

Closed.

## Notes

- Keep code and technical docs in English; user-facing replies stay Chinese.
- Do not touch generated `output/` or `tmp/`.
- Preserve existing connection behavior and diagnostics wording unless tests
  prove the current behavior is wrong.
- Follow Browse/Player architecture: session state and actions first, Compose
  as rendering/platform glue.
- ASCSA-020 through ASCSA-040 are complete:
  - `ConnectionSession` owns connection form/test/save/switch/failure state;
  - `SettingsSession` owns server profile switch and sign-out actions;
  - `TaruAppSession` owns root snapshot and connection visibility.
- Final gates passed on 2026-05-20:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
  - `git diff --check`

## Follow-ons

- User accounts, sessions, OAuth/OIDC, RBAC, and token refresh are outside this
  lane.
