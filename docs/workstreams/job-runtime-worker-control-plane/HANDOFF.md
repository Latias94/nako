# Job Runtime Worker Control Plane Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane now has a first runtime implementation slice for Managed Artwork
ingest plus typed startup recovery for claimed Managed Artwork work. The lane
remains active because cancellation semantics are not yet decided.

## Completed In `JRWCP-010`

Inventory result:

- `RuntimeSupervisor::spawn_job` already supervises immediate one-job tasks.
- `DurableJobRuntime::run_job` already wraps start/succeed/fail for job kinds
  like library scan, metadata refresh/maintenance, and NFO import/export.
- Managed Artwork ingest has stronger typed claim/commit/fail/requeue methods
  because ingest rows and durable job rows must move atomically.
- Managed Artwork is still driven through Admin `process-next`, so it lacks a
  supervised background loop.
- Generic job leases should wait until a second queued worker needs them; the
  first slice should reuse the typed Managed Artwork claim boundary.

## Next Task

Continue with `JRWCP-040` or split it:

- decide whether Managed Artwork ingest cancellation is worth implementing
  before a generic ownership/lease model exists;
- if cancellation is implemented, add only a safe requested-state transition
  that the worker can observe at a checkpoint;
- otherwise close this lane and split cancellation into a later durable job
  control-plane lane.

## Completed In `JRWCP-020`

- Added `[artwork].ingest_worker_enabled`, defaulting to `false`.
- Added `[artwork].ingest_worker_idle_ms`, defaulting to `1000`.
- `TaruApp` starts one `managed_artwork_ingest_worker` through
  `RuntimeSupervisor` after startup workflow completion when the worker is
  enabled.
- `ManagedArtworkAppService::process_next` and the worker share
  `process_next_unit`, keeping manual and background execution on the same safe
  artifact pipeline.
- Added focused HTTP/runtime coverage for worker success without Admin
  `process-next`.
- Public Client image shape is unchanged and worker ingest does not publish
  Selected Artwork.

## Completed In `JRWCP-030`

- Generic `fail_unfinished_jobs` now skips `managed_artwork_ingest` jobs.
- Managed Artwork startup recovery is typed:
  - queued ingests stay queued;
  - claimed `fetching`/`validating` ingests with running jobs fail with
    `startup_recovery`;
  - recovered failures remain requeueable;
  - no artifact is created or duplicated.
- Added DB and server startup tests for the recovery policy.

## Files To Inspect First

- `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
- `docs/adr/0019-server-architecture-hardening-boundaries.md`
- `crates/taru-core/src/job.rs`
- `crates/taru-core/src/repository/jobs.rs`
- `crates/taru-db/src/jobs.rs`
- `crates/taru-server/src/app/runtime.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-db/src/artwork.rs`
- `docs/workstreams/managed-artwork-ingest-runtime-controls/HANDOFF.md`

## Suggested Validation

```powershell
Get-Content docs\workstreams\job-runtime-worker-control-plane\WORKSTREAM.json | ConvertFrom-Json
git diff --check
```

After code changes begin:

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-server job_runtime_worker --no-fail-fast
cargo nextest run -p taru-db managed_artwork_ingest --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```
