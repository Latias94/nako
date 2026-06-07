# Evidence: VFS Repair Diagnostics Architecture Reconciliation

## Changes

- Updated `docs/architecture/STORAGE_VFS.md` to list VFS cache repair Admin
  Jobs diagnostics projection as shipped.
- Updated the VFS cache follow-on wording to keep cache purge/delete,
  invalidation, backend configuration mutation, library file writes, automated
  repair policy, and realtime/incident diagnostics as separate future work.
- Updated `docs/architecture/CONTROL_PLANE.md` to list VFS repair job
  diagnostics projection as shipped under durable jobs and the VFS repair
  control-plane lane.
- Updated `docs/architecture/WORKSTREAM_LINKS.md` to include the archived
  durable repair and diagnostics tasks as evidence, and replaced the stale
  proposed diagnostics slug with
  `proposed:vfs-cache-repair-automation-and-mutation-policy`.

## Verification

- `rg -n "broader operator diagnostics|proposed:vfs-cache-repair-diagnostics|\\.trellis/tasks/06-06-06-06-overnight|\\.trellis/tasks/06-06-scan-originated" docs/architecture`
  found no matches.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-07-06-07-vfs-repair-diagnostics-architecture-reconciliation`
  passed.
- `git diff --check` passed.

## Scope Notes

- No Rust source, generated Admin contract, runtime behavior, route behavior,
  scheduler behavior, storage mutation behavior, or Admin Web UI changed.
- The shipped diagnostics claim is limited to the existing Admin Jobs
  redaction-safe projection for `JobKind::VfsCacheRepair`.
