# Storage/VFS Resilience And Source Identity — Evidence And Gates

Status: Completed
Last updated: 2026-05-30

## Required Gates

Run focused gates for each task before marking it complete:

- `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null`
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-api -p nako-server --tests`

Task-specific gates:

- SVRS-020:
  - `cargo nextest run -p nako-library source_identity scan --no-fail-fast`
  - SQLite/PostgreSQL repository contract tests if persistence changes.
- SVRS-030:
  - `cargo nextest run -p nako-library rename_reconciliation --no-fail-fast`
  - `cargo nextest run -p nako-db scan source_duplicate --no-fail-fast`
- SVRS-040:
  - `cargo nextest run -p nako-vfs --no-fail-fast`
  - `cargo nextest run -p nako-server storage --no-fail-fast`
- SVRS-050:
  - `cargo nextest run -p nako-server system storage --no-fail-fast`
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast` if Admin DTOs change.
- SVRS-060:
  - `cargo check --workspace --tests`
  - `cargo fmt --all -- --check`
  - `git diff --check`
  - `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null`

Broaden to workspace gates when a task changes shared repository contracts,
public/admin DTOs, migrations, or runtime resource behavior:

- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`

If PostgreSQL schema or shared repository contracts change, run the opt-in
PostgreSQL harness when `NAKO_TEST_POSTGRES_URL` or the local harness is
available:

- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts`

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | SVRS-010 | Opened workstream docs and linked architecture index from the non-Web/HLS architecture review. | Verified during SVRS-020 closeout |
| 2026-05-29 | SVRS-020 | Added `SourceFingerprintEvidence`/`SourceFingerprintPolicyInput` in `nako-core` and routed `VfsLibraryScanner` through the layered policy. | Implemented |
| 2026-05-29 | SVRS-020 | `cargo nextest run -p nako-core source_fingerprint --no-fail-fast` | Passed: 4 tests |
| 2026-05-29 | SVRS-020 | `cargo nextest run -p nako-library source_identity scan --no-fail-fast` | Passed: 12 tests |
| 2026-05-29 | SVRS-020 | `cargo fmt --all -- --check`; `git diff --check`; `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-server --tests` | Passed |
| 2026-05-29 | SVRS-020 | SQLite/PostgreSQL repository contract tests | Not required: no schema, migration, or repository contract changed |
| 2026-05-29 | SVRS-030 | Added move/rename reconciliation in `nako-library` source observation commits and atomic duplicate-relationship persistence in SQLite/PostgreSQL scan commits. | Implemented |
| 2026-05-29 | SVRS-030 | `cargo nextest run -p nako-library source_identity rename_reconciliation --no-fail-fast` | Passed: 5 tests |
| 2026-05-29 | SVRS-030 | `cargo nextest run -p nako-db scan source_duplicate --no-fail-fast` | Passed: 7 tests |
| 2026-05-29 | SVRS-030 | `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-server --tests`; `cargo fmt --all -- --check`; `git diff --check`; `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null` | Passed |
| 2026-05-29 | SVRS-030 | PostgreSQL opt-in harness | Not run: `NAKO_TEST_POSTGRES_URL` was not set in this workspace |
| 2026-05-29 | SVRS-040 | Added `StorageFailureClass`, safe storage messages, WebDAV short-read validation, stale-cache safe failure persistence, library scan/probe safe failure messages, storage health backoff, and staging failure redaction. | Implemented |
| 2026-05-29 | SVRS-040 | `cargo nextest run -p nako-core storage_errors_expose_safe_failure_classification --no-fail-fast` | Passed: 1 test |
| 2026-05-29 | SVRS-040 | `cargo nextest run -p nako-vfs --no-fail-fast` | Passed: 48 tests |
| 2026-05-29 | SVRS-040 | `cargo nextest run -p nako-library webdav_scan_records_partial_directory_failures index_service_records_scan_failures_without_blocking_good_sources --no-fail-fast` | Passed: 2 tests |
| 2026-05-29 | SVRS-040 | `cargo nextest run -p nako-server storage --no-fail-fast` | Passed: 18 tests |
| 2026-05-29 | SVRS-040 | `cargo nextest run -p nako-server manifest_recording_backend_rolls_back_reservation_when_stage_fails --no-fail-fast` | Passed: 1 test |
| 2026-05-29 | SVRS-040 | `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-server --tests`; `cargo fmt --all -- --check`; `git diff --check`; `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null` | Passed; `git diff --check` printed only Windows line-ending warnings |
| 2026-05-29 | SVRS-040 | PostgreSQL opt-in harness | Not required: no schema, migration, or repository contract changed |
| 2026-05-30 | SVRS-050 | Added Admin overview catalog governance pressure summary, storage staging cleanup-candidate pressure, storage backend health failure-class/backoff diagnostics, and duplicate-only catalog governance inclusion for SQLite/PostgreSQL query paths. | Implemented |
| 2026-05-30 | SVRS-050 | Synchronized Admin TypeScript contract source and generated outputs under `apps/admin-web/src/adminApi/generated/contract.ts` and `web/src/api/admin/generated/contract.ts`. | Implemented; generated DTO contract only |
| 2026-05-30 | SVRS-050 | `cargo nextest run -p nako-api admin_contract --no-fail-fast` | Passed: 5 tests |
| 2026-05-30 | SVRS-050 | `cargo nextest run -p nako-api storage_backend_diagnostics --no-fail-fast` | Passed: 1 test |
| 2026-05-30 | SVRS-050 | `cargo nextest run -p nako-api admin_overview_response_serializes_safe_summary_fields --no-fail-fast` | Passed: 1 test |
| 2026-05-30 | SVRS-050 | `cargo nextest run -p nako-db catalog_governance --no-fail-fast` | Passed: 1 test |
| 2026-05-30 | SVRS-050 | `cargo nextest run -p nako-db runtime_promotion_contract_covers_facade_dispatch_gap_surfaces --no-fail-fast` | Passed: 1 SQLite contract test |
| 2026-05-30 | SVRS-050 | `cargo nextest run -p nako-server system storage --no-fail-fast` | Passed: 53 tests |
| 2026-05-30 | SVRS-050 | `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-api -p nako-server --tests`; `cargo fmt --all -- --check`; `git diff --check`; `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null` | Passed; `git diff --check` printed only Windows line-ending warnings |
| 2026-05-30 | SVRS-050 | PostgreSQL opt-in harness | Not run: no `NAKO_TEST_POSTGRES_URL` or local PostgreSQL harness was configured in this workspace |
| 2026-05-30 | SVRS-060 | Closed the lane, added `CLOSEOUT.md`, marked workstream docs completed, and split watcher/debounce, remote backend circuit breakers, VFS cache repair, hash escalation, and PostgreSQL runtime harness coverage as proposed follow-ons. | Implemented |
| 2026-05-30 | SVRS-060 | Updated `docs/architecture/STORAGE_VFS.md`, `docs/architecture/LIBRARY_PIPELINE.md`, `docs/architecture/WORKSTREAM_LINKS.md`, and `docs/workstreams/README.md` to mark the first slice as shipped and keep follow-ons explicit. | Implemented |
| 2026-05-30 | SVRS-060 | `cargo check --workspace --tests` | Passed |
| 2026-05-30 | SVRS-060 | `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null`; `git diff --check` | Passed; `git diff --check` printed only Windows line-ending warnings |
| 2026-05-30 | SVRS-060 | `cargo nextest run --workspace --no-fail-fast` | Not run: SVRS-060 is documentation closeout only, and SVRS-020 through SVRS-050 recorded focused behavior gates for the shipped code |

## SVRS-020 Verification Notes

- `cargo nextest run -p nako-core source_fingerprint --no-fail-fast` proves
  confidence classification, raw ETag redaction, content-hash strength,
  malformed hash downgrade, stale downgrade, and locator-only weak evidence.
- `cargo nextest run -p nako-library source_identity scan --no-fail-fast`
  proves scan-derived fingerprints do not require full-file reads, locator-only
  evidence produces no fingerprint, equal fingerprints do not merge distinct
  **Media Sources**, and existing scan/tombstone/stale-cache behavior still
  passes under the focused filter.
- `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-server --tests`
  proves the public `nako-core` type additions and `nako-library` scanner
  integration compile across the related crate boundary.
- Database contract tests were skipped because SVRS-020 did not change schema,
  migrations, repository traits, or repository implementations.

## SVRS-030 Verification Notes

- `cargo nextest run -p nako-library source_identity rename_reconciliation --no-fail-fast`
  proves strong content-hash moves preserve **Media Source** identity, curated
  item metadata, and non-provisional item state while tombstoning the old
  locator. The same gate also proves weak duplicate evidence and simultaneous
  strong duplicate files create suggested **Source Duplicate Relationship**
  records instead of merging sources.
- `cargo nextest run -p nako-db scan source_duplicate --no-fail-fast` proves
  scan source commits and source duplicate repository behavior still pass the
  focused SQLite contract and round-trip gates after duplicate relationships
  became part of the atomic scan source commit.
- `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-server --tests`
  proves the expanded scan commit shape and repository trait requirements
  compile across core, DB, VFS, library, and server crate boundaries.
- PostgreSQL transaction code was compile-checked through the Postgres adapter,
  but the opt-in runtime harness was skipped because no `NAKO_TEST_POSTGRES_URL`
  was configured in this workspace.

## SVRS-040 Verification Notes

- `StorageFailureClass` gives callers a small redaction-safe taxonomy for
  timeout, unavailable, permission, rate-limit, stale-cache, partial-read,
  budget, security, and unknown storage failures. Retryability is explicit:
  permission, security, and unknown classes do not enter backoff.
- `cargo nextest run -p nako-vfs --no-fail-fast` proves WebDAV range reads
  classify short bodies as partial reads and that stale-cache fallback still
  serves safe cached listings for transient storage failures.
- `cargo nextest run -p nako-library ...` proves scan failure persistence uses
  safe storage messages and still records partial WebDAV scan failures without
  blocking healthy sources.
- `cargo nextest run -p nako-server storage --no-fail-fast` proves library
  storage health backoff is process-local, library-scoped, redaction-safe, and
  does not suppress managed-import apply or cleanup compensation paths.
- `cargo nextest run -p nako-server manifest_recording_backend_rolls_back_reservation_when_stage_fails --no-fail-fast`
  proves staging reservation failure records no raw backend details.
- No PostgreSQL harness was required because SVRS-040 did not change schema,
  migrations, repository traits, or repository implementations.

## SVRS-050 Verification Notes

- Admin overview now reports catalog governance pressure as counts only:
  governed items, unknown-kind items, low-confidence items, duplicate
  relationship items, and items missing accepted provider mappings.
- Catalog governance SQL now includes duplicate-only items so source identity
  reconciliation pressure remains visible even when the item is otherwise
  high-confidence.
- Storage staging diagnostics now summarize cleanup-candidate record and byte
  pressure using the same repository cleanup candidate boundary as the cleanup
  job. The response still redacts staging paths and backend locators.
- Storage backend diagnostics expose only the typed last failure class and
  backoff timestamp, not raw backend messages, credentials, paths, ETags, or
  fingerprint values.
- Admin DTO changes were reflected in the hand-maintained TypeScript contract
  source and both generated Admin contract outputs. No Web UI behavior was
  changed in this lane.
- PostgreSQL catalog governance SQL was changed and compile-checked with the
  Postgres adapter, but the runtime PostgreSQL harness was skipped because no
  `NAKO_TEST_POSTGRES_URL` was available.

## SVRS-060 Closeout Verification Notes

- The lane is closed with `WORKSTREAM.json.status = "completed"` and no active
  current task.
- `CLOSEOUT.md` records the shipped behavior, residual risks, and explicit
  proposed follow-ons.
- Architecture maps now treat source identity resilience, storage failure
  classification, VFS cache/staging diagnostics, and mount-hang first-slice
  protection as shipped foundations rather than open work in this lane.
- The workspace compiled with `cargo check --workspace --tests` after closeout
  docs changed.
- Workspace nextest was intentionally skipped for SVRS-060 because no code
  changed in this closeout task; the behavior gates for shipped code remain
  recorded under SVRS-020 through SVRS-050.

## Review Expectations

- Review source identity confidence separately from duplicate detection.
- Review privacy of all diagnostics before accepting Admin/API changes.
- Review storage timeout/backoff changes for unrelated-library isolation.
- Coordinate before editing HLS-specific files because HLS is being developed
  by another agent.
