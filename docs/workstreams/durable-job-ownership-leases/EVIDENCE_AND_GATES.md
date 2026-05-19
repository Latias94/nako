# Durable Job Ownership Leases - Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
Get-Content docs\workstreams\durable-job-ownership-leases\WORKSTREAM.json | ConvertFrom-Json
git diff --check
```

This proves the new lane metadata is parseable and the docs diff has no
whitespace errors.

## Gate Set

### Inventory Gate

```powershell
rg -n "JobStatus|JobRepository|start_job|succeed_job|fail_job|fail_unfinished_jobs|spawn_job|run_job|ManagedArtworkIngest|cancel|lease|heartbeat" crates/taru-core/src crates/taru-db/src crates/taru-server/src/app docs/adr docs/workstreams/durable-job-ownership-leases docs/workstreams/job-runtime-worker-control-plane
```

Use this to verify the design still matches the real code surfaces.

### Core Contract Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo check -p taru-core --tests
cargo fmt --all -- --check
```

Use after `DJOL-020` changes core job types or repository traits.

### SQLite Repository Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-db job_lease --no-fail-fast
cargo nextest run -p taru-db job_cancel --no-fail-fast
cargo nextest run -p taru-db running_jobs_failed_on_startup --no-fail-fast
```

Use after `DJOL-030` adds schema and repository operations.

### Runtime Integration Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_runtime --no-fail-fast
cargo nextest run -p taru-server startup --no-fail-fast
```

Use after `DJOL-040` wires a real runtime path to leases.

### API Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_cancel --no-fail-fast
cargo check -p taru-api -p taru-server --tests
```

Use only if `DJOL-050` exposes Admin cancel-request controls.

### Closeout Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

Broaden to package or workspace `nextest` only when the touched surface
requires it.

## Evidence Log

| Date | Task | Gate | Result | Notes |
| --- | --- | --- | --- | --- |
| 2026-05-19 | DJOL-010 | WORKSTREAM JSON parse, `git diff --check` | Pass | Opening docs only; JSON parsed and diff check passed before commit. |
| 2026-05-19 | DJOL-020 | `cargo check -p taru-core --tests`, `cargo check -p taru-db -p taru-api -p taru-server --tests`, `cargo fmt --all -- --check`, WORKSTREAM JSON parse, `git diff --check` | Pass | Core contract and ADR updated; no schema or API behavior change. |
| 2026-05-19 | DJOL-030 | `cargo nextest run -p taru-db job_lease --no-fail-fast`, `cargo nextest run -p taru-db job_cancel --no-fail-fast`, `cargo nextest run -p taru-db running_jobs_failed_on_startup --no-fail-fast`, `cargo nextest run -p taru-server recovers_unfinished_jobs --no-fail-fast`, `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`, `cargo fmt --all -- --check`, WORKSTREAM JSON parse, `git diff --check` | Pass | SQLite schema/repository proof complete; queued jobs are preserved by generic startup recovery, and recovered running jobs clear lease ownership fields. |
| 2026-05-19 | DJOL-040 | `cargo nextest run -p taru-server job_runtime --no-fail-fast`, `cargo nextest run -p taru-server startup --no-fail-fast`, `cargo nextest run -p taru-db job_lease --no-fail-fast`, `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`, `cargo fmt --all -- --check`, WORKSTREAM JSON parse, `git diff --check` | Pass | Runtime now exact-claims queued jobs, heartbeats leases, and persists success/failure through the run-token fence. |
| 2026-05-19 | DJOL-050 | `cargo nextest run -p taru-api admin_job_cancel --no-fail-fast`, `cargo nextest run -p taru-server job_cancel --no-fail-fast`, `cargo check -p taru-api -p taru-server --tests` | Pass | Admin cancel-request route is truthful for queued/running/terminal jobs and returns redacted job summaries. |
| 2026-05-19 | DJOL-060 | `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`, `cargo nextest run -p taru-db job_lease --no-fail-fast`, `cargo nextest run -p taru-db job_cancel --no-fail-fast`, `cargo nextest run -p taru-server job_runtime --no-fail-fast`, `cargo nextest run -p taru-server startup --no-fail-fast`, `cargo nextest run -p taru-server job_cancel --no-fail-fast`, `cargo nextest run -p taru-api admin_job_cancel --no-fail-fast`, `cargo fmt --all -- --check`, WORKSTREAM JSON parse, `git diff --check` | Pass | Closeout gate passed; remaining worker checkpoints and broader migrations are split follow-ons. |

## Review Expectations

- `review-workstream` before accepting `DJOL-030`, `DJOL-040`, `DJOL-050`, or
  lane closeout.
- Every repository mutation that writes running job state must document whether
  it requires the run token.
- Cancel request must never be described as completed cancellation until a
  worker has acknowledged it.
- Redaction inventory must verify raw job payloads, source locators, storage
  handles, paths, provider payloads, and secrets are not exposed.
