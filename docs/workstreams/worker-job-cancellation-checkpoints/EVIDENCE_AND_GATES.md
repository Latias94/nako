# Worker Job Cancellation Checkpoints - Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-server job_runtime --no-fail-fast
```

This proves the shared leased runtime can distinguish success, failure, and
acknowledged cancellation before worker-specific integrations are broadened.

## Gate Set

### Runtime Iteration Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-server job_runtime --no-fail-fast
```

Expected coverage:

- exact job claim still works;
- heartbeat still persists;
- observed cancel request becomes terminal `cancelled`;
- non-cancellation failures still become `failed`.

### First Worker Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast
cargo nextest run -j 2 -p taru-server metadata --no-fail-fast
```

Expected coverage:

- Admin running cancel request remains truthful;
- metadata maintenance checks cancellation before the next item;
- cancelled metadata maintenance does not emit a success/completed event.

### Additional Worker Boundary Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
cargo nextest run -j 2 -p taru-server background_scan_job_acknowledges_cancellation_before_probe_stage --no-fail-fast
cargo nextest run -j 2 -p taru-server nfo --no-fail-fast
```

Expected coverage:

- library scan cancellation stops before the probe side-effect boundary;
- cancelled library scan does not emit a `LibraryScanned` success event;
- NFO import/export jobs still persist successful summaries and events after
  moving to the context-aware runtime;
- per-sidecar NFO cancellation remains an explicit follow-on, not an implied
  app-layer guarantee.

### Cross-Crate Contract Gate

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
$env:CARGO_BUILD_JOBS='2'
cargo check -j 2 -p taru-core -p taru-db -p taru-api -p taru-server --tests
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
- `WJCC-020`: Added runtime cancellation context support and a runtime test that
  requests cancellation while a job is running, waits for heartbeat-observed
  cancel intent, checks the context, and verifies terminal
  `JobStatus::Cancelled` without summary or error leakage. Gates run:
  `cargo nextest run -p taru-server job_runtime --no-fail-fast`;
  `cargo check -p taru-server --tests`; `cargo fmt --all -- --check`;
  `git diff --check`; WORKSTREAM JSON parse.
- `WJCC-030` (2026-05-19): Migrated metadata maintenance to the
  context-aware durable job path. Checkpoints refresh observed cancel intent
  before each item-level side-effect unit, acknowledged cancellation persists
  terminal `JobStatus::Cancelled`, completed metadata-maintenance outbox events
  are skipped for cancelled runs, and runtime diagnostics count cancelled jobs
  separately from successful jobs. Gates run sequentially with
  `CARGO_BUILD_JOBS=2` and `NEXTEST_TEST_THREADS=1`:
  `cargo check -j 2 -p taru-api -p taru-server --tests` (pass);
  `cargo nextest run -j 2 -p taru-server metadata_maintenance_job_acknowledges_cancellation_before_next_item --no-fail-fast`
  (1 passed);
  `cargo nextest run -j 2 -p taru-server job_runtime --no-fail-fast`
  (5 passed);
  `cargo nextest run -j 2 -p taru-server job_cancel --no-fail-fast`
  (1 passed);
  `cargo nextest run -j 2 -p taru-server runtime --no-fail-fast` (16 passed);
  `cargo nextest run -j 2 -p taru-server metadata --no-fail-fast` (20 passed);
  `cargo fmt --all -- --check` (pass);
  `git diff --check` (pass, CRLF warnings only). After tightening supervisor
  outcome counting to classify `JobStatus::Failed` returned by a supervised
  job as a failed job instead of success, reran
  `cargo check -j 2 -p taru-server --tests` (pass) and
  `cargo nextest run -j 2 -p taru-server runtime --no-fail-fast` (16 passed).
- `WJCC-040` (2026-05-19): Migrated library scan and NFO import/export jobs to
  the context-aware durable runtime. Library scan checkpoints now refresh
  cancellation before indexing, before probe, and before success publication;
  cancelled scan runs skip `LibraryScanned`. NFO import/export have app-level
  pre/post service checkpoints and keep per-sidecar cancellation as an explicit
  `taru-nfo` API follow-on. Gates run sequentially with `CARGO_BUILD_JOBS=2`
  and `NEXTEST_TEST_THREADS=1`:
  `cargo check -j 2 -p taru-server --tests` (pass);
  `cargo nextest run -j 2 -p taru-server background_scan_job_acknowledges_cancellation_before_probe_stage --no-fail-fast`
  (1 passed);
  `cargo nextest run -j 2 -p taru-server nfo --no-fail-fast` (9 passed);
  `cargo nextest run -j 2 -p taru-server job_runtime --no-fail-fast`
  (5 passed);
  `cargo fmt --all -- --check` (pass);
  `git diff --check` (pass, CRLF warnings only);
  WORKSTREAM JSON parse (pass).

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete. Cancellation tests must assert terminal `cancelled`, not only that a
request flag was persisted.
