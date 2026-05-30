# Remote Storage Health And Circuit Breaker - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Required Gates

```text
python -m json.tool docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json
cargo nextest run -p nako-db storage_backend_health --no-fail-fast
cargo nextest run -p nako-server storage_health --no-fail-fast
cargo nextest run -p nako-server storage --no-fail-fast
cargo nextest run -p nako-server admin_v1_storage --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused package gates first. Broaden only when a task changes shared
storage, playback staging, Admin DTOs, schema migrations, or generated client
contracts.

## Evidence Ledger

### RSHC-010 - Scope and evidence freeze

Status: Done

Evidence:

- `docs/workstreams/remote-storage-health-and-circuit-breaker/DESIGN.md`
- `docs/workstreams/remote-storage-health-and-circuit-breaker/TODO.md`
- `docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Notes:

- The first executable task is repository parity, not server runtime policy.
- Playback staging and Admin reset work are deliberately sequenced after the
  durable health contract.

### RSHC-020 - Durable health contract

Status: Done

Evidence:

- `crates/nako-core/src/storage_health.rs`
- `crates/nako-core/src/repository/vfs.rs`
- `crates/nako-db/src/sqlite/vfs_health.rs`
- `crates/nako-db/src/postgres/vfs_health.rs`
- `crates/nako-db/src/contract_tests.rs`
- `crates/nako-db/migrations/baseline.sql`
- `crates/nako-db/migrations/postgres/baseline.sql`

Validated on 2026-05-30:

- `cargo nextest run -p nako-db storage_backend_health --no-fail-fast` -
  passed; ran
  `sqlite_storage_backend_health_contract_records_recovery_and_reset`.
- `cargo fmt --all -- --check` - passed.
- `git diff --check` - passed; Git reported only Windows line-ending
  normalization warnings.

Planner verification on 2026-05-30:

- `cargo nextest run -p nako-db storage_backend_health --no-fail-fast` -
  passed; 1 SQLite contract test run and 162 skipped by filter.
- `cargo fmt --all -- --check` - passed.
- `git diff --check` - passed with only Windows line-ending warnings.

Notes:

- The contract stores backend-scoped health status, circuit-breaker state,
  consecutive failure count, last redaction-safe failure class/message,
  backoff timestamp, and reset behavior.
- Task-level `review-workstream` check found no workstream compliance or code
  quality blockers before verification.
- SQLite and PostgreSQL adapters use the same repository trait and contract
  macro. The PostgreSQL contract case remains ignored by the existing harness
  unless `NAKO_TEST_POSTGRES_URL` is provided.
- No playback staging, cache repair, or Admin route behavior was changed.

### RSHC-030 - Runtime policy adapter

Status: Done

Evidence:

- `crates/nako-server/src/app/storage.rs`
- `crates/nako-server/src/app/tests/storage.rs`

Validated on 2026-05-30:

- `cargo nextest run -p nako-server storage_health --no-fail-fast` -
  passed; ran
  `app::tests::storage::storage_health_records_runtime_updates_and_rejects_durable_circuit`.
- `cargo nextest run -p nako-server storage --no-fail-fast` - passed;
  19 storage-filtered server tests run.
- `cargo fmt --all -- --check` - passed.
- `git diff --check` - passed; Git reported only Windows line-ending
  normalization warnings.

Notes:

- `LibraryStorageBackend` now derives a stable backend key, persists
  redaction-safe health updates through the durable repository, and checks
  durable open circuit state before bounded storage reads, range work, listing,
  string reads, streaming, staging, writes, link planning, storage apply,
  cleanup, and restore.
- Runtime tests prove a retryable timeout records `Unavailable`/`Open`,
  a fresh backend instance is rejected without touching the wrapped backend for
  read and mutation-style runtime work, and a later successful operation
  records `Healthy`/`Closed`.
- Reviewer follow-up on 2026-05-30 found that `write_string`, `write`,
  `plan_link`, `apply`, `cleanup`, and `restore` originally recorded outcomes
  but skipped circuit admission. The runtime test now covers those paths and
  the adapter rejects them before invoking the wrapped backend while the
  durable circuit is open.
- Task-level `review-workstream` check found no workstream compliance or code
  quality blockers before verification.
- No playback staging, cache repair, Admin route, or schema behavior was
  changed.

Planner verification on 2026-05-30:

- `python -m json.tool docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json`
  passed.
- `cargo nextest run -p nako-server storage_health --no-fail-fast` passed with
  1 test run and 490 skipped by filter.
- `cargo nextest run -p nako-server storage --no-fail-fast` passed with 19
  tests run and 472 skipped by filter.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Windows line-ending normalization
  warnings.

## Residual Risks

- Mount-like local paths can still hang below the OS boundary. Circuit-breaker
  state should reduce repeated work admission, not claim to preempt every
  blocking syscall.
- Backend-scoped health may be too coarse for rare source-specific corruption.
  Split a follow-on only after evidence proves source-scoped suppression is
  needed.
- Admin reset can hide an active incident if it is not paired with clear
  diagnostics and updated timestamps.
