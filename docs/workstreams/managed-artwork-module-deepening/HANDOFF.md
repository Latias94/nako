# Managed Artwork Module Deepening Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

The lane is closed. `MAMD-020` through `MAMD-070` are complete.

App-layer splits:

- `crates/nako-server/src/app/artwork/variant.rs` owns Selected Artwork variant
  request validation, original/derived byte envelope creation, resizing, and
  presentation ETag behavior.
- `crates/nako-server/src/app/artwork/artifact_store.rs` owns local Managed
  Artwork Artifact path layout, storage URI validation, writes, reads, deletes,
  inventory, and file classification.
- `crates/nako-server/src/app/artwork/ingest_pipeline.rs` owns remote fetch,
  content-type normalization, image validation, content hash creation, artifact
  file write preparation, success summary serialization, and safe failure
  summary serialization.

DB-layer splits:

- `crates/nako-db/src/artwork/gallery.rs` owns Admin gallery SQL/query/row
  mapping.
- `crates/nako-db/src/artwork/lifecycle.rs` owns artifact lifecycle
  SQL/query/summary/row mapping and unselected artifact cleanup.
- `crates/nako-db/src/artwork/selected.rs` owns Selected Artwork get/list SQL
  and publication/unpublication transactions.
- `crates/nako-db/src/artwork/candidate.rs` owns Artwork Candidate repository
  methods, lookup SQL, and status update helpers.
- `crates/nako-db/src/artwork/ingest.rs` owns Managed Artwork Ingest
  lookup/insert helpers plus job transaction helpers used by ingest state
  transitions.
- `crates/nako-db/src/artwork/artifact.rs` owns Managed Artwork Artifact
  insert/get helpers.

API-layer split:

- `crates/nako-api/src/admin/managed_artwork.rs` owns Managed Artwork Admin
  DTOs, conversion helpers, and DTO-level redaction tests.
- `crates/nako-api/src/admin.rs` re-exports the module so `nako_api::*` and HTTP
  callers keep stable public names.
- `selected_artwork_to_public_image_ref` intentionally remains in
  `public_client.rs` because it is the Public Client protocol DTO conversion
  boundary and remains small.

## Latest Validation

Fresh MAMD-070 closeout validation passed:

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo fmt --all -- --check
cargo check -j 2 -p nako-server -p nako-api -p nako-db --tests
cargo nextest run -j 2 -p nako-api managed_artwork --no-fail-fast
cargo nextest run -j 2 -p nako-db artwork --no-fail-fast
cargo nextest run -j 2 -p nako-server managed_artwork --no-fail-fast
git diff --check
```

## Follow-Ons

No residual follow-on was split from this architecture lane. Product scopes
that remain intentionally out of scope should stay in their own future lanes if
they become priority work.

## Non-Goals To Preserve

- Do not add provider search.
- Do not add Public Client gallery browsing.
- Do not add persisted thumbnail cache eviction.
- Do not add missing-artifact repair or re-ingest.
- Do not add new runtime retry, cancellation, backoff, or lease semantics.
- Do not expose raw source URLs, storage URIs, local paths, cache URIs, or
  content hashes.
