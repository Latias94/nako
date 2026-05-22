# Worker Job Cancellation Checkpoints - Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

Exit criteria:

- Durable cancel-request gap is stated.
- Runtime, worker, Admin, and non-goal boundaries are explicit.
- First executable task is a runtime cancellation context, not broad worker
  migration.

Primary evidence:

- `docs/workstreams/worker-job-cancellation-checkpoints/DESIGN.md`
- `docs/workstreams/worker-job-cancellation-checkpoints/TODO.md`

## M1 - Runtime Cancellation Context

Exit criteria:

- `DurableJobRuntime` has a per-run cancellation context.
- Heartbeat-observed cancel intent can be checked by the operation closure.
- Observed cancellation persists terminal `cancelled` through
  `cancel_leased_job`.
- Cancellation is not recorded as `failed`.

Primary gates:

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
cargo nextest run -p nako-server job_runtime --no-fail-fast
cargo check -p nako-server --tests
```

## M2 - First Real Worker Checkpoints

Exit criteria:

- A real typed worker checks cancellation before a new side-effect unit.
- Admin running cancel request can move to acknowledged terminal `cancelled`.
- Success events are not emitted for cancelled runs.

Primary gates:

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
cargo nextest run -p nako-server job_cancel --no-fail-fast
cargo nextest run -p nako-server metadata --no-fail-fast
```

## M3 - Additional Worker Boundaries

Exit criteria:

- Library scan/probe and NFO import/export either have checkpoints at safe
  boundaries or documented follow-ons with explicit reasons.
- Docs explain that cancellation is cooperative and boundary-based.
- No retry/backoff or distributed scheduling behavior is hidden in this lane.

Primary gates:

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
cargo nextest run -p nako-server job_runtime --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
```

## M4 - Closeout

Exit criteria:

- Gate set is recorded with fresh evidence.
  Result: closeout evidence is recorded in `EVIDENCE_AND_GATES.md`.
- Remaining migrations are completed or split.
  Result: remaining work is split by boundary type, including per-sidecar NFO
  checkpoints, dispatch checkpoints, retry/backoff, lease requeue/stealing, and
  child-process cancellation.
- `WORKSTREAM.json` status matches reality.
  Result: workstream status is `completed`.
- `HANDOFF.md` names the next highest-leverage follow-on.
  Result: handoff names focused follow-ons instead of keeping this lane open.
