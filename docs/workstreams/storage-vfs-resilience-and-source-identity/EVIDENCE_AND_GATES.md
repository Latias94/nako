# Storage/VFS Resilience And Source Identity — Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Required Gates

Run focused gates for each task before marking it complete:

- `python -m json.tool docs/workstreams/storage-vfs-resilience-and-source-identity/WORKSTREAM.json > $null`
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo check -p nako-core -p nako-db -p nako-vfs -p nako-library -p nako-server --tests`

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

## Review Expectations

- Review source identity confidence separately from duplicate detection.
- Review privacy of all diagnostics before accepting Admin/API changes.
- Review storage timeout/backoff changes for unrelated-library isolation.
- Coordinate before editing HLS-specific files because HLS is being developed
  by another agent.
