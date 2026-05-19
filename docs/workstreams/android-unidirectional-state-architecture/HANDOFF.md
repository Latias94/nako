# Android Unidirectional State Architecture - Handoff

Status: Active
Last updated: 2026-05-19

## Current Task

AUSA-060: reduce `TaruBrowseShell` to state rendering and action dispatch.

## Notes

- This is an intentional fearless refactor. Do not preserve shallow local state
  patterns in `TaruBrowseShell` unless a slice has not migrated yet.
- Keep existing coordinator/use case modules and reuse them.
- Keep commits precise by task slice.
- Do not touch generated `output/` or `tmp/` directories.
- AUSA-020 is complete. Navigation actions now enter through
  `BrowseAction`, `BrowseShellState` is saveable through the existing
  navigation payload, and focused browse navigation JVM tests pass.
- AUSA-030 is complete. Home, Library Detail, Search, and Browse Facet loading
  now run through `BrowseSession` and `BrowseDataSource`; shell no longer owns
  those refresh keys or route-driven loaders. Focused loading tests and full
  debug unit tests pass.
- AUSA-040 is complete. Media Item Detail, selected Media Source, source probe,
  playback decision, and related retry state now live in `BrowseSession`.
- AUSA-050 is complete. Playback start now runs through `BrowseSession` and
  `BrowsePlaybackStarter`; `TaruBrowseShell` only dispatches `StartPlayback`.
  Success opens the Player route, and failure leaves playback diagnostics in
  session state.
