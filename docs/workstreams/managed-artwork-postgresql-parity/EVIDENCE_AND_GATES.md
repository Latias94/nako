# Managed Artwork PostgreSQL Parity — Evidence And Gates

Status: Completed
Last updated: 2026-05-20

## Baseline Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-db --tests
cargo check -p nako-server --tests
cargo nextest run -p nako-db artwork --no-fail-fast
cargo nextest run -p nako-server artwork --no-fail-fast
git diff --check
```

When PostgreSQL contracts are added:

```bash
NAKO_TEST_POSTGRES_URL=<url> cargo nextest run -p nako-db postgres_managed_artwork_contract --run-ignored ignored-only --no-fail-fast
```

## Redaction Inventory Gate

```bash
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|artifact_root|local_path|selected_artwork|managed_artwork" crates/nako-api crates/nako-server/src/http docs/api
```

The inventory must prove that public/Admin DTOs remain redacted before runtime
support is claimed.

## PGR-090 Split Evidence

M62 split Managed Artwork parity because the subsystem spans Addon Artwork
Candidates, Managed Artwork Ingest, artifacts, Selected Artwork, galleries,
lifecycle cleanup, drift diagnostics, remediation, thumbnails, artifact-store
files, and redaction-sensitive public/Admin serving. Partial PostgreSQL support
would be worse than an explicit unsupported boundary.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-20 | MAPG-010 inventory | `rg -n "ManagedArtwork\|ArtworkCandidate\|selected_artwork\|managed_artwork\|addon_artwork" crates/nako-core crates/nako-db crates/nako-server docs/workstreams` | Pass. Inventory confirms SQLite has `ArtworkCandidateRepository` and `ManagedArtworkRepository` behavior, PostgreSQL currently has stub implementations returning unsupported errors, and server/runtime DTOs are already redaction-shaped. |
| 2026-05-20 | MAPG-010 inventory | `Get-ChildItem -Recurse crates/nako-db \| Where-Object { $_.Name -match 'artwork\|managed\|selected\|postgres\|sqlite\|migration' }` | Pass. Inventory identified SQLite migrations `0025`-`0028`, SQLite modules under `src/sqlite/artwork/`, and PostgreSQL migration surface `migrations/postgres/0001_contract_jobs.sql` as the parity target. |
| 2026-05-20 | MAPG-020/030/040 SQLite contracts | `cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast` | Pass. 6/6 SQLite backend-neutral Managed Artwork contracts passed: Addon Artwork Candidate intake, Artwork Task queue, acceptance/ingest job creation, ingest processing, failure/recovery/requeue, and selected/gallery/lifecycle. |
| 2026-05-20 | MAPG-020/030/040 PostgreSQL contracts | Ephemeral local PostgreSQL 17 cluster under `target/nako-pg-contract`, `NAKO_TEST_POSTGRES_URL=postgres://postgres@127.0.0.1:<ephemeral>/nako_contract`, `cargo nextest run -p nako-db postgres_managed_artwork_contract --run-ignored ignored-only --no-fail-fast` | Pass. 6/6 ignored PostgreSQL Managed Artwork contracts passed with schema-isolated contract runs. Temporary cluster was stopped and its data directory was removed after the command. |
| 2026-05-20 | MAPG-020/030/040 DB package | `cargo check -p nako-db --tests` | Pass. PostgreSQL migration wiring, repository implementations, and contract tests type-check. |
| 2026-05-20 | MAPG-050 API/server compile | `cargo check -p nako-api --tests`; `cargo check -p nako-server --tests` | Pass. Runtime/API surfaces compile after enabling PostgreSQL Managed Artwork capability and capability-driven worker gating. |
| 2026-05-20 | MAPG-050 API redaction | `cargo nextest run -p nako-api managed_artwork --no-fail-fast` | Pass. 12/12 Managed Artwork API/OpenAPI/SDK tests passed, including redaction for storage URI, `managed-artwork://...`, raw source URLs, cache URI, content hash values, artifact root, and local paths. |
| 2026-05-20 | MAPG-050 server runtime/redaction | `cargo nextest run -p nako-server managed_artwork --no-fail-fast` | Pass. 13/13 server Managed Artwork tests passed, including worker processing, publish/gallery/lifecycle/cleanup/drift/remediation, selected image variants, and locator/hash redaction. |
| 2026-05-20 | MAPG closeout DB focus | `cargo nextest run -p nako-db managed_artwork --no-fail-fast` | Pass. 12/12 DB Managed Artwork focused tests passed, including SQLite contracts, SQLite lifecycle tests, and the PostgreSQL runtime capability test. |
| 2026-05-20 | MAPG redaction inventory | `rg -n "storage_uri\|managed-artwork://\|source_uri\|cache_uri\|content_hash\|artifact_root\|local_path\|selected_artwork\|managed_artwork" crates/nako-api crates/nako-server/src/http docs/api` | Pass. Inventory remains concentrated in docs, tests, internal records, explicit DTO booleans such as `has_content_hash`, and redaction assertions; API/server tests above prove no raw locator/hash field leaks. |
| 2026-05-20 | MAPG final format/diff | `cargo fmt --all -- --check`; `git diff --check` | Pass. Formatting and whitespace checks passed after closeout doc updates. |

## Closeout Review Notes

- Workstream compliance: target state is met. PostgreSQL now has schema and
  repository parity for existing Managed Artwork state, and the server reports
  `managed_artwork: true` for PostgreSQL capability.
- Code quality: implementation keeps binary artifact bytes outside PostgreSQL
  and stores only coordination records/opaque storage handles. Claim, commit,
  fail, recovery, requeue, publish, and cleanup operations remain transactional.
- Missing gates: no blocking gate remains. Full workspace nextest was not run
  because this lane touched focused DB/API/server Managed Artwork surfaces; the
  focused DB/API/server gates above prove the closeout claim.
- Residual risk: repeatable PostgreSQL CI orchestration is still external to
  this repo-local closeout and should be opened as a separate CI/ops lane if
  prioritized.

## MAPG-010 Inventory Notes

- SQLite Managed Artwork tables:
  `addon_artwork_candidates`, `managed_artwork_ingests`,
  `managed_artwork_artifacts`, `selected_artworks`, and soft-delete
  `managed_artwork_artifacts.deleted_at`.
- SQLite repository modules:
  `crates/nako-db/src/sqlite/artwork/{candidate,ingest,artifact,selected,gallery,lifecycle}.rs`.
- Original PostgreSQL gap:
  MAPG-010 found `crates/nako-db/src/postgres.rs` implemented
  `ArtworkCandidateRepository` and `ManagedArtworkRepository` only as explicit
  unsupported stubs. Closeout replaced those stubs with PostgreSQL-backed
  parity implementations.
- Runtime boundary:
  `ManagedArtworkAppService` and Addon artwork_write call `NakoDatabase`
  repository traits; artifact bytes stay in `LocalManagedArtworkArtifactStore`;
  selected-image variants are derived from artifact bytes at read time.
- Redaction boundary:
  `crates/nako-api/src/admin/managed_artwork.rs` exposes IDs, status,
  dimensions, media type, booleans, and public image refs but not raw
  `source_uri`, `storage_uri`, `managed-artwork://...`, local paths, cache URIs,
  or content hashes.
