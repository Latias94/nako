# Worker Job Cancellation Checkpoints

Status: Completed
Last updated: 2026-05-19

## Purpose

This lane turns durable job cancellation from an Admin request into an
execution-time contract that typed workers can observe and acknowledge.

`durable-job-ownership-leases` added the durable truth: queued jobs can be
cancelled immediately, running jobs can receive a cancel request, and only the
owning run token may acknowledge terminal `cancelled`. This lane owns the next
step: passing that cancel intent through `DurableJobRuntime` into real worker
checkpoints without claiming process-kill or rollback semantics Nako does not
have.

## Goals

- Add a runtime cancellation context that is backed by the leased job heartbeat
  state.
- Make cancellation acknowledgement fenced on the current `JobRunToken`.
- Teach at least one real long-running worker to check cancellation before
  starting a new side-effect step.
- Keep Admin/Public API responses redacted and truthful.
- Record where cancellation cannot interrupt an in-flight provider, VFS, probe,
  or filesystem write.

## Closed Result

This lane closed after adding context-aware durable cancellation checkpoints to
`DurableJobRuntime`, metadata maintenance, library scan/probe, and NFO
import/export app boundaries. Remaining work is split by boundary type:
per-sidecar NFO checkpoints need a `nako-nfo` service API; webhook/addon
dispatch, retry/backoff, expired-lease requeue/stealing, and child-process
cancellation remain follow-ons.

## Non-Goals

- Automatic retry/backoff scheduling.
- Distributed scheduling, lease stealing, or multi-process worker balancing.
- Force-killing arbitrary Rust futures or child processes for durable jobs.
- Rewriting all job kinds in one slice.
- Playback/transcode session cancellation changes.
- Public Client API changes.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)

## Related Work

- [durable-job-ownership-leases](../durable-job-ownership-leases/README.md)
- [job-runtime-worker-control-plane](../job-runtime-worker-control-plane/README.md)
- [managed-artwork-ingest-runtime-controls](../managed-artwork-ingest-runtime-controls/README.md)

## Closeout

See [HANDOFF.md](HANDOFF.md) for residual risks and follow-ons, and
[EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md) for the final verification
evidence.
