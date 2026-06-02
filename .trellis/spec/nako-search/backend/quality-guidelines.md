# Quality Guidelines

Search behavior must be deterministic, projection-version-aware, and easy to
test without external services.

## Required Patterns

- Keep evaluation pure and side-effect free.
- Normalize query terms and document text before matching.
- Preserve the current scoring weights:
  - title match: `1.0`
  - alias match: `0.9`
  - body match: `0.7`
  - facet match: `0.5`
- Apply required facet filters before scoring.
- Sort by score descending and item ID ascending.

## Forbidden Patterns

- Do not add nondeterministic tie-breaking.
- Do not add network calls or search engine clients here.
- Do not let callers bypass `PageRequest` bounds.
- Do not mix catalog hydration concerns into scoring code.

## Tests Required

- Query normalization tests.
- Required facet filter tests.
- Weight and tie-break ordering tests.
- Pagination clamping tests.
- Projection version tests when document shape changes.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-search --no-fail-fast`
- Cross-crate:
  `cargo check -p nako-search -p nako-catalog --tests`
