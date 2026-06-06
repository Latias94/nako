# Refresh Storage Source Hash Architecture Follow-ons Evidence

## Implementation Summary

- Updated `docs/architecture/LANES.md` to list read-only VFS cache remediation
  planning, source hash Admin overview/Jobs diagnostics, Admin manual enqueue,
  and source-hash retry/requeue as shipped storage-vfs work.
- Updated `docs/architecture/CONTROL_PLANE.md` to move source hash Admin
  manual commands and diagnostics out of generic follow-ons.
- Updated `docs/architecture/STORAGE_VFS.md` to point VFS cache repair task
  evidence at archived task paths and keep durable remediation workers as the
  remaining VFS cache follow-on.
- Narrowed remaining source hash follow-ons to scan-originated triggering,
  automatic Source Duplicate Relationship reconciliation, broader scheduler
  migration, and PostgreSQL runtime harness work.

## Validation

- `git diff --check -- docs/architecture/LANES.md docs/architecture/CONTROL_PLANE.md docs/architecture/STORAGE_VFS.md .trellis/tasks/06-06-refresh-storage-source-hash-architecture-follow-ons`
  passed with only Git LF/CRLF working-copy warnings.
- `rg -n "operator/Admin diagnostics|operator/API surfaces|scan/operator/API triggering|redaction-safe Admin diagnostics for persisted evidence|\\.trellis/tasks/06-06-vfs-cache|\\.trellis/tasks/06-05-vfs-cache|\\.trellis/tasks/06-04-vfs-cache|\\.trellis/tasks/06-04-06-04-vfs-cache" docs/architecture/CONTROL_PLANE.md docs/architecture/STORAGE_VFS.md docs/architecture/LANES.md`
  found no stale matches.

## Boundary

- No Rust, TypeScript, generated contract, schema, API route, config, or
  runtime behavior changed.
- Existing unrelated Trellis archive/delete changes in the working tree were
  not modified or staged for this task.
