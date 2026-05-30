# Remote Storage Health And Circuit Breaker - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Required Gates

```text
python -m json.tool docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json
cargo nextest run -p nako-db storage_backend_health --no-fail-fast
cargo nextest run -p nako-server storage_health --no-fail-fast
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

## Residual Risks

- Mount-like local paths can still hang below the OS boundary. Circuit-breaker
  state should reduce repeated work admission, not claim to preempt every
  blocking syscall.
- Backend-scoped health may be too coarse for rare source-specific corruption.
  Split a follow-on only after evidence proves source-scoped suppression is
  needed.
- Admin reset can hide an active incident if it is not paired with clear
  diagnostics and updated timestamps.
