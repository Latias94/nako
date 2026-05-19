# Worker Job Cancellation Checkpoints Design

Status: Completed
Last updated: 2026-05-19

## Why This Lane Exists

Taru now has durable job ownership leases and an Admin cancel-request route, but
running job cancellation is still only intent. A running row can show
`cancel_requested_at`, yet the typed executor has no runtime object to observe
that request and no standard way to acknowledge terminal `cancelled`.

That gap matters because Taru has long-running work with side effects:

- Library scan indexes sources, tombstones sources, and probes media.
- Metadata refresh contacts providers and commits canonical metadata.
- Metadata maintenance loops over many items.
- NFO import/export reads and writes sidecar files.
- Future webhook, addon, automation, probe, and transcode workers need the same
  lifecycle vocabulary.

Without this lane, the Admin API can truthfully say "cancel requested", but the
system cannot make progress toward "worker observed and stopped at a safe
checkpoint".

## Relevant Authority

- ADRs:
  - `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
- Workstreams:
  - `docs/workstreams/durable-job-ownership-leases/`
  - `docs/workstreams/job-runtime-worker-control-plane/`
  - `docs/workstreams/managed-artwork-ingest-runtime-controls/`
  - `docs/workstreams/transcode-runtime/`
- Code:
  - `crates/taru-core/src/job.rs`
  - `crates/taru-core/src/repository/jobs.rs`
  - `crates/taru-db/src/jobs.rs`
  - `crates/taru-server/src/app/job_runtime.rs`
  - `crates/taru-server/src/app/jobs.rs`
  - `crates/taru-server/src/app/metadata.rs`
  - `crates/taru-server/src/app/nfo.rs`

## Problem

The current durable state machine is deliberately conservative:

- queued cancellation becomes terminal `cancelled`;
- running cancellation records durable intent and keeps status `running`;
- terminal jobs reject cancellation;
- `cancel_leased_job` exists, but no shared runtime path calls it;
- heartbeats already read the latest lease row, including
  `cancel_requested_at`, but `DurableJobRuntime` discards that signal;
- typed executors receive no context object for checking cancellation.

This creates a product and operations mismatch. Operators can request
cancellation, but a long metadata maintenance job may continue processing every
item because the worker never sees a checkpoint. Worse, if an implementation
blindly converts any `TaruError` into `failed`, then a requested cancellation
would be reported as failure rather than acknowledged operator intent.

## Target State

When this lane closes:

- `DurableJobRuntime` creates a per-run cancellation context after claiming a
  job lease.
- Heartbeat-observed `cancel_requested_at` updates that context.
- Typed worker code can call a small checkpoint API before starting a new
  side-effect unit.
- If a checkpoint observes cancellation, the runtime persists terminal
  `cancelled` with `CancelLeasedJob` and the current run token.
- Cancellation is not reported as a worker failure.
- Side effects that cannot be interrupted mid-flight are documented and tested
  at their next boundary.
- Admin/API docs state the difference between requested and acknowledged
  cancellation.

## In Scope

- Runtime cancellation context and checkpoint API in `taru-server`.
- Fenced cancellation acknowledgement through existing `JobRepository`
  operations.
- Runtime tests for success, failure, and cancellation paths.
- First real worker integration, preferably metadata maintenance because it has
  a natural item loop and a bounded side-effect checkpoint.
- Focused docs updates for Admin job cancellation semantics.
- Follow-on split list for remaining workers.

## Out Of Scope

- Retry/backoff policy after cancellation or failure.
- Generic job requeue policy for expired leases.
- Distributed worker scheduling or lease stealing.
- Cancelling arbitrary in-flight provider HTTP calls, VFS calls, ffprobe
  processes, or filesystem writes unless the owned component already exposes a
  safe cancellation primitive.
- Changing Public Client DTOs.
- Changing playback session cancellation.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Cancellation must be observed at checkpoints, not forced into arbitrary code. | High | ADR 0006 and playback runtime docs distinguish requested vs observed cancellation. | Forceful cancellation would risk partial side effects, stale locks, or misleading status. |
| `DurableJobRuntime` is the right first boundary for shared cancellation. | High | Library scan, metadata, and NFO execution already route through `run_job`. | If some workers bypass it, they need explicit migration tasks rather than hidden behavior. |
| Heartbeat is enough to learn durable cancel intent for the first slice. | Medium | `heartbeat_job_lease` returns `LeasedJob` with cancellation fields. | If heartbeat interval is too slow, add an explicit poll/check method later. |
| Metadata maintenance is the safest first real worker integration. | Medium | It processes a list of items and can stop before the next item. | If tests show setup cost is too high, use a runtime-only proof first and split real-worker integration. |
| Cancellation summaries must stay redacted. | High | Admin job routes already avoid raw input, summary, error, paths, tokens, and storage handles. | Leaking raw summaries would break the existing Admin boundary. |

## Architecture Direction

Use cooperative cancellation. The runtime owns the durable lease and exposes a
small execution context; domain workers decide where it is safe to call it.

Proposed shape:

1. `DurableJobRuntime` exact-claims the job and starts heartbeat as it does
   today.
2. The heartbeat loop writes the latest lease state into a shared cancellation
   context when `cancel_requested_at` is present.
3. `run_job` passes a `DurableJobContext` or `DurableJobCancellation` reference
   to the operation closure.
4. Domain code calls `check_cancelled().await?` before each new logical
   side-effect unit.
5. The check returns a typed cancellation error or outcome that the runtime can
   distinguish from real failure.
6. The runtime stops heartbeat and calls `cancel_leased_job` with the same
   `JobLeaseGuard`.
7. The returned job status is `cancelled`, and no post-success event is emitted.

The important boundary is that cancellation acknowledgement is still a durable
write fenced by `job_id + run_token`. A stale worker may notice a local signal,
but it must not be able to mark a job cancelled after another owner has claimed
the lease.

## Checkpoint Semantics

Checkpoints should be placed before starting a new side-effect unit:

- before processing the next metadata maintenance item;
- before starting a library probe batch;
- before reading/writing the next NFO sidecar;
- before dispatching the next webhook;
- before starting the next addon side effect.

Checkpoints should not claim to roll back work already committed. If a provider
request, VFS operation, or filesystem write is already in flight, the first
slice may finish that operation and stop before the next one.

For NFO library-wide jobs, the app layer can only check before and after the
current `NfoService` call. Per-sidecar checkpoints require a `taru-nfo` service
API that accepts a cancellation boundary before each source read/write; until
that exists, Admin docs must describe NFO cancellation as app-level and
boundary-based.

## Redaction Policy

Admin responses and docs may expose:

- job ID, kind, resource class, status;
- `cancel_requested_at`;
- whether a cancel request is terminal or pending;
- safe lifecycle timestamps.

They must not expose:

- raw `input_json`;
- raw `summary_json`;
- raw `error`;
- provider payloads or addon payloads;
- Source Locators, storage URIs, local paths, cache URIs, artifact roots, or
  content hashes;
- worker run tokens, secrets, headers, or environment values.

## Follow-On Boundaries

Keep these separate unless they become direct blockers:

- automatic retry/backoff policy;
- expired-lease requeue versus fail policy by job kind;
- multi-process worker scheduling and lease stealing;
- process-kill cancellation for child-process-backed jobs;
- full migration of every job kind.

## Closeout Condition

This lane can close when:

- runtime cancellation context and fenced acknowledgement are implemented;
- at least one real typed worker observes cancellation at a safe checkpoint;
- focused tests prove success, failure, queued cancel, running cancel request,
  and acknowledged cancellation remain distinct;
- HTTP/API docs describe requested versus acknowledged cancellation accurately;
- remaining worker migrations are completed or split into follow-ons.

Closeout result: complete. The runtime contract, metadata maintenance,
library scan/probe, and app-level NFO boundaries are implemented and verified.
Per-sidecar NFO cancellation, webhook/addon dispatch checkpoints,
retry/backoff, lease stealing/requeue, and child-process cancellation are
explicit follow-ons.
