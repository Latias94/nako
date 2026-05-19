# Managed Artwork Module Deepening Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is opened and `MAMD-020`, `MAMD-030`, `MAMD-040`, and `MAMD-050`
are complete.

App-layer splits:

- `crates/taru-server/src/app/artwork/variant.rs` owns Selected Artwork variant
  request validation, original/derived byte envelope creation, resizing, and
  presentation ETag behavior.
- `crates/taru-server/src/app/artwork/artifact_store.rs` owns local Managed
  Artwork Artifact path layout, storage URI validation, writes, reads, deletes,
  inventory, and file classification.
- `crates/taru-server/src/app/artwork/ingest_pipeline.rs` owns remote fetch,
  content-type normalization, image validation, content hash creation, artifact
  file write preparation, success summary serialization, and safe failure
  summary serialization.

DB-layer splits:

- `crates/taru-db/src/artwork/gallery.rs` owns Admin gallery SQL/query/row
  mapping.
- `crates/taru-db/src/artwork/lifecycle.rs` owns artifact lifecycle
  SQL/query/summary/row mapping and unselected artifact cleanup.
- `crates/taru-db/src/artwork/selected.rs` owns Selected Artwork get/list SQL
  and publication/unpublication transactions.
- `crates/taru-db/src/artwork/candidate.rs` owns Artwork Candidate repository
  methods, lookup SQL, and status update helpers.
- `crates/taru-db/src/artwork/ingest.rs` owns Managed Artwork Ingest
  lookup/insert helpers plus job transaction helpers used by ingest state
  transitions.
- `crates/taru-db/src/artwork/artifact.rs` owns Managed Artwork Artifact
  insert/get helpers.

`crates/taru-db/src/artwork.rs` still owns the existing repository trait methods
for Managed Artwork and routes through these concern modules.

## Goal

Improve locality and leverage around Managed Artwork app/db/api modules while
preserving existing public/Admin behavior and redaction contracts.

## Latest Validation

Fresh MAMD-050 validation passed:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo fmt --all -- --check
cargo check -j 2 -p taru-db --tests
cargo nextest run -j 2 -p taru-db artwork --no-fail-fast
git diff --check
```

The `git diff --check` command only reported Git line-ending notices.

## Next Task

Continue with `MAMD-060`: audit Managed Artwork DTO locality and redaction
tests in:

- `crates/taru-api/src/admin.rs`
- `crates/taru-api/src/admin/**`
- `crates/taru-api/src/public_client.rs`
- related OpenAPI or HTTP docs only if the DTO boundary actually changes

Recommended approach:

- inventory Managed Artwork DTOs and conversion helpers;
- keep explicit DTO names and redaction tests close to conversion code;
- split API modules only if it reduces caller knowledge or removes real
  concentration;
- do not change OpenAPI/Public Client contracts unless the change is explicitly
  documented and tested.

## Non-Goals To Preserve

- Do not add provider search.
- Do not add Public Client gallery browsing.
- Do not add persisted thumbnail cache eviction.
- Do not add missing-artifact repair or re-ingest.
- Do not add new runtime retry, cancellation, backoff, or lease semantics.
- Do not expose raw source URLs, storage URIs, local paths, cache URIs, or
  content hashes.

## Suggested Validation

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo check -j 2 -p taru-api -p taru-server --tests
cargo nextest run -j 2 -p taru-api managed_artwork --no-fail-fast
cargo nextest run -j 2 -p taru-server managed_artwork --no-fail-fast
git diff --check
```

Narrow the test filters to exact Admin/Public Client redaction tests if
`managed_artwork` is too broad.
