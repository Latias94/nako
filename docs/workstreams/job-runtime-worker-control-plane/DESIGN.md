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

## Open Design Questions

| Question | Initial Direction | Why It Matters |
| --- | --- | --- |
| Lease model | Add process-local owner/heartbeat fields before broad worker adoption. | Existing `running` jobs can be stale after process death. |
| Retry shape | Keep existing Managed Artwork requeue-in-place for now; design new-attempt retry as a later policy. | ADR 0006 prefers new rows for generic retry, but Managed Artwork has a proven in-place requeue contract. |
| Cancellation | Introduce cancellation request semantics only after worker ownership exists. | Cancellation without a registered runner is only state decoration. |
| Resource budgets | Declare resource class permits in runtime config and supervisor. | Artwork fetch, metadata network, webhook, CPU/GPU work need separate limits. |
| Admin controls | Return redacted summaries with presence flags, not raw inputs/errors. | Job input and summary JSON can contain private operational data. |

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
