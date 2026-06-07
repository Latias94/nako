# Storage VFS PostgreSQL Job Runtime Harness Evidence

## Result

Passed.

This task adds a focused `job-runtime` suite to the PostgreSQL contract harness
without changing database schema, repository traits, SQL behavior, runtime
behavior, API shape, or product behavior.

## Shipped Behavior

- Added `job-runtime` to `scripts/postgres-contract-harness.ps1`.
- Added `job-runtime` to `scripts/postgres-contract-harness.sh`.
- The suite maps to existing PostgreSQL contract filters for:
  - job lease claim/filter behavior;
  - heartbeat/completion run-token fencing;
  - cancellation acknowledgement;
  - expired lease recovery;
  - retry backoff and redacted queue pressure;
  - priority ordering and retry preservation.
- Preserved `storage-source-parity` as storage/source-specific; it still runs
  only `storage-runtime` and `source-identity` filters.
- Preserved `all-contracts` as the only broad `postgres_` filter suite.
- Hardened temporary PostgreSQL cleanup:
  - stop timeout increased from 30s to 90s;
  - harness data is not recursively deleted if the local server does not
    confirm shutdown.

## Initial Failure And Fix

The first PowerShell `job-runtime` run proved all six PostgreSQL contracts, but
the harness returned failure during cleanup. PostgreSQL needed longer than the
old 30s stop timeout on Windows; after the stop timeout elapsed, the script
continued into recursive cleanup while the data directory was still locked.

The fix keeps contract behavior unchanged and only makes harness cleanup safer:
the scripts now wait up to 90 seconds for stop confirmation and preserve
`target/postgres-contract/` instead of deleting it when shutdown is not
confirmed.

The stale generated `target/postgres-contract/` directory from the failed run
was inspected: port `55435` was no longer listening and `pg_ctl status` reported
that the directory was not a valid database cluster. It was then removed only
after verifying the resolved path was under `target/`.

## Validation

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-07-storage-vfs-postgres-job-runtime-harness`
  - Passed: `implement.jsonl` 7 entries, `check.jsonl` 7 entries.
- PowerShell parser check for `scripts/postgres-contract-harness.ps1`
  - Passed.
- `bash -n scripts/postgres-contract-harness.sh`
  - Passed. This Windows environment printed WSL networking warnings, but the
    shell syntax check returned success.
- `cargo nextest run -p nako-db job_lease_contract job_retry_contract --no-fail-fast`
  - Passed: 6 SQLite-focused job lease/retry contract tests.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite job-runtime -Port 55436`
  - Passed: local PostgreSQL 17 tooling started a temporary cluster, ran 6/6
    ignored PostgreSQL job lease/retry contracts, stopped PostgreSQL cleanly
    within the 90s window, and removed `target/postgres-contract/`.

## Boundaries Preserved

- No schema or migration change.
- No repository trait change.
- No SQL semantics change.
- No server runtime or scheduler behavior change.
- No Admin/Public API, SDK, or generated contract change.
- No product behavior change.

## Spec Update

Updated `.trellis/spec/nako-db/backend/quality-guidelines.md` so future agents
know `job-runtime` is a valid focused PostgreSQL harness suite and should be
used for durable job runtime parity instead of defaulting to `all-contracts`.
