# NFO Sidecar Cancellation Checkpoints - Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-nfo import --no-fail-fast
```

This starts from the current library import loop where every source is processed
without a per-sidecar cancellation checkpoint.

## Gate Set

### NFO Crate Contract Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo check -j 2 -p taru-nfo --tests
cargo nextest run -j 2 -p taru-nfo nfo_service --no-fail-fast
```

Expected coverage:

- checkpoint API compiles without a `taru-server` dependency;
- no-op import/export paths preserve existing behavior;
- cancelled outcome is distinct from service failure.

### Import Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-nfo import --no-fail-fast
```

Expected coverage:

- import checks before the next source sidecar;
- cancellation is not counted as failed NFO import;
- partial summaries do not include XML or sidecar paths.

### Export Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-nfo export --no-fail-fast
```

Expected coverage:

- export checks before the next source sidecar;
- the cancelled source is not written or backed up;
- existing single-source export behavior remains compatible.

### Server Integration Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-server nfo --no-fail-fast
cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast
```

Expected coverage:

- durable NFO import/export jobs acknowledge cancellation as terminal
  `cancelled`;
- success outbox events are skipped after cancellation;
- Admin cancellation response stays redacted.

### Cross-Crate Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
cargo check -j 2 -p taru-core -p taru-db -p taru-nfo -p taru-server --tests
```

### Formatting And Static Gate

```powershell
cargo fmt --all -- --check
git diff --check
Get-Content docs\workstreams\nfo-sidecar-cancellation-checkpoints\WORKSTREAM.json | ConvertFrom-Json
```

## Evidence Anchors

- `docs/workstreams/nfo-sidecar-cancellation-checkpoints/DESIGN.md`
- `docs/workstreams/nfo-sidecar-cancellation-checkpoints/TODO.md`
- `crates/taru-nfo/src/import.rs`
- `crates/taru-nfo/src/export.rs`
- `crates/taru-nfo/src/summary.rs`
- `crates/taru-server/src/app/nfo.rs`
- `docs/api/HTTP_API.md`

## Current Evidence

- `NSCC-010`: Lane opened after `worker-job-cancellation-checkpoints` closeout
  split NFO per-sidecar cancellation into this follow-on. Fresh gates before
  commit: WORKSTREAM JSON parse and `git diff --check`.

## Notes

Fresh verification is required before marking a task or the lane complete.
Cancellation tests must prove terminal durable `cancelled` at the server layer
and distinct service `Cancelled` outcomes at the `taru-nfo` layer.
