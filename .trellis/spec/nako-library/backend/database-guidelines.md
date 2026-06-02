# Database Guidelines

`nako-library` coordinates persistence through `nako-core` repository traits.

## Rules

- Use workflow traits such as `LibraryIngestionWorkflow` and
  `LibraryProbeWorkflow` to express persistence needs.
- Keep implementations generic over repository traits:
  `CatalogRepository`, `IngestionFailureRepository`, `LibraryItemRepository`,
  `LibraryRepository`, `MediaRepository`, `MediaProbeRepository`, and
  `ScanRepository`.
- Use `PageRequest::MAX_LIMIT` loops when reading all sources or source states.
- Persist scan start/finish, directory observations, source observations,
  probe results, and ingestion failures through repositories.
- Schema or repository contract changes belong in `nako-core` and `nako-db`.

## Good/Base/Bad Cases

- Good: `LibraryIngestionWorkflow::commit_source_observation` builds a typed
  source observation plan and calls `ScanRepository::commit_library_scan_source`.
- Base: `LibraryProbeService` lists sources page by page, probes them, and
  records failures through repository traits.
- Bad: scan code directly opens a database transaction or assumes SQLite row
  shape.

## Evidence

- `crates/nako-library/src/ingestion.rs`
- `crates/nako-library/src/probe.rs`
- `crates/nako-core/src/repository/scan.rs`
- `crates/nako-db/src/contract_tests.rs`
