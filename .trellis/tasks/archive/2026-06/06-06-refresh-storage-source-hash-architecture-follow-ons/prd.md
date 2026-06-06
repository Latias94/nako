# Refresh Storage Source Hash Architecture Follow-ons

## Goal

Refresh architecture-map follow-on language after the recent source fingerprint
hash and VFS cache repair slices, so lane planning no longer suggests already
shipped Admin diagnostics or read-only remediation planning as future work.

## Requirements

- Update `docs/architecture/LANES.md` storage-vfs queue text to distinguish:
  - shipped source fingerprint hash overview/jobs diagnostics,
  - shipped Admin manual enqueue and retry/requeue commands,
  - remaining scan-originated triggering and automatic reconciliation policy,
  - shipped read-only VFS cache remediation plan,
  - remaining durable VFS cache remediation/repair worker work.
- Update `docs/architecture/CONTROL_PLANE.md` source fingerprint hash follow-ons
  so Admin read diagnostics, manual enqueue, and retry/requeue are not
  described as missing.
- Verify `docs/architecture/STORAGE_VFS.md` no longer points completed VFS cache
  tasks at active task paths or describes the shipped read-only remediation plan
  as a future lane.
- Keep this task docs-only. Do not change Rust, TypeScript, generated
  contracts, schemas, API routes, config, or runtime behavior.
- Preserve existing terminology from `CONTEXT.md`: Source Fingerprint evidence,
  Source Duplicate Relationship, VFS cache repair, Admin API, Public Client API,
  and Durable Job.
- Do not touch unrelated Trellis archive/delete changes already present in the
  working tree.

## Acceptance Criteria

- `LANES.md` storage-vfs active queue and lane notes name the correct remaining
  source fingerprint hash work: scan-originated triggering, automatic
  reconciliation policy, broader scheduler migration, and PostgreSQL runtime
  harness.
- `LANES.md` does not list source fingerprint hash operator/Admin read
  diagnostics, manual enqueue, or retry/requeue as candidate next actions.
- `CONTROL_PLANE.md` durable-job/source-hash follow-ons distinguish shipped
  Admin read diagnostics/manual commands from still-missing scan policy and
  automatic reconciliation.
- `STORAGE_VFS.md` references the archived VFS cache repair task path and keeps
  durable remediation queues/workers as the follow-on after the shipped
  read-only remediation plan.
- `git diff --check` passes for the files touched by this task.

## Out Of Scope

- No Public Client playback capability parity implementation.
- No source fingerprint hash triggering implementation.
- No Source Duplicate Relationship mutation or reconciliation implementation.
- No Admin route, DTO, or generated Admin contract changes.
- No Trellis cleanup for unrelated archived tasks.

## Evidence

- Parent synthesis:
  `.trellis/tasks/archive/2026-06/06-05-06-05-cross-lane-architecture-audit/research/next-lane-synthesis.md`.
- Storage/control-plane audit:
  `.trellis/tasks/archive/2026-06/06-05-06-05-cross-lane-architecture-audit/research/storage-library-control-plane.md`.
- Completed VFS remediation plan commit:
  `bf56c38c feat(storage): add vfs cache remediation plan`.
- Completed Admin source hash trigger/retry evidence:
  `.trellis/tasks/archive/2026-06/06-06-admin-source-fingerprint-hash-trigger-first-slice/`
  and
  `.trellis/tasks/archive/2026-06/06-06-source-hash-retry-requeue-admin-command/`.

## Validation

- `git diff --check -- docs/architecture/LANES.md docs/architecture/CONTROL_PLANE.md docs/architecture/STORAGE_VFS.md .trellis/tasks/06-06-refresh-storage-source-hash-architecture-follow-ons`
- Manual diff review against the acceptance criteria.
