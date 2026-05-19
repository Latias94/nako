# Managed Artwork Module Deepening Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "ManagedArtworkAppService|ImageVariantRequest|ManagedArtworkImageBytes|LocalManagedArtworkArtifactStore|ArtifactStoreFileInventory|ManagedArtworkFetcher|ManagedArtworkImageValidator|artifact_lifecycle_diagnostics|artifact_remediation_plan|cleanup_untracked_artifact_files" crates/taru-server/src/app/artwork.rs crates/taru-db/src/artwork.rs crates/taru-api/src/admin.rs
git diff --check
```

This inventory anchors the current concentration of Managed Artwork behavior.

## Gate Set

Use low-concurrency validation when running Rust commands:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
```

### Lane Open Gate

```powershell
Get-Content docs\workstreams\managed-artwork-module-deepening\WORKSTREAM.json | ConvertFrom-Json | Out-Null
git diff --check
```

### Variant Module Gate

```powershell
cargo check -j 2 -p taru-server -p taru-api --tests
cargo nextest run -j 2 -p taru-server managed_artwork_variant --no-fail-fast
cargo nextest run -j 2 -p taru-api managed_artwork_variant --no-fail-fast
git diff --check
```

### Artifact Store Module Gate

```powershell
cargo check -j 2 -p taru-server --tests
cargo nextest run -j 2 -p taru-server "managed_artwork|artifact|lifecycle|remediation|drift" --no-fail-fast
git diff --check
```

### Ingest Pipeline Gate

```powershell
cargo check -j 2 -p taru-server -p taru-db -p taru-api --tests
cargo nextest run -j 2 -p taru-server artwork --no-fail-fast
cargo nextest run -j 2 -p taru-db artwork --no-fail-fast
git diff --check
```

### Redaction Inventory

```powershell
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root" crates/taru-api crates/taru-server/src/http crates/taru-server/src/app docs/api
```

Remaining hits must be internal storage logic, explicit redaction fields,
route/query documentation, or tests proving forbidden values are absent.

## Evidence Anchors

- `CONTEXT.md`
- `docs/adr/0013-bounded-artwork-task-resource-classes.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/managed-artwork-public-serving-selection/HANDOFF.md`
- `docs/workstreams/managed-artwork-thumbnail-variants/HANDOFF.md`
- `docs/workstreams/managed-artwork-gallery-candidate-management/HANDOFF.md`
- `docs/workstreams/managed-artwork-artifact-lifecycle-cleanup/HANDOFF.md`
- `docs/workstreams/managed-artwork-remediation-policy/HANDOFF.md`
- `docs/workstreams/managed-artwork-ingest-runtime-controls/HANDOFF.md`
- `docs/workstreams/selected-artwork-unpublish-delete-policy/HANDOFF.md`

## Fresh Evidence

2026-05-19, MAMD-010:

- Opened this lane from the post-artwork feature follow-on review.
- Scope decision:
  - deepen existing Managed Artwork implementation modules;
  - preserve product behavior and public/Admin contracts;
  - keep provider search, Public Client gallery, persisted thumbnail eviction,
    repair/re-ingest, and new runtime retry/cancel semantics out of scope;
  - keep raw source URLs, storage URIs, local paths, cache URIs, and content
    hashes redacted.
- Fresh validation:
  - `Get-Content docs\workstreams\managed-artwork-module-deepening\WORKSTREAM.json | ConvertFrom-Json | Out-Null`:
    passed.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-020:

- Extracted Selected Artwork variant serving into
  `crates/taru-server/src/app/artwork/variant.rs`.
- App-layer change:
  - `ManagedArtworkAppService::read_selected_image` now validates the request
    through `ImageVariantPolicy`, builds a selected image variant plan before
    reading artifact bytes, and delegates original/derived byte envelope
    creation to the variant Module;
  - invalid dimension limits are still rejected before DB lookups;
  - missing artifact media type is still rejected before file reads;
  - public crate-facing `ImageVariantRequest` and `ManagedArtworkImageBytes`
    re-exports remain stable.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-server -p taru-api --tests`: passed.
  - `cargo nextest run -j 2 -p taru-server managed_artwork_variant --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-api managed_artwork_variant --no-fail-fast`:
    passed; 3 tests passed.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-030:

- Extracted local Managed Artwork Artifact storage and inventory into
  `crates/taru-server/src/app/artwork/artifact_store.rs`.
- App-layer change:
  - local path layout, storage URI validation, write/read/delete operations,
    file status checks, recursive store inventory, discovered file parsing, and
    path-prefix checks moved behind `LocalManagedArtworkArtifactStore`;
  - the store Module now reports internal `ArtifactStoreFileIssue` values;
  - `ManagedArtworkAppService` owns projection from internal store issues to
    Admin storage-drift DTO reasons, keeping `taru-api` DTOs out of the file
    storage Module.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-server --tests`: passed.
  - `cargo nextest run -j 2 -p taru-server managed_artwork_variant --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server admin_process_next_managed_artwork_ingest_stores_internal_artifact_without_public_artwork --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server admin_managed_artwork_cleanup_removes_only_unselected_artifacts_without_locator_leaks --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server admin_managed_artwork_storage_drift_reports_missing_and_stray_files_without_locator_leaks --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server admin_managed_artwork_remediation_requires_confirmation_and_deletes_only_untracked_artifact_files --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server "admin_managed_artwork_(cleanup|storage_drift|remediation)" --no-fail-fast`:
    no tests matched; replaced by the three exact test filters above.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-040:

- Extracted Managed Artwork ingest execution into
  `crates/taru-server/src/app/artwork/ingest_pipeline.rs`.
- App-layer change:
  - remote fetch, content-type normalization, image validation, content hash
    creation, artifact file write preparation, success summary serialization,
    and safe failure summary serialization moved behind
    `ManagedArtworkIngestPipeline`;
  - `ManagedArtworkAppService` still owns durable claim, database artifact
    commit, best-effort artifact rollback after commit failure, and failure
    commit ordering;
  - artifact store writes now return a local write error, and the ingest
    pipeline maps that into the existing `storage_failed` ingest failure.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-server --tests`: passed.
  - `cargo nextest run -j 2 -p taru-server admin_process_next_managed_artwork_ingest_stores_internal_artifact_without_public_artwork --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server admin_process_next_managed_artwork_ingest_fails_with_redacted_safe_summary_for_unsupported_media_type --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -j 2 -p taru-server admin_process_next_managed_artwork_ingest_fails_with_redacted_safe_summary_for_invalid_image --no-fail-fast`:
    passed; 1 test passed.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-050 partial:

- Split the first SQLite Managed Artwork adapter concerns:
  - `crates/taru-db/src/artwork/gallery.rs` now owns Admin gallery SQL, query
    helpers, and row mapping;
  - `crates/taru-db/src/artwork/lifecycle.rs` now owns artifact lifecycle SQL,
    lifecycle summary aggregation, and lifecycle row helpers;
  - `crates/taru-db/src/artwork.rs` keeps the repository trait implementation
    and routes through those modules.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-db --tests`: passed.
  - `cargo nextest run -j 2 -p taru-db artwork --no-fail-fast`: passed; 11
    tests passed.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-050 selected split:

- Split Selected Artwork SQLite adapter concern:
  - `crates/taru-db/src/artwork/selected.rs` now owns Selected Artwork get/list
    SQL and publication/unpublication transactions;
  - `crates/taru-db/src/artwork.rs` keeps repository trait methods and routes
    through the selected module.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-db --tests`: passed.
  - `cargo nextest run -j 2 -p taru-db artwork --no-fail-fast`: passed; 11
    tests passed.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-050 final:

- Completed the SQLite Managed Artwork adapter concern split:
  - `crates/taru-db/src/artwork/candidate.rs` now owns Artwork Candidate
    repository methods, candidate lookup SQL, and status update helpers;
  - `crates/taru-db/src/artwork/ingest.rs` now owns Managed Artwork Ingest
    lookup/insert helpers and job transaction helpers used by ingest state
    transitions;
  - `crates/taru-db/src/artwork/artifact.rs` now owns Managed Artwork Artifact
    insert/get helpers and artifact SQL;
  - `crates/taru-db/src/artwork/lifecycle.rs` now also owns the unselected
    artifact cleanup transaction, keeping lifecycle diagnostics and cleanup in
    one concern;
  - `crates/taru-db/src/artwork.rs` keeps the existing repository trait methods
    and routes through concern modules.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-db --tests`: passed.
  - `cargo nextest run -j 2 -p taru-db artwork --no-fail-fast`: passed; 11
    tests passed.
  - `git diff --check`: passed with Git line-ending notices only.

2026-05-19, MAMD-060:

- Audited Managed Artwork API DTO locality:
  - Managed Artwork Admin DTOs, conversion helpers, and redaction tests were
    concentrated in `crates/taru-api/src/admin.rs` alongside unrelated Admin
    DTOs;
  - moved them into `crates/taru-api/src/admin/managed_artwork.rs`;
  - kept `admin.rs` re-exporting the module so `taru_api::*` and HTTP callers
    retain the same public names;
  - left `selected_artwork_to_public_image_ref` in `public_client.rs` because
    it is already the Public Client protocol DTO conversion boundary and is not
    over-concentrated.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-api --tests`: passed.
  - `cargo check -j 2 -p taru-server --tests`: passed.
  - `cargo nextest run -j 2 -p taru-api managed_artwork --no-fail-fast`:
    passed; 12 tests passed.
  - `cargo nextest run -j 2 -p taru-server managed_artwork --no-fail-fast`:
    passed; 13 tests passed.

2026-05-19, MAMD-070 closeout:

- Closed the lane after final verification.
- No residual follow-on was split from this architecture lane. Deferred product
  scopes remain explicitly out of scope: provider search, Public Client gallery
  browsing, persisted thumbnail eviction, repair/re-ingest, and new runtime
  retry/cancel semantics.
- Fresh validation:
  - `cargo fmt --all -- --check`: passed.
  - `cargo check -j 2 -p taru-server -p taru-api -p taru-db --tests`:
    passed.
  - `cargo nextest run -j 2 -p taru-api managed_artwork --no-fail-fast`:
    passed; 12 tests passed.
  - `cargo nextest run -j 2 -p taru-db artwork --no-fail-fast`: passed; 11
    tests passed.
  - `cargo nextest run -j 2 -p taru-server managed_artwork --no-fail-fast`:
    passed; 13 tests passed.
  - `git diff --check`: passed.
