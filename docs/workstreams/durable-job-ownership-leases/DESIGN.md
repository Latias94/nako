# Durable Job Ownership Leases Design

Status: Active
Last updated: 2026-05-19

## Why This Lane Exists

The previous job-runtime worker lane proved one important runtime shape: a
Managed Artwork ingest worker can be registered through `RuntimeSupervisor`,
drain queued work, and recover claimed ingest rows on startup. It deliberately
split cancellation and generic leases because the durable `jobs` table cannot
currently represent truthful ownership or cancellation.

Without this lane, the next worker migrations would keep stacking process-local
behavior on top of rows that only say `queued`, `running`, `succeeded`, or
`failed`.

## Relevant Authority

- ADRs:
  - `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
- Existing workstreams:
  - `docs/workstreams/durable-job-recovery/`
  - `docs/workstreams/durable-job-runtime-admin-read-model/`
  - `docs/workstreams/managed-artwork-ingest-runtime-controls/`
  - `docs/workstreams/job-runtime-worker-control-plane/`

## Problem

The current durable job model records lifecycle status, but not execution
ownership:

- Until `DJOL-020`, `JobStatus` had only `queued`, `running`, `succeeded`, and
  `failed`; it now also has terminal `cancelled`.
- `jobs` has no worker identity, lease expiry, heartbeat, run token, cancel
  request, or cancellation terminal state.
- `JobRepository::start_job` updates a job by ID without fencing on the prior
  status or an owner token.
- `DurableJobRuntime::run_job` can start/succeed/fail a job, but it cannot
  prove that the same worker still owns the job.
- `RuntimeSupervisor` can cancel or abort process-local tasks, but that state
  disappears on restart.
- Startup recovery can fail unfinished jobs, but it cannot tell stale running
  work from queued work that should still be drained.
- Managed Artwork ingest has a typed claim/requeue/recovery path, but no
  durable cancellation request or reusable lease contract.

The result is an operational API trap: Taru can show that work is running, but
cannot safely promise "who owns it", "when ownership expires", or "whether a
cancel request was observed".

## Target State

When this lane closes:

- durable jobs have a fenced claim model with worker identity, run token, lease
  expiry, and heartbeat timestamps;
- completion, failure, cancellation, and heartbeat writes are conditional on
  the current run token;
- startup recovery is lease-aware instead of blindly failing all unfinished
  work;
- queued cancellation and running cancellation have different truthful
  behavior;
- at least one real job execution path uses the leased lifecycle end to end;
- Admin read/control surfaces expose only redacted lifecycle facts.

## In Scope

- Durable job ownership and lease state in `taru-core` and `taru-db`.
- Repository contracts for claim, heartbeat, finish, fail, request cancel, and
  lease-aware recovery.
- Runtime integration for one real job execution path before broader migration.
- Admin diagnostics/control DTOs only where the backing semantics are truthful.
- ADR/workstream updates when the chosen state model changes the durable job
  contract.

## Out Of Scope

- Automatic retry/backoff scheduling.
- A distributed scheduler across multiple server processes.
- Migrating every existing job kind in the first code slice.
- Generic untyped execution of arbitrary job input payloads.
- Process-kill cancellation semantics.
- Playback/transcode cancellation refactoring.
- Public Client API changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Cancellation request is not a terminal status by itself. | High | ADR 0006 says cancellation is requested and must be observed at a checkpoint. | If treated as terminal, Taru would claim side effects stopped before the worker proved it. |
| A terminal `cancelled` status is cleaner than encoding operator cancellation as `failed`. | Medium | Admin job lists need to distinguish failure from requested cancellation. | If compatibility cost is too high, cancellation can initially be a failed job with a typed safe reason, but that should be explicit. |
| Ownership needs a fencing token, not only a worker ID. | High | A stale worker can outlive a stolen or renewed lease. | Without fencing, old work can complete a job it no longer owns. |
| Lease-aware recovery should not fail queued jobs. | High | Managed Artwork worker recovery already preserves queued ingest work. | If future dispatchers do not resume queued work, recovery policy must remain job-kind-specific. |
| Retry/backoff should remain separate. | High | ADR 0006 requires per-job-kind retry policy. | Mixing retry into leases would obscure ownership correctness and side-effect safety. |

## Current Inventory

### Durable Job Schema

`crates/taru-db/migrations/0003_jobs.sql` creates:

- `id`
- `kind`
- `status`
- `resource_class`
- `library_id`
- `source_id`
- `summary_json`
- `error`
- `queued_at`
- `started_at`
- `completed_at`
- `updated_at`

`0004_job_input_payload.sql` adds `input_json`.

Missing durability fields:

- owner worker ID;
- run or lease token;
- lease expiration timestamp;
- last heartbeat timestamp;
- cancel-request timestamp and safe reason;
- terminal cancellation status.

### Repository Shape

`JobRepository` currently exposes:

- `enqueue_job`
- `start_job`
- `succeed_job`
- `fail_job`
- `fail_unfinished_jobs`
- `get_job`
- `list_jobs`

Missing operations:

- claim next queued job for a worker;
- start a known job only if it is still claimable;
- heartbeat a running job only for the current run token;
- complete/fail/cancel only for the current run token;
- request cancellation without claiming the job stopped;
- recover expired running leases without failing queued jobs.

`DJOL-030` updates the legacy `fail_unfinished_jobs` behavior to preserve
queued jobs and fail only running jobs that have no typed recovery path. Queued
jobs are accepted work, not evidence of an abandoned worker.

`DJOL-040` extends `JobLeaseClaimFilter` with optional `job_id` matching. This
lets synchronous command paths and supervisor-spawned background paths claim the
exact job they just enqueued instead of racing against another queued job of the
same kind or resource class.

### Runtime Shape

`RuntimeSupervisor` owns process-local tasks, shutdown tokens, abort handles,
and diagnostics. `DurableJobRuntime` starts and completes one durable job. Those
are useful layers, but neither one owns cross-process durability. The lease
contract must live in repository methods, while the supervisor passes the
worker identity, run token, and shutdown/cancel observation points into typed
executors.

After `DJOL-040`, `DurableJobRuntime::run_job` is the first shared leased
runtime path. It exact-claims the queued job, records a stable process-local
worker ID, starts a heartbeat loop, and persists success or failure with the
claim run token. Existing library scan, metadata refresh/maintenance, and NFO
import/export execution paths call this shared runtime, so their durable
lifecycle is now leased even though each domain side effect remains typed.

### Managed Artwork Ingest

Managed Artwork ingest already has a typed claim from queued to fetching and a
typed startup recovery path that fails claimed `fetching`/`validating` work
while leaving queued work alone. This proves the desired atomicity shape, but
the domain status is not a reusable generic job ownership model and does not
yet include cancellation.

## Architecture Direction

Use a fenced lease model:

1. A Taru process creates a stable runtime worker identity at startup.
2. Claiming a job writes `status = running`, a unique run token, the worker
   identity, `started_at`, `heartbeat_at`, and `lease_expires_at`.
3. Heartbeat extends the lease only when `job_id`, `status = running`, and the
   run token match.
4. Success, failure, and acknowledged cancellation clear ownership and are
   conditional on the same run token.
5. Cancel request is durable intent. It can be set on queued or running jobs
   without claiming the job has stopped.
6. Queued jobs with a cancel request can become terminal immediately because no
   worker has side effects in flight.
7. Running jobs become terminal cancelled only after the owning worker observes
   the request at a checkpoint and persists acknowledgement with the run token.
8. Expired running leases are recovered by a job-kind policy. The default can
   fail stale running work with a safe reason; idempotent workers may later
   choose requeue.

This lane should prefer explicit repository methods over a generic "run any
input payload" engine. Shared lifecycle code is valuable; shared side effects
are not.

## Candidate Data Model

Names chosen by `DJOL-020` for the shared core contract:

- `worker_id TEXT`
- `run_token TEXT`
- `lease_expires_at TEXT`
- `heartbeat_at TEXT`
- `cancel_requested_at TEXT`
- `cancel_reason TEXT`
- terminal status: `cancelled`

`worker_id` is diagnostic. `run_token` is the fence. Completion without the
current token must fail or return a stale-owner outcome.

## `DJOL-020` Contract Decision

`taru-core` now owns the vocabulary before SQLite migration work starts:

- `JobStatus::Cancelled` is a terminal status distinct from `failed`.
- `JobWorkerId` identifies a process-local or future durable worker instance.
- `JobRunToken` fences one claim attempt.
- `JobLeaseClaimRequest` carries worker identity, lease duration, and an
  optional kind/resource/library/source filter.
- `JobLeaseRecord` carries `job_id`, `worker_id`, `run_token`,
  `heartbeat_at`, `lease_expires_at`, and cancellation-request facts.
- `JobLeaseGuard` is the minimal token required for heartbeat, completion,
  failure, and cancellation acknowledgement.
- `JobRepository` exposes default-unsupported methods for claim, heartbeat,
  fenced success, fenced failure, cancel request, cancellation acknowledgement,
  and expired-lease recovery.

The default-unsupported repository methods are intentional. They let this task
freeze the cross-crate contract without pretending that existing SQLite rows
already carry lease fields. `DJOL-030` owns the migration and adapter tests.

## Redaction Policy

Admin control/read DTOs may expose:

- job ID, kind, resource class, status;
- whether cancellation was requested;
- lease expiry and heartbeat timestamps if useful for operations;
- safe worker identity if it is a generated non-secret instance ID.

They must not expose:

- raw `input_json`;
- raw `summary_json`;
- raw `error`;
- Source Locators;
- raw provider or addon payload JSON;
- storage URIs, artifact roots, local paths, cache URIs, or content hashes;
- tokens, secrets, request headers, or environment values.

## Closeout Condition

This lane can close when:

- the durable ownership state machine is documented and implemented;
- at least one real job execution path uses fenced claim/heartbeat/completion;
- cancellation request behavior is truthful and tested for queued and running
  jobs, or explicitly split with no public control claim;
- startup recovery is lease-aware;
- evidence gates pass; and
- remaining worker migrations are split into follow-ons.
