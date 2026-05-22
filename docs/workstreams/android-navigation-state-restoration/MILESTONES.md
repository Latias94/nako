# Android Navigation State Restoration - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

Exit criteria:

- Workstream docs exist and agree on Android browse navigation restoration
  scope.
- Jetpack Navigation, deep links, and active playback session restoration are
  deferred.

Status: Complete.

## M1 - Saveable Navigation State

Exit criteria:

- Navigation state has an explicit save/restore adapter.
- Safe routes restore across Activity recreation.
- Player route restore drops to the previous safe route.
- `NakoBrowseShell` uses the saver through `rememberSaveable`.
- Focused JVM tests cover valid, transient, and invalid saved states.

Status: Complete.

## M2 - Closeout

Exit criteria:

- Evidence docs reference final reports.
- TODO, DESIGN, HANDOFF, and WORKSTREAM status are closed.
- Follow-ons such as deep links, route URI contracts, and active playback
  session restoration remain split.

Status: Complete.
