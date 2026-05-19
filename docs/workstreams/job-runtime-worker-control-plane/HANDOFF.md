# Job Runtime Worker Control Plane Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane has been opened as the recommended follow-on after Managed Artwork
ingest requeue. No runtime code has changed in this lane yet.

## Next Task

Start with `JRWCP-010`:

- inventory existing job execution paths;
- reconcile ADR 0006 and ADR 0019 with current code;
- choose the minimal shared worker contract;
- identify the first Managed Artwork worker tracer bullet.

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
