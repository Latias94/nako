# Android Unidirectional State Architecture - Milestones

Status: Active
Last updated: 2026-05-19

## M0 - Lane Setup

Exit: AUSA-010 complete.

## M1 - Browse Session Skeleton

Status: Complete.

Exit:

- `BrowseShellState`, `BrowseAction`, and `BrowseSession` exist.
- Navigation actions are tested through `BrowseSession`.
- `TaruBrowseNavigationState` remains the route model.

## M2 - First Async Loading Slice

Exit:

- Home, Library Detail, Search, and Browse Facet loading are handled by
  `BrowseSession`.
- Retry actions are explicit and no longer depend on refresh counters for the
  migrated slice.
- Stale response handling is tested.

## M3 - Detail And Playback Selection Slice

Exit:

- Media Item Detail, Media Source selection, source probe, and playback
  decision state live in `BrowseSession`.
- Source selection and playback decision reset behavior is explicit and tested.

## M4 - Playback Start Integration

Exit:

- Playback start is a `BrowseAction`.
- `BrowseSession` uses `PlaybackStartCoordinator`.
- Player route opening is owned by session state/effect rather than UI-local
  orchestration.

## M5 - Compose Shell Cleanup

Exit:

- `TaruBrowseShell` renders state and dispatches actions.
- Client calls and token reads are removed from the shell.
- Local `refreshKey` counters and route-driven `LaunchedEffect` loaders are
  removed from the shell.

## M6 - Closeout

Exit:

- Focused session tests and full debug unit tests pass.
- Workstream docs record evidence and remaining follow-ons.
