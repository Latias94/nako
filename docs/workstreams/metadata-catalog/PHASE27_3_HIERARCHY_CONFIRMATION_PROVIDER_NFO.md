# Phase 27.3: Hierarchy Confirmation and Provider/NFO Expansion

## Status

Completed.

## Objective

Build on M27.2's source-owned **Local Inference Evidence** and **Provisional
Hierarchy** so NFO and built-in providers can confirm series, season, and
episode items in place instead of replacing Nako item identity.

## Implemented Scope

M27.3 adds these vertical slices:

- `nako-metadata` owns a **Hierarchy Confirmation** service boundary for
  provider/NFO confirmation of provisional items.
- Hierarchy confirmation updates existing `MediaItem` rows in place, marks
  `LibraryItemState.provisional = false`, hydrates catalog/search projections,
  and rejects structural changes to already confirmed items unless a later
  **Hierarchy Repair** flow owns them.
- Provider-backed confirmation writes accepted **Provider Mapping** and
  **Provider Subject** records through the M27.1 repository boundary.
- Metadata refresh now records accepted provider mapping for successful TMDB,
  Douban, and Bangumi fetches instead of treating provider payloads as only
  raw response cache entries.
- `nako-nfo` imports episode NFO hierarchy fields through the shared
  confirmation service, preserving local/NFO authority and confirming
  provisional series/season/episode items in place.
- `nako-metadata` TMDB provider supports series search and movie, series,
  season, and episode fetches.

## Compatibility

- Existing movie metadata refresh remains compatible.
- Existing movie NFO import/export behavior remains compatible.
- NFO confirmation does not create replacement items.
- Provider mappings remain provider-neutral links to Nako `MediaItem`
  identity.

## Non-Goals

- No Source Variant UI.
- No browse API.
- No artwork candidate, selected artwork, or managed artwork expansion.
- No rating, user state, or browse facet work.
- No automatic duplicate merge.
- No general **Hierarchy Repair** flow for confirmed structural mistakes.

## Validation

Close-out commands:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-db --no-fail-fast`
- `cargo nextest run -p nako-library --no-fail-fast`
- `cargo nextest run -p nako-metadata --no-fail-fast`
- `cargo nextest run -p nako-nfo --no-fail-fast`
- `git diff --check`

## Remaining Boundaries

- Provider-driven hierarchy confirmation jobs can now build on the shared
  service boundary instead of adding a second rematch path.
- Douban and Bangumi provider breadth currently enters the shared provider
  mapping boundary; deeper provider-specific hierarchy semantics remain a
  follow-up.
- NFO round-trip preservation for unknown XML remains part of the broader
  **NFO Round Trip** and **Library File Write** boundary.
