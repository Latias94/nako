# Worker Job Cancellation Checkpoints - Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_runtime --no-fail-fast
```

This proves the shared leased runtime can distinguish success, failure, and
acknowledged cancellation before worker-specific integrations are broadened.

## Gate Set

### Runtime Iteration Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_runtime --no-fail-fast
```

Expected coverage:

- exact job claim still works;
- heartbeat still persists;
- observed cancel request becomes terminal `cancelled`;
- non-cancellation failures still become `failed`.

### First Worker Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_cancel --no-fail-fast
cargo nextest run -p taru-server metadata --no-fail-fast
```

Expected coverage:

- Admin running cancel request remains truthful;
- metadata maintenance checks cancellation before the next item;
- cancelled metadata maintenance does not emit a success/completed event.

### Cross-Crate Contract Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
```

Expected coverage:

- core job cancellation types still compile across repository, DB, API, and
  server boundaries.

### Formatting And Static Gate

```powershell
cargo fmt --all -- --check
git diff --check
Get-Content docs\workstreams\worker-job-cancellation-checkpoints\WORKSTREAM.json | ConvertFrom-Json
```

Expected coverage:

- Rust formatting is stable;
- patches have no whitespace errors;
- workstream metadata remains parseable JSON.

### Review Gate

Run `review-workstream` before marking implementation tasks or the lane
complete. Record blocking findings, missing gates, and residual risks here or
in `HANDOFF.md`.

## Evidence Anchors

- `docs/workstreams/worker-job-cancellation-checkpoints/DESIGN.md`
- `docs/workstreams/worker-job-cancellation-checkpoints/TODO.md`
- `crates/taru-server/src/app/job_runtime.rs`
- `crates/taru-server/src/app/metadata.rs`
- `docs/api/HTTP_API.md`

## Current Evidence

- `WJCC-010`: Opened the lane and recorded the cancellation boundary after
  `durable-job-ownership-leases` closed. Fresh gates to run before commit:
  `Get-Content docs\workstreams\worker-job-cancellation-checkpoints\WORKSTREAM.json | ConvertFrom-Json`;
  `git diff --check`.

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete. Cancellation tests must assert terminal `cancelled`, not only that a
request flag was persisted.
