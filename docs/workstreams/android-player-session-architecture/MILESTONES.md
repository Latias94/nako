# Android Player Session Architecture - Milestones

Status: Closed
Last updated: 2026-05-20

## M0 - Lane Ready

Exit criteria:

- player session target state is written;
- predecessor dependency on presentation/runtime adapters is clear;
- evidence plan separates JVM policy tests from optional emulator smoke.

## M1 - Player State Reducer

Exit criteria:

- player status labels, error presentation, retry, back, and dispose policy are
  represented by a testable state module;
- current labels and diagnostics remain stable.

## M2 - Engine And Exit Adapters

Exit criteria:

- Media3 commands are behind a small adapter;
- exit effects are triggered through one idempotent policy;
- existing `PlaybackExitCoordinator` tests still pass.

## M3 - Route Cleanup

Exit criteria:

- `PlaybackPlayerRoute` primarily renders UI and attaches platform callbacks;
- no duplicated exit-effect trigger logic remains in the Composable;
- final JVM gates pass.

## M4 - Closeout

Exit criteria:

- docs reflect shipped behavior;
- follow-ons are explicit;
- workstream is closed or paused with a concrete next task.
