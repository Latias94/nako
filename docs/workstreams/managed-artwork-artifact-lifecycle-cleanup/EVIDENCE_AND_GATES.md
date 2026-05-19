# Managed Artwork Artifact Lifecycle Cleanup Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "managed_artwork_artifacts|selected_artworks|storage_uri|delete_best_effort|read_selected_image|ManagedArtworkArtifact|SelectedArtwork|cleanup|orphan|/admin/v1/artwork" crates docs
git diff --check
```

This inventory anchors the artifact table, Selected Artwork references,
storage redaction terms, existing local artifact-store behavior, and Admin
artwork routes.

## Gate Set

### Dry-Run Diagnostics Gate

```powershell
cargo nextest run -p taru-api managed_artwork_lifecycle --no-fail-fast
cargo nextest run -p taru-db managed_artwork_lifecycle --no-fail-fast
cargo nextest run -p taru-server managed_artwork_lifecycle --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

### Protected Cleanup Gate

```powershell
cargo nextest run -p taru-api managed_artwork_cleanup --no-fail-fast
cargo nextest run -p taru-db sqlite_store_cleanup_marks_only_unselected_managed_artwork_artifacts_deleted --no-fail-fast
cargo nextest run -p taru-server managed_artwork_cleanup --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

### Closeout Gate

```powershell
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root" crates/taru-api crates/taru-server/src/http docs/api
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo nextest run -p taru-db artwork --no-fail-fast
cargo nextest run -p taru-server artwork --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Remaining hits must be explained as internal storage logic, explicit redaction
assertions, or documentation that states values are forbidden in responses.

## Evidence Anchors

- `docs/workstreams/managed-artwork-public-serving-selection/HANDOFF.md`
- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-core/src/repository/metadata.rs`
- `crates/taru-db/migrations/0026_managed_artwork_ingest.sql`
- `crates/taru-db/migrations/0027_selected_artwork_publication.sql`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-server/src/http/admin.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-19, MAALC-010:

- Opened this lane from the MAPS closeout follow-on list.
- Confirmed existing protection:
  - `selected_artworks.artifact_id` has `ON DELETE RESTRICT`.
  - `selected_artworks.id` is the public image authority.
  - Existing Admin/Public artwork DTOs redact storage handles and paths.
- Scope decision:
  - first slice is redacted Admin lifecycle diagnostics and cleanup dry-run;
  - no real deletion in the first slice;
  - thumbnail variants, durable retry/requeue/cancellation, and gallery
    management remain separate lanes.

2026-05-19, MAALC-020:

- Added core lifecycle records and repository filter:
  - `ManagedArtworkArtifactLifecycleRecord`;
  - `ManagedArtworkArtifactLifecycleSummary`;
  - `ManagedArtworkArtifactLifecycleSnapshot`;
  - `ManagedArtworkArtifactLifecycleFilter`.
- Added SQLite lifecycle inventory:
  - joins `managed_artwork_artifacts` to `selected_artworks`;
  - `selected_artwork_count > 0` marks protected artifacts;
  - `selected_artwork_count == 0` marks cleanup candidates;
  - summary tracks total/protected/candidate counts and known byte estimates.
- Added Admin dry-run route:
  - `GET /admin/v1/artwork/artifacts/lifecycle`;
  - optional `cleanup_candidates_only=true`;
  - paginated rows and summary;
  - no deletion behavior.
- Redaction evidence:
  - API DTO exposes `has_content_hash` only, not content hash values;
  - route responses omit `storage_uri`, `managed-artwork://...`, local paths,
    raw source URLs, `source_uri`, `cache_uri`, addon tokens, provider query
    strings, and content hash values.
- Validation:
  - `cargo nextest run -p taru-api managed_artwork_lifecycle --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db managed_artwork_lifecycle --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server managed_artwork_lifecycle --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.

2026-05-19, MAALC-030:

- Added cleanup state:
  - migration `0028_managed_artwork_artifact_cleanup.sql` adds
    `managed_artwork_artifacts.deleted_at`;
  - active artifact lookups and lifecycle diagnostics ignore rows where
    `deleted_at IS NOT NULL`.
- Added protected repository cleanup:
  - cleanup selects paginated candidate artifacts;
  - each artifact is marked deleted only when `deleted_at IS NULL` and no
    `selected_artworks` row references it;
  - `selected_artworks.artifact_id ON DELETE RESTRICT` remains the hard
    retention guard.
- Added Admin cleanup route:
  - `POST /admin/v1/artwork/artifacts/cleanup`;
  - returns examined/candidate counts, redacted cleaned artifact facts, and
    redacted file cleanup counters;
  - never returns `storage_uri`, `managed-artwork://...`, paths, source URLs,
    `source_uri`, `cache_uri`, addon token material, query strings, or content
    hashes.
- Added local file cleanup:
  - only resolves Taru-owned `managed-artwork://artifact/{id}` handles;
  - best-effort removes local files after repository cleanup;
  - reports deleted, missing, and failed counts without exposing paths.
- Validation:
  - `cargo nextest run -p taru-api managed_artwork_cleanup --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db sqlite_store_cleanup_marks_only_unselected_managed_artwork_artifacts_deleted --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server managed_artwork_cleanup --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-api managed_artwork_lifecycle --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db managed_artwork_lifecycle --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server managed_artwork_lifecycle --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
