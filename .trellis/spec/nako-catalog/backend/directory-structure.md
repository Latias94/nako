# Directory Structure

`nako-catalog` currently keeps its implementation in `src/lib.rs`. Split only
when a group of graph builders or projection helpers becomes large enough to
need its own module.

## Current Layout

- `CatalogHydrationPort`: read-model orchestration entry point.
- `CatalogHydrationSummary`: summary counts for hydration runs.
- `CatalogGraphReplacement` / `CatalogItemGraphReplacement`: replacement
  payloads used to publish complete graph state.
- Builder helpers for item, person, genre, tag, collection, studio, and provider
  subject graphs.
- Projection helpers for `CatalogSearchProjection`.

## Module Split Rules

- Keep repository orchestration near the hydration port.
- Move pure graph-building helpers into private modules only when they stop
  being readable in `lib.rs`.
- Keep public output structs stable and explicit.
- Keep search document evaluation out of this crate.

## Naming Rules

- Use `Catalog*Graph` for hydrated read models.
- Use `*Replacement` for complete replacement payloads.
- Use `CatalogSearchProjection` for search-indexable documents.
- Use `ProviderSubject` terminology for accepted external mappings.

## Anti-Patterns

- Do not create a `service` module that mixes hydration, API mapping, and
  search ranking.
- Do not add per-provider adapter modules in catalog.
- Do not create database-specific catalog modules.
