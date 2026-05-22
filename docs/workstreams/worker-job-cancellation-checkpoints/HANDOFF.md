# Worker Job Cancellation Checkpoints - Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

The lane is closed. It follows `durable-job-ownership-leases`, which added
durable job leases, heartbeats, redacted Admin cancel requests, and fenced
`cancel_leased_job`, but deliberately split worker-side cancellation
checkpoints.

`WJCC-020`, `WJCC-030`, and `WJCC-040` are complete. `DurableJobRuntime` now
supports a context-aware run path, heartbeat publication of observed cancel
intent, cooperative checkpoint helpers, and fenced acknowledgement of terminal
`cancelled`. Metadata maintenance is the first item-level worker integration.
Library scan now checks before scan, before probe, and before success
publication. NFO import/export use app-level pre/post service checkpoints, with
per-sidecar cancellation split to a future `nako-nfo` API boundary.

## Closed Task

- Task ID: `WJCC-050`
- Owner: planner
- Files:
  - `docs/workstreams/worker-job-cancellation-checkpoints`
- Validation:
  - closeout gate in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: no blocking workstream-compliance or code-quality findings remained
  at closeout.
- Evidence: `WJCC-020` proved runtime cancellation acknowledgement;
  `WJCC-030` proved metadata maintenance item checkpoints; `WJCC-040` proved
  library scan/probe boundaries and NFO app-level boundaries; `WJCC-050`
  recorded fresh closeout gates.

## Decisions Since Last Update

- Use cooperative cancellation checkpoints instead of force-killing arbitrary
  worker futures.
- Put the first shared contract in `DurableJobRuntime` because current library
  scan, metadata, and NFO job paths already flow through it.
- Use heartbeat-observed `cancel_requested_at` for the first runtime signal.
- Keep retry/backoff, expired-lease policy, distributed scheduling, and
  process-kill cancellation out of this lane.
- Prefer metadata maintenance as the first real worker integration because it
  naturally loops over item-level side-effect units.
- Existing `run_job` callers intentionally keep the pre-existing success/failure
  interface. Real cooperative cancellation requires migrating a worker to
  `run_job_with_context`.
- Runtime diagnostics now expose `cancelled_jobs` separately from
  `succeeded_jobs`; Admin overview should not report acknowledged cancellation
  as success.
- Use low-concurrency validation commands for this workspace unless the user
  explicitly asks for broader parallel verification:
  `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=1`, and `cargo nextest run -j 2`.
- Library scan/probe has a tested checkpoint before probe. It does not kill an
  in-flight VFS scan or ffprobe process.
- NFO import/export now avoid success publication after app-level cancellation,
  but per-sidecar stop-before-read/write needs a dedicated `nako-nfo` service
  API follow-on.

## Blockers

- None.

## Follow-Ons

- Per-sidecar NFO cancellation: add a `nako-nfo` service API that checks before
  each source read/write and proves no success event is emitted after an
  acknowledged cancellation.
- Webhook/addon/automation dispatch checkpoints: add cooperative checkpoints
  before each dispatch or side-effect unit.
- Retry/backoff policy: define cancellation, failure, and retry scheduling
  without mixing it into the checkpoint contract.
- Expired-lease requeue/stealing: define job-kind-specific recovery semantics
  for stale owners.
- Child-process cancellation: decide whether ffprobe/transcode durable jobs
  need process cancellation handles distinct from cooperative Rust checkpoints.

## Next Recommended Action

Open the per-sidecar NFO cancellation follow-on first if the next priority is
user-visible file metadata correctness. Open retry/backoff first if the next
priority is operations reliability for failed or abandoned jobs.
