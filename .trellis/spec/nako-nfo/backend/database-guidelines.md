# Database Guidelines

`nako-nfo` uses repository traits to find media sources and metadata records. It
does not own database schema or migrations.

## Required Patterns

- Use `MediaRepository` or other core traits for source and item lookups.
- Use `StorageBackend` for sidecar content, never direct path reads.
- Keep import/export state as workflow results unless a repository contract
  explicitly persists it.
- Use fingerprints to decide whether sidecar content changed.

## Forbidden Patterns

- Do not import SQL adapters or database pools.
- Do not write canonical metadata tables from codec code.
- Do not treat sidecar file presence as proof of catalog identity.
- Do not persist preview-only decisions.

## Import/Export Rules

- Import can produce decisions such as skip, update, create, or fail depending
  on policy and source state.
- Export should respect existing sidecar content and preservation support.
- Missing source records should fail through repository errors, not local path
  guessing.

## Tests Required

- Repository fake tests for source lookup and missing source cases.
- Fingerprint tests for skip/update decisions.
- Preview tests proving no storage mutation.
