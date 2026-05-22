# Phase 27.2: Local Inference and Provisional Hierarchy

## Status

Completed implementation slice.

## Objective

Connect M27.1's local inference persistence to the scan path so Nako can keep
source-owned parsing evidence and create a provider-neutral provisional video
hierarchy before TMDB, Douban, Bangumi, or NFO confirmation.

## Implemented Scope

M27.2 added these vertical slices:

- `nako-naming` parsed-name output now carries confidence, evidence source,
  evidence value, parser version, inferred kind, title, year, season, and
  episode fields.
- Weak file-name evidence now produces an **Unknown Media Item** instead of a
  confident movie guess.
- `nako-library` scan indexing writes **Local Inference Evidence** for scanned
  sources.
- **Local Inference Evidence** is updated as the current source-owned snapshot
  for each evidence source and parser version instead of appending scan history.
- Rescan preserves confirmed **Canonical Metadata** while still refreshing
  source state and **Local Inference Evidence**.
- Episode-like local inference creates provisional series, season, and episode
  items during scanning.
- `nako-db` persists minimal **Library Item State** membership so source-less
  provisional series and season items remain library-scoped.

## Compatibility

- Confident movie names with a year still index as movie items.
- Existing source locator identity remains library-scoped.
- Provisional hierarchy creation does not add provider mappings, NFO
  confirmation, Source Variant UI, or browse API behavior.
- Local inference evidence is source-owned and does not become canonical
  metadata by itself.
- Repeated scans of the same source update the current local inference evidence
  snapshot rather than accumulating duplicate inference records.
- Local inference may seed canonical fields while an item remains provisional,
  but rescan preserves canonical metadata after the item is confirmed.

## Validation

Commands run:

- `cargo nextest run -p nako-naming` - 6 passed
- `cargo nextest run -p nako-library` - 15 passed
- `cargo nextest run -p nako-db` - 32 passed
- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `git diff --check` - passed with Git CRLF normalization warnings only

## Remaining Boundaries

M27.3 should build on the provisional hierarchy and provider mapping schema:

- TMDB series, season, and episode provider mapping.
- Douban provider MVP.
- Bangumi provider MVP.
- NFO hierarchy confirmation and round-trip preservation.
- Artwork candidate, selected artwork, and managed artwork contracts.

Client browse/sort DTOs and Source Variant UI remain later M27 work unless
their API names and repository queries are explicitly designed first.
