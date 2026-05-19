# Durable Job Ownership Leases - Milestones

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- ADR/workstream authority is linked.
- Existing schema, repository, runtime, and Managed Artwork gaps are recorded.
- Non-goals keep retry/backoff, distributed scheduling, playback cancellation,
  and mass worker migration out of the first slice.

Primary evidence:

- `docs/workstreams/durable-job-ownership-leases/DESIGN.md`
- `docs/workstreams/durable-job-ownership-leases/TODO.md`
- `docs/workstreams/durable-job-ownership-leases/WORKSTREAM.json`

## M1 - State Machine And Contract Freeze

Exit criteria:

- The lane chooses whether to add terminal `cancelled` status or represent
  acknowledged cancellation another way. Result: terminal `cancelled` status.
- Cancel request is defined separately from cancellation acknowledgement.
  Result: request and acknowledgement have separate repository methods.
- Ownership identity, run token, lease expiry, and heartbeat names are fixed.
  Result: `worker_id`, `run_token`, `lease_expires_at`, and `heartbeat_at`.
- Repository contracts are named before database implementation starts.
  Result: core trait methods exist with default unsupported implementations.
- Any ADR 0006 delta is documented. Result: ADR 0006 now records leases,
  fencing, and terminal cancellation.

Primary gates:

- `cargo check -p taru-core --tests`
- `cargo fmt --all -- --check`

## M2 - Durable Schema And Repository Proof

Exit criteria:

- Schema migration adds the durable fields.
- SQLite tests prove claim, stale-token rejection, heartbeat extension,
  completion/failure fencing, cancel request, queued cancellation, and expired
  lease recovery.
- Existing list/get job behavior remains compatible or has an intentional
  Admin DTO update.

Primary gates:

- `cargo nextest run -p taru-db job_lease --no-fail-fast`
- `cargo nextest run -p taru-db job_cancel --no-fail-fast`

## M3 - First Runtime Integration

Exit criteria:

- Shared `DurableJobRuntime::run_job` uses the leased contract end to end.
  Result: library scan, metadata refresh/maintenance, and NFO import/export
  execution paths now use exact claim, heartbeat, and run-token fenced
  completion through the shared runtime.
- Runtime diagnostics remain process-local and do not overclaim durable state.
  Result: worker ID is process-local diagnostic identity; run token remains the
  durable write fence.
- Startup recovery is lease-aware.
- Stale owners cannot complete or fail jobs after the lease token changes.

Primary gates:

- `cargo nextest run -p taru-server job_runtime --no-fail-fast`
- `cargo nextest run -p taru-server startup --no-fail-fast`

## M4 - Truthful Cancel Request Controls

Exit criteria:

- Admin cancel-request behavior exists only if a worker can observe it.
- Queued, running, terminal, and expired-lease cases have separate test
  coverage.
- Responses are redacted and do not expose raw durable payloads.

Primary gates:

- `cargo nextest run -p taru-server job_cancel --no-fail-fast`
- `cargo check -p taru-api -p taru-server --tests`

## M5 - Closeout Or Split

Exit criteria:

- Gate set is recorded with fresh evidence.
- Workstream status is updated.
- Remaining worker migrations are completed, deferred, or split into named
  follow-ons.
