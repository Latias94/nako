# PostgreSQL Source Identity Contract Harness Evidence

Date: 2026-06-06

## Shipped Behavior

- Added a focused `source-identity` suite to both PostgreSQL contract harness
  scripts:
  - `scripts/postgres-contract-harness.ps1`
  - `scripts/postgres-contract-harness.sh`
- The suite runs explicit ignored PostgreSQL contract filters for:
  - library-scoped Media Source identity preservation;
  - scan commit source-unit writes;
  - source duplicate relationship upsert and fingerprint match lookup;
  - VFS staging attribution variants;
  - VFS staging reservation budget and lease preservation.
- Updated durable release-readiness docs to enumerate the current focused
  harness suites.
- Updated `nako-db` quality guidelines with the executable suite-selection
  contract for future PostgreSQL parity work.

## Boundaries Preserved

- No database schema or migration changes.
- No repository trait or adapter behavior changes.
- No API, server runtime, or generated client contract changes.
- Existing harness behavior for caller-provided database URLs, temporary local
  clusters, safe skip, required tooling, keep-data, and cleanup is unchanged.

## Validation

- `cargo nextest run -p nako-db preserves_library_scoped_source_identity writes_full_source_unit_and_resolves_failure source_duplicate_contract round_trips_attribution_variants preserves_reservation_budget_and_leases --no-fail-fast`
  - Passed: 7 tests.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite source-identity -RequireTooling`
  - Passed: 6 ignored PostgreSQL contract tests.
  - The temporary cluster under `target/postgres-contract/` was stopped and
    cleaned after the run.
- `bash scripts/postgres-contract-harness.sh --suite source-identity`
  - Accepted the suite and safely skipped because this Windows Bash environment
    did not have `initdb`, `pg_ctl`, or `createdb` on its Bash `PATH`.
- PowerShell parser check for `scripts/postgres-contract-harness.ps1`
  - Passed.
- `bash -n scripts/postgres-contract-harness.sh`
  - Passed.
- `cargo check -p nako-db --tests`
  - Passed.
- `git diff --check`
  - Passed with only existing LF/CRLF normalization warnings.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-06-06-postgres-source-identity-contract-harness`
  - Passed.

## Follow-ons

- If CI should enforce this slice explicitly, wire `source-identity` into a
  focused release or PostgreSQL parity job.
