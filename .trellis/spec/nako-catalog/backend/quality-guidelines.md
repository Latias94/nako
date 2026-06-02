# Quality Guidelines

Catalog changes must preserve graph completeness, deterministic output, and
clear separation from search scoring.

## Required Patterns

- Build full replacement payloads for graph publication.
- Keep item/source/credit/genre/tag/collection/studio/provider subject output
  deterministic.
- Normalize labels used in search projection facts.
- Keep `CatalogHydrationSummary` aligned with produced graph counts.
- Preserve the current catalog/search split: catalog produces documents, search
  evaluates queries.

## Forbidden Patterns

- Do not publish partial graph state as a replacement.
- Do not add ranking scores or query matching here.
- Do not collapse provider subjects into canonical identities.
- Do not use provider-centric names when a Nako catalog term exists.

## Tests Required

- Hydration summary count tests.
- Graph replacement tests for item relationships and accepted mappings.
- Search projection tests for labels, aliases, facets, and deterministic
  ordering.
- Missing root tests for `NakoError::NotFound`.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-catalog --no-fail-fast`
- Projection contract:
  `cargo check -p nako-catalog -p nako-search --tests`
