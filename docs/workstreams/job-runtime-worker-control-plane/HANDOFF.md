# Job Runtime Worker Control Plane Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane has been opened as the recommended follow-on after Managed Artwork
ingest requeue. No runtime code has changed in this lane yet.

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

Continue with `JRWCP-020`:

- add a concrete Managed Artwork ingest worker registered through
  `RuntimeSupervisor`;
- keep `process-next` as the manual single-step Admin command;
- process queued work through the existing safe artifact pipeline;
- prove success path without calling Admin `process-next`;
- avoid broad generic scheduler or lease schema in this first slice.

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
cargo nextest run -p taru-db job_runtime_worker --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```
