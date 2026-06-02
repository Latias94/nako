# nako-catalog Backend Guidelines

`nako-catalog` hydrates catalog graph read models from core repository traits
and builds search projections. It is a pure orchestration/read-model crate, not
an HTTP, database, or search-engine adapter.

## Current Evidence

- `crates/nako-catalog/src/lib.rs`
- `crates/nako-core/src/lib.rs`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/LANES.md`

## Boundaries

- Use `CatalogHydrationPort` as the composition entry point.
- Load catalog facts through `nako-core` repository traits.
- Return `CatalogItemGraph`, person/genre/tag/collection/studio graphs, and
  `CatalogSearchProjection`.
- Keep search scoring and query evaluation in `nako-search`.
- Keep transport DTO mapping in `nako-api` or `nako-server`.

## Required Patterns

- Hydrate a complete graph replacement before publishing a projection.
- Use accepted provider mappings when adding provider subjects.
- Normalize provider labels and facet labels before projection output.
- Preserve deterministic ordering in hydrated graph components.
- Report missing catalog records with `NakoError::NotFound`.

## Forbidden Patterns

- Do not add direct SQL or storage adapter calls here.
- Do not issue HTTP responses or route-level DTOs from this crate.
- Do not run search ranking in catalog hydration.
- Do not mutate canonical metadata while hydrating read models.

## Validation

- Focused catalog checks:
  `cargo nextest run -p nako-catalog --no-fail-fast`
- Cross-layer compile when projection shape changes:
  `cargo check -p nako-catalog -p nako-search -p nako-api --tests`
