# Android Unidirectional State Architecture - Handoff

Status: Closed
Last updated: 2026-05-19

## Current Task

Closed.

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
- AUSA-060 is complete. Resume position calculation is behind
  `BrowseResumeResolver`; `TaruBrowseShell` no longer owns refresh keys, async
  loads, route-specific state, source selection, playback decision, or playback
  start orchestration.
- AUSA-070 is complete. Focused session tests, full debug unit tests, and
  `git diff --check` passed.

## Follow-ons

- Consider a future presentation/runtime adapter to reduce direct token props
  passed into artwork, detail, and player rendering surfaces. This was left
  outside this workstream because it is not state orchestration and would touch
  screen/player contracts.
