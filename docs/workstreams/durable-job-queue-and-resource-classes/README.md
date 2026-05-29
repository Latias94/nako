# Durable Job Queue And Resource Classes

Status: Closed
Last updated: 2026-05-29

## Purpose

This lane deepens Nako's control plane for background work. The current server
already persists jobs, leases running work, records cancellation intent, and
uses feature-local semaphores for scan, metadata, and webhook concurrency. The
remaining architecture gap is that queue pressure, priority, retry/backoff, and
resource class accounting do not share one schedulable boundary.

The first implementation slice is intentionally local to `nako-server`: create
a process-local resource class registry and make existing scan, metadata, and
webhook permit pools come from it. Later slices can map durable job resource
classes onto scheduler decisions without inventing a generic untyped task
runner.

## Goals

- Give runtime resources one named resource class registry with safe
  diagnostics.
- Preserve existing typed job executors and feature-specific transaction
  boundaries.
- Prepare a scheduler boundary for priority, retry, backoff, queue pressure, and
  future job-kind migrations.
- Keep cancellation, lease ownership, and startup recovery aligned with the
  completed durable job lanes.
- Avoid exposing raw job inputs, summaries, local paths, provider payloads,
  sidecar data, source locators, tokens, or secrets in diagnostics.

## Non-Goals

- New `nako-control-plane` crate in the first slice.
- Redis, external queue engines, or distributed worker scheduling.
- Replacing typed workers with a generic JSON task runner.
- Migrating every job kind in one task.
- Playback/HLS runtime changes.
- Addon process lifecycle management.
- Public Client API changes.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)

## Related Work

- [job-runtime-worker-control-plane](../job-runtime-worker-control-plane/README.md)
- [durable-job-ownership-leases](../durable-job-ownership-leases/README.md)
- [worker-job-cancellation-checkpoints](../worker-job-cancellation-checkpoints/README.md)
- [durable-job-recovery](../durable-job-recovery/README.md)
- [server-runtime-deepening](../server-runtime-deepening/README.md)

## Completed Slices

`DJRC-020` added a runtime resource class registry and routes existing
process-local scan, metadata, and webhook permit pools through it while leaving
existing service constructors stable.

`DJRC-030` added explicit durable job class to budget class mapping.

`DJRC-040` completed the first typed scheduler admission tracer bullet for
library scan jobs.

`DJRC-050` added persisted retry/backoff metadata and redacted queue pressure
diagnostics.

`DJRC-060` closed the lane after review and verification.

## Closeout

This lane shipped the process-local resource registry, explicit durable job to
runtime budget mapping, the first typed scheduler admission path for library
scan jobs, persisted retry/backoff rows, and redacted queue pressure
diagnostics.

Priority policy, distributed scheduling, remote workers, addon process
lifecycle, child-process cancellation, and broader job-kind scheduler migration
remain follow-on lanes. The next highest-leverage split is
`proposed:durable-job-priority-policy-and-scheduler-migration`.
