# Android Unidirectional State Architecture - Handoff

Status: Active
Last updated: 2026-05-19

## Current Task

AUSA-030: migrate Home, Library Detail, Search, and Browse Facet loading from
`TaruBrowseShell` into `BrowseSession`.

## Notes

- This is an intentional fearless refactor. Do not preserve shallow local state
  patterns in `TaruBrowseShell` unless a slice has not migrated yet.
- Keep existing coordinator/use case modules and reuse them.
- Keep commits precise by task slice.
- Do not touch generated `output/` or `tmp/` directories.
- AUSA-020 is complete. Navigation actions now enter through
  `BrowseAction`, `BrowseShellState` is saveable through the existing
  navigation payload, and focused browse navigation JVM tests pass.
