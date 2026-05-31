# Remote Storage Health And Circuit Breaker - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

The lane is closed and linked from the storage/VFS architecture indexes.
`RSHC-010` through `RSHC-050` are complete.

`RSHC-020` added the durable **Storage Backend Health** repository contract,
domain records, SQLite and PostgreSQL adapters, facade dispatch, baseline
schema, and repository contract coverage. The contract records backend-scoped
health status, **Storage Circuit Breaker** state, consecutive failures,
redaction-safe last failure class/message, backoff timestamp, recovery state,
and reset behavior.

`RSHC-030` wired the server storage runtime adapter to the durable health
contract. `LibraryStorageBackend` now records durable health updates for
storage operation outcomes and consults durable open circuit state before
bounded storage work, including read, stream, staging, write, link-plan, apply,
cleanup, and restore paths. The runtime test covers retryable timeout
recording, cross-instance circuit rejection across read and mutation-style
runtime work, and successful recovery to `Healthy`/`Closed`.

`RSHC-040` added Admin operator diagnostics and reset for durable
**Storage Backend Health**. Admin HTTP now uses `NakoApp::storage()` and
`StorageDiagnosticsAppService` for paginated, redaction-safe durable health
listing and reset. The reset route clears **Storage Circuit Breaker** state
through the durable repository contract. Generated Admin TypeScript contracts
were refreshed because `nako-api` enforces generator-output parity for Admin
DTO and route changes.

## Next Task

No task remains inside this workstream.

Follow-ons should be opened as separate workstreams if prioritized:

- VFS cache repair diagnostics and operator remediation;
- source fingerprint partial/full hash escalation policy;
- playback artifact I/O pressure scheduling;
- scan scheduling and watcher/debounce hardening;
- PostgreSQL storage/VFS runtime harness evidence.

## Stop Conditions

Return to planner coordination if:

- the task needs schema migration policy outside storage health;
- playback staging, scan scheduling, or direct database exposure from Admin HTTP
  becomes necessary;
- the health model requires a durable ADR instead of fitting ADR 0016/0017;
- existing user changes appear in files you need to edit.

## RSHC-020 Validation

```text
cargo nextest run -p nako-db storage_backend_health --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

All required gates passed on 2026-05-30. `git diff --check` reported only
Windows line-ending normalization warnings.

## RSHC-030 Validation

```text
cargo nextest run -p nako-server storage_health --no-fail-fast
cargo nextest run -p nako-server storage --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

All required gates passed on 2026-05-30. `git diff --check` reported only
Windows line-ending normalization warnings.

No playback staging, scan scheduling, durable jobs, cache repair, Admin routes,
or schema behavior was changed.

Reviewer follow-up on 2026-05-30 found the initial RSHC-030 adapter only gated
read-like paths. The follow-up fix added durable circuit admission to
`write_string`, `write`, `plan_link`, `apply`, `cleanup`, and `restore`, and
expanded the storage health regression test to prove those calls are rejected
before reaching the wrapped backend while the circuit is open.

## RSHC-040 Validation

```text
cargo nextest run -p nako-server admin_v1_storage --no-fail-fast
cargo nextest run -p nako-server storage_health --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

All required gates passed on 2026-05-31. `git diff --check` reported only
Windows line-ending normalization warnings.

No playback staging, cache repair, hash escalation, scan scheduling, durable
jobs, schema migrations, or raw database exposure was changed.

## Closeout Validation

Final gates passed on 2026-05-31 and are recorded in
`EVIDENCE_AND_GATES.md` and `CLOSEOUT.md`.
