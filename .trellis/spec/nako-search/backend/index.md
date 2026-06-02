# nako-search Backend Guidelines

`nako-search` owns transport-free search documents, query evaluation, and
ranking primitives. It currently performs pure in-memory evaluation over
`SearchDocument` values supplied by catalog projections.

## Current Evidence

- `crates/nako-search/src/lib.rs`
- `crates/nako-catalog/src/lib.rs`
- `crates/nako-core/src/lib.rs`

## Boundaries

- Define search documents, query inputs, hits, facets, and evaluation results.
- Use `nako_core::CATALOG_SEARCH_PROJECTION_VERSION` for projection versioning.
- Keep catalog hydration in `nako-catalog`.
- Keep HTTP query parsing and response DTOs in `nako-api` or `nako-server`.
- Keep external search engine adapters out until a real adapter exists.

## Required Patterns

- Evaluate search deterministically in memory.
- Normalize query text before matching.
- Clamp pagination through `PageRequest`.
- Score title, alias, body, and facet matches with the existing weights.
- Sort by score descending, then item ID ascending.

## Forbidden Patterns

- Do not mutate catalog state during search.
- Do not reach into database or storage adapters.
- Do not add provider-specific search behavior here.
- Do not expose transport-specific error or DTO types.

## Validation

- Focused search tests:
  `cargo nextest run -p nako-search --no-fail-fast`
- Projection contract checks:
  `cargo check -p nako-search -p nako-catalog --tests`
