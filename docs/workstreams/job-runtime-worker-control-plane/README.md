# Job Runtime Worker Control Plane

Status: Completed
Last updated: 2026-05-19

## Purpose

Taru has durable `jobs` rows, explicit job inputs, Admin job diagnostics, and
feature-specific execution paths. Managed Artwork ingest now has manual
`process-next` and requeue controls, but the runtime ownership remains too
feature-local: claim, execute, retry, cancellation, and worker supervision are
not yet a shared control plane.

This lane turned the existing ADR direction into the first implementation path:
a bounded Managed Artwork ingest worker and typed startup recovery boundary.
Broader durable cancellation, generic leases, and migration of other job kinds
are intentionally split into follow-ons.

## Goals

- Define a shared durable job worker/control-plane boundary aligned with ADR
  0006 and ADR 0019.
- Keep per-job-kind execution typed and explicit; avoid a generic untyped task
  runner.
- Make claim/lease/recovery/requeue/cancel semantics observable and testable.
- Use Managed Artwork ingest as the first vertical slice because it already has
  queued/running/succeeded/failed state, safe summaries, and requeue coverage.
- Keep Admin responses redacted: no raw job inputs, source URLs, storage URIs,
  local paths, payload JSON, secret values, or raw errors.

## Non-Goals

- Distributed multi-process scheduling.
- A full cron scheduler or automatic retry policy for every job kind in the
  first slice.
- Migrating all current job kinds at once.
- Removing feature-specific domain commit methods where they encode the correct
  transaction boundary.
- Exposing raw `input_json`, `summary_json`, errors, source locators, artifact
  storage, cache URIs, local paths, or token material through Admin APIs.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Completed Slice

The shipped slice proves one opt-in supervised worker loop can claim and process
Managed Artwork ingest jobs through the existing safe artifact pipeline without
changing the Public Client surface. Startup recovery is typed: queued Managed
Artwork work remains queued, while claimed `fetching`/`validating` work is
failed with safe `startup_recovery` and remains requeueable.

## Split Follow-Ons

- Durable cancellation and ownership leases.
- Migration of metadata, webhook, NFO, automation, scan/probe, and cleanup
  workloads onto shared worker contracts.
- Generic retry/backoff policy.
