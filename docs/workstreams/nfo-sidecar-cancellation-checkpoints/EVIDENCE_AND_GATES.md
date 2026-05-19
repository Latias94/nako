# NFO Sidecar Cancellation Checkpoints - Evidence And Gates

Status: Complete
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
- `NSCC-020`: `crates/taru-nfo/src/summary.rs` defines redacted sidecar
  checkpoint and cancellation outcome types; import/export no-op wrappers keep
  existing callers on the original summary-returning APIs.
- `NSCC-030`: `nfo_service_import_can_cancel_before_next_sidecar_without_failure`
  proves import cancellation happens before the next source sidecar and is not
  counted as an NFO failure.
- `NSCC-040`: `nfo_service_export_can_cancel_before_next_sidecar_without_writing_it`
  proves export cancellation prevents the next sidecar write.
- `NSCC-050`: `nfo_import_job_acknowledges_cancellation_before_next_sidecar`
  and `nfo_export_job_acknowledges_cancellation_before_next_sidecar` prove
  durable server jobs persist terminal `cancelled`, omit success
  summary/error, and skip `NfoImported`/`NfoExported`.

## Fresh Verification - 2026-05-19

All commands used low-concurrency settings:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
```

- `cargo check -j 2 -p taru-nfo --tests`: passed.
- `cargo nextest run -j 2 -p taru-nfo nfo_service --no-fail-fast`: passed,
  18 passed, 4 skipped.
- `cargo nextest run -j 2 -p taru-nfo import --no-fail-fast`: passed, 6
  passed, 16 skipped.
- `cargo nextest run -j 2 -p taru-nfo export --no-fail-fast`: passed, 7
  passed, 15 skipped.
- `cargo nextest run -j 2 -p taru-server nfo --no-fail-fast`: passed, 11
  passed, 149 skipped.
- `cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast`: passed,
  1 passed, 159 skipped.
- `cargo check -j 2 -p taru-core -p taru-db -p taru-nfo -p taru-server --tests`:
  passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed with CRLF working-copy warnings only.
- `Get-Content docs\workstreams\nfo-sidecar-cancellation-checkpoints\WORKSTREAM.json | ConvertFrom-Json | Out-Null`:
  passed.

## Notes

The lane is closed. Cancellation remains cooperative at sidecar boundaries and
does not attempt to interrupt an already-started storage read/write.
