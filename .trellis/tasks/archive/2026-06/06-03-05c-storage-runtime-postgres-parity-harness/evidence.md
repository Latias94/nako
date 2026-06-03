# Storage Runtime PostgreSQL Parity Harness Evidence

Date: 2026-06-03
Selected slice: focused PostgreSQL parity harness for storage admission and staging behavior.

## Selection

Chose the smallest reusable PostgreSQL harness slice already modeled by the
repository contract boundary:

- `postgres_storage_backend_health_contract_*` covers durable storage admission
  facts used by storage circuit / backend health checks.
- `postgres_vfs_staging_contract_*` covers staging reservation budget, lease,
  cleanup-candidate, and cache-summary behavior used by staging pressure and
  staging lifecycle admission.

This keeps the work inside `nako-db` contract parity and existing harness
scripts instead of widening into runtime policy, scheduler fairness, server
rewrite, schema migration, or broader control-plane PostgreSQL rollout.

## Shipped Behavior

- Added a focused `storage-runtime` suite to both
  `scripts/postgres-contract-harness.ps1` and
  `scripts/postgres-contract-harness.sh`.
- The suite runs only the PostgreSQL ignored contracts selected for this task:
  - `postgres_storage_backend_health_contract_records_recovery_and_reset`
  - `postgres_vfs_staging_contract_preserves_reservation_budget_and_leases`
  - `postgres_vfs_staging_contract_round_trips_listing_failures_and_summary`
- Fixed the local temporary PostgreSQL startup path in the harness scripts by
  removing the explicit `-k <target/postgres-contract>` socket directory from
  `pg_ctl -o ...`.
- On this Windows worktree, the previous harness shape failed because the
  generated Unix-domain socket path under the long worktree directory exceeded
  PostgreSQL's 107-byte limit. The fix changes only local harness startup
  compatibility; it does not change storage runtime policy, durable job policy,
  scheduler fairness, schema, or API behavior.

## Boundaries Preserved

- No runtime policy change.
- No scheduler fairness change.
- No schema or migration change.
- No `nako-db` repository contract semantics change.
- No `nako-server` admission behavior change.
- No Admin/Web, API, or SDK change.

## Validation

The final commit candidate was also applied to an isolated detached verification
worktree at
`F:\SourceCodes\Rust\nako-worktrees\_verify-05c-storage-runtime-postgres-parity-harness`
so unrelated dirty files in the current lane worktree did not affect the final
gate results.

- `cargo nextest run -p nako-db storage_backend_health_contract vfs_staging_contract --no-fail-fast`
  passed: 3 SQLite-focused storage/staging contract tests.
- `pwsh -File scripts/postgres-contract-harness.ps1 -Suite storage-runtime -RequireTooling`
  passed: 3 PostgreSQL ignored contract tests.
- `cargo nextest run -p nako-server scan_library_rejects_open_storage_circuit_before_pipeline scan_library_rejects_critical_staging_pressure_before_pipeline scan_library_allows_local_library_during_remote_staging_pressure --no-fail-fast`
  passed: 3 server-focused storage admission / staging pressure tests.
- `cargo nextest run -p nako-db job_lease_contract --no-fail-fast`
  passed: 4 SQLite lease contract tests. This was an audit-only confirmation
  that job lease parity already exists and remains outside the selected
  `storage-runtime` suite boundary.
- `cargo check -p nako-db -p nako-server --tests` passed.
- `bash -n scripts/postgres-contract-harness.sh` passed.
- `bash scripts/postgres-contract-harness.sh --suite storage-runtime --require-tooling`
  did not run end-to-end in this Windows shell environment because `bash` did
  not see `initdb`, `pg_ctl`, or `createdb` on `PATH`. This did not block the
  task because the PowerShell harness ran the same focused PostgreSQL suite
  successfully in the same worktree.
- `cargo fmt --all -- --check` passed in the isolated commit-candidate
  verification worktree.
- `git diff --check` passed with LF/CRLF normalization warnings only.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-03-05c-storage-runtime-postgres-parity-harness`
  passed after task artifacts were updated.

## Follow-ons

- Broader PostgreSQL runtime harness coverage for other storage/control-plane
  paths remains separate.
- If CI should enforce this slice explicitly, wiring `storage-runtime` into a
  release or focused parity gate can be a follow-on.
- Cross-shell PostgreSQL tooling PATH parity for `bash` on Windows remains an
  environment/setup follow-on, not a storage runtime policy task.
