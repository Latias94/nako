# Managed Artwork Artifact Store Drift Inventory Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "managed_artwork_artifacts|storage-drift|artifact_root|managed-artwork://artifact|storage_uri|content_hash|LocalManagedArtworkArtifactStore" crates docs
git diff --check
```

This inventory anchors the artifact table, local artifact-store behavior,
internal storage authority, and redaction terms.

## Gate Set

### Diagnostics Gate

```powershell
cargo nextest run -p nako-api managed_artwork_storage_drift --no-fail-fast
cargo nextest run -p nako-server managed_artwork_storage_drift --no-fail-fast
cargo check -p nako-core -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

### Closeout Gate

```powershell
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root|storage-drift" crates/nako-api crates/nako-server/src/http docs/api
cargo nextest run -p nako-api managed_artwork_storage_drift --no-fail-fast
cargo nextest run -p nako-server managed_artwork_storage_drift --no-fail-fast
cargo check -p nako-core -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

Remaining hits must be explained as internal storage logic, explicit redaction
assertions, or documentation that states values are forbidden in responses.

## Evidence Anchors

- `docs/workstreams/managed-artwork-artifact-lifecycle-cleanup/HANDOFF.md`
- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/app/artwork.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-19, MASDI-010:

- Split artifact-store drift inventory from lifecycle cleanup.
- Scope decision:
  - this lane is read-only diagnostics first;
  - DB-backed active missing files and artifact-root stray files are in scope;
  - deletion, repair, re-ingest, thumbnails, runtime controls, and gallery
    management remain separate work.

2026-05-19, MASDI-020/030:

- Added Admin DTOs for redacted storage drift diagnostics:
  - `AdminManagedArtworkArtifactStorageDriftResponse`;
  - missing DB-backed artifact summaries;
  - stray file classifications;
  - bounded scan summary and truncation flag.
- Added Admin route:
  - `GET /admin/v1/artwork/artifacts/storage-drift`;
  - `limit`/`offset` page active DB-backed artifact checks;
  - `file_scan_limit` bounds artifact-root inventory.
- Added local artifact-store inventory:
  - checks DB-backed artifact expected files through the same
    `managed-artwork://artifact/{id}` and media-type path rules used for image
    serving;
  - enumerates files under the artifact root without returning filenames or
    paths;
  - classifies untracked artifact files, unexpected active artifact paths,
    unsupported extensions, and unrecognized layout.
- Redaction evidence:
  - response omits `storage_uri`, `managed-artwork://...`, local paths,
    filenames, raw source URLs, `source_uri`, `cache_uri`, Source Locators,
    addon tokens, provider query strings, file contents, and content hashes.
- Validation:
  - `cargo nextest run -p nako-api managed_artwork_storage_drift --no-fail-fast`
    passed.
  - `cargo nextest run -p nako-server managed_artwork_storage_drift --no-fail-fast`
    passed.
  - `cargo check -p nako-core -p nako-api -p nako-server --tests` passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
  - Closeout inventory grep found only API documentation, internal DTO fields,
    route names, configuration diagnostics, and explicit redaction assertions.
