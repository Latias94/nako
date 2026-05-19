# NFO Sidecar Cancellation Checkpoints

Status: Complete
Last updated: 2026-05-19

## Purpose

This lane closes the NFO gap left by
`worker-job-cancellation-checkpoints`: NFO import/export jobs now use the
context-aware durable runtime at the app boundary, but `taru-nfo` still loops
over all media sources without a per-sidecar cancellation checkpoint.

The goal is to let NFO library import/export stop before the next sidecar
read/write when the owning durable worker observes an Admin cancel request,
without making `taru-nfo` depend on `taru-server` runtime types.

## Goals

- Add a crate-local cancellation/checkpoint contract in `taru-nfo`.
- Check cancellation before each sidecar read/write unit in import and export.
- Keep existing no-cancellation `import_library` and `export_library` APIs
  source-compatible through no-op defaults.
- Map `taru-server` durable cancellation into the new NFO checkpoint API.
- Prove cancelled NFO jobs persist terminal `cancelled` and skip success
  outbox publication.
- Keep sidecar paths, XML payloads, storage URIs, and source locators out of
  public/Admin cancellation responses.

## Non-Goals

- Retry/backoff policy.
- Lease stealing, expired-lease requeue, or distributed scheduling.
- Child-process cancellation.
- Changing Public Client API shapes.
- Changing NFO XML preservation, backup retention, or storage write policy.
- Forcefully interrupting an already-started storage read/write operation.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)

## Related Work

- [worker-job-cancellation-checkpoints](../worker-job-cancellation-checkpoints/README.md)
- [durable-job-ownership-leases](../durable-job-ownership-leases/README.md)
- [nfo-storage-write-policy](../nfo-storage-write-policy/README.md)
- [nfo-sidecar-backup-policy](../nfo-sidecar-backup-policy/README.md)
- [nfo-round-trip-preservation](../nfo-round-trip-preservation/README.md)

## Outcome

The lane shipped a server-independent `taru-nfo` sidecar checkpoint contract,
checkpoint-aware import/export library variants, no-op compatibility wrappers,
and durable server import/export cancellation mapping. Cancelled NFO jobs now
stop before the next sidecar unit, persist terminal `cancelled`, and skip
success outbox publication without exposing sidecar paths, XML payloads,
storage URIs, source locators, or local filesystem paths in cancellation
responses.
