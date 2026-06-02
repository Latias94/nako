# Logging Guidelines

Catalog hydration should emit redaction-safe operational context if logging or
tracing is added. The current crate mostly returns summaries instead of logging.

## Required Patterns

- Prefer structured fields such as item ID, graph kind, relationship count, and
  projection document count.
- Use `CatalogHydrationSummary` for caller-visible run outcomes.
- Keep provider labels and external IDs out of high-cardinality logs unless
  needed for explicit diagnostics.
- Log accepted mapping counts rather than raw provider payloads.

## Forbidden Patterns

- Do not log metadata provider secrets or raw provider responses.
- Do not log full catalog graph payloads.
- Do not hide missing root errors behind generic hydration-failed messages.

## Useful Fields

- `catalog.item_id`
- `catalog.graph_kind`
- `catalog.relationship_count`
- `catalog.projection_count`
- `catalog.accepted_provider_subject_count`
