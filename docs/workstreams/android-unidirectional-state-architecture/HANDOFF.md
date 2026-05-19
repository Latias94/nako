# Android Unidirectional State Architecture - Handoff

Status: Active
Last updated: 2026-05-19

## Current Task

AUSA-040: migrate Media Item Detail loading, selected Media Source state,
source probe, playback decision, and retry events into `BrowseSession`.

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
