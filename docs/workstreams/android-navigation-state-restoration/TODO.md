# Android Navigation State Restoration - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

- [x] ANS-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-navigation-state-restoration]
  Goal: Open the restoration lane and freeze scope to Android browse navigation
  state.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-navigation-state-restoration/DESIGN.md`.
  Handoff: Completed on 2026-05-19. First implementation slice should add a
  tested saveable snapshot and wire `NakoBrowseShell` to it.

## M1 - Saveable Navigation State

- [x] ANS-020 [owner=codex] [deps=ANS-010] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/test/java/dev/nako/android/ui/browse]
  Goal: Add safe save/restore support for `NakoBrowseNavigationState` and wire
  `NakoBrowseShell` through `rememberSaveable`.
  Validation:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.NakoBrowseNavigationStateSaverTest --no-daemon`
  plus Android compile or broader unit tests as risk requires.
  Review: Confirm Player routes are transient and no playback request material
  appears in saved navigation data.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Completed on 2026-05-19. Added JSON-backed save/restore payload,
  `NakoBrowseNavigationStateSaver`, `rememberSaveable` Shell wiring, and focused
  coverage for valid, invalid, unknown, and transient Player restoration.

## M2 - Closeout

- [x] ANS-030 [owner=planner] [deps=ANS-020] [scope=docs/workstreams/android-navigation-state-restoration]
  Goal: Verify gates, close this lane, and split deeper route/deep-link
  follow-ons.
  Validation: fresh ANS-020 gate plus `git diff --check`.
  Review: Confirm docs teach the restoration policy and remaining work is
  deferred intentionally.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Closed on 2026-05-19 after focused and full Android debug unit gates
  plus diff hygiene passed.
