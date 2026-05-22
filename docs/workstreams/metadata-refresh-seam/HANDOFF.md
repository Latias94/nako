# Metadata Refresh Seam Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M40 has been opened after M39 closed `CatalogHydrationPort`. The first
candidate slice is a metadata refresh workflow port. The implementation is in
place in `crates/nako-metadata/src/strategy.rs`.

## Completed Task

- MRS-010 through MRS-060 are complete.

## Decisions Since Last Update

- Do not add provider breadth in M40.
- Do not change public API, SDK, CLI, playback, NFO Round Trip, or database
  schema in the first slice.
- Keep catalog hydration behind M39's `CatalogHydrationPort`.
- Use `MetadataRefreshPort` for refresh snapshot/commit and
  `MetadataAttemptPort` for attempt diagnostics.
- Keep provider fetch/search outside the persistence port.

## Blockers

- None.

## Next Recommended Action

- Open a later goal for provider runtime or library scan/probe seam deepening
  only after M40 is committed.
