# Metadata Catalog Commit Atomicity Evidence And Gates

Status: Proposed
Last updated: 2026-05-18

## Smallest Current Repro

Before the first implementation slice, the known consistency gap is visible in
the call ordering:

- `crates/taru-catalog/src/lib.rs` calls
  `replace_item_catalog_graph` and then `SearchIndex::upsert`.
- `crates/taru-db/src/catalog.rs` wraps only graph replacement in a SQLite
  transaction.
- `crates/taru-db/src/search.rs` writes the Search Projection separately.

## Gate Set

### Catalog Package Gate

```bash
cargo check -p taru-catalog --tests
cargo nextest run -p taru-catalog --no-fail-fast
```

These commands prove the catalog hydration workflow and fake-port tests still
compile and pass.

### SQLite Adapter Gate

```bash
cargo check -p taru-db --tests
cargo nextest run -p taru-db <filter>
```

The targeted test filter should include the new graph/search commit behavior.

### Workspace Hygiene Gate

```bash
cargo fmt --all -- --check
git diff --check
```

Use broader workspace gates before closing the lane or before merging with a
large metadata refresh commit slice.

## Evidence Anchors

- `docs/workstreams/metadata-catalog-commit-atomicity/DESIGN.md`
- `docs/workstreams/metadata-catalog-commit-atomicity/TODO.md`
- `crates/taru-core/src/media/catalog.rs`
- `crates/taru-core/src/repository/catalog.rs`
- `crates/taru-core/src/repository/metadata.rs`
- `crates/taru-catalog/src/lib.rs`
- `crates/taru-db/src/catalog.rs`
- `crates/taru-db/src/metadata.rs`
- `crates/taru-db/src/search.rs`
- `crates/taru-metadata/src/strategy.rs`

## Fresh Evidence

2026-05-18:

- `cargo check -p taru-catalog --tests` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-nfo --tests` passed.
- `cargo check -p taru-library --tests` passed.
- `cargo check -p taru-server --tests` passed.
- `cargo nextest run -p taru-catalog --no-fail-fast` passed: 3 tests.
- `cargo nextest run -p taru-db sqlite_store_rolls_back_catalog_graph_when_search_projection_commit_fails` passed: 1 test.
- `cargo nextest run -p taru-db sqlite_store_round_trips_scan_state_search_and_artwork_tasks` passed: 1 test.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with LF/CRLF warnings from existing working-copy files.

One attempted command used the wrong test filter:
`cargo nextest run -p taru-db sqlite_store_indexes_search_documents` ran zero
tests because no such test name exists.

2026-05-18, MCC-030:

- `cargo check -p taru-catalog --tests` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-server --tests` passed.
- `cargo nextest run -p taru-db commit_metadata_refresh --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p taru-metadata --no-fail-fast` passed: 27 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with LF/CRLF warnings from existing working-copy
  files.

Behavior proven:

- `MetadataRepository::commit_metadata_refresh` persists Canonical Metadata,
  Provider Raw Response, Provider Subject, accepted Provider Mapping, and
  Library Item State confirmation as one SQLite transaction.
- A provider subject unique-index failure rolls back the item metadata update,
  raw response cache, provider mapping creation, and library confirmation.
- A mismatched Provider Raw Response item ID is rejected before persistence.

2026-05-18, MCC-040 closeout:

- `cargo check -p taru-catalog --tests` passed.
- `cargo check -p taru-db --tests` passed.
- `cargo check -p taru-metadata --tests` passed.
- `cargo check -p taru-server --tests` passed.
- `cargo nextest run -p taru-db commit_metadata_refresh --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p taru-db sqlite_store_rolls_back_catalog_graph_when_search_projection_commit_fails --no-fail-fast`
  passed: 1 test.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

Closeout decision:

- This lane is complete.
- The remaining larger question, if needed, is a follow-up about metadata
  refresh plus prepared catalog hydration or an event-driven projection
  pipeline. That is intentionally out of scope for this lane.

## Notes

Record fresh command output after each task. Do not mark a task complete based
on old evidence from another lane.
