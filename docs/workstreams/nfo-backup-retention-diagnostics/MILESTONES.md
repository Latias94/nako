# NFO Backup Retention And Diagnostics Milestones

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs define retention semantics, admin/public boundaries, and
  validation gates.
- `docs/GOALS.md` names M50 as the implementation goal while the lane is
  active.
- The design keeps XML codec, storage policy, and API adapter responsibilities
  separate.

## M1 - VFS Retention

Exit criteria:

- Storage backup request/report types can express keep-latest retention.
- `LocalFsBackend` prunes only Taru-created backups for the same sidecar.
- Pruning keeps the newest configured backups and preserves unrelated files.
- Pruning failures are explicit diagnostics.

Status: completed. `StorageBackupPolicy` carries retention, local pruning is
same-sidecar and Taru-prefix constrained, and tests cover success,
zero-retention, unrelated-file preservation, and prune failure reporting.

## M2 - NFO Diagnostics

Exit criteria:

- NFO forced export requests retention when it requests backup.
- Export summaries report backup creation, pruning counts, and pruning
  failures.
- Backup/pruning failure behavior is covered by service tests.

Status: completed. Forced NFO export requests retention when backing up an
existing sidecar and records backup/pruning diagnostics in `NfoExportSummary`.

## M3 - Admin/Public Boundary

Exit criteria:

- Admin-facing diagnostics are inspectable through existing job summary or
  admin-only DTOs.
- `taru-client-protocol` remains unchanged.
- Public client route inventory remains clean if API modules are touched.

Status: completed. Existing admin job summaries preserve NFO retention
diagnostics, public OpenAPI inventory remains clean, and `taru-client-protocol`
has no diff.

## M4 - Closeout

Exit criteria:

- Focused `taru-vfs` and `taru-nfo` checks and tests pass.
- Workspace checks and nextest pass.
- `git diff --check` has no whitespace errors.
- `docs/GOALS.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` record M50
  completion evidence and recommended follow-ons.

Status: completed. Focused gates, workspace check, workspace nextest, and diff
check passed.
