# Database Guidelines

`nako-catalog` depends on repository traits from `nako-core`; it does not own
schema, migrations, or SQL.

## Required Patterns

- Read catalog state through `MediaRepository`, `MetadataRepository`, and other
  relevant core traits.
- Use the repository APIs that already return domain records, links, and
  accepted provider mappings.
- Treat hydrated graph output as a read model derived from canonical records.
- Keep repository calls explicit enough that missing data paths are reviewable.

## Forbidden Patterns

- Do not import `sqlx`, `rusqlite`, database pools, or migration files.
- Do not infer storage schema details from repository implementation types.
- Do not persist search projections directly from this crate unless a repository
  trait is explicitly added for that contract.
- Do not backfill canonical metadata from projection labels.

## Missing Data Rules

- Missing requested item/person/genre/tag/collection/studio records should be
  reported as `NakoError::NotFound`.
- Optional relationships may be absent without failing the whole graph.
- Accepted provider mappings are evidence; rejected or tentative mappings must
  not appear in published graph projections.

## Tests Required

- Repository-backed hydration tests should use fake or test repositories.
- Add cases for missing requested records, missing optional relationships, and
  accepted provider mappings.
