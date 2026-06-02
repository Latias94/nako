# Directory Structure

`nako-search` currently fits in `src/lib.rs`. Keep it compact unless query
parsing, scoring, and facet handling each become large enough to justify
private modules.

## Current Layout

- `SearchDocument`: indexable catalog document.
- `SearchQuery`: normalized query input with filters and pagination.
- `SearchHit` / `SearchEvaluationResult`: output from pure evaluation.
- `SearchEvaluationDocument`: helper for evaluation against projection facts.
- Facet conversion helpers using `BrowseFacet::parse_label`.
- `evaluate_search_documents`: pure ranking and pagination function.

## Module Split Rules

- Keep public structs and constructors close together.
- Move private scoring helpers only if they grow beyond simple matching logic.
- Keep projection conversion explicit so version drift is visible.
- Keep adapter-specific code out of this crate.

## Naming Rules

- Use `SearchDocument` for stored or projected search records.
- Use `SearchEvaluationDocument` for runtime scoring input.
- Use `SearchHit` for ranked results.
- Use `BrowseFacet` labels for facet parsing and matching.

## Anti-Patterns

- Do not create modules named after providers.
- Do not add API route modules.
- Do not create database-backed indexer modules in this crate.
