# Managed Artwork Module Deepening Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is opened and `MAMD-020`, `MAMD-030`, and `MAMD-040` are complete.
Selected Artwork variant serving has moved into `artwork/variant.rs`; local
Managed Artwork Artifact file storage and inventory have moved into
`artwork/artifact_store.rs`; ingest fetch/validate/write/failure-summary logic
has moved into `artwork/ingest_pipeline.rs`.

## Goal

Improve locality and leverage around Managed Artwork app/db/api modules while
preserving existing public/Admin behavior and redaction contracts.

## Completed Tasks

`MAMD-020` extracted Selected Artwork variant serving into
`crates/taru-server/src/app/artwork/variant.rs`.

Result:

- `ImageVariantRequest` and `ManagedArtworkImageBytes` remain available to HTTP
  code through the existing app re-export;
- variant policy, artifact media-type planning, original/derived byte envelope
  creation, resizing, and presentation ETag generation live in the private
  variant Module;
- `read_selected_image` keeps orchestration only and preserves the previous
  error ordering.

`MAMD-030` extracted local Managed Artwork Artifact storage and inventory into
`crates/taru-server/src/app/artwork/artifact_store.rs`.

Result:

- local path layout, storage URI validation, write/read/delete operations, file
  status, recursive inventory, discovered file parsing, and path-prefix checks
  are local to the artifact store Module;
- the store Module reports internal `ArtifactStoreFileIssue` values;
- `ManagedArtworkAppService` projects those issues into Admin storage-drift DTO
  reasons, keeping `taru-api` out of the file storage Module.

`MAMD-040` extracted Managed Artwork ingest execution into
`crates/taru-server/src/app/artwork/ingest_pipeline.rs`.

Result:

- remote fetch, content-type normalization, image validation, content hash
  creation, artifact file write preparation, success summary serialization, and
  safe failure summary serialization live in the ingest pipeline Module;
- durable claim, database artifact commit, best-effort file rollback after
  commit failure, and failure commit ordering remain in `ManagedArtworkAppService`;
- no retry, cancellation, backoff, lease, repair, or re-ingest semantics were
  added.

## Next Task

Continue with `MAMD-050`: split `crates/taru-db/src/artwork.rs` into
concern-local SQLite adapter modules while preserving existing `taru-core`
repository traits and public crate exports.

Progress so far:

- `artwork/gallery.rs` owns Admin gallery SQL/query/row mapping.
- `artwork/lifecycle.rs` owns artifact lifecycle SQL/query/summary/row mapping.
- `artwork/selected.rs` owns Selected Artwork get/list SQL and
  publication/unpublication transactions.
- `artwork.rs` still owns repository trait impls and the remaining candidate,
  ingest/artifact, and cleanup transaction helpers.

Recommended next split:

- core ingest/artifact transaction helpers, split after checking whether
  cleanup should remain in the parent repository impl or move beside lifecycle.

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
cargo check -j 2 -p taru-server --tests
cargo check -j 2 -p taru-db --tests
cargo nextest run -j 2 -p taru-db artwork --no-fail-fast
git diff --check
```

## Notes

No subagents were started for this lane. If parallel work is explicitly
requested later, safe splits after `MAMD-050` lands are `MAMD-060` API surface
audit and closeout review.
