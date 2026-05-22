# Managed Artwork Remediation Policy Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "storage-drift|remediation|remediate|UntrackedArtifactFile|MissingFile|SelectedArtwork|storage_uri|managed-artwork://" crates docs
git diff --check
```

This inventory anchors the drift diagnostics route, safe stray-file
classification, missing-file findings, Selected Artwork retention terms, and
redaction boundaries.

## Gate Set

### Remediation Gate

```powershell
cargo nextest run -p nako-api managed_artwork_remediation --no-fail-fast
cargo nextest run -p nako-server managed_artwork_remediation --no-fail-fast
cargo check -p nako-core -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

### Closeout Gate

```powershell
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root|remediation|remediate" crates/nako-api crates/nako-server/src/http docs/api
cargo nextest run -p nako-api managed_artwork_remediation --no-fail-fast
cargo nextest run -p nako-server managed_artwork_remediation --no-fail-fast
cargo check -p nako-core -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
```

Remaining hits must be explained as internal storage logic, route names,
documentation that states values are forbidden, or explicit redaction
assertions.

## Evidence Anchors

- `docs/workstreams/managed-artwork-artifact-store-drift-inventory/HANDOFF.md`
- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/app/artwork.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/query.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-19, MARP-010:

- Opened this lane from the storage drift inventory closeout follow-on.
- Scope decision:
  - first slice is dry-run remediation plan plus confirmed stray-file cleanup;
  - missing DB-backed artifacts are advisory only;
  - only parseable supported untracked artifact files are eligible for deletion;
  - repair/re-ingest, Selected Artwork management, thumbnails, runtime controls,
    and gallery management remain separate work.

2026-05-19, MARP-020/030:

- Added Admin remediation DTOs:
  - dry-run remediation plan;
  - missing-artifact advisory recommendations;
  - stray-file remediation actions;
  - confirmed cleanup report with deleted/missing/failed counters.
- Added Admin routes:
  - `GET /admin/v1/artwork/artifacts/remediation-plan`;
  - `POST /admin/v1/artwork/artifacts/remediate-stray-files?confirm=true`.
- Policy evidence:
  - missing DB-backed artifacts remain advisory only;
  - selected missing artifacts recommend restore or republish;
  - only parseable supported untracked artifact files are delete eligible;
  - cleanup requires `confirm=true`;
  - cleanup re-checks active DB artifact state before file deletion;
  - unexpected active-artifact paths, unsupported extensions, and unrecognized
    layouts are blocked/manual-inspect.
- Redaction evidence:
  - responses omit filenames, local paths, `storage_uri`,
    `managed-artwork://...`, raw source URLs, `source_uri`, `cache_uri`, Source
    Locators, addon tokens, provider query strings, file contents, and content
    hashes.
- Validation:
  - `cargo nextest run -p nako-api managed_artwork_remediation --no-fail-fast`
    passed.
  - `cargo nextest run -p nako-server managed_artwork_remediation --no-fail-fast`
    passed.
  - `cargo nextest run -p nako-api managed_artwork_storage_drift --no-fail-fast`
    passed as regression coverage.
  - `cargo nextest run -p nako-server managed_artwork_storage_drift --no-fail-fast`
    passed as regression coverage.
  - `cargo check -p nako-core -p nako-api -p nako-server --tests` passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
  - Closeout inventory grep found route names, API documentation, internal DTO
    fields, configuration diagnostics, and explicit redaction assertions only.
