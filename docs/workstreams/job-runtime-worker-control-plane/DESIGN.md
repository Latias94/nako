# Job Runtime Worker Control Plane Design

Status: Active
Last updated: 2026-05-19

## Problem

Durable jobs currently provide persistence and Admin visibility, but execution
ownership is fragmented. Some workflows run synchronously in Admin routes,
some are launched through service methods, and some background work has its own
ad hoc loop. This makes it hard to answer operational questions:

- who owns a running job;
- whether a job can be safely cancelled;
- whether a failed job should be requeued in place or retried as a new attempt;
- how stalled running jobs recover after process restart;
- which resource budget applies to each job kind;
- how Admin APIs can expose controls without leaking durable inputs or errors.

Managed Artwork ingest made the gap concrete. Its manual `process-next` route
is correct as a debug/Admin seam, but the long-term architecture needs a
supervised worker boundary that owns repeated claim/execute/fail/succeed loops.

## Architecture Direction

Use a layered control-plane model:

1. `taru-core` owns typed job runtime records and repository traits.
2. `taru-db` owns atomic claim, lease heartbeat, completion, failure, cancel,
   and recovery mutations.
3. `taru-server::app::runtime` owns process-local worker registration,
   cancellation tokens, shutdown, task diagnostics, and resource permits.
4. Feature executors stay typed. A Managed Artwork ingest executor still uses
   Managed Artwork domain records and artifact commit/failure methods.
5. HTTP Admin routes expose redacted controls and diagnostics only.

This lane should not create a generic "deserialize any input and run anything"
engine. Job kinds differ in side effects, cancellation checkpoints, idempotency,
and safe retry rules. The shared boundary should own lifecycle mechanics; each
job kind should own domain execution.

## First Target: Managed Artwork Ingest Worker

The first runtime slice should prove:

- a supervised worker loop can claim queued Managed Artwork ingest jobs;
- the loop uses the existing fetch/validate/store/fail pipeline;
- successful jobs end as stored artifacts with safe summaries;
- failed jobs end as failed with safe summaries;
- Admin `process-next` remains useful as a manual single-step command until the
  worker is mature;
- requeue remains a control-plane command and does not fetch bytes directly.

## Current Runtime Inventory

Existing execution surfaces:

| Surface | Current Owner | Shape | Notes |
| --- | --- | --- | --- |
| Library scan | `LibraryScanAppService` | Enqueue job, then `RuntimeSupervisor::spawn_job`; execution uses `DurableJobRuntime`. | Already close to desired supervised job execution, but no shared claim loop because jobs are spawned immediately after enqueue. |
| Metadata refresh and maintenance | `MetadataAppService` | Enqueue job, then `RuntimeSupervisor::spawn_job`; scheduled policy tasks use `RuntimeSupervisor::spawn`. | Scheduled task enqueues jobs; each job still runs as an immediate supervised task. |
| NFO import/export | `NfoAppService` | Enqueue job, then `RuntimeSupervisor::spawn_job`; execution uses `DurableJobRuntime`. | Similar to library scan. |
| Managed Artwork ingest | `ManagedArtworkAppService` | Admin `process-next` route claims one queued ingest and executes synchronously. | Has strong claim/commit/fail/requeue repository semantics, but no supervised background loop. |
| Webhook delivery | `WebhookAppService` | Admin/API command uses local `JoinSet` for endpoint fan-out. | Delivery attempts have their own domain state, but are not currently durable jobs. |
| Staging lease drop cleanup | `StagingLease::drop` | Fire-and-forget `RuntimeSupervisor::spawn`. | Runtime-supervised task, but not a durable job. |
| Startup recovery | `ServerStartupWorkflow` | Marks unfinished durable jobs failed on startup. | Coarse recovery: no ownership lease, all queued/running unfinished jobs fail. |
| Playback/transcode | `PlaybackAppService` and session manager | Uses transcode session state, not generic `jobs`. | Has its own cancellation and stale recovery semantics; do not fold into the first job-worker slice. |

Existing runtime primitives:

- `RuntimeSupervisor::spawn`: process-local task registration, cancellation on
  shutdown, panic accounting, diagnostics.
- `RuntimeSupervisor::spawn_job`: supervised task tied to one `JobId`, with
  success/failure counters.
- `DurableJobRuntime::run_job`: starts one job, runs typed work, writes
  succeeded or failed job state.
- `JobRepository`: enqueue/start/succeed/fail/list/get, but no typed
  claim-next, lease heartbeat, cancel request, or retry-at/backoff.
- Managed Artwork repository methods already implement a typed claim/commit/fail
  state machine for `managed_artwork_ingest` rows plus their durable job rows.

## Minimal Shared Contract For First Code Slice

Do not start by adding generic `jobs` table leases. The least risky vertical
slice is:

1. extract the duplicated "claim one unit, execute it, report whether work was
   found" shape behind a small worker runner in `taru-server`;
2. keep claim/commit/fail in the Managed Artwork repository because it must
   update ingest and job state atomically;
3. register the runner with `RuntimeSupervisor::spawn`, not bare
   `tokio::spawn`;
4. use configured artwork/fetch resource limits already owned by
   `ManagedArtworkAppService`;
5. keep `process-next` as the manual single-step API while the worker runner is
   introduced.

Candidate contract names for `JRWCP-020`:

```text
ManagedArtworkAppService::process_next_unit()
ManagedArtworkWorker::run_until_idle()
RuntimeWorkerLoop::run_until_idle(...)
```

The first implementation should prefer a concrete Managed Artwork worker over a
generic trait if the generic shape would hide domain invariants. A small
internal helper that owns idle/backoff/shutdown behavior is acceptable once two
workers need it.

## Open Design Questions

| Question | Initial Direction | Why It Matters |
| --- | --- | --- |
| Lease model | Add process-local owner/heartbeat fields before broad worker adoption. | Existing `running` jobs can be stale after process death. |
| Retry shape | Keep existing Managed Artwork requeue-in-place for now; design new-attempt retry as a later policy. | ADR 0006 prefers new rows for generic retry, but Managed Artwork has a proven in-place requeue contract. |
| Cancellation | Introduce cancellation request semantics only after worker ownership exists. | Cancellation without a registered runner is only state decoration. |
| Resource budgets | Declare resource class permits in runtime config and supervisor. | Artwork fetch, metadata network, webhook, CPU/GPU work need separate limits. |
| Admin controls | Return redacted summaries with presence flags, not raw inputs/errors. | Job input and summary JSON can contain private operational data. |

Updated decision for the first implementation slice: postpone generic lease
schema until a second queued worker needs it. Managed Artwork already has a
typed claim transition from queued to fetching/running and a typed startup
recovery path for claimed work. The first worker uses that existing claim
boundary and keeps the design open for later generic job leases.

## `JRWCP-020` Implementation Shape

The first worker slice is intentionally concrete:

- `[artwork].ingest_worker_enabled` defaults to `false`.
- `[artwork].ingest_worker_idle_ms` controls the idle sleep when no queued
  Managed Artwork ingest is available.
- `TaruApp` starts the worker only after startup recovery, cleanup, and
  configured library reconciliation finish.
- The worker is registered through `RuntimeSupervisor::spawn` as
  `managed_artwork_ingest_worker` with resource class `artwork.ingest`.
- Admin `process-next` and the worker both call the same internal
  `process_next_unit` helper, so claim, fetch, validation, artifact write,
  commit, and safe failure persistence cannot drift.
- Public Client image shape is unchanged; successful ingest still stores a
  Managed Artwork Artifact and does not publish Selected Artwork.

This is not a full durable lease model. `JRWCP-030` owns the first concrete
restart recovery policy and keeps generic ownership leases as a follow-on.

## `JRWCP-030` Recovery Shape

Managed Artwork ingest uses typed recovery rather than the generic durable job
startup cleanup:

- queued Managed Artwork ingests remain queued across restart;
- already claimed ingests (`fetching` or `validating`) whose durable job is
  still `running` are failed with `startup_recovery`;
- recovered rows keep `artifact_id = NULL` and can be requeued by the existing
  Admin requeue command;
- the generic `fail_unfinished_jobs` path skips `managed_artwork_ingest` jobs so
  it cannot mark queued artwork work failed before the worker has a chance to
  drain it;
- no raw candidate source URI, job input, job error, storage URI, or local path
  is exposed through startup summaries.

## Redaction Policy

Runtime worker/control-plane responses must not expose:

- raw `input_json`;
- raw `summary_json`;
- raw `error`;
- Source Locators;
- raw candidate `source_uri`;
- addon payload/provenance JSON;
- provider query strings;
- token or secret values;
- `storage_uri` or `managed-artwork://...`;
- local paths, cache URIs, artifact roots, staging paths, or content hashes.

## Closeout Condition

This lane can close only when a shared job runtime boundary has at least one
real vertical implementation slice, fresh validation evidence, and explicit
follow-ons for job kinds not migrated. A design-only inventory task is not
enough to close the lane.
