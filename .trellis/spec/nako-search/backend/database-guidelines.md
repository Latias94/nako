# Database Guidelines

`nako-search` has no database ownership. It evaluates supplied documents and
does not load, persist, or invalidate indexes by itself.

## Required Patterns

- Accept `SearchDocument` or evaluation documents from callers.
- Use projection version values from `nako-core`.
- Keep pagination behavior consistent with `PageRequest`.
- Let catalog or future repository adapters decide where documents are stored.

## Forbidden Patterns

- Do not import SQL, database pools, migrations, or repository adapters.
- Do not persist search documents from inside `evaluate_search_documents`.
- Do not bypass catalog projection generation to read canonical tables.
- Do not add implicit global indexes.

## Versioning Rules

- Preserve `projection_version` on search documents.
- When projection shape changes, update the catalog producer and search
  consumer together.
- Add tests for mixed or mismatched projection data before accepting version
  changes.

## Tests Required

- Pure evaluation tests should construct documents in memory.
- Projection tests should compile with `nako-catalog` when shape changes.
